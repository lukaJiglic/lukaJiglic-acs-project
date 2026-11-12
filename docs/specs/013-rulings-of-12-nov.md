# 013 — Three rulings of 12 November in the engine

Status: **done** (2026-11-12). Executed and reviewed 2026-11-12, committed on the user's go-ahead (D134) as `ce175f6`. The three workflows are green on it: `engine` in 3 min 2 s, `train` with 88 tests passed against the module built from these sources. The criteria restate the user's rulings D125, D126, D131 and D133. Reading C12 is decided (D132) and is built in one marked place.

Tool and model: Claude Code 2.1.306, `claude-fable-5-2`. Session `engine-core`.

## Goal

The engine plays and scores three points the way the user's table does:

1. A valat does not cancel a lost mond (D125).
2. Pagat ultimo follows the side that takes the last trick (D126).
3. Škis, mond and pagat in one trick: the pagat wins only if it falls last (D131).

## In scope

`src/engine`: the trick winner, `score.rs`, their tests, the README. The crate version. Notes in specs 002 and 005 that point here. The golden record if its content changes.

## Out of scope

Toggles: the engine still has none and plays the Default (S40 a, S22 a). Announced bonuses (flow B). Klop and berač (flow C). The Python side, apart from rebuilding the module for the parity test.

## Acceptance criteria

**Valat and the mond (D125)**
1. One side takes all twelve tricks and a player lost the mond. The mond's player writes −20 next to the ±250, in each of the four cases: (i) the declarer's side takes all, an opponent loses the mond to the škis; (ii) the declarer's side takes all, one of the two loses the mond to the other's škis; (iii) the opponents take all, the declarer loses the mond or leaves it in a packet not taken; (iv) the opponents take all, one of them loses the mond to another opponent's škis.
2. In those deals the score parts are: valat ±250, mondfang −20 for the mond's player only, game, difference and the other bonuses zero.
3. A valat without a lost mond scores as before (criterion 41 of spec 005).

**Valat and the packets not taken (D133)**
16. The declarer's side takes all twelve tricks and the mond lies in a packet the declarer did not take: +250 and no mondfang for anyone. The user's words: "In case of valat, the declarer gets the packet he did not pick so he did not lose the mond". Tested with the declarer alone and with a partner. When the opponents take all twelve, the packet stays with them and the declarer still writes −20 (criterion 1, case iii). Added on 2026-11-12 after the review pass.

**Pagat ultimo (D126)**
4. The pagat is played to the last trick. The side of the pagat's player takes the trick, with the pagat or with another card: that side wins the bonus. The declarer's side writes +25 when the pagat's player is on it and −25 when the player is an opponent.
5. The other side takes the trick: the pagat's side loses the bonus. The declarer's side writes −25 when the pagat's player is on it and +25 when the player is an opponent.
6. Tested cases: the declarer's pagat wins; the partner's pagat taken by the declarer's higher tarok (+25); the declarer's pagat taken by the partner's tarok (+25); an opponent's pagat taken by another opponent (−25); the declarer side's pagat taken by an opponent (−25); an opponent's pagat taken by the declarer's side (+25); a solo contract, where the three opponents are one side.
7. The pagat is not in the last trick: 0, as before.

**Trula in one trick (D131)**
8. Škis, mond and pagat are in one trick. The pagat wins the trick when it was played after both the škis and the mond (reading C12, decided by the user in D132; in one marked place). Otherwise the škis wins it, as the highest tarok.
9. Tested with every order of the three cards and the fourth card first, between and last; the fourth card a suit card and another tarok.
10. The mond's player writes −20 in a trula trick whoever wins it (S22 a stays; when the škis wins, it is the plain mondfang).

**Tests of the tests**
11. Each old rule is planted once and fails at least one test: a valat cancels the mondfang; the pagat itself must take the last trick; the pagat wins the trula in any order; the pagat wins only as the fourth card (the other reading of C12).

**Consistency**
12. Rewards still add up to zero per deal and the ten parts of a seat add up to its reward (the property tests of spec 006 pass unchanged).
13. The crate version is raised, because the rules changed and a game record names the engine version. The golden record is written again only if needed; what changed in it is listed.
14. `src/engine/README.md` states the three rules. Spec 002 (criterion 17) and spec 005 (criteria 39 and 41, reading 2) get one line each that points here. Their old text stays.

