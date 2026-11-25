"""Tests for spec 015 (findings of the longer run).

The name of each test starts with its criterion number. Part A: the GPU reading
(criteria 1 to 3). Part B: the memory tags and the limit on PyTorch's allocator
(criteria 4, 6 and 9). Part C: a checkpoint as the opponent in the evaluation
(criteria 11 to 15). The measured runs of criteria 5, 7, 8 and 16 are not tests.
Tests named `test_r<n>_...` belong to point n of the review of 2026-11-24 (see the
section "Review fixes" of the spec's Results).
"""

import gc
import json
import subprocess
from pathlib import Path

import numpy as np
import pytest
import torch

from tarok_train import bench_model, train

from test_spec007 import CPU, scalar_tags, small
from test_spec008 import CI_TOML, ci_cfg, engine

READING = {"utilization_percent": 0, "memory_used_mib": 188, "memory_total_mib": 6144, "temperature_c": 54.3}


class Done:
    """What subprocess.run returns, as far as gpu_snapshot reads it."""

    def __init__(self, stdout: str):
        self.stdout = stdout
        self.returncode = 0


def fake_nvidia_smi(monkeypatch, stdout=None, error=None):
    def run(command, **kwargs):
        assert command[0] == "nvidia-smi"
        if error is not None:
            raise error
        return Done(stdout)

    monkeypatch.setattr(bench_model.subprocess, "run", run)


# --------------------------------------------------------------------------
# Part A, criteria 1 to 3: the GPU reading
# --------------------------------------------------------------------------


@pytest.mark.parametrize(
    "line, temperature",
    [("0, 188, 6144, 54.30", 54.3), ("0, 188, 6144, 54", 54.0), ("0, 188, 6144, 54.30\r\n", 54.3), ("0, 188, 6144, 54\n", 54.0)],
)
def test_c01_the_reading_holds_the_four_numbers_for_a_whole_and_a_decimal_temperature(monkeypatch, line, temperature):
    fake_nvidia_smi(monkeypatch, line)
    reading = bench_model.gpu_snapshot()
    assert set(reading) == set(READING)
    assert reading["utilization_percent"] == 0 and reading["memory_used_mib"] == 188 and reading["memory_total_mib"] == 6144
    assert reading["temperature_c"] == pytest.approx(temperature)
    # Load and memory stay whole numbers: other code prints and compares them as such.
    assert all(isinstance(reading[key], int) for key in ("utilization_percent", "memory_used_mib", "memory_total_mib"))


@pytest.mark.parametrize(
    "output",
    [
        "",
        "\n",
        "0, 188, 6144",
        "0, 188, 6144, 54, 7",
        "[N/A], 188, 6144, 54",
        "0, 188, 6144, hot",
        "nvidia-smi is not recognized",
        # Review, point 7: float() reads these, and NaN is not valid JSON. They are not a temperature.
        "0, 188, 6144, nan",
        "0, 188, 6144, inf",
        "0, 188, 6144, -inf",
        # Two cards print two lines. This code reads one card; it says so by an empty reading (README).
        "0, 188, 6144, 54\n3, 200, 8192, 50",
    ],
)
def test_c02_output_that_cannot_be_read_gives_an_empty_reading(monkeypatch, output):
    fake_nvidia_smi(monkeypatch, output)
    assert bench_model.gpu_snapshot() == {}


@pytest.mark.parametrize("error", [FileNotFoundError("nvidia-smi"), subprocess.TimeoutExpired("nvidia-smi", 20)])
def test_c02_a_missing_or_hanging_nvidia_smi_gives_an_empty_reading(monkeypatch, error):
    fake_nvidia_smi(monkeypatch, error=error)
    assert bench_model.gpu_snapshot() == {}


def test_c02_the_trainer_says_in_one_line_that_the_gpu_state_could_not_be_read(monkeypatch, capsys):
    monkeypatch.setattr(train, "gpu_snapshot", lambda: {})
    assert train.gpu_state("before") == {}
    out = capsys.readouterr().out
    assert out.count("\n") == 1
    assert "could not be read" in out and "before" in out
    # A reading that worked is returned and nothing is printed.
    monkeypatch.setattr(train, "gpu_snapshot", lambda: dict(READING))
    assert train.gpu_state("after") == READING
    assert capsys.readouterr().out == ""


@pytest.mark.gpu
def test_c02_a_gpu_run_whose_state_cannot_be_read_says_so_twice_and_still_runs(tmp_path, monkeypatch, capsys):
    engine()
    monkeypatch.setattr(train, "gpu_snapshot", lambda: {})
    assert train.main(["--config", str(CI_TOML), "--out-dir", str(tmp_path), "--updates", "1", "--device", "cuda"]) == 0
    out = capsys.readouterr().out
    assert out.count("could not be read") == 2  # before the run and after it
    summary = json.loads((tmp_path / "smoke-a-ci-mlp" / "summary.json").read_text(encoding="utf-8"))
    assert summary["gpu_before"] == {} and summary["gpu_after"] == {}


@pytest.mark.gpu
def test_c03_on_this_pc_run_info_and_summary_hold_the_four_numbers_before_and_after(tmp_path, capsys):
    engine()
    assert train.main(["--config", str(CI_TOML), "--out-dir", str(tmp_path), "--updates", "1", "--device", "cuda"]) == 0
    folder = tmp_path / "smoke-a-ci-mlp"
    info = json.loads((folder / "run-info.json").read_text(encoding="utf-8"))
    summary = json.loads((folder / "summary.json").read_text(encoding="utf-8"))
    for reading in (info["gpu_before"], summary["gpu_before"], summary["gpu_after"]):
        assert set(reading) == set(READING)
        assert 0 <= reading["utilization_percent"] <= 100
        assert 0 < reading["memory_used_mib"] <= reading["memory_total_mib"]
        assert 0 < reading["temperature_c"] < 120
    assert info["gpu_before"] == summary["gpu_before"]
    assert "could not be read" not in capsys.readouterr().out


def summary_for_the_report(**changed):
    summary = {
        "finished": True, "failed": None, "stopped_by_time_limit": False, "updates": 2, "updates_wanted": 2,
        "seconds": 10.0, "device": "cuda", "played_per_second": 100.0, "trained_per_second": 50.0,
        "trained_share": 0.5, "trained_share_own_seats": 0.55, "gpu_before": dict(READING), "gpu_after": dict(READING),
        "time_shares": {"environment": 0.1, "acting": 0.4, "bookkeeping": 0.1, "update": 0.4}, "time_shares_sum": 1.0,
        "peak_gpu_memory_mib": 100, "peak_gpu_memory_reserved_mib": 200,
    }  # fmt: skip
    summary.update(changed)
    return summary


def folder_for_the_report(tmp_path, summary):
    from torch.utils.tensorboard import SummaryWriter

    writer = SummaryWriter(str(tmp_path))
    for step in (1, 2):
        writer.add_scalar("loss/total", 1.0, step)
    writer.close()
    (tmp_path / "summary.json").write_text(json.dumps(summary), encoding="utf-8")
    return tmp_path


def test_c02_the_report_says_not_read_for_an_empty_reading(tmp_path):
    from tarok_train import smoke_report

    text = smoke_report.report(folder_for_the_report(tmp_path, summary_for_the_report(gpu_before={}, gpu_after={})))
    assert "| GPU before | not read |" in text and "| GPU after | not read |" in text
    assert "None MiB" not in text and "{}" not in text


