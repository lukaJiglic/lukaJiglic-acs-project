"""Tests for spec 006, criteria 20 to 23: the Python module `tarok_env` and `tarok_train/env.py`.

The name of each test starts with its criterion number. The interface tests of
criterion 22 run against the mock environment of spec 007 and against the engine,
with the same code: what the learner may rely on must hold for both.

The module is built with `maturin develop --release` in `src/train/tarok_env`.
Where it is not built the tests are skipped; with TAROK_REQUIRE_ENV=1 (set in CI)
a missing module is a failure instead.
"""

import os
import threading
import time

import numpy as np
import pytest
import torch

from tarok_train import contract as C
from tarok_train import mock_env, ppo, train

from test_spec007 import CPU, cfg_with, collected, small

if os.environ.get("TAROK_REQUIRE_ENV") == "1":
    import tarok_env
else:
    tarok_env = pytest.importorskip("tarok_env", reason="the tarok_env module is not built (maturin develop --release)")

from tarok_train import env as engine_env  # noqa: E402  (needs tarok_env)

N_STATE = C.STATE_FEATURES
N_CRITIC = C.CRITIC_FEATURES


def make_env(kind: str, tables: int, seed: int):
    """An environment of either kind with the learner's interface."""
    if kind == "mock":
        return mock_env.make(cfg_with(**{"env.tables": tables})["env"], seed)
    return engine_env.EngineEnv(tables, seed, threads=2)


def random_legal(mask: np.ndarray, rng: np.random.Generator) -> np.ndarray:
    """One legal slot per table, uniformly."""
    scores = rng.random(mask.shape)
    scores[~mask] = -1.0
    return scores.argmax(axis=1)


BOTH = pytest.mark.parametrize("kind", ["mock", "engine"])


# --------------------------------------------------------------------------
# Criterion 20: the module, the arrays, the GIL
# --------------------------------------------------------------------------


def test_c20_the_module_exports_the_sizes_of_the_contract():
    assert tarok_env.ENCODING_VERSION == C.ENCODING_VERSION
    assert tarok_env.N_EVENTS == C.N_EVENTS == 288
    assert tarok_env.STATE_FEATURES == C.STATE_FEATURES == 1382
    assert tarok_env.N_ACTIONS == C.N_ACTIONS == 160
    assert tarok_env.MAX_TOKENS == C.MAX_TOKENS == 128
    assert tarok_env.TOKEN_FIELDS == len(C.TOKEN_FIELDS) == 5
    assert tarok_env.CRITIC_EXTRA == C.CRITIC_EXTRA == 274
    assert tarok_env.CRITIC_FEATURES == C.CRITIC_FEATURES == 1656
    assert tarok_env.N_SCORE_PARTS == C.N_SCORE_PARTS == 10
    assert tarok_env.DEALS_PER_RUN == 24
    assert isinstance(tarok_env.ENGINE_VERSION, str) and tarok_env.ENGINE_VERSION


def test_c20_arrays_have_the_documented_shapes_and_types():
    n = 7
    env = engine_env.EngineEnv(n, seed=20, threads=3)
    assert (env.n, env.threads) == (n, 3)
    obs = env.observe()
    expected = {
        "seat": ((n,), np.int64),
        "deal": ((n,), np.int64),
        "tokens": ((n, 128, 5), np.int16),
        "n_tokens": ((n,), np.int64),
        "state": ((n, 1382), np.float32),
        "mask": ((n, 160), np.bool_),
        "critic": ((n, 1656), np.float32),
        "partner_target": ((n,), np.int64),
        "card_target": ((n, 54), np.int64),
    }
    assert set(obs) == set(expected)
    for name, (shape, dtype) in expected.items():
        assert isinstance(obs[name], np.ndarray), name
        assert obs[name].shape == shape, name
        assert obs[name].dtype == dtype, name
        assert obs[name].flags["C_CONTIGUOUS"], name

    done, reward, parts = env.step(random_legal(obs["mask"], np.random.default_rng(0)))
    assert done.shape == (n,) and done.dtype == np.bool_
    assert reward.shape == (n, 4) and reward.dtype == np.float32
    assert parts.shape == (n, 4, 10) and parts.dtype == np.float32
    assert env.entries.shape == (n, 4) and env.entries.dtype == np.int32

    # The low-level class refuses arrays of another type or shape instead of writing into them.
    tables = tarok_env.Tables(n, 20, 1)
    buffers = engine_env.new_buffers(n)
    wrong = dict(buffers)
    wrong["state"] = np.zeros((n, 1382), dtype=np.float64)
    with pytest.raises(TypeError):
        tables.observe_into(*[wrong[name] for name in engine_env.OBSERVATION_ARRAYS])
    wrong = dict(buffers)
    wrong["state"] = np.zeros((n - 1, 1382), dtype=np.float32)
    with pytest.raises(ValueError, match="state"):
        tables.observe_into(*[wrong[name] for name in engine_env.OBSERVATION_ARRAYS])
    with pytest.raises(ValueError):
        tarok_env.Tables(0, 20, 1)
    with pytest.raises(ValueError):
        tarok_env.Tables(4, 20, 0)


