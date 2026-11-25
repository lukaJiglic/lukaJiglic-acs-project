"""A checkpoint on seeded deals: against random legal play (spec 008, criteria 7, 8 and 15),
or against another checkpoint (spec 015, criteria 11 to 15).

Run (cmd, from the repo root, venv active):

    python -m tarok_train.evaluate --checkpoint src\\train\\runs\\<run folder>\\checkpoint-000200.pt --deals 2000 --seed 2008 --records 100
    python -m tarok_train.evaluate --checkpoint <the same> --deals 2000 --seed 2008 --untrained
    python -m tarok_train.evaluate --checkpoint <A> --opponent <B> --deals 2000 --seed 5151

Every deal is played four times: with the agent in seat 0, 1, 2 and 3, and random
legal play in the other three seats. The cards of a deal depend only on the seed and
the deal's number, so the four seatings see the same cards. The value of a deal is
the mean of the agent's four rewards. Reported: the mean over the deals and its 95%
interval (mean plus and minus 1.96 standard errors over the deals). The agent "beats
random" when the lower end of the interval is above zero.

The agent samples from its policy, as in training. With `--untrained` the network
of the checkpoint's config is built with the run's seed and no weights are loaded:
the same network before its first update.

The random numbers of the sampling come from one generator per network for the whole
evaluation, seeded with `--seed` (the opponent's with `--seed` + 1). No chunk of deals
and no seating gets the numbers of another one. (Until the review of spec 015 every
environment started a new generator from the same seed. A table index then met the same
numbers in every chunk, the deal values were tied to each other, and the interval was too
narrow for policies that depend little on the cards. Files made before that fix cannot
be repeated to the digit with this code.)

`--records N` exports the first N deals (agent in seat 0) as game records of spec 006
and replays each one: the replay must give the entries the environment reported.

Against random legal play this is a test of the wiring, not a measure of playing strength.

With `--opponent B` the three other seats are played by checkpoint B, which samples from
its policy too, with its own seeded generator. The value of a deal is then what one seat of
A earns among three seats of B: above zero means A does better in that company than B does.
The result goes to `evaluation-NNNNNN-vs-<run folder of B>-MMMMMM.json` next to A (N: A's
updates, M: B's), so it never replaces the file of the evaluation against random play, and
two runs of one name give two files. The same pair with another seed, other deals or other
tables has the same file name: an existing file is replaced only with `--overwrite`. No
chunk of deals may use the training seed of A or of B.
"""

from __future__ import annotations

import argparse
import json
import sys
from dataclasses import dataclass
from pathlib import Path

import numpy as np
import torch

import tarok_env

from . import contract as C
from .env import EngineEnv
from .model import Actor, build_network

SEATS = 4


def interval(values) -> tuple:
    """(mean, low, high): the mean and its 95% interval, 1.96 standard errors on each side."""
    values = np.asarray(values, dtype=np.float64)
    mean = float(values.mean())
    error = float(values.std(ddof=1) / np.sqrt(len(values))) if len(values) > 1 else float("inf")
    return mean, mean - 1.96 * error, mean + 1.96 * error


def first_legal(rows, obs) -> np.ndarray:
    """The lowest legal slot at each of the tables `rows`. A fixed rule for the tests."""
    return obs["mask"][rows].argmax(axis=1)


class RandomLegal:
    """Random legal play: every legal slot with the same chance."""

    def __init__(self, seed: int):
        self.rng = np.random.default_rng(seed)

    def __call__(self, rows, obs) -> np.ndarray:
        mask = obs["mask"][rows]
        scores = self.rng.random(mask.shape, dtype=np.float32)
        scores[~mask] = -1.0
        return scores.argmax(axis=1)


