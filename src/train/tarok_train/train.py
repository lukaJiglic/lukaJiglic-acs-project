"""The training loop (spec 007), on the mock environment or on the real engine (spec 008).

Run (cmd, from the repo root, venv active):

    python -m tarok_train.train --config src\\train\\configs\\mock.toml
    python -m tarok_train.train --config src\\train\\configs\\smoke-a-mlp.toml

The [env] table of the config names the environment: kind "mock" (tarok_train/mock_env.py)
or kind "engine" (the Rust engine through tarok_train/env.py; the module tarok_env must be
built).

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

A run started from the command line writes two files next to its checkpoints:
`run-info.json` at the start (the commit, `git status --short`, the config as it
ran) and `summary.json` at the end (speed, time split, what failed).
"""

from __future__ import annotations

import argparse
import copy
import datetime
import json
import math
import subprocess
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
        "time_limit_minutes": float,
        "fault": str,
        "gpu_memory_limit_mib": int,
    },
    "env": None,  # depends on the kind: see ENV_KEYS
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
    "entropy_control": {
        "enabled": bool,
        "gain": float,
        "proportional": float,
        "min_weight": float,
        "max_weight": float,
        "band": float,
        "targets": dict,
    },
}

# The keys of [env] for each kind of environment.
ENV_KEYS = {
    "mock": {
        "kind": str,
        "mode": str,
        "tables": int,
        "min_rounds": int,
        "max_rounds": int,
        "legal_per_decision": int,
        "extra_token_probability": float,
        "feature_density": float,
    },
    "engine": {"kind": str, "tables": int, "threads": int},
}

# [run] fault: a planted wiring bug for the control run of spec 008 (criterion 8).
# "rewards_to_next_seat": every seat's reward is handed to the next seat.
FAULTS = ("none", "rewards_to_next_seat")
CONTROL_PREFIX = "control-"

# What the wall time of an update is split into (spec 008, criterion 10).
CLOCKS = ("environment", "acting", "bookkeeping", "update")

# GPU memory right now, read twice per update (spec 015, criterion 4): when acting is over, and
# when the update step is over. "reserved" is what PyTorch's allocator holds from the card,
# "allocated" the tensors in it. The two "gpu/peak_..." tags are the largest values since the start.
MEMORY_TAGS = (
    "gpu/reserved_after_acting_mib",
    "gpu/allocated_after_acting_mib",
    "gpu/reserved_after_update_mib",
    "gpu/allocated_after_update_mib",
)

# Keys that were added to the config after runs had been made, with the value those runs had.
# A checkpoint from before the key existed is read with that value, so it can still be resumed.
# This is not a default for config files: a file without the key is refused.
LATER_KEYS = {
    ("run", "gpu_memory_limit_mib"): 0,  # spec 015: the runs before it had no limit on the allocator
}

# The cards (by number) on which this process has set a limit on PyTorch's allocator. A limit
# holds for the process; a trainer without one must lift what an earlier trainer set.
LIMITED_CARDS = set()

# A limit this much above what PyTorch reports as the card's size is read as "the whole card".
# The card reports a little less than its nominal size (6,143.7 MiB for a card of 6,144 MiB).
CARD_SIZE_TOLERANCE = 0.01


class ConfigError(ValueError):
    """The config file lacks a key, has an unknown key, or a value that cannot be used."""


class EncodingVersionError(RuntimeError):
    """A checkpoint was made for another version of docs/encoding.md."""


