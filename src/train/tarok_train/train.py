"""The training loop of spec 007.

Run (cmd, from the repo root, venv active):

    python -m tarok_train.train --config src\\train\\configs\\mock.toml

One update:
1. Collect. The tables play until `seat_deals_per_update` seat-deals have
   finished that were played from their first decision under the current
   weights. A deal that was already running when the last update happened is
   played to its end but not stored.
2. Train. PPO on the finished seat-deals. Rows of deals still running are
   dropped: after the update they would come from old weights.

So every decision in an update was taken by the weights that the update starts
from ("rollout/decision_age_max" is 0). The price is the decisions that are
played but not trained on ("rollout/trained_share").

Checkpoints hold the config, the encoding version, the step counts, the weights,
the optimiser, the pool of past checkpoints, the random generators and the
environment's state. `--resume` goes on from one; on the CPU the resumed run
gives the same numbers as an unbroken one. TensorBoard files go next to the
checkpoints.

Every hyperparameter comes from the config file. `validate_config` refuses a
config with a missing or an unknown key, so nothing falls back to a default.
"""

from __future__ import annotations

import argparse
import copy
import datetime
import statistics
import sys
import time
import tomllib
from pathlib import Path

import numpy as np
import torch

from . import contract as C
from . import mock_env, opponents, ppo
from .bench_model import gpu_snapshot
from .model import Actor, build_critic, build_network, count_parameters
from .rollout import Rollout

TRAIN_DIR = Path(__file__).resolve().parents[1]
DEFAULT_CONFIG = TRAIN_DIR / "configs" / "mock.toml"

# section -> key -> type. A float key also takes an integer.
SCHEMA = {
    "run": {
        "name": str,
        "seed": int,
        "network": str,
        "updates": int,
        "device": str,
        "checkpoint_every": int,
        "out_dir": str,
    },
    "env": {
        "kind": str,
        "mode": str,
        "tables": int,
        "min_rounds": int,
        "max_rounds": int,
        "legal_per_decision": int,
        "extra_token_probability": float,
        "feature_density": float,
    },
    "model": {
        "width": int,
        "layers": int,
        "heads": int,
        "feed_forward": int,
        "mlp_hidden": list,
        "critic_hidden": list,
        "policy_init_scale": float,
        "cache_initial_tokens": int,
        "act_bucket_tokens": int,
    },
    "ppo": {
        "seat_deals_per_update": int,
        "clip_range": float,
        "epochs": int,
        "gamma": float,
        "gae_lambda": float,
        "learning_rate": float,
        "adam_beta1": float,
        "adam_beta2": float,
        "adam_eps": float,
        "update_batch": int,
        "entropy_weight": float,
        "critic_weight": float,
        "value_head_weight": float,
        "q_weight": float,
        "partner_weight": float,
        "card_location_weight": float,
        "value_loss": str,
        "huber_delta": float,
        "gradient_clip": float,
        "normalise_advantage": bool,
    },
    "opponents": {
        "past_share": float,
        "max_frozen_seats": int,
        "pool_size": int,
        "snapshot_every": int,
    },
}


class ConfigError(ValueError):
    """The config file lacks a key, has an unknown key, or a value that cannot be used."""


class EncodingVersionError(RuntimeError):
    """A checkpoint was made for another version of docs/encoding.md."""


def validate_config(cfg: dict) -> None:
    for section, keys in SCHEMA.items():
        if section not in cfg:
            raise ConfigError(f"the config has no [{section}] table")
        for key, kind in keys.items():
            if key not in cfg[section]:
                raise ConfigError(f"the config has no value for {key} in [{section}]; the code has no default")
            value = cfg[section][key]
            ok = isinstance(value, (int, float)) if kind is float else isinstance(value, kind)
            if not ok or (kind is not bool and isinstance(value, bool)):
                raise ConfigError(f"{key} in [{section}] must be of type {kind.__name__}, got {value!r}")
        for key in cfg[section]:
            if key not in keys:
                raise ConfigError(f"unknown key {key} in [{section}]")
    for section in cfg:
        if section not in SCHEMA:
            raise ConfigError(f"unknown table [{section}]")
    if not 1 <= cfg["opponents"]["max_frozen_seats"] <= 3:
        raise ConfigError("max_frozen_seats in [opponents] must be 1 to 3: one seat of a deal is always trained")
    if cfg["ppo"]["value_loss"] not in ("huber", "mse"):
        raise ConfigError("value_loss in [ppo] must be 'huber' or 'mse'")
    if cfg["ppo"]["seat_deals_per_update"] < 1:
        raise ConfigError("seat_deals_per_update in [ppo] must be at least 1")


