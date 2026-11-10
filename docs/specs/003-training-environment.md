# 003 — Training environment and throughput benchmark

Status: **approved** 2026-11-10 (D112); executed; diff read by the user (D114); committed in `e6af3d0`; CI green on `56f4caf`. **Not done yet: the GPU-free benchmark run is missing** (open point B). See "Results" at the end.

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
9. pytest passes locally. A workflow file `.github/workflows/train.yml` runs the CPU-only tests of `src/train` (the GitHub runner has no GPU; GPU tests are marked and skipped there).

## Files expected to change

New: `src/train/requirements.txt`, `src/train/pyproject.toml`, `src/train/tarok_train/__init__.py`, `envcheck.py`, `bench_model.py`, `src/train/tests/test_env.py`, `src/train/pyo3_probe/` (Cargo.toml, src/lib.rs, pyproject.toml).
New: `.github/workflows/train.yml`. Changed: this spec (status and a "Results" section). `docs/progress.md` is updated by the main session from the Results section (D109).

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

## Results

Executed on 2026-11-10 by a parallel session (D109). Nothing is committed. Rewritten the same day after the review pass (D72 step 4); see "Review fixes" at the end.

The criteria are met locally, except three things that are not this session's to finish: the copy of the numbers into `docs/progress.md` (criteria 2 and 8), the CI run (criterion 9), and the benchmark run with the GPU free (open point B). Every benchmark number below was measured while a game used the GPU.

Paths in this section are shortened: `<repo>` is the repository root, `<rustup>` is the rustup folder in the user profile.

### Criteria

| # | Result | Evidence |
|---|--------|----------|
| 1 | met | `.venv` exists; `git check-ignore -v .venv` gives `.gitignore:11:.venv/`. `src/train/requirements.txt` pins 26 packages with `==` and names the index URL. |
| 2 | met locally; copy pending | `envcheck` output below; exit code 0. The copy into `docs/progress.md` is for the main session (D109). |
| 3 | met | `test_matmul_on_gpu_matches_cpu_within_1e_5` passes. |
| 4 | met, with one setting | The probe builds with maturin on `stable-x86_64-pc-windows-gnu` and the three probe tests pass. It needs `PYO3_USE_RAW_DYLIB=0`. See "PyO3 probe". No MSVC was installed. |
| 5 | met | Transformer and MLP, fp32 and half-precision autocast, several batch sizes each. |
| 6 | met | L1 and L2 both measured. Cached acting is measured in two variants. |
| 7 | met | Tables below. |
| 8 | met locally; copy pending | "End-to-end estimate" below. The copy into `docs/progress.md` is for the main session. |
| 9 | met locally; CI pending | 23 tests pass. `.github/workflows/train.yml` is written. It has not run: nothing is pushed. |

### Environment

PyTorch: the index `https://download.pytorch.org/whl/cu130` was read on 2026-11-10. It lists Windows cp312 wheels for 2.9.0 to 2.14.1. `torch 2.14.1+cu130` is the newest. The Windows wheel carries the CUDA libraries; there are no separate `nvidia-*` packages.

| Package | Version |
|---------|---------|
| python | 3.12.10 |
| torch | 2.14.1+cu130 (CUDA 13.0, cuDNN 9.24.0) |
| numpy | 2.5.4 |
| pytest | 9.2.0 |
| maturin | 1.16.0 |
| tensorboard | 2.21.0 |
| Rust crates in the probe | pyo3 0.30.1, numpy 0.30.0 |

Download: 2,045 MB in total, of which the torch wheel is 1,990.6 MB (from the pip log). `.venv` takes 3.2 GB on disk. The spec estimated 2.5 to 3.5 GB. Nothing was installed outside `.venv`. The global Python packages were not changed.

`envcheck` output (`python -m tarok_train.envcheck`, exit code 0):

```
python: 3.12.10 (<repo>\.venv\Scripts\python.exe)
platform: Windows-11-10.0.26200-SP0
torch: 2.14.1+cu130
torch cuda build: 13.0
numpy: 2.5.4
cuda available: True
device: NVIDIA GeForce GTX 1660 SUPER
capability: (7, 5)
cudnn: 92400
vram total: 6143 MiB
vram free: 5138 MiB
gpu matmul: ok
```

### PyO3 probe