def test_c20_a_python_thread_keeps_running_during_a_long_step():
    """The call releases the GIL: a counting thread makes progress while Rust steps."""
    env = engine_env.EngineEnv(2048, seed=20, threads=2)
    counted = []
    stop = threading.Event()

    def count():
        n = 0
        while not stop.is_set():
            n += 1
            if n % 1000 == 0:
                time.sleep(0)  # let the main thread take the GIL back when the call returns
        counted.append(n)

    # How fast the counter runs alone, as a reference. Over the time it really ran: the main thread
    # asks for the GIL back after its sleep and may wait for it far longer than the 0.2 s it slept
    # (up to about 2 s on the training PC, 2026-11-25). Divided by the nominal 0.2 s the reference
    # came out up to ten times too fast, and the test failed or passed by chance.
    worker = threading.Thread(target=count)
    alone_started = time.perf_counter()
    worker.start()
    time.sleep(0.2)
    stop.set()
    worker.join()
    alone_per_second = counted.pop() / (time.perf_counter() - alone_started)

    stop.clear()
    worker = threading.Thread(target=count)
    started = time.perf_counter()
    worker.start()
    decisions, deals = env.run_random(steps=1500, seed=1)  # one long call into Rust
    elapsed = time.perf_counter() - started
    stop.set()
    worker.join()
    during = counted.pop()

    assert decisions == 1500 * 2048 and deals > 10_000
    assert elapsed > 0.1, f"the call took {elapsed:.3f} s: too short to show anything"
    # With the GIL held for the whole call the counter would stand still. It ran at a
    # fair share of its speed alone.
    assert during > 0.2 * alone_per_second * elapsed, (during, alone_per_second, elapsed)