def load_config(path) -> dict:
    with open(path, "rb") as handle:
        cfg = tomllib.load(handle)
    validate_config(cfg)
    return cfg


class Trainer:
    def __init__(self, cfg: dict, device, log_dir=None):
        validate_config(cfg)
        self.cfg = copy.deepcopy(cfg)
        run, cfg_model, cfg_ppo, cfg_opp = (self.cfg[k] for k in ("run", "model", "ppo", "opponents"))
        self.device = torch.device(device)
        self.kind = run["network"]
        seed = run["seed"]
        torch.manual_seed(seed)
        self.rng = np.random.default_rng(seed)  # minibatch order, frozen seats, opponent choice
        self.generator = torch.Generator(device=self.device).manual_seed(seed)  # action sampling

        self.net = build_network(cfg_model, self.kind).to(self.device)
        self.critic = build_critic(cfg_model).to(self.device)
        self.frozen_net = build_network(cfg_model, self.kind).to(self.device).eval()
        for weight in self.frozen_net.parameters():
            weight.requires_grad_(False)
        self.optimizer = torch.optim.Adam(
            [*self.net.parameters(), *self.critic.parameters()],
            lr=cfg_ppo["learning_rate"],
            betas=(cfg_ppo["adam_beta1"], cfg_ppo["adam_beta2"]),
            eps=cfg_ppo["adam_eps"],
        )

        self.env = mock_env.make(self.cfg["env"], seed)
        self.n = self.cfg["env"]["tables"]
        self.actor = Actor(self.net, self.frozen_net, self.n, self.device, cfg_model)
        self.pool = opponents.OpponentPool(cfg_opp["pool_size"])
        self.frozen = np.zeros((self.n, 4), dtype=bool)  # seats played by the past checkpoint, per table
        self.frozen_active = False  # True once a past checkpoint is loaded
        self.deal_started = np.ones(self.n, dtype=bool)  # no decision taken yet in the table's deal
        # A table is fresh when its current deal began under the current weights. Only fresh
        # tables are stored. After an update every table that is inside a deal is stale.
        self.fresh = np.ones(self.n, dtype=bool)
        self.rollout = Rollout()

        self.updates = 0
        self.decisions = 0  # decisions of trained seats (stored or not)
        self.frozen_decisions = 0  # decisions of seats played by the past checkpoint
        self.collected = 0  # finished seat-deals stored since the last update
        self.acted = 0  # decisions of trained seats since the last update
        self.history = []
        self.parameter_counts = {self.kind: count_parameters(self.net), "critic": count_parameters(self.critic)}
        print(
            f"parameters: {self.kind} {self.parameter_counts[self.kind]:,}, "
            f"critic {self.parameter_counts['critic']:,}"
        )
        self.writer = None
        if log_dir is not None:
            from torch.utils.tensorboard import SummaryWriter

            self.writer = SummaryWriter(str(log_dir))
            for name, count in self.parameter_counts.items():
                self.writer.add_scalar(f"parameters/{name}", count, 0)

    # ---- acting ----

    def use_frozen_seats(self) -> None:
        """Load a past checkpoint from the pool into the frozen network. New deals may get frozen seats."""
        self.pool.load_one(self.frozen_net, self.rng)
        self.frozen_active = True
        self.actor.reset()
        self._assign_frozen(np.flatnonzero(self.deal_started))

    def _assign_frozen(self, tables: np.ndarray) -> None:
        """Draw the frozen seats of the deals that start now. A deal keeps them to its end."""
        if not self.frozen_active or len(tables) == 0:
            return
        cfg_opp = self.cfg["opponents"]
        self.frozen[tables] = opponents.assign_frozen_seats(
            self.rng, len(tables), cfg_opp["past_share"], cfg_opp["max_frozen_seats"]
        )

    def step(self) -> None:
        """One environment step: one decision per table."""
        obs = self.env.observe()
        # Copies of what is needed after env.step: an environment may reuse its buffers.
        seat = obs["seat"].copy()
        deal = obs["deal"].copy()
        n_tokens = obs["n_tokens"].copy()
        rows = np.arange(self.n)
        frozen_rows = self.frozen[rows, seat]
        device = self.device
        # The actor's inputs: tokens, state and mask. The stored rows hold the same arrays.
        action, logp, _ = self.actor.act(
            rows,
            seat,
            deal,
            n_tokens,
            frozen_rows,
            torch.from_numpy(obs["tokens"]).to(device).long(),
            torch.from_numpy(obs["state"]).to(device),
            torch.from_numpy(obs["mask"]).to(device),
            self.generator,
        )
        action = action.cpu().numpy()
        keep = ~frozen_rows & self.fresh
        if keep.any():
            with torch.inference_mode():
                value = self.critic(torch.from_numpy(obs["critic"][keep]).to(device)).cpu().numpy()
            self.rollout.add(
                tokens=obs["tokens"][keep],
                tables=rows[keep],
                seat=seat[keep],
                deal=deal[keep],
                n_tokens=n_tokens[keep],
                state=obs["state"][keep],
                mask=obs["mask"][keep],
                critic_extra=obs["critic"][keep, C.STATE_FEATURES :],
                partner_target=obs["partner_target"][keep],
                card_target=obs["card_target"][keep],
                action=action[keep],
                logp=logp.cpu().numpy()[keep],
                value=value,
                version=np.full(int(keep.sum()), self.updates),
            )
        trained = int((~frozen_rows).sum())
        self.decisions += trained
        self.acted += trained
        self.frozen_decisions += int(frozen_rows.sum())

        done, reward, parts = self.env.step(action)
        ended = np.flatnonzero(done)
        if len(ended):
            stored = ended[self.fresh[ended]]
            if len(stored):
                self.rollout.finish(stored, deal[stored], reward[stored], parts[stored])
                self.collected += int((~self.frozen[stored]).sum())
            self.frozen[ended] = False
            self.fresh[ended] = True  # the next deal of these tables begins under the current weights
            self._assign_frozen(ended)
        self.deal_started = done.copy()

    def collect(self) -> None:
        """Play until enough stored seat-deals have finished."""
        self.collected = 0
        self.acted = 0
        while self.collected < self.cfg["ppo"]["seat_deals_per_update"]:
            self.step()

    # ---- learning ----

    def update(self) -> dict | None:
        """PPO on the stored deals that finished. None if there is none."""
        deals = self.rollout.take()
        self.rollout.discard()  # rows of deals still running would be stale after this update
        self.fresh = self.deal_started.copy()  # only tables that stand at a deal's first decision
        if deals is None:
            return None
        cfg_ppo = self.cfg["ppo"]
        ppo.prepare(deals, cfg_ppo)
        stats = ppo.rollout_stats(deals, self.updates)
        stats["rollout/trained_share"] = deals.decisions / max(self.acted, 1)
        if getattr(self.env, "reward_is_score", False):
            stats["task/score"] = float(deals.reward.mean())
        stats.update(ppo.update(self.net, self.critic, self.optimizer, deals, cfg_ppo, self.device, self.rng))
        self.net.eval()
        self.critic.eval()
        self.actor.reset()  # the weights changed: the cached keys and values are stale
        self.updates += 1
        if self.updates % self.cfg["opponents"]["snapshot_every"] == 0:
            self.pool.add(self.net)
        if len(self.pool):
            self.use_frozen_seats()
        return stats

    def run(self, updates: int, checkpoint_dir=None, report_every: int = 0) -> list:
        """Run `updates` updates. Returns their statistics; they are also appended to `history`."""
        self.net.eval()
        self.critic.eval()
        done = []
        for _ in range(updates):
            started = time.perf_counter()
            before = self.decisions + self.frozen_decisions
            stats = None
            acting = 0.0
            while stats is None:
                clock = time.perf_counter()
                self.collect()
                self._synchronise()
                acting += time.perf_counter() - clock
                stats = self.update()
            self._synchronise()
            seconds = time.perf_counter() - started
            stats["speed/steps_per_second"] = (self.decisions + self.frozen_decisions - before) / seconds
            stats["speed/acting_share"] = acting / seconds  # the rest is the update
            stats["opponents/frozen_share"] = self.frozen_decisions / max(self.decisions + self.frozen_decisions, 1)
            self.history.append(stats)
            done.append(stats)
            if self.writer is not None:
                for name, value in stats.items():
                    self.writer.add_scalar(name, value, self.updates)
            if report_every and (self.updates % report_every == 0 or self.updates == 1):
                score = f"score {stats['task/score']:.3f}  " if "task/score" in stats else ""
                print(
                    f"update {self.updates:>5}  {score}loss {stats['loss/total']:.4f}  "
                    f"entropy {stats['entropy/mean']:.3f}  kl {stats['ppo/approx_kl']:.4f}  "
                    f"trained {stats['rollout/trained_share']:.0%}  "
                    f"{stats['speed/steps_per_second']:,.0f} steps/s",
                    flush=True,
                )
            every = self.cfg["run"]["checkpoint_every"]
            if checkpoint_dir is not None and self.updates % every == 0:
                self.save(Path(checkpoint_dir) / f"checkpoint-{self.updates:06d}.pt")
        return done

    def _synchronise(self) -> None:
        if self.device.type == "cuda":
            torch.cuda.synchronize()

    # ---- checkpoints ----

    def save(self, path) -> None:
        """Write a checkpoint. Call it between updates: rows of a running collection are not saved."""
        path = Path(path)
        path.parent.mkdir(parents=True, exist_ok=True)
        environment = {
            name: torch.from_numpy(value) if isinstance(value, np.ndarray) else value
            for name, value in self.env.get_state().items()
        }
        torch.save(
            {
                "encoding_version": C.ENCODING_VERSION,
                "config": self.cfg,
                "network": self.kind,
                "updates": self.updates,
                "decisions": self.decisions,
                "frozen_decisions": self.frozen_decisions,
                "model": self.net.state_dict(),
                "critic": self.critic.state_dict(),
                "optimizer": self.optimizer.state_dict(),
                # What a resumed run needs to go on exactly as an unbroken one would.
                "pool": self.pool.snapshots,
                "frozen_net": self.frozen_net.state_dict(),
                "frozen_active": self.frozen_active,
                "frozen": torch.from_numpy(self.frozen),
                "deal_started": torch.from_numpy(self.deal_started),
                "fresh": torch.from_numpy(self.fresh),
                "rng": _plain(self.rng.bit_generator.state),
                "generator": self.generator.get_state(),
                "environment": _plain(environment),
            },
            path,
        )

    @classmethod
    def load(cls, path, device, log_dir=None) -> "Trainer":
        """A trainer that goes on from a checkpoint."""
        data = torch.load(path, map_location=device, weights_only=True)
        version = data.get("encoding_version")
        if version != C.ENCODING_VERSION:
            raise EncodingVersionError(
                f"the checkpoint was made for encoding version {version}; this code reads "
                f"encoding version {C.ENCODING_VERSION} (docs/encoding.md). It cannot be loaded."
            )
        trainer = cls(data["config"], device, log_dir)
        trainer.net.load_state_dict(data["model"])
        trainer.critic.load_state_dict(data["critic"])
        trainer.optimizer.load_state_dict(data["optimizer"])
        trainer.updates = data["updates"]
        trainer.decisions = data["decisions"]
        trainer.frozen_decisions = data["frozen_decisions"]
        trainer.pool.snapshots = [{k: v.cpu() for k, v in snapshot.items()} for snapshot in data["pool"]]
        trainer.frozen_net.load_state_dict(data["frozen_net"])
        trainer.frozen_active = data["frozen_active"]
        trainer.frozen = data["frozen"].cpu().numpy().copy()
        trainer.deal_started = data["deal_started"].cpu().numpy().copy()
        trainer.fresh = data["fresh"].cpu().numpy().copy()
        trainer.rng.bit_generator.state = _unplain(data["rng"])
        trainer.generator.set_state(data["generator"].cpu())
        trainer.env.set_state(
            {
                name: value.cpu().numpy() if isinstance(value, torch.Tensor) else value
                for name, value in _unplain(data["environment"]).items()
            }
        )
        return trainer

    def close(self) -> None:
        if self.writer is not None:
            self.writer.close()
            self.writer = None


