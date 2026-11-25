"""Tests for spec 008 (pipeline smoke run on the real engine).

The name of each test starts with its criterion number. `test_d130_...` is the
stale-module check of decision D130. The three long runs of criteria 1, 7 and 8
are not tests: pytest covers the small config `configs/smoke-a-ci.toml`.

The engine's module is built with `maturin develop --release` in
`src/train/tarok_env`. Where it is not built the tests that need it are skipped;
with TAROK_REQUIRE_ENV=1 (set in CI) a missing module is a failure instead.
"""

import copy
import json
import math
import os
import re
import shutil
import time
from pathlib import Path

import numpy as np
import pytest
import torch

from tarok_train import contract as C
from tarok_train import ppo, rollout, train

from test_spec007 import CPU, ENCODING_MD, MOCK_TOML, add_rows, cfg_with, collected, scalar_tags, table_with, tables_of

CONFIGS = Path(__file__).resolve().parents[1] / "configs"
CI_TOML = CONFIGS / "smoke-a-ci.toml"
SMOKE = {"mlp": CONFIGS / "smoke-a-mlp.toml", "transformer": CONFIGS / "smoke-a-transformer.toml"}
ACTING_PHASES = C.PHASES[:-1]  # every phase but "over"


def engine():
    """The module `tarok_train.env` (it needs the built `tarok_env`), or a skip."""
    if os.environ.get("TAROK_REQUIRE_ENV") != "1":
        pytest.importorskip("tarok_env", reason="the tarok_env module is not built (maturin develop --release)")
    from tarok_train import env as engine_env

    return engine_env


def ci_cfg(**overrides):
    """The small engine config with 'section.key' overrides."""
    cfg = train.load_config(CI_TOML)
    for dotted, value in overrides.items():
        section, key = dotted.split(".")
        assert key in cfg[section], dotted
        cfg[section][key] = value
    return cfg


@pytest.fixture(scope="module")
def engine_run():
    """Four updates of the MLP on the engine, with a past checkpoint in play from the second update on."""
    engine()
    trainer = train.Trainer(ci_cfg(**{"run.network": "mlp", "opponents.snapshot_every": 1}), CPU)
    history = trainer.run(4)
    return trainer, history


def state_with(**blocks):
    """One state row with the given entries set: state_with(B1=6, B4=0) sets one bit in each block."""
    state = np.zeros(C.STATE_FEATURES, dtype=np.float32)
    for block, entry in blocks.items():
        for one in np.atleast_1d(entry):
            assert 0 <= one < C.STATE_BLOCKS[block][1], (block, one)
            state[C.STATE_BLOCKS[block][0] + one] = 1.0
    return state


# --------------------------------------------------------------------------
# Criterion 1 (and 18): the loop runs on the real environment
# --------------------------------------------------------------------------


@pytest.mark.parametrize("kind", ["mlp", "transformer"])
def test_c01_c18_two_updates_of_the_ci_config_run_on_the_engine_and_end_with_exit_code_0(kind, tmp_path, capsys):
    engine()
    code = train.main(["--config", str(CI_TOML), "--network", kind, "--out-dir", str(tmp_path)])
    assert code == 0
    out = tmp_path / f"smoke-a-ci-{kind}"
    assert (out / "checkpoint-000002.pt").exists()
    summary = json.loads((out / "summary.json").read_text(encoding="utf-8"))
    assert summary["updates"] == 2
    assert summary["finished"] is True
    assert summary["environment"] == "engine"
    assert summary["illegal_actions"] == 0  # criterion 2
    assert summary["decision_age_max"] == 0  # criterion 5
    assert "2 updates" in capsys.readouterr().out


def test_c01_the_smoke_configs_carry_the_settings_of_the_spec():
    mock = train.load_config(MOCK_TOML)
    changed = {("model", "policy_init_scale"): 0.3, ("ppo", "value_head_weight"): 0.1}  # D127
    loaded = {}
    for network, path in SMOKE.items():
        cfg = loaded[network] = train.load_config(path)
        assert cfg["run"]["network"] == network
        assert cfg["run"]["updates"] == 200  # D124 A
        assert cfg["run"]["time_limit_minutes"] == 90
        assert cfg["run"]["fault"] == "none"
        assert cfg["env"]["kind"] == "engine"
        assert cfg["env"]["tables"] == 512  # D124 B
        assert cfg["ppo"]["seat_deals_per_update"] == 2048  # D124 B
        assert cfg["entropy_control"]["enabled"] is True  # D127
        for (section, key), value in changed.items():
            assert cfg[section][key] == value, (section, key)
        # Everything else as configs/mock.toml.
        for section in ("model", "ppo", "opponents"):
            for key, value in cfg[section].items():
                if (section, key) not in changed:
                    assert value == mock[section][key], (section, key)
    # The two runs differ in the network only.
    mlp, transformer = copy.deepcopy(loaded["mlp"]), copy.deepcopy(loaded["transformer"])
    del mlp["run"]["network"], transformer["run"]["network"]
    assert mlp == transformer


def test_c01_the_environment_kind_decides_which_env_keys_a_config_needs():
    cfg = ci_cfg()
    assert set(cfg["env"]) == {"kind", "tables", "threads"}
    train.validate_config(cfg)
    mixed = copy.deepcopy(cfg)
    mixed["env"]["mode"] = "learnable"  # a key of the mock
    with pytest.raises(train.ConfigError, match="mode"):
        train.validate_config(mixed)
    unknown = copy.deepcopy(cfg)
    unknown["env"]["kind"] = "cards"
    with pytest.raises(train.ConfigError, match="cards"):
        train.validate_config(unknown)
    mock = cfg_with()
    mock["env"]["threads"] = 2  # a key of the engine
    with pytest.raises(train.ConfigError, match="threads"):
        train.validate_config(mock)


def test_c01_a_run_that_hits_its_time_limit_is_reported_as_not_finished(tmp_path, capsys):
    engine()
    args = ["--config", str(CI_TOML), "--out-dir", str(tmp_path), "--updates", "50", "--time-limit-minutes", "0.0001"]
    code = train.main(args)
    assert code != 0
    summary = json.loads((tmp_path / "smoke-a-ci-mlp" / "summary.json").read_text(encoding="utf-8"))
    assert summary["finished"] is False
    assert 1 <= summary["updates"] < 50
    assert "not finished" in capsys.readouterr().out


# --------------------------------------------------------------------------
# Criterion 2: no illegal action
# --------------------------------------------------------------------------


def test_c02_the_rollout_refuses_a_stored_action_that_its_mask_forbids():
    store = rollout.Rollout(zero_sum=False)
    add_rows(store, [0], [0], [0], action=[5])  # the helper's mask allows every slot
    n = 1
    mask = np.zeros((n, C.N_ACTIONS), dtype=bool)
    mask[0, 5] = True
    row = dict(
        tokens=np.zeros((n, C.MAX_TOKENS, 5), dtype=np.int16),
        tables=np.array([1]),
        seat=np.array([0]),
        deal=np.array([0]),
        n_tokens=np.array([1]),
        state=np.zeros((n, C.STATE_FEATURES), dtype=np.float32),
        mask=mask,
        critic_extra=np.zeros((n, C.CRITIC_EXTRA), dtype=np.float32),
        partner_target=np.zeros(n, dtype=np.int64),
        card_target=np.zeros((n, C.N_CARDS), dtype=np.int64),
        logp=np.zeros(n, dtype=np.float32),
        value=np.zeros((n, C.N_SCORE_PARTS), dtype=np.float32),
    )
    store.add(action=np.array([5]), **row)  # the legal slot
    before = store.pending_rows
    with pytest.raises(ValueError, match="illegal"):
        store.add(action=np.array([6]), **row)
    assert store.pending_rows == before  # nothing of the refused rows was stored


def test_c02_the_engine_refuses_an_illegal_slot_and_a_run_ends_with_zero_refusals(engine_run):
    engine_env = engine()
    env = engine_env.EngineEnv(4, seed=2, threads=1)
    illegal = (~env.observe()["mask"]).argmax(axis=1)
    with pytest.raises(ValueError, match="illegal"):
        env.step(illegal)
    assert env.illegal_actions == 1
    trainer, history = engine_run
    assert trainer.env.illegal_actions == 0
    assert all(stats["env/illegal_actions"] == 0 for stats in history)


# --------------------------------------------------------------------------
# Criterion 3: no NaN and no infinity
# --------------------------------------------------------------------------


def test_c03_every_logged_number_of_a_run_on_the_engine_is_finite(engine_run):
    _, history = engine_run
    assert len(history) == 4
    for stats in history:
        for name, value in stats.items():
            assert math.isfinite(value), name
    for wanted in ("loss/total", "grad/actor", "grad/critic", "reward/mean_abs", "reward/std"):
        assert wanted in history[0]


@pytest.mark.parametrize("bad", [float("nan"), float("inf")])
def test_c03_a_logged_number_that_is_not_finite_stops_the_run(bad, monkeypatch):
    trainer = train.Trainer(cfg_with(**{"env.tables": 8, "ppo.seat_deals_per_update": 24, "ppo.update_batch": 16}), CPU)
    real = ppo.update

    def broken(*args, **kwargs):
        stats = real(*args, **kwargs)
        stats["loss/q"] = bad
        return stats

    monkeypatch.setattr(ppo, "update", broken)
    with pytest.raises(FloatingPointError, match="loss/q"):
        trainer.run(1)


# --------------------------------------------------------------------------
# Criterion 4: rewards add up to zero, parts to the reward
# --------------------------------------------------------------------------


def test_c04_the_storage_refuses_a_deal_whose_four_rewards_do_not_add_up_to_zero():
    def parts_of(reward):
        parts = np.zeros((1, 4, C.N_SCORE_PARTS), dtype=np.float32)
        parts[0, :, 0] = reward
        return parts

    store = rollout.Rollout(zero_sum=True)
    add_rows(store, [0, 1], [0, 0], [0, 0])
    good = np.array([[1.0, -0.5, -0.25, -0.25]], dtype=np.float32)
    store.finish([0], [0], good, parts_of(good[0]))
    bad = np.array([[1.0, -0.5, -0.25, -0.2]], dtype=np.float32)
    with pytest.raises(ValueError, match="zero"):
        store.finish([1], [0], bad, parts_of(bad[0]))
    # The mock's rewards are a score, not zero-sum: its storage does not check.
    loose = rollout.Rollout(zero_sum=False)
    add_rows(loose, [1], [0], [0])
    loose.finish([1], [0], bad, parts_of(bad[0]))


