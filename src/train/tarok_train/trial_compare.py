"""Two checkpoints compared from their two evaluation files, one seat against three and the other way round.

Run (cmd, from the repo root, venv active), after both evaluations of `tarok_train.evaluate --opponent`:

    python -m tarok_train.trial_compare src\\train\\runs\\<run folder of X> src\\train\\runs\\<run folder of Y>
    python -m tarok_train.trial_compare <X> <Y> --updates 2000 --updates-y 200
    python -m tarok_train.trial_compare <X> <Y> --sub seed-5252

Reads `evaluation-NNNNNN-vs-<run folder of Y>-MMMMMM.json` next to X and its mirror next to Y.
Prints; writes nothing. Used for spec 017 (criteria 12 to 14).

The two directions:
- "one": one seat of X among three seats of Y, mean reward per deal of X's seat;
- "other": one seat of Y among three seats of X.

Both are played on the same deals (same seed, same number of deals, same chunk size), so the
two per-deal values of a deal belong together. From them, per deal:
- half the difference, (one - other) / 2;
- half the sum, (one + other) / 2: what the single seat gets whichever of the two it is.

Their 95% intervals are taken over these per-deal numbers (paired). Treating the two directions
as independent gives a wrong interval: the two values of a deal are correlated.

What half the difference means. If the single seat's handicap were the same in both directions,
half the difference would be X's lead over Y with the handicap taken out. Nothing here shows
that it is the same in both directions. Read it as a summary of the two numbers, not as a lead.

The rule of spec 017, criterion 14 (`reading`): "trial ahead" when both intervals lie on the
trial's side of zero, one above and one below, in enough of the pairs; "trial behind" is the
mirror; everything else is "no difference shown".
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

import numpy as np

from .evaluate import interval


def load(path: Path) -> dict:
    return json.loads(Path(path).read_text(encoding="utf-8"))


def result_file(x: Path, y: Path, updates_x: int, updates_y: int, sub: str | None = None) -> Path:
    """Where `evaluate --opponent` put X (one seat) against Y (three seats). `sub`: a subfolder of X's run folder."""
    folder = Path(x) if sub is None else Path(x) / sub
    return folder / f"evaluation-{updates_x:06d}-vs-{Path(y).name}-{updates_y:06d}.json"


def paired(one: dict, other: dict) -> dict:
    """Half the difference and half the sum of the two directions, each as (mean, low, high), paired over the deals."""
    for key in ("seed", "deals", "tables"):
        if one.get(key) != other.get(key):
            raise ValueError(f"the two files are not the same deals: {key} is {one.get(key)} in one and {other.get(key)} in the other")
    # The same game too (spec 009): a file without the key was written on engine 0.2.0.
    engines = [result.get("engine_version", "0.2.0") for result in (one, other)]
    if engines[0] != engines[1]:
        raise ValueError(f"the two files are not the same game: engine version {engines[0]} in one and {engines[1]} in the other")
    a, b = np.asarray(one["values"], dtype=np.float64), np.asarray(other["values"], dtype=np.float64)
    if len(a) != len(b):
        raise ValueError(f"the two files are not the same deals: {len(a)} and {len(b)} values")
    return {
        "half_difference": interval((a - b) / 2),
        "half_sum": interval((a + b) / 2),
        "correlation": float(np.corrcoef(a, b)[0, 1]),
    }


def side(one: dict, other: dict):
    """On whose side of zero both intervals lie: "trial" (X), "base" (Y) or None.

    one: one trial seat among three base seats; other: one base seat among three trial seats.
    """
    if one["low"] > 0 and other["high"] < 0:
        return "trial"
    if one["high"] < 0 and other["low"] > 0:
        return "base"
    return None


def largest(results) -> float:
    """The largest difference of a set of comparisons: the largest absolute mean (criterion 13)."""
    return max(abs(result["mean"]) for result in results)


def reading(pairs, needed: int, chance: float | None = None) -> str:
    """The rule of criterion 14 for one network.

    pairs: (one trial seat among three base seats, one base seat among three trial seats) per seed.
    needed: in how many pairs both intervals must lie on one arm's side. chance: if given, both
    differences of a counted pair must also be larger than it (the MLP's sentence of the rule).
    """

    def counted(one, other, which):
        if side(one, other) != which:
            return False
        return chance is None or min(abs(one["mean"]), abs(other["mean"])) > chance

    ahead = sum(counted(one, other, "trial") for one, other in pairs)
    behind = sum(counted(one, other, "base") for one, other in pairs)
    if ahead >= needed:
        return "trial ahead"
    if behind >= needed:
        return "trial behind"
    return "no difference shown"


def compare(x: Path, y: Path, updates_x: int = 500, updates_y: int | None = None, sub: str | None = None) -> dict:
    """Both directions of X against Y from their files, with the paired numbers."""
    x, y = Path(x), Path(y)
    updates_y = updates_x if updates_y is None else updates_y
    one = load(result_file(x, y, updates_x, updates_y, sub))
    other = load(result_file(y, x, updates_y, updates_x, sub))
    for result, opponent in ((one, y), (other, x)):
        if result["opponent"]["folder"] != opponent.name:
            raise ValueError(f"the file names the opponent's run folder {result['opponent']['folder']}, not {opponent.name}")
    return {"one": one, "other": other, "seed": one["seed"], "deals": one["deals"], **paired(one, other)}


def cell(result) -> str:
    """A mean with its interval, from an evaluation file or from an (mean, low, high) of `paired`."""
    mean, low, high = (result["mean"], result["low"], result["high"]) if isinstance(result, dict) else result
    return f"{mean:+.4f} ({low:+.4f} to {high:+.4f})"


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description="Two checkpoints from their two evaluation files (spec 017).")
    parser.add_argument("x", help="run folder of X")
    parser.add_argument("y", help="run folder of Y")
    parser.add_argument("--updates", type=int, default=500, help="the checkpoint of X (and of Y, unless --updates-y is given)")
    parser.add_argument("--updates-y", type=int, default=None, help="the checkpoint of Y")
    parser.add_argument("--sub", default=None, help="a subfolder of each run folder that holds the files, for example seed-5252")
    args = parser.parse_args(argv)
    x, y = Path(args.x), Path(args.y)
    n = compare(x, y, args.updates, args.updates_y, args.sub)
    print(f"X: {x.name}; Y: {y.name}; {n['deals']} deals in four seatings, seed {n['seed']}. Mean reward per deal, 95% interval.")
    print(f"One seat of X among three of Y: {cell(n['one'])}")
    print(f"One seat of Y among three of X: {cell(n['other'])}")
    print(f"Half the difference, paired over the deals: {cell(n['half_difference'])}")
    print(f"Half the sum, paired over the deals: {cell(n['half_sum'])}")
    print(f"Correlation of the two directions over the deals: {n['correlation']:+.3f}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