_BIG = "int:"


def _plain(value):
    """Integers wider than 62 bits (the state of a numpy generator) as text, so any loader accepts them."""
    if isinstance(value, dict):
        return {key: _plain(item) for key, item in value.items()}
    if isinstance(value, int) and not isinstance(value, bool) and value.bit_length() > 62:
        return f"{_BIG}{value}"
    return value


def _unplain(value):
    if isinstance(value, dict):
        return {key: _unplain(item) for key, item in value.items()}
    if isinstance(value, str) and value.startswith(_BIG):
        return int(value[len(_BIG) :])
    return value


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description="Train on the environment named in a run config (spec 007).")
    parser.add_argument("--config", default=str(DEFAULT_CONFIG), help="run config (TOML)")
    parser.add_argument("--resume", default=None, help="a checkpoint to go on from; its config is used")
    parser.add_argument("--network", default=None, help="transformer or mlp; overrides [run] network")
    parser.add_argument("--updates", type=int, default=None, help="overrides [run] updates (the total, also on resume)")
    parser.add_argument("--tables", type=int, default=None, help="overrides [env] tables")
    parser.add_argument("--mode", default=None, help="mock mode: learnable, tokens or random; overrides [env] mode")
    parser.add_argument("--seat-deals", type=int, default=None, help="overrides [ppo] seat_deals_per_update")
    parser.add_argument("--device", default=None, help="cuda or cpu; overrides [run] device")
    parser.add_argument("--out-dir", default=None, help="folder for checkpoints and logs; overrides [run] out_dir")
    args = parser.parse_args(argv)

    resume = None
    if args.resume is not None:
        for name in ("network", "tables", "mode", "seat_deals"):
            if getattr(args, name) is not None:
                parser.error(f"--{name.replace('_', '-')} cannot change on --resume: the checkpoint's config is used")
        resume = Path(args.resume)
        cfg = torch.load(resume, map_location="cpu", weights_only=True)["config"]
    else:
        cfg = load_config(args.config)
    # Overrides go into the config, so the checkpoint records what really ran.
    for section, key, value in (
        ("run", "network", args.network),
        ("env", "tables", args.tables),
        ("env", "mode", args.mode),
        ("ppo", "seat_deals_per_update", args.seat_deals),
        ("run", "device", args.device),
    ):
        if value is not None:
            cfg[section][key] = value
    total = args.updates if args.updates is not None else cfg["run"]["updates"]
    device = cfg["run"]["device"]
    if device == "auto":
        device = "cuda" if torch.cuda.is_available() else "cpu"

    name = f"{cfg['run']['name']}-{cfg['run']['network']}"
    if resume is not None:
        out = Path(args.out_dir) if args.out_dir is not None else resume.parent
    elif args.out_dir is not None:
        out = Path(args.out_dir) / name
    else:
        stamp = datetime.datetime.now().strftime("%Y%m%d-%H%M%S")
        out = TRAIN_DIR / cfg["run"]["out_dir"] / f"{name}-{stamp}"

    print(f"run {name}: {total} updates on {device}, {cfg['env']['tables']} tables, mode {cfg['env']['mode']}")
    if device == "cuda":
        print(f"gpu: {torch.cuda.get_device_name(0)}; load before the run: {gpu_snapshot()}")
    if resume is not None:
        trainer = Trainer.load(resume, device, log_dir=out)
        print(f"resumed from {resume.name} at update {trainer.updates}")
    else:
        cfg["run"]["updates"] = total
        trainer = Trainer(cfg, device, log_dir=out)
    remaining = max(total - trainer.updates, 0)
    started = time.perf_counter()
    first = trainer.decisions + trainer.frozen_decisions
    history = trainer.run(remaining, checkpoint_dir=out, report_every=max(1, total // 20))
    seconds = time.perf_counter() - started
    final = out / f"checkpoint-{trainer.updates:06d}.pt"
    if not final.exists():
        trainer.save(final)
    trainer.close()
    if not history:
        print("nothing to do: the checkpoint already has that many updates")
        return 0

    speeds = [h["speed/steps_per_second"] for h in history]
    acting = statistics.median(h["speed/acting_share"] for h in history)
    trained = statistics.median(h["rollout/trained_share"] for h in history)
    age = max(h["rollout/decision_age_max"] for h in history)
    played = trainer.decisions + trainer.frozen_decisions - first
    print(f"time per update: acting {acting:.0%}, update {1 - acting:.0%} (median)")
    print(f"decisions trained on: {trained:.0%} of those played (median); oldest decision in an update: age {age:.0f}")
    print(
        f"done: {len(history)} updates, {played:,} decisions in {seconds:.1f} s; "
        f"median {statistics.median(speeds):,.0f} steps/s per update, {played / seconds:,.0f} steps/s overall"
    )
    if "task/score" in history[0]:
        print(f"task score: first {history[0]['task/score']:.3f}, last {history[-1]['task/score']:.3f}")
    if device == "cuda":
        print(f"gpu load after the run: {gpu_snapshot()}; peak memory {torch.cuda.max_memory_allocated() // 2**20} MiB")
    print(f"checkpoints and TensorBoard files: {out.name} in {out.parent.name}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
