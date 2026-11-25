"""Tests for spec 017 (trial: the entropy controller's lower limit at 0.001).

Criteria 1 and 2: the configs of the two arms, key by key, and the seed of every run.
The runs and the comparisons of criteria 3 to 13 are not tests. The later tests are for the
two scripts that make every number in the tables of the Results: `tarok_train/trial_report.py`
(a run folder: criteria 7 to 10) and `tarok_train/trial_compare.py` (two evaluation files of
one pair of checkpoints: criteria 12 to 14, and the two directions taken together).
"""

import json
import math
import re
from pathlib import Path

import numpy as np
import pytest
import torch

from tarok_train import contract as C
from tarok_train import train

from test_spec008 import engine

CONFIGS = Path(__file__).resolve().parents[1] / "configs"
SEEDS = (14, 15, 16)
ARMS = {"base": 0.01, "low": 0.001}  # arm -> the lower limit of the entropy weight
LONG = {"mlp": CONFIGS / "long-a-mlp.toml", "transformer": CONFIGS / "long-a-transformer.toml"}


def trial_file(arm: str, network: str, seed: int = 14) -> Path:
    """The config of one run. Seed 14 is the file spec 017 names; 15 and 16 have a file each."""
    suffix = "" if seed == 14 else f"-s{seed}"
    return CONFIGS / f"trial-floor-{arm}-{network}{suffix}.toml"


def flat(cfg: dict) -> dict:
    """'section.key' -> value, with the targets of the controller as 'entropy_control.targets.<phase>'."""
    out = {}
    for section, table in cfg.items():
        for key, value in table.items():
            if isinstance(value, dict):
                out.update({f"{section}.{key}.{name}": item for name, item in value.items()})
            else:
                out[f"{section}.{key}"] = value
    return out


def differences(a: dict, b: dict) -> set:
    """The keys in which two configs differ, a key in only one of them included."""
    a, b = flat(a), flat(b)
    return {key for key in a.keys() | b.keys() if key not in a or key not in b or a[key] != b[key]}


def test_the_comparison_sees_a_changed_a_missing_and_a_nested_key():
    cfg = train.load_config(LONG["mlp"])
    assert differences(cfg, cfg) == set()
    other = json.loads(json.dumps(cfg))
    other["ppo"]["gamma"] = 0.5
    other["entropy_control"]["targets"]["play"] = 0.3
    del other["run"]["fault"]
    assert differences(cfg, other) == {"ppo.gamma", "entropy_control.targets.play", "run.fault"}


def test_c01_the_base_arm_of_the_mlp_differs_from_the_long_run_in_three_keys_only():
    long_run, base = train.load_config(LONG["mlp"]), train.load_config(trial_file("base", "mlp"))
    assert differences(long_run, base) == {"run.name", "run.updates", "run.time_limit_minutes"}
    assert base["run"]["updates"] == 500 and base["run"]["seed"] == 14 and base["run"]["network"] == "mlp"
    assert base["entropy_control"]["min_weight"] == 0.01


def test_c01_the_trial_arm_of_the_mlp_differs_in_those_three_and_in_the_lower_limit():
    long_run, low = train.load_config(LONG["mlp"]), train.load_config(trial_file("low", "mlp"))
    assert differences(long_run, low) == {"run.name", "run.updates", "run.time_limit_minutes", "entropy_control.min_weight"}
    assert low["run"]["updates"] == 500 and low["run"]["seed"] == 14
    assert low["entropy_control"]["min_weight"] == 0.001
    # The two arms differ from each other in the name and the lower limit, in nothing else.
    assert differences(train.load_config(trial_file("base", "mlp")), low) == {"run.name", "entropy_control.min_weight"}


