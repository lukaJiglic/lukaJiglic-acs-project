# 005 — Deal flow A: bidding, king call, talon, scoring of the normal contracts

Status: **done 2026-11-11.** Approved 2026-11-10 (D118), executed 2026-11-10, committed in `2bd5f21` (D122), CI green on `819220d`. Needs spec 002 (done). See "Results" at the end.

Build-plan row 005. Tool and model: Claude Code 2.1.306, `claude-fable-5-2`.

## Goal

A whole deal in the engine for the six normal contracts: tri, dve, ena, solo tri, solo dve, solo ena. From the deal through bidding, the king call, the talon exchange and the twelve tricks to the score each player writes. This is the first of three rule slices (D108). Announcements and kontra are flow B (spec 009). Klop, berač, solo brez, valat as a contract and radelci are flow C (spec 010).

Rules source: pagat.com, Slovenian Tarok, four-player game (read again on 2026-11-10 for bidding, talon and scoring), `docs/planning/research/01` section B3, and the Default preset in `docs/planning/rule-switches.md` section 7. No code from other tarok engines (D9).

## In scope

- `Game`: a state machine over the phases bidding, contract choice, king call, talon pick, discard, play, over. `legal_actions()`, `apply(action)`, `phase()`, `to_act()`. An illegal action returns an error and changes nothing.
- `Action`: one type for all phases, with a fixed mapping to the 160 action slots of `docs/encoding.md` (bids 0–12, kings 13–16, talon packets 17–22, discards 23–76, card plays 90–143). Slots of things not built yet are never legal.
- The dealer is a parameter (seat 0–3). Forehand is the next seat. Hands come from `Deal::from_seed` by seat index, whoever deals, so the golden deals of spec 002 do not change.
- What each seat may know: accessors for the public state and for one seat's private view (own hand, own discards). The true partner and the hidden discards are available only through a separate full-information accessor, used by tests and later by the critic.
- `score()`: the entry each seat writes, and its parts (game, difference, trula, kralji, kralj ultimo, pagat ultimo, valat, mondfang), since the decomposed value head needs the parts (D15).
- A constructor for tests that takes chosen hands, talon and dealer.
- A benchmark: whole deals with random legal actions per second, one thread.

## Out of scope

Announcements and kontra (silent bonuses only here). Klop, berač, solo brez, valat as a bid, odprti berač, barvni valat: their bid slots exist and are never legal in this spec. Radelci and sessions. Non-default toggle values. The encoding of observations and the JSON game record (they move to spec 006, because they follow the encoding contract). Bots.

**Temporary rule of this slice:** when the other three pass, forehand must name tri or a higher normal contract. Klop joins in flow C.

## Acceptance criteria

The user approves these before tests are written (D73). Seats in the examples: A is forehand, then B, C, D in order of play; D is the dealer. Each numbered scenario is one named test.

**Bidding**
1. B speaks first, then C, then D, then A. After that the turn passes round the players who have not passed.
2. B's legal actions at the start: pass, dve, ena, solo tri, solo dve, solo ena. Not klop, not tri.
3. A player with lower priority than the last bidder must bid higher or pass. After B bids dve, C may not bid dve.
4. A player with higher priority may bid the same. Source example: B bids dve, C passes, D bids ena; A may bid ena. D must then bid above ena or pass.
5. Priority runs A (highest), B, C, D (lowest).
6. A player who has passed gets no further turn.
7. The auction ends when three players have passed. The one who has not passed is the declarer.
8. B bids, C and D pass: A may equal B's bid, bid higher, or pass. If A passes, B is the declarer.
9. B bids dve, C bids ena, D and A pass: B has priority over C and may bid ena. C must then go higher or pass.
10. B, C and D pass: A must name a contract. Legal: tri, dve, ena, solo tri, solo dve, solo ena. Pass is not legal. (Klop joins in flow C.)
11. The declarer then names the contract to play: the winning bid or any higher normal contract (S3 a). A lower one is not legal.

**King call**
12. In tri, dve and ena the declarer names one of the four suits. All four are legal, also a suit whose king the declarer holds.
13. In the solo contracts there is no king call.
14. The partner is the seat that holds the called king. The public state does not show it until the king is played.
15. A declarer who holds the called king plays alone against three.
16. A declarer with all four kings can only call a king of their own, and plays alone.

**Talon**
17. After the call the six talon cards are public, in packets: two of three in tri and solo tri, three of two in dve and solo dve, six of one in ena and solo ena. Packets follow the talon's order.
18. The declarer takes one packet into the hand.
19. The declarer then discards the same number of cards, one per action. Kings, škis, mond and pagat are never legal discards.
20. Other taroks may be discarded freely (S14 a). A discarded tarok is public. Other discards are hidden from the other seats.
21. The discards count for the declarer's side. The packets not taken count for the opponents.
    One exception since spec 013 (D133): when the declarer's side takes all twelve tricks, the packets not taken count for the declarer's side. See `docs/specs/013-rulings-of-12-nov.md`, criterion 16.

**Called king in the talon (S12 a)**
22. If the called king is among the talon cards, there is no partner and the declarer plays alone. This is public once the talon is shown.
23. The declarer takes the king's packet, later plays the king and wins that trick: the rest of the talon goes to the declarer's pile.
24. The declarer takes the king's packet, plays the king and loses that trick: the rest of the talon goes to the opponents.
25. The declarer does not take the king's packet: the rest of the talon stays with the opponents.