**Quality gates**
15. `cargo fmt --check`, `cargo clippy --locked --all-targets -- -D warnings`, `cargo test --locked` and `cargo test --locked --no-default-features` pass for the engine; format, lints and tests pass for the binding crate; after the module is built again, `pytest tests/test_spec006.py` passes.

## Checks

Criteria 1–10, 12: `cargo test`. Criterion 11: planted in a scratch copy outside the repo, table in Results. Criteria 13, 14: by reading. Criterion 15: the commands' output in Results.

## Files expected to change

`src/engine/src/` (trick winner, `score.rs`), `src/engine/tests/` (`trick_winner.rs`, `scoring.rs`, written deals if their entries change), `src/engine/Cargo.toml`, `Cargo.lock`, `src/engine/README.md`, `src/engine/tests/golden/` if needed, one line each in `docs/specs/002-engine-core.md` and `docs/specs/005-deal-flow-a.md`, this spec. `src/train/tarok_env/Cargo.lock` if the engine version is named there.

## Commits

One commit: `fix(engine): valat keeps the mondfang, pagat ultimo by side, trula only when the pagat falls last (spec 013)`. Made by the main session after the review pass and the user's read of the diff (D72, D109).

## Open points for the user

- **C12.** "Falls last": (a) after both the škis and the mond, a fourth card may follow; (b) only as the fourth card of the trick. Answered with (a) on 2026-11-12 (D132). (a) is what is built.

## Results

Executed on 2026-11-12 by the session `engine-core` (Claude Code, `claude-fable-5-2`). Nothing is staged or committed. Tests were written from the criteria first and run against the old code, then the code was changed. With the tests as they are after the review, the code of HEAD fails 16 of the 182 tests, the same 16 in every run: `c17`, both `c39_` tests, `c42`, the five `d125_` tests, the `d133_` test, and six tests that compare the engine with the tests' own second versions of the rules (`c19`, `c21`, `c29`, `c47`, `winner_matches_the_reference_on_random_tricks`, `score_matches_the_second_scorer_on_random_deals`). The first version of this line said 12 of 180; one of those twelve, `winner_matches_the_reference_on_random_tricks`, depended on the clock seed and passed in the reviewer's run. See "Review fixes", items 3 and 4.

### What changed

- `src/trick.rs`: škis, mond and pagat in one trick. The pagat wins when it was played after both the škis and the mond. Otherwise the highest tarok wins as in every trick, which is the škis. The condition is one line, in `pagat_falls_last`, marked "Reading C12".
- `src/score.rs`: pagat ultimo is signed by the side that takes the last trick. The mondfang is written outside the valat branch, so a valat keeps it.
- Crate version 0.1.0 to 0.2.0: `Cargo.toml`, `Cargo.lock`, the `tarok-engine` entry in `src/train/tarok_env/Cargo.lock`.
- Tests: `tests/trick_winner.rs` (`c17`), `tests/scoring.rs` (`c39`, `c42`, five `d125_` tests in place of the five `reading2_` tests), and the tests' own second versions of the rules: `reference_winner` in `tests/common/mod.rs` and `tests/common/second_scorer.rs`.
- After the user's ruling D133 (criterion 16): `talon_rest_won` in `src/game.rs` is also true when the declarer's side took all twelve tricks; the piles of the full view, the card points and the talon mondfang follow from it. Comments in `src/score.rs`. The `d133_` test in `tests/scoring.rs`, the second scorer, and the test of criterion 21 of spec 005 in `tests/talon.rs`. One line each at criteria 21 and 44 of spec 005.
- After the review: two more tests in `tests/scoring.rs`, fixed seeds in `run_cases` (`tests/common/mod.rs`), a new `winner_matches_the_reference_on_random_tricks`, one doc line in `src/record.rs`, one doc comment in `tests/whole_games.rs`. See "Review fixes".
- `src/engine/README.md`: a section "Rules of the user's table". One line in spec 002 (criterion 17) and three in spec 005 (criteria 39 and 41, reading 2).

### Criteria