def test_c04_on_the_engine_the_four_rewards_add_up_to_zero_and_the_ten_parts_to_the_reward():
    engine()
    trainer = train.Trainer(ci_cfg(), CPU)
    assert trainer.rollout.zero_sum is True
    deals = collected(trainer)
    assert deals.parts.shape[1] == 10
    assert np.abs(deals.parts.sum(axis=1) - deals.reward).max() < 1e-5
    # Before the first update no seat is played by a past checkpoint: all four seats of a deal are stored.
    keys = deals.table * 1_000_000 + deals.deal
    unique, inverse, counts = np.unique(keys, return_inverse=True, return_counts=True)
    assert len(unique) >= 8 and (counts == 4).all()
    sums = np.zeros(len(unique))
    np.add.at(sums, inverse, deals.reward.astype(np.float64))
    assert np.abs(sums).max() < 1e-5
    assert np.abs(deals.reward).max() > 0.01  # not trivially zero


# --------------------------------------------------------------------------
# Criterion 5: every trained decision was played under the current weights
# --------------------------------------------------------------------------


def test_c05_the_decision_age_is_zero_at_every_update_also_with_a_past_checkpoint_in_play(engine_run):
    trainer, history = engine_run
    assert [stats["rollout/decision_age_max"] for stats in history] == [0.0] * 4
    assert trainer.frozen_decisions > 0  # seats of a past checkpoint did play


# --------------------------------------------------------------------------
# Criterion 6: resume
# --------------------------------------------------------------------------


def test_c06_a_resumed_run_reaches_the_same_update_count_from_the_checkpoints_state(tmp_path, capsys):
    engine()
    base = ["--config", str(CI_TOML), "--out-dir"]
    assert train.main([*base, str(tmp_path / "whole"), "--updates", "3"]) == 0
    assert train.main([*base, str(tmp_path / "broken"), "--updates", "2"]) == 0
    folder = tmp_path / "broken" / "smoke-a-ci-mlp"
    checkpoint = folder / "checkpoint-000002.pt"
    data = torch.load(checkpoint, map_location="cpu", weights_only=True)
    assert data["updates"] == 2 and len(data["pool"]) >= 1

    # What a resumed trainer starts from is what the checkpoint holds.
    trainer = train.Trainer.load(checkpoint, CPU)
    assert trainer.updates == 2
    saved, loaded = data["optimizer"]["state"], trainer.optimizer.state_dict()["state"]
    assert saved.keys() == loaded.keys() and len(saved) > 0
    for key in saved:
        assert saved[key]["step"] == loaded[key]["step"]
        assert torch.equal(saved[key]["exp_avg"], loaded[key]["exp_avg"])
        assert torch.equal(saved[key]["exp_avg_sq"], loaded[key]["exp_avg_sq"])
    assert len(trainer.pool.snapshots) == len(data["pool"])
    for mine, theirs in zip(trainer.pool.snapshots, data["pool"]):
        assert all(torch.equal(mine[name], theirs[name]) for name in theirs)
    assert trainer.entropy.weights.tolist() == pytest.approx(data["entropy_control"]["weights"])
    assert trainer.entropy.integral.tolist() == pytest.approx(data["entropy_control"]["integral"])
    trainer.close()

    capsys.readouterr()
    assert train.main(["--resume", str(checkpoint), "--updates", "3"]) == 0
    assert "resumed from checkpoint-000002.pt at update 2" in capsys.readouterr().out
    assert (folder / "checkpoint-000003.pt").exists()
    resumed = torch.load(folder / "checkpoint-000003.pt", map_location="cpu", weights_only=True)
    whole = torch.load(tmp_path / "whole" / "smoke-a-ci-mlp" / "checkpoint-000003.pt", map_location="cpu", weights_only=True)
    assert resumed["updates"] == whole["updates"] == 3
    # The engine's state is restored (spec 006), so on the CPU the resumed run equals the unbroken one.
    assert resumed["decisions"] == whole["decisions"]
    assert all(torch.equal(resumed["model"][name], whole["model"][name]) for name in whole["model"])


# --------------------------------------------------------------------------
# Criterion 7: the evaluation against random legal play
# --------------------------------------------------------------------------


def test_c07_the_interval_is_the_mean_and_1_96_standard_errors_on_each_side():
    from tarok_train import evaluate

    mean, low, high = evaluate.interval(np.array([1.0, 2.0, 3.0, 4.0]))
    # standard deviation with n - 1: sqrt(5 / 3) = 1.29099; standard error 0.645497; half width 1.265175
    assert mean == pytest.approx(2.5)
    assert low == pytest.approx(2.5 - 1.265175, abs=1e-5)
    assert high == pytest.approx(2.5 + 1.265175, abs=1e-5)


def test_c07_each_deal_is_played_four_times_with_the_agent_in_another_seat():
    engine()
    from tarok_train import evaluate

    seats_asked = []

    def make_agent(n_tables):
        asked = set()
        seats_asked.append(asked)

        def agent(rows, obs):
            asked.update(obs["seat"][rows].tolist())
            return evaluate.first_legal(rows, obs)

        return agent

    # All four seats play the same fixed rule, so the four seatings are the same game. The
    # agent's rewards over its four seats are then the four rewards of one deal: they add up to zero.
    result = evaluate.evaluate(make_agent, deals=24, tables=12, seed=5, threads=2, others=evaluate.first_legal)
    assert seats_asked == [{0}, {1}, {2}, {3}] * 2  # two chunks of 12 tables; only its own seat is asked
    assert result["deals"] == 24
    by_seat = np.array(result["rewards"])  # (deals, 4): the agent's reward in each seating
    assert by_seat.shape == (24, 4)
    assert np.abs(by_seat).max() > 0.01
    # The first legal slot of the round of announcements is an announcement (spec 009), so
    # these deals end with rewards above 100. A 32-bit reward of that size is exact to about
    # 1e-5, not to 1e-6: the room grows with the largest reward (D147, 1 a).
    # Per deal, as in test_c21 of spec 006: the room of a deal is that of its own largest reward.
    room = 1e-6 * np.maximum(1.0, np.abs(by_seat).max(axis=1))
    assert (np.abs(by_seat.sum(axis=1)) < 10 * room).all()
    assert (np.abs(np.asarray(result["values"]) - by_seat.mean(axis=1)) <= room).all()
    assert abs(result["mean"]) < float(room.mean())
    # Other chunks play other deals.
    assert not np.allclose(by_seat[:12], by_seat[12:])


def test_c07_the_agent_gets_the_reward_of_its_own_seat():
    engine_env = engine()
    from tarok_train import evaluate

    # The agent plays the first legal slot, the others the last one. Played by hand on a second
    # environment of the same seed, the reward of seat 2 must be what the evaluation reports.
    def last_legal(rows, obs):
        mask = obs["mask"][rows]
        return mask.shape[1] - 1 - mask[:, ::-1].argmax(axis=1)

    n = 6
    seating = evaluate.play_seating(engine_env.EngineEnv(n, seed=9, threads=1), 2, evaluate.first_legal, last_legal)
    env = engine_env.EngineEnv(n, seed=9, threads=1)
    expected = np.zeros(n, dtype=np.float32)
    open_tables = np.ones(n, dtype=bool)
    rows = np.arange(n)
    while open_tables.any():
        obs = env.observe()
        action = np.where(obs["seat"] == 2, evaluate.first_legal(rows, obs), last_legal(rows, obs))
        done, reward, _ = env.step(action)
        ended = done & open_tables
        expected[ended] = reward[ended, 2]
        open_tables &= ~done
    assert np.array_equal(seating.reward, expected)
    assert np.abs(expected).max() > 0.01


def test_c07_c15_a_checkpoint_and_the_untrained_network_are_evaluated_from_the_command_line(tmp_path, capsys):
    engine()
    from tarok_train import evaluate

    assert train.main(["--config", str(CI_TOML), "--out-dir", str(tmp_path), "--updates", "1"]) == 0
    checkpoint = tmp_path / "smoke-a-ci-mlp" / "checkpoint-000001.pt"
    args = ["--checkpoint", str(checkpoint), "--deals", "24", "--tables", "12", "--seed", "5", "--threads", "2"]
    assert evaluate.main([*args, "--device", "cpu", "--records", "4"]) == 0
    result = json.loads((checkpoint.parent / "evaluation-000001.json").read_text(encoding="utf-8"))
    assert result["deals"] == 24 and result["seatings_per_deal"] == 4
    assert result["opponents"] == "random legal play"
    assert result["low"] <= result["mean"] <= result["high"]
    assert result["beats_random"] == (result["low"] > 0)
    assert result["records_exported"] == result["records_replayed_equal"] == 4  # criterion 15
    assert len(list((checkpoint.parent / "records-000001").glob("*.json"))) == 4
    assert evaluate.main([*args, "--device", "cpu", "--records", "0", "--untrained"]) == 0
    untrained = json.loads((checkpoint.parent / "evaluation-untrained.json").read_text(encoding="utf-8"))
    assert untrained["deals"] == 24 and untrained["checkpoint"] is None
    assert "95%" in capsys.readouterr().out


@pytest.mark.parametrize("kind", ["mlp", "transformer"])
def test_c07_the_network_agent_samples_legal_actions_for_its_seat(kind):
    engine()
    from tarok_train import evaluate

    cfg = ci_cfg(**{"run.network": kind})
    torch.manual_seed(0)
    net = train.build_network(cfg["model"], kind).eval()
    make_agent = evaluate.network_agent(net, cfg["model"], CPU, seed=3)
    result = evaluate.evaluate(make_agent, deals=8, tables=8, seed=6, threads=2, others=evaluate.RandomLegal(1))
    assert result["deals"] == 8 and np.isfinite(result["values"]).all()  # an illegal slot would have raised
    again = evaluate.evaluate(
        evaluate.network_agent(net, cfg["model"], CPU, seed=3), deals=8, tables=8, seed=6, threads=2, others=evaluate.RandomLegal(1)
    )
    assert result["values"] == again["values"]  # seeded: the same deals, the same play


# --------------------------------------------------------------------------
# Criterion 8: the fault switch of the control run
# --------------------------------------------------------------------------


def test_c08_the_fault_key_is_refused_unless_the_run_name_starts_with_control():
    with pytest.raises(train.ConfigError, match="control-"):
        train.validate_config(ci_cfg(**{"run.fault": "rewards_to_next_seat"}))
    train.validate_config(ci_cfg(**{"run.fault": "rewards_to_next_seat", "run.name": "control-smoke-a"}))
    with pytest.raises(train.ConfigError, match="fault"):
        train.validate_config(ci_cfg(**{"run.fault": "something", "run.name": "control-smoke-a"}))
    train.validate_config(ci_cfg(**{"run.name": "control-smoke-a"}))  # the name alone changes nothing