def test_c03_the_report_prints_a_reading_that_worked(tmp_path):
    from tarok_train import smoke_report

    text = smoke_report.report(folder_for_the_report(tmp_path, summary_for_the_report()))
    assert "not read" not in text
    assert "all programs (nvidia-smi) | 188 MiB |" in text


# --------------------------------------------------------------------------
# Part B, criterion 4: the allocator's size and the tensors' size, per update
# --------------------------------------------------------------------------


def test_c04_on_the_cpu_the_memory_tags_are_absent_and_nothing_fails(tmp_path):
    assert train.gpu_memory_now(CPU) is None
    trainer = train.Trainer(small(), CPU, log_dir=tmp_path)
    history = trainer.run(2)
    trainer.close()
    for stats in history:
        assert not [tag for tag in stats if tag.startswith("gpu/")]
    assert not [tag for tag in scalar_tags(tmp_path) if tag.startswith("gpu/")]


def test_c04_the_four_tags_hold_the_reading_before_and_the_reading_after_the_update_step(monkeypatch):
    assert train.MEMORY_TAGS == (
        "gpu/reserved_after_acting_mib",
        "gpu/allocated_after_acting_mib",
        "gpu/reserved_after_update_mib",
        "gpu/allocated_after_update_mib",
    )
    log = []
    readings = []

    def memory(device):
        reading = (1000.0 + len(readings), 500.0 + len(readings))  # (reserved, allocated): every call differs
        readings.append(reading)
        log.append("memory")
        return reading

    real_update = train.ppo.update

    def update(*args, **kwargs):
        log.append("update")
        return real_update(*args, **kwargs)

    monkeypatch.setattr(train, "gpu_memory_now", memory)
    monkeypatch.setattr(train.ppo, "update", update)
    trainer = train.Trainer(small(), CPU)
    history = trainer.run(3)
    # One reading right before every update step (acting is over) and one right after it.
    assert log == ["memory", "update", "memory"] * 3
    for index, stats in enumerate(history):
        before, after = readings[2 * index], readings[2 * index + 1]
        assert stats["gpu/reserved_after_acting_mib"] == before[0]
        assert stats["gpu/allocated_after_acting_mib"] == before[1]
        assert stats["gpu/reserved_after_update_mib"] == after[0]
        assert stats["gpu/allocated_after_update_mib"] == after[1]


@pytest.mark.gpu
def test_c04_on_the_gpu_the_four_tags_are_logged_next_to_the_two_peaks(tmp_path):
    trainer = train.Trainer(small(), torch.device("cuda"), log_dir=tmp_path)
    history = trainer.run(2)
    trainer.close()
    for stats in history:
        for when in ("acting", "update"):
            reserved, allocated = stats[f"gpu/reserved_after_{when}_mib"], stats[f"gpu/allocated_after_{when}_mib"]
            assert 0 < allocated <= reserved <= stats["gpu/peak_memory_reserved_mib"]
            assert allocated <= stats["gpu/peak_memory_mib"]
    assert set(train.MEMORY_TAGS) <= scalar_tags(tmp_path)


# --------------------------------------------------------------------------
# Part B, criteria 6 and 9: the limit on PyTorch's allocator, a key of the config
# --------------------------------------------------------------------------

CONFIGS = CI_TOML.parent
FINISHED_RUNS = ("smoke-a-mlp.toml", "smoke-a-transformer.toml", "long-a-mlp.toml", "long-a-transformer.toml")


class Card:
    total_memory = 6144 * 2**20


def fake_card(monkeypatch, card=Card):
    """A card of 6,144 MiB for `limit_gpu_memory`, without a GPU. Returns the list of fractions that were set."""
    fractions = []
    monkeypatch.setattr(train, "LIMITED_CARDS", set())  # as in a new process: no limit was set before
    monkeypatch.setattr(torch.cuda, "current_device", lambda: 0)
    monkeypatch.setattr(torch.cuda, "get_device_properties", lambda device: card())
    monkeypatch.setattr(torch.cuda, "set_per_process_memory_fraction", lambda fraction, device=None: fractions.append((fraction, device)))
    return fractions


def test_c06_the_limit_is_set_as_that_share_of_the_cards_memory(monkeypatch):
    fractions = fake_card(monkeypatch)
    cuda = torch.device("cuda")
    assert train.limit_gpu_memory(5000, cuda) == pytest.approx(5000 / 6144)
    # PyTorch wants the number of the card here: "cuda" without a number is the current card, 0.
    assert len(fractions) == 1 and fractions[0][0] == pytest.approx(5000 / 6144) and fractions[0][1] == 0
    assert train.limit_gpu_memory(6144, torch.device("cuda:1")) == pytest.approx(1.0)
    assert len(fractions) == 2 and fractions[1][1] == 1


def test_c06_no_limit_and_the_cpu_set_nothing(monkeypatch):
    fractions = fake_card(monkeypatch)
    assert train.limit_gpu_memory(0, torch.device("cuda")) is None  # 0: the allocator is left as it is
    assert train.limit_gpu_memory(5000, CPU) is None
    assert train.limit_gpu_memory(0, CPU) is None
    assert fractions == []


def test_c06_a_limit_above_the_cards_memory_is_refused(monkeypatch):
    fractions = fake_card(monkeypatch)
    with pytest.raises(train.ConfigError, match="above the card"):
        train.limit_gpu_memory(6300, torch.device("cuda"))  # more than a rounding matter above 6,144 (review, point 6)
    assert fractions == []


def test_r5_no_limit_lifts_a_limit_set_earlier_in_the_same_process(monkeypatch):
    fractions = fake_card(monkeypatch)
    cuda = torch.device("cuda")
    assert train.limit_gpu_memory(0, cuda) is None and fractions == []  # a new process: nothing to lift
    assert train.limit_gpu_memory(5000, cuda) == pytest.approx(5000 / 6144)
    # The limit holds for the process. A trainer without one must not inherit it:
    # the whole card is the widest setting PyTorch takes.
    assert train.limit_gpu_memory(0, cuda) == 1.0
    assert fractions[-1] == (1.0, 0) and len(fractions) == 2
    assert train.limit_gpu_memory(0, cuda) is None and len(fractions) == 2  # lifted once; nothing more to do
    # Another card of the same process is not touched.
    assert train.limit_gpu_memory(4000, torch.device("cuda:1")) == pytest.approx(4000 / 6144)
    assert train.limit_gpu_memory(0, cuda) is None and len(fractions) == 3


def test_r5_the_line_at_the_start_of_a_run_says_what_holds():
    assert "limited to 5,000 MiB" in train.gpu_limit_line(5000, 5000 / 6144)
    assert train.gpu_limit_line(0, None) == "gpu memory: no limit on PyTorch's allocator"
    lifted = train.gpu_limit_line(0, 1.0)
    assert "lifted" in lifted and "no limit" not in lifted  # the cap at the card's size is not "no limit"


class SmallerCard:
    total_memory = 6144 * 2**20 - 5 * 2**20 // 2  # what a card of nominally 6,144 MiB may report: 6,141.5 MiB


def test_r6_a_limit_of_the_cards_nominal_size_is_taken_when_pytorch_reports_a_little_less(monkeypatch):
    fractions = fake_card(monkeypatch, SmallerCard)
    cuda = torch.device("cuda")
    assert train.limit_gpu_memory(6144, cuda) == 1.0  # the whole card, not a refusal by 2.5 MiB
    assert fractions == [(1.0, 0)]
    assert train.limit_gpu_memory(6141, cuda) == pytest.approx(6141 / 6141.5)
    with pytest.raises(train.ConfigError, match="above the card"):
        train.limit_gpu_memory(6300, cuda)  # 2.6% above what the card reports: not a rounding matter
    with pytest.raises(train.ConfigError, match="above the card"):
        train.limit_gpu_memory(8192, cuda)


