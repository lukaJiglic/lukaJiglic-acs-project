"""The numbers of a run for spec 017 (the trial of the entropy controller's lower limit), as Markdown.

Run (cmd, from the repo root, venv active):

    python -m tarok_train.trial_report src\\train\\runs\\<run folder> [more run folders]
    python -m tarok_train.trial_report --at 500 src\\train\\runs\\<run folder of a longer run>

Reads the TensorBoard scalars and `run-info.json` of each folder. Prints; writes nothing.
Per run (spec 017, criteria 7 to 10, and the loop checks of criterion 4):
- the entropy of the play and its weight at the updates 100, 200, ..., and the lowest weight it reached;
- when the play's weight first went under 0.01, the lower limit of spec 014;
- every phase whose weight went under 0.01, with its lowest value;
- the entropy of every phase at the last update beside its target, and the phases outside the band;
- the contract shares and the share of deals the declarer's side won at the last update;
- the same as means over the last 100 updates (one update is a few hundred deals and swings);
- the lowest entropy of every phase over the run, and how long it was under the lower edge of its band.

What a logged weight is: `entropy_weight/<phase>` at update N is the weight the controller set
after update N, from that update's entropy. Update N + 1 trains with it.

`--at N` reads a run as if it had ended at update N: for the run of spec 014, whose checkpoint
of update 500 is the base arm of the transformer. The comparison of two checkpoints is in
`tarok_train/trial_compare.py`.
"""

from __future__ import annotations

import argparse
import json
import math
import sys
from pathlib import Path

from . import contract as C
from .smoke_report import scalars

OLD_LIMIT = 0.01  # [entropy_control] min_weight of spec 014
# The logged weights are 32-bit numbers: 0.01 is stored as 0.0099999998. "Under the limit" is clearly under it.
UNDER = OLD_LIMIT * 0.999
EVERY = 100  # the play is reported at every hundredth update
WINDOW = 100  # means are taken over this many last updates
MONITORS = (
    "entropy_choice/play",
    "forced_share/play",
    "deals/declarer_wins",
    "critic/explained_variance",
    "aux/card_accuracy",
    "ppo/approx_kl",
    "entropy/phases_outside_band",
)


def mean(values) -> float:
    values = list(values)
    return sum(values) / len(values)


