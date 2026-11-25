"""The real environment: the Rust engine behind the interface of the mock (spec 006).

    env = EngineEnv(n_tables=1024, seed=3, threads=12)
    obs = env.observe()                  # the acting seat of every table
    done, reward, parts = env.step(a)    # a finished table starts its next deal at once
    env.step(a, hold)                    # tables with hold[i] true do not move (spec 009)
    state = env.get_state()              # for a checkpoint; env.set_state(state) restores it

The arrays are those of docs/encoding.md, one row per table:

    seat (N,) int64            the seat to act, 0 to 3
    deal (N,) int64            deals the table has finished; changes when a new deal starts
    tokens (N, 128, 5) int16   event, relative seat, trick, position, index; zero after n_tokens
    n_tokens (N,) int64
    state (N, 1382) float32
    mask (N, 160) bool
    critic (N, 1656) float32   the state, then the 274 hidden features
    partner_target (N,) int64  relative seat of the one team-mate, 1 to 3; 0 = none
    card_target (N, 54) int64  1 to 3 = that other seat holds the card; 0 = any other place

`step` returns done (N,) bool, reward (N, 4) float32 and parts (N, 4, 10) float32 by
absolute seat; the last two are zero where the deal goes on.

The observation arrays belong to this object and are written in place by the engine:
the next `step` overwrites them. Copy what you keep (the trainer does). The three
arrays that `step` returns are fresh copies.

Threads. The engine releases the GIL while it observes, steps or runs random steps,
and writes into the observation arrays during that time. The rule: no other thread
may read, write or resize those arrays while a call is running. The methods of an
EngineEnv take a lock, so a second thread that calls a method waits for the running
call. The lock cannot protect an array a thread already holds: a thread that keeps
the dict from `observe()` and reads it while another thread steps sees rows of two
different steps. One thread should drive one environment.

Each table plays runs of 24 deals with the dealer moving one seat per deal. The
cards of a deal depend only on (seed, table, run, deal number).

Stale module (D130). The installed module `tarok_env` carries a fingerprint of the
Rust sources it was built from: `SOURCE_MANIFEST` (one line per file: path, size,
CRC-32) and `SOURCE_HASH` (a hash of that text). An EngineEnv compares them with
the files on disk and refuses to start if they differ, so Python never runs old
rules without a sign. Build again with `maturin develop --release` in
src/train/tarok_env. Where the sources are not on disk there is nothing to compare.

Benchmark of the environment alone (criterion 23), from src/train with the venv active:

    python -m tarok_train.env --tables 1024 --threads 1 12 --seconds 5
"""

from __future__ import annotations

import argparse
import json
import threading
import time
import zlib
from pathlib import Path

import numpy as np

import tarok_env

from . import contract as C
from . import mock_env

# The observation arrays in the order the engine takes them: name, shape after N, type.
OBSERVATION_ARRAYS = {
    "seat": ((), np.int64),
    "deal": ((), np.int64),
    "tokens": ((C.MAX_TOKENS, len(C.TOKEN_FIELDS)), np.int16),
    "n_tokens": ((), np.int64),
    "state": ((C.STATE_FEATURES,), np.float32),
    "mask": ((C.N_ACTIONS,), np.bool_),
    "critic": ((C.CRITIC_FEATURES,), np.float32),
    "partner_target": ((), np.int64),
    "card_target": ((C.N_CARDS,), np.int64),
}
# The arrays of a step's result, in the engine's order.
RESULT_ARRAYS = {
    "done": ((), np.bool_),
    "reward": ((4,), np.float32),
    "parts": ((4, C.N_SCORE_PARTS), np.float32),
    "entries": ((4,), np.int32),
}
# The arrays that make up the environment's state (for checkpoints).
STATE_ARRAYS = ("run", "deal_in_run", "deals_done", "n_actions", "actions")


# ---- the stale-module check (D130) ----

# The repo's `src` folder. The fingerprint covers the engine crate and the binding crate under it.
SOURCE_ROOT = Path(__file__).resolve().parents[2]
# crate folder under SOURCE_ROOT -> its files outside `src` that count. Every .rs file under `src` counts.
# The same list is in src/train/tarok_env/build.rs.
SOURCE_CRATES = {
    "engine": ("Cargo.toml",),
    "train/tarok_env": ("Cargo.toml", "Cargo.lock", "build.rs"),
}


