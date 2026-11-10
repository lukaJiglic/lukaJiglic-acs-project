# 006 — Observation encoding, vectorised environment, Python bindings, game record

Status: **draft** (2026-11-10). Not approved. Needs spec 004 (approved) and spec 005 (in execution). Its details may shift once spec 005's API exists; any shift is reported before execution.

Build-plan row 006. Tool and model: Claude Code 2.1.306, `claude-fable-5-2`.

## Goal

Make the engine usable by the learner. Four parts:

1. The encoding of `docs/encoding.md` version 1, computed in Rust: tokens, state features with the exact inference bits, action mask, critic features, auxiliary targets, reward.
2. A vectorised environment in Rust: many tables stepped at once on all CPU threads.
3. A Python module built with PyO3 and maturin that hands these arrays to the learner as NumPy arrays, behind the same interface as the mock environment of spec 007.
4. The JSON game record, first version (D59), with replay.

After this spec the pipeline smoke run (spec 008) can start.

## In scope

- `src/engine`: a public event log on `Game`; a module `encode` (tokens, state blocks A–G, mask, critic, targets, reward); a module `vecenv` (N tables, each a run of 24 deals with the dealer moving one seat per deal, D117); a module `record` (JSON, replay). Features and actions of rules not built yet stay zero and masked.
- `src/train/tarok_env/`: a separate small Rust crate with the PyO3 bindings, depending on `tarok-engine` by path. The engine crate itself stays free of Python, because it must also compile to WASM. The build setting found in spec 003 (`PYO3_USE_RAW_DYLIB=0`) lives in this crate's `.cargo/config.toml`.
- `src/train/tarok_train/env.py`: the wrapper with the interface of `mock_env.py`.
- An environment-only benchmark from Python: steps per second with a random policy.
- CI: the `engine` workflow runs the new Rust tests; the `train` workflow builds the module on Linux and runs the parity tests.

## Out of scope

The learner (spec 007). Any training run (spec 008). Announcements, kontra, klop, berač, radelci (flows B and C; their slots and features exist and are zero). WASM. The app's use of the record.

## Acceptance criteria

The user approves these before tests are written (D73).

**Encoding**
1. The sizes exported by the engine (288 events, 1,382 state features, 160 actions, sequence limit 128, 274 critic extras) equal the tables of `docs/encoding.md`; a test reads the document.
2. Tokens of a scripted deal equal a hand-written list: event ids, relative seats, trick numbers, positions in trick, indices.
3. Before card play every token has trick 0 and position 0. In card play the positions run 1 to 4 and the trick numbers 1 to 12.
4. Hidden discards never appear as tokens. A discarded tarok appears as a "discard shown" token.
5. Seat rotation: the same moment seen from seat s and from the next seat differs in every per-seat block exactly by one rotation, and the tokens' seat fields likewise.
6. **No leak.** Take a random moment of a random deal. Swap two cards that the observing seat cannot see between two hidden places (two other hands, or a hand and the hidden discards). The observing seat's tokens, state and mask do not change. 10,000 cases.
7. Each inference feature F1–F6 has a test on a hand-built position: a void shown by not following suit; "cannot hold" from a seen card and from a void; "certainly holds" for the revealed called king and by elimination; unseen cards; own highest unseen card of a suit; each derived count.
8. **Soundness.** At every decision of 10,000 random deals: where F1 says void, the seat truly holds no card of that suit; where F2 says "cannot hold", it truly does not; where F3 says "certainly holds", it truly does.
9. The mask equals the engine's legal actions mapped to their slots. At least one slot is legal at every decision.
10. The critic features are the state plus the true hands of the three other seats in relative order, the hidden talon cards, the hidden discards and the true partner seat.
11. Targets: the partner target is the true partner's relative seat or "none"; the card-location target of every unseen card is its true place.
12. Reward: for every deal the four rewards sum to zero within 1e-6 and each equals (own entry − mean of the other three) / 50. One example is computed by hand.
13. No sequence exceeds 128 tokens in 100,000 random deals. The longest seen is reported.
14. Every feature value lies in 0 to 1.