def test_c01_the_trial_arm_of_the_transformer_differs_in_those_four_and_in_the_memory_limit():
    long_run, low = train.load_config(LONG["transformer"]), train.load_config(trial_file("low", "transformer"))
    four = {"run.name", "run.updates", "run.time_limit_minutes", "entropy_control.min_weight"}
    found = differences(long_run, low)
    # One further difference (D150): the limit on PyTorch's allocator (spec 015). It is a setting of the
    # machine and changes no number of the training; the base arm, the run of spec 014, had none.
    assert found == four | {"run.gpu_memory_limit_mib"}
    assert low["run"]["gpu_memory_limit_mib"] == 5000 and long_run["run"]["gpu_memory_limit_mib"] == 0
    for arm in ARMS:  # the MLP configs keep 0
        for seed in SEEDS:
            assert train.load_config(trial_file(arm, "mlp", seed))["run"]["gpu_memory_limit_mib"] == 0
    assert low["run"]["updates"] == 500 and low["run"]["seed"] == 14 and low["run"]["network"] == "transformer"
    assert low["entropy_control"]["min_weight"] == 0.001
    assert not trial_file("base", "transformer").exists()  # its base arm is checkpoint 500 of the run of spec 014


def test_c01_the_start_weight_lies_inside_both_limits_so_the_lower_limit_is_the_only_thing_that_moves():
    for arm, floor in ARMS.items():
        cfg = train.load_config(trial_file(arm, "mlp"))
        control = cfg["entropy_control"]
        assert control["min_weight"] == floor and control["enabled"] is True
        # Every weight starts at [ppo] entropy_weight. In the base arm that is the lower limit itself;
        # in the trial arm there is room under it.
        assert control["min_weight"] <= cfg["ppo"]["entropy_weight"] == 0.01 < control["max_weight"]


@pytest.mark.parametrize("arm", list(ARMS))
@pytest.mark.parametrize("seed", [15, 16])
def test_c02_the_seeds_15_and_16_have_a_config_each_that_differs_in_the_seed_and_the_name_only(arm, seed):
    first, other = train.load_config(trial_file(arm, "mlp")), train.load_config(trial_file(arm, "mlp", seed))
    assert differences(first, other) == {"run.seed", "run.name"}
    assert other["run"]["seed"] == seed


def test_c02_the_names_of_the_runs_tell_the_arm_and_the_seed():
    names = set()
    for arm in ARMS:
        for seed in SEEDS:
            cfg = train.load_config(trial_file(arm, "mlp", seed))
            assert cfg["run"]["name"] == f"trial-floor-{arm}-s{seed}"
            names.add(cfg["run"]["name"])
    assert len(names) == 6
    assert train.load_config(trial_file("low", "transformer"))["run"]["name"] == "trial-floor-low-s14"
    # Seven config files and no other trial file.
    assert len(list(CONFIGS.glob("trial-floor-*.toml"))) == 7


def test_c01_every_setting_of_the_trial_configs_has_its_origin_in_the_file():
    for path in sorted(CONFIGS.glob("trial-floor-*.toml")):
        cfg = train.load_config(path)
        settings = 0
        for line in path.read_text(encoding="utf-8").splitlines():
            if re.match(r'^\s*("[^"]+"|[A-Za-z_0-9]+)\s*=', line):
                settings += 1
                assert re.search(r"#\s*\S", line), f"{path.name}: no origin on the line '{line}'"
        expected = sum(len(table) for table in cfg.values()) - 1 + len(cfg["entropy_control"]["targets"])
        assert settings == expected, path.name
        text = path.read_text(encoding="utf-8")
        assert "spec 017" in text.splitlines()[0]
        for key in ("name", "updates", "time_limit_minutes"):
            (line,) = [row for row in text.splitlines() if row.startswith(f"{key} = ")]
            assert "spec 017" in line, (path.name, key)
        (line,) = [row for row in text.splitlines() if row.startswith("min_weight = ")]
        assert ("spec 017" in line) == ("-low-" in path.name), path.name


