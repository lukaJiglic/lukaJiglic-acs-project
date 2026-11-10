# Build plan — shortest path to the first training run

Status: **draft 2, 2026-11-10, updated the same day.** Batch 10 answered (D107–D113). Specs 001 and 002 are done; 003 is committed and waits for the GPU-free benchmark run; the layout is L2 (D116). Specs 004, 005 and 007 are drafts waiting for approval. Priority set by the user (D106): get model training running as soon as possible.

This file orders the work. The decisions behind it are in `docs/planning/decisions.md`; the design is in `docs/planning/plan.md`. Each row below becomes a spec in `docs/specs/` that the user approves before an agent executes it (D70, D73).

## 1. What "first training run" means (D107)

The user chose to train **everything from the start**. No phase-by-phase curriculum for the first run.

- Self-play on the Default preset (D43). Four seats, all the same network; 20% of games with past checkpoints once checkpoints exist (D14).
- The network makes **every decision**: bids, king call, talon packet, discards, announcements, kontra, cards.
- Contracts: klop, tri, dve, ena, solo tri, solo dve, solo ena, berač, solo brez, valat. Odprti berač and barvni valat stay pinned off (D44).
- Reward: the deal score per seat, divided by a constant, at the end of the deal only (research/05 C4).
- Monitors from the first step, because training all phases at once is where the documented failures sit (research/05 e1, e1b, e2): declare rate and pass rate per contract, klop rate, announcement and kontra rates, policy entropy per phase, partner-prediction accuracy, illegal-move count (must be zero).
- Done when: a run of at least 10^7 steps completes, checkpoints are saved, the monitors are logged, and the checkpoint is measured against the random bot and the rule bot on seeded duplicate deals.

Consequence: the engine needs the whole Default-preset rule set before this run. That makes the path longer than the stage-1 path of draft 1. Section 3 adds a short pipeline test on partial rules so the training code is proven early.

## 2. Critical path

```
001 repo skeleton
 ├─ 002 engine core ─ 005 deal flow A ─ 006 bindings ─────────┬─ 008 pipeline smoke run ─┐
 │                         └─ 009 flow B ─ 010 flow C ─ 011 bots ───────────────────────┤
 ├─ 004 encoding contract (document) ─────┬───────────────────┤                          ├─ 012 first full training run
 └─ 003 environment and benchmark ─ 007 model and learner ────┘──────────────────────────┘
```

Two tracks run side by side in separate sessions (D109): Rust (002, 005, 006, 009, 010, 011) and Python (003, 007). They touch different folders. 004 is the contract between them.

| # | Spec | Track | Needs | Output | Done when |
|---|------|-------|-------|--------|-----------|
| 001 | Repo skeleton | — | — | README with goals, `CLAUDE.md`, progress and architecture files, licences, layout, first CI job | **approved**; see the spec |
| 002 | Engine core | Rust | 001 | Crate `tarok-engine`: cards, card sets, card points, seeded deal, trick winner, legal cards for normal contracts, card-play state machine | **approved**; see the spec |
| 003 | Training environment and benchmark | Python | 001 | Pinned venv with the CUDA wheel; GPU check; PyO3 probe on this toolchain; throughput numbers for the transformer in both sequence layouts (D110) and for the MLP | **approved**; see the spec |
| 004 | Encoding contract | doc | — | `docs/encoding.md`: card indices, token vocabulary and action vector for **all** phases, state-token features with the exact inference bits (D30), critic features, reward scale, how radelci enter | the user approves the document |
| 005 | Deal flow A | Rust | 002 | Bidding with priority, king call (incl. king in the talon, S12 a), talon packets and discards (S14 a), scoring of tri to solo ena with silent bonuses, mondfang, rounding (S25 a) | scenario tests per rule; property tests on 10,000 random deals; hand-written example deals score correctly |
| 006 | Python bindings, vectorised environment | Rust + Python | 004, 005 | PyO3 module: N games at once; observations, masks, critic features, rewards as NumPy arrays; actions for rules not built yet are masked off; the JSON game record, first version (moved here from 005, since it follows the encoding) | parity with the Rust API; seat-relative encoding tests; episode-boundary tests; environment-only steps/s measured |
| 007 | Model and PPO learner | Python | 003, 004 | Transformer (D18) in layout L2 (D116), MLP control, PPO with the full-information critic, all heads, checkpoints, TensorBoard logs; built against a mock environment | unit tests: masking, GAE at episode ends, loss shapes, checkpoint round trip; a short mock run moves the loss |
| 008 | Pipeline smoke run | both | 006, 007 | A short run (about 10^6 steps) on flow A only: six contracts, forehand plays tri if all pass. **A test of the pipeline, not the first training run.** | the run finishes; no illegal moves; the agent beats the random bot; end-to-end steps/s measured against G3 |
| 009 | Deal flow B | Rust | 005 | Announcements (trula, kralji, kralj ultimo, pagat ultimo, valat) and kontra, four levels (S18 a), their scoring | scenario tests per announcement and kontra level; property tests |
| 010 | Deal flow C | Rust | 009 | Klop (talon cards on the first six tricks, must overtake, pagat last, S28 b), berač, solo brez, valat, radelci (S32 a, S33 a) | scenario tests per contract; property tests; Default preset complete |
| 011 | Bots and arena | Rust | 010 | Random bot; rule bot v0 (heuristics listed in the spec for the user to correct); seeded duplicate tournaments (8 seatings, D25); per-deal score spread | rule bot beats random with the 95% interval above zero on 1,000 deals; spread in `docs/progress.md` |
| 012 | First full training run | both | 006–011 | Run config in `src/train/configs/`; a 10^7-step run with every phase learned; evaluation; monitors | as in section 1; results and failures in `docs/progress.md`; then longer runs |