def test_c06_the_trainer_applies_the_limit_of_its_config_before_it_builds_anything(monkeypatch):
    calls = []
    real_build = train.build_network

    def limit(limit_mib, device):
        calls.append(("limit", limit_mib, torch.device(device).type))
        return None

    def build(*args, **kwargs):
        calls.append(("network",))
        return real_build(*args, **kwargs)

    monkeypatch.setattr(train, "limit_gpu_memory", limit)
    monkeypatch.setattr(train, "build_network", build)
    trainer = train.Trainer(small(**{"run.gpu_memory_limit_mib": 4321}), CPU)
    trainer.close()
    assert calls[0] == ("limit", 4321, "cpu")  # first: nothing may be on the card before the limit holds
    assert [call[0] for call in calls].count("limit") == 1 and ("network",) in calls


def test_c09_every_config_file_has_the_key_with_its_origin_and_finished_runs_keep_their_value():
    files = sorted(CONFIGS.glob("*.toml"))
    assert {path.name for path in files} >= {"mock.toml", "smoke-a-ci.toml", *FINISHED_RUNS}
    for path in files:
        cfg = train.load_config(path)  # refuses a file without the key
        assert isinstance(cfg["run"]["gpu_memory_limit_mib"], int)
        (line,) = [text for text in path.read_text(encoding="utf-8").splitlines() if text.startswith("gpu_memory_limit_mib")]
        comment = line.split("#", 1)[1]
        assert "spec 015" in comment and "MiB" in comment, path.name
        if path.name in FINISHED_RUNS:
            # The runs of specs 008 and 014 had no limit: their files say so and keep that value.
            assert cfg["run"]["gpu_memory_limit_mib"] == 0, path.name
            assert "added later" in comment and "had no limit" in comment, path.name


@pytest.mark.parametrize("bad", [-1, 5000.0, True, "5000"])
def test_c09_the_key_must_be_a_whole_number_of_mib_and_not_negative(bad):
    cfg = small()
    cfg["run"]["gpu_memory_limit_mib"] = bad
    with pytest.raises(train.ConfigError, match="gpu_memory_limit_mib"):
        train.validate_config(cfg)


def test_c09_a_config_file_without_the_key_is_refused(tmp_path):
    lines = [line for line in CI_TOML.read_text(encoding="utf-8").splitlines() if not line.startswith("gpu_memory_limit_mib")]
    path = tmp_path / "old.toml"
    path.write_text("\n".join(lines), encoding="utf-8")
    with pytest.raises(train.ConfigError, match="gpu_memory_limit_mib"):
        train.load_config(path)


def test_c09_a_checkpoint_from_before_the_key_is_resumed_with_the_value_its_run_had(tmp_path, capsys):
    engine()
    assert train.main(["--config", str(CI_TOML), "--out-dir", str(tmp_path), "--updates", "1"]) == 0
    checkpoint = tmp_path / "smoke-a-ci-mlp" / "checkpoint-000001.pt"
    data = torch.load(checkpoint, map_location="cpu", weights_only=True)
    del data["config"]["run"]["gpu_memory_limit_mib"]  # as the checkpoints of specs 008 and 014 are
    torch.save(data, checkpoint)

    trainer = train.Trainer.load(checkpoint, CPU)
    assert trainer.cfg["run"]["gpu_memory_limit_mib"] == 0
    trainer.close()
    assert train.main(["--resume", str(checkpoint), "--updates", "2"]) == 0
    resumed = torch.load(checkpoint.parent / "checkpoint-000002.pt", map_location="cpu", weights_only=True)
    assert resumed["updates"] == 2 and resumed["config"]["run"]["gpu_memory_limit_mib"] == 0


def test_c09_the_limit_can_be_set_from_the_command_line_also_on_resume(tmp_path, monkeypatch):
    engine()
    seen = []
    monkeypatch.setattr(train, "limit_gpu_memory", lambda limit_mib, device: seen.append(limit_mib))
    base = ["--config", str(CI_TOML), "--out-dir", str(tmp_path), "--updates", "1"]
    assert train.main([*base, "--gpu-memory-limit-mib", "3000"]) == 0
    checkpoint = tmp_path / "smoke-a-ci-mlp" / "checkpoint-000001.pt"
    info = json.loads((checkpoint.parent / "run-info.json").read_text(encoding="utf-8"))
    assert info["config"]["run"]["gpu_memory_limit_mib"] == 3000  # the run log holds what really ran
    assert seen == [3000]
    assert train.main(["--resume", str(checkpoint), "--updates", "2"]) == 0
    assert seen == [3000, 3000]  # the checkpoint's value
    assert train.main(["--resume", str(checkpoint.parent / "checkpoint-000002.pt"), "--updates", "3", "--gpu-memory-limit-mib", "0"]) == 0
    assert seen == [3000, 3000, 0]  # replaced: the limit belongs to the machine, not to the training
    last = torch.load(checkpoint.parent / "checkpoint-000003.pt", map_location="cpu", weights_only=True)
    assert last["config"]["run"]["gpu_memory_limit_mib"] == 0


@pytest.mark.gpu
def test_c06_on_the_gpu_the_allocator_frees_its_cache_instead_of_passing_the_limit():
    cuda = torch.device("cuda")
    mib = 2**20
    torch.cuda.empty_cache()
    base = torch.cuda.memory_reserved(cuda) // mib  # what earlier tests left in use
    limit = int(base) + 200
    try:
        assert train.limit_gpu_memory(limit, cuda) == pytest.approx(limit * mib / torch.cuda.get_device_properties(0).total_memory)
        first = torch.empty(150 * mib, dtype=torch.uint8, device=cuda)
        del first  # the block stays in the allocator's cache
        assert torch.cuda.memory_reserved(cuda) // mib >= base + 150
        # 160 MiB do not fit into the cached block of 150, and a second block would pass the limit:
        # the allocator has to free its cache and then take the new block.
        second = torch.empty(160 * mib, dtype=torch.uint8, device=cuda)
        assert torch.cuda.memory_reserved(cuda) // mib <= limit
        # Tensors that do not fit under the limit are an error, not a spill into shared memory.
        with pytest.raises(torch.OutOfMemoryError):
            third = torch.empty(100 * mib, dtype=torch.uint8, device=cuda)  # noqa: F841
        del second
    finally:
        torch.cuda.set_per_process_memory_fraction(1.0, 0)
        train.LIMITED_CARDS.clear()
        torch.cuda.empty_cache()


@pytest.mark.gpu
def test_c06_a_gpu_run_with_the_limit_says_so_and_stays_under_it(tmp_path, capsys):
    engine()
    cuda = torch.device("cuda")
    try:
        args = ["--config", str(CI_TOML), "--out-dir", str(tmp_path), "--updates", "2", "--device", "cuda", "--gpu-memory-limit-mib", "1500"]
        assert train.main(args) == 0
        out = capsys.readouterr().out
        assert "limited to 1,500 MiB" in out
        summary = json.loads((tmp_path / "smoke-a-ci-mlp" / "summary.json").read_text(encoding="utf-8"))
        assert summary["last_update"]["gpu/reserved_after_update_mib"] <= 1500
    finally:
        torch.cuda.set_per_process_memory_fraction(1.0, 0)
        train.LIMITED_CARDS.clear()