**Vectorised environment**
15. The same base seed and the same actions give identical arrays, with 1 thread and with 12.
16. Tables are independent: changing the actions of one table changes nothing in another.
17. When a deal ends, that table's next observation is a new deal with the dealer one seat on. After 24 deals the table starts a new run.
18. Each deal's rewards and score parts are delivered once. The parts of each seat sum to its entry.
19. A random-policy run of 100,000 deals raises no error and no panic.

**Python module**
20. `maturin develop --release` builds the module on this PC's toolchain. Arrays have the documented shapes and types. The call releases the GIL: a Python thread keeps running during a long step.
21. Parity: 1,000 random deals driven from Python with seeded random legal actions give the same entries as a pure-Rust run with the same choices.
22. `env.py` passes the interface tests that `mock_env.py` passes (the shared tests of spec 007).
23. Environment-only speed, random policy, 1,024 tables: steps per second with 1 thread and with 12. The numbers go into `docs/progress.md`. No threshold; they bound G3 from above.

**Game record**
24. A finished deal exports a JSON record with: record version, encoding version, engine version, seed, dealer, preset and toggles, and per decision the acting seat, the legal slots and the chosen slot; then the entries and their parts. Replaying the record reproduces the same entries.
25. A record with an illegal action is refused, with the index of the decision in the message.
26. The record of a seeded deal is stable: a golden file in the tests.

**Quality gates**
27. `cargo fmt --check`, `cargo clippy --locked --all-targets -- -D warnings`, `cargo test --locked` pass for both crates; `pytest` passes; both workflows are green.
28. The engine crate has no Python dependency (`cargo tree` shows no `pyo3`). All earlier tests still pass.

## Files expected to change

New: `src/engine/src/encode/` (tokens, state, inference, critic, reward), `src/engine/src/vecenv.rs`, `src/engine/src/record.rs`, tests; `src/train/tarok_env/` (Cargo.toml, src/lib.rs, pyproject.toml, .cargo/config.toml); `src/train/tarok_train/env.py`; tests in `src/train/tests/`.
Changed: `src/engine/Cargo.toml` (serde, serde_json, rayon, pinned), `src/engine/src/game.rs` (event log), `.github/workflows/engine.yml`, `.github/workflows/train.yml`, `src/train/requirements.txt` if needed, both README files, this spec.

Because it touches both folders, this spec runs in one session after spec 005 is committed, not in parallel with other engine work.

## Commits

(a) event log, tokens, state blocks A–E and G, mask, tests 1–5, 9, 13, 14; (b) inference block F, critic, targets, reward, tests 6–8, 10–12; (c) vectorised environment, tests 15–19; (d) bindings and wrapper, tests 20–23; (e) record, tests 24–26. Each with `Spec: docs/specs/006-bindings-and-environment.md`.

## Open points for the user

- **A. What the record stores per decision.** (a) acting seat, legal slots, chosen slot; observations are rebuilt by replay, recommended: a record stays a few kilobytes. (b) the full observation too, as D59 literally says: about 1,400 numbers per decision. With (a), the agent's outputs (values, beliefs) are an optional field added by whoever has a model.
- **B. Where the binding crate lives.** (a) `src/train/tarok_env`, recommended: the engine stays free of Python and compiles to WASM unchanged. (b) inside `src/engine` behind a feature flag.
- **C. Array types.** (a) plain 32-bit floats for `state` and `critic`, recommended to start: simple, and measured in criterion 23. (b) packed bits, unpacked on the GPU: less data to copy, more code. Revisit only if the copy shows up in the smoke run.
- **D. Seeds of the tables.** Proposal: deal seed = a hash of (base seed, table index, run number, deal number), so any deal of any run can be dealt again from four numbers.