## 3. The pipeline smoke run (Claude's addition, open to veto)

With everything trained from the start, the first real run waits for the whole rule set. Spec 008 runs the training code on the rules that exist after flow A, for about an hour. It is not counted as a result. It answers early: does the Rust to Python loop work, are seats and rewards wired correctly (research/05 e10), how many steps per second does the whole loop reach. The encoding reserves every later action from the start, so nothing built for 008 is thrown away.

## 4. Deferred until training is running

- Non-default values of the T0 toggles; odprti berač and barvni valat; session handling in the app.
- The full G1 test count beyond what the specs above bring.
- Hand-written Rust forward pass and its golden test (D54, D111), WASM build, parity tests (G2).
- Search benchmark, Deep Monte Carlo control run, rule vector.
- The app shell, cards, PWA, Pages.
- The report skeleton (D85): a small spec right after 001.

## 5. What was found on the training PC (read-only checks, 2026-11-10)

| Item | Found | Consequence |
|------|-------|-------------|
| GPU | GTX 1660 SUPER, driver 610.88, 6144 MiB, 2836 MiB in use by the desktop | CUDA 13 wheels are usable (research/06 f). About 3.3 GB is free while the desktop is in use; close other GPU apps for long runs. |
| PyTorch | 2.12.0+cpu in the global Python 3.12.10; `cuda.is_available()` is False | Spec 003 makes a project venv with a pinned CUDA wheel. The global install is left alone. |
| Rust | cargo 1.94.1, toolchain `x86_64-pc-windows-gnu` only | PyO3 and maturin are usually run with the MSVC toolchain on Windows (research/06 f, from memory). Spec 003 tests the GNU toolchain first. If it fails: install the MSVC build tools, or use file hand-off. The user decides then. |
| Missing tools | maturin, wasm-pack, the `wasm32-unknown-unknown` target | maturin comes with spec 003. The WASM tools come with the WASM spec. |

## 6. Risk: transformer cost against the speed gate

All numbers here are **inferred by arithmetic**, not measured. Spec 003 measures both layouts; the choice follows the measurement (D110).

- A 4-layer, 256-wide transformer with a 1024-wide feed-forward has about 3.1 M parameters, so about 6 MFLOP per token.
- A deal has about 60 decisions and a sequence that grows to about 70 tokens.
- **Layout L1:** every decision recomputes the whole sequence. Acting costs about 250 MFLOP per decision, a PPO update with 4 epochs about 3,000 MFLOP per step. On this GPU that is roughly 600 steps/s, far under G3's 10,000.
- **Layout L2:** play and bid tokens are computed once per seat and deal; each decision adds only its state token, which reads the tokens before it through a mask. Roughly 5,000 steps/s, more with fewer epochs or half precision.

The MLP control costs about 100 times less per step and is not at risk.

## 7. How each spec runs (D70–D79, short form)

1. Claude drafts the spec with acceptance criteria. The user approves the criteria.
2. An agent writes the tests from the criteria, then the code.
3. Before each commit: criteria met, tests pass locally, the user reads the diff, a separate agent review pass, CI green after push.
4. Commit with `Spec:`, `Produced-by:` and `Session:` trailers. Update `docs/progress.md` and the spec status.

Parallel sessions (D109) share one checkout. Each touches only its own spec's files and commits nothing; results go into a "Results" section of its own spec file. The main session collects the results into `docs/progress.md` and commits after the user has read the diff. Each track has its own CI workflow file (`ci.yml` for the repo checks, `engine.yml`, `train.yml`) so the tracks never edit the same file.

## 8. After the first full run (order, not dates)

Long runs and monitors · Rust forward pass and golden test · WASM build and parity · app shell with Play against the rule bot · explanations · Learn · Adviser · evaluation · report (D82).