**Play**
26. Forehand leads the first trick, whoever the declarer is.
27. Legal cards and trick winners are those of spec 002 (checked against `trick::legal_cards` and `trick::winner` on random deals).
28. When the called king is played, the partner is public from that moment.

**Scoring**
29. The declarer's side counts the tricks of the declarer and the partner, the discards, and the rest of the talon in the case of criterion 23. The opponents count everything else. The two counts sum to 70.
30. The declarer's side wins with 36 or more. 35 is a loss.
31. Difference = points − 35, rounded to the nearest 5. Source examples: 43 points is +10; 37 is 0; 21 is −15. For every count 0–70 the rounded value is the multiple of 5 nearest to points − 35.
32. Game values: tri 10, dve 20, ena 30, solo tri 40, solo dve 50, solo ena 60.
33. A win scores game value plus rounded difference. Source example: ena won with 43 points scores 40.
34. A loss scores minus (game value plus the size of the rounded difference). Dve lost with 21 points scores −35. Tri with exactly 35 points scores −10.
35. The declarer and the partner write the same entry (S26 a). The opponents write 0 for the game.
36. Silent trula: +10 if škis, mond and pagat are all in the declarer side's piles; −10 if all three are in the opponents' piles (untaken talon included); else 0.
37. Silent kralji: the same with the four kings, ±10.
38. Silent kralj ultimo: if the called king is played to the last trick, +10 when the declarer's side wins that trick and −10 when the opponents win it. Otherwise 0. Never in a solo contract.
39. Silent pagat ultimo: if the pagat is played to the last trick, the declarer's side scores +25 when their pagat wins it or an opponent's pagat loses it, and −25 when an opponent's pagat wins it or their own pagat loses it. Otherwise 0.
    Changed by spec 013 (D126): the bonus follows the side that takes the last trick, as in criterion 38. See `docs/specs/013-rulings-of-12-nov.md`, criteria 4 to 7.
40. Game and bonuses are independent: a lost game with a silent trula scores the loss plus 10.
41. Silent valat: the declarer's side wins all twelve tricks. The entry is +250 and nothing else is scored (no game, no difference, no other bonus). See open point A for the opposite case.
    Changed by spec 013 (D125): a lost mond is still written, −20 for its player next to the ±250. See `docs/specs/013-rulings-of-12-nov.md`, criteria 1 to 3.
42. Mondfang: škis and mond in the same trick. The player of the mond writes −20, alone, whoever wins the trick, also when the škis was the partner's and also in a trick the pagat wins (S22 a, S29 a, S30 a).
43. An opponent who loses the mond writes −20 although the game entry of opponents is 0.
44. The mond lies in a packet the declarer does not take: the declarer writes −20. Not in the case of criterion 23, where the declarer wins the rest of the talon.
    Also not when the declarer's side takes all twelve tricks (spec 013, D133). See `docs/specs/013-rulings-of-12-nov.md`, criterion 16.
45. Three whole deals written out by hand, with chosen hands and plays, give the expected entries for all four seats. One has a partner, one a solo, one the called king in the talon.

**Whole deals (property tests, 10,000 random deals with random legal actions)**
46. Every deal reaches "over". The legal set is never empty before that.
47. All 54 cards end in the piles of the two sides, each once. The counts sum to 70.
48. The declarer's and the partner's entries are equal apart from a mondfang. Every opponent's entry is 0 or −20.
49. The same seed, dealer and choices give the same result.
50. Every action offered as legal maps to a slot of the encoding contract that belongs to the current phase.

**Quality gates**
51. `cargo fmt --check` and `cargo clippy --locked --all-targets -- -D warnings` pass on Rust 1.94.1. No `unsafe`. The `engine` workflow is green.
52. The benchmark prints whole random deals per second on one thread. The number goes into `docs/progress.md`. No threshold.
53. Spec 002's 47 tests still pass. Its public API changes only where this spec needs it (a rules parameter is not needed yet); each change is listed in the Results.

## Files expected to change

New: `src/engine/src/game.rs`, `bidding.rs`, `contract.rs`, `talon.rs`, `score.rs`, `action.rs`, tests under `src/engine/tests/` (one file per criteria group), a benchmark binary.
Changed: `src/engine/src/lib.rs`, `play.rs` (hooks the game needs), `src/engine/README.md`, this spec (status, Results). `docs/progress.md` and `docs/architecture.md` by the main session.

## Commits

(a) contracts, actions, bidding, tests 1–11; (b) king call and talon, tests 12–25; (c) play and scoring, tests 26–45; (d) property tests 46–50, benchmark. Each with `Spec: docs/specs/005-deal-flow-a.md`.

## Open points for the user

These are places where the source is silent or where your table decides (the Default preset is your table, D43).

