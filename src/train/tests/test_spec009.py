"""Tests for stage (d) of spec 009 (deal flow B): the Python side.

The name of each test starts with its criterion number. Criterion 52: `step(actions)`
works as before, `step(actions, hold)` holds. Criterion 53: the engine version in a
checkpoint; a checkpoint of another engine version, or of none, is not resumed, and
the evaluation takes it only with a flag. Criterion 45: the round of announcements is
in play in the trainer and its entropy is logged (the smoke run itself is in the
Results of the spec).
"""

import numpy as np
import pytest
import torch

from tarok_train import contract as C
from tarok_train import train

from test_spec007 import CPU, small
from test_spec008 import CI_TOML, ci_cfg, engine
from test_spec015 import result_of, run_evaluation, trained_checkpoint, versus

ANNOUNCEMENTS = C.PHASES.index("announcements")


def phase_of(obs: dict) -> np.ndarray:
    """The index into C.PHASES of every table (state block B1)."""
    start, size = C.STATE_BLOCKS["B1"]
    return obs["state"][:, start : start + size].argmax(axis=1)


def lowest_legal(obs: dict) -> np.ndarray:
    return obs["mask"].argmax(axis=1)


def by_observation(obs: dict) -> np.ndarray:
    """A legal slot per table that depends on nothing but the table's own observation."""
    mask = obs["mask"]
    pick = (obs["tokens"].astype(np.int64).sum(axis=(1, 2)) + obs["n_tokens"]) % mask.sum(axis=1)
    order = np.argsort(~mask, axis=1, kind="stable")  # the legal slots first, lowest first
    return order[np.arange(len(mask)), pick]


def with_partner_games(obs: dict) -> np.ndarray:
    """As `by_observation`, but in the auction the lowest legal slot three times out of four.

    Drawn evenly, nearly every auction ends in a solo: no king is called and no partner is hidden.
    """
    actions = by_observation(obs)
    # Which decisions: by the cards the seat holds, so that it is not always the same bidder.
    draw = obs["state"][:, :54].astype(np.int64) @ np.arange(1, 55) + obs["deal"] + obs["seat"]
    auction = (phase_of(obs) < C.PHASES.index("king call")) & (draw % 4 != 0)
    actions[auction] = lowest_legal(obs)[auction]
    return actions


def copies(obs: dict) -> dict:
    return {name: value.copy() for name, value in obs.items()}


# --------------------------------------------------------------------------
# Criterion 52: the hold flag in EngineEnv.step
# --------------------------------------------------------------------------


def test_c52_step_without_hold_works_as_before_and_no_table_held_is_the_same():
    engine_env = engine()
    n = 8
    plain, with_none, with_flags = (engine_env.EngineEnv(n, 52, 2) for _ in range(3))
    nobody = np.zeros(n, dtype=bool)
    done_seen = 0
    for _ in range(400):
        actions = by_observation(plain.observe())
        results = [plain.step(actions), with_none.step(actions, None), with_flags.step(actions, hold=nobody)]
        for other in results[1:]:
            for one, two in zip(results[0], other):
                assert np.array_equal(one, two)
        for env in (with_none, with_flags):
            for name, value in plain.observe().items():
                assert np.array_equal(value, env.observe()[name]), name
            assert np.array_equal(plain.entries, env.entries)
        done_seen += int(results[0][0].sum())
    assert done_seen >= 8