Outcome: **it works on the GNU toolchain.** rustc 1.94.1 `stable-x86_64-pc-windows-gnu`, maturin 1.16.0, pyo3 0.30.1. The function `fill_parallel(n, n_threads, rounds)` releases the GIL, fills the array on 12 OS threads and returns NumPy arrays. Every element also costs a fixed amount of CPU work, so the tests can time it.

The tests check:

- every returned value, and that each of the 12 threads wrote a part;
- that the threads run in parallel: the same CPU work takes 0.577 s on 1 thread and 0.113 s on 12 threads here, a factor of 5.1 (the test asks for a factor of 2);
- that the GIL is released: a Python thread keeps counting while Rust works on 2 threads.

The default build failed twice. Both errors, as printed (paths shortened):

```
error: error calling dlltool 'dlltool.exe': program not found
error: could not compile `pyo3-ffi` (lib) due to 1 previous error
```

Then, with the toolchain's own `dlltool.exe` put on `PATH`:

```
error: dlltool could not create import library with <rustup>\toolchains\stable-x86_64-pc-windows-gnu\lib\rustlib\x86_64-pc-windows-gnu\bin\self-contained\dlltool.exe -d ...\python312.dll_imports.def -D python312.dll -l ...\python312.dll_imports.lib -m i386:x86-64 -f --64 --no-leading-underscore --temp-prefix ...\python312.dll:
       ...\self-contained\dlltool.exe: CreateProcess
error: could not compile `pyo3-ffi` (lib) due to 1 previous error
```

Cause (read in `pyo3-ffi-0.30.1/build.rs`): PyO3 0.30 links `python312.dll` with rustc's raw-dylib. On the GNU target rustc calls `dlltool`, and dlltool calls an assembler `as.exe`. No `as.exe` exists on this PC (searched `PATH` and the rustup folder). That dlltool fails for this reason is inferred from the `CreateProcess` message.

Fix: the environment variable `PYO3_USE_RAW_DYLIB=0`, which PyO3's build script documents. PyO3 then links `python312.lib` from the Python install. It is set in `src/train/pyo3_probe/.cargo/config.toml`.

**Note for spec 006.** cargo reads `.cargo/config.toml` from the folder it is run in and its parents, not from the crate's folder. `maturin develop --release` works when run from `src\train\pyo3_probe`. The real binding crate needs its own `.cargo/config.toml`, or one at the repo root. Fallbacks (a) and (b) of criterion 4 are not needed.

The probe was rebuilt after the main session disabled the root `rust-toolchain.toml` (it had selected the MSVC host). cargo in the repo is `stable-x86_64-pc-windows-gnu`; the build and the three probe tests pass.

### Benchmark

Command: `python -m tarok_train.bench_model --label "..."`. Raw rows of all runs are in `src/train/bench_results/` (numbers only; no process names, no paths). Eager mode, no `torch.compile`.

Method: every row is timed in 3 repeats after 2 warm-up calls. A repeat lasts at least 1 second and at least 3 calls. CUDA is synchronised around every call. The tables give the median of the 3 repeats with the minimum and maximum. Each row uses the fastest batch size of its sweep.

**Label: as used daily, a game running.** A game was open during every run and kept the GPU 75 to 97% busy. The absolute numbers measure a loaded GPU, not this card. The GPU-free run is pending.

| Run | Method | GPU before the run (nvidia-smi) | Wall time | Note |
|-----|--------|--------------------------------|-----------|------|
| 1, 19:10 | one 1-second sample per row | 97% busy, 5,179 of 6,144 MiB used | 333 s | A second GPU process, not started by this session, ended during the run. The load changed between rows. |
| 2, 19:16 | one 1-second sample per row | 75% busy, 1,582 MiB used | 308 s | Steady load. |
| 3, 19:43 | 3 repeats, wider sweeps | 76% busy, 1,936 MiB used | 426 s | **The tables below are run 3.** |
| 4, 19:50 | 3 repeats, fp32 only | 78% busy, 1,937 MiB used | 209 s | State width 1,382, 160 actions, 288 token ids (`docs/encoding.md`, draft). |

The stand-in transformer has 3,494,937 parameters (4 layers, width 256, 4 heads, feed-forward 1024; the layers alone are 3.16 M). The MLP has 825,113. Heads: 140-way masked policy, 140-way Q, value.