- **A. The opponents take all twelve tricks.** The source gives no rule. (a) the declarer scores −250, a silent valat against; (b) a normal lost game with its difference. Proposal: (a).
- **B. Trula and kings lying in the untaken talon** count toward the opponents' silent bonus, since those cards "count as the opponents' tricks" in the source. Agree?
- **C. Kralj ultimo when the declarer called their own king,** or took the called king from the talon: does it score when that king is played to the last trick? Proposal: yes.
- **D. Criterion 39,** the four cases of the silent pagat ultimo. The source says the bonus is lost when the pagat is played to the last trick and fails. Check against your table.
- **E. Criterion 42:** the mond's player pays 20 also in a trick that the pagat wins. This is the pagat.com rule and the current default (S22 a). The TARZS rules say no penalty there. Which does your table play?
- **F. Check criteria 1–11** against how your table bids, above all who speaks first and that only forehand may play tri.

## Answers to the open points (D118)

- A. The opponents take all twelve tricks: the declarer scores −250 and nothing else is scored, the mirror of criterion 41. Add a test for it next to criterion 41.
- B. Yes: trula and kings in the untaken talon count toward the opponents' silent bonus.
- C. Yes: kralj ultimo scores whenever a king was called and that king is played to the last trick, also when the declarer holds it or took it from the talon.
- D. Criterion 39 stands as written.
- E. Criterion 42 stands: the mond's player pays 20 also in a trick the pagat wins (S22 a).
- F. Criteria 1–11 stand as written.

## Results

Executed on 2026-11-10 by a parallel session (D109). Claude Code, model `claude-fable-5-2`. Nothing is staged or committed. The tests were written from the criteria before the code. Local runs used rustc 1.94.1 (`stable-x86_64-pc-windows-gnu`), plain `cargo`.

### Criteria

Test files are in `src/engine/tests/`. Seats in the tests: A is seat 0 (forehand), B, C, D follow; D (seat 3) deals. Several tests repeat a case with another dealer.

