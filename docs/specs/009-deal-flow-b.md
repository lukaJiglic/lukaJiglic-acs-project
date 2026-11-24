# 009 — Deal flow B: announcements and kontra

Status: **approved** (2026-11-24, D143), with the answers to Batch 13 (D141) and the wait of D142 written in. In execution in the `engine-core` session on the branch `spec-009`. Needs specs 005, 006 and 013 (done).

Build-plan row 009. Tool and model: Claude Code 2.1.306, `claude-fable-5-2`.

## Goal

The round of announcements between the talon exchange and the first lead, for the six normal contracts: announced trula, kralji, kralj ultimo, pagat ultimo and valat, and kontra in four levels on the game and on each announced bonus. Their scoring. The observation, the action slots and the game record carry them, so the trainer can learn them (D107).

Second of the three rule slices (D108). Klop, berač, solo brez, valat as a contract and radelci are flow C (spec 010).

Rules source: pagat.com, Slovenian Tarok (`https://www.pagat.com/tarot/sltarok.html`, page dated 31 October 2020), read again on 2026-11-24 for the round of announcements, the bonus table, kontra and the three worked examples. `docs/planning/research/01` section B3. The Default preset in `docs/planning/rule-switches.md`: S15 a, S16 a, S18 a, S20 a, S21 a. No code from other tarok engines (D9).

What the source says and what it leaves open is listed under "Source check". Every gap is an open point or a reading of Claude, marked as such.

## In scope

- `Phase::Announcements` in `Game`, after the last discard and before the first lead, in the six normal contracts.
- Actions for the slots 77–88 of `docs/encoding.md`: announce (5), kontra per target (6), done (1). Slot 89 stays never legal.
- No duty to keep an announced ultimo card (D141, B13-3 b): the legal cards of the play do not change.
- A hold flag per table in the environment's step, so the trainer can make a finished table wait (D142, B14-1 a; criteria 50 to 52).
- `score()`: announced bonuses, announced valat, the kontra multipliers. The ten score parts keep their order.
- The observation: phase id "announcements", the tokens 146–156, state block D (41 features), the action mask. The critic input does not change in size.
- The game record: the decisions of the round are in it like every other decision. `engine_version` becomes `0.3.0`.
- D135: replay of a record written by another engine version keeps going and names both versions when the entries differ.
- The binding module is rebuilt; the Python tests pass; the CI config of spec 008 runs with the round in play.
- `docs/encoding.md` gets the readings of this spec in section 4a. The version stays 1 if open point A is answered (a).

## Out of scope

Klop, berač, solo brez, valat as a contract, barvni valat, radelci (flow C). Non-default values of S15 to S21. Kontra in klop (S19 a: none). A training run on flow B: its own spec, with its own config. Bots.

## Order of work

Spec 015 runs in the shared checkout on the installed binding module, which checks the engine's sources on disk at every start. So this spec is built on the branch `spec-009` in its own worktree (`.worktrees/spec-009`, D77), not in the shared checkout.

- Stages (a) to (c) and (e) are Rust only: `cargo` in the worktree. The session commits each stage on the branch.
- **The binding module is not rebuilt into the shared `.venv`** (`maturin develop`) before the main session says that the GPU work of spec 015 is finished. Stage (d), criteria 45 and 46, waits for that.
- The main session verifies, orders the review pass, and merges the branch into `main`.

## Source check

Checked on the page on 2026-11-24:

- The round comes after the talon exchange and before the play. "Beginning with the declarer", each player passes or makes "one or more announcements" for their side. It ends when three players have passed in a row. The order after the declarer is not given; the page says that in practice there is no order.
- Values, silent / announced: trula 10 / 20, kralji 10 / 20, kralj ultimo 10 / 20, pagat ultimo 25 / 50, valat 250 / 500. Kralj ultimo only by the called player. Pagat ultimo only by the holder of the pagat.
- A valat removes every other score of the hand.
- An announced ultimo is lost when the card is played to the last trick and does not win, when it is forced out earlier, and when the side does not hold the card. The side must keep the card as long as the rules of play allow. After an announced valat this duty ends. (The user's table does not enforce the duty: D141, B13-3 b.)
- Kontra on the game is said by a defender and doubles game and difference. Then rekontra by the declarer's side, subkontra by the defenders, mordkontra by the declarer's side: 16 times at the end. A bonus that was announced can be kontra'd by an opponent of the announcer, with the same four levels. Game and each bonus are doubled separately. The speaker names the target.
- Nobody may kontra a partner. A kontra on an announcement is illegal unless the speaker knows for certain that the announcer is on the other side. When a player other than the declarer announces, "it may not be clear which team they belong to".
- Bonuses exist only in the six normal contracts.
- The captured mond is not touched by kontra or radelc.
- Three worked examples with announcements (criterion 31). No worked example with kontra.

Not on the page, so decided here or asked: whether both sides may announce the same bonus (A); what an announced valat that fails costs (E); whether a silent bonus can stand next to an announced one of the same kind (criterion 20); the order of the seats after the declarer (F).

## Acceptance criteria

The user approves these before tests are written (D73). Seats in the examples: A is forehand, then B, C, D in order of play; D is the dealer. Each numbered scenario is one named test.

**The round**
1. After the last discard the phase is "announcements" in tri, dve, ena, solo tri, solo dve and solo ena. The declarer has the first turn. Then the seats follow in order of play (open point F).
2. A turn is any number of announcements and kontras, one per action, and then "done" (slot 88). "Done" is always legal in the round. A turn of "done" alone is a pass.
3. The round ends when every seat has had a turn and the last three turns were passes. Four passes at the start end it. After a turn with an action the round goes on until three passes follow.
4. After the round forehand leads the first trick, also after an announced valat.
5. A deal in which all four pass plays and scores as in specs 005 and 013. Their tests pass with nothing changed but the four passes, added in the test helper.

**Announcements**
6. Trula, kralji and valat: every seat may announce, for its side.
7. Kralj ultimo: only the seat that holds the called king, the declarer included. Never in a solo contract. Never when the called king lies in a packet that was not taken.
8. Pagat ultimo: only the seat that holds the pagat. Nobody when the pagat lies in a packet that was not taken.
9. Each of the five can be announced once in a deal. After that it is not legal for any seat (open point A).
10. An announcement by a seat other than the declarer does not show that seat's side. One exception: kralj ultimo by another seat shows that seat as the partner, from that action on.

**Kontra**
11. Six targets: the game and each of the five bonuses. Four levels on each: kontra, re, sub, mort (S18 a). One action raises one target by one level.
12. The game belongs to the declarer's side. A bonus belongs to the side of the seat that announced it. Kontra and sub are said by the other side, re and mort by the owning side.
13. A bonus that was not announced cannot be kontra'd. Mort cannot be raised.
14. The game: an opponent of the declarer may say kontra and sub; the declarer or the partner may say re and mort. A partner who says re or mort on the game is public from that action on.
15. A bonus: a seat may raise it only when it knows the announcer's side (open point B). It knows it when the announcer is the declarer, when the announcer is the seat itself, when the partner is public, or when the seat itself holds the called king.
16. A seat's legal actions depend only on what that seat knows. Test: two deals with the same public history and the same hand of seat Y, in which another seat is once the partner and once an opponent, give Y the same legal set at every turn of the round.

**No duty to keep the card (D141, B13-3 b)**
17. A seat that announced pagat ultimo or kralj ultimo may play that card whenever the rules of play allow it. The legal cards of the play are those of spec 002, with and without an announcement. A card played before the last trick loses the announced bonus (criteria 21 and 22).

**Scoring.** Every amount is written by the declarer and the partner (S26 a); the opponents write 0 for it (S20 a). "For the owner" means plus when the declarer's side owns the bonus and minus when the opponents own it.
18. Announced trula: 20 for the owner when the owner's side has škis, mond and pagat in its piles at the end. Otherwise 20 against the owner.
19. Announced kralji: the same with the four kings.
20. An announced bonus replaces the silent one of its kind. An announced trula that fails while the other side has all three costs 20, not 30.
21. Announced kralj ultimo: +20 when the called king is played to the last trick and the declarer's side takes that trick. −20 in every other case.
22. Announced pagat ultimo: 50 for the owner when the pagat is played to the last trick and the owner's side takes that trick, also with another card (D126; open point D). 50 against the owner in every other case.
23. Announced valat: 500 for the owner when the owner's side takes all twelve tricks. Otherwise 500 against the owner. In both cases nothing else is scored: no game, no difference, no other bonus, announced or silent (open point E). A lost mond is still written (D125).
24. A silent valat next to announced bonuses: ±250 and no other bonus, also not the announced ones. A lost mond is still written.
25. A bonus that nobody announced is scored as in flow A.
26. Kontra on the game multiplies game value plus rounded difference by 2, 4, 8, 16. It multiplies nothing else.
27. Kontra on a bonus multiplies that announced bonus by 2, 4, 8, 16, won or lost. It multiplies nothing else.
28. Silent bonuses and the silent valat are never multiplied (S21 a). The mondfang is never multiplied.
29. When a valat is scored, a kontra on the game or on another bonus has no effect. A kontra on the announced valat multiplies the ±500.
30. Kontra examples, each a whole deal: tri won with 43 points writes 20; with kontra 40, re 80, sub 160, mort 320. Tri lost with 21 points and kontra writes −50. Announced trula lost under kontra costs 40. Game under kontra, announced trula won without kontra: twice game and difference, plus 20.
31. The three examples of the source, without the radelc doubling of flow C: dve with 35 points and an announced pagat ultimo that fails writes −70. Solo tri won by 15 with an announced trula writes 75. Ena won by 10, kralj ultimo announced by the partner and won, silent trula: 70; an opponent whose mond is captured writes −20.
32. The ten score parts of a seat add up to its entry. A multiplied part holds the multiplied amount.
33. Three whole deals written out by hand give the expected entries for all four seats: one with an announced bonus under kontra, one with an announced valat that fails, one with an announcement by the opponents.

**Observation, actions, record**
34. Slots 77–81 are the announcements, 82–87 the kontra targets, 88 "done", in the order of `docs/encoding.md` section 5. Every legal action of the round maps to one of them.
35. Every announcement and every kontra writes one token (ids 146–156) with the acting seat. "Done" writes no token (reading 1).
36. State block D: D1 holds each announcement on the side that owns it, as far as the observing seat knows the side by the rule of criterion 15; an announcement of unknown side sets both sides' bits (reading 2). D2 holds the level of each of the six targets.
37. No legal deal has more than 128 tokens. Test: 10,000 random deals, and one built deal with the longest round (5 announcements and 24 kontras).
38. Hidden sides stay hidden. Test as in spec 006: the observation of a seat does not change when the true partner changes and the seat's own hand and the public history do not.
39. A record of a deal with announcements and kontras replays to the same entries and parts. `engine_version` is `0.3.0`.
40. D135: replay of a record with another `engine_version` runs to the end when its decisions are still legal. When the entries differ, or a decision is not legal, the error names both versions.

**Whole deals (property tests, 10,000 random deals)**
41. Every deal reaches "over". The legal set is never empty before that. The round is bounded: at most 29 announcement and kontra actions in a deal.
42. The second scorer of the tests, extended from criteria 18 to 29, agrees with `score()` on all parts of all seats.
43. The entries of the declarer and the partner are equal apart from a mondfang. Every opponent's entry is 0 or −20.
44. With fixed seeds the random deals cover: each of the five announcements, by the declarer, by the partner and by an opponent where the rules allow it; each kontra level on the game and on a bonus; an announced valat won and lost; an announced ultimo card played before the last trick.

**Training side**
45. The binding module builds from the new sources. `pytest` in `src\train` passes on the CPU and with the GPU tests. `smoke-a-ci.toml` runs with the round in play; the entropy of the phase "announcements" is logged.
46. Reported, no threshold: whole random deals per second (`bench_game`), decisions and tokens per seat-deal (mean and largest), the share of a deal's decisions that fall in the round.

**Quality gates**
47. `cargo fmt --check` and `cargo clippy --locked --all-targets -- -D warnings` pass on Rust 1.94.1. No `unsafe`. The `engine` and `train` workflows are green.
48. Every earlier test passes. Each one that had to change is listed in the Results with the reason.
49. Check of the tests: at least 30 wrong rules, put into the code one at a time, each fail a numbered test. The list is in the Results.

**The wait for aligned updates (D142, B14-1 a)**
50. The environment's step takes a hold flag per table. A held table does not move: the same observation and seat to act after the step, no reward, not done. Its action is ignored and need not be legal.
51. Holding never changes what a table plays. Test: 200 tables with random legal actions, once without holds and once with random holds; every table's sequence of deals, records and entries is the same.
52. The binding passes the flag through: `step(actions)` works as before, `step(actions, hold)` holds. A test in the binding's tests.

## Readings chosen by Claude

These are not table rules. They are choices about the machine side. Say so if one should be different.

1. **"Done" writes no token.** With a token per "done" a legal deal can reach about 190 tokens; the limit is 128. Without it the longest deal has 103 (26 before the round, 29 in it, 48 plays). The passes are not lost to the network through the tokens of what was said; what a seat did not say is not shown.
2. **An announcement of unknown side sets both bits of D1.** The token carries the seat. The side becomes one bit when the observing seat knows it.
3. **The engine shows the partner as public in four cases only:** the called king is played, the called king lies in the talon, a seat other than the declarer announces kralj ultimo, a seat other than the declarer says re or mort on the game. It does not work out sides from other kontras. The network may.
4. **A deal's reward can be large.** Mort on a won solo ena with full difference is 16 × 95 = 1,520 points, a reward of about 30 with the scale of 50 (D117). The scale is not changed here; the training spec for flow B looks at it.

## Checks

Criteria 1 to 33: named tests in `src/engine/tests/` (`announcements.rs`, `kontra.rs`, `scoring_b.rs`, `written_deals.rs`). 34 to 40: `tests/encode.rs`, `tests/record.rs` and the binding tests. 41 to 44: `tests/whole_games.rs`. 50 to 52: `tests/vecenv.rs` and the binding's tests. 45 and 46: pytest output, `bench_game`, a short run of `smoke-a-ci.toml`. 47 to 49: command output and the table of wrong rules in the Results.

## Files expected to change

Engine: `src/engine/src/game.rs`, `action.rs`, `score.rs`, `encode.rs`, `record.rs`, `vecenv.rs`, `lib.rs`, `Cargo.toml` (0.3.0), a new `announce.rs`; tests under `src/engine/tests/`; `src/engine/README.md`.
Training: `src/train/tarok_env` (rebuild; `lib.rs` only if an export changes), tests under `src/train/tests/` where they count phases or slots.
Docs: `docs/encoding.md` (section 4a), this spec (Results). `docs/progress.md` and `docs/architecture.md` by the main session.

## Commits

(a) the round, the actions and their legality, tests 1–17; (b) scoring, tests 18–33; (c) observation, record and D135, tests 34–40; (d) property tests, training side, 41–46. (e) the hold flag, tests 50–52. Each with `Spec: docs/specs/009-deal-flow-b.md`. The session commits on the branch `spec-009`; the main session merges it into `main` after its verification and the review pass (D144).

## Open points for the user (Batch 13)

Places where the source is silent or where your table decides.

- **A. The same bonus by both sides.** (a) each bonus can be announced once in a deal; the other side answers with kontra. Recommended: the page has one kontra ladder per bonus, and so has the encoding. (b) each side may announce it; the encoding then needs a second ladder per bonus and becomes version 2.
- **B. Kontra on a bonus when the announcer's side is hidden.** (a) only a seat that knows the announcer's side may say it, as on the page. Recommended. Effect: before the partner is known, a defender can kontra the declarer's announcements but not another seat's. (b) every announcement shows the announcer's side to all; then anyone on the other side may kontra.
- **C. The duty to keep an announced ultimo card.** (a) the engine enforces it: the card is not a legal play before the last trick while another card is legal. Recommended: it is the page's rule and an agent cannot break it by mistake. (b) free play; an early pagat just loses the bonus.
- **D. Announced pagat ultimo when the pagat's side takes the last trick with another card.** (a) won, as you ruled for the silent one (D126). Recommended, for one rule in both cases. (b) the announced one needs the pagat itself to take the trick, as on the page.
- **E. An announced valat that fails.** (a) −500 for the announcing side and nothing else is scored. Recommended: the page says a valat removes all other scores. (b) −500 and the game is scored as usual next to it.
- **F. Order of the round.** (a) the declarer first, then in order of play; a seat may say several things in its turn; the round ends after three passes in a row. Recommended: the page's rule, with a fixed order added. (b) another order from your table.
- **G. Kralj ultimo by a declarer who holds the called king** (called their own, or took it from the talon). (a) the declarer may announce it. Recommended: spec 005 answer C already scores the silent one in this case. (b) only a partner may.

## Answers to the open points (D141)

- A. (a) Each bonus can be announced once in a deal. The encoding stays version 1.
- B. (a) Only a seat that knows the announcer's side may kontra a bonus. Criteria 15 and 16 stand.
- C. **(b) Free play.** The engine does not enforce a duty to keep an announced ultimo card. Criterion 17 is rewritten. Claude had recommended (a).
- D. (a) Announced pagat ultimo is won when the pagat's side takes the last trick, also with another card. Criterion 22 stands.
- E. (a) An announced valat that fails costs 500 and nothing else is scored. Criterion 23 stands.
- F. (a) The declarer first, then in order of play. Criteria 1 to 3 stand.
- G. (a) A declarer who holds the called king may announce kralj ultimo. Criterion 7 stands.

The four readings chosen by Claude were not objected to and stand.

## Results

(empty until executed)
