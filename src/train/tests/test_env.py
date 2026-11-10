"""Tests for spec 003, acceptance criteria 2, 3 and 4.

Written from the criteria before the code they test.
"""

import subprocess
import sys
import threading
import time
from pathlib import Path

import numpy as np
import pytest
import torch

TRAIN_DIR = Path(__file__).resolve().parents[1]


# --- Criterion 2: envcheck -------------------------------------------------


@pytest.mark.gpu
def test_envcheck_reports_this_gpu_and_exits_zero():
    """`python -m tarok_train.envcheck` prints the three required lines and exits 0."""
    done = subprocess.run(
        [sys.executable, "-m", "tarok_train.envcheck"],
        cwd=TRAIN_DIR,
        capture_output=True,
        text=True,
        timeout=120,
    )
    out = done.stdout
    assert done.returncode == 0, out + done.stderr
    assert "cuda available: True" in out
    assert "GeForce GTX 1660 SUPER" in out
    assert "capability: (7, 5)" in out
    for label in ("python:", "torch:", "numpy:", "vram free:"):
        assert label in out


def test_envcheck_exits_nonzero_without_cuda(monkeypatch, capsys):
    """With CUDA not usable, envcheck says so and returns a non-zero exit code."""
    from tarok_train import envcheck

    monkeypatch.setattr(torch.cuda, "is_available", lambda: False)
    code = envcheck.main()
    out = capsys.readouterr().out
    assert code != 0
    assert "cuda available: False" in out


# --- Criterion 3: GPU result equals CPU result ------------------------------


@pytest.mark.gpu
def test_matmul_on_gpu_matches_cpu_within_1e_5():
    gen = torch.Generator().manual_seed(3)
    a = torch.rand(64, 48, generator=gen)
    b = torch.rand(48, 32, generator=gen)
    on_cpu = a @ b
    on_gpu = (a.cuda() @ b.cuda()).cpu()
    assert on_gpu.shape == on_cpu.shape
    assert torch.max(torch.abs(on_gpu - on_cpu)).item() < 1e-5


# --- Criterion 4: PyO3 probe ------------------------------------------------

# Fixed CPU work for the timing tests: 1.2 million elements, 400 generator steps
# each. The total is the same whatever the thread count.
WORK_N = 1_200_000
WORK_ROUNDS = 400


@pytest.mark.pyo3
def test_pyo3_probe_values_and_twelve_threads():
    """The Rust function fills the array on 12 threads; every value is checked."""
    probe = pytest.importorskip("tarok_pyo3_probe", reason="PyO3 probe is not built")
    n = 120_007  # not a multiple of 12: the last chunks are shorter
    values, worker, os_threads = probe.fill_parallel(n, 12, 0)

    assert isinstance(values, np.ndarray) and values.dtype == np.float32
    assert values.shape == (n,)
    idx = np.arange(n, dtype=np.int64)
    expected = ((idx * 7 + 3) % 1000).astype(np.float32)
    np.testing.assert_array_equal(values, expected)

    assert isinstance(worker, np.ndarray) and worker.shape == (n,)
    assert sorted(set(worker.tolist())) == list(range(12))  # each of the 12 wrote a part
    assert os_threads == 12  # 12 distinct OS thread ids


def _wall_time(probe, n_threads: int, tries: int = 3) -> float:
    """Best wall time of the fixed work on n_threads threads."""
    best = float("inf")
    for _ in range(tries):
        start = time.perf_counter()
        probe.fill_parallel(WORK_N, n_threads, WORK_ROUNDS)
        best = min(best, time.perf_counter() - start)
    return best


@pytest.mark.pyo3
def test_pyo3_probe_threads_run_in_parallel():
    """The same CPU work on 1 thread and on 12 threads: 12 must be clearly faster.

    Distinct thread ids alone would also pass if the threads ran one after another.
    This PC has 6 cores and 12 hardware threads; the test asks for a factor of 2 only.
    """
    probe = pytest.importorskip("tarok_pyo3_probe", reason="PyO3 probe is not built")
    one = _wall_time(probe, 1)
    twelve = _wall_time(probe, 12)
    assert one > 0.05, f"the work is too short to time: {one:.3f} s"
    assert twelve < one / 2, f"1 thread {one:.3f} s, 12 threads {twelve:.3f} s: no clear speed-up"


@pytest.mark.pyo3
def test_pyo3_probe_releases_the_gil():
    """A Python thread keeps running while the Rust function works.

    The Rust side uses 2 threads here, so cores stay free for the Python thread.
    With the GIL held the counter could not move at all during the call.
    """
    probe = pytest.importorskip("tarok_pyo3_probe", reason="PyO3 probe is not built")
    ticks = 0
    stop = threading.Event()

    def count():
        nonlocal ticks
        while not stop.is_set():
            ticks += 1
            time.sleep(0.001)

    t = threading.Thread(target=count)
    t.start()
    try:
        time.sleep(0.05)
        before = ticks
        start = time.perf_counter()
        probe.fill_parallel(WORK_N, 2, WORK_ROUNDS)
        took = time.perf_counter() - start
        during = ticks - before
    finally:
        stop.set()
        t.join()
    assert took > 0.05, f"the call is too short to observe: {took:.3f} s"
    assert during >= 5, f"only {during} ticks in {took:.3f} s of Rust work: the GIL was not released"