def test_c52_a_held_table_does_not_move_and_its_action_need_not_be_legal():
    engine_env = engine()
    n = 6
    env = engine_env.EngineEnv(n, 5252, 2)
    for _ in range(7):  # some way into the deals
        env.step(lowest_legal(env.observe()))
    before = copies(env.observe())
    hold = np.array([True, False, True, False, False, True])
    actions = lowest_legal(before)
    # The held tables get actions that are not legal: a slot that is never legal, a
    # slot outside the 160, and the highest slot. They are ignored.
    illegal = int(np.flatnonzero(~before["mask"][0])[0])
    actions[0], actions[2], actions[5] = illegal, 10_000, C.N_ACTIONS - 1
    assert not before["mask"][5, C.N_ACTIONS - 1]
    done, reward, parts = env.step(actions, hold)
    after = env.observe()
    for name, value in before.items():
        assert np.array_equal(after[name][hold], value[hold]), f"{name} of a held table changed"
    assert not done[hold].any() and not reward[hold].any() and not parts[hold].any()
    assert not env.entries[hold].any()
    assert env.illegal_actions == 0
    # The tables that are not held took their action: one more token, or a hidden discard.
    moved = ~hold
    assert (after["n_tokens"][moved] >= before["n_tokens"][moved]).all()
    assert any(not np.array_equal(after[name][moved], before[name][moved]) for name in before)

    # An illegal action of a table that is not held is still an error, and nothing moves.
    before = copies(env.observe())
    actions = lowest_legal(before)
    actions[1] = int(np.flatnonzero(~before["mask"][1])[0])
    with pytest.raises(ValueError, match="illegal action"):
        env.step(actions, hold)
    for name, value in before.items():
        assert np.array_equal(env.observe()[name], value), name
    assert env.illegal_actions == 1


def test_c52_holding_never_changes_what_a_table_plays():
    engine_env = engine()
    n = 12
    free = engine_env.EngineEnv(n, 525, 2)
    held = engine_env.EngineEnv(n, 525, 2)
    rng = np.random.default_rng(525)

    def deals_of(env, steps, holds):
        """Per table: the entries of its finished deals, in order, and the round's decisions seen."""
        entries = [[] for _ in range(n)]
        in_round = 0
        for _ in range(steps):
            obs = env.observe()
            hold = rng.random(n) < 0.5 if holds else None
            if hold is None:
                done, _, _ = env.step(by_observation(obs))
                in_round += int((phase_of(obs) == ANNOUNCEMENTS).sum())
            else:
                actions = by_observation(obs)
                actions[hold] = -1  # ignored
                done, _, _ = env.step(actions, hold)
                assert not done[hold].any()
            for table in np.flatnonzero(done):
                entries[table].append(env.entries[table].tolist())
        return entries, in_round

    plain, in_round = deals_of(free, 1500, holds=False)
    with_holds, _ = deals_of(held, 3200, holds=True)
    compared = 0
    for table in range(n):
        shared = min(len(plain[table]), len(with_holds[table]))
        assert shared >= 5, f"table {table}: {shared} deals"
        assert plain[table][:shared] == with_holds[table][:shared], f"table {table}"
        compared += shared
    assert compared >= 100
    assert in_round > 1000, "the round of announcements is in play"


def test_c52_a_hold_of_another_shape_or_type_is_refused_and_nothing_moves():
    engine_env = engine()
    env = engine_env.EngineEnv(4, 52, 1)
    before = copies(env.observe())
    actions = lowest_legal(before)
    with pytest.raises(ValueError, match=r"hold.*\(4,\)"):
        env.step(actions, np.zeros(3, dtype=bool))
    with pytest.raises(ValueError, match=r"hold.*\(4,\)"):
        env.step(actions, np.zeros((4, 1), dtype=bool))
    with pytest.raises(TypeError, match="hold.*bool"):
        env.step(actions, np.zeros(4, dtype=np.int64))
    with pytest.raises(TypeError, match="hold.*bool"):
        env.step(actions, [0, 1, 0, 1])
    for name, value in before.items():
        assert np.array_equal(env.observe()[name], value), name
    assert env.illegal_actions == 0
    # A list of bools is taken.
    done, _, _ = env.step(actions, [True, True, True, True])
    assert not done.any()
    for name, value in before.items():
        assert np.array_equal(env.observe()[name], value), name


# --------------------------------------------------------------------------
# Criterion 53: the engine version in checkpoints
# --------------------------------------------------------------------------


def engine_version() -> str:
    import tarok_env

    return tarok_env.ENGINE_VERSION