def test_c02_the_run_folder_holds_the_config_as_it_ran_with_its_seed(tmp_path):
    engine()
    # The config of seed 16, cut down to a size the CPU plays in seconds. The sizes come from the command line;
    # they are written into the config that the run folder holds, next to the seed of the file.
    path = trial_file("low", "mlp", 16)
    args = ["--config", str(path), "--out-dir", str(tmp_path), "--updates", "1", "--tables", "16", "--seat-deals", "32", "--device", "cpu"]
    assert train.main(args) == 0
    folder = tmp_path / "trial-floor-low-s16-mlp"
    info = json.loads((folder / "run-info.json").read_text(encoding="utf-8"))
    assert info["config_file"] == path.name
    assert info["config"]["run"]["seed"] == 16 and info["config"]["run"]["name"] == "trial-floor-low-s16"
    assert info["config"]["entropy_control"]["min_weight"] == 0.001
    checkpoint = torch.load(folder / "checkpoint-000001.pt", map_location="cpu", weights_only=True)
    assert checkpoint["config"]["run"]["seed"] == 16
    assert checkpoint["config"]["entropy_control"]["min_weight"] == 0.001


# --------------------------------------------------------------------------
# trial_report.py: the numbers of criteria 7 to 10 from a run folder
# --------------------------------------------------------------------------

PLAY_EDGE = 0.25 * 1.75  # the upper edge of the play's band: target 0.25, band 0.75
BIDDING_EDGE = 0.2 * 0.25  # the lower edge of the bidding's band


def fake_run(folder: Path, floor: float, play_weights: dict, talon_weights: dict) -> Path:
    """A run folder with 500 logged updates: the tags the report reads, with values a test can check.

    The play's entropy falls from 0.5 to 0.25 (0.45 at update 100). The bidding dips to 0.01 in
    updates 10 to 19. Every other phase sits at its target. "ena" is 80% of the deals up to
    update 450 and 90% after it.
    """
    from torch.utils.tensorboard import SummaryWriter

    cfg = train.load_config(LONG["mlp"])
    cfg["entropy_control"]["min_weight"] = floor
    cfg["run"]["name"] = "fake"
    writer = SummaryWriter(str(folder))
    for update in range(1, 501):
        writer.add_scalar("loss/total", 0.2, update)
        play = 0.5 - update / 2000
        bidding = 0.01 if 10 <= update <= 19 else 0.2
        for name in C.PHASES[:-1]:
            if name == "announcements":
                continue  # flow A has none: the tags are absent
            entropy = {"play": play, "bidding": bidding}.get(name, cfg["entropy_control"]["targets"][name])
            writer.add_scalar(f"entropy/{name}", entropy, update)
            weight = 0.01
            if name == "play":
                weight = next(value for start, value in sorted(play_weights.items(), reverse=True) if update >= start)
            if name == "talon pick":
                weight = next(value for start, value in sorted(talon_weights.items(), reverse=True) if update >= start)
            writer.add_scalar(f"entropy_weight/{name}", weight, update)
        writer.add_scalar("entropy/phases_outside_band", float((play > PLAY_EDGE) + (bidding < BIDDING_EDGE)), update)
        writer.add_scalar("entropy_choice/play", play / 0.7, update)
        writer.add_scalar("forced_share/play", 0.3, update)
        writer.add_scalar("deals/count", 500.0, update)
        writer.add_scalar("deals/contract_ena", 0.8 if update <= 450 else 0.9, update)
        writer.add_scalar("deals/contract_dve", 0.2 if update <= 450 else 0.1, update)
        writer.add_scalar("deals/declarer_wins", 0.6, update)
        writer.add_scalar("aux/card_accuracy", 0.37, update)
        writer.add_scalar("critic/explained_variance", 0.6, update)
        writer.add_scalar("ppo/approx_kl", 0.002, update)
        writer.add_scalar("env/illegal_actions", 0.0, update)
        writer.add_scalar("reward/deal_sum_abs_max", 1e-7, update)
        writer.add_scalar("rollout/decision_age_max", 0.0, update)
    writer.close()
    info = {"started": "2026-11-24T00:00:00", "commit": "0" * 40, "git_status_short": "", "config_file": "fake.toml", "run": "fake-mlp", "config": cfg}
    (folder / "run-info.json").write_text(json.dumps(info), encoding="utf-8")
    return folder