def test_c08_the_fault_hands_the_reward_of_every_seat_to_the_next_seat():
    engine()
    sound = train.Trainer(ci_cfg(), CPU)
    faulty = train.Trainer(ci_cfg(**{"run.fault": "rewards_to_next_seat", "run.name": "control-smoke-a"}), CPU)
    a, b = collected(sound), collected(faulty)
    # The same seed and the same weights: the same deals and the same actions.
    assert np.array_equal(a.table, b.table) and np.array_equal(a.seat, b.seat) and np.array_equal(a.deal, b.deal)
    assert np.array_equal(a.action, b.action)
    by_key = {(t, d, s): i for i, (t, d, s) in enumerate(zip(a.table, a.deal, a.seat))}
    moved = 0
    for i, (t, d, s) in enumerate(zip(b.table, b.deal, b.seat)):
        source = by_key[(t, d, (s - 1) % 4)]
        assert b.reward[i] == a.reward[source]
        assert np.array_equal(b.parts[i], a.parts[source])
        moved += b.reward[i] != a.reward[by_key[(t, d, s)]]
    # In a solo the three opponents earn the same, so the hand-over changes two rewards of four:
    # the declarer's and the next seat's. With a called partner it can change more.
    assert moved >= len(b) // 2


def test_c08_the_control_run_is_started_from_the_command_line(tmp_path):
    engine()
    args = ["--config", str(CI_TOML), "--out-dir", str(tmp_path), "--updates", "1"]
    with pytest.raises(train.ConfigError, match="control-"):
        train.main([*args, "--fault", "rewards_to_next_seat"])
    assert train.main([*args, "--fault", "rewards_to_next_seat", "--name", "control-smoke-a"]) == 0
    data = torch.load(tmp_path / "control-smoke-a-mlp" / "checkpoint-000001.pt", map_location="cpu", weights_only=True)
    assert data["config"]["run"]["fault"] == "rewards_to_next_seat"


# --------------------------------------------------------------------------
# Criteria 9 to 14: the measurements and monitors
# --------------------------------------------------------------------------


def test_c09_played_and_trained_decisions_per_second_are_logged(engine_run):
    _, history = engine_run
    for stats in history:
        played, trained = stats["speed/steps_per_second"], stats["speed/trained_per_second"]
        assert 0 < trained <= played
        assert stats["speed/trained_per_second"] == pytest.approx(stats["rollout/decisions"] / stats["time/seconds"])
        assert 0 < stats["rollout/trained_share"] <= 1


def test_c10_the_four_time_shares_add_up_to_95_to_100_percent_of_the_wall_time(engine_run):
    _, history = engine_run
    names = ("environment", "acting", "bookkeeping", "update")
    for stats in history:
        shares = [stats[f"time/{name}_share"] for name in names]
        assert all(share > 0 for share in shares)
        assert 0.95 <= sum(shares) <= 1.0 + 1e-9, shares


def test_c11_the_peak_gpu_memory_is_in_the_summary(tmp_path):
    engine()
    assert train.main(["--config", str(CI_TOML), "--out-dir", str(tmp_path), "--updates", "1"]) == 0
    summary = json.loads((tmp_path / "smoke-a-ci-mlp" / "summary.json").read_text(encoding="utf-8"))
    assert summary["device"] == "cpu"
    assert summary["peak_gpu_memory_mib"] is None and summary["peak_gpu_memory_reserved_mib"] is None
    for key in (
        "played",
        "trained",
        "played_per_second",
        "trained_per_second",
        "trained_share",
        "trained_share_own_seats",
        "time_shares",
        "seconds",
        "gpu_before",
        "gpu_after",
    ):
        assert key in summary
    assert summary["played_per_second"] == pytest.approx(summary["played"] / summary["seconds"])
    assert summary["trained_per_second"] == pytest.approx(summary["trained"] / summary["seconds"])
    assert summary["trained_share"] == pytest.approx(summary["trained"] / summary["played"])


def test_c09_to_c14_the_report_script_reads_the_numbers_from_a_run_folder(tmp_path):
    engine()
    from tarok_train import evaluate, smoke_report

    assert train.main(["--config", str(CI_TOML), "--out-dir", str(tmp_path)]) == 0
    folder = tmp_path / "smoke-a-ci-mlp"
    assert evaluate.main(["--checkpoint", str(folder / "checkpoint-000002.pt"), "--deals", "8", "--tables", "8", "--seed", "5", "--device", "cpu"]) == 0
    text = smoke_report.report(folder)
    for wanted in (
        "| Updates logged | 2 |",
        "| Illegal actions refused by the engine (criterion 2) | 0 |",
        "| Logged numbers that are not finite (criterion 3) | 0 of ",
        "| Oldest trained decision, in updates (criterion 5) | 0 |",
        "Decisions played per second (criterion 9; old G3: 10,000)",
        "Time split (criterion 10)",
        "Share of played decisions trained on (all seats' decisions count as played)",
        "Peak GPU memory of the run's tensors (criterion 11; `max_memory_allocated`)",
        "Peak GPU memory held by PyTorch's allocator (`max_memory_reserved`)",
        "Memory in use on the whole card at the end, all programs (nvidia-smi)",
        "| Entropy over the decisions with two or more legal slots, play |",
        "| Share of decisions with one legal slot, play |",
        "Collapse by the spec's fixed limit (entropy under 0.05 at the last update)",
        "Collapse relative to the phase's target",
        "Tokens per seat-deal (criterion 12)",
        "| Entropy, play |",
        "| Contract solo ena |",
        "| Declarer's side wins |",
        "| Card-location head, accuracy |",
        "Evaluation, `checkpoint-000002.pt`: 8 deals in four seatings against random legal play",
    ):
        assert wanted in text, wanted
    assert "Users" not in text and str(tmp_path) not in text  # no machine paths in what goes into the spec


def test_c12_decisions_and_tokens_per_seat_deal_mean_and_largest(engine_run):
    store = rollout.Rollout(zero_sum=False)
    # Seat-deal (0, 0): 3 decisions, 9 tokens at the last. Seat-deal (0, 1): 1 decision, 4 tokens.
    add_rows(store, [0, 0, 0, 0], [0, 0, 0, 1], [0, 0, 0, 0], n_tokens=[2, 5, 9, 4])
    zeros = np.zeros((1, 4), dtype=np.float32)
    store.finish([0], [0], zeros, np.zeros((1, 4, C.N_SCORE_PARTS), dtype=np.float32))
    deals = store.take()
    ppo.prepare(deals, cfg_with()["ppo"])
    stats = ppo.rollout_stats(deals, 0)
    assert stats["rollout/decisions_per_seat_deal_mean"] == 2.0
    assert stats["rollout/decisions_per_seat_deal_max"] == 3.0
    assert stats["rollout/tokens_per_seat_deal_mean"] == 6.5
    assert stats["rollout/tokens_per_seat_deal_max"] == 9.0
    _, history = engine_run
    for stats in history:
        assert 1 < stats["rollout/tokens_per_seat_deal_mean"] <= stats["rollout/tokens_per_seat_deal_max"] <= C.MAX_TOKENS
        # A seat decides more often since spec 009: the round of announcements has up to 29
        # things said and 91 turns. At most 4 calls in the auction, 6 decisions of the declarer
        # before the round, 29 + 23 in the round and 12 cards: 74. It was 30 in flow A.
        assert 10 < stats["rollout/decisions_per_seat_deal_mean"] <= stats["rollout/decisions_per_seat_deal_max"] <= 74


def test_c13_the_names_of_the_bids_are_those_of_the_contract():
    rows = table_with(tables_of(ENCODING_MD.read_text(encoding="utf-8")), "Id", "Count", "Name")
    bid = next(row for row in rows if row[2] == "`BID_x`")
    names = [name.strip().replace("*", "").replace("č", "c") for name in bid[3].split(":")[1].split(",")]
    names[0] = names[0].split(" ")[0]  # "naprej (pass)"
    assert list(C.BIDS) == names and len(C.BIDS) == C.STATE_BLOCKS["B2"][1] == 13
    assert len(C.ROLES) == C.STATE_BLOCKS["B4"][1]


def test_c13_deal_monitors_of_a_hand_built_batch():
    store = rollout.Rollout(zero_sum=True)
    tri, dve, solo_ena = C.BIDS.index("tri"), C.BIDS.index("dve"), C.BIDS.index("solo ena")
    declarer, partner, opponent = (C.ROLES.index(name) for name in ("declarer", "called partner", "opponent"))
    bos, bid = C.EVENT_GROUPS["BOS"][0], C.EVENT_GROUPS["BID"][0]
    game = C.SCORE_PARTS.index("game")
    valat = C.SCORE_PARTS.index("valat")

    def deal(table, seats, contract, opening, part, value):
        """One finished deal: (seat, role) pairs stored, its contract, its first three calls, the declarer's part."""
        n = len(seats)
        tokens = np.zeros((n, C.MAX_TOKENS, 5), dtype=np.int16)
        tokens[:, 0, 0] = bos
        tokens[:, 1:4, 0] = [bid + call for call in opening]
        state = np.stack([state_with(B1=C.PHASES.index("play"), B2=contract, B4=role) for _, role in seats])
        store.add(
            tokens=tokens,
            tables=np.full(n, table),
            seat=np.array([seat for seat, _ in seats]),
            deal=np.zeros(n, dtype=np.int64),
            n_tokens=np.full(n, 20),
            state=state,
            mask=np.ones((n, C.N_ACTIONS), dtype=bool),
            critic_extra=np.zeros((n, C.CRITIC_EXTRA), dtype=np.float32),
            partner_target=np.zeros(n, dtype=np.int64),
            card_target=np.zeros((n, C.N_CARDS), dtype=np.int64),
            action=np.zeros(n, dtype=np.int64),
            logp=np.zeros(n, dtype=np.float32),
            value=np.zeros((n, C.N_SCORE_PARTS), dtype=np.float32),
        )
        # The declarer's side gets +value in `part`, the others what makes the deal add up to zero.
        # A seat that is not stored is the declarer (table 1).
        roles = dict(seats)
        side = [seat for seat in range(4) if roles.get(seat, declarer) != opponent]
        parts = np.zeros((1, 4, C.N_SCORE_PARTS), dtype=np.float32)
        for seat in range(4):
            parts[0, seat, part] = value if seat in side else -value * len(side) / (4 - len(side))
        store.finish([table], [0], parts.sum(axis=-1), parts)

    # Table 0: forehand's forced tri (three opening passes), declarer seat 1 with partner seat 3, won.
    deal(0, [(0, opponent), (1, declarer), (2, opponent), (3, partner)], tri, [0, 0, 0], game, 0.5)
    # Table 1: dve, lost; the declarer (seat 0) is a past checkpoint's seat and is not stored.
    deal(1, [(1, opponent), (2, partner), (3, opponent)], dve, [0, 3, 0], game, -0.25)
    # Table 2: solo ena won by valat: the game part is zero, the valat part pays.
    deal(2, [(0, opponent), (1, opponent), (2, declarer), (3, opponent)], solo_ena, [7, 0, 0], valat, 5.0)
    # Table 3: solo ena, lost.
    deal(3, [(0, declarer), (1, opponent), (2, opponent), (3, opponent)], solo_ena, [0, 0, 7], game, -1.0)
    deals = store.take()
    stats = ppo.deal_stats(deals)
    assert stats["deals/count"] == 4.0
    assert stats["deals/contract_tri"] == 0.25
    assert stats["deals/contract_dve"] == 0.25
    assert stats["deals/contract_solo_ena"] == 0.5
    assert sum(value for name, value in stats.items() if name.startswith("deals/contract_")) == pytest.approx(1.0)
    assert set(name for name in stats if name.startswith("deals/contract_")) == {
        "deals/contract_" + name.replace(" ", "_") for name in ("tri", "dve", "ena", "solo tri", "solo dve", "solo ena")
    }
    assert stats["deals/forehand_forced"] == 0.25
    assert stats["deals/forehand_forced_tri"] == 0.25
    assert stats["deals/declarer_wins"] == 0.5