What is timed:

- **L1 padded:** one sequence per decision, prefix lengths spread evenly over 5 to 70, the batch padded to its longest prefix plus the state token. Nothing is cached.
- **L1 bucketed:** the same, with the batch sorted by length and cut into 4 slices, each padded to its own longest. About 47 padded tokens per decision. This is the fair version of L1.
- **L2 update:** one sequence per seat and deal, 70 play tokens and 15 state tokens, all 15 decisions in one pass through the mask. 85 tokens for 15 decisions.
- **Cached acting, idealised:** keys and values of the play tokens are cached; each step computes about 5 new play tokens and one state token. Every deal in the batch is at the same decision. The action stays on the GPU.
- **Cached acting, ragged:** the same cache, with deals at different decisions in one batch, one mask per deal, and the action copied to the host at every step. This is what an actor that feeds an engine does.

Cached acting is not a property of L2. L1 here is causal, so a cached L1 actor would run the same computation. The acting rows compare cached with uncached. **The layouts differ in the update.**

`tests/test_bench.py` checks that L1, L2 in one pass, cached acting, and ragged cached acting give the same outputs for the same decision (within 1e-4).

All numbers in the next three tables are **measured** (run 3). Peak VRAM is the peak allocated by PyTorch tensors; the CUDA context comes on top. For MLP and L1 acting the actions are copied to the host too.

Acting, decisions per second:

| Network | Row | Precision | Batch | Median (min to max) | Peak VRAM (MiB) |
|---|---|---|---|---|---|
| MLP 512-512-256 | | fp32 | 65,536 | 358,828 (344,798 to 377,640) | 453 |
| MLP 512-512-256 | | fp16 autocast | 65,536 | 132,175 (130,338 to 153,537) | 350 |
| transformer | L1 padded, uncached | fp32 | 512 | 1,223 (1,205 to 1,223) | 530 |
| transformer | L1 padded, uncached | fp16 autocast | 512 | 268 (254 to 276) | 320 |
| transformer | L1 bucketed, uncached | fp32 | 2,048 | 1,593 (1,590 to 1,821) | 536 |
| transformer | L1 bucketed, uncached | fp16 autocast | 512 | 390 (360 to 397) | 104 |
| transformer | cached, idealised | fp32 | 256 deals | 10,281 (9,800 to 10,500) | 207 |
| transformer | cached, idealised | fp16 autocast | 1,024 deals | 2,766 (2,555 to 2,771) | 418 |
| transformer | cached, ragged, host copy | fp32 | 4,096 deals | 7,935 (7,336 to 8,040) | 2,885 |
| transformer | cached, ragged, host copy | fp16 autocast | 4,096 deals | 2,395 (2,219 to 2,476) | 1,589 |

Update (forward, loss, backward, clipping, optimizer step), samples per second:

| Network | Layout | Precision | Batch | Median (min to max) | Peak VRAM (MiB) |
|---|---|---|---|---|---|
| MLP 512-512-256 | | fp32 | 16,384 | 142,829 (141,233 to 150,831) | 229 |
| MLP 512-512-256 | | fp16 autocast | 16,384 | 46,382 (46,194 to 50,971) | 198 |
| transformer | L1 padded | fp32 | 64 | 416 (372 to 424) | 378 |
| transformer | L1 padded | fp16 autocast | 128 | 75 (75 to 76) | 431 |
| transformer | L1 bucketed | fp32 | 256 | 592 (566 to 601) | 853 |
| transformer | L1 bucketed | fp16 autocast | 128 | 123 (118 to 123) | 297 |
| transformer | L2 | fp32 | 256 deals | 4,909 (4,708 to 5,054) | 1,608 |
| transformer | L2 | fp16 autocast | 64 deals | 1,131 (1,060 to 1,182) | 292 |

The fp32 update sweep, median samples per second (run 3). The sweeps now reach their plateau:

| Layout | Batch 1 | Batch 2 | Batch 3 | Batch 4 |
|---|---|---|---|---|
| L1 padded (decisions) | 64: 416 | 128: 353 | 256: 352 | 512: 367 |
| L1 bucketed (decisions) | 64: 429 | 128: 498 | 256: 592 | 512: 512 |
| L2 (deals of 15 decisions) | 16: 3,142 | 64: 3,818 | 256: 4,909 | 512: 4,750 |

