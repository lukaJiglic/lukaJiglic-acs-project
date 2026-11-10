# 002 — Engine core

Status: **done** 2026-11-10. Approved (D112, D113), executed, diff read by the user (D114), committed in `0d8ab9a`, CI green on `56f4caf` after the compiler was pinned in the workflow (D115, criterion 32). Open for the user: how the compiler is pinned locally; "fant" or "pob". See "Results" at the end.

Build-plan row 002. Tool and model: Claude Code 2.1.306, `claude-fable-5-2`.

## Goal

The first part of the Rust crate `tarok-engine` in `src/engine`: cards, card sets, card points, a seeded deal, the trick winner, legal moves for normal contracts, and a card-play state machine. Enough to play the twelve tricks of a normal contract with random legal cards and count the points.

Rules source: `docs/planning/research/01-rules-and-variants.md` sections B1 and B3, and the baseline in `docs/planning/rule-switches.md`. No code from other tarok engines (D9).

## In scope

- `Card`: 54 cards with a fixed index 0–53. Taroks I–XXI are 0–20, škis is 21. Then srce, karo, pik, križ, eight cards each, low to high (srce and karo: 4, 3, 2, 1, fant, kaval, dama, kralj; pik and križ: 7, 8, 9, 10, fant, kaval, dama, kralj). The order is frozen by spec 004.
- `CardSet`: a 64-bit mask with insert, remove, contains, count, iteration, set operations.
- Card values: kralj 5, škis 5, mond 5, pagat 5, dama 4, kaval 3, fant 2, all others 1.
- Counting a pile: groups of three, sum minus 2 per group; a leftover of one or two cards minus 1. Done in integer thirds inside, rounded once per pile.
- Seats 0–3. "Next seat" is +1 modulo 4. Direction on screen is a display matter (D91).
- Deal: a seed (u64) gives 12 cards to each seat and 6 to the talon. The shuffle is written in the crate (Fisher–Yates over a ChaCha8 stream from `rand_chacha`, pinned), so the same seed gives the same deal on native, WASM and through Python.
- Trick winner, normal contracts: the highest tarok wins; with no tarok, the highest card of the led suit; škis is the highest tarok; if škis, mond and pagat are in one trick, the pagat wins.
- Legal cards, normal contracts: follow the led suit; if void, play a tarok; if void and without taroks, any card. If a tarok is led: a tarok if held, else any card. No duty to overtake.
- `PlayState`: four hands, the leader, the current trick, the pile of each seat. `legal_cards()`, `play(card)`, `is_over()`, `trick_history()`, `points(seat)`. An illegal card returns an error and changes nothing.
- A benchmark binary that plays random legal cards through whole deals on one thread and prints card plays per second.

## Out of scope

Bidding, king call, talon exchange, contracts, teams, scoring beyond raw card points, klop and berač rules (must overtake, pagat last), announcements, toggles, game records, encoding, Python or WASM bindings. Those are specs 004–007 and later.

## Acceptance criteria

The user approves these before tests are written (D73). Each scenario below becomes one named test.

**Cards and counting**
1. There are 54 distinct cards; index → card → index is the identity for 0–53.
2. Values: each kralj 5, each dama 4, each kaval 3, each fant 2, škis, mond, pagat 5 each, every other card 1.
3. Three kings count 13. Kralj, dama, kaval of one suit count 10. Three pip cards count 1.
4. A pile of two cards (kralj, dama) counts 8. A pile of one kralj counts 4. An empty pile counts 0.
5. The whole deck counts 70.
6. Property: for any split of the 54 cards into two piles, the two counts sum to 70. (10,000 random splits.)

**Deal**
7. Any seed gives four hands of 12 and a talon of 6, all 54 cards exactly once. (10,000 seeds.)
8. The same seed gives the same deal twice in one process.
9. Seed 0 gives a deal equal to a golden listing stored in the test file. (Guards against a silent change of the shuffle.)
10. Two different seeds among 1,000 tried give different deals in at least 999 cases.