def test_c13_the_monitors_of_d107_are_logged_on_real_deals(engine_run, tmp_path):
    _, history = engine_run
    for stats in history:
        assert {"entropy/bidding", "entropy/play", "entropy/talon pick", "entropy/discard"} <= set(stats)
        shares = [value for name, value in stats.items() if name.startswith("deals/contract_")]
        assert sum(shares) == pytest.approx(1.0)
        assert 0 <= stats["deals/forehand_forced_tri"] <= stats["deals/forehand_forced"] <= 1
        assert 0 <= stats["deals/declarer_wins"] <= 1
        assert stats["deals/count"] >= 8
    # The mock's state features are noise: it has no deal monitors.
    mock = train.Trainer(cfg_with(**{"env.tables": 8, "ppo.seat_deals_per_update": 24, "ppo.update_batch": 16}), CPU)
    assert not any(name.startswith("deals/") for name in mock.run(1)[0])
    # And they reach TensorBoard.
    engine()
    trainer = train.Trainer(ci_cfg(), CPU, log_dir=tmp_path)
    trainer.run(1)
    trainer.close()
    tags = scalar_tags(tmp_path)
    for tag in (
        "entropy/bidding",
        "entropy/play",
        "deals/declarer_wins",
        "deals/forehand_forced_tri",
        "rollout/decisions_per_seat_deal_mean",
        "rollout/tokens_per_seat_deal_max",
        "speed/steps_per_second",
        "speed/trained_per_second",
        "time/environment_share",
        "time/acting_share",
        "time/bookkeeping_share",
        "time/update_share",
        "aux/partner_accuracy",
        "aux/partner_guess",
        "aux/card_accuracy",
        "aux/card_guess",
        "entropy_weight/play",
        "entropy_choice/play",
        "forced_share/play",
        "rollout/played",
        "rollout/played_own_seats",
        "rollout/trained_share",
        "rollout/trained_share_own_seats",
        "env/illegal_actions",
    ):
        assert tag in tags, tag
    assert any(tag.startswith("deals/contract_") for tag in tags)


def test_c14_the_guessing_level_of_hand_built_states():
    kings = list(C.KINGS)
    play, bidding = C.PHASES.index("play"), C.PHASES.index("bidding")
    declarer, partner, opponent = (C.ROLES.index(name) for name in ("declarer", "called partner", "opponent"))
    f2 = C.STATE_BLOCKS["F2"][0]
    suit_card, tarok = 22, 3
    assert suit_card not in kings and tarok < C.N_TAROKS

    # During the bidding every unseen card may lie with any of the three others or in the talon.
    state = torch.from_numpy(state_with(B1=bidding))[None]
    places = ppo.possible_places(state)
    assert places.shape == (1, C.N_CARDS, C.CARD_LOCATION_CLASSES) and places.all()

    # An opponent in play: a low suit card may lie in the declarer's hidden discards, a tarok or a king may not.
    row = state_with(B1=play, B4=opponent)
    row[f2 + 0 * C.N_CARDS + suit_card] = 1.0  # the next seat cannot hold the suit card
    row[f2 + 2 * C.N_CARDS + tarok] = 1.0  # the previous seat cannot hold the tarok
    places = ppo.possible_places(torch.from_numpy(row)[None])[0]
    assert places[suit_card].tolist() == [True, False, True, True]
    assert places[tarok].tolist() == [False, True, True, False]
    assert places[kings[0]].tolist() == [False, True, True, True]
    # The declarer has seen its own discards: nothing unseen is out of play.
    places = ppo.possible_places(torch.from_numpy(state_with(B1=play, B4=declarer))[None])[0]
    assert places[suit_card].tolist() == [False, True, True, True]

    # Partner seat. Relative seats 1 to 3; entry 0 is "none" and never possible.
    def partners(**blocks):
        return ppo.possible_partners(torch.from_numpy(state_with(**blocks))[None])[0].tolist()

    assert partners(B1=play, B4=declarer, B3=0) == [False, True, True, True]
    assert partners(B1=play, B4=opponent, B3=2) == [False, True, False, True]  # not the declarer
    assert partners(B1=play, B4=partner, B3=3) == [False, False, False, True]  # the declarer
    # B6 is the public seat of the declarer's partner. An opponent then has the one seat left.
    assert partners(B1=play, B4=opponent, B3=2, B6=1) == [False, False, False, True]
    assert partners(B1=play, B4=declarer, B3=0, B6=3) == [False, False, False, True]
    assert partners(B1=bidding) == [False, True, True, True]  # no teams yet (the target is "none" there)


def test_c14_on_real_deals_the_true_place_is_always_among_the_possible_ones():
    engine_env = engine()
    env = engine_env.EngineEnv(64, seed=14, threads=2)
    rng = np.random.default_rng(14)
    f4, width = C.UNSEEN_BLOCK
    cards = partners = fewer_places = fewer_partners = 0
    for _ in range(400):
        obs = env.observe()
        state = torch.from_numpy(obs["state"])
        unseen = obs["state"][:, f4 : f4 + width] > 0.5
        places = ppo.possible_places(state).numpy()
        target = obs["card_target"]
        truth = np.take_along_axis(places, target[:, :, None], axis=2)[:, :, 0]
        assert truth[unseen].all()
        cards += int(unseen.sum())
        fewer_places += int((places.sum(axis=2) < 4)[unseen].sum())
        has = obs["partner_target"] != C.PARTNER_NONE
        possible = ppo.possible_partners(state).numpy()
        assert possible[np.flatnonzero(has), obs["partner_target"][has]].all()
        assert not possible[:, 0].any()
        partners += int(has.sum())
        fewer_partners += int((possible.sum(axis=1) < 3)[has].sum())
        # Random legal play bids itself up to a solo almost always. Passing most of the time
        # gives contracts with a called king, and so decisions with a partner.
        action = engine_env._random_legal(obs["mask"], rng)
        may_pass = obs["mask"][:, 0] & (rng.random(env.n) < 0.85)
        env.step(np.where(may_pass, 0, action))
    # The test saw enough cases, and the rules do narrow the places down.
    assert cards > 100_000 and fewer_places > cards // 4
    assert partners > 300 and fewer_partners > partners // 4


def test_c14_the_accuracy_of_both_heads_is_logged_next_to_the_guessing_level(engine_run):
    _, history = engine_run
    for stats in history:
        for name in ("aux/partner_accuracy", "aux/partner_guess", "aux/card_accuracy", "aux/card_guess"):
            assert 0 <= stats[name] <= 1, name
        assert 0.2 < stats["aux/card_guess"] < 0.6  # between one of four places and one of two
    # Hand-computed: two cards counted; the head is right on one; one has two possible places, one four.
    hit = torch.tensor([[True, False, True]])
    counted = torch.tensor([[True, True, False]])
    n_places = torch.tensor([[2.0, 4.0, 1.0]])
    sums = ppo.accuracy_and_guess(hit, counted, n_places)
    assert sums.tolist() == [1.0, 0.75, 2.0]  # hits, expected hits of a uniform guess (1/2 + 1/4), cards


# --------------------------------------------------------------------------
# Criterion 15: game records
# --------------------------------------------------------------------------


def test_c15_exported_records_replay_to_the_same_entries():
    engine()
    import tarok_env

    from tarok_train import evaluate

    result = evaluate.evaluate(
        lambda n: evaluate.first_legal, deals=12, tables=12, seed=15, threads=2, others=evaluate.RandomLegal(15), records=5
    )
    assert len(result["records"]) == 5
    for record in result["records"]:
        text = record["json"]
        assert json.loads(text)["seed"] == str(tarok_env.deal_seed(15, record["table"], 0, 0))
        assert tarok_env.replay_record(text) == record["entries"]
        assert any(record["entries"])
    assert result["records_replayed_equal"] == 5
    # A record whose actions were changed does not pass as the same deal.
    data = json.loads(result["records"][0]["json"])
    assert evaluate.replays_equal(result["records"][0]["json"], result["records"][0]["entries"])
    wrong = [entry + 1 for entry in result["records"][0]["entries"]]
    assert not evaluate.replays_equal(result["records"][0]["json"], wrong)
    assert isinstance(data, dict)


# --------------------------------------------------------------------------
# Criteria 16 and 17: the entropy controller (D127)
# --------------------------------------------------------------------------

CONTROL = {"enabled": True, "gain": 0.2, "proportional": 1.5, "min_weight": 0.01, "max_weight": 0.5, "band": 0.5}


def controller(start=0.01, **overrides):
    cfg = {**CONTROL, "targets": {name: 0.5 for name in ACTING_PHASES}, **overrides}
    return ppo.EntropyController(cfg, start)


