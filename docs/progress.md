# Progress log

One entry per session (D71). Newest first. Measurements are numbers that were actually measured; estimates are marked as such.

## Entry format

```
## YYYY-MM-DD — session N (tool, model)
**Done:** what was finished.
**Measurements:** numbers measured this session, with how.
**Broken:** what does not work or was dropped.
**Next:** the next step.
```

## 2026-11-10 — session 2 continued and session 3, ai-log rows 4–5 (Claude Code 2.1.306, claude-fable-5-1 then claude-fable-5-2)

**Done:** Batches 7–10 answered; decisions D70–D113. Plan finalised as `docs/planning/plan.md`. Screen spec draft 2. Figma wireframe: 28 greyscale frames and one styled sample (`docs/wireframe/wireframe-overview.png`). Pitch text draft. Build plan with twelve specs to the first full training run. Specs 001–003 approved; spec 001 (this skeleton) executed.
**Measurements:** none of the product. PC check, read-only: GTX 1660 SUPER, driver 610.88, 6144 MiB with 2836 MiB in use by the desktop; global PyTorch is 2.12.0+cpu; Rust 1.94.1 on the `x86_64-pc-windows-gnu` toolchain; maturin and wasm-pack not installed. `python tools/check_skeleton.py`: all 10 checks pass; they cover criteria 1–9 of spec 001 (criterion 10 and the CI run are checked by hand). `python tools/check_trailers.py`: all commits since 0d5045d carry the trailer.
**Broken:** nothing built yet. The proposal format is unknown. Wireframe frames A-3 and A-5 do not show the D104 step.
**Next:** specs 002 (engine core) and 003 (training environment and benchmark) in parallel; spec 004 (encoding contract) for approval.

## 2026-11-10 — first CI runs after the commit of specs 001–003

**Done:** five commits pushed (`011e288` to `3dbe257`).
**Measurements:** workflow `ci` (trailers, skeleton): success. Workflow `train` (CPU tests of `src/train`): success. Workflow `engine`: **failure**.
**Broken:** the `engine` run used the runner's newest stable compiler (clippy 1.99), because the toolchain file is switched off. That clippy has a lint the pinned 1.94.1 does not (`chunks_exact_to_as_chunks`, `src/engine/src/deal.rs:81`) and `-D warnings` turns it into an error. Tests did not run. The fix pins 1.94.1 inside `engine.yml`; it waits for the user's read of the diff.
**Next:** commit the workflow fix, re-run, then mark specs 001 and 002 done.

## 2026-11-10 — spec 003, training environment and benchmark (parallel session `train-env`, claude-fable-5-2)

**Done:** `.venv` with pinned packages (`src/train/requirements.txt`), `envcheck`, a PyO3 probe built with maturin on the GNU toolchain, a throughput benchmark of the stand-in transformer (3.49 M parameters) and the MLP control (0.83 M) in both sequence layouts. A review subagent found no functional bug, four method gaps, five overclaims and two test gaps; all were fixed and the benchmark was re-run with the better method. Not committed: waiting for the user's read of the diff.

**Measurements.** Every benchmark run had a game on the GPU (75 to 97% busy). The absolute numbers measure a loaded GPU. **The GPU-free run is pending.**

`python -m tarok_train.envcheck` (exit code 0, re-run by the main session):

