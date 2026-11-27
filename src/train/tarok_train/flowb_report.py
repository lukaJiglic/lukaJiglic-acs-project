"""The numbers of spec 018 (first runs on deal flow B), as Markdown.

Run (cmd, from the repo root, venv active):

    python -m tarok_train.flowb_report --random-entropy --deals 32000 --seed 3018
    python -m tarok_train.flowb_report src\\train\\runs\\<run folder of the MLP> src\\train\\runs\\<run folder of the transformer>

`--random-entropy` (criterion 8): the policy entropy of uniform random legal play, per phase,
measured on the engine as the six values of D136 were: every seat plays uniform random legal
actions; a decision with n legal slots has the entropy ln(n); the value of a phase is the mean
over all its decisions, those with one legal slot included. The first deal of every table of
fresh environments of `--tables` tables, with the engine seeds `--seed`, `--seed` + 1, ...
`--json PATH` also writes the numbers to PATH.

With run folders (criteria 14, 15 and 19; 16 to 18 from the evaluation files next to the
checkpoints): reads the TensorBoard scalars, `run-info.json` and the evaluation files of each
folder. Prints; writes nothing.

The target rule of D136 (criterion 9, open point A of spec 018): a quarter of the entropy of
random legal play, rounded to 0.05 (to 0.01 under 0.1). `target_of` is that rule.
"""

from __future__ import annotations

import argparse
import json
import math
import sys
from pathlib import Path

import numpy as np

from . import contract as C

UPDATES = (1, 5, 10, 25, 50, 100, 150, 200)  # criterion 14
VALAT_LEVELS = (0.50, 0.20, 0.05)  # criterion 15
LAST = (151, 200)  # criterion 19: the updates over which the reading is taken
VALAT_LIMIT = 0.05  # criterion 19 (D149, B a)
SIZE_LIMIT = 2.0  # criterion 19 (D149, B a)
# Criterion 8, measured on 2026-11-25 with `--random-entropy --deals 32000 --seed 3018` on engine 0.3.0
# (sources fe9b13e811e50d2b): the entropy of uniform random legal play in the round of announcements.
# The same call gives the six values of D136 for the other phases: 0.87, 0.11, 1.39, 1.74, 2.43, 0.95.
ROUND_RANDOM_ENTROPY = 0.9150


# --------------------------------------------------------------------------
# Criterion 8: the entropy of random legal play
# --------------------------------------------------------------------------


def random_entropy(deals: int, seed: int, tables: int = 1000, threads: int = 6) -> dict:
    """Per phase: the mean of ln(number of legal slots) over the decisions of uniform random legal play.

    Returns {"deals", "seed", "tables", "phases": {name: {"entropy", "decisions", "forced_share"}}}.
    """
    from .env import EngineEnv
    from .evaluate import RandomLegal, chunk_seeds

    sums = np.zeros(len(C.PHASES))
    counts = np.zeros(len(C.PHASES))
    forced = np.zeros(len(C.PHASES))
    seeds = chunk_seeds(deals, tables, seed)
    for chunk, first in enumerate(range(0, deals, tables)):
        n = min(tables, deals - first)
        env = EngineEnv(n, seeds[chunk], threads)
        pick = RandomLegal(seeds[chunk])
        rows = np.arange(n)
        open_tables = np.ones(n, dtype=bool)
        while open_tables.any():
            obs = env.observe()
            live = np.flatnonzero(open_tables)
            legal = obs["mask"][live].sum(axis=1)
            start, width = C.PHASE_BLOCK
            phase = obs["state"][live, start : start + width].argmax(axis=1)
            np.add.at(sums, phase, np.log(legal))
            np.add.at(counts, phase, 1)
            np.add.at(forced, phase, legal == 1)
            done, _, _ = env.step(pick(rows, obs))
            open_tables &= ~done
    phases = {
        name: {"entropy": float(sums[i] / counts[i]), "decisions": int(counts[i]), "forced_share": float(forced[i] / counts[i])}
        for i, name in enumerate(C.PHASES)
        if counts[i]
    }
    return {"deals": deals, "seed": seed, "tables": tables, "phases": phases}


def target_of(entropy: float) -> float:
    """The target of D136's rule: a quarter of the entropy of random legal play, rounded to 0.05 (0.01 under 0.1)."""
    quarter = entropy / 4
    step = 0.01 if quarter < 0.1 else 0.05
    return round(round(quarter / step) * step, 2)


# --------------------------------------------------------------------------
# Criteria 12 and 14 to 19: the run folders and their evaluation files
# --------------------------------------------------------------------------

