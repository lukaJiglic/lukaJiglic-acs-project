"""A checkpoint against random legal play on seeded deals (spec 008, criteria 7, 8 and 15).

Run (cmd, from the repo root, venv active):

    python -m tarok_train.evaluate --checkpoint src\\train\\runs\\<run folder>\\checkpoint-000200.pt --deals 2000 --seed 2008 --records 100
    python -m tarok_train.evaluate --checkpoint <the same> --deals 2000 --seed 2008 --untrained

Every deal is played four times: with the agent in seat 0, 1, 2 and 3, and random
legal play in the other three seats. The cards of a deal depend only on the seed and
the deal's number, so the four seatings see the same cards. The value of a deal is
the mean of the agent's four rewards. Reported: the mean over the deals and its 95%
interval (mean plus and minus 1.96 standard errors over the deals). The agent "beats
random" when the lower end of the interval is above zero.

The agent samples from its policy, as in training. With `--untrained` the network
of the checkpoint's config is built with the run's seed and no weights are loaded:
the same network before its first update.

`--records N` exports the first N deals (agent in seat 0) as game records of spec 006
and replays each one: the replay must give the entries the environment reported.

This is a test of the wiring, not a measure of playing strength.
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
    with its own acting cache (the transformer) and its own seeded generator.
    """
    device = torch.device(device)

    def make_agent(n_tables: int):
        actor = Actor(net, None, n_tables, device, cfg_model)
        generator = torch.Generator(device=device).manual_seed(seed)

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


def play_seating(env, agent_seat: int, agent, others, n_record: int = 0) -> Seating:
    """Play the first deal of every table of a fresh `env`.

    agent(rows, obs) and others(rows, obs) return one legal slot for each of the tables
    `rows`. The agent is asked only where the seat to act is `agent_seat`. A table whose
    first deal has ended goes on with `others` in every seat until all are done; those
    later deals are not counted.
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
        mine = np.flatnonzero(open_tables & (obs["seat"] == agent_seat))
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
    make_agent, deals: int, tables: int, seed: int, threads: int, others, records: int = 0, training_seed=None
) -> dict:
    """`deals` seeded deals, each in four seatings.

    make_agent(n_tables) gives the agent of one environment; `others` plays the other
    seats. The deals are played in chunks of `tables` tables; chunk c uses the engine
    seed `seed + c`, so deal number d is table d mod `tables` of chunk d // `tables`,
    its first deal. `records`: export that many deals of the first chunk (agent in
    seat 0) as game records and replay them. `training_seed`: the seed of the run that
    made the agent; no chunk may use it, or its deals would be deals the agent trained on.
    """
    seeds = chunk_seeds(deals, tables, seed)
    if training_seed is not None and training_seed in seeds:
        raise ValueError(
            f"the evaluation would play a chunk of deals with the engine seed {training_seed}, the seed of the "
            f"training run (its chunks use the seeds {seeds[0]} to {seeds[-1]}): choose a seed outside that range"
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
            seating = play_seating(env, seat, make_agent(n), others, n_record)
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


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description="A checkpoint against random legal play on seeded deals (spec 008).")
    parser.add_argument("--checkpoint", required=True, help="a checkpoint of tarok_train.train")
    parser.add_argument("--deals", type=int, required=True, help="number of deals; each is played in four seatings")
    parser.add_argument("--seed", type=int, required=True, help="seed of the deals; not the seed of the training run")
    parser.add_argument("--tables", type=int, default=500, help="deals played at the same time (memory of the acting cache)")
    parser.add_argument("--threads", type=int, default=6, help="threads of the engine")
    parser.add_argument("--device", default="auto", help="cuda, cpu or auto")
    parser.add_argument("--records", type=int, default=0, help="export this many deals as game records and replay them")
    parser.add_argument("--untrained", action="store_true", help="the same network before its first update")
    args = parser.parse_args(argv)

    path = Path(args.checkpoint)
    data = torch.load(path, map_location="cpu", weights_only=True)
    if data.get("encoding_version") != C.ENCODING_VERSION:
        print(f"the checkpoint was made for encoding version {data.get('encoding_version')}, this code reads {C.ENCODING_VERSION}")
        return 2
    cfg = data["config"]
    if cfg["run"]["seed"] in chunk_seeds(args.deals, args.tables, args.seed):
        print(
            f"the evaluation would use the seed of the training run ({cfg['run']['seed']}) for one of its chunks of "
            "deals: choose a seed outside that range, or the deals are training deals"
        )
        return 2
    device = args.device
    if device == "auto":
        device = "cuda" if torch.cuda.is_available() else "cpu"
    torch.manual_seed(cfg["run"]["seed"])  # the trainer builds its network first, from this seed
    net = build_network(cfg["model"], data["network"])
    if not args.untrained:
        net.load_state_dict(data["model"])
    net = net.to(device).eval()

    result = evaluate(
        network_agent(net, cfg["model"], device, args.seed),
        args.deals,
        args.tables,
        args.seed,
        args.threads,
        RandomLegal(args.seed + 1),
        args.records,
        training_seed=cfg["run"]["seed"],
    )
    label = "untrained" if args.untrained else f"{data['updates']:06d}"
    records = result.pop("records")
    if records:
        folder = path.parent / f"records-{label}"
        folder.mkdir(parents=True, exist_ok=True)
        for record in records:
            (folder / f"deal-{record['table']:04d}.json").write_text(record["json"], encoding="utf-8")
    result.update(
        {
            "checkpoint": None if args.untrained else path.name,
            "run": f"{cfg['run']['name']}-{data['network']}",
            "updates": 0 if args.untrained else data["updates"],
            "fault": cfg["run"]["fault"],
            "opponents": "random legal play",
            "agent": "samples from its policy",
            "seed": args.seed,
            "device": device,
            "engine_sources": tarok_env.SOURCE_HASH,
        }
    )
    (path.parent / f"evaluation-{label}.json").write_text(json.dumps(result, indent=1), encoding="utf-8")

    who = "the untrained network" if args.untrained else f"{path.name} ({result['updates']} updates)"
    print(f"{result['run']}, {who}: {args.deals} deals, each in four seatings, against random legal play")
    print(
        f"mean reward per deal {result['mean']:+.4f}; 95% interval {result['low']:+.4f} to {result['high']:+.4f} "
        f"(reward 1 = 50 points of the paper score)"
    )
    print("by seat of the agent: " + ", ".join(f"{value:+.4f}" for value in result["mean_by_seat"]))
    print("beats random: " + ("yes, the whole interval is above zero" if result["beats_random"] else "no"))
    if records:
        print(
            f"game records: {result['records_exported']} exported, "
            f"{result['records_replayed_equal']} replay to the same entries"
        )
    return 0


if __name__ == "__main__":
    sys.exit(main())