@pytest.mark.gpu
def test_r4_on_resume_the_limit_is_set_before_anything_of_the_checkpoint_is_on_the_card(tmp_path, monkeypatch, capsys):
    engine()
    try:
        base = ["--config", str(CI_TOML), "--out-dir", str(tmp_path), "--device", "cuda", "--gpu-memory-limit-mib", "1500"]
        assert train.main([*base, "--updates", "1"]) == 0
        checkpoint = tmp_path / "smoke-a-ci-mlp" / "checkpoint-000001.pt"
        gc.collect()
        torch.cuda.empty_cache()
        before = torch.cuda.memory_allocated(0)
        on_the_card = []
        real = train.limit_gpu_memory

        def spy(limit_mib, device):
            on_the_card.append((limit_mib, torch.cuda.memory_allocated(0) - before))
            return real(limit_mib, device)

        monkeypatch.setattr(train, "limit_gpu_memory", spy)
        assert train.main(["--resume", str(checkpoint), "--updates", "2"]) == 0
        # When the limit is set, the checkpoint's tensors are not on the card yet.
        assert on_the_card == [(1500, 0)]
        assert "resumed from checkpoint-000001.pt" in capsys.readouterr().out
    finally:
        torch.cuda.set_per_process_memory_fraction(1.0, 0)
        train.LIMITED_CARDS.clear()


@pytest.mark.gpu
def test_r5_on_the_gpu_a_run_without_a_limit_after_a_limited_one_says_lifted_and_is_not_held_by_it(tmp_path, capsys):
    engine()
    mib = 2**20
    try:
        base = ["--config", str(CI_TOML), "--updates", "1", "--device", "cuda"]
        assert train.main([*base, "--out-dir", str(tmp_path / "first"), "--gpu-memory-limit-mib", "600"]) == 0
        assert "limited to 600 MiB" in capsys.readouterr().out
        assert train.main([*base, "--out-dir", str(tmp_path / "second")]) == 0  # the config's 0
        out = capsys.readouterr().out
        assert "lifted" in out and "limited to" not in out
        gc.collect()
        torch.cuda.empty_cache()
        block = torch.empty(700 * mib, dtype=torch.uint8, device="cuda")  # more than the first run's limit
        del block
    finally:
        torch.cuda.set_per_process_memory_fraction(1.0, 0)
        train.LIMITED_CARDS.clear()
        torch.cuda.empty_cache()


# --------------------------------------------------------------------------
# Part C, criteria 11 to 15: a checkpoint as the opponent
# --------------------------------------------------------------------------

STAMP = "20260101-000000"  # run folders carry a date and time; the run's name alone does not name a run


def last_legal(rows, obs):
    mask = obs["mask"][rows]
    return mask.shape[1] - 1 - mask[:, ::-1].argmax(axis=1)


def run_folder(folder: Path, name: str, kind: str, stamp: str = STAMP) -> Path:
    return folder / f"{name}-{kind}-{stamp}"


def rule_checkpoint(folder: Path, name: str, seed: int, updates: int, lowest: bool, kind: str = "mlp") -> Path:
    """A checkpoint whose network always plays the lowest (or the highest) legal slot.

    The policy head ignores its input: weight 0 and a bias that falls (or rises) by 1,000 per slot.
    After the mask one legal slot has probability 1, so sampling is the fixed rule.
    """
    trainer = train.Trainer(ci_cfg(**{"run.name": name, "run.seed": seed, "run.network": kind}), CPU)
    with torch.no_grad():
        head = trainer.net.heads.policy
        head.weight.zero_()
        slope = torch.arange(head.bias.numel(), dtype=torch.float32) * 1000.0
        head.bias.copy_(-slope if lowest else slope)
    trainer.updates = updates
    path = run_folder(folder, name, kind) / f"checkpoint-{updates:06d}.pt"
    trainer.save(path)
    trainer.close()
    return path


def trained_checkpoint(folder: Path, name: str, seed: int, kind: str = "mlp", updates: int = 1, stamp: str = STAMP) -> Path:
    """A checkpoint after `updates` updates of the small config: its network samples."""
    trainer = train.Trainer(ci_cfg(**{"run.name": name, "run.seed": seed, "run.network": kind}), CPU)
    trainer.run(updates)
    path = run_folder(folder, name, kind, stamp) / f"checkpoint-{updates:06d}.pt"
    trainer.save(path)
    trainer.close()
    return path


def by_hand(engine_env, seeds, n, agent_rule, others_rule) -> np.ndarray:
    """(deals, 4): the first deal of every table, `agent_rule` in one seat and `others_rule` in the other three."""
    blocks = []
    rows = np.arange(n)
    for seed in seeds:
        block = np.zeros((n, 4), dtype=np.float32)
        for seat in range(4):
            env = engine_env.EngineEnv(n, seed, 2)
            open_tables = np.ones(n, dtype=bool)
            while open_tables.any():
                obs = env.observe()
                action = np.where(obs["seat"] == seat, agent_rule(rows, obs), others_rule(rows, obs))
                done, reward, _ = env.step(action)
                ended = done & open_tables
                block[ended, seat] = reward[ended, seat]
                open_tables &= ~done
        blocks.append(block)
    return np.concatenate(blocks)


def run_evaluation(*args, threads: int = 2):
    from tarok_train import evaluate

    return evaluate.main([*map(str, args), "--threads", str(threads), "--device", "cpu"])