**Trick winner**
11. Suit led, all follow, no tarok: the highest card of the led suit wins.
12. Court order in a suit: kralj beats dama beats kaval beats fant beats every pip.
13. Red pips: 1 beats 2 beats 3 beats 4. Black pips: 10 beats 9 beats 8 beats 7.
14. A card of another suit never wins, whatever its rank.
15. Any tarok beats any suit card.
16. A higher tarok beats a lower one; škis beats mond.
17. Škis, mond and pagat in one trick: the pagat wins, **in any order of play**. Tested with the pagat played first, second, third and fourth, and with a fourth card that is a suit card or another tarok. (Checked on 2026-11-10: TARZS 2010 says "v poljubnem vrstnem redu"; D113.)
18. Pagat with only one of škis and mond in the trick: the pagat does not win.
19. The winner of a trick leads the next one.

**Legal cards**
20. Suit led, hand holds that suit: only cards of that suit are legal.
21. Suit led, void in it, holds taroks: only taroks are legal.
22. Suit led, void in it, no taroks: every card in hand is legal.
23. Tarok led, holds taroks: only taroks are legal.
24. Tarok led, no taroks: every card in hand is legal.
25. On the lead: every card in hand is legal.
26. `play` with an illegal card returns an error and the state is unchanged.

**Whole deals (property tests, 10,000 random deals with random legal play)**
27. The legal set is never empty before the deal is over.
28. After 12 tricks every hand is empty and the four piles hold 48 distinct cards.
29. The points of the four piles plus the talon, counted as one pile per side for any two-way grouping, sum to 70.
30. Replaying the same seed with the same choices gives the same trick history.

**Quality gates**
31. `cargo fmt --check` and `cargo clippy -- -D warnings` pass. No `unsafe`.
32. A workflow file `.github/workflows/engine.yml` runs `cargo test`, fmt and clippy for `src/engine` on every push; it is green.
33. The benchmark runs in release mode and prints card plays per second on one thread. The number goes into `docs/progress.md`. Expected range from research/05 D4 (inferred, not measured): 10^5 to 10^6. Below 10^5 is reported as a finding, not hidden.

## Files expected to change

New: `src/engine/Cargo.toml`, `src/engine/src/lib.rs`, `src/engine/src/card.rs`, `cardset.rs`, `points.rs`, `deal.rs`, `trick.rs`, `play.rs`, `src/engine/tests/*.rs`, `src/engine/benches` or `src/bin/bench_play.rs`, `rust-toolchain.toml`, `Cargo.lock`.
New: `.github/workflows/engine.yml`. Changed: this spec (status and a "Results" section). `docs/progress.md` and `docs/architecture.md` are updated by the main session from the Results section (D109).

Dependencies: `rand_chacha` (pinned). Dev: `proptest`. General libraries are allowed (D27).

## Commits

Small steps, each with `Spec: docs/specs/002-engine-core.md`: (a) crate, cards, card sets, values, counting, with tests 1–6; (b) deal, tests 7–10; (c) trick winner and legal cards, tests 11–26; (d) `PlayState`, property tests 27–30, benchmark; (e) CI job.

## Answers to the open points (D112, D113)

- A. Slovenian card names in code: yes.
- B. `Fant` is the working name; the user has not chosen between "fant" and "pob".
- C. Criterion 17 was checked and kept. If the user's table lets the pagat win only when it falls last, that becomes a toggle (C11 in `docs/planning/open-questions.md`).

## Results

Executed on 2026-11-10 by a parallel session (D109). Claude Code, model `claude-fable-5-2`. Nothing is staged or committed. Tests were written from the criteria before the code. Local runs used rustc 1.94.1 (`stable-x86_64-pc-windows-gnu`).

### Criteria

Test files are in `src/engine/tests/`.