def test_c07_c09_the_report_reads_the_play_and_every_phase_that_went_under_the_old_limit(tmp_path):
    from tarok_train import trial_report

    # The play: 0.01 until update 149, 0.004 from 150, 0.002 from 400. The talon pick dips to 0.006 from 300.
    folder = fake_run(tmp_path, 0.001, {1: 0.01, 150: 0.004, 400: 0.002}, {1: 0.01, 300: 0.006})
    numbers = trial_report.read(folder)
    assert numbers["updates"] == 500 and numbers["min_weight"] == 0.001
    assert [row["update"] for row in numbers["play"]] == [100, 200, 300, 400, 500]  # criterion 7
    assert [row["weight"] for row in numbers["play"]] == pytest.approx([0.01, 0.004, 0.004, 0.002, 0.002])
    assert [row["entropy"] for row in numbers["play"]] == pytest.approx([0.45, 0.40, 0.35, 0.30, 0.25])
    assert numbers["play_lowest_weight"] == pytest.approx(0.002)
    # The weight logged at update N is the one the controller set after update N; update N + 1 trains with it.
    assert numbers["play_first_under_old_limit"] == {"set_after_update": 150, "used_from_update": 151}
    # Criterion 9: every phase whose weight went under 0.01, with its lowest value.
    assert numbers["under_old_limit"] == {"play": pytest.approx(0.002), "talon pick": pytest.approx(0.006)}
    # Criterion 8: every phase at update 500 beside its target, and the phases outside the band.
    assert numbers["phases"]["play"] == {"entropy": pytest.approx(0.25), "target": 0.25, "weight": pytest.approx(0.002), "in_band": True}
    assert numbers["phases"]["bidding"]["target"] == 0.2
    assert "announcements" not in numbers["phases"]
    assert numbers["phases_outside_band"] == 0 and numbers["phases_outside_band_logged"] == 0
    # Criterion 10 and the loop checks of criterion 4.
    assert numbers["contracts"] == {"dve": pytest.approx(0.1), "ena": pytest.approx(0.9)}
    assert numbers["declarer_wins"] == pytest.approx(0.6)
    assert numbers["checks"] == {"illegal_actions": 0.0, "deal_sum_abs_max": pytest.approx(1e-7), "decision_age_max": 0.0, "not_finite": 0}


def test_c07_a_run_whose_weights_never_leave_the_limit_reports_that(tmp_path):
    from tarok_train import trial_report

    numbers = trial_report.read(fake_run(tmp_path, 0.01, {1: 0.01}, {1: 0.01}))
    assert numbers["play_lowest_weight"] == pytest.approx(0.01)
    assert numbers["play_first_under_old_limit"] is None
    assert numbers["under_old_limit"] == {}
    text = trial_report.report(tmp_path)
    assert "never" in text and "| 500 |" in text
    assert "set after" in text  # the report says what the logged weight is


def test_c08_the_phases_outside_the_band_are_counted_from_the_entropies_and_checked_against_the_logged_count(tmp_path):
    from tarok_train import trial_report

    folder = fake_run(tmp_path, 0.01, {1: 0.01}, {1: 0.01})
    early = trial_report.read(folder, at=100)  # the play at 0.45, above the upper edge 0.4375
    assert early["phases"]["play"]["in_band"] is False and early["phases_outside_band"] == 1 == early["phases_outside_band_logged"]
    dip = trial_report.read(folder, at=15)  # the bidding at 0.01, under the lower edge 0.05; the play above its band
    assert dip["phases"]["bidding"]["in_band"] is False and dip["phases_outside_band"] == 2 == dip["phases_outside_band_logged"]
    assert "Phases outside the band at update 100: 1" in trial_report.report(folder, at=100)


