# Build plan — shortest path to the first training run

Status: **draft 1, 2026-11-10. Waiting for the user's answers to Batch 10** (`docs/planning/open-questions.md`) and approval of specs 001–003. Priority set by the user (D106): get model training running as soon as possible.

This file orders the work. The decisions behind it are in `docs/planning/decisions.md`; the design is in `docs/planning/plan.md`. Each row below becomes a spec in `docs/specs/` that the user approves before an agent executes it (D70, D73).

## 1. What "first training run" means

Stage 1 of the curriculum (D21, D26), as proposed in B10-1:

- Self-play on the Default preset (D43). Four seats, all the same network, 20% of games with past checkpoints once checkpoints exist (D14).
- The network learns **card play**. Bidding, king call, talon choice and discards are made by rule-based code. Their tokens are in the sequence from the start (D21).
- Contracts in stage 1: tri, dve, ena, solo tri, solo dve, solo ena. If the other three pass, forehand plays tri (legal under the baseline), so no klop yet.
- No announcements and no kontra yet. Silent bonuses and mondfang are scored. Radelci are off (each deal stands alone).
- Reward: the deal score per seat, divided by a constant, at the end of the deal only (research/05 C4).
- Done when: a run of at least 10^7 steps completes, checkpoints are saved, the monitors are logged, and the checkpoint is measured against the random bot and the rule bot on seeded duplicate deals.

## 2. Critical path

```
001 repo skeleton
 ├─ 002 engine core ─ 005 deal flow (stage-1 rules) ─ 006 bots and arena ─┐
 ├─ 004 encoding contract (document) ──────────────┬─ 007 Python bindings ─┤
 └─ 003 training environment and benchmark ─ 008 model and PPO learner ────┴─ 009 first training run
```

Two tracks can run side by side after 001 (B10-3): Rust (002, 005, 006, 007) and Python (003, 008). They touch different folders. 004 is the contract between them and is written first.

| # | Spec | Track | Needs | Output | Done when |
|---|------|-------|-------|--------|-----------|
| 001 | Repo skeleton | — | — | README with goals, `CLAUDE.md`, progress and architecture files, licences, layout, first CI job | drafted; see the spec |
| 002 | Engine core | Rust | 001 | Crate `tarok-engine`: cards, card sets, card points, seeded deal, trick winner, legal moves for normal contracts, card-play state machine | drafted; see the spec |
| 003 | Training environment and benchmark | Python | 001 | Pinned venv with the CUDA wheel; GPU check; PyO3 probe on this toolchain; throughput numbers for the transformer and the MLP on synthetic data | drafted; see the spec |
| 004 | Encoding contract | doc | — | `docs/encoding.md`: card indices, token vocabulary with reserved slots for later phases, state-token features with the exact inference bits (D30), action vector (~140 slots), critic features, reward scale | the user approves the document; both tracks build against it |
| 005 | Deal flow, stage-1 rules | Rust | 002 | Bidding with priority, king call (incl. king in the talon, S12 a), talon packets and discards (S14 a), scoring of the six stage-1 contracts with silent bonuses, mondfang, rounding (S25 a); game record JSON, first version | scenario tests per rule; property tests on 10,000 random deals; scores of hand-written example deals match |
| 006 | Bots and arena | Rust | 005 | Random bot; rule bot v0 (bidder, caller, talon, card play; heuristics listed in the spec for the user to correct); CLI that plays seeded duplicate tournaments (8 seatings, D25); measurement of the per-deal score spread | rule bot beats random with the 95% interval above zero on 1,000 deals; spread written to `docs/progress.md` |
| 007 | Python bindings, vectorised environment | Rust + Python | 004, 005, 006 | PyO3 module: N games in parallel, observations, masks, critic features and rewards as NumPy arrays; rule-based seats handled inside Rust | parity tests against the Rust API; seat-relative encoding tests; episode-boundary tests; environment-only steps/s measured |
| 008 | Model and PPO learner | Python | 003, 004 | Transformer (D18) and MLP control on the contract; PPO with the full-information critic; heads (policy, Q per action, decomposed value, partner seat, card location); checkpoints; logging; built against a mock environment that emits random legal data | unit tests: masking, GAE at episode ends, loss shapes, checkpoint round trip; a short run on the mock environment does not crash and loss moves |
| 009 | First training run, stage 1 | both | 006, 007, 008 | Run config in `src/train/configs/`; a 10^7-step run; evaluation against random and rule bot; G3 throughput measured; monitors (entropy, partner-prediction accuracy, illegal-move count) | as in section 1; results and failures written to `docs/progress.md`; then longer runs start |