| # | Test | File | Result |
|---|---|---|---|
| 1 | `c01_54_distinct_cards_and_index_round_trip` | `cards_and_counting.rs` | pass |
| 2 | `c02_card_values` | `cards_and_counting.rs` | pass |
| 3 | `c03_piles_of_three_cards` | `cards_and_counting.rs` | pass |
| 4 | `c04_piles_of_two_one_and_zero_cards` | `cards_and_counting.rs` | pass |
| 5 | `c05_whole_deck_counts_70` | `cards_and_counting.rs` | pass |
| 6 | `c06_any_two_way_split_of_the_deck_counts_70` (10,000 cases) | `cards_and_counting.rs` | pass |
| 7 | `c07_any_seed_deals_four_hands_of_12_and_a_talon_of_6` (10,000 cases) | `deal.rs` | pass |
| 8 | `c08_same_seed_gives_the_same_deal_twice` | `deal.rs` | pass |
| 9 | `c09_seed_0_matches_the_golden_deal`, `c09_seed_0x0123456789abcdef_matches_the_golden_deal` | `deal.rs` | pass |
| 10 | `c10_different_seeds_give_different_deals` (1,000 random pairs; also seeds 0–999 against the next seed) | `deal.rs` | pass |
| 11 | `c11_suit_led_all_follow_highest_of_led_suit_wins` | `trick_winner.rs` | pass |
| 12 | `c12_court_order_kralj_dama_kaval_fant_then_pips` | `trick_winner.rs` | pass |
| 13 | `c13_red_pips_1_high_black_pips_10_high` | `trick_winner.rs` | pass |
| 14 | `c14_card_of_another_suit_never_wins` | `trick_winner.rs` | pass |
| 15 | `c15_any_tarok_beats_any_suit_card` | `trick_winner.rs` | pass |
| 16 | `c16_higher_tarok_beats_lower_and_skis_beats_mond` | `trick_winner.rs` | pass |
| 17 | `c17_trula_in_one_trick_pagat_wins_any_order` | `trick_winner.rs` | pass |
| 18 | `c18_pagat_with_only_one_of_skis_and_mond_does_not_win` | `trick_winner.rs` | pass |
| 19 | `c19_winner_of_a_trick_leads_the_next` | `trick_winner.rs` | pass |
| 20 | `c20_suit_led_and_held_only_that_suit_is_legal` | `legal_cards.rs` | pass |
| 21 | `c21_suit_led_void_with_taroks_only_taroks_are_legal` | `legal_cards.rs` | pass |
| 22 | `c22_suit_led_void_without_taroks_every_card_is_legal` | `legal_cards.rs` | pass |
| 23 | `c23_tarok_led_with_taroks_only_taroks_are_legal` | `legal_cards.rs` | pass |
| 24 | `c24_tarok_led_without_taroks_every_card_is_legal` | `legal_cards.rs` | pass |
| 25 | `c25_on_the_lead_every_card_is_legal` | `legal_cards.rs` | pass |
| 26 | `c26_illegal_card_returns_an_error_and_changes_nothing` | `legal_cards.rs` | pass |
| 27 | `c27_legal_set_is_never_empty_before_the_deal_is_over` (10,000 deals) | `whole_deals.rs` | pass |
| 28 | `c28_after_12_tricks_hands_are_empty_and_piles_hold_48_distinct_cards` (10,000 deals) | `whole_deals.rs` | pass |
| 29 | `c29_any_two_way_grouping_of_piles_and_talon_sums_to_70` (10,000 deals, all 32 groupings each) | `whole_deals.rs` | pass |
| 30 | `c30_same_seed_and_choices_give_the_same_trick_history` (10,000 deals). It can only fail if the engine is not deterministic; that is what the criterion asks | `whole_deals.rs` | pass |
| 31 | `cargo fmt --check`; `cargo clippy -- -D warnings`; `cargo clippy --all-targets -- -D warnings`; `#![forbid(unsafe_code)]` in `lib.rs` and `bench_play.rs` | | pass locally |
| 32 | `.github/workflows/engine.yml` is written; it runs on every push and on pull requests | | **not verified**: nothing is pushed, so the workflow has not run |
| 33 | `src/engine/src/bin/bench_play.rs`, release mode, one thread | | pass: about 3.5 × 10^7 card plays per second. The entry in `docs/progress.md` is left to the main session (D109) |

Criterion 17 covers all 24 orders of play for each of the 51 possible fourth cards (32 suit cards, 19 taroks). It also has eight written-out tricks: the pagat first, second, third and fourth, once with a suit card and once with another tarok.