def with_engine_version(path, version, keep=True):
    """Rewrite a checkpoint as one made with another engine version, or (keep=False) before the guard."""
    data = torch.load(path, map_location="cpu", weights_only=True)
    if keep:
        data["engine_version"] = version
    else:
        del data["engine_version"]
    torch.save(data, path)


def ci_checkpoint(tmp_path):
    assert train.main(["--config", str(CI_TOML), "--out-dir", str(tmp_path), "--updates", "1"]) == 0
    return tmp_path / "smoke-a-ci-mlp" / "checkpoint-000001.pt"


def test_c53_a_checkpoint_holds_the_engine_version_and_the_encoding_version_stays_1(tmp_path, capsys):
    engine()
    import tarok_env

    checkpoint = ci_checkpoint(tmp_path)
    data = torch.load(checkpoint, map_location="cpu", weights_only=True)
    assert data["engine_version"] == tarok_env.ENGINE_VERSION
    assert data["encoding_version"] == 1 == C.ENCODING_VERSION == tarok_env.ENCODING_VERSION
    # The same engine: the run goes on, and the next checkpoint holds the version too.
    capsys.readouterr()
    assert train.main(["--resume", str(checkpoint), "--updates", "2"]) == 0
    assert "engine version" not in capsys.readouterr().out
    resumed = torch.load(checkpoint.parent / "checkpoint-000002.pt", map_location="cpu", weights_only=True)
    assert resumed["engine_version"] == tarok_env.ENGINE_VERSION


@pytest.mark.parametrize("made_with", ["0.2.0", None])
def test_c53_a_checkpoint_of_another_engine_version_or_of_none_is_not_resumed(tmp_path, capsys, made_with):
    engine()
    checkpoint = ci_checkpoint(tmp_path)
    with_engine_version(checkpoint, made_with, keep=made_with is not None)
    named = "0.2.0" if made_with is not None else "no engine version"
    with pytest.raises(train.EngineVersionError) as error:
        train.Trainer.load(checkpoint, CPU)
    assert named in str(error.value) and engine_version() in str(error.value)

    # From the command line: one message that names both, exit code 2, nothing written.
    before = sorted(path.name for path in checkpoint.parent.iterdir())
    capsys.readouterr()
    assert train.main(["--resume", str(checkpoint), "--updates", "2"]) == 2
    out = capsys.readouterr().out
    assert named in out and engine_version() in out and "engine version" in out
    assert sorted(path.name for path in checkpoint.parent.iterdir()) == before


def test_c53_a_checkpoint_of_the_mock_environment_names_no_engine_and_is_resumed(tmp_path):
    trainer = train.Trainer(small(), CPU)
    trainer.run(1)
    path = tmp_path / "mock.pt"
    trainer.save(path)
    trainer.close()
    data = torch.load(path, map_location="cpu", weights_only=True)
    assert data["engine_version"] is None  # no engine played these deals
    train.Trainer.load(path, CPU).close()
    # A mock checkpoint from before the guard is resumed as well: it holds no rules of the engine.
    with_engine_version(path, None, keep=False)
    train.Trainer.load(path, CPU).close()


@pytest.mark.parametrize("made_with", ["0.2.0", None])
def test_c53_the_evaluation_refuses_such_a_checkpoint_as_agent_and_as_opponent(tmp_path, capsys, made_with):
    engine()
    a = trained_checkpoint(tmp_path, "a", seed=8)
    b = trained_checkpoint(tmp_path, "b", seed=9)
    with_engine_version(b, made_with, keep=made_with is not None)
    named = "0.2.0" if made_with is not None else "no engine version"
    # As the opponent.
    capsys.readouterr()
    assert run_evaluation("--checkpoint", a, "--opponent", b, "--deals", 24, "--tables", 12, "--seed", 5) == 2
    out = capsys.readouterr().out
    assert "opponent" in out and named in out and engine_version() in out and "--other-engine" in out
    # As the agent, against random play and against a checkpoint.
    assert run_evaluation("--checkpoint", b, "--deals", 24, "--tables", 12, "--seed", 5) == 2
    out = capsys.readouterr().out
    assert named in out and engine_version() in out and "--other-engine" in out
    assert run_evaluation("--checkpoint", b, "--opponent", a, "--deals", 24, "--tables", 12, "--seed", 5) == 2
    assert not list(a.parent.glob("evaluation-*")) and not list(b.parent.glob("evaluation-*"))