def validate_config(cfg: dict) -> None:
    for section, keys in SCHEMA.items():
        if section not in cfg:
            raise ConfigError(f"the config has no [{section}] table")
        if section == "env":
            kind = cfg["env"].get("kind")
            if kind is None:
                raise ConfigError("the config has no value for kind in [env]; the code has no default")
            if kind not in ENV_KEYS:
                raise ConfigError(f"unknown environment kind {kind!r} in [env]: use 'mock' or 'engine'")
            keys = ENV_KEYS[kind]
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
    run = cfg["run"]
    if run["time_limit_minutes"] < 0:
        raise ConfigError("time_limit_minutes in [run] must be 0 (no limit) or more")
    if run["gpu_memory_limit_mib"] < 0:
        raise ConfigError("gpu_memory_limit_mib in [run] must be 0 (no limit) or more")
    if run["fault"] not in FAULTS:
        raise ConfigError(f"fault in [run] must be one of {FAULTS}, got {run['fault']!r}")
    if run["fault"] != "none" and not run["name"].startswith(CONTROL_PREFIX):
        raise ConfigError(
            f"fault = {run['fault']!r} in [run] is refused: a planted fault is for control runs only, "
            f"and the name of such a run starts with '{CONTROL_PREFIX}'"
        )
    control = cfg["entropy_control"]
    phases = C.PHASES[:-1]  # every phase in which a seat decides
    for name in phases:
        if name not in control["targets"]:
            raise ConfigError(f"[entropy_control.targets] has no target for the phase '{name}'")
    for name, target in control["targets"].items():
        if name not in phases:
            raise ConfigError(f"unknown phase '{name}' in [entropy_control.targets]")
        if isinstance(target, bool) or not isinstance(target, (int, float)) or target <= 0:
            raise ConfigError(f"the target entropy of '{name}' in [entropy_control.targets] must be above 0")
    if not 0 < control["min_weight"] < control["max_weight"]:
        raise ConfigError("[entropy_control] needs 0 < min_weight < max_weight")
    if control["gain"] <= 0 or control["proportional"] < 0 or not 0 < control["band"] < 1:
        raise ConfigError("[entropy_control] needs gain above 0, proportional 0 or more and band between 0 and 1")


def load_config(path) -> dict:
    with open(path, "rb") as handle:
        cfg = tomllib.load(handle)
    validate_config(cfg)
    return cfg


def gpu_state(when: str) -> dict:
    """The reading of nvidia-smi `when` ("before" or "after") the run. An empty reading is said in one line."""
    reading = gpu_snapshot()
    if not reading:
        print(f"the GPU state {when} the run could not be read: nvidia-smi gave no output this code can read")
    return reading


def with_later_keys(cfg: dict) -> dict:
    """The config of a checkpoint, with every key of LATER_KEYS it lacks set to the value its run had."""
    for (section, key), value in LATER_KEYS.items():
        if section in cfg and key not in cfg[section]:
            cfg[section][key] = value
    return cfg


def limit_gpu_memory(limit_mib: int, device) -> float | None:
    """Limit PyTorch's allocator on a CUDA device to `limit_mib` MiB (spec 015). 0: no limit.

    Why: on Windows the driver does not refuse memory above what the card has free. It places
    the excess in shared system memory, and the run slows down without an error. The allocator
    keeps freed blocks as a cache, and that cache is what grows past the card (measured in spec
    015: tensors 2.2 GB between the phases and 4.4 GB at their peak, allocator 6.5 GB). With a
    limit the allocator frees its cache and retries when a new block would pass the limit; only
    if the tensors themselves do not fit does it raise an out-of-memory error.

    The limit holds for the whole process. Returns the fraction of the card's memory that was
    set, or None when nothing was set (the CPU, or no limit in a process that never set one).

    No limit after a limit: if this process has limited the card before, 0 sets the fraction
    back to 1.0, the whole card, and returns 1.0. That is the widest setting PyTorch takes. It
    is not the same as never having set one: without any setting the driver may hand out more
    than the card has (the spill this limit is for); with 1.0 the allocator stops at the
    card's size.
    """
    device = torch.device(device)
    if device.type != "cuda":
        return None
    # The calls below want the number of the card; "cuda" without a number is the current one.
    card = device.index if device.index is not None else torch.cuda.current_device()
    if limit_mib == 0:
        if card not in LIMITED_CARDS:
            return None
        torch.cuda.set_per_process_memory_fraction(1.0, card)
        LIMITED_CARDS.discard(card)
        return 1.0
    total_mib = torch.cuda.get_device_properties(card).total_memory / 2**20
    if limit_mib > total_mib * (1.0 + CARD_SIZE_TOLERANCE):
        raise ConfigError(f"gpu_memory_limit_mib in [run] is {limit_mib}, above the card's {total_mib:.0f} MiB")
    fraction = min(1.0, limit_mib / total_mib)
    torch.cuda.set_per_process_memory_fraction(fraction, card)
    LIMITED_CARDS.add(card)
    return fraction