def test_c17_the_update_rule_on_hand_computed_numbers():
    # error    = (target - entropy) / target, cut to [-1, 1]
    # integral <- integral * exp(gain * error),         kept inside [min_weight, max_weight]
    # weight    = integral * exp(proportional * error), kept inside [min_weight, max_weight]
    step = ppo.EntropyController.next_weights  # (integral, entropy, target, gain, proportional, min_weight, max_weight)

    # Entropy 0.25 under a target of 0.5: error 0.5. Integral 0.02 * e^0.1, weight that * e^0.75.
    integral, weight = step(0.02, 0.25, 0.5, 0.2, 1.5, 0.01, 0.5)
    assert integral == pytest.approx(0.02 * math.exp(0.1)) and integral == pytest.approx(0.0221034, abs=1e-7)
    assert weight == pytest.approx(0.02 * math.exp(0.85)) and weight == pytest.approx(0.0467929, abs=1e-7)
    # Entropy 0.75 above the target: error -0.5. Integral 0.2 * e^-0.1, weight that * e^-0.75.
    integral, weight = step(0.2, 0.75, 0.5, 0.2, 1.5, 0.01, 0.5)
    assert integral == pytest.approx(0.1809675, abs=1e-7)
    assert weight == pytest.approx(0.2 * math.exp(-0.85)) and weight == pytest.approx(0.0854830, abs=1e-7)
    # On target nothing moves.
    assert step(0.05, 0.5, 0.5, 0.2, 1.5, 0.01, 0.5) == (0.05, 0.05)
    # The error is cut to -1: an entropy of 3 counts like one of 1 (twice the target).
    assert step(0.2, 3.0, 0.5, 0.2, 1.5, 0.01, 0.5) == step(0.2, 1.0, 0.5, 0.2, 1.5, 0.01, 0.5)
    integral, weight = step(0.2, 3.0, 0.5, 0.2, 1.5, 0.01, 0.5)
    assert integral == pytest.approx(0.2 * math.exp(-0.2)) and weight == pytest.approx(0.2 * math.exp(-1.7))
    # Limits. Entropy 0: error 1; 0.45 * e^0.2 = 0.5496 is cut to max_weight, and so is the weight.
    assert step(0.45, 0.0, 0.5, 0.2, 1.5, 0.01, 0.5) == (0.5, 0.5)
    # 0.011 * e^-0.2 = 0.009006 is cut to min_weight, and so is the weight.
    assert step(0.011, 1.0, 0.5, 0.2, 1.5, 0.01, 0.5) == (0.01, 0.01)
    # Without a proportional part the weight is the integral.
    integral, weight = step(0.02, 0.25, 0.5, 0.2, 0.0, 0.01, 0.5)
    assert integral == weight and weight == pytest.approx(0.0221034, abs=1e-7)

    control = controller(start=0.02)
    play, bidding = C.PHASES.index("play"), C.PHASES.index("bidding")
    assert control.weights.tolist() == control.integral.tolist() == [0.02] * len(C.PHASES)
    control.step({"play": 0.25, "bidding": 0.75})
    assert control.integral[play] == pytest.approx(0.0221034, abs=1e-7)
    assert control.weights[play] == pytest.approx(0.0467929, abs=1e-7)
    assert control.integral[bidding] == pytest.approx(0.02 * math.exp(-0.1))
    assert control.weights[bidding] == 0.01  # 0.02 * e^-0.85 = 0.00855 is cut to min_weight
    # A phase without decisions in this update keeps its weight.
    others = [i for i in range(len(C.PHASES)) if i not in (play, bidding)]
    assert control.weights[others].tolist() == control.integral[others].tolist() == [0.02] * len(others)
    assert control.in_band({"play": 0.25, "bidding": 0.76, "discard": 0.74}) == {"play": True, "bidding": False, "discard": True}
    # A second step goes on from the integral, not from the weight.
    control.step({"play": 0.5})
    assert control.integral[play] == control.weights[play] and control.weights[play] == pytest.approx(0.0221034, abs=1e-7)
    # The state round-trips (the checkpoint keeps it).
    other = controller()
    other.load_state(control.state())
    assert other.weights.tolist() == control.weights.tolist() and other.integral.tolist() == control.integral.tolist()


def test_c17_a_controller_that_is_off_never_changes_a_weight():
    control = controller(enabled=False)
    control.step({"play": 0.0, "bidding": 3.0})
    assert control.weights.tolist() == control.integral.tolist() == [0.01] * len(C.PHASES)
    assert control.tensor(CPU) is None  # the update then uses [ppo] entropy_weight as before


def test_c17_the_controller_changes_only_the_entropy_weight():
    cfg = cfg_with(**{"env.tables": 8, "ppo.seat_deals_per_update": 24, "ppo.update_batch": 16})
    trainer = train.Trainer(cfg, CPU)
    deals = collected(trainer)
    batch = deals.to_torch(CPU)
    with torch.no_grad():
        out, value = ppo.evaluate(trainer.net, trainer.critic, batch)
    plain = ppo.compute_losses(out, value, batch, cfg["ppo"])
    # Equal weights in every phase give the loss without a controller.
    same = ppo.compute_losses(out, value, batch, cfg["ppo"], torch.full((len(C.PHASES),), cfg["ppo"]["entropy_weight"]))
    assert float(same["total"]) == pytest.approx(float(plain["total"]), abs=1e-6)
    weights = torch.full((len(C.PHASES),), cfg["ppo"]["entropy_weight"])
    weights[C.PHASES.index("play")] = 0.3
    steered = ppo.compute_losses(out, value, batch, cfg["ppo"], weights)
    for name in ppo.TERMS:
        assert float(steered[name]) == float(plain[name]), name  # the unweighted terms are untouched
        if name != "entropy":
            assert float(steered["weighted"][name]) == float(plain["weighted"][name]), name
    # The entropy term is the sum over the phases of weight times the entropy of that phase's decisions.
    v = batch.valid
    entropy = ppo.policy_entropy(torch.log_softmax(out.logits[v], dim=-1), batch.mask[v])
    phase = batch.state[v][:, C.PHASE_BLOCK[0] : C.PHASE_BLOCK[0] + C.PHASE_BLOCK[1]].argmax(dim=1)
    expected = -(weights[phase] * entropy).mean()
    assert float(steered["weighted"]["entropy"]) == pytest.approx(float(expected), abs=1e-6)
    assert float(steered["weighted"]["entropy"]) < float(plain["weighted"]["entropy"])
    # In a run the controller moves its weights and nothing else of the config.
    on = copy.deepcopy(cfg)
    on["entropy_control"]["enabled"] = True
    trainer = train.Trainer(on, CPU)
    before = copy.deepcopy(trainer.cfg)
    history = trainer.run(2)
    assert trainer.cfg == before
    # After two updates the entropy is still far above its target: the weights sit at their lower limit.
    limit = cfg["entropy_control"]["min_weight"]
    assert all(limit <= weight <= cfg["entropy_control"]["max_weight"] for weight in trainer.entropy.weights)
    assert history[-1]["entropy_weight/play"] == pytest.approx(trainer.entropy.weights[C.PHASES.index("play")])
    trainer.entropy.step({"play": 0.01})  # a collapsed phase raises its weight
    assert trainer.entropy.weights[C.PHASES.index("play")] > limit


def test_c17_the_config_names_a_target_for_every_phase_and_the_band():
    for path in (MOCK_TOML, CI_TOML, *SMOKE.values()):
        control = train.load_config(path)["entropy_control"]
        assert set(control) == {"enabled", "gain", "proportional", "min_weight", "max_weight", "band", "targets"}
        assert set(control["targets"]) == set(ACTING_PHASES)
        assert 0 < control["min_weight"] < control["max_weight"] and 0 < control["band"] < 1
    cfg = cfg_with()
    del cfg["entropy_control"]["targets"]["play"]
    with pytest.raises(train.ConfigError, match="play"):
        train.validate_config(cfg)
    cfg = cfg_with()
    cfg["entropy_control"]["targets"]["bidding"] = 0.0
    with pytest.raises(train.ConfigError, match="bidding"):
        train.validate_config(cfg)


@pytest.mark.slow
def test_c16_with_the_controller_the_entropy_stays_in_its_band_and_without_it_falls_under():
    """200 updates of the transformer on the mock's token task, once with the controller, once without.

    An untrained policy is near uniform (entropy ln 4 = 1.39), far above the band, and it takes
    the controller some updates to find the weight each phase needs. So "stays within the band
    over 200 updates" is read as: from update 61 to update 200 the entropy of every phase is
    inside its band at every update. Without the controller the same run ends under the band.
    """
    device = torch.device("cuda" if torch.cuda.is_available() else "cpu")
    sizes = {
        "run.network": "transformer",
        "env.mode": "tokens",
        "env.tables": 128,
        "ppo.seat_deals_per_update": 512,
        "ppo.update_batch": 64,
        "run.checkpoint_every": 1000,
    }
    updates, settle = 200, 60
    histories = {}
    for enabled in (True, False):
        cfg = cfg_with(**sizes)
        cfg["entropy_control"]["enabled"] = enabled
        trainer = train.Trainer(cfg, device)
        histories[enabled] = trainer.run(updates)
    control = cfg_with()["entropy_control"]
    phases = [name for name in ACTING_PHASES if any(f"entropy/{name}" in stats for stats in histories[True])]
    assert set(phases) == {"bidding", "king call", "talon pick", "discard", "announcements", "play"}
    # The numbers first, so they are printed also when an assertion below fails (pytest -s shows them).
    best, chance = 1.0, 0.25
    scores = {enabled: float(np.mean([stats["task/score"] for stats in histories[enabled][-20:]])) for enabled in histories}
    series = {}
    print(f"band: within {control['band']:.0%} of the target; entropy from update {settle + 1} to {updates} (without: the last 50 updates)")
    for name in phases:
        on = [stats[f"entropy/{name}"] for stats in histories[True][settle:] if f"entropy/{name}" in stats]
        off = [stats[f"entropy/{name}"] for stats in histories[False][-50:] if f"entropy/{name}" in stats]
        series[name] = (on, off)
        print(
            f"{name}: with {min(on):.3f} to {max(on):.3f} (mean {np.mean(on):.3f}), "
            f"without mean {np.mean(off):.3f}, at most {max(off):.3f}"
        )
    print(f"task score, mean of the last 20 updates: with the controller {scores[True]:.3f}, without {scores[False]:.3f}")
    for name in phases:
        target = control["targets"][name]
        low, high = target * (1 - control["band"]), target * (1 + control["band"])
        on, off = series[name]
        assert len(on) >= 100 and len(off) >= 30
        outside = [round(value, 3) for value in on if not low <= value <= high]
        assert not outside, (name, (low, high), outside)
        assert abs(np.mean(on) - target) < 0.1 * target, (name, np.mean(on))  # on average it sits on the target
        # Without the controller: the collapse of spec 007. Single updates spike, so the test takes
        # the mean of the last 50 updates and asks that nine of ten lie under the band.
        assert np.mean(off) < low / 2, (name, low, np.mean(off))
        assert np.mean([value < low for value in off]) >= 0.9, (name, low, sorted(off)[-6:])
    # The learner still learns the task with the controller on.
    assert scores[True] > chance + 0.5 * (best - chance)