Single-thread CPU forward time, batch 1, fp32 (`torch.set_num_threads(1)`), milliseconds per decision, median (min to max):

| Network | Row | ms per decision |
|---|---|---|
| MLP 512-512-256 | one decision | 0.159 (0.158 to 0.161) |
| transformer | L1, mean over the 15 decisions of a deal | 5.147 (4.972 to 5.175) |
| transformer | L1, longest prefix (70 tokens) | 7.809 (7.637 to 7.949) |
| transformer | cached, mean over the 15 decisions of a deal | 2.930 (2.892 to 2.970) |

These CPU times are a weak hint for the phone goal (G14). At batch 1 most of the time is PyTorch's overhead per operation, not arithmetic. A hand-written Rust forward pass has a different cost profile. The numbers say only that the cached layout is not slower on a CPU.

Read of the numbers:

- **Update, L2 against L1 bucketed, fastest batch of each:** 6.9 (run 1), 6.8 (run 2), 8.3 (run 3; 7.8 to 8.9 from the repeats' extremes), 6.9 (run 4; 6.7 to 7.7). Over all batch-size pairings of run 3 the ratio runs from 5.3 to 11.4. Against L1 padded it is larger (5.7 to 11.8 over the runs).
- **This ratio follows from the synthetic shape.** L2 processes 85 tokens for 15 decisions, 5.7 per decision. L1 bucketed processes about 47 padded tokens per decision. The token ratio is 8.3. With other token counts the ratio changes. The real counts follow from the encoding contract (`docs/encoding.md`, draft: sequence limit 128, state token of 1,382 features). Longer sequences favour L2 more (inferred).
- **State width 1,382 changes little** (run 4): L2 update 4,707 (4,681 to 4,964), L1 bucketed update 681 (647 to 695), cached ragged acting 8,667 (8,492 to 9,351). The state width only enters the first projection. Run 4 kept 70 play tokens and 15 decisions: the contract gives a limit, not a typical count.
- **Acting, cached against uncached:** 5.0 times (ragged, host copy) and 6.5 times (idealised) over L1 bucketed in run 3. This is the effect of the cache, not of the layout.
- **Ragged acting with the host copy costs about a quarter** against the idealised row: 7,935 against 10,281.
- **Half precision: use fp32.** A raw 2048 x 2048 matmul on this card takes 24 ms in fp32 and 122 ms in fp16 (measured, same loaded GPU): fp16 is 5 times slower. In run 3 every fp16 autocast row is slower than the fp32 row at the same batch size, by 2.4 to 4.7 times. In run 1, where the load changed, one row went the other way. The autocast use is standard (`torch.autocast` with `GradScaler`), and all losses stayed finite. The card has no fast half-precision path; research/06 (f) expected little gain.
- VRAM is not the limit. The fastest update batches peak at 1.6 GB or less.
- The update dominates the step. For L2 with 4 epochs it is about 87% of the time per step.

### End-to-end estimate (derived)

Formula, for one GPU that acts once and learns `E` times on every step:

```
steps/s = 1 / (1/A + E/U)
A = acting decisions/s, U = update samples/s, E = PPO epochs
```

Example, L2 with ragged cached acting, fp32, 4 epochs: 1 / (1/7,935 + 4/4,909) = 1,063.

All numbers in this table are **derived** from the measured medians of run 3, fp32. The range in brackets uses the repeats' minimum and maximum. The engine, the copy of observations to the GPU, GAE and the full-information critic are not counted, so the real loop is slower.

| Case | Steps/s, 4 epochs | Steps/s, 2 epochs |
|---|---|---|
| MLP 512-512-256 | 32,476 (32,028 to 34,284) | 59,561 (58,612 to 62,862) |
| L1 padded, acting uncached | 96 (86 to 98) | 178 (161 to 181) |
| L1 bucketed, acting uncached | 135 (130 to 139) | 250 (240 to 258) |
| L1 bucketed update, acting cached (ragged) | 145 (139 to 147) | 285 (273 to 290) |
| L2, acting cached (idealised) | 1,096 (1,051 to 1,128) | 1,981 (1,898 to 2,037) |
| L2, acting cached (ragged) | 1,063 (1,014 to 1,092) | 1,875 (1,782 to 1,923) |

With fp16 autocast every case is 3 to 6 times lower (in the result file). Run 4 (state width 1,382) gives 1,036 and 1,851 for L2 with ragged acting, and 167 and 328 for the L1 bucketed update with cached acting.

Is G3's 10,000 steps/s reachable?

- **This cannot be answered yet for the transformer.** Every run had a game on the GPU. How much faster a free GPU is has not been measured.
- What the loaded GPU shows: the MLP is above 10,000 even so. The transformer is below 10,000 in both layouts.
- **For B10-4 the numbers favour L2.** Like for like (both with cached acting), L2 gives 7.3 times the steps of L1 bucketed in run 3 and 6.2 times in run 4. That ratio was measured under load; that it holds on a free GPU is inferred.
- The GPU-free run decides how far L2 is from G3, and whether fewer epochs, a shorter sequence, a smaller model or `torch.compile` are needed.

Against build-plan section 6 (arithmetic for a free GPU: L1 about 600, L2 about 5,000, a ratio of about 8): the measured ratio is 6 to 8. The absolute numbers are not comparable while the GPU is loaded.

### pytest output

`python -m pytest -v` in `src\train`, GPU present, probe built:

```
platform win32 -- Python 3.12.10, pytest-9.2.0, pluggy-1.7.0
rootdir: <repo>\src\train
configfile: pyproject.toml
testpaths: tests
collected 23 items

tests/test_bench.py::test_l1_l2_and_cached_acting_give_the_same_outputs PASSED
tests/test_bench.py::test_ragged_cached_acting_equals_l2_for_deals_at_different_points PASSED
tests/test_bench.py::test_ragged_step_wraps_around_and_returns_host_actions PASSED
tests/test_bench.py::test_l1_bucketed_equals_l1_padded PASSED
tests/test_bench.py::test_l2_state_tokens_do_not_see_each_other_or_later_play_tokens PASSED
tests/test_bench.py::test_l2_mask_shape_and_rows PASSED
tests/test_bench.py::test_illegal_actions_get_zero_probability PASSED
tests/test_bench.py::test_update_step_gives_finite_loss_and_changes_weights[mlp] PASSED
tests/test_bench.py::test_update_step_gives_finite_loss_and_changes_weights[L1 padded] PASSED
tests/test_bench.py::test_update_step_gives_finite_loss_and_changes_weights[L1 bucketed] PASSED
tests/test_bench.py::test_update_step_gives_finite_loss_and_changes_weights[L2] PASSED
tests/test_bench.py::test_sizes_match_the_plan PASSED
tests/test_bench.py::test_end_to_end_formula PASSED
tests/test_bench.py::test_time_step_gives_one_value_per_repeat PASSED
tests/test_bench.py::test_quick_benchmark_runs_on_cpu PASSED
tests/test_bench.py::test_sizes_can_be_set_to_the_encoding_contract PASSED
tests/test_bench.py::test_half_precision_update_is_finite_on_the_gpu PASSED
tests/test_env.py::test_envcheck_reports_this_gpu_and_exits_zero PASSED
tests/test_env.py::test_envcheck_exits_nonzero_without_cuda PASSED
tests/test_env.py::test_matmul_on_gpu_matches_cpu_within_1e_5 PASSED
tests/test_env.py::test_pyo3_probe_values_and_twelve_threads PASSED
tests/test_env.py::test_pyo3_probe_threads_run_in_parallel PASSED
tests/test_env.py::test_pyo3_probe_releases_the_gil PASSED
============================= 23 passed in 16.16s =============================
```

With the GPU hidden (`CUDA_VISIBLE_DEVICES=-1`): 20 passed, 3 skipped. On the GitHub runner the 3 probe tests skip as well, because the module is not built there.

The tests in `tests/test_env.py` for criteria 2 to 4 were written before `envcheck.py` and the probe. The first `envcheck` run failed on a wrong expected value in its own self-check (115 where the matmul gives 83). It now compares the GPU result with the CPU result.

### Open issues

1. **GPU-free run is pending (open point B).** It needs the user to close the game and other GPU apps. Then, in cmd from the repo root:

   ```bat
   .venv\Scripts\activate
   python -m tarok_train.bench_model --label "gpu free"
   ```

   It takes about 7 minutes and prints the same tables. A "desktop only" state (no game) was not measured either.
2. **Runs 1 and 2 used the first method** (one sample per row, narrower sweeps). Their files are kept and marked as superseded. Run 1 also had a changing load. Use runs 3 and 4.
3. **VRAM cap.** The benchmark caps PyTorch at the free VRAM that torch reports, less 200 MiB, and lifts the cap at the end. In run 1 torch reported 4,149 MiB free while nvidia-smi showed 5,179 of 6,144 MiB in use; large rows of run 1 may have used Windows shared memory. Runs 2 to 4 had room.
4. **CI has not run.** Nothing is pushed. Checked instead: a `pip install --dry-run` of `requirements-ci.txt` for `manylinux_2_28_x86_64` and Python 3.12 resolves to `torch 2.14.1+cpu`.
5. **Not measured:** the engine, data transfer to the GPU, GAE, the full-information critic of spec 007, `torch.compile`.
6. **Token counts of the contract.** Run 4 used the contract's state width, action count and vocabulary, but 70 play tokens and 15 decisions. The benchmark takes `--play-tokens` and `--decisions`; run it again when typical counts are known.
7. **L1 uses the causal mask**, so that its outputs equal L2's. A bidirectional L1 would cost the same in the update (inferred: the mask does not change the work), but it could not use the cache in acting.
8. **Rust toolchains.** `rustup show` lists `stable-x86_64-pc-windows-msvc` besides the active GNU one. The main session found that a root `rust-toolchain.toml` without a host selected MSVC, and disabled that file. This session used the GNU toolchain only.
9. **Files beyond the spec's list:** `src/train/requirements-ci.txt`, `tests/conftest.py`, `tests/test_bench.py`, `src/train/.gitignore`, `pyo3_probe/.cargo/config.toml`, `pyo3_probe/Cargo.lock`, four JSON files in `src/train/bench_results/`, and a rewrite of `src/train/README.md`. The user decides whether the JSON files are committed.
10. **To run the spec's commands from the repo root**, `tarok_train` is installed into the venv in editable mode: `.venv\Scripts\python -m pip install --no-build-isolation --no-deps -e src\train`. This is done on this PC and written in `src/train/README.md`.
11. **The first import of torch after the install took over two minutes.** Later starts take seconds. The cause is not checked; an antivirus scan of the new DLLs is a guess.
12. **Still to do before a commit (D72):** the user's read of the diff, CI after the push, and the copy of the `envcheck` output, the tables and the estimate into `docs/progress.md` (D109).

### Review fixes

From the review pass of the main session, items 1 to 13:

1. **Timing.** Every row: 3 repeats of at least 1 second and 3 calls each; median, minimum and maximum reported. `--seconds` and `--repeats` change it.
2. **Update sweeps extended** to 512 deals (L2), 512 decisions (L1) and 65,536 (MLP). They now reach the plateau.
3. **Acting.** The Results say that the acting rows compare cached with uncached, and that a causal L1 could use the same cache. Added: cached acting with deals at different decisions, one mask per deal, and the action copied to the host at every step. The idealised row is kept and labelled. No separate "cached L1" row: it would be the same computation as the cached row.
4. **Shape dependence** is stated. Added options `--state-features`, `--play-tokens`, `--decisions`, `--actions`, `--vocab`. One run with state width 1,382 (run 4).
5. **Result files** hold GPU load and memory numbers only. Process names are removed from the two earlier files too. The memory cap is lifted at the end of a run.
6. **Probe tests.** The probe does a fixed amount of CPU work. A new test compares the wall time on 1 and on 12 threads. The GIL test uses 2 Rust threads and asks for 5 ticks.
7. **Update-step test** checks that all weights and gradients are finite after the step.
8. **No multiples to 10,000** are stated. The ratio is given as a range over runs, repeats and batch sizes. The absolute numbers are labelled as a loaded GPU.
9. **Half precision** wording corrected; a raw matmul timing was added as evidence.
10. **"All nine criteria are met"** replaced by "met locally, except…".
11. **CPU times** are marked as a weak hint.
12. **Machine paths** replaced by `<repo>` and `<rustup>` in this section. The benchmark no longer prints or stores absolute paths. A test checks the result data for paths and process names.
13. **Note for spec 006** on where cargo reads `PYO3_USE_RAW_DYLIB` added here and in `src/train/README.md`.

Not done: nothing from the list. Not possible now: a run without the game.