def test_c06_a_longer_run_is_read_as_if_it_had_ended_at_an_earlier_update(tmp_path, capsys):
    from tarok_train import trial_report

    # The base arm of the transformer is the run of spec 014 read up to update 500: every number of it goes this way.
    folder = fake_run(tmp_path, 0.001, {1: 0.01, 150: 0.004, 400: 0.002}, {1: 0.01, 300: 0.006})
    numbers = trial_report.read(folder, at=300)
    assert numbers["updates"] == 300
    assert [row["update"] for row in numbers["play"]] == [100, 200, 300]
    assert numbers["phases"]["play"]["entropy"] == pytest.approx(0.35) and numbers["phases"]["play"]["weight"] == pytest.approx(0.004)
    assert numbers["play_lowest_weight"] == pytest.approx(0.004)  # the 0.002 of update 400 is not read
    assert numbers["under_old_limit"] == {"play": pytest.approx(0.004), "talon pick": pytest.approx(0.006)}
    assert (numbers["window"]["first"], numbers["window"]["last"]) == (201, 300)
    assert numbers["window"]["phases"]["play"]["entropy"] == pytest.approx(0.5 - 250.5 / 2000)
    assert numbers["contracts"] == {"dve": pytest.approx(0.2), "ena": pytest.approx(0.8)}
    assert trial_report.main(["--at", "300", str(folder)]) == 0
    assert "300 updates read" in capsys.readouterr().out
    # Past the end nothing changes. A run shorter than the window is read whole.
    assert trial_report.read(folder, at=900)["updates"] == 500
    short = trial_report.read(folder, at=50)
    assert short["updates"] == 50 and short["play"] == []
    assert (short["window"]["first"], short["window"]["last"], short["window"]["updates"]) == (1, 50, 50)


def test_c08_c10_the_means_over_the_last_hundred_updates(tmp_path):
    from tarok_train import trial_report

    # The play's weight: 0.002 from update 400, at the lower limit 0.001 from update 451.
    folder = fake_run(tmp_path, 0.001, {1: 0.01, 150: 0.004, 400: 0.002, 451: 0.001}, {1: 0.01, 300: 0.006})
    window = trial_report.read(folder)["window"]
    assert (window["first"], window["last"], window["updates"]) == (401, 500, 100)
    play = window["phases"]["play"]
    assert play["entropy"] == pytest.approx(0.5 - 450.5 / 2000)
    assert play["weight"] == pytest.approx(0.0015)
    assert play["under_old_limit"] == 1.0 and play["at_lower_limit"] == 0.5
    talon = window["phases"]["talon pick"]
    assert talon["weight"] == pytest.approx(0.006) and talon["under_old_limit"] == 1.0 and talon["at_lower_limit"] == 0.0
    discard = window["phases"]["discard"]
    assert discard["under_old_limit"] == 0.0 and discard["at_lower_limit"] == 0.0  # 0.01 is the old limit, not under it
    assert window["contracts"] == {"dve": pytest.approx(0.15), "ena": pytest.approx(0.85)}
    assert window["deals_per_update"] == pytest.approx(500.0)
    assert window["monitors"]["aux/card_accuracy"] == pytest.approx(0.37)
    assert window["monitors"]["entropy_choice/play"] == pytest.approx((0.5 - 450.5 / 2000) / 0.7)
    assert window["monitors"]["entropy/phases_outside_band"] == 0.0
    # In a run with the limit 0.01 a weight of 0.01 is at the lower limit and not under the old one.
    base = trial_report.read(fake_run(tmp_path / "base", 0.01, {1: 0.01}, {1: 0.01}))["window"]["phases"]["play"]
    assert base["under_old_limit"] == 0.0 and base["at_lower_limit"] == 1.0
    # A shorter window.
    assert trial_report.read(folder, window=50)["window"]["contracts"]["ena"] == pytest.approx(0.9)


def test_the_dips_of_every_phase_over_the_run(tmp_path):
    from tarok_train import trial_report

    dips = trial_report.read(fake_run(tmp_path, 0.01, {1: 0.01}, {1: 0.01}))["dips"]
    assert dips["bidding"] == {"lowest": pytest.approx(0.01), "lowest_at": 10, "under_edge": 10, "last_under_edge": 19, "edge": pytest.approx(0.05)}
    assert dips["play"]["lowest"] == pytest.approx(0.25) and dips["play"]["lowest_at"] == 500
    assert dips["play"]["under_edge"] == 0 and dips["play"]["last_under_edge"] is None


# --------------------------------------------------------------------------
# trial_compare.py: two evaluation files of one pair of checkpoints
# --------------------------------------------------------------------------