ROUND_TAGS = (
    *(f"round/announced_{name}" for name in ("trula", "kralji", "kralj_ultimo", "pagat_ultimo", "valat")),
    *(f"round/game_{name}" for name in ("none", "kontra", "re", "sub", "mort")),
    "round/bonus_kontra",
    "reward/abs_mean",
    "reward/abs_max",
    "reward/share_above_10",
    "round/decision_share",
    "round/decisions_per_seat_deal",
)
MEMORY_TAGS = (
    "gpu/reserved_after_acting_mib",
    "gpu/allocated_after_acting_mib",
    "gpu/reserved_after_update_mib",
    "gpu/allocated_after_update_mib",
)
FIXED = {  # the evaluation files of criteria 16 and 18, by what they hold
    "200 against silent-random": "evaluation-000200-vs-silent-random.json",
    "200 against random": "evaluation-000200.json",
    "untrained against silent-random": "evaluation-untrained-vs-silent-random.json",
}


def mean(values) -> float:
    values = list(values)
    return sum(values) / len(values)


def stays_under(series: dict, level: float):
    """The first update from which the value stays under `level` to the last update, or None."""
    first = None
    for update in sorted(series):
        if series[update] < level:
            if first is None:
                first = update
        else:
            first = None
    return first


def evaluation(path: Path) -> dict:
    """An evaluation file: its mean and interval, and the deals that would make the interval ±0.1."""
    data = json.loads(path.read_text(encoding="utf-8"))
    values = np.asarray(data["values"], dtype=np.float64)
    sd = float(values.std(ddof=1))
    return {
        "mean": data["mean"],
        "low": data["low"],
        "high": data["high"],
        "deals": data["deals"],
        "seed": data["seed"],
        "sd": sd,
        "deals_for_0.1": int(math.ceil((1.96 * sd / 0.1) ** 2)),
        "file": path.name,
    }


def read(folder: Path) -> dict:
    """The numbers of one run folder of spec 018."""
    from .smoke_report import scalars
    from . import trial_compare

    folder = Path(folder)
    series = {tag: dict(points) for tag, points in scalars(folder).items()}
    info = json.loads((folder / "run-info.json").read_text(encoding="utf-8"))
    summary_path = folder / "summary.json"
    summary = json.loads(summary_path.read_text(encoding="utf-8")) if summary_path.exists() else {}
    updates = max(series["loss/total"])
    contracts = sorted(tag for tag in series if tag.startswith("deals/contract_"))
    tags = (
        *ROUND_TAGS,
        *(f"entropy/{name}" for name in C.PHASES[:-1]),
        *(f"entropy_weight/{name}" for name in C.PHASES[:-1]),
        *contracts,
        "deals/declarer_wins",
        "time/seconds",
        *MEMORY_TAGS,
    )
    at = {update: {tag: series[tag].get(update) for tag in tags if tag in series} for update in UPDATES if update <= updates}

    valat = series["round/announced_valat"]
    reading = {"holds": None, "why": f"the run did not reach update {LAST[1]}"}
    if updates >= LAST[1]:
        span = range(LAST[0], LAST[1] + 1)
        valat_mean = mean(valat[update] for update in span)
        size_mean = mean(series["reward/abs_mean"][update] for update in span)
        reading = {
            "valat_mean": valat_mean,
            "size_mean": size_mean,
            "holds": bool(valat_mean < VALAT_LIMIT and size_mean < SIZE_LIMIT),
            "why": f"updates {LAST[0]} to {LAST[1]}: valat announced in {valat_mean:.1%} of deals (limit {VALAT_LIMIT:.0%}), "
            f"mean reward size {size_mean:.3f} (limit {SIZE_LIMIT})",
        }

    largest = max(series["reward/abs_max"].values())
    deal_sum = max(series["reward/deal_sum_abs_max"].values())
    checks = {
        "illegal_actions": max(series["env/illegal_actions"].values()),
        "deal_sum_abs_max": deal_sum,
        "largest_reward": largest,
        # D147: the four rewards of a deal add up to zero within 1e-6 times the largest reward (1e-6 up to 1).
        "deal_sum_within_tolerance": bool(deal_sum <= 1e-6 * max(1.0, largest)),
        "finished": summary.get("finished"),
        "failed": summary.get("failed"),
        "stopped_by_time_limit": summary.get("stopped_by_time_limit"),
    }

    evaluations = {name: evaluation(folder / file) for name, file in FIXED.items() if (folder / file).exists()}
    progress = None
    late = folder / f"evaluation-000200-vs-{folder.name}-000100.json"
    early = folder / f"evaluation-000100-vs-{folder.name}-000200.json"
    if late.exists() and early.exists():
        one, other = trial_compare.load(late), trial_compare.load(early)
        progress = {"one": evaluation(late), "other": evaluation(early), **trial_compare.paired(one, other)}
        progress["side"] = trial_compare.side(progress["one"], progress["other"])

    return {
        "folder": folder.name,
        "run": info["run"],
        "network": info["config"]["run"]["network"],
        "targets": info["config"]["entropy_control"]["targets"],
        "updates": updates,
        "at": at,
        "valat_stays_under": {level: stays_under(valat, level) for level in VALAT_LEVELS},
        "reading": reading,
        "checks": checks,
        "evaluations": evaluations,
        "progress": progress,
    }