def result_of(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def versus(a: Path, b: Path) -> Path:
    """The result file of checkpoint A against checkpoint B: next to A, named by B's run folder and updates."""
    updates = lambda path: path.stem.split("-")[1]  # noqa: E731
    return a.parent / f"evaluation-{updates(a)}-vs-{b.parent.name}-{updates(b)}.json"


@pytest.mark.parametrize("kind", ["mlp", "transformer"])
def test_c11_c13_the_agents_seat_is_played_by_a_only_and_the_other_three_by_b_only(kind, tmp_path):
    engine_env = engine()
    from tarok_train import evaluate

    low = rule_checkpoint(tmp_path, "low", seed=8, updates=3, lowest=True, kind=kind)
    high = rule_checkpoint(tmp_path, "high", seed=9, updates=7, lowest=False, kind=kind)
    assert run_evaluation("--checkpoint", low, "--opponent", high, "--deals", 24, "--tables", 12, "--seed", 5) == 0
    assert versus(low, high).name == f"evaluation-000003-vs-high-{kind}-{STAMP}-000007.json"
    result = result_of(versus(low, high))
    expected = by_hand(engine_env, [5, 6], 12, evaluate.first_legal, last_legal)
    assert np.abs(expected).max() > 0.01
    assert np.allclose(result["rewards"], expected, atol=1e-6)
    assert np.allclose(result["values"], expected.mean(axis=1), atol=1e-6)
    # The two wrong seatings give other numbers on these deals, so the comparison above can tell them apart.
    assert not np.allclose(expected, by_hand(engine_env, [5, 6], 12, last_legal, evaluate.first_legal), atol=1e-6)
    assert not np.allclose(expected, by_hand(engine_env, [5, 6], 12, evaluate.first_legal, evaluate.first_legal), atol=1e-6)
    assert not np.allclose(expected, by_hand(engine_env, [5, 6], 12, last_legal, last_legal), atol=1e-6)

    # The other way round: B in one seat, A in three. Its file lies next to B.
    assert run_evaluation("--checkpoint", high, "--opponent", low, "--deals", 24, "--tables", 12, "--seed", 5) == 0
    swapped = result_of(versus(high, low))
    assert np.allclose(swapped["rewards"], by_hand(engine_env, [5, 6], 12, last_legal, evaluate.first_legal), atol=1e-6)


def test_c13_with_an_opponent_every_table_is_asked_once_per_step_and_the_agent_only_in_its_seat():
    engine_env = engine()
    from tarok_train import evaluate

    n = 6
    logs = []  # one list per environment: who was asked for which tables, in order

    def make_agent(n_tables):
        logs.append([])
        log = logs[-1]

        def agent(rows, obs):
            log.append(("agent", rows.copy(), obs["seat"][rows].copy()))
            return evaluate.first_legal(rows, obs)

        return agent

    def make_others(n_tables):
        log = logs[-1]  # the environment whose agent was just made

        def others(rows, obs):
            log.append(("others", rows.copy(), obs["seat"][rows].copy()))
            return last_legal(rows, obs)

        return others

    result = evaluate.evaluate(make_agent, deals=n, tables=n, seed=9, threads=1, make_others=make_others)
    assert len(logs) == 4  # one environment per seating
    for agent_seat, log in enumerate(logs):
        step = []
        for who, rows, seats in log:
            assert len(rows) > 0  # nobody is asked for zero tables
            if who == "agent":
                assert (seats == agent_seat).all()
            step.extend(rows.tolist())
            if len(step) >= n:
                # Every table once per step: the tables the agent plays are not asked of the opponent.
                assert sorted(step) == list(range(n))
                step = []
        assert step == []
        assert any(who == "agent" for who, _, _ in log)
        # While a table's first deal runs, the opponent is asked only in the three other seats.
        first_agent = next(index for index, entry in enumerate(log) if entry[0] == "agent")
        for who, rows, seats in log[:first_agent]:
            assert (seats != agent_seat).all()
    assert np.allclose(result["rewards"], by_hand(engine_env, [9], n, evaluate.first_legal, last_legal), atol=1e-6)


def test_c11_without_an_opponent_random_legal_play_is_asked_at_every_table_as_before():
    engine_env = engine()
    from tarok_train import evaluate

    n, agent_seat = 6, 2
    sizes = []

    class Spy(evaluate.RandomLegal):
        def __call__(self, rows, obs):
            sizes.append(len(rows))
            return super().__call__(rows, obs)

    seating = evaluate.play_seating(engine_env.EngineEnv(n, seed=9, threads=1), agent_seat, evaluate.first_legal, Spy(4))
    assert set(sizes) == {n}  # the stream of random numbers is the one of spec 008
    # The loop of spec 008, written out: random play draws for every table, the agent's slot replaces it.
    env = engine_env.EngineEnv(n, seed=9, threads=1)
    others = evaluate.RandomLegal(4)
    expected = np.zeros(n, dtype=np.float32)
    open_tables = np.ones(n, dtype=bool)
    rows = np.arange(n)
    while open_tables.any():
        obs = env.observe()
        action = np.asarray(others(rows, obs)).astype(np.int64)
        mine = np.flatnonzero(open_tables & (obs["seat"] == agent_seat))
        action[mine] = evaluate.first_legal(mine, obs)
        done, reward, _ = env.step(action)
        ended = done & open_tables
        expected[ended] = reward[ended, agent_seat]
        open_tables &= ~done
    assert np.array_equal(seating.reward, expected)


def test_c11_without_the_flag_the_evaluation_is_against_random_legal_play(tmp_path, capsys):
    engine()
    a = trained_checkpoint(tmp_path, "a", seed=8)
    assert run_evaluation("--checkpoint", a, "--deals", 24, "--tables", 12, "--seed", 5) == 0
    result = result_of(a.parent / "evaluation-000001.json")
    assert result["opponents"] == "random legal play" and result["opponent"] is None
    assert result["beats_random"] == (result["low"] > 0)
    assert result["tables"] == 12 and result["threads"] == 2 and result["seed"] == 5  # review, point 3
    assert "against random legal play" in capsys.readouterr().out
    assert [path.name for path in a.parent.glob("evaluation-*")] == ["evaluation-000001.json"]
    # As before spec 015, this file is replaced by the next call: nothing changes without --opponent.
    assert run_evaluation("--checkpoint", a, "--deals", 24, "--tables", 12, "--seed", 15) == 0
    assert result_of(a.parent / "evaluation-000001.json")["seed"] == 15


def test_c11_both_sample_from_their_policy_each_with_its_own_seeded_generator(tmp_path, monkeypatch):
    engine()
    from tarok_train import evaluate

    a = trained_checkpoint(tmp_path, "a", seed=8)
    b = trained_checkpoint(tmp_path, "b", seed=9)
    seeds = []
    real = evaluate.network_agent

    def spy(net, cfg_model, device, seed):
        seeds.append(seed)
        return real(net, cfg_model, device, seed)

    monkeypatch.setattr(evaluate, "network_agent", spy)
    args = ("--checkpoint", a, "--opponent", b, "--deals", 24, "--tables", 12)
    assert run_evaluation(*args, "--seed", 5) == 0
    assert len(seeds) == 2 and seeds[0] != seeds[1]  # two generators, two seeds
    first = result_of(versus(a, b))
    assert first["agent"] == "samples from its policy"
    assert "samples from its policy" in first["opponents"]
    assert run_evaluation(*args, "--seed", 5, "--overwrite") == 0
    again = result_of(versus(a, b))
    assert again["values"] == first["values"]  # seeded: the same deals and the same samples
    assert run_evaluation(*args, "--seed", 15, "--overwrite") == 0
    other = result_of(versus(a, b))
    assert other["values"] != first["values"]
    # Sampling networks do not play a fixed rule: the result is not the one of two equal rules.
    assert np.abs(np.array(first["rewards"])).max() > 0.01


def test_r1_two_environments_of_one_evaluation_do_not_get_the_same_random_numbers():
    engine_env = engine()
    from tarok_train import evaluate, model

    cfg = ci_cfg()
    torch.manual_seed(0)
    net = model.build_network(cfg["model"], "mlp").eval()  # untrained: close to uniform over the legal slots
    make_agent = evaluate.network_agent(net, cfg["model"], CPU, seed=3)
    obs = engine_env.EngineEnv(64, seed=11, threads=1).observe()
    rows = np.arange(64)
    first = make_agent(64)(rows, obs)
    second = make_agent(64)(rows, obs)  # the agent of the next chunk or seating, here on the same decisions
    # Before the review fix every environment got a new generator with the same seed: the two were equal.
    assert not np.array_equal(first, second)
    # The evaluation as a whole stays seeded.
    again = evaluate.network_agent(net, cfg["model"], CPU, seed=3)
    assert np.array_equal(again(64)(rows, obs), first) and np.array_equal(again(64)(rows, obs), second)


def chunk_correlation(values, chunks: int) -> float:
    """The mean correlation, over the pairs of chunks, of the deal values at the same table index."""
    by_chunk = np.array(values).reshape(chunks, -1)
    return float(np.corrcoef(by_chunk)[np.triu_indices(chunks, 1)].mean())


def test_r1_the_deal_values_of_one_table_index_are_not_tied_from_chunk_to_chunk(tmp_path):
    engine()
    # The recipe of criterion 15, small: a network after two updates against itself. Such a policy reads
    # the cards weakly, so with the same random numbers in every chunk a table index played nearly the same
    # deal after deal (measured before the fix: correlation +0.72), and the interval was far too narrow.
    a = trained_checkpoint(tmp_path, "a", seed=8, updates=2)
    for seed in (100, 120):
        assert run_evaluation("--checkpoint", a, "--opponent", a, "--deals", 800, "--tables", 200, "--seed", seed, "--overwrite") == 0
        correlation = chunk_correlation(result_of(versus(a, a))["values"], chunks=4)
        assert abs(correlation) < 0.25, (seed, correlation)  # 200 pairs per correlation: about 0.07 by chance
    # The agent's own random numbers, against random legal play: the untrained network of the same recipe.
    assert run_evaluation("--checkpoint", a, "--deals", 800, "--tables", 200, "--seed", 100, "--untrained") == 0
    correlation = chunk_correlation(result_of(a.parent / "evaluation-untrained.json")["values"], chunks=4)
    assert abs(correlation) < 0.25, correlation


def test_r1_the_result_does_not_depend_on_the_number_of_threads(tmp_path):
    engine()
    a = trained_checkpoint(tmp_path, "a", seed=8)
    b = trained_checkpoint(tmp_path, "b", seed=9)
    args = ("--checkpoint", a, "--opponent", b, "--deals", 36, "--tables", 12, "--seed", 5, "--overwrite")
    assert run_evaluation(*args, threads=1) == 0
    one = result_of(versus(a, b))
    assert run_evaluation(*args, threads=3) == 0
    three = result_of(versus(a, b))
    assert one["threads"] == 1 and three["threads"] == 3
    assert one["rewards"] == three["rewards"]


def sampling_checkpoint(folder: Path, name: str, seed: int, kind: str, updates: int, **model) -> Path:
    """A checkpoint with its own model sizes whose policy depends strongly on what the network reads."""
    cfg = ci_cfg(**{"run.name": name, "run.seed": seed, "run.network": kind, **{f"model.{key}": value for key, value in model.items()}})
    trainer = train.Trainer(cfg, CPU)
    with torch.no_grad():
        trainer.net.heads.policy.weight.mul_(10.0)  # far from uniform: another trunk gives other actions
    trainer.updates = updates
    path = run_folder(folder, name, kind) / f"checkpoint-{updates:06d}.pt"
    trainer.save(path)
    trainer.close()
    return path


def reference_rewards(engine_env, seeds, n, agent, others, seed) -> np.ndarray:
    """One seat of `agent` among three of `others`, written out apart from evaluate.py.

    agent, others: (network, its model config). A fresh actor for every call, so nothing is carried in a
    cache from step to step. One generator per network for the whole evaluation: `seed` and `seed` + 1.
    """
    from tarok_train.model import Actor

    generators = (torch.Generator().manual_seed(seed), torch.Generator().manual_seed(seed + 1))

    def ask(who, rows, obs):
        net, cfg_model = (agent, others)[who]
        actor = Actor(net, None, n, CPU, cfg_model)
        action, _, _ = actor.act(
            rows, obs["seat"][rows], obs["deal"][rows], obs["n_tokens"][rows], np.zeros(len(rows), dtype=bool),
            torch.from_numpy(obs["tokens"][rows]).long(), torch.from_numpy(obs["state"][rows]), torch.from_numpy(obs["mask"][rows]),
            generators[who],
        )  # fmt: skip
        return action.numpy()

    blocks = []
    for chunk_seed in seeds:
        block = np.zeros((n, 4), dtype=np.float32)
        for seat in range(4):
            env = engine_env.EngineEnv(n, chunk_seed, 2)
            open_tables = np.ones(n, dtype=bool)
            while open_tables.any():
                obs = env.observe()
                is_mine = open_tables & (obs["seat"] == seat)
                action = np.zeros(n, dtype=np.int64)
                if (~is_mine).any():
                    action[~is_mine] = ask(1, np.flatnonzero(~is_mine), obs)
                if is_mine.any():
                    action[is_mine] = ask(0, np.flatnonzero(is_mine), obs)
                done, reward, _ = env.step(action)
                ended = done & open_tables
                block[ended, seat] = reward[ended, seat]
                open_tables &= ~done
        blocks.append(block)
    return np.concatenate(blocks)


def network_of(path: Path):
    """(network, model config, class name) of a checkpoint, built here and not by evaluate.py."""
    from tarok_train import model

    data = torch.load(path, map_location="cpu", weights_only=True)
    net = model.build_network(data["config"]["model"], data["network"])
    net.load_state_dict(data["model"])
    return net.eval(), data["config"]["model"], type(net).__name__


def test_r2_two_networks_of_different_kinds_and_sizes_each_play_with_their_own_config(tmp_path, monkeypatch):
    engine_env = engine()
    from tarok_train import evaluate

    # An MLP and a real transformer (its policy reads its trunk), each with model sizes of its own.
    a = sampling_checkpoint(tmp_path, "amlp", seed=8, kind="mlp", updates=3, mlp_hidden=[96, 48])
    b = sampling_checkpoint(
        tmp_path, "btr", seed=9, kind="transformer", updates=7,
        width=64, layers=2, heads=2, feed_forward=128, cache_initial_tokens=16, act_bucket_tokens=4,
    )  # fmt: skip
    net_a, cfg_a, kind_a = network_of(a)
    net_b, cfg_b, kind_b = network_of(b)
    assert (kind_a, kind_b) == ("MLPNet", "TransformerNet") and cfg_a != cfg_b

    built = []

    class Spy(evaluate.Actor):
        def __init__(self, net, frozen_net, n_tables, device, cfg_model):
            built.append((type(net).__name__, json.dumps(cfg_model, sort_keys=True)))
            super().__init__(net, frozen_net, n_tables, device, cfg_model)

    monkeypatch.setattr(evaluate, "Actor", Spy)
    own = {(kind_a, json.dumps(cfg_a, sort_keys=True)), (kind_b, json.dumps(cfg_b, sort_keys=True))}

    # The transformer through the opponent's path.
    assert run_evaluation("--checkpoint", a, "--opponent", b, "--deals", 16, "--tables", 8, "--seed", 5) == 0
    assert set(built) == own  # each network acts with the model config of its own checkpoint
    result = result_of(versus(a, b))
    expected = reference_rewards(engine_env, [5, 6], 8, (net_a, cfg_a), (net_b, cfg_b), seed=5)
    assert np.abs(expected).max() > 0.01
    assert np.allclose(result["rewards"], expected, atol=1e-6)
    # The reference can tell a mix-up: the same two networks the other way round give other numbers.
    swapped_reference = reference_rewards(engine_env, [5, 6], 8, (net_b, cfg_b), (net_a, cfg_a), seed=5)
    assert not np.allclose(expected, swapped_reference, atol=1e-6)

    # The transformer through the agent's path, the MLP as the opponent.
    built.clear()
    assert run_evaluation("--checkpoint", b, "--opponent", a, "--deals", 16, "--tables", 8, "--seed", 5) == 0
    assert set(built) == own
    assert np.allclose(result_of(versus(b, a))["rewards"], swapped_reference, atol=1e-6)


def test_c12_the_result_names_both_checkpoints_and_never_overwrites_the_one_against_random_play(tmp_path, capsys):
    engine()
    a = trained_checkpoint(tmp_path, "a", seed=8)
    b = rule_checkpoint(tmp_path, "b", seed=9, updates=7, lowest=True)
    base = ("--checkpoint", a, "--deals", 24, "--tables", 12, "--seed", 5, "--records", 4)
    assert run_evaluation(*base) == 0
    random_file = a.parent / "evaluation-000001.json"
    random_bytes = random_file.read_bytes()
    random_records = {path.name: path.read_bytes() for path in (a.parent / "records-000001").glob("*.json")}
    assert len(random_records) == 4
    capsys.readouterr()

    assert run_evaluation(*base, "--opponent", b) == 0
    out = capsys.readouterr().out
    assert random_file.read_bytes() == random_bytes
    assert {path.name: path.read_bytes() for path in (a.parent / "records-000001").glob("*.json")} == random_records
    name = f"000001-vs-b-mlp-{STAMP}-000007"  # A's updates, B's run folder, B's updates
    result = result_of(a.parent / f"evaluation-{name}.json")
    assert result["checkpoint"] == "checkpoint-000001.pt" and result["run"] == "a-mlp" and result["updates"] == 1
    assert result["opponent"] == {"checkpoint": "checkpoint-000007.pt", "run": "b-mlp", "updates": 7, "folder": f"b-mlp-{STAMP}"}
    assert "b-mlp" in result["opponents"] and "checkpoint-000007.pt" in result["opponents"]
    assert result["seed"] == 5 and result["deals"] == 24 and result["tables"] == 12 and result["threads"] == 2
    assert "beats_random" not in result  # that name belongs to the evaluation against random play
    assert result["above_zero"] == (result["low"] > 0) and result["below_zero"] == (result["high"] < 0)
    assert "random legal play" not in out and "b-mlp" in out and "checkpoint-000007.pt" in out
    # Records with an opponent: their own folder, and they replay to the same entries (criterion 15).
    assert result["records_exported"] == result["records_replayed_equal"] == 4
    assert len(list((a.parent / f"records-{name}").glob("*.json"))) == 4
    assert sorted(p.name for p in a.parent.glob("evaluation-*")) == [f"evaluation-{name}.json", "evaluation-000001.json"]
    assert not list(b.parent.glob("evaluation-*"))  # the result lies next to A, not next to B


def test_r3_an_existing_result_is_not_replaced_without_overwrite(tmp_path, capsys):
    engine()
    a = trained_checkpoint(tmp_path, "a", seed=8)
    b = trained_checkpoint(tmp_path, "b", seed=9)
    base = ("--checkpoint", a, "--opponent", b, "--deals", 24, "--tables", 12)
    assert run_evaluation(*base, "--seed", 5, "--records", 4) == 0
    path = versus(a, b)
    records = a.parent / path.stem.replace("evaluation-", "records-")
    kept = path.read_bytes()
    assert len(list(records.glob("*.json"))) == 4
    capsys.readouterr()
    # The same pair with another seed, other deals or other tables has the same file name.
    for other in (("--seed", 15), ("--seed", 5, "--deals", 12), ("--seed", 5, "--tables", 24)):
        assert run_evaluation(*base, *other) == 2
        out = capsys.readouterr().out
        assert path.name in out and "--overwrite" in out and out.count("\n") == 1
        assert path.read_bytes() == kept
    # With the flag it is replaced, and records of the earlier call do not stay behind.
    assert run_evaluation(*base, "--seed", 15, "--records", 2, "--overwrite") == 0
    assert result_of(path)["seed"] == 15
    assert len(list(records.glob("*.json"))) == 2


def test_r3_two_opponents_with_one_run_name_from_two_run_folders_give_two_files(tmp_path):
    engine()
    a = trained_checkpoint(tmp_path, "a", seed=8)
    # Two runs with the same name, as three runs of one config with three seeds would be.
    b1 = trained_checkpoint(tmp_path, "b", seed=9, stamp="20260101-000000")
    b2 = trained_checkpoint(tmp_path, "b", seed=10, stamp="20260102-000000")
    for b in (b1, b2):
        assert run_evaluation("--checkpoint", a, "--opponent", b, "--deals", 24, "--tables", 12, "--seed", 5) == 0
    first, second = result_of(versus(a, b1)), result_of(versus(a, b2))
    assert versus(a, b1) != versus(a, b2)
    assert first["opponent"]["run"] == second["opponent"]["run"] == "b-mlp"
    assert first["opponent"]["folder"] == "b-mlp-20260101-000000" and second["opponent"]["folder"] == "b-mlp-20260102-000000"
    assert first["values"] != second["values"]
    assert len(list(a.parent.glob("evaluation-*"))) == 2


def test_c14_an_opponent_of_another_encoding_version_is_refused(tmp_path, capsys):
    engine()
    a = trained_checkpoint(tmp_path, "a", seed=8)
    b = trained_checkpoint(tmp_path, "b", seed=9)
    data = torch.load(b, map_location="cpu", weights_only=True)
    data["encoding_version"] += 1
    torch.save(data, b)
    assert run_evaluation("--checkpoint", a, "--opponent", b, "--deals", 24, "--tables", 12, "--seed", 5) == 2
    out = capsys.readouterr().out
    assert "opponent" in out and "encoding version" in out
    assert not list(a.parent.glob("evaluation-*"))
    # The same check for A stays.
    assert run_evaluation("--checkpoint", b, "--opponent", a, "--deals", 24, "--tables", 12, "--seed", 5) == 2


def test_c14_a_seed_range_that_holds_the_training_seed_of_a_or_of_b_is_refused(tmp_path, capsys):
    engine()
    from tarok_train import evaluate

    a = trained_checkpoint(tmp_path, "a", seed=8)
    b = trained_checkpoint(tmp_path, "b", seed=11)
    base = ("--checkpoint", a, "--opponent", b, "--deals", 24)
    assert run_evaluation(*base, "--tables", 24, "--seed", 11) == 2  # one chunk, the opponent's seed
    out = capsys.readouterr().out
    assert "opponent" in out and "11" in out
    assert run_evaluation(*base, "--tables", 12, "--seed", 10) == 2  # chunks 10 and 11
    assert "opponent" in capsys.readouterr().out
    assert run_evaluation(*base, "--tables", 24, "--seed", 8) == 2  # A's own seed, as before
    assert "opponent" not in capsys.readouterr().out
    assert not list(a.parent.glob("evaluation-*"))
    assert run_evaluation(*base, "--tables", 12, "--seed", 9) == 0  # chunks 9 and 10: neither seed
    assert run_evaluation(*base, "--tables", 24, "--seed", 12, "--overwrite") == 0
    # The function refuses too, whoever calls it.
    with pytest.raises(ValueError, match="opponent"):
        evaluate.evaluate(
            lambda n: evaluate.first_legal, deals=24, tables=12, seed=10, threads=1,
            make_others=lambda n: evaluate.first_legal, opponent_training_seed=11,
        )  # fmt: skip


def test_r8_a_path_that_is_no_checkpoint_is_refused_in_one_line(tmp_path, capsys):
    engine()
    a = trained_checkpoint(tmp_path, "a", seed=8)
    missing = tmp_path / "nowhere" / "checkpoint-000001.pt"
    not_one = tmp_path / "notes.pt"
    not_one.write_text("this is not a checkpoint", encoding="utf-8")
    base = ("--deals", 24, "--tables", 12, "--seed", 5)
    capsys.readouterr()  # what making the checkpoint printed
    for args, who, path in (
        (("--checkpoint", missing), "the checkpoint", missing),
        (("--checkpoint", a, "--opponent", missing), "the opponent's checkpoint", missing),
        (("--checkpoint", not_one), "the checkpoint", not_one),
        (("--checkpoint", a, "--opponent", not_one), "the opponent's checkpoint", not_one),
        (("--checkpoint", tmp_path), "the checkpoint", tmp_path),  # a folder
    ):
        assert run_evaluation(*args, *base) == 2
        out = capsys.readouterr().out
        assert out.count("\n") == 1 and who in out and str(path) in out, out
    assert not list(a.parent.glob("evaluation-*"))


def test_r8_a_checkpoint_of_the_mock_environment_is_refused_as_agent_and_as_opponent(tmp_path, capsys):
    engine()
    a = trained_checkpoint(tmp_path, "a", seed=8)
    trainer = train.Trainer(small(), CPU)  # the mock environment: the same network sizes, another game
    trainer.run(1)
    mock = tmp_path / "mock-run" / "checkpoint-000001.pt"
    trainer.save(mock)
    trainer.close()
    base = ("--deals", 24, "--tables", 12, "--seed", 5)
    capsys.readouterr()  # what making the checkpoints printed
    assert run_evaluation("--checkpoint", mock, *base) == 2
    out = capsys.readouterr().out
    assert "mock" in out and "the checkpoint" in out and out.count("\n") == 1
    assert run_evaluation("--checkpoint", a, "--opponent", mock, *base) == 2
    out = capsys.readouterr().out
    assert "mock" in out and "opponent" in out and out.count("\n") == 1
    assert not list(a.parent.glob("evaluation-*")) and not list(mock.parent.glob("evaluation-*"))


def test_c11_the_function_wants_random_play_or_an_opponent_not_both_and_not_neither():
    from tarok_train import evaluate

    with pytest.raises(ValueError):
        evaluate.evaluate(lambda n: evaluate.first_legal, deals=4, tables=4, seed=1, threads=1)
    with pytest.raises(ValueError):
        evaluate.evaluate(
            lambda n: evaluate.first_legal, deals=4, tables=4, seed=1, threads=1,
            others=evaluate.first_legal, make_others=lambda n: evaluate.first_legal,
        )  # fmt: skip


def test_c12_the_report_names_the_opponent_and_does_not_say_beats_random(tmp_path):
    from tarok_train import smoke_report

    folder = folder_for_the_report(tmp_path, summary_for_the_report())
    common = {"deals": 24, "seed": 5, "mean": 0.1, "low": 0.05, "high": 0.15, "records_exported": 0, "records_replayed_equal": 0, "checkpoint": "checkpoint-000001.pt"}
    (folder / "evaluation-000001.json").write_text(
        json.dumps({**common, "opponents": "random legal play", "opponent": None, "beats_random": True}), encoding="utf-8"
    )
    (folder / "evaluation-000001-vs-b-mlp-20260101-000000-000007.json").write_text(
        json.dumps({**common, "opponents": "b-mlp, checkpoint-000007.pt (7 updates), samples from its policy", "above_zero": True, "below_zero": False}),
        encoding="utf-8",
    )
    text = smoke_report.report(folder)
    assert text.count("Beats random: yes") == 1
    assert "against b-mlp, checkpoint-000007.pt (7 updates)" in text
    assert "The whole interval is above zero" in text


def bounded_statistics(values) -> dict:
    """Two statistics of the deal values whose spread does not grow with the size of a deal.

    sign: the share of deals with a positive value minus the share with a negative one, with its
    95% interval (the values -1, 0, +1; 1.96 standard errors, as for a binomial share). clipped: the
    mean of the values clipped at -2 and +2, with its interval. Each interval is at most 0.09 (sign)
    and 0.18 (clipped) wide on 2,000 deals, however wide the plain interval is, and a bias of the
    single seat shows in them. Their expected value against itself is zero only if the deal values
    are symmetric about zero; the plain mean is zero by the symmetry of the seats (the four rewards
    of a play add up to zero), but a skewed spread of the values moves the share and the clipped
    mean a little. So they are bounded in size, not asked to hold zero in their interval.
    """
    from tarok_train import evaluate

    values = np.asarray(values, dtype=np.float64)
    return {"sign": evaluate.interval(np.sign(values)), "clipped": evaluate.interval(np.clip(values, -2.0, 2.0))}


@pytest.mark.slow
@pytest.mark.gpu
def test_c15_a_checkpoint_against_itself_on_2000_deals_has_an_interval_that_includes_zero(tmp_path):
    engine()
    from tarok_train import evaluate

    assert train.main(["--config", str(CI_TOML), "--out-dir", str(tmp_path), "--updates", "2", "--device", "cuda"]) == 0
    checkpoint = tmp_path / "smoke-a-ci-mlp" / "checkpoint-000002.pt"
    path = checkpoint.parent / "evaluation-000002-vs-smoke-a-ci-mlp-000002.json"
    base = ["--checkpoint", str(checkpoint), "--opponent", str(checkpoint), "--deals", "2000"]
    assert evaluate.main([*base, "--seed", "5151", "--records", "20"]) == 0
    result = result_of(path)
    assert result["deals"] == 2000 and result["device"] == "cuda" and result["tables"] == 500
    assert result["records_exported"] == result["records_replayed_equal"] == 20
    # The plain interval is measured, not bounded. On deal flow A it was under 0.2 wide. On deal flow
    # B (spec 009) a network after two updates announces and doubles nearly at random: a deal can be
    # worth thousands of points, and the interval of 2,000 deals is about 2.8 wide. It cannot resolve
    # a bias of the single seat of 1 reward. So the bound is on two statistics that stay small.
    # The bounds. Against itself on flow B the measured sizes over these 21 runs were at most 0.095
    # (clipped) and 0.048 (sign); pooled over the 42,000 deals +0.049 and +0.027, the offset of a
    # skewed spread (spec 009, Results). A bias of the single seat of about 1 reward would move
    # both by more than the bounds (inferred, not measured: no biased network was built).
    bounds = {"sign": 0.12, "clipped": 0.25}
    sizes = {name: [] for name in bounds}

    def bounded(values, seed):
        for name, (mean, low, high) in bounded_statistics(values).items():
            assert high - low < 2 * bounds[name], (seed, name, high - low)
            sizes[name].append(mean)

    bounded(result["values"], 5151)
    # One seed proves little: a right interval misses zero in one of 20 cases, and before the review fix
    # of spec 015 this recipe missed it in 8 of 20 (the same random numbers in every chunk). So: 20 seeds.
    # With a right interval more than 3 of 20 miss in under 2% of the cases; with the old code 3 or
    # fewer missed in under 2%. The same count for the two bounded statistics.
    outside = {"plain": [], "sign": [], "clipped": []}  # sign and clipped: measured, not asserted
    widths = []
    correlations = []
    for seed in range(6000, 6400, 20):  # no chunk holds the training seed 8
        assert evaluate.main([*base, "--seed", str(seed), "--overwrite"]) == 0
        result = result_of(path)
        outside["plain"].append(not (result["low"] <= 0.0 <= result["high"]))
        for name, (_, low, high) in bounded_statistics(result["values"]).items():
            outside[name].append(not (low <= 0.0 <= high))
        bounded(result["values"], seed)
        widths.append(result["high"] - result["low"])
        correlations.append(chunk_correlation(result["values"], chunks=4))
        assert result["above_zero"] == (result["low"] > 0) and result["below_zero"] == (result["high"] < 0)
    print(
        "20 seeds, zero outside the interval: "
        + ", ".join(f"{name} {sum(flags)}" for name, flags in outside.items())
        + f"; plain width {min(widths):.3f} to {max(widths):.3f}; chunk correlation {np.mean(correlations):+.4f}; "
        + "; ".join(f"{name} from {min(v):+.4f} to {max(v):+.4f}, mean {np.mean(v):+.4f}" for name, v in sizes.items())
    )
    assert sum(outside["plain"]) <= 3, outside["plain"]
    for name, values in sizes.items():
        assert max(abs(v) for v in values) < bounds[name], (name, values)
    assert abs(float(np.mean(correlations))) < 0.1, correlations