| # | Criterion | Status | Where |
|---|---|---|---|
| 1 | −20 next to ±250 in the four cases | met | (i) `d125_valat_and_an_opponent_who_loses_the_mond`; (ii) `d125_valat_and_a_mond_lost_inside_the_declarers_side`, both ways round; (iii) `d125_valat_against_and_the_declarers_lost_mond` and `d125_valat_against_and_the_mond_left_in_the_talon`; (iv) `d125_valat_against_and_an_opponents_mond_taken_by_an_opponent`. |
| 2 | Parts: valat ±250, mondfang −20 for the mond's player only, the rest zero | met | the helper `valat_and_mondfang` in every `d125_` test: both parts, the six other parts zero, entry = valat + mondfang |
| 3 | A valat without a lost mond scores as before | met | the three `c41_` tests, unchanged |
| 4 | Pagat in the last trick, its side takes it: the side wins the bonus | met | `c39_silent_pagat_ultimo`, cases 1 to 4; with the whole trula in the last trick: `c39_pagat_ultimo_when_the_trula_falls_in_the_last_trick` |
| 5 | The other side takes it: the pagat's side loses the bonus | met | `c39_silent_pagat_ultimo`, cases 5 and 6; `c39_pagat_ultimo_when_the_trula_falls_in_the_last_trick` |
| 6 | The seven listed cases | met | `c39`: 1 the declarer's pagat wins (+25); 2 the partner's pagat under the declarer's XIX (+25); 3 the declarer's pagat under the partner's XIX (+25, C declares dve); 4 an opponent's pagat under another opponent's XIX, in solo tri (−25); 5 the declarer side's pagat taken by an opponent (−25, declarer's and partner's); 6 an opponent's pagat taken by the declarer (+25); the solo case is 4 |
| 7 | Pagat not in the last trick: 0 | met | `c39`, case 7 |
| 8 | Trula trick: the pagat wins only after both the škis and the mond | met | `c17_trula_in_one_trick_the_pagat_wins_only_when_it_falls_last`; `c42` and `c39_pagat_ultimo_when_the_trula_falls_in_the_last_trick` play it in whole deals; `winner_matches_the_reference_on_random_tricks` has 2,519 trula tricks |
| 9 | Every order, the fourth card first, between and last, a suit card and a tarok | met | `c17`: a hand-written table of the 24 orders, run with four fourth cards; then every order with each of the 51 other cards |
| 10 | The mond's player writes −20 in a trula trick whoever wins | met | `c42_mondfang_the_mond_player_writes_minus_20`: mond, škis, pagat (the pagat wins); pagat, škis, mond and škis, pagat, mond (the škis wins). In the last trick: `c39_pagat_ultimo_when_the_trula_falls_in_the_last_trick` |
| 11 | Each old rule planted fails a test | met | table below |
| 12 | Rewards add up to zero, the ten parts add up to the reward | met | `tests/critic_targets_reward.rs` passes unchanged; so do `test_c12_…` and the parity tests of `test_spec006.py` |
| 13 | Crate version raised; golden record | met | 0.2.0. See "Golden files" |
| 14 | README and the pointer lines | met | by reading; see "What changed" |
| 15 | Quality gates | met locally, with one note | "Command output". The Python tests ran against a module built into a scratch folder, not into `.venv`; see "Open issues" 1 |
| 16 | The declarer's valat and the mond in a packet not taken: +250, no mondfang | met | `d133_in_a_valat_the_declarer_wins_the_packets_not_taken`: alone (250, 0, 0, 0) and with a partner who takes one of the twelve tricks (250, 0, 250, 0). The other side stays pinned: `d125_valat_against_and_the_mond_left_in_the_talon` (−250 and −20) |

### Planted wrong rules (criterion 11, the review and D133)