# --------------------------------------------------------------------------
# Criterion 19: the configs name the origin of every setting; the run names its commit
# --------------------------------------------------------------------------


def test_c19_every_setting_of_the_three_configs_has_its_origin_in_the_file():
    for path in (CI_TOML, *SMOKE.values()):
        cfg = train.load_config(path)
        settings = 0
        for line in path.read_text(encoding="utf-8").splitlines():
            if re.match(r'^\s*("[^"]+"|[A-Za-z_0-9]+)\s*=', line):
                settings += 1
                assert re.search(r"#\s*\S", line), f"{path.name}: no origin on the line '{line}'"
        expected = sum(len(table) for table in cfg.values()) - 1 + len(cfg["entropy_control"]["targets"])
        assert settings == expected, path.name


def test_c19_the_run_folder_names_the_commit_the_state_of_the_tree_and_the_config(tmp_path):
    engine()
    assert train.main(["--config", str(CI_TOML), "--out-dir", str(tmp_path), "--updates", "1"]) == 0
    info = json.loads((tmp_path / "smoke-a-ci-mlp" / "run-info.json").read_text(encoding="utf-8"))
    assert re.fullmatch(r"[0-9a-f]{40}", info["commit"])
    assert isinstance(info["git_status_short"], str)
    assert info["config_file"] == "smoke-a-ci.toml"
    assert info["config"]["run"]["updates"] == 1  # what really ran, with the command line's changes
    assert re.fullmatch(r"[0-9a-f]{16}", info["engine_sources"])
    assert info["started"].startswith("20")
    summary = json.loads((tmp_path / "smoke-a-ci-mlp" / "summary.json").read_text(encoding="utf-8"))
    assert summary["seconds"] > 0 and summary["failed"] is None


# --------------------------------------------------------------------------
# D130 (B12-15): a module built from other sources is refused
# --------------------------------------------------------------------------


def test_d130_a_module_built_from_other_engine_sources_is_refused(tmp_path, monkeypatch):
    engine_env = engine()
    import tarok_env

    # The installed module is current: its hash is the hash of the files on disk.
    assert re.fullmatch(r"[0-9a-f]{16}", tarok_env.SOURCE_HASH)
    manifest = engine_env.source_manifest(engine_env.SOURCE_ROOT)
    assert manifest == tarok_env.SOURCE_MANIFEST
    assert engine_env.manifest_hash(manifest) == tarok_env.SOURCE_HASH
    assert "engine/src/score.rs" in manifest and "train/tarok_env/src/lib.rs" in manifest
    assert "engine/tests" not in manifest and "target" not in manifest
    engine_env.EngineEnv(2, seed=1, threads=1)

    # A copy of the sources with one rule file changed by one byte.
    copy_root = tmp_path / "src"
    for line in manifest.splitlines():
        relative = line.split("\t")[0]
        (copy_root / relative).parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(engine_env.SOURCE_ROOT / relative, copy_root / relative)
    monkeypatch.setattr(engine_env, "SOURCE_ROOT", copy_root)
    engine_env.EngineEnv(2, seed=1, threads=1)  # an exact copy passes
    changed = copy_root / "engine" / "src" / "score.rs"
    changed.write_bytes(changed.read_bytes() + b"\n// a change after the build\n")
    with pytest.raises(engine_env.StaleModuleError, match=r"engine/src/score\.rs") as error:
        engine_env.EngineEnv(2, seed=1, threads=1)
    assert "maturin develop --release" in str(error.value)
    # A new file and a missing file are found too.
    changed.unlink()
    (copy_root / "engine" / "src" / "new_rule.rs").write_text("// new\n", encoding="utf-8")
    with pytest.raises(engine_env.StaleModuleError) as error:
        engine_env.EngineEnv(2, seed=1, threads=1)
    assert "engine/src/score.rs" in str(error.value) and "engine/src/new_rule.rs" in str(error.value)
    # Where the sources are not on disk (an installed wheel alone) there is nothing to compare.
    monkeypatch.setattr(engine_env, "SOURCE_ROOT", tmp_path / "nowhere")
    engine_env.EngineEnv(2, seed=1, threads=1)


# --------------------------------------------------------------------------
# Tests added after the review of this spec (2026-11-12). Each one is aimed at a
# planted bug that the tests above let through; Results lists them.
# --------------------------------------------------------------------------


class EnvSpy:
    """Records what really happens at the environment of a trainer, step by step.

    actions[(table, deal, seat)]: the slots that seat took in that deal, in order.
    frozen[(table, deal, seat)]: the seat was played by the past checkpoint.
    paid[(table, deal)]: (reward (4,), parts (4, 10)) as the environment returned them.
    steps, frozen_rows: counts since the spy was attached.
    """

    def __init__(self, trainer):
        self.actions, self.frozen, self.paid = {}, {}, {}
        self.steps = self.frozen_rows = 0
        env, n = trainer.env, trainer.n
        real = env.step
        rows = np.arange(n)

        def step(action):
            obs = env.observe()
            seat, deal = obs["seat"].copy(), obs["deal"].copy()
            frozen = trainer.frozen[rows, seat].copy()
            done, reward, parts = real(action)
            self.steps += 1
            self.frozen_rows += int(frozen.sum())
            for table in range(n):
                key = (table, int(deal[table]), int(seat[table]))
                self.actions.setdefault(key, []).append(int(action[table]))
                self.frozen[key] = bool(frozen[table])
                if done[table]:
                    self.paid[(table, int(deal[table]))] = (reward[table].copy(), parts[table].copy())
            return done, reward, parts

        env.step = step


def busy_cfg(**overrides):
    """The small engine config with a past checkpoint in most deals from the second update on."""
    return ci_cfg(**{"opponents.snapshot_every": 1, "opponents.past_share": 0.9, **overrides})


def test_c04_c08_in_a_normal_run_every_stored_seat_has_its_own_decisions_and_the_reward_the_engine_paid_it():
    engine()
    trainer = train.Trainer(busy_cfg(), CPU)
    spy = EnvSpy(trainer)
    trainer.run(2)
    deals = collected(trainer)  # the third collection: past checkpoints play, and deals straddled two updates
    assert len(deals) >= 16
    stored = set()
    for i in range(len(deals)):
        key = (int(deals.table[i]), int(deals.deal[i]), int(deals.seat[i]))
        stored.add(key)
        n = int(deals.valid[i].sum())
        # The whole deal of that seat, from its first decision, and no other seat's decisions.
        assert deals.action[i, :n].tolist() == spy.actions[key], key
        assert not spy.frozen[key], key  # a seat of the past checkpoint is never stored
        reward, parts = spy.paid[key[:2]]
        assert deals.reward[i] == reward[key[2]], key  # the reward of this seat, not of a neighbour
        assert np.array_equal(deals.parts[i], parts[key[2]]), key
        assert np.array_equal(deals.true_parts[i], parts[key[2]]), key
    # The collection did hold seats of the past checkpoint, and none of them is in the storage.
    in_these_deals = [key for key in spy.frozen if key[:2] in {k[:2] for k in stored}]
    assert sum(spy.frozen[key] for key in in_these_deals) >= 4
    assert all(not spy.frozen[key] for key in stored)
    assert stored == {key for key in in_these_deals if not spy.frozen[key]}


def test_c02_one_illegal_row_among_legal_ones_is_refused():
    n = 3
    mask = np.zeros((n, C.N_ACTIONS), dtype=bool)
    mask[:, 5] = True
    rows = dict(
        tokens=np.zeros((n, C.MAX_TOKENS, 5), dtype=np.int16),
        tables=np.arange(n),
        seat=np.zeros(n, dtype=np.int64),
        deal=np.zeros(n, dtype=np.int64),
        n_tokens=np.ones(n, dtype=np.int64),
        state=np.zeros((n, C.STATE_FEATURES), dtype=np.float32),
        mask=mask,
        critic_extra=np.zeros((n, C.CRITIC_EXTRA), dtype=np.float32),
        partner_target=np.zeros(n, dtype=np.int64),
        card_target=np.zeros((n, C.N_CARDS), dtype=np.int64),
        logp=np.zeros(n, dtype=np.float32),
        value=np.zeros((n, C.N_SCORE_PARTS), dtype=np.float32),
    )
    store = rollout.Rollout(zero_sum=False)
    with pytest.raises(ValueError, match="1 stored decisions took an illegal action"):
        store.add(action=np.array([5, 6, 5]), **rows)  # the middle row only
    assert store.pending_rows == 0
    store.add(action=np.array([5, 5, 5]), **rows)
    assert store.pending_rows == 3


def test_c05_a_deal_that_began_under_older_weights_shows_an_age_above_zero():
    engine()
    trainer = train.Trainer(ci_cfg(), CPU)
    trainer.run(1)
    assert trainer.fresh.sum() < trainer.n  # most tables stand inside a deal that began before the update
    # What a broken loop would do: store the rest of those deals as if they were fresh.
    trainer.fresh[:] = True
    deals = collected(trainer)
    stats = ppo.rollout_stats(deals, trainer.updates)
    assert stats["rollout/decision_age_max"] == 1.0
    assert 0 < stats["rollout/decision_age_mean"] <= 1
    # The sound loop, same point: nothing is older than the current weights.
    trainer = train.Trainer(ci_cfg(), CPU)
    trainer.run(1)
    stats = ppo.rollout_stats(collected(trainer), trainer.updates)
    assert stats["rollout/decision_age_max"] == 0.0


def test_c06_the_weights_of_the_entropy_controller_come_back_from_a_checkpoint(tmp_path):
    cfg = cfg_with(**{"run.network": "mlp", "env.tables": 8, "ppo.seat_deals_per_update": 24, "ppo.update_batch": 16})
    cfg["entropy_control"]["enabled"] = True
    cfg["entropy_control"]["targets"] = {name: 5.0 for name in ACTING_PHASES}  # above any entropy: the weights rise
    trainer = train.Trainer(cfg, CPU)
    trainer.run(2)
    start = cfg["ppo"]["entropy_weight"]
    play = C.PHASES.index("play")
    assert trainer.entropy.weights[play] > start and trainer.entropy.integral[play] > start
    assert trainer.entropy.weights[play] != trainer.entropy.integral[play]
    trainer.save(tmp_path / "c.pt")
    loaded = train.Trainer.load(tmp_path / "c.pt", CPU)
    assert loaded.entropy.weights.tolist() == trainer.entropy.weights.tolist()
    assert loaded.entropy.integral.tolist() == trainer.entropy.integral.tolist()
    assert loaded.deal_version.tolist() == trainer.deal_version.tolist()