def test_c20_an_array_of_another_shape_or_memory_order_is_refused():
    """Review item B7. The engine fills an array's memory as it lies.

    An array with the right number of entries but another shape or order would be
    filled with the right numbers in the wrong places. Each is refused with an error
    that names the array, its shape and the shape the engine needs.
    """
    n = 4
    tables = tarok_env.Tables(n, 20, 1)
    good = engine_env.new_buffers(n)
    observation = list(engine_env.OBSERVATION_ARRAYS)
    everything = [*engine_env.OBSERVATION_ARRAYS, *engine_env.RESULT_ARRAYS]
    tables.observe_into(*[good[name] for name in observation])
    reference = {name: array.copy() for name, array in good.items()}
    actions = np.array([int(np.flatnonzero(row)[0]) for row in good["mask"]], dtype=np.int64)

    def calls(arrays):
        """The three calls that take arrays, with these arrays."""
        return (
            lambda: tables.observe_into(*[arrays[name] for name in observation]),
            lambda: tables.step_into(actions, *[arrays[name] for name in everything]),
            lambda: tables.run_random(3, 1, *[arrays[name] for name in everything]),
        )

    def refused(name, array, *needles):
        wrong = dict(good)
        wrong[name] = array
        for index, call in enumerate(calls(wrong)):
            if index == 0 and name not in engine_env.OBSERVATION_ARRAYS:
                continue  # observe_into takes no result arrays
            with pytest.raises(ValueError) as caught:
                call()
            message = str(caught.value)
            for needle in (name, *needles):
                assert needle in message, f"{name}: {message}"

    # A Fortran-ordered state: the same shape, another order in memory.
    fortran = np.asfortranarray(good["state"])
    assert fortran.shape == (n, 1382) and not fortran.flags["C_CONTIGUOUS"]
    refused("state", fortran, "(4, 1382)", "C-contiguous")
    # Axes swapped: the same number of entries.
    refused("tokens", np.zeros((n, 5, 128), dtype=np.int16), "(4, 5, 128)", "(4, 128, 5)")
    refused("state", np.zeros((1382, n), dtype=np.float32), "(1382, 4)", "(4, 1382)")
    refused("parts", np.zeros((n, 10, 4), dtype=np.float32), "(4, 10, 4)", "(4, 4, 10)")
    refused("reward", np.asfortranarray(np.zeros((n, 4), dtype=np.float32)), "(4, 4)", "C-contiguous")
    # Every array: one table too many, and every second row of a larger array.
    for name, (shape, dtype) in {**engine_env.OBSERVATION_ARRAYS, **engine_env.RESULT_ARRAYS}.items():
        expected = str((n, *shape))
        refused(name, np.zeros((n + 1, *shape), dtype=dtype), str((n + 1, *shape)), expected)
        strided = np.zeros((2 * n, *shape), dtype=dtype)[::2]
        assert strided.shape == (n, *shape) and not strided.flags["C_CONTIGUOUS"]
        refused(name, strided, "C-contiguous")
    # A wrong number of actions is refused the same way, by name.
    with pytest.raises(ValueError, match=r"actions.*\(3,\).*\(4,\)"):
        tables.step_into(actions[:3].copy(), *[good[name] for name in everything])

    # Nothing was written and nothing moved: the right arrays still hold the first observation.
    for name in observation:
        assert np.array_equal(good[name], reference[name]), name
    tables.observe_into(*[good[name] for name in observation])
    for name in observation:
        assert np.array_equal(good[name], reference[name]), name
    tables.step_into(actions, *[good[name] for name in everything])

    # The state arrays of set_state are checked too.
    state = dict(tables.get_state())
    names = ("run", "deal_in_run", "deals_done", "n_actions", "actions")
    tables.set_state(state["base_seed"], *[state[name] for name in names])
    swapped = dict(state)
    swapped["actions"] = np.ascontiguousarray(state["actions"].T)
    # 192 places for the actions of a deal since spec 009 (the round of announcements); 96 before.
    with pytest.raises(ValueError, match=r"actions.*\(192, 4\).*\(4, 192\)"):
        tables.set_state(state["base_seed"], *[swapped[name] for name in names])
    fortran_state = dict(state)
    fortran_state["actions"] = np.asfortranarray(state["actions"])
    with pytest.raises(ValueError, match="C-contiguous"):
        tables.set_state(state["base_seed"], *[fortran_state[name] for name in names])


def test_c20_a_second_thread_waits_for_a_running_call():
    """Review item B8. While a call runs with the GIL released the engine writes into the arrays.

    EngineEnv holds a lock around its methods: a second thread that calls observe,
    get_state or step waits for the running call and then sees whole rows.
    """
    env = engine_env.EngineEnv(2048, seed=20, threads=2)
    order = []

    def long_call():
        env.run_random(steps=1500, seed=1)
        order.append("run_random returned")

    worker = threading.Thread(target=long_call)
    worker.start()
    deadline = time.perf_counter() + 5.0
    while not env._lock.locked():  # the worker is inside the call
        assert time.perf_counter() < deadline, "the long call never started"
        time.sleep(0.0005)
    obs = env.observe()
    order.append("observe returned")
    whole = np.array_equal(obs["critic"][:, :N_STATE], obs["state"])
    state = env.get_state()
    worker.join()
    assert order == ["run_random returned", "observe returned"]
    assert whole, "the state and the critic's copy of it are from different steps"
    assert int(state["n_actions"].max()) > 0

    # The same from the other side: a step waits for a running call too.
    order.clear()
    worker = threading.Thread(target=long_call)
    worker.start()
    while not env._lock.locked():
        time.sleep(0.0005)
    mask = env.observe()["mask"]
    env.step(random_legal(mask, np.random.default_rng(0)))
    order.append("step returned")
    worker.join()
    assert order == ["run_random returned", "step returned"]
    assert env.illegal_actions == 0