def evaluation(values, seed=5151, deals=None, tables=500, folder="y-run", updates=500):
    """What evaluate.py writes for one direction, as far as the comparison reads it."""
    from tarok_train import evaluate

    values = [float(v) for v in values]
    mean, low, high = evaluate.interval(values)
    return {
        "mean": mean, "low": low, "high": high, "values": values, "seed": seed, "deals": deals or len(values), "tables": tables,
        "updates": updates, "opponent": {"folder": folder, "updates": updates, "run": "y", "checkpoint": f"checkpoint-{updates:06d}.pt"},
    }  # fmt: skip


def test_the_two_directions_together_are_paired_over_the_deals():
    from tarok_train import evaluate, trial_compare

    one = evaluation([0.1, 0.3, -0.2, 0.0])  # X in one seat among three Y
    other = evaluation([-0.1, 0.1, 0.2, -0.4])  # Y in one seat among three X, on the same four deals
    numbers = trial_compare.paired(one, other)
    # Per deal: half the difference [0.1, 0.1, -0.2, 0.2], half the sum [0.0, 0.2, 0.0, -0.2].
    assert numbers["half_difference"] == pytest.approx(evaluate.interval([0.1, 0.1, -0.2, 0.2]))
    assert numbers["half_sum"] == pytest.approx(evaluate.interval([0.0, 0.2, 0.0, -0.2]))
    assert numbers["half_difference"][0] == pytest.approx(0.05) and numbers["half_sum"][0] == pytest.approx(0.0)
    assert numbers["correlation"] == pytest.approx(float(np.corrcoef(one["values"], other["values"])[0, 1]))
    # The interval is not the one of two independent numbers. Two directions that mirror each other deal by
    # deal have a half sum of exactly zero in every deal; taken as independent it would get a wide interval.
    one, other = evaluation([0.4, -0.4, 0.2, -0.2]), evaluation([-0.4, 0.4, -0.2, 0.2])
    mirrored = trial_compare.paired(one, other)
    assert mirrored["correlation"] == pytest.approx(-1.0)
    assert mirrored["half_sum"] == pytest.approx((0.0, 0.0, 0.0), abs=1e-12)
    se = lambda r: (r["high"] - r["low"]) / (2 * 1.96)  # noqa: E731
    assert 1.96 * math.sqrt(se(one) ** 2 + se(other) ** 2) / 2 > 0.2
    assert mirrored["half_difference"] == pytest.approx(evaluate.interval([0.4, -0.4, 0.2, -0.2]))


def test_two_directions_that_are_not_the_same_deals_are_refused():
    from tarok_train import trial_compare

    one = evaluation([0.1, 0.3, -0.2, 0.0])
    for changed in (evaluation([0.1, 0.3, -0.2, 0.0], seed=5252), evaluation([0.1, 0.3, -0.2]), evaluation([0.1, 0.3, -0.2, 0.0], tables=2)):
        with pytest.raises(ValueError, match="same deals"):
            trial_compare.paired(one, changed)


def interval_only(mean, half):
    return {"mean": mean, "low": mean - half, "high": mean + half}


def test_c14_the_side_on_which_both_intervals_lie():
    from tarok_train import trial_compare

    ahead = (interval_only(+0.05, 0.01), interval_only(-0.04, 0.01))  # the trial seat gains, the base seat loses
    behind = (interval_only(-0.05, 0.01), interval_only(+0.04, 0.01))
    both_lose = (interval_only(-0.05, 0.01), interval_only(-0.04, 0.01))  # what the trial found between two runs
    touches = (interval_only(+0.05, 0.01), interval_only(-0.005, 0.01))  # one interval includes zero
    assert trial_compare.side(*ahead) == "trial" and trial_compare.side(*behind) == "base"
    assert trial_compare.side(*both_lose) is None and trial_compare.side(*touches) is None