```
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

- `pytest` in `src/train`: 23 tests pass (re-run by the main session).
- Download 2,045 MB; `.venv` takes 3.2 GB on disk.
- PyO3 probe: works on `stable-x86_64-pc-windows-gnu` with `PYO3_USE_RAW_DYLIB=0` (pyo3 0.30.1, maturin 1.16.0). Without it the build fails: `dlltool` finds no assembler on this PC. No MSVC needed.

Benchmark run 3 (3 repeats per row, median, fp32, loaded GPU):

| Network | Case | Acting decisions/s | Update samples/s |
|---|---|---|---|
| MLP 512-512-256 | | 358,828 | 142,829 |
| transformer | L1 bucketed (recompute per decision), acting uncached | 1,593 | 592 |
| transformer | L2 (one pass per seat and deal), acting cached, deals at different points, action copied to host | 7,935 | 4,909 |

- Update, L2 against L1 bucketed: 6.8 to 8.3 times over four runs. The ratio follows from the synthetic shape (85 tokens for 15 decisions against about 47 padded tokens per decision).
- State width 1,382 as in the encoding draft changes little: L2 update 4,707, L1 bucketed 681.
- Half precision is slower on this card: a raw 2048 × 2048 matmul takes 24 ms in fp32 and 122 ms in fp16. Use fp32.
- Single-thread CPU, batch 1: MLP 0.16 ms, transformer 2.9 ms (cached) to 5.1 ms per decision. Mostly PyTorch overhead; a weak hint for G14.

End-to-end estimate, **derived**: `steps/s = 1 / (1/A + E/U)` with A acting decisions/s, U update samples/s, E PPO epochs. The engine, data transfer, GAE and the critic are not counted.

| Case (fp32, loaded GPU) | 4 epochs | 2 epochs |
|---|---|---|
| MLP | 32,476 | 59,561 |
| transformer L1 bucketed, acting cached | 145 | 285 |
| transformer L2, acting cached | 1,063 | 1,875 |

G3 (10,000 steps/s): the MLP is above it even on the loaded GPU. For the transformer it cannot be answered until the GPU-free run; under load both layouts are below it, L2 by less.

**Broken:** no GPU-free run yet. CI not run. An unknown Python 3.14 process used the GPU during run 1 and ended on its own. The first import of torch after the install took over two minutes; cause unknown.
**Next:** the user runs the GPU-free benchmark; the layout choice (D110) follows; spec 007 (model and learner) is drafted on that choice.

## 2026-11-10 — spec 002, engine core (parallel session `engine-core`, claude-fable-5-2)

**Done:** `src/engine`, crate `tarok-engine`: cards, card sets, card points, seeded deal, trick winner, legal cards for normal contracts, card-play state machine, benchmark binary. Tests were written from the 33 criteria before the code. A review subagent found no rule bug and seven test or API gaps; all seven were fixed. Not committed: waiting for the user's read of the diff.
**Measurements:**
- `cargo test --locked`: 47 tests pass (re-run by the main session). `cargo fmt --check` and `cargo clippy --locked --all-targets -- -D warnings`: clean.
- Property tests: 10,000 cases each for criteria 6, 7, 27–30.
- Benchmark, `cargo run --release --bin bench_play`, one thread, Ryzen 5 5600: 35.8 to 36.6 million card plays per second (about 750,000 deals per second), three runs. The reviewer measured 34.4 million with the CPU shared. Card play only: no bidding, no encoding, no network. research/05 D4 had inferred 10^5 to 10^6.
- Shuffle check by the reviewer: chi-square 2742 for 2809 degrees of freedom on 540,000 decks. The Rust deal equals a separate pure-Python derivation on five seeds.
- Mutation check by the author: seven wrong rules put into the code one at a time; each made a numbered test fail.
**Broken:**
- `rust-toolchain.toml` at the root makes plain `cargo` fail on this PC: rustup's default host is MSVC and that toolchain cannot link here. Runs used `RUSTUP_TOOLCHAIN=stable-x86_64-pc-windows-gnu`. The user chooses the fix.
- Incident: a review agent ran `rustup toolchain list` in the repository; the toolchain file made rustup install `1.94.1-x86_64-pc-windows-msvc` (588 MB) without anyone asking. The main session uninstalled it and renamed the file to `rust-toolchain.toml.disabled`. The two toolchains that were there before (stable GNU, stable MSVC) are untouched.
- proptest is built without its `std` feature (no `dlltool` assembler on this PC), so the tests seed their own runner; regression files and `PROPTEST_*` settings are lost.
- Criterion 32 (CI green) is unverified until the first push.
**Next:** spec 005 (deal flow A). Notes for it: legality and trick rules need a rules parameter for klop and berač; `PlayState` needs a constructor for positions inside a deal; which seat is forehand is not defined yet.

## 2026-11-09 — session 2 (Claude Code 2.1.306, claude-fable-5-1)

**Done:** Batches 1–6 answered; decisions D1–D69. Git repository and private GitHub repo. Deep research on training methods, network and browser inference, and explanation methods (`docs/planning/research/05`–`07`). Rule-switch list with 39 toggles. Goals G1–G19.
**Measurements:** none.
**Broken:** nothing.
**Next:** agent workflow batch.

## 2026-11-06 — session 1 (Claude Code 2.1.306, claude-fable-5-1)

**Done:** Course requirements extracted (R1–R20). First research pass (`docs/planning/research/01`–`04`): rules and variants, existing software, bot methods, card artwork licensing. Planning files created.
**Measurements:** none.
**Broken:** nothing.
**Next:** first decision batch.
