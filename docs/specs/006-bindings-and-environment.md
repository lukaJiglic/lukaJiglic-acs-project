# 006 — Observation encoding, vectorised environment, Python bindings, game record

Status: **done** (2026-11-12). Approved on 2026-11-10 (D120), executed 2026-11-11, committed on the user's go-ahead (D123) as `8f24dbb` and `8cb2bb0`. The three workflows are green on `8cb2bb0`; the `train` run built the module on Linux and passed 88 tests. See "Results" at the end.

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
18. Each deal's rewards and `reward_parts` (4 seats × 10 parts, D121) are delivered once. The ten parts of each seat add up to its reward within 1e-6.
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

## Answers to the open points (D120)

- A. The record stores the acting seat, the legal slots and the chosen slot per decision. Observations are rebuilt by replay. Agent outputs are an optional field.
- B. The binding crate is `src/train/tarok_env`.
- C. `state` and `critic` are 32-bit floats. Packing is revisited only if the copy shows up in the smoke run.
- D. Deal seed = a hash of (base seed, table index, run number, deal number).

## Results

Executed on 2026-11-11 by a parallel session (D109). Claude Code, model `claude-fable-5-2`. Nothing is staged or committed. The tests were written from the criteria before the code, step by step in the order (a) to (e) of "Commits". Local runs used rustc 1.94.1 (`stable-x86_64-pc-windows-gnu`), plain `cargo`, Python 3.12.10 and maturin 1.16.0 in the repo's `.venv`.

### The readings of the contract (Batch 12) and where each is built

The main session's answer of 2026-11-11: build each recommended reading in one marked place; the user rules on them as a batch. Each place carries a comment that starts with `Reading B12-n (Batch 12, open)`. `docs/encoding.md` is not edited. B12-9 to B12-11 were added by the review (items 9 and 10). Line numbers are those of the final sources.

| Item | Reading built | Marked place |
|---|---|---|
| B12-1 | G1 holds the one-hot of the Default preset's nine toggles (S15 a, S22 a, S25 a, S26 a, S28 b, S29 a, S31 a, S32 a of three, S33 a), not zeros | `src/engine/src/encode/mod.rs:171` (`SCORING_TOGGLES`) |
| B12-2 | Critic "true partner seat": the declarer's true partner, relative to the observer, like B6. All four zero when the declarer plays alone. | `src/engine/src/encode/critic.rs:58` |
| B12-3 | Partner target: 0 ("none") unless exactly one other seat is on the observer's side; else that seat's relative number, 1 to 3 | `src/engine/src/encode/critic.rs:83` |
| B12-4 | F5: every own card that no unseen card of its suit beats | `src/engine/src/encode/inference.rs:220` (`highest_unseen`) |
| B12-5 | F3: a taken king or trula card until it is played; once the discards are made, a taken tarok not shown as a discard, and every taken card if no discard was hidden; elimination | `src/engine/src/encode/inference.rs:165` |
| B12-6 | F6: most taroks = min(taroks the seat can still hold, its hand size, 12) / 12; card points unseen = the unseen cards counted as one pile, / 70 | `src/engine/src/encode/state.rs:252` and `:283` |
| B12-7 | Token seat: none (4) for the start and the six shown talon cards; the declarer for contract, called king, taken packet, shown discard | `src/engine/src/game.rs:399` (`log`) |
| B12-8 | Record seed as a decimal string; the mixing function of the deal seed; the first dealer of a run | `src/engine/src/record.rs:68`, `src/engine/src/vecenv.rs:58` and `:69` |
| B12-9 | C4 "last bid per seat": a seat's last real bid, kept after it passes; nothing if it never bid. **Changed by the review**: before, a pass replaced the bid. | `src/engine/src/encode/state.rs:188` |
| B12-10 | F4: the whole shown talon counts as seen, the packet the declarer took included, and so do the declarer's own discards. F4 is 0 for a taken card, so the card-location loss (unseen cards only) never covers one, although the other seats do not know whether it is in the declarer's hand or among the hidden discards. No code change. | `src/engine/src/encode/inference.rs:129` |
| B12-11 | B7 "the called king lies in the shown talon" stays 1 after the king is taken and after it is played. E3 counts and rounds each seat's pile on its own. E4's trick number is 0 before card play and stays 12/12 after the last trick. No code change. | `src/engine/src/encode/state.rs:158`, `:210`, `:215` |

Notes the main session asked for:

- **B12-2: `env.py` and the mock differ here.** The mock's four "true partner" features hold the acting seat's own team-mate (the same number as its partner target). The engine's hold the declarer's partner. They agree only when the declarer looks. When the partner looks, the mock names the declarer and the engine names the partner itself (seat 0). When an opponent looks, the mock names the other opponent and the engine names the declarer's partner. The learner treats the critic features as opaque, so nothing in it depends on this.
- **B12-3: the learner's coding is used as it is.** `contract.py` has `PARTNER_NONE = 0` and classes 1 to 3 for the relative seat; the engine's `PARTNER_NONE` is 0 and `ppo.partner_loss` skips it. An opponent in a two-against-two game has class "the other opponent". "None" is the target for every seat before the teams are fixed, for a declarer who plays alone and for each of its three opponents. `c11_partner_target_and_card_location_target` tests each of these cases.
- **B12-5: criterion 8 covers every source.** `c08` now counts the "certainly holds" bits it checked by source and asserts more than 20,000 of each. Last run, 10,000 deals: 1,031,736 bits; 216,690 a taken king or trula card, 389,193 a taken tarok, 110,367 another taken card, 315,486 by elimination. Wrong rules 13 and 15 below show the test fails when a source is wrong.
- **B12-9: what the 13 entries of a seat in C4 are.** Entry 0 is the pass; entries 1 to 12 are the twelve contracts in ladder order, the same numbering as the bid slots and as blocks B2 and C1. Under this reading entry 0 is never set: a seat that passed keeps its last bid, a seat that never bid has thirteen zeros, and block C3 holds the passes. So one of the 52 features is always 0 per seat. The other way to use it: set entry 0 for a seat that passed without ever bidding. That is not built; it is a question for the batch.
- **B12-8: the exact definitions**, for the contract:
  - Record: the field `seed` is the deal's 64-bit seed written in decimal as a JSON string, for example `"seed":"42"`. `Deal::from_seed(seed)` gives the cards (spec 002).
  - Deal seed, all arithmetic on unsigned 64-bit numbers with wrap-around:

    ```
    mix(z):  z = (z xor (z >> 30)) * 0xBF58476D1CE4E5B9
             z = (z xor (z >> 27)) * 0x94D049BB133111EB
             return z xor (z >> 31)
    K = 0x9E3779B97F4A7C15
    s = mix(base_seed + K)
    s = mix(s xor (table + 2 * K))
    s = mix(s xor (run + 3 * K))
    deal_seed = mix(s xor (deal + 4 * K))
    ```

    `mix` is the finishing step of SplitMix64. `table` counts from 0, `run` from 0, `deal` is the deal's number in its run, 0 to 23.
  - Dealer: the first dealer of run `run` at table `table` is seat `(table + run) mod 4`. The dealer of deal `deal` of that run is `(table + run + deal) mod 4`.
  - Frozen by `deal_seeds_are_frozen_and_differ` in `tests/vecenv.rs`. Six golden values, (base seed, table, run, deal) → deal seed:

    ```
    (0, 0, 0, 0)                 12321809464288559627
    (1, 0, 0, 0)                 12174098620933058904
    (0, 1, 0, 0)                 14815432490932255636
    (0, 0, 1, 0)                  4514656745954956636
    (0, 0, 0, 1)                 17960153663297965470
    (2^64 - 1, 1023, 77, 23)      1794999558585611450
    ```

    The definition above, computed again in plain Python, gives these six numbers. The same test checks that 40,000 neighbouring positions give 40,000 different seeds.