def test_c14_the_rule_fixed_before_the_runs():
    from tarok_train import trial_compare

    ahead = (interval_only(+0.15, 0.01), interval_only(-0.14, 0.01))
    small = (interval_only(+0.05, 0.01), interval_only(-0.04, 0.01))  # on the trial's side, but no larger than chance
    behind = (interval_only(-0.15, 0.01), interval_only(+0.14, 0.01))
    nothing = (interval_only(-0.05, 0.01), interval_only(-0.04, 0.01))
    chance = 0.1  # the largest difference between two base runs (criterion 13)
    # The MLP: at least two of the three seeds, and the differences larger than chance.
    assert trial_compare.reading([ahead, ahead, nothing], needed=2, chance=chance) == "trial ahead"
    assert trial_compare.reading([ahead, nothing, nothing], needed=2, chance=chance) == "no difference shown"
    assert trial_compare.reading([ahead, small, nothing], needed=2, chance=chance) == "no difference shown"
    assert trial_compare.reading([behind, behind, ahead], needed=2, chance=chance) == "trial behind"
    assert trial_compare.reading([nothing, nothing, nothing], needed=2, chance=chance) == "no difference shown"
    assert trial_compare.reading([ahead, ahead, behind], needed=2, chance=chance) == "trial ahead"
    # The transformer: one pair, and no condition on chance in its sentence of the rule.
    assert trial_compare.reading([small], needed=1) == "trial ahead"
    assert trial_compare.reading([behind], needed=1) == "trial behind"
    assert trial_compare.reading([nothing], needed=1) == "no difference shown"
    assert trial_compare.largest([interval_only(+0.02, 0.01), interval_only(-0.1, 0.01), interval_only(0.05, 0.01)]) == pytest.approx(0.1)


def test_the_comparison_reads_both_files_of_a_pair_of_run_folders(tmp_path, capsys):
    from tarok_train import trial_compare

    x, y = tmp_path / "x-run", tmp_path / "y-run"
    for folder in (x, y, x / "seed-5252", y / "seed-5252"):
        folder.mkdir(parents=True)
    rng = np.random.default_rng(1)
    a, b = rng.normal(0.05, 0.3, 200), rng.normal(-0.08, 0.3, 200)
    (x / "evaluation-000500-vs-y-run-000500.json").write_text(json.dumps(evaluation(a, folder="y-run")), encoding="utf-8")
    (y / "evaluation-000500-vs-x-run-000500.json").write_text(json.dumps(evaluation(b, folder="x-run")), encoding="utf-8")
    (x / "seed-5252" / "evaluation-000500-vs-y-run-000500.json").write_text(json.dumps(evaluation(a + 1, seed=5252, folder="y-run")), encoding="utf-8")
    (y / "seed-5252" / "evaluation-000500-vs-x-run-000500.json").write_text(json.dumps(evaluation(b + 1, seed=5252, folder="x-run")), encoding="utf-8")
    numbers = trial_compare.compare(x, y)
    assert numbers["one"]["mean"] == pytest.approx(a.mean()) and numbers["other"]["mean"] == pytest.approx(b.mean())
    assert numbers["half_difference"][0] == pytest.approx((a.mean() - b.mean()) / 2)
    assert numbers["half_sum"][0] == pytest.approx((a.mean() + b.mean()) / 2)
    assert numbers["seed"] == 5151
    again = trial_compare.compare(x, y, sub="seed-5252")  # a second set of files, kept in a subfolder
    assert again["seed"] == 5252 and again["half_sum"][0] == pytest.approx((a.mean() + b.mean()) / 2 + 1)
    # A file that names another opponent folder than the one asked for is refused.
    wrong = evaluation(a, folder="z-run")
    (x / "evaluation-000500-vs-y-run-000500.json").write_text(json.dumps(wrong), encoding="utf-8")
    with pytest.raises(ValueError, match="z-run"):
        trial_compare.compare(x, y)
    (x / "evaluation-000500-vs-y-run-000500.json").write_text(json.dumps(evaluation(a, folder="y-run")), encoding="utf-8")
    assert trial_compare.main([str(x), str(y)]) == 0
    out = capsys.readouterr().out
    assert "Half the difference" in out and "Half the sum" in out and "x-run" in out