Tests without a criterion number: `card_index_order_matches_the_spec`, `winner_matches_the_reference_on_random_tricks` (10,000 tricks), five `cardset_*` tests (`tests/cardset.rs`), seven unit tests inside `src/`, two doc tests. 47 tests in all (42 before the review fixes).

Check of the tests themselves: seven wrong rules were put into the code one at a time (pagat wins the trula only when played last; another suit can win; no duty to play a tarok when void; count rounds down; winner does not lead; talon dealt from the wrong end; illegal card accepted). Each one made at least one numbered test fail. The code was restored after each. After the review an eighth was tried: the seed written into the key in the other byte order. See "Review fixes".

### Review fixes (2026-11-10, after the D72 step 4 review)

The review found no rule bug. It found test gaps and three weak spots in the API. What changed, by item:

1. **Seed-to-key mapping.** New test `c09_seed_0x0123456789abcdef_matches_the_golden_deal` with a second frozen deal (seed 81985529216486895). The listing was printed by the Rust code (`examples/print_deal.rs`) and is the same from `tests/golden_deal_reference.py`. Proof: with `to_be_bytes` in `src/deal.rs` the seed-0 test still passes and the new test fails (hand of seat 0 differs). Switched back; both pass.
2. **Seed after a panic.** `run_cases` now prints `property test: N cases, seed S` before the first case; `cargo test` shows it for any failing test. It also catches a panic inside a case and gives it to proptest as a failed case, so the failing value is shrunk and printed. Proof: a temporary `panic!` in `PlayState::finish_trick` made `c28` fail with the seed line, the panic text and a minimal failing input; the run with `TAROK_TEST_SEED` set to that seed failed the same way. The panic was removed.
3. **Workflow.** `on: [push, pull_request]`. `--locked` on the clippy, test and run steps (`cargo fmt` has no such flag and reads no dependencies).
4. **Panics and a hang in the API.**
   - `Deal`: the fields are now private, with `hands()`, `hand(seat)`, `talon()`, `talon_set()`. A `Deal` can only come from `Deal::from_seed`, so `PlayState::from_deal` cannot fail and stays without a `Result`. Hand-built hands go through `PlayState::new`, which returns a `Result`. Chosen over a `Result` on `from_deal` because the callers in training loops then have no error to handle. Test: a `compile_fail` doc test on `Deal`.
   - `Seat::plus` reduces the steps modulo 4 first. Test: `seat_plus_takes_any_number_of_steps` (with `usize::MAX`).
   - `CardSet::nth(n)` returns `None` when `n >= len()`. Test: added to `cardset_iterates_in_index_order` (with `u32::MAX`).
5. **Rejection branch.** The bounded draw takes its word source as a closure. Test `draw_skips_words_under_the_threshold`: for every bound 2 to 54 the word `2^32 mod n - 1` is rejected and the word `2^32 mod n` is kept.
6. **Python reference.** A workflow step runs `tests/golden_deal_reference.py` with the runner's `python3` and compares it with `cargo run --example print_deal` for five seeds, the two golden seeds among them. Run locally in Git Bash: same deal for all five. The script uses only `struct` and `sys`.
7. **Weak tests.**
   - `c19` no longer uses `trick::winner` for its expectation. The tests have their own `reference_winner` (`tests/common/mod.rs`), written from the order lists of the rules, not from card indices. Seat numbers are added by hand.
   - `c14` now puts the three off-suit cards in all six orders.
   - `c30`: left as it is; noted in the table above.
   - New: `winner_matches_the_reference_on_random_tricks`.

Benchmark after the fixes (three runs): 36.6, 35.9 and 35.8 million card plays per second. No change.

### Command output (trimmed)

Run in `src\engine`.

```
> cargo fmt --check
(no output, exit 0)

> cargo clippy --locked --all-targets -- -D warnings
    Checking tarok-engine v0.1.0 (<repo>\src\engine)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.72s

> cargo test --locked
     Running unittests src\lib.rs
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests\cards_and_counting.rs
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.30s
     Running tests\cardset.rs
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests\deal.rs
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.33s
     Running tests\legal_cards.rs
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests\trick_winner.rs
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.66s
     Running tests\whole_deals.rs
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.11s
   Doc-tests tarok_engine
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
```