### Shifts against spec 005's API (wording only)

No criterion changed its meaning. Three shifts, reported before the code was written:

1. `Game` had no event log. It now keeps one (`PublicView::events()`), and the slots of its actions (`FullView::action_slots()`). Spec 006 names this under "In scope".
2. `ScoreParts` of spec 005 has eight parts. The contract has ten (D121). Klop and radelc are added as zeros in the encoding; the engine's type is unchanged.
3. `docs/encoding.md` leaves eight details open. They went to the main session before the code was written and are Batch 12. The recommended reading of each is built; see the table above.

### What was built

**Engine (`src/engine`)**

- `game.rs`: `Game` keeps a log of its public events (`Event`, `EventKind`: start, bid or pass, contract, called king, the six shown talon cards, taken packet, shown discard, played card with trick number and position) and the slots of its actions. New accessors: `PublicView::events()`, `PublicView::called_king_played()`, `PrivateView::is_called_partner()`, `FullView::actions()`, `FullView::action_slots()`.
- `encode/mod.rs`: the sizes of the contract as constants, the tables `EVENT_GROUPS`, `STATE_BLOCKS`, `ACTION_GROUPS`, `CRITIC_EXTRA_BLOCKS`, `SCORE_PARTS`, the function `relative(observer, seat)`, `observe(game, seat)` (one seat, own vectors) and `write_mask`.
- `encode/tokens.rs`: `event_id`, `write_tokens`. One token per public event: event id, relative seat, trick, position, index.
- `encode/state.rs`: `write_state`, the 1,382 features. Blocks A8, D, G2 and G3 stay zero (rules not built).
- `encode/inference.rs`: `knowledge(game, observer)`: unseen cards, shown voids, "cannot hold", "certainly holds". Only facts that follow with certainty. The rules are listed at the top of the file.
- `encode/critic.rs`: the 274 hidden features and the two targets.
- `encode/reward.rs`: `rewards(&Score)`: entries, their ten parts, the four rewards and the 4 × 10 reward parts.
- `vecenv.rs` (feature `vecenv`, on by default): `Tables` (N tables, writes into arrays of the caller) and `VecEnv` (owns its arrays). Runs of 24 deals, dealer one seat on per deal. `observe`, `step`, `state` / `set_state`, `run_random` (random policy inside the engine, for the benchmark). Threads from a `rayon` pool of its own.
- `record.rs`: `Record` with `of_game`, `to_json`, `from_json`, `replay`.
- `examples/record_deal.rs`: prints the record of a seeded deal. It wrote the golden file.

**Binding crate (`src/train/tarok_env`)**: PyO3 0.30.1 and `numpy` 0.30.0, pinned. The class `Tables` (`observe_into`, `step_into`, `run_random`, `get_state`, `set_state`, `position`) and the functions `deal_seed`, `first_dealer`, `play_deal`, `record_json`, `replay_record`. The sizes of the contract are module constants. `observe_into`, `step_into` and `run_random` release the GIL while the engine works; the other calls (`get_state`, `set_state`, `position`, the functions) are short and keep it. Every array must have exactly its documented shape and be C-contiguous. No rule and no encoding lives in this crate.

**Python (`src/train/tarok_train/env.py`)**: `EngineEnv(n_tables, seed, threads)` with the mock's interface (`observe`, `step`, `get_state`, `set_state`), a lock around its methods, `make(cfg_env, seed)` for `kind = "mock"` or `"engine"`, and the benchmark (`python -m tarok_train.env`).


### Criteria

Engine tests are in `src/engine/tests/`, Python tests in `src/train/tests/test_spec006.py`. A test name starts with the criterion number.