class StaleModuleError(RuntimeError):
    """The installed `tarok_env` module was built from other sources than those on disk."""


def source_manifest(root) -> str | None:
    """The fingerprint of the Rust sources under `root`, as the build script writes it.

    One line per file, sorted by path: path relative to `root` with forward slashes,
    a tab, the size in bytes, a tab, the CRC-32 of the bytes as eight hex digits.
    None if the engine's sources are not there.
    """
    root = Path(root)
    if not (root / "engine" / "src").is_dir():
        return None
    files = {}
    for crate, names in SOURCE_CRATES.items():
        base = root / crate
        for name in names:
            if (base / name).is_file():
                files[f"{crate}/{name}"] = base / name
        for path in (base / "src").rglob("*.rs"):
            files[f"{crate}/{path.relative_to(base).as_posix()}"] = path
    lines = []
    for name in sorted(files):
        data = files[name].read_bytes()
        lines.append(f"{name}\t{len(data)}\t{zlib.crc32(data):08x}\n")
    return "".join(lines)


def manifest_hash(manifest: str) -> str:
    """FNV-1a, 64 bits, of the manifest text, as sixteen hex digits (the module's SOURCE_HASH)."""
    value = 0xCBF29CE484222325
    for byte in manifest.encode("utf-8"):
        value = ((value ^ byte) * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return f"{value:016x}"


def check_module_is_current() -> None:
    """Raise StaleModuleError if the installed module was built from other sources than those on disk."""
    on_disk = source_manifest(SOURCE_ROOT)
    if on_disk is None:  # an installed wheel without the repo: nothing to compare
        return
    if not hasattr(tarok_env, "SOURCE_HASH"):  # a module built before the fingerprint existed
        raise StaleModuleError(
            "the installed tarok_env module carries no fingerprint of its sources: it was built before the "
            "stale-module check existed. Build it again: maturin develop --release in src/train/tarok_env"
        )
    if manifest_hash(on_disk) == tarok_env.SOURCE_HASH:
        return
    built = dict(line.split("\t", 1) for line in tarok_env.SOURCE_MANIFEST.splitlines())
    now = dict(line.split("\t", 1) for line in on_disk.splitlines())
    lists = {
        "changed": sorted(name for name in now if name in built and now[name] != built[name]),
        "new": sorted(name for name in now if name not in built),
        "no longer there": sorted(name for name in built if name not in now),
    }
    parts = [f"{label}: {', '.join(names)}" for label, names in lists.items() if names]
    raise StaleModuleError(
        "the installed tarok_env module was built from other sources than those on disk "
        f"({'; '.join(parts)}). Build it again: maturin develop --release in src/train/tarok_env"
    )


def new_buffers(n_tables: int) -> dict:
    """Zeroed arrays of the documented shapes and types for `n_tables` tables."""
    return {
        name: np.zeros((n_tables, *shape), dtype=dtype)
        for name, (shape, dtype) in {**OBSERVATION_ARRAYS, **RESULT_ARRAYS}.items()
    }


def _check_contract() -> None:
    """The engine and tarok_train/contract.py must describe the same encoding."""
    pairs = {
        "ENCODING_VERSION": C.ENCODING_VERSION,
        "N_EVENTS": C.N_EVENTS,
        "STATE_FEATURES": C.STATE_FEATURES,
        "N_ACTIONS": C.N_ACTIONS,
        "MAX_TOKENS": C.MAX_TOKENS,
        "CRITIC_FEATURES": C.CRITIC_FEATURES,
        "N_SCORE_PARTS": C.N_SCORE_PARTS,
    }
    for name, expected in pairs.items():
        found = getattr(tarok_env, name)
        if found != expected:
            raise RuntimeError(
                f"the engine was built with {name} = {found}, tarok_train/contract.py has {expected}: "
                "rebuild tarok_env (maturin develop --release) or update the contract"
            )


class EngineEnv:
    zero_sum = True  # the four rewards of a deal add up to zero; the learner's storage checks it
    real_deals = True  # the state features are those of real deals: the deal monitors apply
    reward_is_score = False  # the mean reward is zero by construction, not a score to log

    def __init__(self, n_tables: int, seed: int, threads: int):
        _check_contract()
        check_module_is_current()
        self.source_hash = tarok_env.SOURCE_HASH  # the fingerprint of the sources the module was built from
        # The version of the engine's rules. A checkpoint holds it (spec 009, criterion 53).
        self.engine_version = tarok_env.ENGINE_VERSION
        self.n = n_tables
        self.seed = seed
        self.threads = threads
        self.illegal_actions = 0
        # One call into the engine at a time: see "Threads" at the top of this file.
        self._lock = threading.Lock()
        self._tables = tarok_env.Tables(n_tables, seed, threads)
        self._buffers = new_buffers(n_tables)
        self._obs = {name: self._buffers[name] for name in OBSERVATION_ARRAYS}
        self._observation_arrays = [self._buffers[name] for name in OBSERVATION_ARRAYS]
        self._all_arrays = [self._buffers[name] for name in (*OBSERVATION_ARRAYS, *RESULT_ARRAYS)]
        self._tables.observe_into(*self._observation_arrays)

    def observe(self) -> dict:
        """The arrays of the acting seat of every table. The next step overwrites them."""
        with self._lock:
            return self._obs

    def step(self, actions: np.ndarray, hold=None):
        """Apply one action (a slot number) per table.

        Returns done (N,) bool, reward (N, 4) and parts (N, 4, parts) by absolute seat.
        An illegal action is an error: a ValueError is raised and no table has moved.
        Actions must be whole numbers (any integer type), one per table: a float array
        is a TypeError, another count a ValueError. Neither counts as an illegal action.

        `hold` (spec 009, criterion 52): one bool per table, or None for "no table is
        held". A held table does not move: its action is ignored and need not be legal,
        it is not done and gets no reward, and its rows of the observation stay as they
        were. Holding never changes what a table plays. A `hold` that is not bool is a
        TypeError, another count a ValueError.
        """
        actions = np.asarray(actions)
        if not np.issubdtype(actions.dtype, np.integer):
            raise TypeError(f"actions must be whole numbers (slot numbers), not {actions.dtype}")
        if actions.shape != (self.n,):
            raise ValueError(
                f"step needs one action per table: shape ({self.n},), not {actions.shape}"
            )
        actions = np.ascontiguousarray(actions, dtype=np.int64)
        if hold is not None:
            hold = np.asarray(hold)
            if hold.dtype != np.bool_:
                raise TypeError(f"hold must be bool, one flag per table, not {hold.dtype}")
            if hold.shape != (self.n,):
                raise ValueError(f"hold needs one flag per table: shape ({self.n},), not {hold.shape}")
            hold = np.ascontiguousarray(hold)
        with self._lock:
            try:
                self._tables.step_into(actions, *self._all_arrays, hold)
            except ValueError as error:
                self.illegal_actions += 1
                raise ValueError(f"an illegal action reached the environment: {error}") from error
            buffers = self._buffers
            return buffers["done"].copy(), buffers["reward"].copy(), buffers["parts"].copy()

    @property
    def entries(self) -> np.ndarray:
        """(N, 4) int32: the paper entries of the deals that ended with the last step, zero elsewhere."""
        with self._lock:
            return self._buffers["entries"].copy()

    def position(self, table: int) -> tuple:
        """(run, deal number in the run, deals finished) of a table."""
        with self._lock:
            return self._tables.position(table)

    def run_random(self, steps: int, seed: int) -> tuple:
        """`steps` steps with random legal actions chosen inside the engine. Returns (decisions, deals).

        One call into Rust: Ctrl+C does not stop it before it returns. Keep `steps` small
        enough (the benchmark uses 50 per call).
        """
        with self._lock:
            return self._tables.run_random(steps, seed, *self._all_arrays)

    # ---- checkpoints ----

    def get_state(self) -> dict:
        """Everything needed to go on from here: where each table stands and the actions of its deal.

        NumPy arrays and plain values, so a checkpoint can hold it. The deals are dealt
        again from their seeds and the actions replayed by `set_state`.
        """
        with self._lock:
            state = dict(self._tables.get_state())
            state["kind"] = "engine"
            state["encoding_version"] = C.ENCODING_VERSION
            state["illegal_actions"] = self.illegal_actions
            return state

    def set_state(self, state: dict) -> None:
        """Restore a state from `get_state` and rebuild the pending observation."""
        if state.get("kind") != "engine":
            raise ValueError("this state was not saved by the engine environment")
        if state["encoding_version"] != C.ENCODING_VERSION:
            raise ValueError(
                f"the state was saved with encoding version {state['encoding_version']}, "
                f"this code reads {C.ENCODING_VERSION}"
            )
        arrays = [np.ascontiguousarray(state[name], dtype=np.int64) for name in STATE_ARRAYS[:-1]]
        actions = np.ascontiguousarray(state["actions"], dtype=np.uint8)
        with self._lock:
            self._tables.set_state(int(state["base_seed"]), *arrays, actions)
            self.illegal_actions = int(state["illegal_actions"])
            for name in RESULT_ARRAYS:
                self._buffers[name][...] = 0
            self._tables.observe_into(*self._observation_arrays)


def make(cfg_env: dict, seed: int):
    """The environment a config asks for: kind "mock" (spec 007) or kind "engine".

    An engine config has the keys kind, tables and threads.
    """
    kind = cfg_env["kind"]
    if kind == "mock":
        return mock_env.make(cfg_env, seed)
    if kind == "engine":
        return EngineEnv(cfg_env["tables"], seed, cfg_env["threads"])
    raise ValueError(f"unknown environment '{kind}': use 'mock' or 'engine'")


# ---- benchmark (spec 006, criterion 23) ----


def _random_legal(mask: np.ndarray, rng: np.random.Generator) -> np.ndarray:
    """One legal slot per table, uniformly: the largest of 160 random numbers among the legal slots."""
    scores = rng.random(mask.shape, dtype=np.float32)
    scores[~mask] = -1.0
    return scores.argmax(axis=1)


def bench(tables: int, threads, seconds: float, seed: int = 0) -> list:
    """Steps per second of the environment alone, with a random policy, for each thread count.

    A step is one decision at one table. Two numbers per thread count:
    - python_policy: observe, a random legal action chosen with NumPy, step. What a
      learner that costs nothing would see.
    - rust_policy: the random actions are chosen inside the engine, in one call. The
      engine and its arrays alone.
    """
    rows = []
    for count in threads:
        env = EngineEnv(tables, seed, count)
        rng = np.random.default_rng(seed)
        env.run_random(200, seed)  # past the first bidding rounds

        steps, deals = 0, 0
        started = time.perf_counter()
        while time.perf_counter() - started < seconds:
            done, _, _ = env.step(_random_legal(env.observe()["mask"], rng))
            steps += 1
            deals += int(done.sum())
        python_seconds = time.perf_counter() - started

        chunk, rust_steps, rust_deals = 50, 0, 0
        started = time.perf_counter()
        while time.perf_counter() - started < seconds:
            _, finished = env.run_random(chunk, seed + rust_steps + 1)
            rust_steps += chunk
            rust_deals += finished
        rust_seconds = time.perf_counter() - started

        rows.append(
            {
                "tables": tables,
                "threads": count,
                "python_policy_steps_per_second": steps * tables / python_seconds,
                "rust_policy_steps_per_second": rust_steps * tables / rust_seconds,
                "python_policy_deals_per_second": deals / python_seconds,
                "rust_policy_deals_per_second": rust_deals / rust_seconds,
                "deals": deals + rust_deals,
                "seconds_each": seconds,
            }
        )
    return rows


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description="Environment-only speed with a random policy (spec 006).")
    parser.add_argument("--tables", type=int, default=1024)
    parser.add_argument("--threads", type=int, nargs="+", default=[1, 12])
    parser.add_argument("--seconds", type=float, default=5.0, help="measuring time per number")
    parser.add_argument("--repeats", type=int, default=3, help="repeats; the median is printed")
    parser.add_argument("--json", action="store_true", help="print the raw rows as JSON")
    args = parser.parse_args(argv)

    runs = [bench(args.tables, args.threads, args.seconds, seed=repeat) for repeat in range(args.repeats)]
    if args.json:
        print(json.dumps(runs, indent=1))
        return 0
    print(f"engine {tarok_env.ENGINE_VERSION}, {args.tables} tables, {args.seconds} s per number, {args.repeats} repeats")
    print("a step is one decision at one table; median (min to max)")
    for index, count in enumerate(args.threads):
        for key, label in (
            ("python_policy_steps_per_second", "random policy in Python"),
            ("rust_policy_steps_per_second", "random policy in the engine"),
        ):
            values = sorted(run[index][key] for run in runs)
            median = values[len(values) // 2]
            print(f"{count:>3} threads, {label:<28} {median:>12,.0f} steps/s ({values[0]:,.0f} to {values[-1]:,.0f})")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