def read(folder: Path, at: int | None = None, window: int = WINDOW) -> dict:
    """The numbers of one run folder.

    at: read only the updates up to this one. window: the means of "window" are taken over this
    many last updates (fewer if the run is shorter).
    """
    folder = Path(folder)
    data = scalars(folder)
    if at is not None:
        data = {tag: [(step, value) for step, value in points if step <= at] for tag, points in data.items()}
        data = {tag: points for tag, points in data.items() if points}
    info = json.loads((folder / "run-info.json").read_text(encoding="utf-8"))
    control = info["config"]["entropy_control"]
    floor = control["min_weight"]
    band = control["band"]
    series = {tag: dict(points) for tag, points in data.items()}
    updates = max(series["loss/total"])
    names = [name for name in C.PHASES[:-1] if f"entropy/{name}" in series]

    def last(tag):
        return series[tag][updates]

    def in_band(name, entropy):
        target = control["targets"][name]
        return target * (1 - band) <= entropy <= target * (1 + band)

    play = [
        {"update": update, "entropy": series["entropy/play"][update], "weight": series["entropy_weight/play"][update]}
        for update in range(EVERY, updates + 1, EVERY)
    ]
    under = {}
    for name in names:
        lowest = min(series[f"entropy_weight/{name}"].values())
        if lowest < UNDER:
            under[name] = lowest
    first_under = next((step for step in sorted(series["entropy_weight/play"]) if series["entropy_weight/play"][step] < UNDER), None)
    phases = {}
    for name in names:
        if updates not in series[f"entropy/{name}"]:
            continue  # the phase had no decision in the last update
        entropy = last(f"entropy/{name}")
        phases[name] = {"entropy": entropy, "target": control["targets"][name], "weight": last(f"entropy_weight/{name}"), "in_band": in_band(name, entropy)}

    def contracts(value_of):
        return {tag.removeprefix("deals/contract_").replace("_", " "): value_of(tag) for tag in sorted(series) if tag.startswith("deals/contract_")}

    first = max(1, updates - window + 1)
    span = [update for update in range(first, updates + 1) if update in series["loss/total"]]

    def over(tag):
        return mean(series[tag][update] for update in span if update in series[tag])

    window_phases = {}
    for name in names:
        weights = [series[f"entropy_weight/{name}"][update] for update in span if update in series[f"entropy_weight/{name}"]]
        window_phases[name] = {
            "entropy": over(f"entropy/{name}"),
            "weight": mean(weights),
            # under 0.01, the lower limit of spec 014: the room only the trial arm has
            "under_old_limit": sum(weight < UNDER for weight in weights) / len(weights),
            # at the lower limit of this run: the controller asks for less and cannot
            "at_lower_limit": sum(weight <= floor * 1.001 for weight in weights) / len(weights),
        }
    dips = {}
    for name in names:
        entropies = series[f"entropy/{name}"]
        edge = control["targets"][name] * (1 - band)
        lowest_at = min(entropies, key=lambda update: (entropies[update], update))
        below = [update for update in sorted(entropies) if entropies[update] < edge]
        dips[name] = {"lowest": entropies[lowest_at], "lowest_at": lowest_at, "under_edge": len(below), "last_under_edge": below[-1] if below else None, "edge": edge}

    not_finite = sum(1 for points in data.values() for _, value in points if not math.isfinite(value))
    return {
        "run": info["run"],
        "folder": folder.name,
        "seed": info["config"]["run"]["seed"],
        "min_weight": floor,
        "band": band,
        "updates": updates,
        "play": play,
        "play_lowest_weight": min(series["entropy_weight/play"].values()),
        "play_first_under_old_limit": None if first_under is None else {"set_after_update": first_under, "used_from_update": first_under + 1},
        "under_old_limit": under,
        "phases": phases,
        # Counted here from the entropies and the band. The trainer logs its own count; the two must agree.
        "phases_outside_band": sum(not row["in_band"] for row in phases.values()),
        "phases_outside_band_logged": int(last("entropy/phases_outside_band")),
        "contracts": contracts(last),
        "declarer_wins": last("deals/declarer_wins"),
        "window": {
            "first": first,
            "last": updates,
            "updates": len(span),
            "deals_per_update": over("deals/count") if "deals/count" in series else None,
            "phases": window_phases,
            "contracts": contracts(over),
            "monitors": {tag: over(tag) for tag in MONITORS if tag in series},
        },
        "dips": dips,
        "checks": {
            "illegal_actions": max(series["env/illegal_actions"].values()),
            "deal_sum_abs_max": max(series["reward/deal_sum_abs_max"].values()),
            "decision_age_max": max(series["rollout/decision_age_max"].values()),
            "not_finite": not_finite,
        },
    }


