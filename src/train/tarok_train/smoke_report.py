"""The numbers of a run folder for the Results of spec 008, as Markdown.

Run (cmd, from the repo root, venv active):

    python -m tarok_train.smoke_report src\\train\\runs\\<run folder>

Reads the TensorBoard scalars of every update, `run-info.json`, `summary.json` and any
`evaluation-*.json` of the folder. Prints; writes nothing.
"""

from __future__ import annotations

import argparse
import json
import math
import sys
from pathlib import Path

from . import contract as C

OLD_G3 = 10_000  # decisions per second, the goal before D128
COLLAPSE = 0.05  # spec 008, criterion 13: a phase under this entropy at the end is listed as a collapse


def scalars(folder: Path) -> dict:
    """tag -> list of (update, value), from the TensorBoard files of the folder."""
    from tensorboard.backend.event_processing.event_accumulator import EventAccumulator

    accumulator = EventAccumulator(str(folder), size_guidance={"scalars": 0})
    accumulator.Reload()
    return {tag: [(event.step, event.value) for event in accumulator.Scalars(tag)] for tag in accumulator.Tags()["scalars"]}


def _load(path: Path):
    return json.loads(path.read_text(encoding="utf-8")) if path.exists() else None


def report(folder: Path) -> str:
    data = scalars(folder)
    info, summary = _load(folder / "run-info.json"), _load(folder / "summary.json")
    lines = []

    def say(text=""):
        lines.append(text)

    def series(tag):
        return [value for _, value in data.get(tag, [])]

    def first_last(tag, form="{:.3f}"):
        values = series(tag)
        return (form.format(values[0]), form.format(values[-1])) if values else ("-", "-")

    updates = len(series("loss/total"))
    say(f"**Run `{folder.name}`**")
    say()
    if info:
        dirty = info["git_status_short"].splitlines()
        say(f"- Started {info['started']}, commit `{info['commit']}`, {len(dirty)} uncommitted paths in `git status --short`.")
        say(f"- Config `{info['config_file']}`, run name `{info['run']}`, fault `{info['config']['run']['fault']}`, engine sources `{info['engine_sources']}`.")
    if summary:
        state = "finished" if summary["finished"] else f"NOT finished (failed: {summary['failed']}; time limit: {summary['stopped_by_time_limit']})"
        say(f"- {summary['updates']} of {summary['updates_wanted']} updates in {summary['seconds'] / 60:.1f} minutes on {summary['device']}: {state}.")
    say()

    bad = [(tag, step, value) for tag, points in data.items() for step, value in points if not math.isfinite(value)]
    say("| Check | Value |")
    say("|---|---|")
    say(f"| Updates logged | {updates} |")
    say(f"| Illegal actions refused by the engine (criterion 2) | {max(series('env/illegal_actions'), default=float('nan')):.0f} |")
    say(f"| Logged numbers that are not finite (criterion 3) | {len(bad)} of {sum(len(points) for points in data.values())} |")
    say(f"| Largest sum of a deal's four rewards (criterion 4) | {max(series('reward/deal_sum_abs_max'), default=float('nan')):.1e} |")
    say(f"| Oldest trained decision, in updates (criterion 5) | {max(series('rollout/decision_age_max'), default=float('nan')):.0f} |")
    if summary:
        say(f"| Decisions played per second (criterion 9; old G3: {OLD_G3:,}) | {summary['played_per_second']:,.0f} |")
        say(f"| Decisions trained per second | {summary['trained_per_second']:,.0f} |")
        say(f"| Share of played decisions trained on (all seats' decisions count as played) | {summary['trained_share']:.1%} |")
        say(f"| The same against the decisions of the current network's seats only | {summary['trained_share_own_seats']:.1%} |")
        # An empty reading: nvidia-smi gave nothing the trainer could read (spec 015).
        say(f"| GPU before | {summary['gpu_before'] or 'not read'} |")
        say(f"| GPU after | {summary['gpu_after'] or 'not read'} |")
        split = ", ".join(f"{name} {share:.1%}" for name, share in summary["time_shares"].items())
        say(f"| Time split (criterion 10) | {split}; together {summary['time_shares_sum']:.1%} |")
        say(f"| Peak GPU memory of the run's tensors (criterion 11; `max_memory_allocated`) | {summary['peak_gpu_memory_mib']} MiB |")
        say(f"| Peak GPU memory held by PyTorch's allocator (`max_memory_reserved`) | {summary['peak_gpu_memory_reserved_mib']} MiB |")
        used = (summary["gpu_after"] or {}).get("memory_used_mib")
        say(f"| Memory in use on the whole card at the end, all programs (nvidia-smi) | {'not read' if used is None else f'{used} MiB'} |")
    for what in ("decisions", "tokens"):
        mean = series(f"rollout/{what}_per_seat_deal_mean")
        largest = series(f"rollout/{what}_per_seat_deal_max")
        if mean:
            say(f"| {what.capitalize()} per seat-deal (criterion 12) | mean {sum(mean) / len(mean):.1f}, largest {max(largest):.0f} |")
    say()

    say("| Monitor (criteria 13 and 14) | First update | Last update |")
    say("|---|---|---|")
    control = info["config"]["entropy_control"] if info else None
    collapsed, under_band = [], []
    for name in C.PHASES[:-1]:
        values = series(f"entropy/{name}")
        if values:
            say(f"| Entropy, {name} | {values[0]:.3f} | {values[-1]:.3f} |")
            if values[-1] < COLLAPSE:
                collapsed.append(name)
            if control:
                edge = control["targets"][name] * (1 - control["band"])
                if values[-1] < edge:
                    under_band.append(f"{name} ({values[-1]:.3f} under {edge:.3f})")
            first, last = first_last(f"entropy_choice/{name}")
            say(f"| Entropy over the decisions with two or more legal slots, {name} | {first} | {last} |")
            first, last = first_last(f"forced_share/{name}", "{:.1%}")
            say(f"| Share of decisions with one legal slot, {name} | {first} | {last} |")
            first, last = first_last(f"entropy_weight/{name}", "{:.4f}")
            say(f"| Entropy weight, {name} | {first} | {last} |")
    for tag in sorted(tag for tag in data if tag.startswith("deals/contract_")):
        first, last = first_last(tag, "{:.1%}")
        say(f"| Contract {tag.removeprefix('deals/contract_').replace('_', ' ')} | {first} | {last} |")
    for tag, label in (
        ("deals/forehand_forced", "Three passes before forehand"),
        ("deals/forehand_forced_tri", "Forehand's forced tri"),
        ("deals/declarer_wins", "Declarer's side wins"),
        ("aux/partner_accuracy", "Partner head, accuracy"),
        ("aux/partner_accuracy_hidden", "Partner head, accuracy while the partner is not public"),
        ("aux/partner_guess", "Partner, uniform guess over the seats still possible"),
        ("aux/card_accuracy", "Card-location head, accuracy"),
        ("aux/card_guess", "Card location, uniform guess over the places still possible"),
    ):
        first, last = first_last(tag, "{:.1%}")
        say(f"| {label} | {first} | {last} |")
    say()
    say(f"Collapse by the spec's fixed limit (entropy under {COLLAPSE} at the last update): {', '.join(collapsed) if collapsed else 'none'}.")
    if control:
        say(
            f"Collapse relative to the phase's target (entropy at the last update under the lower edge of its band, "
            f"target x {1 - control['band']:.2f}): {', '.join(under_band) if under_band else 'none'}."
        )
        targets = ", ".join(f"{name} {control['targets'][name]:g}" for name in C.PHASES[:-1] if series(f"entropy/{name}"))
        say(f"Targets: {targets}. The fixed limit is above the target of a phase whose target is under {COLLAPSE / (1 - control['band']):.2f}.")

    for path in sorted(folder.glob("evaluation-*.json")):
        result = _load(path)
        who = "untrained network" if result["checkpoint"] is None else f"`{result['checkpoint']}`"
        say()
        if "beats_random" in result:
            verdict = f"Beats random: {'yes' if result['beats_random'] else 'no'}."
        elif result["above_zero"]:  # a checkpoint as the opponent (spec 015)
            verdict = "The whole interval is above zero."
        elif result["below_zero"]:
            verdict = "The whole interval is below zero."
        else:
            verdict = "The interval includes zero."
        say(
            f"Evaluation, {who}: {result['deals']} deals in four seatings against {result['opponents']}, seed {result['seed']}. "
            f"Mean reward per deal {result['mean']:+.4f}, 95% interval {result['low']:+.4f} to {result['high']:+.4f}. "
            f"{verdict} "
            f"Records: {result['records_replayed_equal']} of {result['records_exported']} replay to the same entries."
        )
    return "\n".join(lines)


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description="The numbers of a run folder for spec 008, as Markdown.")
    parser.add_argument("folder", help="a run folder under src\\train\\runs")
    args = parser.parse_args(argv)
    print(report(Path(args.folder)))
    return 0


if __name__ == "__main__":
    sys.exit(main())