# --------------------------------------------------------------------------
# Criterion 21: parity with a pure-Rust run
# --------------------------------------------------------------------------


def test_c21_parity_of_1000_deals_driven_from_python_with_a_pure_rust_run():
    """Python picks every action; Rust plays the same deals again alone with the same choices."""
    tables, base_seed = 16, 2106
    env = engine_env.EngineEnv(tables, base_seed, threads=4)
    rng = np.random.default_rng(21)
    choices = [[] for _ in range(tables)]  # the raw random numbers of the deal each table is in
    position = [(0, 0)] * tables  # (run, deal in run) of that deal
    compared = 0
    nonzero = 0
    while compared < 1000:
        obs = env.observe()
        mask = obs["mask"]
        raw = rng.integers(0, 2**31, size=tables)
        legal_counts = mask.sum(axis=1)
        picks = raw % legal_counts
        # The pick-th legal slot of each table, in slot order.
        order = np.cumsum(mask, axis=1) - 1
        actions = np.argmax(mask & (order == picks[:, None]), axis=1)
        for t in range(tables):
            choices[t].append(int(raw[t]))
        done, reward, parts = env.step(actions)
        entries = env.entries
        for t in np.flatnonzero(done):
            run, deal = position[t]
            seed = tarok_env.deal_seed(base_seed, int(t), run, deal)
            dealer = (tarok_env.first_dealer(int(t), run) + deal) % 4
            rust_entries, decisions, slots = tarok_env.play_deal(seed, dealer, choices[t])
            assert decisions == len(choices[t]) == len(slots)
            assert list(entries[t]) == rust_entries, f"table {t}, run {run}, deal {deal}"
            # The reward follows from the same entries.
            e = np.array(rust_entries, dtype=np.float64)
            expected = (e - (e.sum() - e) / 3.0) / C.REWARD_SCALE
            # Rewards are 32-bit numbers. With kontra (spec 009) a reward can be above 100, where
            # its last place is about 1e-5: the room grows with the largest reward of the deal
            # (D147, 1 a). For rewards up to 1 it is the 1e-6 it was.
            room = 1e-6 * max(1.0, float(np.abs(expected).max()))
            assert np.abs(reward[t] - expected).max() < room
            assert np.abs(parts[t].sum(axis=-1) - reward[t]).max() < room
            nonzero += int(any(rust_entries))
            compared += 1
            choices[t] = []
            deal += 1
            position[t] = (run + 1, 0) if deal == tarok_env.DEALS_PER_RUN else (run, deal)
        for t in range(tables):
            assert env.position(t)[:2] == position[t]
    assert compared >= 1000 and nonzero > 900
    # Every table went into its second or third run of 24 deals.
    assert min(run for run, _ in position) >= 2


def test_c21_a_record_made_in_python_replays_in_rust():
    seed, dealer = 77, 2
    entries, decisions, slots = tarok_env.play_deal(seed, dealer, list(range(100)))
    text = tarok_env.record_json(seed, dealer, slots)
    assert f'"seed":"{seed}"' in text and '"record_version":1' in text
    assert tarok_env.replay_record(text) == entries
    with pytest.raises(ValueError, match="decision 5"):
        broken = list(slots)
        broken[5] = 159
        tarok_env.record_json(seed, dealer, broken)


# --------------------------------------------------------------------------
# Criterion 22: the interface of the mock, for the mock and for the engine
# --------------------------------------------------------------------------