def gpu_limit_line(limit_mib: int, fraction) -> str:
    """What a GPU run prints at its start about the limit. `fraction`: what `limit_gpu_memory` returned."""
    if limit_mib:
        return f"gpu memory: PyTorch's allocator is limited to {limit_mib:,} MiB"
    if fraction is None:
        return "gpu memory: no limit on PyTorch's allocator"
    return "gpu memory: a limit set earlier in this process was lifted; PyTorch's allocator may take the whole card and not more"


def gpu_memory_now(device) -> tuple | None:
    """(reserved, allocated) in MiB at this moment, of this process on a CUDA device. None on the CPU."""
    if torch.device(device).type != "cuda":
        return None
    return torch.cuda.memory_reserved(device) / 2**20, torch.cuda.memory_allocated(device) / 2**20


def make_env(cfg_env: dict, seed: int):
    """The environment a config asks for. The engine's module is imported only when it is needed."""
    if cfg_env["kind"] == "mock":
        return mock_env.make(cfg_env, seed)
    from . import env as engine_env  # needs the built tarok_env module

    return engine_env.make(cfg_env, seed)


class Trainer:
    def __init__(self, cfg: dict, device, log_dir=None):
        validate_config(cfg)
        self.cfg = copy.deepcopy(cfg)
        run, cfg_model, cfg_ppo, cfg_opp = (self.cfg[k] for k in ("run", "model", "ppo", "opponents"))
        self.device = torch.device(device)
        # Before anything is put on the device. The limit holds for the whole process (spec 015).
        self.gpu_memory_fraction = limit_gpu_memory(run["gpu_memory_limit_mib"], self.device)
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

        self.env = make_env(self.cfg["env"], seed)
        self.fault = run["fault"]
        self.entropy = ppo.EntropyController(self.cfg["entropy_control"], cfg_ppo["entropy_weight"])
        self.n = self.cfg["env"]["tables"]
        self.actor = Actor(self.net, self.frozen_net, self.n, self.device, cfg_model)
        self.pool = opponents.OpponentPool(cfg_opp["pool_size"])
        self.frozen = np.zeros((self.n, 4), dtype=bool)  # seats played by the past checkpoint, per table
        self.frozen_active = False  # True once a past checkpoint is loaded
        self.deal_started = np.ones(self.n, dtype=bool)  # no decision taken yet in the table's deal
        # A table is fresh when its current deal began under the current weights. Only fresh
        # tables are stored. After an update every table that is inside a deal is stale.
        self.fresh = np.ones(self.n, dtype=bool)
        # On the real game the storage also checks that a deal's four rewards add up to zero.
        self.rollout = Rollout(zero_sum=getattr(self.env, "zero_sum", False))
        self.clock = dict.fromkeys(CLOCKS, 0.0)  # seconds since the last update, by kind of work
        self.clock_total = dict.fromkeys(CLOCKS, 0.0)  # the same over all updates of this object
        self.stopped_by_time_limit = False

        self.updates = 0
        self.decisions = 0  # decisions of trained seats (stored or not)
        self.frozen_decisions = 0  # decisions of seats played by the past checkpoint
        self.collected = 0  # finished seat-deals stored since the last update
        # The number of updates done when each table's current deal began. A stored decision carries
        # it as its version: a deal that began under older weights shows as an age above 0.
        self.deal_version = np.zeros(self.n, dtype=np.int64)
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

    def _lap(self, name: str, since: float) -> float:
        """Add the time since `since` to the clock `name`. Returns the time now."""
        now = time.perf_counter()
        self.clock[name] += now - since
        return now

    def step(self) -> None:
        """One environment step: one decision per table."""
        lap = time.perf_counter()
        obs = self.env.observe()
        lap = self._lap("environment", lap)
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
            logp = logp.cpu().numpy()
            lap = self._lap("acting", lap)
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
                logp=logp[keep],
                value=value,
                version=self.deal_version[keep],
            )
        self.decisions += int((~frozen_rows).sum())
        self.frozen_decisions += int(frozen_rows.sum())
        # Without stored rows this lap is acting time; with them it is the storing.
        lap = self._lap("bookkeeping" if keep.any() else "acting", lap)

        done, reward, parts = self.env.step(action)
        lap = self._lap("environment", lap)
        true_parts = parts  # what the environment paid: the monitors read these
        if self.fault == "rewards_to_next_seat":  # the control run: seat s gets what seat s - 1 earned
            reward, parts = np.roll(reward, 1, axis=1), np.roll(parts, 1, axis=1)
        ended = np.flatnonzero(done)
        if len(ended):
            stored = ended[self.fresh[ended]]
            if len(stored):
                self.rollout.finish(stored, deal[stored], reward[stored], parts[stored], true_parts[stored])
                self.collected += int((~self.frozen[stored]).sum())
            self.frozen[ended] = False
            self.fresh[ended] = True  # the next deal of these tables begins under the current weights
            self.deal_version[ended] = self.updates
            self._assign_frozen(ended)
        self.deal_started = done.copy()
        self._lap("bookkeeping", lap)

    def collect(self) -> None:
        """Play until enough stored seat-deals have finished."""
        self.collected = 0
        while self.collected < self.cfg["ppo"]["seat_deals_per_update"]:
            self.step()

    # ---- learning ----

    def update(self) -> dict | None:
        """PPO on the stored deals that finished. None if there is none."""
        lap = time.perf_counter()
        deals = self.rollout.take()
        self.rollout.discard()  # rows of deals still running would be stale after this update
        self.fresh = self.deal_started.copy()  # only tables that stand at a deal's first decision
        if deals is None:
            self._lap("bookkeeping", lap)
            return None
        cfg_ppo = self.cfg["ppo"]
        ppo.prepare(deals, cfg_ppo)
        stats = ppo.rollout_stats(deals, self.updates)
        stats["env/illegal_actions"] = float(self.env.illegal_actions)
        if getattr(self.env, "reward_is_score", False):
            stats["task/score"] = float(deals.reward.mean())
        if getattr(self.env, "real_deals", False):
            stats.update(ppo.deal_stats(deals))
        memory = [gpu_memory_now(self.device)]  # acting is over: the acting cache is at its largest
        lap = self._lap("bookkeeping", lap)
        stats.update(
            ppo.update(
                self.net, self.critic, self.optimizer, deals, cfg_ppo, self.device, self.rng,
                self.entropy.tensor(self.device),
            )
        )
        self._synchronise()
        lap = self._lap("update", lap)
        memory.append(gpu_memory_now(self.device))  # the update step is over
        if memory[0] is not None and memory[1] is not None:
            for tag, value in zip(MEMORY_TAGS, (*memory[0], *memory[1])):
                stats[tag] = float(value)
        self.net.eval()
        self.critic.eval()
        self.actor.reset()  # the weights changed: the cached keys and values are stale
        self.updates += 1
        self.deal_version[self.deal_started] = self.updates  # these deals begin under the new weights
        if self.updates % self.cfg["opponents"]["snapshot_every"] == 0:
            self.pool.add(self.net)
        if len(self.pool):
            self.use_frozen_seats()
        # The entropy controller: the next update's weights, from this update's entropy per phase.
        entropies = {name: stats[f"entropy/{name}"] for name in C.PHASES if f"entropy/{name}" in stats}
        self.entropy.step(entropies)
        if self.entropy.enabled:
            stats["entropy/phases_outside_band"] = float(sum(not ok for ok in self.entropy.in_band(entropies).values()))
        for name, weight in zip(C.PHASES[:-1], self.entropy.weights):
            stats[f"entropy_weight/{name}"] = float(weight)
        self._lap("bookkeeping", lap)
        return stats

    def run(self, updates: int, checkpoint_dir=None, report_every: int = 0, time_limit_seconds: float = 0.0) -> list:
        """Run `updates` updates. Returns their statistics; they are also appended to `history`.

        time_limit_seconds: 0 for none. When the run has taken longer, it stops after the
        update in hand and `stopped_by_time_limit` is True.
        A logged number that is not finite stops the run with a FloatingPointError.
        """
        self.net.eval()
        self.critic.eval()
        self.stopped_by_time_limit = False
        begun = time.perf_counter()
        done = []
        for index in range(updates):
            started = time.perf_counter()
            before_own, before_frozen = self.decisions, self.frozen_decisions
            self.clock = dict.fromkeys(CLOCKS, 0.0)
            stats = None
            while stats is None:
                self.collect()
                stats = self.update()
            seconds = time.perf_counter() - started
            stats["time/seconds"] = seconds
            # Decisions played since the last update: by the seats of the current network, and by all
            # seats (those of a past checkpoint too). "rollout/decisions" are the ones trained on.
            own = self.decisions - before_own
            played = own + self.frozen_decisions - before_frozen
            stats["rollout/played"] = float(played)
            stats["rollout/played_own_seats"] = float(own)
            stats["rollout/trained_share"] = stats["rollout/decisions"] / max(played, 1)
            stats["rollout/trained_share_own_seats"] = stats["rollout/decisions"] / max(own, 1)
            stats["speed/steps_per_second"] = played / seconds
            stats["speed/trained_per_second"] = stats["rollout/decisions"] / seconds
            for name in CLOCKS:  # the four shares add up to the wall time of the update, but for loop overhead
                stats[f"time/{name}_share"] = self.clock[name] / seconds
                self.clock_total[name] += self.clock[name]
            stats["speed/acting_share"] = 1.0 - stats["time/update_share"]  # all but the update (spec 007's tag)
            stats["opponents/frozen_share"] = self.frozen_decisions / max(self.decisions + self.frozen_decisions, 1)
            if self.device.type == "cuda":
                # Allocated: the tensors. Reserved: what PyTorch's allocator holds from the card.
                stats["gpu/peak_memory_mib"] = float(torch.cuda.max_memory_allocated() // 2**20)
                stats["gpu/peak_memory_reserved_mib"] = float(torch.cuda.max_memory_reserved() // 2**20)
            for name, value in stats.items():
                if not math.isfinite(value):
                    raise FloatingPointError(
                        f"{name} is {value} at update {self.updates}: a logged number is not finite, the run stops"
                    )
            self.history.append(stats)
            done.append(stats)
            lap = time.perf_counter()
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
            # Logging and checkpoints are bookkeeping of the run; they are not part of an update's shares.
            self.clock_total["bookkeeping"] += time.perf_counter() - lap
            if time_limit_seconds and index + 1 < updates and time.perf_counter() - begun > time_limit_seconds:
                self.stopped_by_time_limit = True  # updates are left: the run is not finished
                break
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
                "entropy_control": self.entropy.state(),
                "frozen_active": self.frozen_active,
                "frozen": torch.from_numpy(self.frozen),
                "deal_started": torch.from_numpy(self.deal_started),
                "deal_version": torch.from_numpy(self.deal_version),
                "fresh": torch.from_numpy(self.fresh),
                "rng": _plain(self.rng.bit_generator.state),
                "generator": self.generator.get_state(),
                "environment": _plain(environment),
            },
            path,
        )

    @classmethod
    def load(cls, path, device, log_dir=None, gpu_memory_limit_mib=None) -> "Trainer":
        """A trainer that goes on from a checkpoint.

        gpu_memory_limit_mib: None keeps the checkpoint's value; a number replaces it. The limit
        is a setting of the machine, not of the training, so a resumed run may change it.
        """
        # To the CPU first: the trainer sets the limit on the allocator when it is built, and until
        # then nothing may be put on the card. `load_state_dict` moves the weights over afterwards.
        data = torch.load(path, map_location="cpu", weights_only=True)
        version = data.get("encoding_version")
        if version != C.ENCODING_VERSION:
            raise EncodingVersionError(
                f"the checkpoint was made for encoding version {version}; this code reads "
                f"encoding version {C.ENCODING_VERSION} (docs/encoding.md). It cannot be loaded."
            )
        cfg = with_later_keys(data["config"])
        if gpu_memory_limit_mib is not None:
            cfg["run"]["gpu_memory_limit_mib"] = gpu_memory_limit_mib
        trainer = cls(cfg, device, log_dir)
        trainer.net.load_state_dict(data["model"])
        trainer.critic.load_state_dict(data["critic"])
        trainer.optimizer.load_state_dict(data["optimizer"])
        trainer.updates = data["updates"]
        trainer.decisions = data["decisions"]
        trainer.frozen_decisions = data["frozen_decisions"]
        trainer.pool.snapshots = [{k: v.cpu() for k, v in snapshot.items()} for snapshot in data["pool"]]
        trainer.frozen_net.load_state_dict(data["frozen_net"])
        trainer.entropy.load_state(data["entropy_control"])
        trainer.frozen_active = data["frozen_active"]
        trainer.frozen = data["frozen"].cpu().numpy().copy()
        trainer.deal_started = data["deal_started"].cpu().numpy().copy()
        trainer.deal_version = data["deal_version"].cpu().numpy().copy()
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