| # | Criterion | Test | Result |
|---|---|---|---|
| 1 | Exported sizes equal the tables of `docs/encoding.md` | `encoding_contract.rs`: `c01_exported_sizes_equal_the_tables_of_encoding_md`. Reads the document: the array shapes, the token fields, the event groups, the state blocks with their offsets, the action groups, the critic blocks, the reward scale and the ten score parts. | pass |
| 2 | Tokens of a scripted deal equal a hand-written list | `tokens.rs`: `c02_tokens_of_a_scripted_deal_equal_a_hand_written_list`; `c02_tokens_carry_the_named_contract_and_the_taken_packet` (three more scripted deals); `tokens_name_the_actions_that_were_taken` (2,000 random deals) | pass |
| 3 | Trick 0 and position 0 before card play; positions 1 to 4, tricks 1 to 12 | `tokens.rs`: `c03_trick_and_position_fields` | pass |
| 4 | Hidden discards are no tokens; a discarded tarok is one | `tokens.rs`: `c04_hidden_discards_are_no_tokens_and_a_discarded_tarok_is_one` | pass |
| 5 | Seat rotation | `state_features.rs`: `c05_seat_rotation`. 3,000 random moments, four observers each: ten per-seat blocks, B8, the public blocks, F1, the tokens' seat field, the critic's other hands, the card target. **F2, F3 and the last three of F6 are outside this test**: they hold what the observer knows, which differs from seat to seat, so the next observer's values are no rotation of this one's. Their seat order is tested by the hand-built positions of criterion 7 and by criterion 8 (wrong rule 3 below). | pass |
| 6 | No leak, 10,000 cases | `no_leak.rs`: `c06_no_leak_when_hidden_cards_change_places`. 30,000 cases drawn, four kinds of change; the test asserts at least 10,000 compared and a minimum per kind. Counts below. | pass |
| 7 | Each of F1 to F6 on a hand-built position | `inference.rs`: seven tests `c07_f1_…` to `c07_f6_…` (F3 has two: the taken king, and a king by elimination for a seat that is not the declarer) | pass |
| 8 | Soundness at every decision of 10,000 random deals | `inference.rs`: `c08_inference_is_sound_at_every_decision_of_10_000_random_deals`. All four observers at every decision. | pass |
| 9 | Mask equals the legal actions; one slot legal at every decision | `state_features.rs`: `c09_mask_equals_the_legal_actions` | pass |
| 10 | Critic features | `critic_targets_reward.rs`: `c10_critic_features_are_the_state_plus_the_hidden_information` | pass |
| 11 | Targets | `critic_targets_reward.rs`: `c11_partner_target_and_card_location_target` | pass |
| 12 | Reward: zero sum, formula, one example by hand | `critic_targets_reward.rs`: `c12_reward_is_zero_sum_and_relative_to_the_other_three`. Two written deals by hand, then 10,000 deals. | pass |
| 13 | No sequence above 128 tokens in 100,000 random deals | `tokens.rs`: `c13_no_sequence_exceeds_128_tokens_in_100_000_random_deals`; `longest_possible_sequence_of_this_slice` | pass; longest seen 66, longest possible 71 |
| 14 | Every feature in 0 to 1 | `state_features.rs`: `c14_every_feature_lies_in_0_to_1` (state and critic, all four observers, every decision of 2,000 deals) | pass |
| 15 | Same seed and actions give identical arrays with 1 thread and with 12 | `vecenv.rs`: `c15_…`; also `random_runs_do_not_depend_on_the_thread_count` | pass |
| 16 | Tables are independent | `vecenv.rs`: `c16_tables_are_independent` | pass |
| 17 | Next deal with the dealer one seat on; a new run after 24 deals | `vecenv.rs`: `c17_…`; `deal_seeds_are_frozen_and_differ` | pass |
| 18 | Rewards and parts once per deal; the ten parts add up within 1e-6 | `vecenv.rs`: `c18_rewards_and_parts_are_delivered_once_per_deal` | pass |
| 19 | 100,000 random deals without error or panic | `vecenv.rs`: `c19_a_random_policy_run_of_100_000_deals_raises_no_error` | pass |
| 20 | `maturin develop --release` builds here; shapes and types; the GIL is released | Build output below. `test_c20_the_module_exports_the_sizes_of_the_contract`, `test_c20_arrays_have_the_documented_shapes_and_types`, `test_c20_an_array_of_another_shape_or_memory_order_is_refused`, `test_c20_a_python_thread_keeps_running_during_a_long_step`, `test_c20_a_second_thread_waits_for_a_running_call` | pass |
| 21 | Parity of 1,000 deals driven from Python with a pure-Rust run | `test_c21_parity_of_1000_deals_driven_from_python_with_a_pure_rust_run`, `test_c21_a_record_made_in_python_replays_in_rust` | pass |
| 22 | `env.py` passes the mock's interface tests | Five tests `test_c22_…[mock]` and `[engine]`: the same test body runs on both. **Spec 007's own test file was not run on the engine**: it builds the mock directly. Five more tests run the learner on the engine: one update with each network, a resumed trainer equals an unbroken one, `make` gives either kind, and the bodies of spec 007's `test_c06_acting_and_update_see_byte_identical_actor_inputs` and `test_c16_a_fresh_rollout_has_probability_ratio_one_on_every_decision`, each for both networks. | pass |
| 23 | Environment-only speed, 1,024 tables, 1 and 12 threads | `python -m tarok_train.env`; `test_c23_the_benchmark_runs_and_reports_steps_per_second` checks that it runs. Numbers below. | measured |
| 24 | Record with all fields; replay gives the same entries | `record.rs`: `c24_…` | pass |
| 25 | A record with an illegal action is refused with the decision index | `record.rs`: `c25_a_record_with_an_illegal_action_…`; `c25_a_record_with_another_preset_toggle_seed_form_or_field_is_refused` | pass |
| 26 | Golden record of a seeded deal | `record.rs`: `c26_…`, file `tests/golden/record_seed_42.json` | pass |
| 27 | fmt, clippy, tests for both crates; pytest; both workflows green | Local runs below: all pass. **The workflows are not verified: nothing is pushed.** | local part pass |
| 28 | No `pyo3` in the engine's tree; earlier tests pass | `cargo tree` below; a CI step in `engine.yml` repeats it. All tests of specs 002 and 005 pass. | pass |

Criterion 22, how the shared tests were made: the interface tests of spec 007 in `test_spec007.py` build the mock directly, and that file is not mine to change. So spec 007's tests themselves did not run on the engine. The same interface checks are written once in `test_spec006.py` and run on both environments with `pytest.mark.parametrize`; two of spec 007's learner tests are copied there with the engine swapped in. The trainer tests replace `train.mock_env.make` with `env.make` for the length of the test (monkeypatch), because `train.py` builds only the mock.

### Check of the tests: wrong rules planted

One wrong rule at a time was put into the code, the whole engine suite was run (`cargo test --no-fail-fast --tests`), and the file was restored. For the parity test the Python module was rebuilt with the wrong rule and `test_spec006.py` was run. After all runs the sources were compared with a copy made before (`diff -rq`: equal) and the module was rebuilt from them.