@BOTH
def test_c22_observation_keys_shapes_types_and_ranges(kind):
    n = 6
    env = make_env(kind, n, seed=22)
    rng = np.random.default_rng(22)
    for _ in range(150):
        obs = env.observe()
        assert set(obs) == {
            "seat", "deal", "tokens", "n_tokens", "state", "mask", "critic", "partner_target", "card_target",
        }
        assert obs["seat"].shape == (n,) and obs["seat"].dtype == np.int64
        assert obs["deal"].shape == (n,) and obs["deal"].dtype == np.int64
        assert obs["tokens"].shape == (n, C.MAX_TOKENS, 5) and obs["tokens"].dtype == np.int16
        assert obs["n_tokens"].shape == (n,) and obs["n_tokens"].dtype == np.int64
        assert obs["state"].shape == (n, N_STATE) and obs["state"].dtype == np.float32
        assert obs["mask"].shape == (n, C.N_ACTIONS) and obs["mask"].dtype == np.bool_
        assert obs["critic"].shape == (n, N_CRITIC) and obs["critic"].dtype == np.float32
        assert obs["partner_target"].shape == (n,) and obs["partner_target"].dtype == np.int64
        assert obs["card_target"].shape == (n, C.N_CARDS) and obs["card_target"].dtype == np.int64

        assert ((obs["seat"] >= 0) & (obs["seat"] < 4)).all()
        assert obs["mask"].any(axis=1).all(), "a table without a legal action"
        assert (obs["state"] >= 0).all() and (obs["state"] <= 1).all()
        assert (obs["critic"] >= 0).all() and (obs["critic"] <= 1).all()
        assert np.array_equal(obs["critic"][:, :N_STATE], obs["state"]), "the critic starts with the state"
        assert ((obs["partner_target"] >= 0) & (obs["partner_target"] < C.PARTNER_CLASSES)).all()
        assert ((obs["card_target"] >= 0) & (obs["card_target"] < C.CARD_LOCATION_CLASSES)).all()
        # Exactly one phase bit.
        first, width = C.PHASE_BLOCK
        assert (obs["state"][:, first : first + width].sum(axis=1) == 1).all()

        # Tokens: values inside each field's range, zero after the last token, the index field counts up.
        tokens, n_tokens = obs["tokens"], obs["n_tokens"]
        assert ((n_tokens >= 1) & (n_tokens <= C.MAX_TOKENS)).all()
        for field, size in enumerate(C.TOKEN_FIELD_SIZES):
            assert (tokens[:, :, field] >= 0).all() and (tokens[:, :, field] < size).all()
        for t in range(n):
            k = int(n_tokens[t])
            assert not tokens[t, k:].any(), "padding is not zero"
            assert np.array_equal(tokens[t, :k, 4], np.arange(k))
            assert tokens[t, 0, 0] == C.EVENT_GROUPS["BOS"][0] and tokens[t, 0, 1] == C.NO_SEAT
        env.step(random_legal(obs["mask"], rng))


@BOTH
def test_c22_step_gives_done_reward_and_parts_once_per_deal(kind):
    n = 6
    env = make_env(kind, n, seed=23)
    rng = np.random.default_rng(23)
    finished = 0
    # A random deal of the engine has about 88 decisions since spec 009 (57 before): 700 steps
    # for at least 30 deals at six tables, where 400 were enough.
    for _ in range(700):
        obs = env.observe()
        deal_before = obs["deal"].copy()
        done, reward, parts = env.step(random_legal(obs["mask"], rng))
        assert done.shape == (n,) and done.dtype == np.bool_
        assert reward.shape == (n, 4) and reward.dtype == np.float32
        assert parts.shape == (n, 4, C.N_SCORE_PARTS) and parts.dtype == np.float32
        # Zero where the deal goes on; the parts of a seat add up to its reward. The room
        # grows with the largest reward of each deal: a reward can be above 100 with kontra
        # (D147, 1 a). Per table, as in test_c21: a large reward at one table widens nothing
        # at another.
        assert not reward[~done].any() and not parts[~done].any()
        room = 1e-5 * np.maximum(1.0, np.abs(reward).max(axis=1))
        assert (np.abs(parts.sum(axis=-1) - reward).max(axis=1) < room).all()
        assert not parts[..., C.SCORE_PARTS.index("klop")].any()
        assert not parts[..., C.SCORE_PARTS.index("radelc")].any()
        # A finished table starts its next deal at once: the deal counter moves there and only there.
        deal_after = env.observe()["deal"]
        assert np.array_equal(deal_after, deal_before + done)
        finished += int(done.sum())
    assert finished >= 6 * 5