def report(folder: Path, at: int | None = None, window: int = WINDOW) -> str:
    n = read(folder, at, window)
    w = n["window"]
    lines = [
        f"**Run `{n['folder']}`** ({n['run']}, seed {n['seed']}, lower limit {n['min_weight']:g}, {n['updates']} updates read)",
        "",
        "The weight in a column is the one set after that update; the next update trains with it.",
        "",
        "| Update | " + " | ".join(str(row["update"]) for row in n["play"]) + " |",
        "|---|" + "---|" * len(n["play"]),
        "| Entropy of the play | " + " | ".join(f"{row['entropy']:.3f}" for row in n["play"]) + " |",
        "| Weight of the play | " + " | ".join(f"{row['weight']:.4f}" for row in n["play"]) + " |",
        "",
    ]
    first = n["play_first_under_old_limit"]
    lines.append(
        f"Lowest weight of the play: {n['play_lowest_weight']:.4f}. "
        + (
            f"A weight under {OLD_LIMIT:g} was first set after update {first['set_after_update']} and used from update {first['used_from_update']}."
            if first is not None
            else f"It never went under {OLD_LIMIT:g}."
        )
    )
    if n["under_old_limit"]:
        lines.append("Phases whose weight went under 0.01, with the lowest value: " + ", ".join(f"{name} {value:.4f}" for name, value in n["under_old_limit"].items()) + ".")
    else:
        lines.append(f"No phase's weight went under {OLD_LIMIT:g}.")
    span = f"{w['first']} to {w['last']}"
    lines += [
        "",
        f"| Phase | Entropy at update {n['updates']} | Target | In the band | Weight | Entropy, mean over {span} | Weight, mean | Share of updates with the weight under 0.01 | Share at the lower limit | Lowest entropy of the run (at update) | Updates under the band's lower edge (last) |",
        "|---|---|---|---|---|---|---|---|---|---|---|",
    ]
    for name, row in n["phases"].items():
        mid, dip = w["phases"][name], n["dips"][name]
        lines.append(
            f"| {name} | {row['entropy']:.3f} | {row['target']:g} | {'yes' if row['in_band'] else 'no'} | {row['weight']:.4f} | {mid['entropy']:.3f} | {mid['weight']:.4f} | "
            f"{mid['under_old_limit']:.0%} | {mid['at_lower_limit']:.0%} | {dip['lowest']:.3f} ({dip['lowest_at']}) | {dip['under_edge']} ({dip['last_under_edge'] if dip['last_under_edge'] is not None else '-'}) |"
        )
    lines.append("")
    agree = "" if n["phases_outside_band"] == n["phases_outside_band_logged"] else f" (the trainer logged {n['phases_outside_band_logged']}: the two counts differ)"
    lines.append(f"Phases outside the band at update {n['updates']}: {n['phases_outside_band']}{agree}.")
    deals = f"; one update is {w['deals_per_update']:.0f} deals on average" if w["deals_per_update"] else ""
    lines.append(
        f"Contracts at update {n['updates']}: " + ", ".join(f"{name} {share:.1%}" for name, share in n["contracts"].items()) + f". The declarer's side won {n['declarer_wins']:.1%} of the deals."
    )
    lines.append(f"Contracts, mean over updates {span}: " + ", ".join(f"{name} {share:.1%}" for name, share in w["contracts"].items()) + deals + ".")
    lines.append(f"Monitors, mean over updates {span}: " + ", ".join(f"{tag} {value:.4f}" for tag, value in w["monitors"].items()) + ".")
    checks = n["checks"]
    lines.append(
        f"Loop checks over the updates read: illegal actions {checks['illegal_actions']:.0f}; largest sum of a deal's rewards {checks['deal_sum_abs_max']:.1e}; "
        f"oldest trained decision {checks['decision_age_max']:.0f}; logged numbers that are not finite {checks['not_finite']}."
    )
    return "\n".join(lines)


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description="The numbers of a run for spec 017, as Markdown.")
    parser.add_argument("folders", nargs="+", help="run folders under src\\train\\runs")
    parser.add_argument("--at", type=int, default=None, help="read the run only up to this update")
    parser.add_argument("--window", type=int, default=WINDOW, help="means are taken over this many last updates")
    parser.add_argument("--json", action="store_true", help="print the numbers as JSON instead of Markdown")
    args = parser.parse_args(argv)
    for folder in args.folders:
        if args.json:
            print(json.dumps(read(Path(folder), args.at, args.window)))
        else:
            print(report(Path(folder), args.at, args.window))
            print()
    return 0


if __name__ == "__main__":
    sys.exit(main())