26 wrong rules in 29 runs. 24 were caught. Two were not; both are explained below the table.

| # | For criterion | Wrong rule | Tests that failed |
|---|---|---|---|
| 1 | 5 | The state turns seats the wrong way (`relative(seat, observer)`) | `c05`, five `block_…` tests, `c17` |
| 2 | 5 | Played cards (A2) by absolute seat | `c05`, `block_a_…` |
| 3 | 5 | Block F lists the other seats in reverse order | first run: five `c07` tests and `c08`, **not `c05`**. `c05` was then extended (see below). Second run: `c05` too. |
| 4 | 5 | Token seats are absolute | `c05`, `c02`, `c04` |
| 5 | 5 | The critic lists the other hands in reverse order | `c05`, `c10` |
| 6 | 5 | The card target counts the seats backwards | `c05`, `c11` |
| 7 | 6 | B6 shows the true partner before the king is played | `c06`, `block_b6_and_b7_…`, `c10` |
| 8 | 6 | Every discard is logged as shown (in `game.rs`) | `c06`, `c02`, `c04` |
| 9 | 6 | Hidden discards are left out of the unseen cards (F4) | `c06`, `c07_f4` |
| 10 | 6 | "Cannot hold" is read from the true hands. Sound, but a leak. | `c06`, four `c07` tests. `c08` passes, as it must. |
| 11 | 8 | A tarok played on a led suit marks a tarok void | `c08`, five `c07` tests |
| 12 | 8 | The declarer cannot hold the packet it took | `c08`, two `c07` tests |
| 13 | 8 | Elimination ignores hidden discards | `c08`, two `c07_f3` tests |
| 14 | 8 | Elimination also before the talon is turned up | **none** |
| 15 | 8 | The declarer certainly holds every taken card | `c08`, `c07_f3_…` |
| 16 | 9 | A seat that is not to act gets the mask too | `c09`, `c06` |
| 17 | 9 | The pass slot is always open | `c09`, seven `vecenv` tests |
| 18 | 9 | Playing the pagat (slot 90) is never open | `c09`, seven `vecenv` tests |
| 19 | 12 | Reward: minus the mean of all four seats | `c12` |
| 20 | 12 | Reward: divided by 100 | `c12` |
| 21 | 12 | The mondfang part is left out | `c12` |
| 22 | 12, 18 | The rounding rest is not added to a part | **none** |
| 23 | 17, 21 | The dealer does not move within a run | `c17`, `c18`; from Python: `test_c21_parity_…` |
| 24 | 18, 21 | Entries are written one seat off | `c18`; from Python: `test_c21_parity_…` |
| 25 | 21 | The module's `first_dealer` is one seat off | `test_c21_parity_…` |
| 26 | 21 | The module's `play_deal` counts the choice from the top of the legal list | `test_c21_parity_…` |

Rows 23 and 24 are two runs each (the engine suite, then the module with pytest). Row 3 was run twice.

**Row 14, not caught: no change of behaviour in this slice.** Before the talon is turned up no card has been played, so no seat has shown a void, and every unseen card has three seats that may hold it. Elimination never finds a single holder there, with or without the guard. In all six contracts of this slice the talon is turned up before the first card. The guard starts to matter with contracts that never show the talon (klop, berač, solo brez; spec 010). The soundness test must be run again there.

**Row 22, not caught: the code was removed.** `rewards` added the rounding rest of a seat's ten parts to its largest part. Measured over 40,000 seat-deals: the largest gap between a reward and the sum of its parts was 2.4e-7 with the correction and 2.4e-7 without it (exactly equal in 80 % and 75 % of the cases). The criteria ask for 1e-6. The correction had no effect a test could see, so it is gone; the parts are now plain (own part − mean of the others' part) / 50.