@BOTH
def test_c22_a_seats_token_sequence_only_grows_within_a_deal(kind):
    """The acting cache and the L2 pass rely on it: an earlier sequence is a prefix of a later one."""
    n = 5
    env = make_env(kind, n, seed=24)
    rng = np.random.default_rng(24)
    last = {}  # (table, seat, deal) -> tokens at that seat's previous decision
    grown, unchanged = 0, 0
    for _ in range(500):
        obs = env.observe()
        for t in range(n):
            key = (t, int(obs["seat"][t]), int(obs["deal"][t]))
            now = obs["tokens"][t, : int(obs["n_tokens"][t])].copy()
            if key in last:
                before = last[key]
                assert len(now) >= len(before), "a token sequence got shorter"
                assert np.array_equal(now[: len(before)], before), f"not a prefix for {key}"
                grown += len(now) > len(before)
                unchanged += len(now) == len(before)
            last[key] = now
        env.step(random_legal(obs["mask"], rng))
    assert grown > 500
    if kind == "engine":
        # The engine differs from the mock in one point: a hidden discard is no public
        # event, so the declarer decides again with the same tokens as one decision before.
        assert unchanged > 0
    else:
        assert unchanged == 0


@BOTH
def test_c22_an_illegal_action_is_an_error_and_not_a_silent_fix(kind):
    n = 4
    env = make_env(kind, n, seed=25)
    rng = np.random.default_rng(25)
    for _ in range(10):
        env.step(random_legal(env.observe()["mask"], rng))
    obs = env.observe()
    before = {name: array.copy() for name, array in obs.items()}
    actions = random_legal(obs["mask"], rng)
    actions[2] = int(np.flatnonzero(~obs["mask"][2])[0])
    with pytest.raises(ValueError, match="illegal"):
        env.step(actions)
    assert env.illegal_actions >= 1
    if kind == "engine":
        # Nothing moved, at any table; the same step with a legal action works.
        after = env.observe()
        for name in before:
            assert np.array_equal(after[name], before[name]), name
        actions[2] = int(np.flatnonzero(obs["mask"][2])[0])
        env.step(actions)


@BOTH
def test_c22_the_state_restores_the_environment_exactly(kind):
    """`get_state` and `set_state` (for --resume): a second environment goes on exactly like the first."""
    n = 8
    env = make_env(kind, n, seed=26)
    rng = np.random.default_rng(26)
    for _ in range(137):
        env.step(random_legal(env.observe()["mask"], rng))
    state = env.get_state()
    # A checkpoint can hold it: NumPy arrays and plain values, through the trainer's own conversion.
    for name, value in state.items():
        assert isinstance(value, (np.ndarray, int, float, str, bool, dict)), f"{name}: {type(value)}"
    plain = train._plain({k: torch.from_numpy(v) if isinstance(v, np.ndarray) else v for k, v in state.items()})
    back = {
        k: v.numpy() if isinstance(v, torch.Tensor) else v for k, v in train._unplain(plain).items()
    }

    other = make_env(kind, n, seed=26)
    other.set_state(back)
    for step in range(120):
        one, two = env.observe(), other.observe()
        for name in one:
            assert np.array_equal(one[name], two[name]), f"{name} at step {step}"
        actions = random_legal(one["mask"], rng)
        a, b = env.step(actions), other.step(actions)
        for x, y in zip(a, b):
            assert np.array_equal(x, y)