def run_summary(history: list, seconds: float) -> dict:
    """Counts and rates of a run from the statistics of its updates and its wall time.

    played: decisions of all seats, those of a past checkpoint too. trained: the decisions the
    updates trained on. trained_share: trained / played (spec 008, criterion 9).
    trained_share_own_seats: trained / the decisions of the current network's seats.
    Only logged updates count: an update that failed is in neither number.
    """
    played = sum(stats["rollout/played"] for stats in history)
    own = sum(stats["rollout/played_own_seats"] for stats in history)
    trained = sum(stats["rollout/decisions"] for stats in history)
    return {
        "played": int(played),
        "played_own_seats": int(own),
        "trained": int(trained),
        "played_per_second": played / seconds if seconds > 0 else 0.0,
        "trained_per_second": trained / seconds if seconds > 0 else 0.0,
        "trained_share": trained / played if played else 0.0,
        "trained_share_own_seats": trained / own if own else 0.0,
    }


def _git(*args) -> str:
    """The output of a git command run in the repo, or "unknown"."""
    try:
        done = subprocess.run(["git", *args], cwd=TRAIN_DIR, capture_output=True, text=True, timeout=60)
    except (OSError, subprocess.SubprocessError):
        return "unknown"
    return done.stdout.strip() if done.returncode == 0 else "unknown"