def test_c53_with_the_flag_the_evaluation_takes_it_and_prints_a_warning(tmp_path, capsys):
    engine()
    a = trained_checkpoint(tmp_path, "a", seed=8)
    b = trained_checkpoint(tmp_path, "b", seed=9)
    # Two checkpoints of this engine: no warning, and the result names the engine.
    capsys.readouterr()
    assert run_evaluation("--checkpoint", a, "--opponent", b, "--deals", 24, "--tables", 12, "--seed", 5) == 0
    assert "warning" not in capsys.readouterr().out.lower()
    result = result_of(versus(a, b))
    assert result["engine_version"] == engine_version()
    assert result["checkpoint_engine_version"] == engine_version()
    assert result["opponent_engine_version"] == engine_version()
    # The flag alone changes nothing for them.
    assert run_evaluation("--checkpoint", a, "--deals", 24, "--tables", 12, "--seed", 5, "--other-engine") == 0
    assert "warning" not in capsys.readouterr().out.lower()

    # The opponent was made with another engine: taken with the flag, with a warning.
    with_engine_version(b, "0.2.0")
    assert run_evaluation("--checkpoint", a, "--opponent", b, "--deals", 24, "--tables", 12, "--seed", 5, "--other-engine", "--overwrite") == 0
    out = capsys.readouterr().out
    assert "warning" in out.lower() and "0.2.0" in out and engine_version() in out and "opponent" in out
    result = result_of(versus(a, b))
    assert result["engine_version"] == engine_version()
    assert result["checkpoint_engine_version"] == engine_version()
    assert result["opponent_engine_version"] == "0.2.0"

    # The agent was made before the guard: the same, and the result says "none".
    with_engine_version(b, None, keep=False)
    assert run_evaluation("--checkpoint", b, "--deals", 24, "--tables", 12, "--seed", 5, "--other-engine") == 0
    out = capsys.readouterr().out
    assert "warning" in out.lower() and "no engine version" in out and engine_version() in out
    result = result_of(b.parent / "evaluation-000001.json")
    assert result["checkpoint_engine_version"] is None and result["engine_version"] == engine_version()
    assert result["opponent_engine_version"] is None  # against random legal play


# --------------------------------------------------------------------------
# Criterion 45: the round of announcements in the trainer
# --------------------------------------------------------------------------


def test_c45_the_round_is_in_play_and_its_entropy_is_logged():
    engine()
    trainer = train.Trainer(ci_cfg(**{"run.network": "mlp"}), CPU)
    history = trainer.run(2)
    trainer.close()
    for stats in history:
        # Decisions of the round reach the learner, and their entropy has its own tag.
        assert "entropy/announcements" in stats
        assert 0.0 < stats["entropy/announcements"] <= np.log(12) + 1e-6  # at most the 12 slots of the round
        assert stats["rollout/decisions_per_seat_deal_mean"] > 15  # flow A had about 14 per seat
    # The round has its own entry in the entropy control.
    assert len(trainer.entropy.weights) == len(C.PHASES)


