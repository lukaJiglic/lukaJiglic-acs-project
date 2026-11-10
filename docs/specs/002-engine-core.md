# 002 — Engine core

Status: **approved** by the user on 2026-11-10 (D112), with criterion 17 checked against the sources (D113). Needs spec 001 done.

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