Planted in a copy of `src/engine` outside the repo (in the session's scratch folder, with `docs/encoding.md` next to it and its own target folder). One text replacement per rule; the whole suite ran each time; the file was restored after. The table is the run after D133 was built. The tests have fixed seeds, so the lists are the same in every run.

| Wrong rule planted | File | Tests that failed |
|---|---|---|
| A valat cancels the mondfang (reading 2 of spec 005) | `src/score.rs` | the five `d125_` tests; `c29_each_side_counts_its_cards_and_the_counts_sum_to_70` and `score_matches_the_second_scorer_on_random_deals` (both compare with the second scorer) |
| The pagat itself must take the last trick (criterion 39 of spec 005) | `src/score.rs` | `c39_silent_pagat_ultimo`; `c39_pagat_ultimo_when_the_trula_falls_in_the_last_trick`; `c29_…`; `score_matches_the_second_scorer_on_random_deals` |
| The pagat wins the trula trick in any order (criterion 17 of spec 002) | `src/trick.rs` | `c17_trula_in_one_trick_the_pagat_wins_only_when_it_falls_last`; `c42_mondfang_the_mond_player_writes_minus_20`; `c39_pagat_ultimo_when_the_trula_falls_in_the_last_trick`; `c19_winner_of_a_trick_leads_the_next`; `winner_matches_the_reference_on_random_tricks` |
| The pagat wins only as the fourth card (the reading of C12 that was not chosen) | `src/trick.rs` | `c17_…`; `c42_…` and `c39_pagat_ultimo_when_…` (each has a trick with a card after the pagat); `winner_matches_the_reference_on_random_tricks` |
| D133 off: the talon mondfang is written in the declarer's valat (the code before D133) | `src/game.rs` | `d133_in_a_valat_the_declarer_wins_the_packets_not_taken`; `c21_discards_count_for_declarer_untaken_packets_for_opponents`; `c47_all_54_cards_end_in_the_two_piles_and_the_counts_sum_to_70`; `score_matches_the_second_scorer_on_random_deals` |
| D133 too wide: the talon mondfang is skipped in every valat, also in the valat against | `src/score.rs` | `d125_valat_against_and_the_mond_left_in_the_talon`; `c29_…`; `score_matches_the_second_scorer_on_random_deals` |
| D133 too wide: the packets not taken go to the declarer in the valat against too | `src/game.rs` | `d125_valat_against_and_the_mond_left_in_the_talon`; `c29_…`; `c47_…`; `score_matches_the_second_scorer_on_random_deals` |
| Review item 2: no pagat ultimo when the last trick holds the škis and the mond | `src/score.rs` | `c39_pagat_ultimo_when_the_trula_falls_in_the_last_trick` |
| Review item 2: in a trula trick the pagat itself must win the trick | `src/score.rs` | `c39_pagat_ultimo_when_the_trula_falls_in_the_last_trick` |

9 planted, 9 caught. The clean copy passes all 182 tests before the first and after the last rule.

The first D133 rule is the one the main session asked for under (1), the second is (2). The third is (2) again, planted where the rule is built. The rule of review item 1 (`valat <= 0 &&` at the talon mondfang) is no longer a wrong rule: it gives the entries D133 asks for.

The two rules of review item 2 are caught by the hand-written test only: the whole trula in the last trick did not come up in the random deals. The D133 rules are caught by the random deals too.

### Golden files

- `tests/golden/record_seed_42.json`: one line changed, `"engine_version": "0.1.0"` to `"0.2.0"`. The decisions, the entries and the parts are the same: `c26` compares the whole text and passes.
- No other frozen value changed: the golden deals of `tests/deal.rs`, the six deal seeds of `tests/vecenv.rs`, the three written deals, the seeded deals of `c31` to `c34`, and every number in `test_spec006.py`.

### Command output (trimmed)

Engine, run from `src\engine`:

```
> cargo --version
cargo 1.94.1 (29ea6fb6a 2026-03-24)
> cargo fmt --check
(no output)
> cargo clippy --locked --all-targets -- -D warnings
    Checking tarok-engine v0.2.0 (...\src\engine)
    Finished `dev` profile [optimized + debuginfo] target(s) in 7.91s
> cargo test --locked
     Running unittests src\lib.rs            test result: ok. 20 passed; 0 failed
     Running tests\actions.rs                test result: ok. 6 passed; 0 failed
     Running tests\bidding.rs                test result: ok. 13 passed; 0 failed
     Running tests\cards_and_counting.rs     test result: ok. 7 passed; 0 failed
     Running tests\cardset.rs                test result: ok. 5 passed; 0 failed
     Running tests\critic_targets_reward.rs  test result: ok. 3 passed; 0 failed
     Running tests\deal.rs                   test result: ok. 5 passed; 0 failed
     Running tests\encoding_contract.rs      test result: ok. 1 passed; 0 failed
     Running tests\game_play.rs              test result: ok. 3 passed; 0 failed
     Running tests\inference.rs              test result: ok. 8 passed; 0 failed
     Running tests\king_call.rs              test result: ok. 5 passed; 0 failed
     Running tests\legal_cards.rs            test result: ok. 7 passed; 0 failed
     Running tests\no_leak.rs                test result: ok. 1 passed; 0 failed
     Running tests\record.rs                 test result: ok. 4 passed; 0 failed
     Running tests\scoring.rs                test result: ok. 28 passed; 0 failed
     Running tests\state_features.rs         test result: ok. 9 passed; 0 failed
     Running tests\talon.rs                  test result: ok. 9 passed; 0 failed
     Running tests\tokens.rs                 test result: ok. 7 passed; 0 failed
     Running tests\trick_winner.rs           test result: ok. 10 passed; 0 failed
     Running tests\vecenv.rs                 test result: ok. 11 passed; 0 failed
     Running tests\views.rs                  test result: ok. 3 passed; 0 failed
     Running tests\whole_deals.rs            test result: ok. 4 passed; 0 failed
     Running tests\whole_games.rs            test result: ok. 7 passed; 0 failed
     Running tests\written_deals.rs          test result: ok. 3 passed; 0 failed
   Doc-tests tarok_engine                    test result: ok. 3 passed; 0 failed
(182 tests, 0 failed)
> cargo test --locked --no-default-features
(171 tests, 0 failed)
> cargo tree --locked --depth 1
tarok-engine v0.2.0 (...\src\engine)
├── rand_chacha v0.10.0
├── rayon v1.12.0
├── serde v1.0.229
└── serde_json v1.0.154
[dev-dependencies]
└── proptest v1.12.0
```

The output is the run after the review fixes and D133. The count was 180 before the review, as before the spec: five `reading2_` tests became five `d125_` tests, and `c17` was replaced. The review added two tests in `tests/scoring.rs`: 182. D133 changed one of the two and added none.

Binding crate, run from `src\train\tarok_env`:

```
> cargo fmt --check
(no output)
> cargo clippy --locked --all-targets -- -D warnings
    Checking tarok-engine v0.2.0 (...\src\engine)
    Checking tarok-env v0.1.0 (...\src\train\tarok_env)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 10.75s
> cargo test --locked
test result: ok. 0 passed; 0 failed
```

Python, run from `src\train`, with the new module on `PYTHONPATH` (see "Open issues" 1):

```
> maturin build --release --out <scratch>\wheel013
📦 Built wheel for CPython 3.12 to <scratch>\wheel013\tarok_env-0.1.0-cp312-cp312-win_amd64.whl
> python -m pip install --no-deps --target <scratch>\site013 <scratch>\wheel013\tarok_env-0.1.0-cp312-cp312-win_amd64.whl
> set PYTHONPATH=<scratch>\site013
> set TAROK_REQUIRE_ENV=1
> python -c "import tarok_env; print(tarok_env.ENGINE_VERSION)"
0.2.0
> python -m pytest tests\test_spec006.py -q -p no:cacheprovider
26 passed in 32.37s
```

That is the run after D133, with the wheel built again from the final sources. A run before the review took 10 minutes for the same 26 tests, next to the other session's jobs.

### Readings and deviations

1. **Reading C12.** The task named reading (a) as open; the user decided it the same day (D132). The comment in `src/trick.rs` says "Reading C12 (decided, D132)" and names the reading that was not chosen as `pagat == 3`. Criterion 11 plants that one as a wrong rule.
2. **Pagat ultimo in the code is one question:** does the declarer's side take the last trick that holds the pagat? Yes is +25, no is −25. Who played the pagat does not change the sign. This is criteria 4 and 5 worked through: all four combinations of "whose pagat" and "which side takes the trick" give that sign. The tests still build each case apart.
3. **Version 0.2.0.** The spec says "raised". A change of the rules is taken as a minor step. The binding crate and its `pyproject.toml` stay at 0.1.0; `tarok_env.ENGINE_VERSION` now reads 0.2.0.
4. **Tests beyond the listed cases.** Criterion 1 (ii) the other way round (the partner's mond under the declarer's škis); (iii) with the partner as the mond's player in a valat against; criterion 6 case 4 again in tri when the declarer called its own king.
5. **Test names.** The `reading2_` tests are now `d125_`. `c17` and `c39` keep their numbers from specs 002 and 005, because the files are named by those criteria; their doc comments point here.
6. **`src/record.rs`: one doc line.** The example in its doc comment now shows `"engine_version": "0.2.0"` (review item 5). Nothing else in the file changed. See open issue 3.

7. **D133 is built as "the packets are the declarer's", not as "the −20 is skipped".** The user's words are "the declarer gets the packet he did not pick". The engine already had one place that hands the rest of the talon to the declarer, for criterion 23 of spec 005: `talon_rest_won` in `src/game.rs`. D133 is a second way to make it true: the declarer's side took all twelve tricks. `score.rs` is unchanged apart from comments: it gets an empty set of "talon cards with the opponents" and writes no mondfang. What follows from this choice:
   - In the declarer's valat the full view shows all 54 cards in the declarer's pile and none with the opponents; the card points are 70 and 0. No entry depends on that: a valat scores no game and no difference.
   - The packets move after the twelfth trick. Until then they lie with the opponents; the `d133_` test checks this after eleven tricks.
   - Criterion 21 of spec 005 ("the packets not taken count for the opponents") has an exception now. Its test in `tests/talon.rs` plays six contracts with the lowest legal cards; two of them end in a valat of the declarer's side. The test now expects the packets with the declarer in those two and with the opponents in the other four, and with the opponents in all six after eleven tricks.
   - Spec 005 has one line each at criteria 21 and 44 that points here. The main session did not ask for these two lines.
   - The other way (one condition at the talon mondfang in `score.rs`) would give the same entries and leave the piles as they were.
8. **Files outside the spec's list.** `src/game.rs` and `tests/talon.rs` (D133), `src/record.rs` and `tests/whole_games.rs` (review items 5 and 7).

### Review fixes (2026-11-12, after the D72 step 4 review)

The review found no defect in the code. It found two test gaps, two statements in this text that did not hold in every run, and stale text. No rule changed. Each item has its number from the review.

**A. Test gaps**

1. **The declarer's side takes all twelve tricks and the mond lies in a packet not taken.** The first fix was a test from the reviewer's draft that pinned what the code did: +250 and −20, entries 230, 0, 0, 0. The main session asked the user, and the user ruled the other way (D133): "In case of valat, the declarer gets the packet he did not pick so he did not lose the mond". It is criterion 16 now. The test is `d133_in_a_valat_the_declarer_wins_the_packets_not_taken`: +250 and no mondfang, alone and with a partner. How it is built: reading 7 above.
2. **The whole trula in the last trick.** New test `c39_pagat_ultimo_when_the_trula_falls_in_the_last_trick`, on the reviewer's deal (A holds the škis, B the mond, C the pagat; all three keep them for the last trick). Five plays of it:
   - (a) škis, mond, pagat, karo 1, A alone in tri: the pagat of the opponent C wins. −25 and the trula −10 for A; −20 for B.
   - (a) the same trick with C as the declarer in solo tri: the declarer's own pagat wins. +25 and the trula +10 for C; −20 for B.
   - (b) mond, pagat, karo 1, škis, A alone: the škis wins and takes the opponent's pagat. +25 and +10 for A; −20 for B.
   - (b) the same trick, C plays dve and A is the partner: the pagat loses the trick, but its side takes it with the škis. +25 for C and A.
   - (b) the same trick, B plays solo tri: the opponent A takes the pagat of the opponent C with the škis. −25, the trula −10 and the mondfang −20 for B.

   The last two are the cases the review asked for: the pagat's own side takes the trick with the škis. Every play is also compared with the second scorer.

   The two rules of the review for this item were planted: both fail. See the table above.

**B. Tests seeded from the clock**

3. **`run_cases` has fixed seeds now.** It is shared by 24 calls in 11 test files of specs 002, 005 and 006, so all of them changed together:
   - Each call has its own seed, made from one constant (`FIXED_SEED`), the name of the test file and the line of the call. The name is taken without its folder, so Windows and Linux give the same seeds. Checked: two single-threaded runs print the same 24 seeds, all different.
   - `set TAROK_TEST_SEED=clock` gives a new seed on every run, as before. A number gives that seed to every test, which replays a printed seed.
   - Why not keep the clock as the default: a test that passes or fails by the clock makes statements like the ones of item 4 untrue in some runs. What is lost: new cases on every run. The `clock` setting keeps that for whoever wants it; CI does not use it. A CI job with the clock seed would be a change of the workflow file, which is not in this spec.
   - A line added above a call changes that call's seed. The cases are then other cases, still the same in every run.
   - `winner_matches_the_reference_on_random_tricks` builds a quarter of its 10,000 tricks with the škis, the mond, the pagat and one more card, in a random order of play. It counts and asserts what it saw: 2,519 trula tricks (the pagat won 844, the škis 1,675); of the others a tarok won 6,632 and a suit card 849. Before, about two tricks in 10,000 held the trula.
   - The counts that the tests of spec 006 print are fixed too. In this tree: no-leak 15,341 cases compared; 1,015,347 "certainly holds" bits; the longest token sequence 66. Every count those tests assert still holds.

**C. This text**

4. Both statements are corrected. The opening line of Results: the code of HEAD fails 16 of the 182 tests, in every run (14 before D133). The planted-rules table: `winner_matches_the_reference_on_random_tricks` fails for both wrong trula rules, in every run. The reviewer's count of 11 and my 12 were both true runs of a test that depended on the clock.

**D. Stale text**

5. `src/record.rs`: the doc example shows `"engine_version": "0.2.0"`.
6. `docs/specs/005-deal-flow-a.md`, reading 3: one line that points here.
7. `tests/whole_games.rs`: the doc comment of `score_matches_the_second_scorer_on_random_deals` names spec 013.

Also changed with item 3: the paragraph on test seeds in `src/engine/README.md`.

**Counts after the review and D133:** engine 182 tests (171 without the feature `vecenv`); `fmt` and `clippy` clean for both crates; binding crate 0 Rust tests; `test_spec006.py` 26 passed against a wheel built from the final sources.

### Open issues

1. **The module in `.venv` is still the 0.1.0 build.** The other session had Python jobs running on `.venv` while this spec ran, so the module was not installed over them. It was built as a wheel into a scratch folder and the tests ran against it through `PYTHONPATH`. To install it, from `src\train\tarok_env` with the venv active: `maturin develop --release`. Until then the installed module plays the old three rules.
2. **One run of `test_spec006.py` in between failed.** The first run against the new module had 18 failures: the other session's new stale-module check (`build.rs`, untracked) had changed after the wheel was built. After a second build, one run had 6 failures (`test_c22_make_…`, the four `spec007` tests, one more) while `env.py`, `ppo.py` and `rollout.py` were being edited by that session; the output was not kept. Two later runs passed 26 of 26, one of them in 10 minutes under load. Nothing in this spec touches those files. Worth one more run after spec 008 is done.
3. **Records written by engine 0.1.0.** Replay does not look at `engine_version`. A 0.1.0 record whose deal has a trula trick, a pagat in the last trick or a valat with a lost mond replays under 0.2.0 to other entries, or stops at a decision with a mismatch. The error does not say that the engine version differs. Question 1 below.
4. **S40 is not in the record's toggle list.** The record lists the nine scoring toggles; S40 changes who wins a trick and is none of them. It belongs to the spec that gives the engine its toggles.
5. **CI is unverified.** Nothing is pushed.
6. **Solo brez and the mondfang** (the note in D125) is not touched: the contract is not built yet.
7. **A stale comment** in `tests/common/flow.rs` says the longest deal has 67 actions; it has 64. Not changed here.
8. **The doc line in `src/record.rs` changes the engine sources.** A check that compares the installed module with the sources on disk reports the module as stale until it is built again. It has to be built again for 0.2.0 anyway (open issue 1).

### Questions for the user

1. Should replay refuse a record of another engine version, or name the version in the error when the entries differ? Recommended: name both versions in the error and keep replaying, because most old records still replay.