def test_c45_the_slots_of_the_round_are_legal_only_in_the_round_and_block_d_is_filled():
    engine_env = engine()
    env = engine_env.EngineEnv(64, 45, 2)
    slots = np.arange(77, 89)
    d1, d1_size = C.STATE_BLOCKS["D1"]
    d2, d2_size = C.STATE_BLOCKS["D2"]
    seen_in_round = np.zeros(C.N_ACTIONS, dtype=bool)
    announced = 0
    for _ in range(600):
        obs = env.observe()
        in_round = phase_of(obs) == ANNOUNCEMENTS
        assert not obs["mask"][~in_round][:, slots].any(), "a slot of the round is legal outside it"
        assert obs["mask"][in_round, 88].all(), '"done" is always legal in the round'
        assert not obs["mask"][in_round][:, :77].any() and not obs["mask"][in_round][:, 89:].any()
        seen_in_round |= obs["mask"][in_round].any(axis=0)
        # D2: one of five entries per target, at every decision of a deal.
        levels = obs["state"][:, d2 : d2 + d2_size].reshape(-1, 6, 5)
        assert (levels.sum(axis=2) == 1).all()
        announced += int(obs["state"][:, d1 : d1 + d1_size].any(axis=1).sum())
        env.step(with_partner_games(obs))
    assert seen_in_round[slots].all(), "every slot of the round was legal at some decision"
    assert announced > 1000


# --------------------------------------------------------------------------
# The review of stage (d), points 2, 3 and 6. Tests named test_r<n>_ belong to
# point n (see the section on that review in the spec's Results).
# --------------------------------------------------------------------------

CONFIGS = CI_TOML.parent


def test_r2_run_info_and_summary_name_the_engine_version(tmp_path):
    engine()
    import json

    assert train.main(["--config", str(CI_TOML), "--out-dir", str(tmp_path), "--updates", "1"]) == 0
    folder = tmp_path / "smoke-a-ci-mlp"
    for name in ("run-info.json", "summary.json"):
        data = json.loads((folder / name).read_text(encoding="utf-8"))
        assert data["engine_version"] == engine_version(), name


def test_r2_a_mock_run_names_no_engine_in_run_info_and_summary(tmp_path):
    import json

    from test_spec007 import MOCK_TOML

    assert train.main(["--config", str(MOCK_TOML), "--out-dir", str(tmp_path), "--updates", "1", "--tables", "8", "--seat-deals", "16"]) == 0
    (folder,) = [path for path in tmp_path.iterdir() if path.is_dir()]
    for name in ("run-info.json", "summary.json"):
        assert json.loads((folder / name).read_text(encoding="utf-8"))["engine_version"] is None, name


def test_r2_no_config_says_that_the_engine_has_no_announcements():
    engine_configs = 0
    for path in sorted(CONFIGS.glob("*.toml")):
        text = path.read_text(encoding="utf-8")
        assert "deal flow A has no announcements" not in text, path.name
        if 'kind = "engine"' not in text:
            continue  # the mock environment has its own phases (mock.toml)
        engine_configs += 1
        for line in text.splitlines():
            if line.startswith("kind = \"engine\""):
                assert "deal flow B" in line and "0.3.0" in line, path.name
            if line.startswith("announcements ="):
                assert "D149" in line and "spec 018" in line, path.name
    assert engine_configs >= 12  # the twelve engine configs that existed when spec 009 was merged; new ones add to it


def test_r2_the_readme_says_which_engine_the_runs_of_flow_a_were_made_on():
    text = (CONFIGS.parent / "README.md").read_text(encoding="utf-8")
    assert "The run is on deal flow A only" not in text and "The trial is on deal flow A." not in text
    assert text.count("engine 0.2.0") >= 2


def fake_evaluation(path, **keys):
    import json

    data = {"mean": 0.0, "low": -1.0, "high": 1.0, "values": [0.0], "seed": 5, "deals": 24, "tables": 12, **keys}
    path.write_text(json.dumps(data), encoding="utf-8")
    return path.read_text(encoding="utf-8")


def test_r3_with_another_engine_the_file_against_random_play_is_replaced_only_with_overwrite(tmp_path, capsys):
    engine()
    b = trained_checkpoint(tmp_path, "b", seed=9)
    with_engine_version(b, "0.2.0")
    # The file an evaluation on engine 0.2.0 wrote: it names no engine version.
    earlier = fake_evaluation(b.parent / "evaluation-000001.json")
    capsys.readouterr()
    assert run_evaluation("--checkpoint", b, "--deals", 24, "--tables", 12, "--seed", 5, "--other-engine") == 2
    out = capsys.readouterr().out
    assert "evaluation-000001.json" in out and "--overwrite" in out
    assert (b.parent / "evaluation-000001.json").read_text(encoding="utf-8") == earlier
    assert run_evaluation("--checkpoint", b, "--deals", 24, "--tables", 12, "--seed", 5, "--other-engine", "--overwrite") == 0
    assert result_of(b.parent / "evaluation-000001.json")["engine_version"] == engine_version()