## 3. Deferred until training is running

These are T0 and will be built, but they do not block the first run. The GPU works on stage 1 while they are written (D82: app work fills GPU time).

- Engine: klop, berač, solo brez, valat, announcements, kontra, radelci and sessions, the remaining T0 toggles, the full G1 test count.
- Hand-written Rust forward pass and its golden test (D54), WASM build, parity tests (G2). See B10-5.
- Search benchmark (stage 3), Deep Monte Carlo control run, rule vector.
- The app shell, cards, PWA, Pages.
- The report skeleton (D85) gets its own small spec right after 001.

To keep later stages from breaking earlier checkpoints, the encoding contract (004) fixes the token vocabulary and the action vector for **all** phases from the start, with unused slots masked in stage 1 (plan section 3, "keep the vocabulary fixed").

## 4. What was found on the training PC today (read-only checks, 2026-11-10)

| Item | Found | Consequence |
|------|-------|-------------|
| GPU | GTX 1660 SUPER, driver 610.88, 6144 MiB, 2836 MiB in use by the desktop | CUDA 13 wheels are usable (research/06 f). About 3.3 GB is free while the desktop is in use; close other GPU apps for long runs. |
| PyTorch | 2.12.0+cpu in the global Python 3.12.10; `cuda.is_available()` is False | Spec 003 makes a project venv with a pinned CUDA wheel. The global install is left alone. |
| Rust | cargo 1.94.1, toolchain `x86_64-pc-windows-gnu` only | PyO3 and maturin are usually run with the MSVC toolchain on Windows (research/06 f, from memory). Spec 003 tests the GNU toolchain first. If it fails: install the MSVC build tools, or use file hand-off. The user decides then. |
| Missing tools | maturin, wasm-pack, the `wasm32-unknown-unknown` target | maturin comes with spec 003. The WASM tools come with the WASM spec. |

## 5. Risk found while planning: transformer cost against the speed gate

All numbers here are **inferred by arithmetic**, not measured. Spec 003 measures them.

- A 4-layer, 256-wide transformer with a 1024-wide feed-forward has about 3.1 M parameters, so about 6 MFLOP per token.
- A deal has about 60 decisions and a sequence that grows to about 70 tokens.
- If every decision recomputes the whole sequence, acting costs about 250 MFLOP per decision, and a PPO update with 4 epochs about 3,000 MFLOP per step. On this GPU (about 5 TFLOPS peak, less in practice) that is roughly **600 steps/s**, far under G3's 10,000.
- If the play and bid tokens are computed once and each decision adds only its state token, training one seat's deal is one pass, and acting reuses cached tokens. The same arithmetic gives roughly **5,000 steps/s**, and more with fewer epochs or half precision.

So the sequence layout decides whether the gate is reachable. B10-4 asks about it. The MLP control costs about 100 times less per step and is not at risk.

## 6. How each spec runs (D70–D79, short form)

1. Claude drafts the spec with acceptance criteria. The user approves the criteria.
2. An agent writes the tests from the criteria, then the code.
3. Before each commit: criteria met, tests pass locally, the user reads the diff, a separate agent review pass, CI green after push.
4. Commit with `Spec:`, `Produced-by:` and `Session:` trailers. Update `docs/progress.md` and the spec status.

## 7. After the first run (order, not dates)

010 stage-1 long runs and monitors · 011 engine: announcements and kontra · 012 engine: klop, berač, solo brez, valat, radelci · 013 Rust forward pass and golden test · 014 WASM build and parity · 015 app shell with Play against the rule bot · then stage 2 training (talon, bidding, announcements), explanations, Learn, Adviser, evaluation, report (D82).
