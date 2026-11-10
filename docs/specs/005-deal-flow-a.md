# 005 — Deal flow A: bidding, king call, talon, scoring of the normal contracts

Status: **approved** by the user on 2026-11-10 (D118). Needs spec 002 (done).

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
40. Game and bonuses are independent: a lost game with a silent trula scores the loss plus 10.
41. Silent valat: the declarer's side wins all twelve tricks. The entry is +250 and nothing else is scored (no game, no difference, no other bonus). See open point A for the opposite case.
42. Mondfang: škis and mond in the same trick. The player of the mond writes −20, alone, whoever wins the trick, also when the škis was the partner's and also in a trick the pagat wins (S22 a, S29 a, S30 a).
43. An opponent who loses the mond writes −20 although the game entry of opponents is 0.
44. The mond lies in a packet the declarer does not take: the declarer writes −20. Not in the case of criterion 23, where the declarer wins the rest of the talon.
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