def test_r3_a_file_of_another_engine_is_not_replaced_without_overwrite_and_one_of_this_engine_is(tmp_path, capsys):
    engine()
    a = trained_checkpoint(tmp_path, "a", seed=8)
    path = a.parent / "evaluation-000001.json"
    earlier = fake_evaluation(path, engine_version="0.2.0")
    capsys.readouterr()
    assert run_evaluation("--checkpoint", a, "--deals", 24, "--tables", 12, "--seed", 5) == 2
    assert "--overwrite" in capsys.readouterr().out and path.read_text(encoding="utf-8") == earlier
    # A file of this engine: an evaluation against random play replaces it, as before.
    fake_evaluation(path, engine_version=engine_version())
    assert run_evaluation("--checkpoint", a, "--deals", 24, "--tables", 12, "--seed", 5) == 0
    assert result_of(path)["deals"] == 24 and len(result_of(path)["values"]) == 24


def test_r3_the_two_directions_of_a_comparison_must_name_one_engine():
    from tarok_train import trial_compare
    from test_spec017 import evaluation

    one, other = evaluation([0.1, -0.1]), evaluation([0.2, 0.0])
    # Files without the key were written on engine 0.2.0.
    trial_compare.paired(one, other)
    trial_compare.paired({**one, "engine_version": "0.2.0"}, other)
    trial_compare.paired({**one, "engine_version": "0.3.0"}, {**other, "engine_version": "0.3.0"})
    for a, b in ((one, {**other, "engine_version": "0.3.0"}), ({**one, "engine_version": "0.3.0"}, {**other, "engine_version": "0.4.0"})):
        with pytest.raises(ValueError, match="engine"):
            trial_compare.paired(a, b)


def with_sources(path, sources, keep=True):
    data = torch.load(path, map_location="cpu", weights_only=True)
    if keep:
        data["engine_sources"] = sources
    else:
        del data["engine_sources"]
    torch.save(data, path)


def test_r6_a_checkpoint_holds_the_hash_of_the_engine_sources_and_another_hash_gives_a_warning(tmp_path, capsys):
    engine()
    import tarok_env

    checkpoint = ci_checkpoint(tmp_path)
    assert torch.load(checkpoint, map_location="cpu", weights_only=True)["engine_sources"] == tarok_env.SOURCE_HASH
    capsys.readouterr()
    train.Trainer.load(checkpoint, CPU).close()
    assert "warning" not in capsys.readouterr().out.lower()
    # The same version, other sources: a warning that names both hashes, and the run goes on.
    with_sources(checkpoint, "0123456789abcdef")
    train.Trainer.load(checkpoint, CPU).close()
    out = capsys.readouterr().out
    assert "warning" in out.lower() and "0123456789abcdef" in out and tarok_env.SOURCE_HASH in out
    # The evaluation says the same.
    assert run_evaluation("--checkpoint", checkpoint, "--deals", 24, "--tables", 12, "--seed", 5) == 0
    out = capsys.readouterr().out
    assert "warning" in out.lower() and "0123456789abcdef" in out
    # Without the hash (a checkpoint from before it was kept): no warning.
    with_sources(checkpoint, None, keep=False)
    train.Trainer.load(checkpoint, CPU).close()
    assert "warning" not in capsys.readouterr().out.lower()


def test_r6_a_mock_checkpoint_holds_no_hash():
    trainer = train.Trainer(small(), CPU)
    import tempfile
    from pathlib import Path

    with tempfile.TemporaryDirectory() as folder:
        path = Path(folder) / "mock.pt"
        trainer.save(path)
        trainer.close()
        assert torch.load(path, map_location="cpu", weights_only=True)["engine_sources"] is None