This is the run after the review fixes. The second doc-test line is the `compile_fail` test.

### Benchmark

Command, in `src\engine`:

```
cargo run --release --bin bench_play
```

One million deals (seeds 0 to 999,999), 48 random legal cards each, one thread. The time covers the deal, the legal-card sets, the choice and the plays. Machine: AMD Ryzen 5 5600, Windows 11. Default release profile.

| Run | Seconds | Card plays per second |
|---|---|---|
| 1 | 1.368 | 35,091,426 |
| 2 | 1.444 | 33,245,986 |
| 3 | 1.357 | 35,365,655 |

Number for `docs/progress.md`: **about 3.5 × 10^7 card plays per second** (about 730,000 deals per second). This is above the range expected in research/05 D4 (10^5 to 10^6, inferred). The benchmark is card play only: no bidding, no encoding, no network.

### Public API

- `Card`: `from_index`, `index`, `tarok(1..=21)`, `court(Suit, Rank)`, `pip(Suit, number)`, `all()`, `is_tarok`, `suit`, `face`, `value`; constants `PAGAT`, `MOND`, `SKIS`, `COUNT`. `Display` gives `XXI`, `skis`, `srce kralj`, `pik 7`.
- `Suit` (`Srce`, `Karo`, `Pik`, `Kriz`), `Rank` (`Fant`, `Kaval`, `Dama`, `Kralj`), `Face` (`Tarok(n)`, `Skis`, `Pip(suit, n)`, `Court(suit, rank)`).
- `CardSet`: `EMPTY`, `FULL`, `TAROKS`, `suit(Suit)`, `single`, `insert`, `remove`, `contains`, `len`, `is_empty`, `union`, `intersection`, `difference`, `complement`, `is_subset`, `is_disjoint`, `iter`, `nth`, `bits`, `from_bits`; operators `|`, `&`, `-`; `FromIterator<Card>`.
- `points::count(CardSet) -> u8`, `points::thirds(CardSet) -> u16`, `points::TOTAL` (70).
- `Seat`: `new(0..=3)`, `ALL`, `index`, `next`, `plus(n)`.
- `Deal` (private fields): `from_seed(u64)`, `hands() -> [CardSet; 4]`, `hand(Seat)`, `talon() -> [Card; 6]` in the order it lies, `talon_set()`. `shuffled_deck(u64) -> [Card; 54]`.
- `trick::winner(&[Card; 4]) -> usize` (position in the order of play), `trick::legal_cards(hand, led: Option<Card>) -> CardSet`.
- `PlayState`: `new(hands, leader) -> Result`, `from_deal(&Deal, leader)` (cannot fail), `leader`, `to_move`, `hand(Seat)`, `pile(Seat)`, `current_trick`, `trick_history() -> &[Trick]`, `is_over`, `legal_cards`, `play(Card) -> Result<(), PlayError>`, `points(Seat)`.
- `Trick { leader, cards: [Card; 4], winner }`. `PlayError`: `DealOver`, `NotInHand(Card)`, `NotLegal(Card)`, `InvalidHands`.

### How the deal is made

Written in `src/engine/src/deal.rs`:

1. ChaCha8 key: the seed as 8 little-endian bytes, then 24 zero bytes. Stream 0.
2. Deck in index order. Fisher–Yates from position 53 down to 1.
3. A draw below `n` takes 32-bit words, skips words under `2^32 mod n`, returns the word modulo `n`. Only `u32` arithmetic.
4. Positions 0–5 are the talon, in order. Then packets of six to seats 0, 1, 2, 3, 0, 1, 2, 3.

The key is built in the crate, not with the library's `seed_from_u64`. So only the ChaCha8 block function comes from `rand_chacha` (`=0.10.0`). A separate pure-Python ChaCha8 and shuffle (`src/engine/tests/golden_deal_reference.py`) gives the same deals as the Rust code for both golden seeds. The golden listings were frozen after that. The workflow compares the two programs on every run.

### Deviations