def test_c17_the_controller_is_connected_to_the_trainer(monkeypatch):
    """Fast, for CI: the weights the controller holds are the weights the next update's loss gets."""
    cfg = cfg_with(**{"run.network": "mlp", "env.tables": 8, "ppo.seat_deals_per_update": 24, "ppo.update_batch": 16})
    cfg["entropy_control"]["enabled"] = True
    cfg["entropy_control"]["targets"] = {name: 5.0 for name in ACTING_PHASES}  # above any entropy: the weights rise
    trainer = train.Trainer(cfg, CPU)
    start = cfg["ppo"]["entropy_weight"]
    play, bidding = C.PHASES.index("play"), C.PHASES.index("bidding")
    given, held = [], []
    real = ppo.update

    def update(*args, **kwargs):
        weights = kwargs.get("entropy_weights", args[7] if len(args) > 7 else None)
        given.append(None if weights is None else weights.tolist())
        held.append(trainer.entropy.weights.astype(np.float32).tolist())
        return real(*args, **kwargs)

    monkeypatch.setattr(ppo, "update", update)
    history = trainer.run(3)
    assert given[0] == pytest.approx([start] * len(C.PHASES))
    for index in (1, 2):
        assert given[index] == held[index]  # what the loss got is what the controller held
        assert given[index][play] > given[index - 1][play] and given[index][bidding] > given[index - 1][bidding]
    # The rule, from the logged entropy of the first update: error (5 - entropy) / 5.
    control = cfg["entropy_control"]
    integral, weight = ppo.EntropyController.next_weights(
        start, history[0]["entropy/play"], 5.0, control["gain"], control["proportional"], control["min_weight"], control["max_weight"]
    )
    assert given[1][play] == pytest.approx(weight, rel=1e-6) and weight > integral > start
    assert history[0]["entropy_weight/play"] == pytest.approx(weight)
    # The entropy term of the loss is what those weights make of the entropy.
    assert history[1]["loss/entropy"] < -start * history[1]["entropy/mean"]
    # With the controller off the loss gets no weights at all.
    cfg["entropy_control"]["enabled"] = False
    given.clear()
    train.Trainer(cfg, CPU).run(1)
    assert given == [None]


def test_c07_the_interval_is_taken_over_the_deals_not_over_the_single_seatings():
    engine()
    from tarok_train import evaluate

    result = evaluate.evaluate(
        lambda n: evaluate.first_legal, deals=40, tables=20, seed=31, threads=2, others=evaluate.RandomLegal(31)
    )
    rewards = np.array(result["rewards"])
    values = rewards.mean(axis=1)
    # The room grows with the largest reward: with kontra a reward can be above 100 (D147, 1 a).
    room = 1e-6 * max(1.0, float(np.abs(rewards).max()))
    assert rewards.shape == (40, 4) and result["values"] == pytest.approx(values.tolist(), abs=room)
    mean, low, high = evaluate.interval(values)
    assert (result["mean"], result["low"], result["high"]) == pytest.approx((mean, low, high), abs=room)
    # Hand formula: 1.96 standard errors of the 40 deal values.
    half = 1.96 * values.std(ddof=1) / math.sqrt(40)
    assert result["high"] - result["mean"] == pytest.approx(half, abs=1e-6)
    # Not the interval over the 160 seatings, which is another number.
    _, wrong_low, wrong_high = evaluate.interval(rewards.reshape(-1))
    assert abs((wrong_high - wrong_low) - (high - low)) > 1e-3
    assert result["beats_random"] == (low > 0)


def test_c07_the_trained_evaluation_plays_the_checkpoints_weights_and_untrained_the_first_weights(tmp_path):
    engine()
    from tarok_train import evaluate

    assert train.main(["--config", str(CI_TOML), "--out-dir", str(tmp_path), "--updates", "2"]) == 0
    checkpoint = tmp_path / "smoke-a-ci-mlp" / "checkpoint-000002.pt"
    data = torch.load(checkpoint, map_location="cpu", weights_only=True)
    cfg = data["config"]

    def expected(load_weights):
        torch.manual_seed(cfg["run"]["seed"])
        net = train.build_network(cfg["model"], "mlp")
        if load_weights:
            net.load_state_dict(data["model"])
        agent = evaluate.network_agent(net.eval(), cfg["model"], CPU, seed=5)
        return evaluate.evaluate(agent, deals=16, tables=16, seed=5, threads=2, others=evaluate.RandomLegal(6))["values"]

    args = ["--checkpoint", str(checkpoint), "--deals", "16", "--tables", "16", "--seed", "5", "--threads", "2", "--device", "cpu"]
    assert evaluate.main(args) == 0
    assert evaluate.main([*args, "--untrained"]) == 0
    trained = json.loads((checkpoint.parent / "evaluation-000002.json").read_text(encoding="utf-8"))
    untrained = json.loads((checkpoint.parent / "evaluation-untrained.json").read_text(encoding="utf-8"))
    assert trained["values"] == expected(load_weights=True)
    assert untrained["values"] == expected(load_weights=False)
    assert trained["values"] != untrained["values"]
    assert (trained["updates"], untrained["updates"]) == (2, 0)


def test_c07_random_legal_play_is_uniform_over_the_legal_slots():
    from_engine = engine()
    from tarok_train import evaluate

    mask = np.zeros((2, C.N_ACTIONS), dtype=bool)
    mask[0, [3, 40, 41, 159]] = True
    mask[1, [7]] = True
    obs = {"mask": mask}
    player = evaluate.RandomLegal(4)
    rows = np.array([0, 1])
    counts = {}
    for _ in range(4000):
        first, second = player(rows, obs)
        counts[int(first)] = counts.get(int(first), 0) + 1
        assert second == 7
    assert set(counts) == {3, 40, 41, 159}
    assert all(850 < count < 1150 for count in counts.values()), counts  # 1,000 each, six standard deviations wide
    # Only the rows asked for are answered.
    assert player(np.array([1]), obs).tolist() == [7]
    # Another seed plays otherwise; the same seed the same.
    a, b, c = (evaluate.RandomLegal(seed) for seed in (1, 1, 2))
    many = np.zeros(200, dtype=np.int64)
    assert a(many, obs).tolist() == b(many, obs).tolist() != c(many, obs).tolist()
    assert from_engine is not None


def test_c07_no_chunk_of_the_evaluation_may_use_the_seed_of_the_training_run(tmp_path, capsys):
    engine()
    from tarok_train import evaluate

    assert evaluate.chunk_seeds(deals=2000, tables=500, seed=5) == [5, 6, 7, 8]
    assert evaluate.chunk_seeds(deals=24, tables=12, seed=7) == [7, 8]
    assert evaluate.chunk_seeds(deals=25, tables=12, seed=7) == [7, 8, 9]
    with pytest.raises(ValueError, match="seed of the training run"):
        evaluate.evaluate(
            lambda n: evaluate.first_legal, deals=24, tables=12, seed=7, threads=1, others=evaluate.first_legal, training_seed=8
        )
    assert train.main(["--config", str(CI_TOML), "--out-dir", str(tmp_path), "--updates", "1"]) == 0  # its seed is 8
    checkpoint = tmp_path / "smoke-a-ci-mlp" / "checkpoint-000001.pt"
    assert train.load_config(CI_TOML)["run"]["seed"] == 8
    base = ["--checkpoint", str(checkpoint), "--deals", "24", "--tables", "12", "--threads", "1", "--device", "cpu"]
    capsys.readouterr()
    assert evaluate.main([*base, "--seed", "7"]) == 2  # the second chunk would use seed 8
    assert "seed of the training run" in capsys.readouterr().out
    assert evaluate.main([*base, "--seed", "8"]) == 2
    assert not (checkpoint.parent / "evaluation-000001.json").exists()
    assert evaluate.main([*base, "--seed", "9"]) == 0
    assert evaluate.main([*base, "--seed", "6", "--tables", "24"]) == 0  # one chunk, seed 6


def test_c08_under_the_fault_the_deal_monitors_still_read_the_true_rewards():
    engine()
    sound = train.Trainer(ci_cfg(), CPU)
    faulty = train.Trainer(ci_cfg(**{"run.fault": "rewards_to_next_seat", "run.name": "control-smoke-a"}), CPU)
    a, b = collected(sound), collected(faulty)
    assert np.array_equal(a.parts, a.true_parts)  # without a fault they are the same numbers
    assert np.array_equal(b.true_parts, a.parts)  # with it: what the engine paid
    assert not np.array_equal(b.parts, b.true_parts)
    assert ppo.deal_stats(b) == ppo.deal_stats(a)
    assert ppo.deal_stats(b)["reward/deal_sum_abs_max"] < 1e-5


def test_c09_played_trained_and_the_two_shares_are_hand_counted_with_a_past_checkpoint_in_play(monkeypatch):
    engine()
    trainer = train.Trainer(busy_cfg(), CPU)
    spy = EnvSpy(trainer)
    marks, trained = [], []
    real = ppo.update

    def update(net, critic, optimizer, deals, *args, **kwargs):
        marks.append((spy.steps, spy.frozen_rows))
        keys = zip(deals.table.tolist(), deals.deal.tolist(), deals.seat.tolist())
        trained.append(sum(len(spy.actions[key]) for key in keys))
        return real(net, critic, optimizer, deals, *args, **kwargs)

    monkeypatch.setattr(ppo, "update", update)
    history = trainer.run(3)
    n = trainer.n
    before = (0, 0)
    for stats, mark, count in zip(history, marks, trained):
        steps, frozen = mark[0] - before[0], mark[1] - before[1]
        before = mark
        played = n * steps  # one decision per table and step
        own = played - frozen
        assert stats["rollout/played"] == played
        assert stats["rollout/played_own_seats"] == own
        assert stats["rollout/decisions"] == count
        assert stats["rollout/trained_share"] == pytest.approx(count / played)
        assert stats["rollout/trained_share_own_seats"] == pytest.approx(count / own)
        assert stats["speed/steps_per_second"] * stats["time/seconds"] == pytest.approx(played)
        assert stats["speed/trained_per_second"] * stats["time/seconds"] == pytest.approx(count)
    assert spy.frozen_rows > 50  # the past checkpoint did play, so the two shares differ
    assert history[-1]["rollout/trained_share"] < history[-1]["rollout/trained_share_own_seats"]
    assert trainer.frozen_decisions == spy.frozen_rows
    assert trainer.decisions == n * spy.steps - spy.frozen_rows
    assert history[-1]["opponents/frozen_share"] == pytest.approx(spy.frozen_rows / (n * spy.steps))


