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

## 2026-11-12 — spec 006 committed, the user's answers recorded, specs 008 and 013 started (session 3, Claude Code 2.1.306, claude-fable-5-2; ai-log row 6)

**Done:** the user answered everything open ("Specs accepted, continue"; D123–D131). Spec 006 is committed in two commits (`8f24dbb` engine, `8cb2bb0` bindings) and pushed. Batch 11, Batch 12 and C11 are recorded. `docs/encoding.md` has section 4a with the fifteen readings; the version stays 1. Spec 008 is approved and carries the settings of D127 and the stale-module check. Spec 013 is new: three rulings that change the engine (a valat keeps the mondfang; pagat ultimo by side; the pagat wins a trula trick only if it falls last). The local compiler pin file is deleted (D129). Spec 013 went to the `engine-core` session and spec 008 to the `train-env` session.
**Measurements:**
- CI on `8cb2bb0`: `ci` success; `engine` success in 3 min 1 s; `train` success in 4 min 45 s. The `train` log shows the wheel of `tarok_env` built on Linux in 27 s and pytest with 88 passed, 8 skipped (GPU), 5 deselected (slow). The binding crate has no Rust tests of its own ("0 passed"), so `cargo test` there only proves that it links.
- After the amendment of `docs/encoding.md`: `tools/check_encoding.py` 3 of 3, the engine's `c01` contract test and the learner's three `c01` tests pass.
**Broken or open:**
- The engine plays the old readings of the three rulings until spec 013 is in.
- C12 is open: what "falls last" means exactly. Spec 013 builds reading (a) in one marked place.
- G3 has no figure now (D128); it is proposed after the smoke run (C13).
- Open issues of spec 006 stay as listed in its entry below.
**Next:** verify and review the output of spec 013 and of spec 008's code; the user's read of each diff; then the three smoke runs (about three hours with the GPU free).

## 2026-11-11 — spec 006, encoding, vectorised environment, bindings, record (session `engine-core`, claude-fable-5-2; verified by the main session, ai-log rows 6 and 6c)