def network_agent(net, cfg_model: dict, device, seed: int):
    """`make_agent` for `evaluate`: the network samples from its masked policy.

    Every call of the result gives an agent for one environment of `n_tables` tables,
    with its own acting cache (the transformer). All of them draw from ONE generator,
    made here and seeded with `seed`: the environments of an evaluation are played one
    after the other, so each gets the next random numbers and none gets them twice.
    A new generator per environment from the same seed would give every chunk of deals
    and every seating the same numbers at the same table (review of spec 015, point 1).
    Make a new `network_agent` for a new evaluation.
    """
    device = torch.device(device)
    generator = torch.Generator(device=device).manual_seed(seed)

    def make_agent(n_tables: int):
        actor = Actor(net, None, n_tables, device, cfg_model)

        def agent(rows, obs) -> np.ndarray:
            action, _, _ = actor.act(
                rows,
                obs["seat"][rows],
                obs["deal"][rows],
                obs["n_tokens"][rows],
                np.zeros(len(rows), dtype=bool),
                torch.from_numpy(obs["tokens"][rows]).to(device).long(),
                torch.from_numpy(obs["state"][rows]).to(device),
                torch.from_numpy(obs["mask"][rows]).to(device),
                generator,
            )
            return action.cpu().numpy()

        return agent

    return make_agent


@dataclass
class Seating:
    """The first deal of every table of an environment, with the agent in one seat."""

    reward: np.ndarray  # (n,) the agent's reward
    entries: np.ndarray  # (n, 4) the paper entries of the deal
    slots: list  # the action slots of the deal, for the first `n_record` tables
    decisions: int  # decisions the agent took


def play_seating(env, agent_seat: int, agent, others, n_record: int = 0, others_skip_agent: bool = False) -> Seating:
    """Play the first deal of every table of a fresh `env`.

    agent(rows, obs) and others(rows, obs) return one legal slot for each of the tables
    `rows`. The agent is asked only where the seat to act is `agent_seat`. A table whose
    first deal has ended goes on with `others` in every seat until all are done; those
    later deals are not counted.

    others_skip_agent: False (random legal play, spec 008): `others` is asked for every
    table at every step and the agent's slot replaces its answer, so its stream of random
    numbers does not depend on the agent. True (a network as the opponent, spec 015):
    `others` is asked only for the tables the agent does not play at this step.
    """
    n = env.n
    all_rows = np.arange(n)
    reward = np.zeros(n, dtype=np.float32)
    entries = np.zeros((n, SEATS), dtype=np.int32)
    slots = [[] for _ in range(n_record)]
    open_tables = np.ones(n, dtype=bool)
    decisions = 0
    while open_tables.any():
        obs = env.observe()
        is_mine = open_tables & (obs["seat"] == agent_seat)
        mine = np.flatnonzero(is_mine)
        if others_skip_agent:
            action = np.zeros(n, dtype=np.int64)
            rest = np.flatnonzero(~is_mine)
            if len(rest):
                action[rest] = others(rest, obs)
        else:
            action = np.asarray(others(all_rows, obs)).astype(np.int64)
        if len(mine):
            action[mine] = agent(mine, obs)
            decisions += len(mine)
        for table in range(n_record):
            if open_tables[table]:
                slots[table].append(int(action[table]))
        done, step_reward, _ = env.step(action)
        ended = done & open_tables
        if ended.any():
            reward[ended] = step_reward[ended, agent_seat]
            entries[ended] = env.entries[ended]
        open_tables &= ~done
    return Seating(reward, entries, slots, decisions)


def replays_equal(record: str, entries) -> bool:
    """True if the game record replays in the engine to exactly these four entries."""
    try:
        return list(tarok_env.replay_record(record)) == [int(entry) for entry in entries]
    except ValueError:
        return False