def test_c09_the_summary_of_a_run_on_hand_computed_numbers():
    history = [
        {"rollout/played": 100.0, "rollout/played_own_seats": 80.0, "rollout/decisions": 40.0},
        {"rollout/played": 60.0, "rollout/played_own_seats": 60.0, "rollout/decisions": 30.0},
    ]
    assert train.run_summary(history, seconds=10.0) == {
        "played": 160,
        "played_own_seats": 140,
        "trained": 70,
        "played_per_second": 16.0,
        "trained_per_second": 7.0,
        "trained_share": 70 / 160,
        "trained_share_own_seats": 0.5,
    }
    assert train.run_summary([], seconds=10.0)["trained_share"] == 0.0


def test_c09_the_summary_of_a_failed_run_counts_only_the_updates_that_were_logged(tmp_path, monkeypatch):
    engine()
    real = ppo.update
    calls = []

    def update(*args, **kwargs):
        stats = real(*args, **kwargs)
        calls.append(1)
        if len(calls) == 2:
            stats["loss/q"] = float("nan")
        return stats

    monkeypatch.setattr(ppo, "update", update)
    with pytest.raises(FloatingPointError, match="loss/q"):
        train.main(["--config", str(CI_TOML), "--out-dir", str(tmp_path), "--updates", "3"])
    summary = json.loads((tmp_path / "smoke-a-ci-mlp" / "summary.json").read_text(encoding="utf-8"))
    assert summary["finished"] is False and "loss/q" in summary["failed"]
    assert summary["updates"] == 1 and summary["updates_wanted"] == 3
    first = summary["first_update"]
    assert summary["played"] == first["rollout/played"] and summary["trained"] == first["rollout/decisions"]
    assert summary["trained_share"] == pytest.approx(first["rollout/trained_share"])


def test_c01_the_time_limit_of_the_config_reaches_the_loop_in_seconds(tmp_path, monkeypatch):
    engine()
    seen = []
    real = train.Trainer.run

    def run(self, updates, **kwargs):
        seen.append(kwargs["time_limit_seconds"])
        return real(self, updates, **kwargs)

    monkeypatch.setattr(train.Trainer, "run", run)
    base = ["--config", str(CI_TOML), "--updates", "1", "--out-dir"]
    assert train.main([*base, str(tmp_path / "a")]) == 0
    assert train.main([*base, str(tmp_path / "b"), "--time-limit-minutes", "1.5"]) == 0
    assert train.load_config(CI_TOML)["run"]["time_limit_minutes"] == 10
    assert seen == [600.0, 90.0]


def test_c10_each_clock_takes_the_time_of_its_own_work(monkeypatch):
    """A slow environment shows in the environment clock, a slow update in the update clock, and so on."""
    cfg = cfg_with(**{"run.network": "mlp", "env.tables": 8, "ppo.seat_deals_per_update": 24, "ppo.update_batch": 16})
    trainer = train.Trainer(cfg, CPU)
    slept = dict.fromkeys(train.CLOCKS, 0.0)

    def slow(function, clock, seconds):
        def wrapper(*args, **kwargs):
            started = time.perf_counter()
            time.sleep(seconds)
            slept[clock] += time.perf_counter() - started
            return function(*args, **kwargs)

        return wrapper

    trainer.env.step = slow(trainer.env.step, "environment", 0.03)
    trainer.actor.act = slow(trainer.actor.act, "acting", 0.02)
    trainer.rollout.take = slow(trainer.rollout.take, "bookkeeping", 0.5)
    monkeypatch.setattr(ppo, "update", slow(ppo.update, "update", 0.6))
    (stats,) = trainer.run(1)
    spent = {name: stats[f"time/{name}_share"] * stats["time/seconds"] for name in slept}
    smallest = min(slept.values())
    assert smallest > 0.25, slept
    for name in slept:
        # Each clock holds its own sleep and its own work, and nothing near the size of another clock's sleep.
        assert slept[name] * 0.999 <= spent[name] < slept[name] + 0.5 * smallest, (name, spent, slept)
    assert 0.95 <= sum(stats[f"time/{name}_share"] for name in slept) <= 1.0 + 1e-9


@pytest.mark.gpu
def test_c11_on_the_gpu_the_summary_holds_the_peak_of_the_tensors_the_reserved_peak_and_nvidia_smi(tmp_path):
    engine()
    assert train.main(["--config", str(CI_TOML), "--out-dir", str(tmp_path), "--updates", "1", "--device", "cuda"]) == 0
    summary = json.loads((tmp_path / "smoke-a-ci-mlp" / "summary.json").read_text(encoding="utf-8"))
    assert 0 < summary["peak_gpu_memory_mib"] <= summary["peak_gpu_memory_reserved_mib"]
    assert summary["gpu_after"]["memory_used_mib"] >= summary["peak_gpu_memory_reserved_mib"]
    last = summary["last_update"]
    assert last["gpu/peak_memory_mib"] <= last["gpu/peak_memory_reserved_mib"]


def test_c13_entropy_over_the_decisions_with_a_choice_and_the_share_of_forced_decisions(engine_run):
    _, history = engine_run
    for stats in history:
        for name in ("bidding", "contract choice", "talon pick", "discard", "play"):
            forced = stats[f"forced_share/{name}"]
            assert 0 <= forced <= 1
            if forced < 1:
                # A decision with one legal slot has entropy 0, so the two means differ by exactly that share.
                assert stats[f"entropy/{name}"] == pytest.approx(stats[f"entropy_choice/{name}"] * (1 - forced), abs=1e-5)
        assert stats["forced_share/talon pick"] == 0 and stats["forced_share/discard"] == 0
        assert 0.1 < stats["forced_share/play"] < 0.6 and 0.5 < stats["forced_share/contract choice"] <= 1
        assert stats["entropy_choice/play"] > stats["entropy/play"]


def test_c13_the_report_names_a_collapse_by_the_fixed_limit_and_one_relative_to_the_target(tmp_path):
    from torch.utils.tensorboard import SummaryWriter

    from tarok_train import smoke_report

    cfg = train.load_config(SMOKE["mlp"])
    assert cfg["entropy_control"]["targets"]["contract choice"] == 0.03 and cfg["entropy_control"]["band"] == 0.75
    writer = SummaryWriter(str(tmp_path))
    last = {"bidding": 0.30, "contract choice": 0.04, "play": 0.01, "discard": 0.2}
    for step in (1, 2):
        writer.add_scalar("loss/total", 1.0, step)
        for name, value in last.items():
            writer.add_scalar(f"entropy/{name}", 1.0 if step == 1 else value, step)
    writer.close()
    info = {"started": "2026-11-12T00:00:00", "commit": "0" * 40, "git_status_short": "", "config_file": "x.toml", "run": "x", "config": cfg, "engine_sources": "0" * 16}
    (tmp_path / "run-info.json").write_text(json.dumps(info), encoding="utf-8")
    text = smoke_report.report(tmp_path)
    # Fixed limit 0.05: contract choice (0.04) and play (0.01). Relative: the lower edge is a quarter of the
    # target. Contract choice 0.04 is above 0.0075; play 0.01 is under 0.0625; discard 0.2 is above 0.15.
    assert "Collapse by the spec's fixed limit (entropy under 0.05 at the last update): contract choice, play." in text
    assert "target x 0.25): play (0.010 under 0.062)." in text or "target x 0.25): play (0.010 under 0.063)." in text


def test_c14_at_the_talon_pick_and_the_discard_no_unseen_card_is_out_of_play():
    for phase in ("talon pick", "discard"):
        for role in ("declarer", "opponent"):
            state = torch.from_numpy(state_with(B1=C.PHASES.index(phase), B4=C.ROLES.index(role)))[None]
            places = ppo.possible_places(state)[0]
            assert not places[:, 0].any(), (phase, role)  # the talon lies open, nothing is discarded unseen
            assert places[:, 1:].all()
    for phase in ("bidding", "contract choice", "king call"):
        places = ppo.possible_places(torch.from_numpy(state_with(B1=C.PHASES.index(phase)))[None])[0]
        assert places[:, 0].all(), phase  # the talon is face down


def test_c14_the_guessing_level_counts_the_seats_still_possible_for_each_decision():
    declarer, partner, opponent = (C.ROLES.index(name) for name in ("declarer", "called partner", "opponent"))
    play = C.PHASES.index("play")
    state = torch.from_numpy(
        np.stack(
            [
                state_with(B1=play, B4=declarer, B3=0),  # three seats possible; the head says 2, the truth is 2
                state_with(B1=play, B4=opponent, B3=1),  # two seats possible; the head says 2, the truth is 3
                state_with(B1=play, B4=partner, B3=3),  # one seat possible; the head says 2, the truth is 3
                state_with(B1=C.PHASES.index("bidding")),  # no partner yet: not counted
            ]
        )
    )
    partner_logits = torch.zeros(4, C.PARTNER_CLASSES)
    partner_logits[:, 2] = 1.0
    partner_target = torch.tensor([2, 3, 3, C.PARTNER_NONE])
    card_logits = torch.zeros(4, C.N_CARDS, C.CARD_LOCATION_CLASSES)
    card_logits[:, :, 1] = 1.0  # the head says "seat 1" for every card
    card_target = torch.ones(4, C.N_CARDS, dtype=torch.long)
    card_target[0, 0] = 2
    unseen = torch.zeros(4, C.N_CARDS, dtype=torch.bool)
    unseen[0, :2] = True  # the declarer in play: three places for each card
    unseen[3, 30] = True  # bidding: four places
    sums = ppo.aux_sums(partner_logits, card_logits, state, partner_target, card_target, unseen)
    assert sums[0].tolist() == pytest.approx([1.0, 1 / 3 + 1 / 2 + 1.0, 3.0])  # hits, hits of a uniform guess, decisions
    assert sums[1].tolist() == pytest.approx([2.0, 1 / 3 + 1 / 3 + 1 / 4, 3.0])


def test_d130_a_module_built_before_the_fingerprint_existed_is_refused(monkeypatch):
    engine_env = engine()
    import tarok_env

    monkeypatch.delattr(tarok_env, "SOURCE_HASH")
    with pytest.raises(engine_env.StaleModuleError, match="maturin develop --release"):
        engine_env.EngineEnv(2, seed=1, threads=1)