1. **proptest is built without its `std` feature.** With `std` it needs `getrandom`, which needs `dlltool` and an assembler on the windows-gnu toolchain. This machine has neither on PATH (`dlltool.exe: program not found`; with the toolchain's own `dlltool`: `CreateProcess` fails, no `as.exe`). Without `std` proptest cannot seed itself. The helper `run_cases` in `tests/common/mod.rs` seeds the proptest runner from the clock, prints the seed before the first case and catches panics (`TAROK_TEST_SEED` replays a run). The `proptest!` macro is not used. Shrinking works. There are no regression files.
2. **Files beyond the expected list:** `tests/common/mod.rs` (helpers), `tests/cardset.rs`, `tests/golden_deal_reference.py`, `examples/print_deal.rs`, and an updated `src/engine/README.md`. `Seat` lives in `deal.rs`.
3. **`CardSet::len()`**, not `count()`, with `is_empty()` (the Rust naming). The pile count is `points::count`.
4. **`Cargo.toml` fields not named in the spec:** `rust-version = "1.94"`, `publish = false`, `license = "MIT"` as in `LICENSE`.

### Open issues

1. **`rust-toolchain.toml` does not work on this machine as it is.** rustup's default host is `x86_64-pc-windows-msvc`, so the file resolves to `1.94.1-x86_64-pc-windows-msvc`. That toolchain is not installed and cannot link here (no MSVC linker). A plain `cargo` call in the repo now fails with "toolchain is not installed", or starts a download of it if rustup's auto-install is on (the download was not tried). All local runs above used `set RUSTUP_TOOLCHAIN=stable-x86_64-pc-windows-gnu` (the same rustc 1.94.1). The file is kept because CI needs the pin. See question 1.
2. **Criterion 32 is not verified.** The workflow uses only `actions/checkout@v4` and the runner's rustup. Its first step is `rustup show active-toolchain || rustup toolchain install`, which installs from `rust-toolchain.toml`. The last step needs `python3` on the runner (`ubuntu-latest` has it). It needs the first push to show green.
3. **Seat 0 and the dealer.** The deal gives packets to seats 0–3 in seat order. Which seat is the dealer or forehand is not defined yet (bidding is a later spec). Changing the packet order later changes the golden deal.
4. **`PlayState::new` takes exactly twelve cards per hand.** Shorter hands are refused. A later spec may need a second constructor for positions in the middle of a deal.
5. **`points(seat)` rounds per pile.** A side of two seats is counted on the union of their piles, not as the sum of two calls. The doc comment says so.
6. **No `dlltool` and no assembler** will likely matter again for PyO3 on windows-gnu. Not needed for this spec.

### Questions for the user

1. `rust-toolchain.toml` on this machine. Options: (a) `rustup set default-host x86_64-pc-windows-gnu`, then `rustup toolchain install 1.94.1`; plain `cargo` then works in the repo. (b) Keep rustup as it is and type `set RUSTUP_TOOLCHAIN=stable-x86_64-pc-windows-gnu` in each cmd window. (c) Remove the file and pin 1.94.1 only in the workflow. Recommendation: (a). It is one change, and local and CI then read the same file.
2. Install MinGW-w64 binutils or the MSVC build tools? Not needed for spec 002. Likely needed for the Python binding. Recommendation: decide with the binding spec.
3. `Fant` or `Pob` is still open (answer B above). The code uses `Fant`.

### Note from the main session (2026-11-10, after the reviews)

`rust-toolchain.toml` is renamed to `rust-toolchain.toml.disabled`. With it in place, any `cargo` or `rustup` call in the repository made rustup download and select `1.94.1-x86_64-pc-windows-msvc` (the file names no host and rustup's default host on this PC is MSVC). A review agent triggered that install (588 MB); the main session removed it again. With the file disabled, `cargo` in the repository uses the installed `stable-x86_64-pc-windows-gnu` (1.94.1) and no environment variable is needed. The workflow then uses the runner's default stable compiler, unpinned, until the user chooses: (a) set rustup's default host to GNU and restore the file; (b) restore the file and set `RUSTUP_TOOLCHAIN` by hand; (c) delete the file and pin 1.94.1 inside the workflow.