def test_c22_the_trainer_runs_one_update_on_the_engine(monkeypatch):
    """The learner of spec 007, unchanged, on the real environment (the mock's make() swapped out)."""
    made = []

    def engine_make(cfg_env, seed):
        made.append(engine_env.EngineEnv(cfg_env["tables"], seed, threads=2))
        return made[-1]

    monkeypatch.setattr(train.mock_env, "make", engine_make)
    for kind in ("mlp", "transformer"):
        trainer = train.Trainer(small(**{"run.network": kind, "ppo.seat_deals_per_update": 16}), CPU)
        assert isinstance(trainer.env, engine_env.EngineEnv)
        (stats,) = trainer.run(1)
        for name, value in stats.items():
            assert np.isfinite(value), f"{name} is {value}"
        assert stats["rollout/decisions"] > 0
        assert trainer.env.illegal_actions == 0
    assert len(made) == 2


def test_c22_a_resumed_trainer_on_the_engine_equals_an_unbroken_one(monkeypatch, tmp_path):
    monkeypatch.setattr(
        train.mock_env, "make", lambda cfg_env, seed: engine_env.EngineEnv(cfg_env["tables"], seed, threads=1)
    )
    cfg = small(**{"run.network": "mlp", "run.seed": 6, "ppo.seat_deals_per_update": 16})
    watched = ("loss/total", "loss/policy", "loss/critic", "entropy/mean", "rollout/decisions")

    def numbers(history):
        return [tuple(h[name] for name in watched) for h in history]

    unbroken = numbers(train.Trainer(cfg, CPU).run(4))
    first = train.Trainer(cfg, CPU)
    start = numbers(first.run(2))
    first.save(tmp_path / "two.pt")
    resumed = train.Trainer.load(tmp_path / "two.pt", CPU)
    assert start + numbers(resumed.run(2)) == unbroken


def test_c22_make_gives_the_mock_or_the_engine():
    cfg = cfg_with(**{"env.tables": 4})["env"]
    assert isinstance(engine_env.make(cfg, seed=1), mock_env.MockEnv)
    real = engine_env.make({"kind": "engine", "tables": 4, "threads": 1}, seed=1)
    assert isinstance(real, engine_env.EngineEnv) and real.n == 4
    with pytest.raises(ValueError, match="unknown environment"):
        engine_env.make({"kind": "other", "tables": 4}, seed=1)


def test_c22_step_refuses_actions_that_are_not_whole_numbers_or_not_one_per_table():
    """Review item D15: no silent rounding, and a wrong count is not called an illegal action."""
    n = 4
    env = engine_env.EngineEnv(n, seed=27, threads=1)
    obs = env.observe()
    before = {name: array.copy() for name, array in obs.items()}
    legal = random_legal(obs["mask"], np.random.default_rng(27))

    for wrong in (legal.astype(np.float64) + 0.9, legal.astype(np.float32), legal.astype(bool), legal.astype(str)):
        with pytest.raises(TypeError, match="whole numbers"):
            env.step(wrong)
    for wrong in (legal[:3], np.concatenate([legal, legal]), legal.reshape(2, 2), np.int64(legal[0])):
        with pytest.raises(ValueError, match="one action per table") as caught:
            env.step(wrong)
        assert "illegal" not in str(caught.value)
    # None of these counts as an illegal action, and nothing moved.
    assert env.illegal_actions == 0
    for name, array in env.observe().items():
        assert np.array_equal(array, before[name]), name
    # Whole numbers of any integer type and a plain list are fine.
    env.step(legal.astype(np.int32))
    env.step([int(np.flatnonzero(row)[0]) for row in env.observe()["mask"]])
    # An illegal slot is still reported as one.
    actions = random_legal(env.observe()["mask"], np.random.default_rng(1))
    actions[0] = 159
    with pytest.raises(ValueError, match="illegal action"):
        env.step(actions)
    assert env.illegal_actions == 1


