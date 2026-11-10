# 003 — Training environment and throughput benchmark

Status: **draft** (2026-11-10). Not approved. Needs spec 001 done. Independent of spec 002.

Build-plan row 003. Tool and model: Claude Code 2.1.306, `claude-fable-5-2`.

## Goal

Prove that this PC can train the planned network, and measure how fast, before the engine exists. Three questions get a measured answer:

1. Does a pinned PyTorch CUDA build run on the GTX 1660 SUPER here?
2. Does a Rust extension built with PyO3 and maturin load into Python with the Rust toolchain installed here (`x86_64-pc-windows-gnu`)?
3. How many decisions per second can the transformer (D18) act on and learn from, on synthetic data, for the two sequence layouts of B10-4? Same for the MLP control.

Found on 2026-11-10 (read-only): driver 610.88; 6144 MiB of which 2836 MiB in use by the desktop; global Python 3.12.10 has `torch 2.12.0+cpu`; maturin is not installed.

## In scope

- A virtual environment `.venv` at the repo root (already gitignored), Python 3.12.
- `src/train/requirements.txt` with exact versions: PyTorch (newest stable release that has a Windows CUDA 13.0 wheel for Python 3.12; the exact version and index URL written into the file), numpy, pytest, maturin, tensorboard.
- `src/train/tarok_train/envcheck.py`: prints versions, device name, compute capability, free VRAM; exits non-zero if CUDA is not usable.
- `src/train/pyo3_probe/`: a throwaway Rust crate with one function that releases the GIL, fills an array on 12 threads and returns it as a NumPy array. Built with maturin into the venv.
- `src/train/tarok_train/bench_model.py`: builds the networks and times them on random data shaped like a deal (up to about 70 play and bid tokens, about 15 state tokens of about 700 features, a 140-way masked policy head, a value head).
- A results table in `docs/progress.md`.

## Out of scope

The real encoding (spec 004), the real model and learner (spec 008), any engine code, any training run. The benchmark model is a stand-in; spec 008 may reuse parts of it.

## Cost

Download of the PyTorch CUDA wheel and its libraries: about 2.5–3.5 GB into `.venv` (size from memory of earlier releases; the real size is reported). Nothing is installed outside `.venv` except, if the user chooses fallback (a) below, the MSVC build tools.

## Acceptance criteria

The user approves these before anything is installed or written (D73).

1. `.venv` exists and is not tracked by git. `requirements.txt` pins every package with `==` and records the PyTorch index URL.
2. `envcheck` prints `cuda available: True`, the device name `GeForce GTX 1660 SUPER`, capability `(7, 5)`, and exits 0. Its output is pasted into `docs/progress.md`.
3. A tensor operation on the GPU gives the same result as on the CPU within 1e-5 (one matmul, checked by a pytest test).
4. The PyO3 probe builds with maturin on the installed toolchain, imports in the venv, and a pytest test checks the returned array's values and that 12 threads ran. **If the build fails on the GNU toolchain,** the spec stops there, the error goes into `docs/progress.md`, and the user chooses: (a) install the MSVC build tools and the MSVC Rust toolchain, or (b) file hand-off between Rust and Python instead of PyO3 (research/06 f). Criteria 1–3 and 5–8 still count.
5. The benchmark runs for the transformer (4 layers, 256 wide, 4 heads, feed-forward 1024) and the MLP control (512-512-256), each in fp32 and with half-precision autocast, at batch sizes that fit in the free VRAM.
6. For the transformer it measures both layouts: **(L1)** the state token appended last and the whole sequence recomputed for every decision; **(L2)** play and bid tokens computed once per seat and deal, state tokens attending to their prefix through a mask (training in one pass; acting with cached tokens).
7. For each network and layout it reports: acting decisions per second on the GPU; update samples per second (forward and backward); peak VRAM; and single-thread CPU forward time per decision (an early hint for the Rust forward pass and the phone goal G14).
8. From those numbers it derives an estimate of end-to-end steps per second with 4 PPO epochs and with 2, and states whether G3's 10,000 looks reachable for each layout. The estimate and its formula are in `docs/progress.md`. No pass or fail threshold: the numbers feed B10-4 and D22.
9. pytest passes locally. CI gains a job that runs the CPU-only tests of `src/train` (the GitHub runner has no GPU; GPU tests are marked and skipped there).

## Files expected to change

New: `src/train/requirements.txt`, `src/train/pyproject.toml`, `src/train/tarok_train/__init__.py`, `envcheck.py`, `bench_model.py`, `src/train/tests/test_env.py`, `src/train/pyo3_probe/` (Cargo.toml, src/lib.rs, pyproject.toml).
Changed: `.github/workflows/ci.yml`, `docs/progress.md`, this spec's status.

## Commits

(a) requirements, envcheck, GPU test; (b) PyO3 probe; (c) benchmark and the results in `docs/progress.md`. Each with `Spec: docs/specs/003-training-environment.md`.

## Commands the user can run afterwards (cmd syntax)

```bat
.venv\Scripts\activate
python -m tarok_train.envcheck
python -m tarok_train.bench_model
```

## Open points for the user

- **A. TensorBoard** for training curves (a general library, allowed by D27), or plain CSV files and plots made by a script?
- **B. Long benchmarks need the GPU free.** The desktop uses 2.8 GB of the 6 GB now. Run the benchmark with other GPU apps closed, or measure "as used daily"? Proposal: both, labelled.