def _write_json(path: Path, data: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(data, indent=1), encoding="utf-8")


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description="Train on the environment named in a run config (specs 007, 008).")
    parser.add_argument("--config", default=str(DEFAULT_CONFIG), help="run config (TOML)")
    parser.add_argument("--resume", default=None, help="a checkpoint to go on from; its config is used")
    parser.add_argument("--network", default=None, help="transformer or mlp; overrides [run] network")
    parser.add_argument("--updates", type=int, default=None, help="overrides [run] updates (the total, also on resume)")
    parser.add_argument("--tables", type=int, default=None, help="overrides [env] tables")
    parser.add_argument("--mode", default=None, help="mock mode: learnable, tokens or random; overrides [env] mode")
    parser.add_argument("--seat-deals", type=int, default=None, help="overrides [ppo] seat_deals_per_update")
    parser.add_argument("--device", default=None, help="cuda or cpu; overrides [run] device")
    parser.add_argument("--out-dir", default=None, help="folder for checkpoints and logs; overrides [run] out_dir")
    parser.add_argument("--name", default=None, help="overrides [run] name")
    parser.add_argument("--fault", default=None, help="control runs only: overrides [run] fault")
    parser.add_argument("--time-limit-minutes", type=float, default=None, help="overrides [run] time_limit_minutes")
    parser.add_argument(
        "--gpu-memory-limit-mib", type=int, default=None, help="overrides [run] gpu_memory_limit_mib; 0 is no limit; also on --resume"
    )
    args = parser.parse_args(argv)

    resume = None
    if args.resume is not None:
        for name in ("network", "tables", "mode", "seat_deals", "name", "fault"):
            if getattr(args, name) is not None:
                parser.error(f"--{name.replace('_', '-')} cannot change on --resume: the checkpoint's config is used")
        resume = Path(args.resume)
        cfg = with_later_keys(torch.load(resume, map_location="cpu", weights_only=True)["config"])
    else:
        cfg = load_config(args.config)
    # Overrides go into the config, so the checkpoint records what really ran.
    for section, key, value in (
        ("run", "network", args.network),
        ("run", "name", args.name),
        ("run", "fault", args.fault),
        ("run", "time_limit_minutes", args.time_limit_minutes),
        ("run", "gpu_memory_limit_mib", args.gpu_memory_limit_mib),
        ("env", "tables", args.tables),
        ("env", "mode", args.mode),
        ("ppo", "seat_deals_per_update", args.seat_deals),
        ("run", "device", args.device),
    ):
        if value is not None:
            if key not in cfg[section]:
                parser.error(f"--{key.replace('_', '-')} does not apply: [{section}] of this config has no {key}")
            cfg[section][key] = value
    validate_config(cfg)
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

    kind = cfg["env"]["kind"]
    what = f"mode {cfg['env']['mode']}" if kind == "mock" else f"{cfg['env']['threads']} threads"
    print(f"run {name}: {total} updates on {device}, environment {kind}, {cfg['env']['tables']} tables, {what}")
    gpu_before = gpu_state("before") if device == "cuda" else None
    if device == "cuda":
        print(f"gpu: {torch.cuda.get_device_name(0)}; load before the run: {gpu_before}")
    if resume is not None:
        trainer = Trainer.load(resume, device, log_dir=out, gpu_memory_limit_mib=args.gpu_memory_limit_mib)
        print(f"resumed from {resume.name} at update {trainer.updates}")
    else:
        cfg["run"]["updates"] = total
        trainer = Trainer(cfg, device, log_dir=out)
    if device == "cuda":
        print(gpu_limit_line(trainer.cfg["run"]["gpu_memory_limit_mib"], trainer.gpu_memory_fraction))

    # The run log needs the commit and the state of the tree: a run may start on uncommitted work.
    info = {
        "started": datetime.datetime.now().isoformat(timespec="seconds"),
        "run": name,
        "commit": _git("rev-parse", "HEAD"),
        "git_status_short": _git("status", "--short"),
        "config_file": Path(args.config).name if resume is None else None,
        "resumed_from": resume.name if resume is not None else None,
        "config": trainer.cfg,
        "device": device,
        "gpu_before": gpu_before,
        "engine_sources": getattr(trainer.env, "source_hash", None),
        "parameters": trainer.parameter_counts,
    }
    print(f"commit {info['commit']}; uncommitted changes: {'yes' if info['git_status_short'] else 'no'}")
    _write_json(out / ("run-info.json" if resume is None else f"run-info-from-{trainer.updates:06d}.json"), info)

    remaining = max(total - trainer.updates, 0)
    limit = cfg["run"]["time_limit_minutes"] * 60.0
    started = time.perf_counter()
    updates_before = trainer.updates
    failed = None
    try:
        trainer.run(remaining, checkpoint_dir=out, report_every=max(1, total // 20), time_limit_seconds=limit)
    except Exception as error:  # the summary names what failed; the error still ends the run
        failed = f"{type(error).__name__}: {error}"
        raise
    finally:
        seconds = time.perf_counter() - started
        history = trainer.history  # the updates of this call only: a checkpoint does not hold the history
        counts = run_summary(history, seconds)
        played = counts["played"]
        finished = failed is None and not trainer.stopped_by_time_limit and trainer.updates >= total
        gpu_after = gpu_state("after") if device == "cuda" else None
        peak = int(torch.cuda.max_memory_allocated() // 2**20) if device == "cuda" else None
        reserved = int(torch.cuda.max_memory_reserved() // 2**20) if device == "cuda" else None
        shares = {name: trainer.clock_total[name] / seconds for name in CLOCKS} if seconds > 0 else {}
        summary = {
            "run": name,
            "environment": kind,
            "network": cfg["run"]["network"],
            "device": device,
            # Updates done and logged. An update that failed its check of the logged numbers is not counted.
            "updates": updates_before + len(history),
            "updates_wanted": total,
            "finished": finished,
            "failed": failed,
            "stopped_by_time_limit": trainer.stopped_by_time_limit,
            "seconds": seconds,
            **counts,
            "time_shares": shares,
            "time_shares_sum": sum(shares.values()),
            "illegal_actions": int(trainer.env.illegal_actions),
            "decision_age_max": max((h["rollout/decision_age_max"] for h in history), default=0.0),
            # Three different numbers. peak_gpu_memory_mib: the largest sum of this run's tensors.
            # peak_gpu_memory_reserved_mib: the most PyTorch's allocator held from the card for them.
            # gpu_before / gpu_after "memory_used_mib": what nvidia-smi shows for the whole card,
            # every program on it, at that moment (the run still holds its memory at "after").
            "peak_gpu_memory_mib": peak,
            "peak_gpu_memory_reserved_mib": reserved,
            "gpu_before": gpu_before,
            "gpu_after": gpu_after,
            "first_update": history[0] if history else None,
            "last_update": history[-1] if history else None,
        }
        if resume is None or history:
            _write_json(out / "summary.json", summary)

    final = out / f"checkpoint-{trainer.updates:06d}.pt"
    if not final.exists():
        trainer.save(final)
    trainer.close()
    if not history:
        print("nothing to do: the checkpoint already has that many updates")
        return 0

    split = ", ".join(f"{name} {share:.0%}" for name, share in shares.items())
    print(f"time: {split}; together {summary['time_shares_sum']:.1%} of the wall time")
    print(
        f"decisions trained on: {summary['trained_share']:.0%} of those played "
        f"({summary['trained_share_own_seats']:.0%} of those of the current network's seats); "
        f"oldest decision in an update: age {summary['decision_age_max']:.0f}; "
        f"illegal actions refused by the environment: {summary['illegal_actions']}"
    )
    print(
        f"done: {len(history)} updates, {played:,} decisions in {seconds:.1f} s; "
        f"{summary['played_per_second']:,.0f} played per second, {summary['trained_per_second']:,.0f} trained per second"
    )
    if "task/score" in history[0]:
        print(f"task score: first {history[0]['task/score']:.3f}, last {history[-1]['task/score']:.3f}")
    if device == "cuda":
        print(f"gpu load after the run: {gpu_after}; peak memory of the tensors {peak} MiB, reserved by PyTorch {reserved} MiB")
    print(f"checkpoints and TensorBoard files: {out.name} in {out.parent.name}")
    if not finished:
        print(
            f"not finished: the time limit of {cfg['run']['time_limit_minutes']:g} minutes was reached "
            f"at update {trainer.updates} of {total}"
        )
        return 3
    return 0


if __name__ == "__main__":
    sys.exit(main())