def chunk_seeds(deals: int, tables: int, seed: int) -> list:
    """The engine seed of every chunk of tables: `seed`, `seed + 1`, ..."""
    return [seed + chunk for chunk in range((deals + tables - 1) // tables)]


def evaluate(
    make_agent,
    deals: int,
    tables: int,
    seed: int,
    threads: int,
    others=None,
    records: int = 0,
    training_seed=None,
    make_others=None,
    opponent_training_seed=None,
) -> dict:
    """`deals` seeded deals, each in four seatings.

    make_agent(n_tables) gives the agent of one environment. The other three seats are
    played by `others` (one callable for all environments: random legal play) or by
    make_others(n_tables) (an opponent with a state per environment: a network); give
    exactly one of the two. The deals are played in chunks of `tables` tables; chunk c
    uses the engine seed `seed + c`, so deal number d is table d mod `tables` of chunk
    d // `tables`, its first deal. `records`: export that many deals of the first chunk
    (agent in seat 0) as game records and replay them. `training_seed`: the seed of the run
    that made the agent; no chunk may use it, or its deals would be deals the agent trained
    on. `opponent_training_seed`: the same for the run that made the opponent.
    """
    if (others is None) == (make_others is None):
        raise ValueError("give either `others` (random legal play) or `make_others` (an opponent), not both and not neither")
    seeds = chunk_seeds(deals, tables, seed)
    if training_seed is not None and training_seed in seeds:
        raise ValueError(
            f"the evaluation would play a chunk of deals with the engine seed {training_seed}, the seed of the "
            f"training run (its chunks use the seeds {seeds[0]} to {seeds[-1]}): choose a seed outside that range"
        )
    if opponent_training_seed is not None and opponent_training_seed in seeds:
        raise ValueError(
            f"the evaluation would play a chunk of deals with the engine seed {opponent_training_seed}, the seed of the "
            f"opponent's training run (its chunks use the seeds {seeds[0]} to {seeds[-1]}): choose a seed outside that range"
        )
    if records > min(tables, deals):
        raise ValueError(f"at most {min(tables, deals)} records: they come from the first chunk of tables")
    rewards = np.zeros((deals, SEATS), dtype=np.float32)
    exported = []
    decisions = 0
    for chunk, first in enumerate(range(0, deals, tables)):
        n = min(tables, deals - first)
        for seat in range(SEATS):
            env = EngineEnv(n, seeds[chunk], threads)
            n_record = records if (chunk, seat) == (0, 0) else 0
            if make_others is None:
                seating = play_seating(env, seat, make_agent(n), others, n_record)
            else:
                seating = play_seating(env, seat, make_agent(n), make_others(n), n_record, others_skip_agent=True)
            rewards[first : first + n, seat] = seating.reward
            decisions += seating.decisions
            for table in range(n_record):
                # The first deal of a table: run 0, deal 0. Its dealer is the first dealer of the run.
                deal_seed = tarok_env.deal_seed(seeds[chunk], table, 0, 0)
                text = tarok_env.record_json(deal_seed, tarok_env.first_dealer(table, 0), seating.slots[table])
                exported.append({"table": table, "json": text, "entries": seating.entries[table].tolist()})
    values = rewards.mean(axis=1)
    mean, low, high = interval(values)
    return {
        "deals": deals,
        "seatings_per_deal": SEATS,
        "mean": mean,
        "low": low,
        "high": high,
        "beats_random": bool(low > 0),
        "mean_by_seat": rewards.mean(axis=0).tolist(),
        "agent_decisions": decisions,
        "values": values.tolist(),
        "rewards": rewards.tolist(),
        "records": exported,
        "records_exported": len(exported),
        "records_replayed_equal": sum(replays_equal(record["json"], record["entries"]) for record in exported),
    }


CHECKPOINT_KEYS = ("config", "network", "updates", "model", "encoding_version")


def load_checkpoint(path: Path, who: str, other_engine: bool = False):
    """The data of a checkpoint of a run on the engine, or None with one printed line that says why not.

    A checkpoint made with another version of the engine, or before checkpoints held the
    engine's version, was trained on another game (spec 009, criterion 53). It is refused
    unless `other_engine` is set; then one line of warning is printed and it is taken.
    """
    if not path.is_file():
        print(f"{who} cannot be read: there is no file {path}")
        return None
    try:
        data = torch.load(path, map_location="cpu", weights_only=True)
    except Exception as error:  # whatever the unpickler says: the file is not one of ours
        print(f"{who} cannot be read: {path} is not a checkpoint of tarok_train.train ({type(error).__name__})")
        return None
    if not isinstance(data, dict) or any(key not in data for key in CHECKPOINT_KEYS):
        print(f"{who} cannot be read: {path} is not a checkpoint of tarok_train.train")
        return None
    if data["encoding_version"] != C.ENCODING_VERSION:
        print(f"{who} was made for encoding version {data['encoding_version']}, this code reads {C.ENCODING_VERSION}")
        return None
    kind = data["config"].get("env", {}).get("kind")
    if kind != "engine":
        # The mock environment has the same array sizes and another game: its network would play, and mean nothing.
        print(f"{who} ({path}) comes from a run on the {kind} environment: only checkpoints of runs on the engine are evaluated")
        return None
    if data.get("engine_version") != tarok_env.ENGINE_VERSION:
        made = (
            "no engine version recorded (it was made before checkpoints held one)"
            if data.get("engine_version") is None
            else f"engine version {data['engine_version']}"
        )
        if not other_engine:
            print(
                f"{who} was made with {made}; this code runs engine version {tarok_env.ENGINE_VERSION}. "
                "It was trained on another game. To evaluate it all the same, add --other-engine"
            )
            return None
        print(
            f"warning: {who} was made with {made}; this code runs engine version {tarok_env.ENGINE_VERSION}. "
            "It plays a game it was not trained on."
        )
    return data


def load_network(data: dict, device, untrained: bool = False):
    """The actor of a checkpoint on the device, in evaluation mode."""
    cfg = data["config"]
    torch.manual_seed(cfg["run"]["seed"])  # the trainer builds its network first, from this seed
    net = build_network(cfg["model"], data["network"])
    if not untrained:
        net.load_state_dict(data["model"])
    return net.to(device).eval()


def run_name(data: dict) -> str:
    return f"{data['config']['run']['name']}-{data['network']}"


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(
        description="A checkpoint on seeded deals, against random legal play (spec 008) or another checkpoint (spec 015)."
    )
    parser.add_argument("--checkpoint", required=True, help="a checkpoint of tarok_train.train")
    parser.add_argument("--deals", type=int, required=True, help="number of deals; each is played in four seatings")
    parser.add_argument("--seed", type=int, required=True, help="seed of the deals; not the seed of the training run")
    parser.add_argument("--tables", type=int, default=500, help="deals played at the same time (memory of the acting cache)")
    parser.add_argument("--threads", type=int, default=6, help="threads of the engine")
    parser.add_argument("--device", default="auto", help="cuda, cpu or auto")
    parser.add_argument("--records", type=int, default=0, help="export this many deals as game records and replay them")
    parser.add_argument("--untrained", action="store_true", help="the same network before its first update")
    parser.add_argument(
        "--opponent",
        default=None,
        help="a checkpoint that plays the three other seats (spec 015); without it they are random legal play",
    )
    parser.add_argument(
        "--overwrite",
        action="store_true",
        help="with --opponent: replace the result file of an earlier call for the same pair of checkpoints",
    )
    parser.add_argument(
        "--other-engine",
        action="store_true",
        help="take a checkpoint made with another version of the engine, or before checkpoints held one (spec 009); a warning is printed",
    )
    args = parser.parse_args(argv)

    path = Path(args.checkpoint)
    data = load_checkpoint(path, "the checkpoint", args.other_engine)
    if data is None:
        return 2
    cfg = data["config"]
    seeds = chunk_seeds(args.deals, args.tables, args.seed)
    if cfg["run"]["seed"] in seeds:
        print(
            f"the evaluation would use the seed of the training run ({cfg['run']['seed']}) for one of its chunks of "
            "deals: choose a seed outside that range, or the deals are training deals"
        )
        return 2
    opponent_path = opponent_data = None
    if args.opponent is not None:
        opponent_path = Path(args.opponent)
        opponent_data = load_checkpoint(opponent_path, "the opponent's checkpoint", args.other_engine)
        if opponent_data is None:
            return 2
        if opponent_data["config"]["run"]["seed"] in seeds:
            print(
                f"the evaluation would use the seed of the opponent's training run ({opponent_data['config']['run']['seed']}) "
                "for one of its chunks of deals: choose a seed outside that range, or the deals are training deals of the opponent"
            )
            return 2
    label = "untrained" if args.untrained else f"{data['updates']:06d}"
    if opponent_data is not None:
        # The opponent's run folder, not its run name: the folder carries the date and time of the run,
        # so two runs of one name (one config, several seeds) give two files.
        label = f"{label}-vs-{opponent_path.resolve().parent.name}-{opponent_data['updates']:06d}"
        existing = path.parent / f"evaluation-{label}.json"
        if existing.exists() and not args.overwrite:
            # Seed, deals and tables are not in the name: another call for this pair would replace the file.
            print(f"{existing.name} exists next to the checkpoint: it is replaced only with --overwrite")
            return 2
    device = args.device
    if device == "auto":
        device = "cuda" if torch.cuda.is_available() else "cpu"
    net = load_network(data, device, args.untrained)
    make_agent = network_agent(net, cfg["model"], device, args.seed)

    if opponent_data is None:
        opponents = "random legal play"
        result = evaluate(
            make_agent,
            args.deals,
            args.tables,
            args.seed,
            args.threads,
            RandomLegal(args.seed + 1),
            args.records,
            training_seed=cfg["run"]["seed"],
        )
    else:
        # The opponent samples from its policy, as the agent does, from a generator of its own.
        opponent_net = load_network(opponent_data, device)
        opponents = (
            f"{run_name(opponent_data)}, {opponent_path.name} ({opponent_data['updates']} updates), samples from its policy"
        )
        result = evaluate(
            make_agent,
            args.deals,
            args.tables,
            args.seed,
            args.threads,
            records=args.records,
            training_seed=cfg["run"]["seed"],
            make_others=network_agent(opponent_net, opponent_data["config"]["model"], device, args.seed + 1),
            opponent_training_seed=opponent_data["config"]["run"]["seed"],
        )
        # "Beats random" belongs to the evaluation against random play. Here: where the interval lies.
        del result["beats_random"]
        result["above_zero"] = bool(result["low"] > 0)
        result["below_zero"] = bool(result["high"] < 0)
    records = result.pop("records")
    folder = path.parent / f"records-{label}"
    if opponent_data is not None and folder.is_dir():
        for stale in folder.glob("deal-*.json"):  # of the call that --overwrite replaces
            stale.unlink()
    if records:
        folder.mkdir(parents=True, exist_ok=True)
        for record in records:
            (folder / f"deal-{record['table']:04d}.json").write_text(record["json"], encoding="utf-8")
    result.update(
        {
            "checkpoint": None if args.untrained else path.name,
            "run": run_name(data),
            "updates": 0 if args.untrained else data["updates"],
            "fault": cfg["run"]["fault"],
            "opponents": opponents,
            # The opponent's checkpoint; `folder` is its run folder, which tells two runs of one name apart.
            "opponent": None
            if opponent_data is None
            else {
                "checkpoint": opponent_path.name,
                "run": run_name(opponent_data),
                "updates": opponent_data["updates"],
                "folder": opponent_path.resolve().parent.name,
            },
            "agent": "samples from its policy",
            "seed": args.seed,
            "tables": args.tables,
            "threads": args.threads,
            "device": device,
            "engine_sources": tarok_env.SOURCE_HASH,
            # The engine that played these deals, and the one the agent's checkpoint was made with
            # and the opponent's (None: before checkpoints held it, or no opponent). They differ only with --other-engine.
            "engine_version": tarok_env.ENGINE_VERSION,
            "checkpoint_engine_version": data.get("engine_version"),
            "opponent_engine_version": None if opponent_data is None else opponent_data.get("engine_version"),
        }
    )
    (path.parent / f"evaluation-{label}.json").write_text(json.dumps(result, indent=1), encoding="utf-8")

    who = "the untrained network" if args.untrained else f"{path.name} ({result['updates']} updates)"
    if opponent_data is None:
        print(f"{result['run']}, {who}: {args.deals} deals, each in four seatings, against random legal play")
    else:
        print(f"{result['run']}, {who}, in one seat: {args.deals} deals, each in four seatings, against three seats of {opponents}")
    print(
        f"mean reward per deal {result['mean']:+.4f}; 95% interval {result['low']:+.4f} to {result['high']:+.4f} "
        f"(reward 1 = 50 points of the paper score)"
    )
    print("by seat of the agent: " + ", ".join(f"{value:+.4f}" for value in result["mean_by_seat"]))
    if opponent_data is None:
        print("beats random: " + ("yes, the whole interval is above zero" if result["beats_random"] else "no"))
    elif result["above_zero"] or result["below_zero"]:
        print(f"the whole interval is {'above' if result['above_zero'] else 'below'} zero")
    else:
        print("the interval includes zero")
    if records:
        print(
            f"game records: {result['records_exported']} exported, "
            f"{result['records_replayed_equal']} replay to the same entries"
        )
    return 0


if __name__ == "__main__":
    sys.exit(main())