**`c05` extended after row 3.** The test looked at the ten blocks with four seats and at the tokens. Blocks with three other seats (F1 to F3, the last three of F6, the critic's hands, the card target) were covered only by the hand-built tests of criterion 7 and by `c08`. `c05` now also compares F1 (public, so two of its three parts must be equal for the next observer), the critic's other hands and the card target. F2, F3 and F6 depend on what the observer knows and cannot be compared across observers.

### Review fixes (2026-11-11, after the D72 step 4 review)

The review found no leak, no unsound bit and no wrong reward, but six wrong rules that passed the whole suite, and defects in the binding and in this text. Each item below has its number from the review. For every test gap the test was written first, then the wrong rule was planted and the suite run; the table after the list gives the failing tests.

**A. Test gaps**

1. **Which taken card the declarer hid.** `c06` has a fourth kind of case: the same deal and the same actions, but the declarer puts down a card of the packet it took in one world and a card it was dealt in the other. Both cards may be seen ones (the packet lies open); what is hidden is which was put down. Every other seat's tokens, state and mask must be equal. 1,287 such cases in the last run; the test asserts at least 300. The reviewer's rule (a taken card is "certainly held" if the declarer truly holds it) now fails `c06`.
2. **Packet token.** `c02_tokens_carry_the_named_contract_and_the_taken_packet`: three scripted deals take the fourth of six, the third of three and the second of two packets. `tokens_name_the_actions_that_were_taken`: on 2,000 random deals every token but the start and the talon cards equals the action that was taken, in order; the test asserts that more than 500 deals take a later packet.
3. **Contract token.** The same two tests: the declarer wins with dve and names ena, dve and solo tri; on the random deals more than 200 name a higher contract than the winning bid.
4. **Third card in A3.** `block_a_cards_of_the_scripted_position` goes one card further: three cards on the table, the fourth seat to act; then an empty table after the trick.
5. **Void in the running trick.** `c07_f1_a_void_is_shown_by_not_following`: C plays a suit card on a tarok lead; D, who acts next in the same trick, sees C void in taroks in F1 and in F2.
6. **Preset and toggles on replay.** `c25_a_record_with_another_preset_toggle_seed_form_or_field_is_refused`: another preset, another option of a toggle, a missing toggle and an unknown toggle are each refused. The message names the preset or the toggle and what the engine plays.

**B. Binding**

7. **Shapes and memory order.** `tarok_env.Tables` checks every array before it writes: the exact shape and C order. A Fortran-ordered `state`, `tokens` of shape (n, 5, 128), `state` of shape (1382, n) and every other mismatch raise `ValueError`, for example "the array tokens has shape (4, 5, 128); the engine needs (4, 128, 5)". The arrays of `set_state` and the actions are checked the same way. Test: `test_c20_an_array_of_another_shape_or_memory_order_is_refused` (every array: one table too many, a strided view; the named cases; nothing written, nothing moved).
8. **Threads.** (a) `EngineEnv` holds a lock around `observe`, `step`, `entries`, `position`, `run_random`, `get_state` and `set_state`: a second thread waits for the running call. Test: `test_c20_a_second_thread_waits_for_a_running_call`. (b) The rule is written in the docstring of `env.py`, in the module text of the binding and in `src/train/README.md`: no other thread may read, write or resize the arrays during a call. What remains is in "Open issues".

**C. Contract**

9. **C4** is now the seat's last real bid, kept after a pass (B12-9; table and note at the top of Results). The hand-written test changed with it, and a new part of `block_c_bidding_of_the_scripted_position` checks every decision of 200 random deals for every observer, with more than 1,000 cases of a bid kept after a pass.
10. **B12-10 and B12-11** are marked in the code and in the table at the top. No code change.

**D. Robustness**

11. **Log limits.** A full event log or action history panics with a message that names the limit ("a deal has more than 128 public events (MAX_EVENTS in game.rs, the sequence limit of docs/encoding.md): the limit must be raised"). Three unit tests in `game.rs`: the logs take exactly their limits; one more event panics; one more action panics. `longest_possible_sequence_of_this_slice` now also asserts the limits against the longest deal and plays that deal to its end. A correction on the way: the longest deal has 64 actions, not 67 as a comment said (13 calls, the contract, the packet, 1 discard, 48 cards).
12. **`cargo test --no-default-features`** compiles and passes (169 tests): `tests/vecenv.rs` has `required-features = ["vecenv"]`. The `engine` workflow runs these tests instead of only building. Locally the extra run takes 24 seconds with the build up to date.
13. **`set_state`** refuses a state whose count of finished deals is not 24 × run + deal, and a run so large that the count leaves 63 bits. The message names the table and the numbers.
14. **Record.** The seed must be in the one canonical form (digits, no sign, no leading zero, at most 2^64 − 1): "+31", "031", " 31" and seven more forms are refused. An unknown field at the top or in a decision is refused when the text is read (`deny_unknown_fields`); the message names it. The `agent` field stays free-form.
15. **Actions in `EngineEnv.step`.** A float, bool or string array is a `TypeError` ("actions must be whole numbers"); it is no longer cut to integers. A wrong number of actions is its own `ValueError` ("step needs one action per table"). Neither counts as an illegal action. Test: `test_c22_step_refuses_actions_that_are_not_whole_numbers_or_not_one_per_table`.

**E. This text**

16. "What was built" now says which calls release the GIL: `observe_into`, `step_into`, `run_random`.
17. The row of criterion 22 says that spec 007's own tests were not run on the engine. The bodies of its c06 (byte-identical actor inputs) and c16 (probability ratio one) are tests on `EngineEnv` in `test_spec006.py`, each for the transformer and the MLP. All four pass.
18. The row of criterion 5 says that F2, F3 and the last three of F6 are outside `c05`, and why.
19. Two open issues added (the engine version, a long `run_random`), and a proposal for a stale-module check under "Questions for the user". It is not built.

**Wrong rules planted for the review items.** Same method as above. 22 planted, 22 caught.

| Item | Wrong rule | Tests that failed |
|---|---|---|
| 1 | A taken card is certain if the declarer truly holds it (the reviewer's rule; sound, but a leak) | `c06` |
| 2 | The taken-packet token always says packet 0 | `c02_tokens_carry_…`, `tokens_name_the_actions_…` |
| 3 | The contract token carries the winning bid | `c02_tokens_carry_…`, `tokens_name_the_actions_…` |
| 4 | The third card of the trick is not written to A3 | `block_a_cards_of_the_scripted_position` |
| 5 | Voids are read from finished tricks only | `c07_f1_a_void_is_shown_by_not_following` |
| 6 | The preset is not checked on replay | `c25_a_record_with_another_preset_…` |
| 6 | The option of a toggle is not checked on replay | `c25_a_record_with_another_preset_…` |
| 6 | Unknown toggles are accepted on replay | `c25_a_record_with_another_preset_…` |
| 7 | The binding does not check the shape of an array | `test_c20_an_array_of_another_shape_or_memory_order_is_refused` |
| 7 | The binding does not check the memory order of an array | `test_c20_an_array_of_another_shape_or_memory_order_is_refused` |
| 8 | `observe` does not take the lock | `test_c20_a_second_thread_waits_for_a_running_call` |
| 8 | `run_random` does not take the lock | `test_c20_a_second_thread_waits_for_a_running_call` |
| 9 | C4 holds the last call; a pass replaces the bid (the old rule) | `block_c_bidding_of_the_scripted_position` |
| 11 | A full event log stops silently | `game::tests::the_event_log_panics_when_it_is_full` |
| 11 | A full action history stops silently | `game::tests::the_action_history_panics_when_it_is_full` |
| 13 | `deals_done` is not compared with run and deal | `the_state_restores_an_environment_exactly` |
| 13 | The run is multiplied with wrap-around | `the_state_restores_an_environment_exactly` |
| 14 | Any text that parses is a seed | `c25_a_record_with_another_preset_…` |
| 14 | Unknown fields of a decision are ignored | `c25_a_record_with_another_preset_…` |
| 14 | Unknown top-level fields are ignored | `c25_a_record_with_another_preset_…` |
| 15 | Float actions are cut to whole numbers | `test_c22_step_refuses_actions_…` |
| 15 | The number of actions is not checked | `test_c22_step_refuses_actions_…` |

The three new Python tests of items 7, 8 and 15 were also run against the module as it was before the fixes: all three failed.

**The two workflows on Linux, by reasoning.** Checked in the PyO3 0.30.1 source (`pyo3-build-config`, `is_linking_libpython_for_target`): on Linux a build links `libpython` unless `PYO3_BUILD_EXTENSION_MODULE` is set or the `extension-module` feature is on. maturin sets the variable, so the wheel that `pip install ./tarok_env` builds does not link it. `cargo test --locked` in the binding crate does: it needs a shared `libpython3.12` at link time and at start. Inferred, not checked: the Python of `actions/setup-python` is built with a shared library and the action puts its folder on the library path, so the step should link and run. It stays an open issue until the push. If it fails there, the smallest fix is to drop `cargo test` for this crate in `train.yml` (it has no Rust tests) and keep `fmt` and `clippy`.

### Measured numbers

**Longest token sequence (criterion 13).** 66 tokens in 100,000 random deals (seed 5408). The same deal has the most actions, 64; the room in a game is 128 events and 96 actions. The longest possible in this slice is 71: solo ena after the longest auction. Start 1, calls 13, contract 1, shown talon 6, taken packet 1, shown discard 1, cards 48. `longest_possible_sequence_of_this_slice` walks every auction and contract choice and takes one shown discard per card taken. The limit is 128.

**No leak (criterion 6), last run.** 30,000 cases drawn, 15,415 compared: 11,597 two other hands, 2,027 a hand and the hidden discards, 504 a hand and the talon before it is turned up, 1,287 which card the declarer hid (a card of the packet it took in one world, a card it was dealt in the other; added by the review). 2,895 changes did not give a legal deal with the same actions and were not compared; the rest of the 30,000 had no card to change. The test asserts at least 10,000 compared, at least 5,000, 300, 300 and 300 per kind. The seed comes from the clock and is printed; `TAROK_TEST_SEED` replays it.

**Soundness (criterion 8).** 10,000 random deals, four observers at every decision. Last run: 4,224,903 void bits, 223,113,934 "cannot hold" bits and 1,031,736 "certainly holds" bits checked against the true hands. The test asserts more than 100,000 void bits, more than 100,000 "certainly holds" bits and more than 20,000 from each source of "certainly holds" (taken king or trula, taken tarok, other taken card, elimination), so it cannot pass on empty features.

**Speed of the environment alone (criterion 23).** 1,024 tables, random policy, 5 seconds per number, 3 repeats, median (lowest to highest). A step is one decision at one table: the action is applied and the next observation of that table is written (tokens, state, mask, critic, targets). AMD Ryzen 5 5600, 6 cores, 12 threads. Nothing else ran on the machine besides the desktop and this session; no training, no other build.

| Threads | Policy in Python (NumPy picks from the mask) | Policy in the engine (`run_random`) |
|---|---|---|
| 1 | 396,822 steps/s (375,349 to 397,639) | 564,587 steps/s (522,550 to 580,034) |
| 12 | 976,682 steps/s (970,308 to 984,508) | 3,213,112 steps/s (3,073,856 to 3,292,095) |

This is the run after the review fixes (shape checks in the binding, the lock in `EngineEnv`). Two earlier runs of the same command the same day gave 415,927, 594,237, 1,014,416, 3,278,107 and 369,546, 550,646, 968,187, 3,285,728. The main session measured 0.97 million through Python with 12 threads on its own. Read them as: about 0.4 and 1.0 million steps per second through Python, about 0.55 and 3.3 million inside the engine. With 12 threads the Python side (picking the actions and copying the results, one thread) is the limit, not the engine. Both columns write the full observation at every step. These numbers bound G3 from above; the environment is not what limits the pipeline.

Command:

```
cd src\train
python -m tarok_train.env --tables 1024 --threads 1 12 --seconds 5
```

**Test counts.**

| Suite | Tests | Result |
|---|---|---|
| `src/engine`, `cargo test --locked` | 180 (20 unit, 157 in 23 test files, 3 doc) | all pass. New in this spec: 44 tests in eight test files and 6 unit tests. |
| `src/engine`, `cargo test --locked --no-default-features` | 169 | all pass; the 11 tests of `tests/vecenv.rs` are left out |
| `src/train/tarok_env`, `cargo test --locked` | 0 | builds; the crate is tested from Python |
| `src/train`, `pytest -m "not slow"` (as CI) | 96 passed, 5 deselected | 175 s; 26 are `test_spec006.py` |
| `src/train`, `pytest -m slow` | 5 passed, 96 deselected | 75 s; spec 007's tests, unchanged |

### Command output (trimmed)

The runs after the review fixes.

Engine, run from `src\engine`:

```
> cargo --version
cargo 1.94.1 (29ea6fb6a 2026-03-24)
> cargo fmt --check
(no output)
> cargo clippy --locked --all-targets -- -D warnings
    Finished `dev` profile [optimized + debuginfo] target(s) in 2.07s
> cargo test --locked
     Running unittests src\lib.rs            test result: ok. 20 passed; 0 failed
     Running tests\actions.rs                test result: ok. 6 passed; 0 failed
     Running tests\bidding.rs                test result: ok. 13 passed; 0 failed
     Running tests\cards_and_counting.rs     test result: ok. 7 passed; 0 failed
     Running tests\cardset.rs                test result: ok. 5 passed; 0 failed
     Running tests\critic_targets_reward.rs  test result: ok. 3 passed; 0 failed; finished in 1.05s
     Running tests\deal.rs                   test result: ok. 5 passed; 0 failed
     Running tests\encoding_contract.rs      test result: ok. 1 passed; 0 failed
     Running tests\game_play.rs              test result: ok. 3 passed; 0 failed
     Running tests\inference.rs              test result: ok. 8 passed; 0 failed; finished in 8.26s
     Running tests\king_call.rs              test result: ok. 5 passed; 0 failed
     Running tests\legal_cards.rs            test result: ok. 7 passed; 0 failed
     Running tests\no_leak.rs                test result: ok. 1 passed; 0 failed; finished in 0.37s
     Running tests\record.rs                 test result: ok. 4 passed; 0 failed
     Running tests\scoring.rs                test result: ok. 26 passed; 0 failed
     Running tests\state_features.rs         test result: ok. 9 passed; 0 failed; finished in 1.48s
     Running tests\talon.rs                  test result: ok. 9 passed; 0 failed
     Running tests\tokens.rs                 test result: ok. 7 passed; 0 failed; finished in 0.41s
     Running tests\trick_winner.rs           test result: ok. 10 passed; 0 failed
     Running tests\vecenv.rs                 test result: ok. 11 passed; 0 failed; finished in 3.68s
     Running tests\views.rs                  test result: ok. 3 passed; 0 failed
     Running tests\whole_deals.rs            test result: ok. 4 passed; 0 failed
     Running tests\whole_games.rs            test result: ok. 7 passed; 0 failed
     Running tests\written_deals.rs          test result: ok. 3 passed; 0 failed
   Doc-tests tarok_engine                    test result: ok. 3 passed; 0 failed
(180 tests, 0 failed; 32 seconds with the build up to date)
> cargo test --locked --no-default-features
(169 tests, 0 failed; tests\vecenv.rs is left out; 24 seconds)
> cargo test --locked --test tokens --test no_leak --test inference -- --nocapture
longest token sequence in 100,000 random deals: 66 (seed 5408)
no-leak cases compared: 15415 (two hands 11597, hand and hidden discards 2027, hand and unshown talon 504, which card the declarer hid 1287); 2895 changes were not a legal deal
certainly-holds bits checked: 1031736 (taken king or trula 216690, taken tarok 389193, other taken card 110367, by elimination 315486); void bits 4224903; cannot-hold bits 223113934
> cargo tree --locked
tarok-engine v0.1.0
├── rand_chacha v0.10.0
├── rayon v1.12.0
├── serde v1.0.229
└── serde_json v1.0.154
[dev-dependencies]
└── proptest v1.12.0
("pyo3" appears 0 times in the whole tree)
```

Binding crate, run from `src\train\tarok_env` with the venv active:

```
> cargo fmt --check
(no output)
> cargo clippy --locked --all-targets -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.64s
> cargo test --locked
test result: ok. 0 passed; 0 failed
> maturin develop --release
📡 Using build options locked from pyproject.toml
📦 Built wheel for CPython 3.12 to ...\tarok_env-0.1.0-cp312-cp312-win_amd64.whl
🛠 Installed tarok-env-0.1.0
```

A build from scratch of the module takes about 15 seconds.

Python, run from `src\train` with the venv active:

```
> set TAROK_REQUIRE_ENV=1
> python -m pytest -m "not slow" -q
96 passed, 5 deselected in 174.64s (0:02:54)
> python -m pytest tests\test_spec006.py -q
26 passed in 25.63s
> python -m pytest -m slow -q
5 passed, 96 deselected in 75.20s (0:01:15)
> python -m tarok_train.env --tables 1024 --threads 1 12 --seconds 5
engine 0.1.0, 1024 tables, 5.0 s per number, 3 repeats
a step is one decision at one table; median (min to max)
  1 threads, random policy in Python           396,822 steps/s (375,349 to 397,639)
  1 threads, random policy in the engine       564,587 steps/s (522,550 to 580,034)
 12 threads, random policy in Python           976,682 steps/s (970,308 to 984,508)
 12 threads, random policy in the engine     3,213,112 steps/s (3,073,856 to 3,292,095)
```

### Files changed

Nothing is staged. `git status` shows only these, plus the main session's `rust-toolchain.toml.disabled`.

Changed (9): `.github/workflows/engine.yml`, `.github/workflows/train.yml`, `src/engine/Cargo.toml`, `src/engine/Cargo.lock`, `src/engine/README.md`, `src/engine/src/game.rs`, `src/engine/src/lib.rs`, `src/engine/tests/common/mod.rs`, `src/train/README.md`. And this spec file.

New (26):

- `src/engine/src/encode/`: `mod.rs`, `tokens.rs`, `state.rs`, `inference.rs`, `critic.rs`, `reward.rs`
- `src/engine/src/vecenv.rs`, `src/engine/src/record.rs`, `src/engine/examples/record_deal.rs`
- `src/engine/tests/`: `encoding_contract.rs`, `tokens.rs`, `state_features.rs`, `inference.rs`, `no_leak.rs`, `critic_targets_reward.rs`, `vecenv.rs`, `record.rs`, `common/encoding.rs`, `golden/record_seed_42.json`
- `src/train/tarok_env/`: `Cargo.toml`, `Cargo.lock`, `pyproject.toml`, `.cargo/config.toml`, `src/lib.rs`
- `src/train/tarok_train/env.py`, `src/train/tests/test_spec006.py`

Not changed: `model.py`, `ppo.py`, `rollout.py`, `mock_env.py`, `contract.py`, `train.py`, `docs/encoding.md`, `src/train/requirements*.txt`.

New dependencies, all pinned exactly, lock files in step: `serde` 1.0.229, `serde_json` 1.0.154, `rayon` 1.12.0 (engine); `pyo3` 0.30.1, `numpy` 0.30.0 (binding crate); `maturin` 1.16.0 as the build tool in `pyproject.toml`. No `unsafe` in the engine crate (`#![forbid(unsafe_code)]`). No code from other tarok engines.

### How a seat's view is kept free of hidden cards

The token and state writers read `game.public()` and `game.private(observer)` only. `game.full()` is read in `critic.rs` (critic features and targets) and nowhere else in `encode/`. Criterion 6 tests this from outside; the wrong-rule checks above (rows 7 to 10) show the test fails when a writer reads the truth.

### Readings and deviations

The readings of the contract (B12-1 to B12-11) are listed at the top of Results with their marked places.

Deviations from the spec's text and additions the spec does not name:

1. **Test builds are optimised.** `[profile.dev] opt-level = 2` in `src/engine/Cargo.toml`. Without it the soundness test alone took 215 seconds. Overflow checks and debug assertions stay on. Compiling takes longer.
2. **A feature `vecenv`** (default on) holds the thread pool. `cargo test --no-default-features` builds and tests the crate without `rayon`. Reason: a browser build has no threads. The WASM build itself was not tried (out of scope, and it needs a rustup target).
3. **Two types for the environment.** `Tables` writes into arrays of the caller, so NumPy arrays are filled without a copy. `VecEnv` owns its arrays and is used by the Rust tests. The spec names one module, not the types.
4. **The observation arrays of `EngineEnv` are reused.** `observe()` returns the same NumPy arrays on every call; `step` overwrites them. The mock returns new arrays. The trainer copies what it keeps before it steps, so its tests pass. `done`, `reward` and `parts` are fresh copies.
5. **`run_random`** in the engine and in the module: steps with a random legal action chosen in Rust. Used by the GIL test of criterion 20 and for the second benchmark number; tested by `random_runs_do_not_depend_on_the_thread_count`. Criterion 19 does not use it: that test picks its actions from the mask.
6. **Extra functions in the module** for the tests: `play_deal`, `record_json`, `replay_record`, `deal_seed`, `first_dealer`.
7. **The record has an optional field `agent`** (any JSON), as D120 A allows. `preset` is `"default"` and `toggles` holds the nine toggle values of B12-1.
8. **`ScoreParts` of the engine still has eight parts.** Klop and radelc are added as zeros in `encode/reward.rs`.
9. **`entries`** is a fourth result array of a step (4 integers per table, the paper entries). `EngineEnv.entries` gives it. The mock has no such array.
10. **`tests/golden/`** and **`examples/record_deal.rs`** are new paths under `src/engine`.
11. **Criterion 23 says the numbers go into `docs/progress.md`.** That file is the main session's (D109). The numbers are here.
12. **`tests/vecenv.rs` needs the feature `vecenv`** (`required-features` in `Cargo.toml`), and the `engine` workflow tests the crate without the feature instead of only building it (review item 12).
13. **A full event log or action history is a panic**, not a silent stop (review item 11). The limits are 128 events and 96 actions; the longest deal of this slice has 71 and 64.

One finding about the data, not a deviation: **a seat can decide twice on the same token sequence.** The declarer puts down two or three cards one at a time, and a hidden discard is no token. Between two such decisions only the state token differs (hand, own discards). The mock never does this. The trainer's tests pass on the engine; a model that reads only the tokens would see two equal inputs. `test_c22_a_seats_token_sequence_only_grows_within_a_deal` now asserts "never shorter, the old tokens unchanged" and that the engine has such repeats.

### Open issues

1. **CI is not verified.** Nothing is pushed. `engine.yml` and `train.yml` were changed and parse as YAML; whether the Linux build of the module, `cargo test` of the binding crate on Linux and the 40-minute limit hold is seen only after a push. The reasoning about linking on Linux is at the end of "Review fixes". The `engine` workflow now runs the tests a second time without the feature `vecenv`.
2. **The trainer still starts the mock.** `train.py` calls `mock_env.make`, and its config check accepts only the mock's `[env]` keys. `env.make` exists and is tested; the switch is in `train.py`, which this spec may not touch (it is in spec 008's scope).
3. **The binding crate has no Rust tests of its own** (`cargo test --locked` there runs 0 tests). Its behaviour is tested from Python.
4. **`maturin develop` installed `tarok-env` into the repo's `.venv`.** After any change in `src/engine` the module must be built again, or Python runs the old engine. The README says so.
5. **Test time.** `cargo test` takes about 30 seconds with the build up to date and about 45 seconds after a change in the library.
6. **The longest sequence** is 71 tokens at most in this slice. Announcements and kontra (spec 009) add tokens; the limit of 128 has to be checked again there. A deal that passes it now panics with a message that names the limit.
7. **How often the F2 and F3 bits are set was not measured.** Criterion 8 shows they are never wrong; it does not show they are complete. A fact the rules in `inference.rs` do not derive stays unset.
8. **A stale installed module is not detected.** `tarok_env.ENGINE_VERSION` is the crate version, always "0.1.0". After a change in `src/engine` without a rebuild, Python runs the old engine and nothing says so. The sizes of the contract are compared at start (`_check_contract`), the rules are not. A proposal is under "Questions for the user".
9. **A long `run_random` cannot be interrupted.** It is one call into Rust; Ctrl+C acts only when it returns. The benchmark uses 50 steps per call. A huge step count blocks the process until it is done.
10. **The lock of `EngineEnv` does not cover arrays a thread already holds.** A thread that keeps the dict from `observe()` and reads it while another thread steps still sees rows of two different steps (the reviewer measured this before the lock: `critic[:, :1382] != state` in 503 of 504 reads during `run_random`). The rule is documented; the code cannot enforce it without copying every observation. One thread should drive one environment.
11. **A second thread that calls `tarok_env.Tables` directly**, not through `EngineEnv`, during a running `step_into` or `run_random` gets a `RuntimeError` from PyO3 for every call on that object (measured: "Already mutably borrowed" for `position` and `get_state`, "Already borrowed" for `step_into`). It can still read the arrays. `Tables` is the low-level class; `EngineEnv` is the one to use.

### Questions for the user

1. Batch 12, items B12-1 to B12-11: the readings at the top of Results. Each is built as recommended; another ruling is a small edit in the marked place. For B12-9 also: should entry 0 of a seat's thirteen in C4 be set when the seat passed without ever bidding? (a) no, it stays 0 and C3 holds the passes (built); (b) yes. Recommended: (a). C3 already says it, and the meaning "last real bid" stays simple.
2. Deviation 1: (a) optimised test builds, as now; (b) the long tests behind `--release` or `--ignored`. Recommended: (a). One command runs every test, locally and in CI, and the checks for overflow stay on.
3. Deviation 4: (a) `EngineEnv.observe()` keeps reusing its arrays, the caller copies what it keeps; (b) it returns new arrays like the mock, one copy of about 14 MB per step at 1,024 tables. Recommended: (a). The trainer already copies, and the docstring and the README say so. (b) is safer for new callers.
4. The repeated token sequence at hidden discards: (a) leave it, the state token differs; (b) add a token "the declarer put a card down" without the card, which changes `docs/encoding.md` (one new event id). Recommended: (a). The model reads the state token at every decision. Look again if the smoke run shows poor discards.
5. A check for a stale module. (a) Build it: a build script in `src/train/tarok_env` hashes the engine's source files and `Cargo.lock` at build time, the module exposes the hash, and `EngineEnv` (and one test) compares it with a hash of the files on disk and refuses to start on a mismatch; where the sources are not on disk the check is skipped. About 40 lines. (b) Leave it to the README rule "build again after every change". Recommended: (a), built with spec 008 before the first run. A stale module would train on old rules without any sign. Not built now, as asked.