@pytest.mark.parametrize("kind", ["transformer", "mlp"])
def test_c22_spec007_c06_acting_and_update_see_identical_inputs_on_the_engine(monkeypatch, kind):
    """The body of spec 007's test_c06_acting_and_update_see_byte_identical_actor_inputs, on the engine.

    The engine reuses its observation arrays between steps, so this is the test that
    the trainer copies what it keeps.
    """
    monkeypatch.setattr(
        train.mock_env, "make", lambda cfg_env, seed: engine_env.EngineEnv(cfg_env["tables"], seed, threads=2)
    )
    trainer = train.Trainer(small(**{"run.network": kind}), CPU)
    assert isinstance(trainer.env, engine_env.EngineEnv)
    given = {}
    counts = {}
    real_act = trainer.actor.act

    def recording(tables, seat, deal, n_tokens, frozen_rows, tokens, state, mask, generator):
        for i, table in enumerate(tables):
            key = (int(table), int(seat[i]), int(deal[i]))
            counts[key] = counts.get(key, 0) + 1
            n = int(n_tokens[i])
            given[(*key, counts[key] - 1)] = (n, tokens[i, :n].numpy().copy(), state[i].numpy().copy(), mask[i].numpy().copy())
        return real_act(tables, seat, deal, n_tokens, frozen_rows, tokens, state, mask, generator)

    trainer.actor.act = recording
    deals = collected(trainer)
    batch = deals.to_torch(CPU)
    checked = 0
    for g in range(len(deals)):
        for k in range(int(deals.valid[g].sum())):
            n, tokens, state, mask = given[(int(deals.table[g]), int(deals.seat[g]), int(deals.deal[g]), k)]
            assert int(batch.points[g, k]) == n
            assert np.array_equal(batch.tokens[g, :n].numpy(), tokens)  # the earlier sequence is a prefix
            assert batch.state[g, k].numpy().tobytes() == state.tobytes()
            assert np.array_equal(batch.mask[g, k].numpy(), mask)
            checked += 1
    assert checked >= 50
    assert trainer.env.illegal_actions == 0


@pytest.mark.parametrize("kind", ["transformer", "mlp"])
def test_c22_spec007_c16_a_fresh_rollout_has_probability_ratio_one_on_the_engine(monkeypatch, kind):
    """The body of spec 007's test_c16_a_fresh_rollout_has_probability_ratio_one_on_every_decision, on the engine."""
    monkeypatch.setattr(
        train.mock_env, "make", lambda cfg_env, seed: engine_env.EngineEnv(cfg_env["tables"], seed, threads=2)
    )
    cfg = small(**{"run.network": kind, "opponents.snapshot_every": 1, "opponents.past_share": 0.5})
    trainer = train.Trainer(cfg, CPU)
    assert isinstance(trainer.env, engine_env.EngineEnv)
    history = trainer.run(2)  # the weights have changed twice; a past checkpoint is in play
    assert trainer.frozen_active and len(trainer.pool) == 2
    before = trainer.frozen_decisions
    deals = collected(trainer)
    assert trainer.frozen_decisions > before, "no frozen seat was in play"
    batch = deals.to_torch(CPU)
    out, value = ppo.evaluate(trainer.net, trainer.critic, batch)
    logp = ppo.compute_losses(out, value, batch, cfg["ppo"])["logp"]
    assert batch.valid.sum() >= 80
    assert (logp - batch.logp[batch.valid]).abs().max().item() < 1e-5
    # The update measures the same thing itself, before its first step, at every update.
    assert all(h["ppo/behaviour_mismatch"] < 1e-5 for h in history)
    # And every decision it trains on is from the current weights.
    assert all(h["rollout/decision_age_max"] == 0 for h in history)
    assert np.all(deals.version[deals.valid] == trainer.updates)


# --------------------------------------------------------------------------
# Criterion 23: environment-only speed
# --------------------------------------------------------------------------


def test_c23_the_benchmark_runs_and_reports_steps_per_second():
    rows = engine_env.bench(tables=64, threads=(1, 2), seconds=0.2)
    assert [row["threads"] for row in rows] == [1, 2]
    for row in rows:
        assert row["tables"] == 64
        assert row["python_policy_steps_per_second"] > 0
        assert row["rust_policy_steps_per_second"] > 0
        assert row["deals"] > 0