| # | Test | File | Result |
|---|---|---|---|
| 1 | `c01_b_speaks_first_then_c_d_a_then_round_the_players_still_in` | `bidding.rs` | pass |
| 2 | `c02_first_speaker_may_pass_or_bid_dve_to_solo_ena` | `bidding.rs` | pass |
| 3 | `c03_lower_priority_must_bid_higher_or_pass` | `bidding.rs` | pass |
| 4 | `c04_higher_priority_may_bid_the_same` | `bidding.rs` | pass |
| 5 | `c05_priority_runs_a_b_c_d` | `bidding.rs` | pass |
| 6 | `c06_a_player_who_passed_gets_no_further_turn` | `bidding.rs` | pass |
| 7 | `c07_auction_ends_after_three_passes` | `bidding.rs` | pass |
| 8 | `c08_forehand_may_equal_raise_or_pass_after_one_bid` | `bidding.rs` | pass |
| 9 | `c09_b_holds_c_bid_then_c_must_go_higher_or_pass` | `bidding.rs` | pass |
| 10 | `c10_three_passes_forehand_must_name_tri_or_higher` | `bidding.rs` | pass |
| 11 | `c11_declarer_names_the_winning_bid_or_a_higher_contract` | `bidding.rs` | pass |
| 12 | `c12_declarer_names_one_of_four_suits_also_an_own_king` | `king_call.rs` | pass |
| 13 | `c13_solo_contracts_have_no_king_call` | `king_call.rs` | pass |
| 14 | `c14_partner_holds_the_called_king_and_stays_hidden_until_it_is_played` | `king_call.rs` | pass |
| 15 | `c15_declarer_with_the_called_king_plays_alone` | `king_call.rs` | pass |
| 16 | `c16_declarer_with_all_four_kings_calls_an_own_king_and_plays_alone` | `king_call.rs` | pass |
| 17 | `c17_talon_is_shown_in_packets_by_contract` | `talon.rs` | pass |
| 18 | `c18_declarer_takes_one_packet_into_the_hand` | `talon.rs` | pass |
| 19 | `c19_declarer_discards_as_many_cards_one_by_one_never_kings_or_trula` | `talon.rs` | pass |
| 20 | `c20_discarded_taroks_are_public_other_discards_hidden` | `talon.rs` | pass |
| 21 | `c21_discards_count_for_declarer_untaken_packets_for_opponents` | `talon.rs` | pass |
| 22 | `c22_called_king_in_the_talon_declarer_alone_and_public` | `talon.rs` | pass |
| 23 | `c23_king_packet_taken_king_wins_its_trick_rest_of_talon_to_declarer` | `talon.rs` | pass |
| 24 | `c24_king_packet_taken_king_loses_its_trick_rest_of_talon_to_opponents` | `talon.rs` | pass |
| 25 | `c25_king_packet_not_taken_rest_of_talon_stays_with_opponents` | `talon.rs` | pass |
| 26 | `c26_forehand_leads_the_first_trick_whoever_declares` (all 16 pairs of dealer and declarer) | `game_play.rs` | pass |
| 27 | `c27_legal_cards_and_trick_winners_are_those_of_spec_002` (2,000 random deals) | `game_play.rs` | pass |
| 28 | `c28_called_king_played_makes_the_partner_public` | `game_play.rs` | pass |
| 29 | `c29_each_side_counts_its_cards_and_the_counts_sum_to_70` (3,000 random deals; the second scorer of the tests collects each pile on its own) | `scoring.rs` | pass |
| 30 | `c30_declarer_wins_with_36_or_more_and_35_is_a_loss` (also four whole deals: tri with 36 writes +10, tri with 35 writes −10, solo tri with 36 writes +40, dve with 35 writes −20) | `scoring.rs` | pass |
| 31 | `c31_difference_is_points_minus_35_rounded_to_the_nearest_5` (the three source examples and every count 0–70) | `scoring.rs` | pass |
| 32 | `c32_game_values` | `scoring.rs` | pass |
| 33 | `c33_a_win_scores_game_value_plus_rounded_difference` (ena with 43 is 40, also as a whole deal: declarer and partner write 40) | `scoring.rs` | pass |
| 34 | `c34_a_loss_scores_minus_game_value_and_difference` (dve with 21 is −35; tri with 35 is −10; both also as whole deals) | `scoring.rs` | pass |
| 35 | `c35_partner_writes_the_same_entry_and_opponents_write_no_game` | `scoring.rs` | pass |
| 36 | `c36_silent_trula` (+10, −10, −10 with the pagat in the untaken talon, 0); `c36_c37_trula_and_kralji_count_the_tricks_of_the_partner` | `scoring.rs` | pass |
| 37 | `c37_silent_kralji` (+10, −10, −10 with a king in the untaken talon, 0); `c36_c37_trula_and_kralji_count_the_tricks_of_the_partner` | `scoring.rs` | pass |
| 38 | `c38_silent_kralj_ultimo` (+10 own king, −10, +10 by the partner, 0 earlier, 0 in a solo); `c38_kralj_ultimo_follows_the_side_that_takes_the_last_trick` | `scoring.rs` | pass |
| 39 | `c39_silent_pagat_ultimo` (the four cases, three cases with a higher tarok of the pagat's own side in the trick, and 0) | `scoring.rs` | pass |
| 40 | `c40_game_and_bonuses_are_independent` | `scoring.rs` | pass |
| 41 | `c41_silent_valat_scores_250_and_nothing_else`; answer A: `c41_opponents_take_all_twelve_tricks_scores_minus_250` (each alone and with a partner); `c41_valat_counts_the_tricks_of_the_side_not_of_the_declarer` | `scoring.rs` | pass |
| 42 | `c42_mondfang_the_mond_player_writes_minus_20` (opponent's škis, partner's škis, a trick the pagat wins); `c42_a_partner_who_loses_the_mond_pays_alone` | `scoring.rs` | pass |
| 43 | `c43_an_opponent_who_loses_the_mond_writes_minus_20` | `scoring.rs` | pass |
| 44 | `c44_mond_in_a_packet_not_taken_costs_the_declarer_20` (also: not in the case of criterion 23) | `scoring.rs` | pass |
| 45 | `c45_written_deal_1_partner_game_tri`, `c45_written_deal_2_solo_tri_lost`, `c45_written_deal_3_called_king_in_the_talon` | `written_deals.rs` | pass |
| 46 | `c46_every_deal_reaches_over_and_the_legal_set_is_never_empty_before` (10,000 deals; at most 13 calls and 67 actions) | `whole_games.rs` | pass |
| 47 | `c47_all_54_cards_end_in_the_two_piles_and_the_counts_sum_to_70` (10,000 deals) | `whole_games.rs` | pass |
| 48 | `c48_declarer_and_partner_write_the_same_and_opponents_0_or_minus_20` (10,000 deals) | `whole_games.rs` | pass |
| 49 | `c49_same_seed_dealer_and_choices_give_the_same_result` (10,000 deals). It can only fail if the engine is not deterministic; that is what the criterion asks | `whole_games.rs` | pass |
| 50 | `c50_every_legal_action_maps_to_a_slot_of_the_current_phase` (10,000 deals) | `whole_games.rs` | pass |
| 51 | `cargo fmt --check`; `cargo clippy --locked --all-targets -- -D warnings`; `#![forbid(unsafe_code)]` in `lib.rs` and the binaries | | pass locally on 1.94.1. **The workflow is not verified:** nothing is pushed |
| 52 | `src/engine/src/bin/bench_game.rs`, release mode, one thread | | pass: about 350,000 whole deals per second. The entry in `docs/progress.md` is left to the main session |
| 53 | The 47 tests of spec 002 | | pass, all 47, unchanged. API changes are listed below |

The three written deals (criterion 45) are in `tests/common/written.rs`: all hands, the talon, the bids, the discards and the 48 plays, with the paper count next to each deal. Expected entries: deal 1 (tri with a partner, 52 points, silent trula) A 35, C 35, B 0, D 0; deal 2 (solo tri lost with 31 points, silent trula) B −35, the others 0; deal 3 (dve, called king in the talon and won, 54 points, silent kralji, B loses the mond) D 50, B −20, A 0, C 0. The engine gave these entries on the first run.

Tests without a criterion number:

- `tests/actions.rs` (6): the 160 slots against `docs/encoding.md` section 5, `ActionSet`, `Deal::new`, the constructors, every illegal action in every phase of a deal is refused and changes nothing.
- `tests/views.rs` (3): the debug text of the public view, and of the private view of a seat that is not involved, does not change when hidden cards change hands, when the unshown talon differs, when hidden discards differ, or when the true partner differs.
- `tests/bidding.rs` (2): `every_auction_agrees_with_a_model_of_criteria_1_to_11` (all 1,297 auctions for each dealer) and `longest_auction_of_this_slice_and_of_the_full_ladder`.
- `tests/whole_games.rs` (2): `score_matches_the_second_scorer_on_random_deals` (10,000 deals) and `random_deals_cover_contracts_seats_and_partner_cases`.
- `tests/scoring.rs` (5): the `reading2_…` tests, see "Review fixes" item 7.
- Seven unit tests in `src/` and one doc test.

130 tests in all: the 47 of spec 002 and 83 new (115 before the review fixes).

### Check of the tests

32 wrong rules were put into the code one at a time. After each one the whole suite was run and the file was restored. The table lists the numbered tests that failed.

| # | Wrong rule | Criteria whose tests failed |
|---|---|---|
| 1 | A lower seat may hold a bid | 1, 3, 4, 5, 6, 8, 9 (and 30, 34, 40, 45 through deal 2) |
| 2 | Forehand may pass after three passes | 10, 27, 29, 35, 46–50 |
| 3 | Tri open to the first speaker | 2 |
| 4 | The contract choice allows a lower contract | 11 |
| 5 | A king call also in solo tri | 13 (and nine others) |
| 6 | The partner is public right after the call | 14, 15, 22, 28 |
| 7 | A declarer who holds the called king still gets a partner | 15, 16 |
| 8 | Dve cuts the talon in packets of three | 17, 18, 19, 21 (and four others) |
| 9 | Kings may be discarded | 19, 29 |
| 10 | All discards are shown | 20 |
| 11 | The declarer sees no own discards | 19, 20 |
| 12 | The rest of the talon is won even if the king loses its trick | 24, 29, 44 |
| 13 | The rest of the talon is never won | 23, 29, 33, 44, 45 |
| 14 | The declarer leads the first trick | 26, 27 (and six others) |
| 15 | 35 points win | 30, 32, 34 |
| 16 | The difference rounds toward zero | 31, 33, 34, 40, 45 |
| 17 | Ena is worth 40 | 32, 33, 34 |
| 18 | The partner writes nothing | 35, 38, 41, 42, 45, 48 |
| 19 | The trula and kings against ignore the untaken talon | 36, 37 |
| 20 | Kralj ultimo also without a called king | 38 |
| 21 | Kralj ultimo is always plus | 38 |
| 22 | An opponent's lost pagat costs the declarer | 39 |
| 23 | The pagat "wins" when its side takes the last trick | none at first; 39 after a fix, see below |
| 24 | A valat does not cancel the game | 41 |
| 25 | No valat against | 41 (the test for answer A) |
| 26 | The škis player pays the mondfang | 42, 43, 45 |
| 27 | No mondfang in a trick the pagat wins | 42 |
| 28 | No mondfang for the mond left in the talon | 44 |
| 29 | Mondfang for the talon mond even when the rest is won | 44 |
| 30 | The discards count for the opponents | 21, 24, 29, 33, 34, 45, 47 |
| 31 | The play slots start at 89 | only unnumbered tests at first; 50 after a fix, see below |
| 32 | An illegal action is applied in the bidding | 3, 10 |

Two gaps showed and were closed:

- **Rule 23.** No test had the pagat in the last trick together with a higher tarok of the same side. `c39` now has three more cases on a new built deal: an opponent's pagat that loses to another opponent (+25), the partner's pagat that loses to an opponent (−25), and the partner's pagat in a trick the declarer takes (−25, reading 3 below).
- **Rule 31.** `c50` took the slot ranges from the engine's own constants. It now has the ranges written out from `docs/encoding.md`.

After the two fixes rules 23 and 31 were run again: rule 23 fails `c39`, rule 31 fails `c50` (and three unnumbered tests). So each of the 32 wrong rules fails at least one numbered test.

One wrong rule (2) made the auction endless and the first run of the check hung. The test helpers now stop with a failure after 200 actions, and the turn search in `bidding.rs` is bounded.

### Review fixes (2026-11-10, after the D72 step 4 review)

The review found one bug and gaps in the tests. What changed, by item:

1. **Bug: hidden information in the debug text.** `PublicView` and `PrivateView` derived `Debug` over the whole `Game`, so `format!("{:?}", game.public())` printed all hands, the unshown talon, the discards and the partner. Both now have a hand-written `Debug`: the public view prints what its accessors show, the private view the seat, its hand and its own discards. `PublicView` also got `phase()` and `to_act()`. New file `tests/views.rs` (3 tests): the debug text of the public view stays the same, step by step into the play, when a card of B and a card of C change hands (40 seeds), when the unshown talon differs, when the hidden discards differ, and when the true partner differs. Proof: four leaks were put back one at a time (the debug text printing the hands, the true partner, all discards, the unshown talon); each failed at least one test of `tests/views.rs`.
2. **Scoring gaps.** A second scorer for the tests, `tests/common/second_scorer.rs`, written from criteria 29–44 and the D118 answers. It does not call `score.rs` or the pile accessors: it collects both piles from the tricks, the discards and the talon and works out the eight parts seat by seat. `score_matches_the_second_scorer_on_random_deals` compares it with `Game::score` on 10,000 random deals (both piles, both counts, all parts of all seats); `c29` uses it too. One built case for each wrong rule of the review:
   - (a) valat counted on the declarer's own tricks: `c41_valat_counts_the_tricks_of_the_side_not_of_the_declarer` (the partner takes all twelve, the declarer none: +250 for both; and the partner takes one, the opponents eleven: no valat).
   - (b) kralj ultimo only when the king itself wins: `c38_kralj_ultimo_follows_the_side_that_takes_the_last_trick` (the partner's king in the last trick, taken by the declarer's tarok: +10).
   - (c) the declarer pays the partner's mondfang: `c42_a_partner_who_loses_the_mond_pays_alone`.
   - (d) trula and kralji ignore the partner's tricks: `c36_c37_trula_and_kralji_count_the_tricks_of_the_partner`.

   Proof: each of the four wrong rules was put into `score.rs`. Each failed its built test, `c29` and the second-scorer test: (a) also `reading2_valat_and_the_declarers_mond_taken_by_the_partner`; (b) also `c38_silent_kralj_ultimo`.
3. **End to end.** Whole deals whose final entries show the source examples: tri with 36 points writes +10 and tri with 35 writes −10 (declarer and partner); solo tri with 36 writes +40; dve with 35 writes −20; ena with 43 writes 40; dve with 21 writes −35. They are fixed random deals (`seeded_game(seed)`, seeds 548, 69, 1049, 476, 115, 5083), found by trying seeds for a deal with the wanted contract and count and no bonus. The count in each is taken from the second scorer, not from the engine. In `c30`, `c33`, `c34`.
4. **Every auction.** `every_auction_agrees_with_a_model_of_criteria_1_to_11` walks all 1,297 auctions for each of the four dealers. At every node the seat to speak, the legal calls, the record and the passes must agree with a small model written in the test from criteria 1–11 (it keeps only the list of calls); at the end of each auction the declarer, the winning bid and the legal contract choices.
5. **Counts that could not fail.** In `c21`, `c23`, `c24` and `c47` the two piles are now collected one by one by the second scorer and each is counted by itself; the engine's counts must match both, and the sum of the two independent counts must be 70.
6. **The auction bound.** 13 calls, not 23; corrected in `bidding.rs`, in `c46` (13 calls, 67 actions) and in open issue 5, with the number for the full ladder (19 calls).
7. **Reading 2 pinned.** Behaviour unchanged. Five tests fix what the code does now: `reading2_valat_and_an_opponent_who_loses_the_mond`, `reading2_valat_and_the_declarers_mond_taken_by_the_partner`, `reading2_valat_against_and_the_declarers_lost_mond`, `reading2_valat_against_and_the_mond_left_in_the_talon`, and a fourth case the review did not list, `reading2_valat_against_and_an_opponents_mond_taken_by_an_opponent`. The second scorer has one switch per case at the top of its file. Proof: with the valat branch changed so that the rest is still scored, all five fail (and the three `c41` tests).

### Command output (trimmed)

Run in `src\engine`.

```
> cargo fmt --check
(no output, exit 0)

> cargo clippy --locked --all-targets -- -D warnings
    Checking tarok-engine v0.1.0 (C:\Users\Bush\Desktop\agentic project\src\engine)
    Finished `dev` profile [unoptimized + debuginfo] target(s)

> cargo test --locked
     Running unittests src\lib.rs
test result: ok. 14 passed; 0 failed
     Running tests\actions.rs
test result: ok. 6 passed; 0 failed
     Running tests\bidding.rs
test result: ok. 13 passed; 0 failed
     Running tests\cards_and_counting.rs
test result: ok. 7 passed; 0 failed
     Running tests\cardset.rs
test result: ok. 5 passed; 0 failed
     Running tests\deal.rs
test result: ok. 5 passed; 0 failed
     Running tests\game_play.rs
test result: ok. 3 passed; 0 failed
     Running tests\king_call.rs
test result: ok. 5 passed; 0 failed
     Running tests\legal_cards.rs
test result: ok. 7 passed; 0 failed
     Running tests\scoring.rs
test result: ok. 26 passed; 0 failed
     Running tests\talon.rs
test result: ok. 9 passed; 0 failed
     Running tests\trick_winner.rs
test result: ok. 10 passed; 0 failed
     Running tests\views.rs
test result: ok. 3 passed; 0 failed
     Running tests\whole_deals.rs
test result: ok. 4 passed; 0 failed
     Running tests\whole_games.rs
test result: ok. 7 passed; 0 failed
     Running tests\written_deals.rs
test result: ok. 3 passed; 0 failed
   Doc-tests tarok_engine
test result: ok. 2 passed; 0 failed
test result: ok. 1 passed; 0 failed
```

This is the final run, after the review fixes. The tests take about 10 seconds to run in a debug build.

### Benchmark

Command, in `src\engine`:

```
cargo run --release --bin bench_game
```

500,000 deals (seeds 0 to 499,999, the dealer is seat `seed mod 4`), every action drawn uniformly from the legal actions, one thread. The time covers the deal, the legal sets, the choice, the actions and the score. Machine: AMD Ryzen 5 5600, Windows 11. Default release profile. A training run of the other session was using the machine at the same time.

| Run | Seconds | Whole deals per second | Actions per second |
|---|---|---|---|
| 1 | 1.411 | 354,401 | 20,144,964 |
| 2 | 1.382 | 361,879 | 20,569,987 |
| 3 | 1.433 | 349,032 | 19,839,739 |

Number for `docs/progress.md`: **about 350,000 whole random deals per second** on one thread (56.8 actions per deal on average). These are the runs on the final code. An earlier run, while the training run was busier, gave about 300,000; `bench_play` of spec 002 gave 30.7 million card plays per second at that moment (35 million on an idle machine).

### Public API (new in this spec)

- `Contract`: the twelve contracts in bidding order (`Klop` … `Valat`); `ALL`, `NORMAL`, `index`, `from_index`, `is_normal`, `calls_king`, `packet_size`, `game_value`.
- `Action`: `Pass`, `Bid(Contract)`, `CallKing(Suit)`, `TakePacket(Packet)`, `Discard(Card)`, `Play(Card)`; `slot()`, `from_slot(usize)`, `SLOTS` (160). `action::slots`: `BIDS` 0–12, `KINGS` 13–16, `PACKETS` 17–22, `DISCARDS` 23–76, `PLAYS` 90–143. Slots 77–89 and 144–159 have no action yet.
- `Packet`: `new(0..=5)`, `index`, `ALL`.
- `ActionSet`: a 160-bit mask. `EMPTY`, `insert`, `contains`, `contains_slot`, `len`, `is_empty`, `iter`, `nth`, `words`, `discards(CardSet)`, `plays(CardSet)`.
- `Game`: `new(&Deal, dealer)`, `from_seed(seed, dealer)`, `phase()`, `to_act() -> Option<Seat>`, `legal_actions() -> ActionSet`, `apply(Action) -> Result<(), GameError>`, `score() -> Option<Score>`, `public()`, `private(seat)`, `full()`.
- `Phase`: `Bidding`, `ContractChoice`, `KingCall`, `TalonPick`, `Discard`, `Play`, `Over`. `GameError`: `Over`, `NotLegal(Action)`.
- `PublicView`: `phase`, `to_act`, `dealer`, `forehand`, `bids`, `has_passed`, `highest_bid`, `declarer`, `winning_bid`, `contract`, `called_suit`, `called_king`, `talon`, `packet_size`, `packet`, `taken_packet`, `taken_cards`, `discards_made`, `shown_discards`, `king_in_talon`, `partner() -> PublicPartner`, `tricks`, `current_trick`, `leader`, `tricks_won(seat)`, `hand_size(seat)`.
- `PublicPartner`: `NotKnown`, `Alone`, `With(Seat)`.
- `PrivateView`: `seat`, `hand`, `discards` (the declarer's own; empty for the others).
- `FullView`: `hands`, `hand(seat)`, `talon`, `partner() -> Option<Seat>`, `on_declarer_side(seat)`, `discards`, `talon_rest`, `talon_rest_won`, `declarer_pile`, `opponents_pile`.
- `Score`: `parts(seat) -> &ScoreParts`, `entry(seat)`, `entries() -> [i32; 4]`, `declarer_points`, `opponents_points`, `declarer_won`.
- `ScoreParts`: `game`, `difference`, `trula`, `kralji`, `kralj_ultimo`, `pagat_ultimo`, `valat`, `mondfang`; `entry()` is their sum.
- `score::rounded_difference(points)`, `score::game_and_difference(contract, points)`, the bonus constants.
- `Bidding`, `BidEvent { seat, bid: Option<Contract> }`: the auction on its own (read-only from outside).
- `talon::packet_cards`, `packet_count`, `discardable`, `NEVER_DISCARDED`, `TRULA`, `KINGS`.

The constructor for tests with chosen hands, talon and dealer is `Game::new(&Deal::new(hands, talon)?, dealer)`.

### API changes to spec 002

- Added: `Deal::new(hands: [CardSet; 4], talon: [Card; 6]) -> Result<Deal, DealError>` and `DealError::NotWhole`. It checks twelve cards per hand and every card once, so a `Deal` stays whole by construction and `PlayState::from_deal` still cannot fail.
- Nothing was changed or removed. `play.rs` needed no hooks: the game reads `PlayState`'s own accessors. Two doc comments in `deal.rs` and `play.rs` now name `Deal::new`.
- `Deal::from_seed` and the golden deals are unchanged. The dealer is a parameter of `Game` only.

### How the hidden information is kept apart

- `PublicView` has no accessor for a hand or for the hidden discards. It shows the talon only from the talon pick on, the taken packet, the number of discards, the discarded taroks, and the partner only as `PublicPartner`. Its `Debug` is written by hand and prints these things only (review fix 1); so is the `Debug` of `PrivateView`. `Game` and `FullView` print everything.
- `PrivateView` adds one seat's hand, and the discards for the declarer only.
- The true partner, all hands, the unshown talon and all discards are in `FullView` only. `score()` uses it.
- `tests/views.rs` checks it: two deals with the same public history and different hidden cards give the same debug text of the public view, step by step into the play. The same test on the encoded observation belongs to spec 006.

### Readings and deviations

1. **Forehand after three passes acts twice.** Forehand first names a contract as a bid in the bidding (criterion 10), then names the contract to play in the contract choice like every declarer (criterion 11): the bid or a higher one. Every deal so has exactly one contract-choice decision.
2. **A valat cancels every mondfang, on both sides.** Criterion 41 says "nothing else is scored (no game, no difference, no other bonus)"; research/01 B3 says "Valat cancels all other scores of the hand". When one side takes all twelve tricks, the code writes ±250 and no −20 for anyone. That covers four cases: (i) the declarer's side takes all twelve and an opponent loses the mond to the škis; (ii) the declarer's side takes all twelve and one of the two loses the mond to the other's škis; (iii) the opponents take all twelve and the declarer loses the mond in a trick or leaves it in the talon; (iv) the opponents take all twelve and one of them loses the mond to another opponent's škis. One place in `score.rs`. Each case has a `reading2_…` test that fixes the present behaviour. The main session puts the cases to the user; the behaviour is unchanged until then. See question 1.
   Replaced by spec 013 (D125): a valat does not cancel a lost mond, in any of the four cases. The five `reading2_…` tests are now the `d125_…` tests. See `docs/specs/013-rulings-of-12-nov.md`.
3. **"The pagat wins the last trick"** is read as: the pagat is the card that takes the trick. If the partner's pagat is in the last trick and the declarer takes that trick with a higher tarok, that is "own pagat loses", −25. The last case of `c39` fixes this reading. See question 2.
   Replaced by spec 013 (D126): pagat ultimo follows the side that takes the last trick, also when it takes it with another card. See `docs/specs/013-rulings-of-12-nov.md`, criteria 4 to 7.
4. **Kralj ultimo follows criterion 38 as written:** the side that wins the last trick decides, not the king itself. A partner's king taken by the declarer's tarok in the last trick is +10.
5. **Random test deals are not uniform in the bidding.** With uniform choices nearly every auction climbs to solo ena; of 3,000 random deals 5 had a partner. The test player takes the lowest legal action three times out of four in the bidding and in the contract choice (`tests/common/flow.rs`, `play_random`). A coverage test with fixed seeds checks that all six contracts, all four declarer positions, partner games, own-king games, the king in the talon and the mondfang all occur.
6. **Files beyond the expected list:** `tests/common/flow.rs` and `tests/common/written.rs` (helpers and the written deals), `tests/actions.rs`. No `tests/contract.rs`: contract values are tested in `scoring.rs` (criterion 32).

### Spec against research/01

No criterion contradicts research/01 B3 or rule-switches sections 1 and 7. Three places where the spec says more than the research file, all followed as the spec has them:

- Opponents taking all twelve tricks: −250 (answer A). The research file has no rule.
- Trula and kings in the untaken talon count for the opponents' silent bonus (answer B). The research file says only that those cards go to the opponents.
- The four cases of the silent pagat ultimo and the two of kralj ultimo (criteria 38, 39). The research file gives the values only.

One place where the research file is wider than the spec: "Valat cancels all other scores of the hand" (reading 2 above).

### Open issues

1. **Criterion 51, the workflow, is not verified.** It needs a push. The workflow file was not touched.
2. **The benchmark ran next to a training run** of the other session. Three runs gave 349,000 to 362,000; an earlier three gave about 300,000. On an idle machine the number may be higher.
3. **`Game::apply` contains two `expect` calls** (`PlayState::new` after the exchange, `PlayState::play` for a legal card). Both are unreachable while a `Deal` is whole and legality is checked first. Every deal of the test suite goes through both.
4. **No rules parameter yet.** Klop, berač and the toggles will need one in `trick::legal_cards`, the bidding and the score (flows B and C). The bidding already refuses every contract that is not normal in one place (`Bidding::may_bid`).
5. **The longest auction of this slice has 13 calls**, not 23 as first written here. There are 1,297 auctions; the test walks them all for each dealer. One of the longest: B dve, C pass, D pass, A dve, B ena, A ena, B solo tri, A solo tri, B solo dve, A solo dve, B solo ena, A solo ena, B pass. **For the full ladder of the Default preset the longest auction has 19 calls** (36,182 auctions), counted by the model of the test under these assumptions: dve, ena, the three solos, berač, solo brez and valat for all seats; klop and tri for forehand after three passes; odprti berač and barvni valat off; the same bidding rules as in this slice. `docs/encoding.md` counts "about 24 bids" toward the 128-token limit, so 19 fits. The array in `bidding.rs` keeps room for 51 calls, a limit that needs no assumption.

### Questions for the user

1. Valat and mondfang (reading 2). The four cases are listed there; the main session asks about them one by one. For each: (a) the valat cancels the −20; (b) the −20 is still written next to ±250. The code does (a) in all four. A ruling changes one `reading2_…` test per case, one switch in `tests/common/second_scorer.rs` and the valat branch in `score.rs`.
2. Pagat ultimo when the pagat's own side wins the last trick but not with the pagat (reading 3). Options: (a) it counts as lost, −25 for that side; (b) it counts as won. Recommendation: (a), the usual meaning of "pagat ultimo": the pagat itself must take the last trick.
3. Kralj ultimo when the called king is in the last trick and the declarer's side wins it with another card (reading 4). The criterion says +10. Is that how your table plays it, or must the king itself win the trick? Recommendation: keep the criterion.