**Done:** in `src/engine`: an event log on `Game`; the module `encode` (tokens, the 1,382 state features with the exact inference bits, mask, critic features, targets, reward with ten parts); `vecenv` (N tables, runs of 24 deals, own thread pool, behind a feature so a build without threads stays possible); `record` (JSON, replay). A separate binding crate `src/train/tarok_env` (PyO3 0.30.1, no `unsafe`, no rule in it) and `src/train/tarok_train/env.py` with the mock's interface. A review subagent found no leak, no unsound bit and no wrong reward in the code. It found six planted wrong rules that the suite let through (one a leak), loose array checks in the binding, a contract reading nobody had listed (C4) and three overclaims; 19 items, all fixed. Not committed: waiting for the user's read of the diff.
**Measurements:**
- Re-run by the main session on the fixed tree: `cargo fmt --check` and clippy clean for both crates; `cargo test --locked` 180 tests; `--no-default-features` 169; no `pyo3` in the engine's dependency tree. Full pytest with the GPU and the slow tests: 101 passed in 253 s.
- Environment alone, 1,024 tables, random legal play, a full observation written at every step (session's run after the fixes, median of 3): through Python 396,822 steps/s with 1 thread and 976,682 with 12; inside the engine 564,587 and 3,213,112. The main session measured 374,161 and 968,028 through Python before the fixes. With 12 threads the single Python thread is the limit. The environment does not limit G3.
- Longest token sequence: 66 in 100,000 random deals; 71 is the longest possible in this slice. The limit is 128.
- No-leak test, last run: 15,415 cases compared in four kinds, 1,287 of them the kind the review added (which card the declarer put down).
- Soundness: 10,000 deals, four observers at every decision; 4.2 million void bits, 223 million "cannot hold" bits and 1.0 million "certainly holds" bits checked against the true hands, none wrong.
- Planted wrong rules: 26 by the author (24 caught; one changes nothing in this slice, one was dead code and is removed), 6 by the reviewer (0 caught before the fixes), 22 for the review items after the fixes (all caught). The main session planted the reviewer's leak again in a scratch copy: the no-leak test fails, the soundness test passes, as it should.
**Broken or open:**
- The two workflows are unverified for this code until the push: the build of the module on Linux, and `cargo test` of the binding crate there.
- A thread that holds the observation arrays can read half-written rows while another thread steps. `EngineEnv` has a lock for its own calls and the rule is written down; the code cannot enforce the rest without copying every observation.
- A stale installed module is not detected: after a change in `src/engine` the module must be built again (`maturin develop --release` in `src\train\tarok_env`).
- The trainer still starts the mock; the switch to the engine is in spec 008.
- The declarer's hidden discards give repeated token sequences; only the state token differs between them.
- Fifteen points wait for the user in Batch 12.
**Next:** the user's read of the diff of spec 006; commit; push; the two workflows. Then spec 008 once approved.

## 2026-11-11 — commit of specs 005 and 007, CI (main session, ai-log row 6; Claude Code 2.1.306, claude-fable-5-2)

**Done:** five commits pushed, `2bd5f21` to `819220d`: the engine's deal flow A (spec 005), the GPU-free benchmark results (spec 003), the encoding check script (spec 004), the model and learner (spec 007), and D122. The user's go-ahead was the word "Continue" in answer to the commit question; D122 records it as said. Specs 003, 004, 005 and 007 are done. Spec 006 is handed to the `engine-core` session. The open rulings are written down as Batch 11 in `docs/planning/open-questions.md`.
**Measurements:**
- Before the commits, on this PC: `cargo fmt --check` and clippy clean, 130 engine tests pass; the full training suite, with the GPU and the slow tests: 75 passed in 1,100 s. The CI set without a GPU (`-m "not slow"`, `CUDA_VISIBLE_DEVICES=-1`): 65 passed, 5 skipped, 5 deselected in 285 s.
- CI on `819220d`: `ci` success; `engine` success (format, clippy, 130 tests, the deal against the Python reference); `train` success: 62 passed, 8 skipped, 5 deselected in 148 s on the runner. The three more skips than here are the PyO3 probe tests; the probe is not built on the runner.
**Broken:** nothing new. The open points of the entries below stand: G3 and the transformer, the plateau on the token task, the value head's gradient, the acting cache, the valat and mond ruling.
**Next:** spec 006 (encoding in Rust, vectorised environment, bindings, record). Spec 008 (pipeline smoke run) to be drafted for approval. Batch 11 for the user.

## 2026-11-10 — session 2 continued and session 3, ai-log rows 4–5 (Claude Code 2.1.306, claude-fable-5-1 then claude-fable-5-2)

**Done:** Batches 7–10 answered; decisions D70–D113. Plan finalised as `docs/planning/plan.md`. Screen spec draft 2. Figma wireframe: 28 greyscale frames and one styled sample (`docs/wireframe/wireframe-overview.png`). Pitch text draft. Build plan with twelve specs to the first full training run. Specs 001–003 approved; spec 001 (this skeleton) executed.
**Measurements:** none of the product. PC check, read-only: GTX 1660 SUPER, driver 610.88, 6144 MiB with 2836 MiB in use by the desktop; global PyTorch is 2.12.0+cpu; Rust 1.94.1 on the `x86_64-pc-windows-gnu` toolchain; maturin and wasm-pack not installed. `python tools/check_skeleton.py`: all 10 checks pass; they cover criteria 1–9 of spec 001 (criterion 10 and the CI run are checked by hand). `python tools/check_trailers.py`: all commits since 0d5045d carry the trailer.
**Broken:** nothing built yet. The proposal format is unknown. Wireframe frames A-3 and A-5 do not show the D104 step.
**Next:** specs 002 (engine core) and 003 (training environment and benchmark) in parallel; spec 004 (encoding contract) for approval.

## 2026-11-11 — GPU-free benchmark (spec 003, open point B) and spec 007, model and PPO learner (session `train-env`, claude-fable-5-2)

**Done:** the benchmark of spec 003 ran once with the GPU idle. Spec 007 is written: the transformer in layout L2, the MLP control, the full-information critic, PPO with ten-part decomposed values (D121), whole-deal collection, past-checkpoint seats, checkpoints with resume, TensorBoard logs, a mock environment. A review subagent found no bug in the PPO math or the model, but showed that the first learning test let ten planted bugs through; thirteen findings were fixed. Not committed: waiting for the user's read of the diff.

**Measurements, GPU free** (nvidia-smi before and after: 0% busy, 234 MiB used; 3 repeats per row, medians, fp32; repeats agree within 1%):

| Network | Case | Acting decisions/s | Update samples/s |
|---|---|---|---|
| MLP 512-512-256 | | 1,546,968 | 640,075 |
| transformer | L1 bucketed, acting uncached | 7,673 | 2,532 |
| transformer | L2, acting cached, ragged, action copied to host | 34,911 | 19,864 |

- The free GPU is 4.0 to 4.8 times faster than under the game, in every row.
- Update, L2 against L1 bucketed: 7.8 times. The layout choice (D116) now stands on free-GPU numbers.
- Half precision is 2.6 to 4.5 times slower. fp32 stays.
- End-to-end estimate, **derived**, learner alone, the benchmark's synthetic shape: L2 gives 7,732 steps/s with 2 epochs, 4,348 with 4, 12,660 with 1. MLP 265,177 with 2 epochs.

**Measurements, spec 007** (session's numbers; tests re-run by the main session):

- Parameters: transformer 3,767,076; MLP control 1,242,916; critic 1,244,424.
- Mock task that can only be solved from the observation (random play 0.25, best 1.0), CPU test, 30 updates: transformer 0.28 → 0.80 in 49 s (the rewarded slot is named by a token); MLP 0.25 → 0.94 in 12 s (named by a state feature). Controls: reward to the wrong seat stays at 0.25; the MLP on the token task stays under 0.52.
- Ten planted bugs (mask left out of the update, stored log-probability zeroed, critic never trained, advantages reversed, no bootstrapping, reward averaged or swapped across deals, state from another seat-deal, decision points zeroed, cache never reset): each now fails at least one test. With the first version of the test all ten passed.
- 200 updates on the free GPU, 1,024 tables: MLP plays 19,604 decisions per second (median), transformer 5,621. Only whole deals under the current weights are trained on, so 39% of the decisions played are trained on with 2,048 seat-deals per update. Decision age in every update: 0. Peak GPU memory of the transformer run: 3,621 MiB.
- Gradient norms on the trunk at the transformer's first update: policy 1.33, value head 19.9, Q head 2.21; whole actor 22.2 against a clip of 0.5.

**Broken or open:**
- G3 (10,000 steps/s): not reached by the transformer in any measured or derived number with 2 or more epochs, before the engine is counted. The MLP is far above it.
- The transformer stops at 0.81 on the token task: it follows the token (1.000 right) and ignores the state hint (0.248 right), with policy entropy near zero from update 20. The cause is not established; an early entropy collapse is the guess.
- The policy head's first scale had to go from 0.01 to 1.0 for anything to learn on the stronger task; with 1.0 the first policy is not uniform.
- The acting cache needs about 8 KB per token per table and seat: 1,024 tables with 128-token deals do not fit on the 6 GB card next to the update.
- Whole-deal collection wastes the deal that straddles each update.
- The `train` workflow is unverified for this code until the push.

**Next:** the user's answers on loss weights, first scale and the plateau; the read of the diff; commit; then spec 006.

## 2026-11-10 — spec 005, deal flow A (session `engine-core`, claude-fable-5-2)

**Done:** in `src/engine`: bidding with priority, the declarer's contract choice, king call, talon packets and discards, the called king in the talon, play, and the score of the six normal contracts with silent bonuses and mondfang. Actions map to the 160 slots of the encoding contract. Public, private and full-information views are separate types. A review subagent found one bug (the debug text of the public view printed hidden cards) and test gaps in scoring and bidding; all are fixed. Not committed: waiting for the user's read of the diff.
**Measurements:**
- `cargo test --locked`: 130 tests pass (re-run by the main session); 47 of them are spec 002's. Format and clippy clean.
- Benchmark, `cargo run --release --bin bench_game`, one thread, CPU shared with a training run: 349,000 to 362,000 whole random deals per second, about 20 million actions per second, 56.8 actions per deal on average. The reviewer measured 316,000.
- Auctions: 1,297 possible in this slice, all walked for each dealer against a model written from the criteria. The longest has 13 calls. For the full ladder of the Default preset the longest has 19 calls (36,182 auctions, by the same model), so the 128-token limit of the encoding holds with room.
- A second scorer written only from the criteria agrees with the engine on 10,000 random deals.
- Mutation checks: 32 wrong rules by the author, 4 more scoring rules and 4 leaks after the review; each fails at least one test.
**Broken:**
- Open ruling: what a valat does to a lost mond (three cases). The current behaviour (the valat cancels every mondfang) is pinned by five tests.
- Random test auctions are thin (85% have one bid); the exhaustive walk covers the bidding instead.
- No rules parameter yet for score values, phase order and legality; flows B and C need one. No ordered action history yet; spec 006 adds the event log.
- The `engine` workflow is unverified for this code until the push.
**Next:** the user's read of the diff; commit; then spec 006.

## 2026-11-10 — first CI runs after the commit of specs 001–003

**Done:** five commits pushed (`011e288` to `3dbe257`).
**Measurements:** workflow `ci` (trailers, skeleton): success. Workflow `train` (CPU tests of `src/train`): success. Workflow `engine`: **failure**.
**Broken:** the `engine` run used the runner's newest stable compiler (clippy 1.99), because the toolchain file is switched off. That clippy has a lint the pinned 1.94.1 does not (`chunks_exact_to_as_chunks`, `src/engine/src/deal.rs:81`) and `-D warnings` turns it into an error. Tests did not run. The fix pins 1.94.1 inside `engine.yml`; it waits for the user's read of the diff.
**Next:** commit the workflow fix, re-run, then mark specs 001 and 002 done.

Update, same day: the fix is commit `56f4caf` (D115). All three workflows are green on it. In `engine`: format, clippy, 47 tests, and the deal compared with the Python reference on five seeds all passed on Rust 1.94.1. Specs 001 and 002 are done. Spec 003 waits only for the GPU-free benchmark run.

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