def _cell(value, form="{:.3f}"):
    return "-" if value is None else form.format(value)


def report(runs) -> str:
    lines = []
    say = lines.append
    for run in runs:
        say(f"### {run['run']} (`{run['folder']}`), {run['updates']} updates")
        say("")
        checks = run["checks"]
        say(
            f"Loop checks: illegal actions {checks['illegal_actions']:.0f}; largest sum of a deal's four rewards "
            f"{checks['deal_sum_abs_max']:.2e} with a largest reward of {checks['largest_reward']:.1f} "
            f"({'within' if checks['deal_sum_within_tolerance'] else 'NOT within'} the tolerance of D147); "
            f"finished {checks['finished']}, failed {checks['failed']}, stopped by the time limit {checks['stopped_by_time_limit']}."
        )
        say("")
        updates = sorted(run["at"])
        head = "| Tag | " + " | ".join(str(update) for update in updates) + " |"
        say(head)
        say("|---|" + "---|" * len(updates))
        tags = sorted({tag for row in run["at"].values() for tag in row}, key=lambda tag: (not tag.startswith(("round/", "reward/")), tag))
        for tag in tags:
            form = "{:.0f}" if tag.startswith("gpu/") else "{:.2f}" if tag == "time/seconds" else "{:.3f}"
            say(f"| {tag} | " + " | ".join(_cell(run["at"][update].get(tag), form) for update in updates) + " |")
        say("")
        say(
            "Valat announced in under 50%, 20%, 5% of deals from update on (criterion 15): "
            + ", ".join(f"{level:.0%}: {_cell(update, '{}')}" for level, update in run["valat_stays_under"].items())
            + "."
        )
        reading = run["reading"]
        verdict = "not read" if reading["holds"] is None else ("holds" if reading["holds"] else "does not hold")
        say(f"Criterion 19, the flood ends by itself: {verdict}. {reading['why']}.")
        say("")
        if run["evaluations"]:
            say("| Evaluation (criteria 16, 18) | Seed | Deals | Mean | 95% interval | SD of a deal | Deals for +/-0.1 |")
            say("|---|---|---|---|---|---|---|")
            for name, result in run["evaluations"].items():
                say(
                    f"| {name} | {result['seed']} | {result['deals']} | {result['mean']:+.4f} | "
                    f"{result['low']:+.4f} to {result['high']:+.4f} | {result['sd']:.3f} | {result['deals_for_0.1']:,} |"
                )
            say("")
        progress = run["progress"]
        if progress:
            say("Progress inside the run (criterion 17), seed " + str(progress["one"]["seed"]) + ":")
            say(f"- update 200 in one seat among three of update 100: {progress['one']['mean']:+.4f} ({progress['one']['low']:+.4f} to {progress['one']['high']:+.4f})")
            say(f"- update 100 in one seat among three of update 200: {progress['other']['mean']:+.4f} ({progress['other']['low']:+.4f} to {progress['other']['high']:+.4f})")
            mean_, low, high = progress["half_difference"]
            say(f"- half the difference, paired over the deals: {mean_:+.4f} ({low:+.4f} to {high:+.4f}); correlation of the two directions {progress['correlation']:+.3f}")
            side = {"trial": "update 200 ahead", "base": "update 100 ahead", None: "no difference shown"}[progress["side"]]
            say(f"- the old rule (both intervals on one side of zero): {side}")
            say("")
    return "\n".join(lines)


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description="The numbers of spec 018 (first runs on deal flow B).")
    parser.add_argument("folders", nargs="*", help="run folders")
    parser.add_argument("--random-entropy", action="store_true", help="measure the entropy of random legal play per phase (criterion 8)")
    parser.add_argument("--deals", type=int, default=32000)
    parser.add_argument("--seed", type=int, default=3018)
    parser.add_argument("--tables", type=int, default=1000)
    parser.add_argument("--threads", type=int, default=6)
    parser.add_argument("--json", default=None, help="also write the numbers of --random-entropy to this file")
    args = parser.parse_args(argv)
    if args.random_entropy:
        result = random_entropy(args.deals, args.seed, args.tables, args.threads)
        print(f"Uniform random legal play, {result['deals']} deals (engine seeds from {result['seed']}, {result['tables']} tables per chunk).")
        print("| Phase | Entropy (nats) | Decisions | One legal slot | A quarter | Target (D136's rule) |")
        print("|---|---|---|---|---|---|")
        for name, row in result["phases"].items():
            print(f"| {name} | {row['entropy']:.4f} | {row['decisions']:,} | {row['forced_share']:.1%} | {row['entropy'] / 4:.4f} | {target_of(row['entropy'])} |")
        if args.json:
            Path(args.json).write_text(json.dumps(result, indent=1), encoding="utf-8")
        return 0
    if not args.folders:
        parser.error("give run folders, or --random-entropy")
    print(report([read(Path(folder)) for folder in args.folders]))
    return 0


if __name__ == "__main__":
    sys.exit(main())
