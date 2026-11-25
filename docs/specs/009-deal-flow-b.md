# 009 — Deal flow B: announcements and kontra

Status: **approved** (2026-11-24, D143), with the answers to Batch 13 (D141) and the wait of D142 written in. Executed on 2026-11-24 in the `engine-core` session on the branch `spec-009` as far as Rust goes: stages (a), (b), (c), (e) and the property tests of (d) are committed there, and so are the answers of D147, D148, D150 and D151 and the points of the review pass. Criteria 45, 46 and 53 and the Python side of 52 wait for the binding module; see "Results". Not merged. Needs specs 005, 006 and 013 (done).

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
10. An announcement by a seat other than the declarer does not show that seat's side. One exception: kralj ultimo by another seat shows that seat as the partner, from that action on. Kralj ultimo by the declarer shows that the declarer plays alone, from that action on: only the holder of the called king may announce it (D147).

**Kontra**
11. Six targets: the game and each of the five bonuses. Four levels on each: kontra, re, sub, mort (S18 a). One action raises one target by one level.
12. The game belongs to the declarer's side. A bonus belongs to the side of the seat that announced it. Kontra and sub are said by the other side, re and mort by the owning side.
13. A bonus that was not announced cannot be kontra'd. Mort cannot be raised.
14. The game: an opponent of the declarer may say kontra and sub; the declarer or the partner may say re and mort. A partner who says re or mort on the game is public from that action on. So is a seat other than the declarer who says re or mort on a bonus the declarer announced: those levels belong to the declarer's side (D148, case ii).
15. A bonus: a seat may raise it only when it knows the announcer's side (open point B). It knows it when the announcer is the declarer, when the announcer is the seat itself, when the partner is public, when the announcer is shown as an opponent (D150), or when the seat itself holds the called king.
    - **Shown as an opponent (D150, D151).** A seat that says kontra or sub on the game, or on a bonus the declarer announced, is an opponent, and every seat knows it from that action on: those levels belong to the other side than the declarer's (criterion 12). The same holds for a seat that says re or mort on a bonus of another seat that is shown as an opponent: those levels belong to the opponents there (D151). When all three other seats are shown as opponents, the declarer plays alone and all know it.
    - **A raise can show who holds the called king (D148).** While the partner is hidden, a seat that raises a bonus of another seat, where that other seat is not the declarer and is not shown as an opponent, must hold the called king. If the raising seat is not the declarer, it is the partner and is public from that action on (case i). If it is the declarer, the declarer plays alone and all know it from that action on (case iii).
    - **A bonus of a seat shown as an opponent (D150).** Every seat knows that it is the opponents'. A seat other than the declarer that says kontra or sub on it acts for the declarer's side: it is the partner and is public from that action on (case i of D148 holds here too, for another reason). The declarer's kontra or sub on it shows nothing: the declarer knows the side from what was said, not from its hand. Re and mort on it by another seat than its announcer show that seat as an opponent (D151, above). The announcer's own re or mort shows nothing new.
    - **The rule behind all of these (D151).** An action of the round shows that the seat that takes it holds the called king when a seat that does not hold it could not take it, in the same public state. It shows that the seat is an opponent when a seat that holds the called king could not take it. It shows nothing when both could. A seat knows the announcer's side by the list at the head of this criterion, and by nothing else: what only one seat can work out from its own cards is not used (D151, item 2).
16. A seat's legal actions depend only on what that seat knows. Test: two deals with the same public history and the same hand of seat Y, in which another seat is once the partner and once an opponent, give Y the same legal set at every turn of the round. What every seat knows about the sides (the public partner, and since D150 the seats shown as opponents) depends only on what was said and on the cards all have seen. Test (D150): it is worked out a second time from the public events alone, on random deals, and is the same as the engine's at every decision of the round; and it is true of the deal. Test (D151), the pair of deals: for a public history, the called king is put into each other seat's hand in turn and the same actions are taken again. A seat is a possible holder if they are all legal there. The engine shows a seat as the partner (or the declarer as alone) exactly when it is the only possible holder, and shows a seat as an opponent exactly when it is not a possible holder: on random deals, at every decision of the round.

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
36. State block D: D1 holds each announcement on the side that owns it, as far as the observing seat knows the side by the rule of criterion 15; an announcement of unknown side sets both sides' bits (reading 2). An announcement of a seat shown as an opponent is on the opponents' side for every observer (D150). D2 holds the level of each of the six targets.
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

**Checkpoints (D148, B17-2 a; in stage (d))**
53. The trainer writes the engine version into every checkpoint. Resuming a checkpoint of another engine version, or of none (made before this guard), is refused with an error that names both versions. The evaluation takes such a checkpoint as agent or as opponent only with a flag, and prints a warning. `encoding_version` stays 1. Tests in the Python tests of this spec, written before the code.

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
3. **The engine shows the partner as public in nine cases only.** The first execution had four: the called king is played; the called king lies in the talon; a seat other than the declarer announces kralj ultimo; a seat other than the declarer says re or mort on the game. The user added four on 2026-11-24, each a fact that follows from what was said and from who may say what. D147: the declarer announces kralj ultimo (it holds the called king and plays alone). D148: (i) a seat other than the declarer raises a bonus of another seat that is not the declarer while the partner is hidden (it is the partner); (ii) a seat other than the declarer says re or mort on a bonus of the declarer (it is the partner); (iii) the declarer raises a bonus of a seat that is not the declarer while the partner is hidden (it plays alone). **D150 (2026-11-24)** adds a fact about single seats and a ninth case. The engine keeps, for each seat, "shown as an opponent": the seat said kontra or sub on the game or on a bonus the declarer announced. A seat may raise a bonus of such a seat, and block D1 shows that seat's announcements on the opponents' side. Ninth case: all three other seats are shown as opponents (the declarer plays alone). With D150, cases (i) and (iii) hold where the announcer is not shown as an opponent; criterion 15 has the wording. **D151 (2026-11-25), the standing rule:** the engine shows every fact about the sides that every seat can work out strictly from what was said and from who may say what, and no other fact. A case is checked with two deals that look the same from outside and differ in the true sides: if such a pair exists, the case does not follow and is not built. The nine cases and "shown as an opponent" are what the rule gives; D151 adds one way to be shown as an opponent (re or mort on a bonus of a seat shown as an opponent). The full table is in the Results. What only one seat can work out from its own cards is not used, neither in the legal set nor in the observation (D151, item 2): for example a declarer without the called king who sees two seats shown as opponents and so knows the third is its partner. The network may draw such conclusions.
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

## Answers after the first execution (D147)

The Results asked three questions. The user answered on 2026-11-24.

- **Rewards as 32-bit numbers.** The room for "the rewards of a deal add up to zero" grows with the largest reward of the deal, as built.
- **A declarer who announces kralj ultimo with its own called king** is shown as alone to every seat. Criterion 10 and reading 3 are changed above; the test `c10` and the code follow in one commit.
- **Block F and the announced ultimo cards.** Block F will use what an announced pagat ultimo or kralj ultimo shows about the holder. Not in this spec: it is built with the training spec for flow B.

## Answers to Batch 17 (D148)

- **Three more cases in which the sides follow from what was said** (B17-1 a), on the condition that each is checked against the rules of who may say what before it is built. The check is in the Results ("The three cases of D148"). Criteria 14 and 15 and reading 3 are changed above; tests and code follow in one commit with the case of D147.
- **A guard by engine version for checkpoints** (B17-2 a): criterion 53, in stage (d).

## Answer to Batch 19, item 1 (D150)

- **A seat shown as an opponent by its own kontra** (question 4 of the Results, options b and c). Built on the branch before the merge. Criteria 15, 16 and 36 and reading 3 are changed above; the tests and the code follow in one commit. The encoding gets no new feature. Each consequence is checked against the rules of who may say what; the check is in the Results ("A seat shown as an opponent").

## Answer to Batch 20 (D151)

- **A standing rule for the public sides** (item 1 a). The engine shows every fact about the sides that every seat can work out strictly from what was said and from who may say what. Each case is checked with a pair of deals, built with its tests first and reported in the Results; cases are no longer asked one by one. Built under the rule: re or mort on a bonus of a seat shown as an opponent, by another seat, shows that seat as an opponent. Reading 10 of the Results (case (i) of D148 kept on such a bonus) stands, with its pair-of-deals check written down. The actions of the round are gone through once more; the table is in the Results.
- **Private conclusions are not used** (item 2 a). Reading 3 and criterion 15 say it.

## Results

Executed on 2026-11-24 by the session `engine-core` (Claude Code 2.1.306, `claude-fable-5-2`) on the branch `spec-009`, in the worktree `.worktrees/spec-009`. Nothing is on `main`.

**State: stages (a), (b), (c) and (e) are built and committed; of stage (d) the property tests are. The answers of D147, D148, D150 and D151 and the points of the two review passes are built and committed. Open: criteria 45, 46 and 53 and the Python side of 52. They need the binding module built again, which waits for the main session's word that the runs of spec 017 have ended (D146).**

| Commit | Stage | Holds |
|---|---|---|
| `af56384` | (a) | the round, the three actions, who may say what; criteria 1 to 17 |
| `f72721a` | (b) | the score; criteria 18 to 33 |
| `717268f` | (c) | tokens, state block D, the record, D135, `docs/encoding.md` section 4b; criteria 34 to 40 |
| `d35aa5e` | (e) | the hold flag in the engine and in the binding's Rust; criteria 50 and 51 |
| `42a2001` | (d), part 1 | the property tests; criteria 41 to 44 |
| `3ce38c8` | (d), part 1 | `bench_game` counts decisions and tokens (for criterion 46) |
| `5f198bc` | | four comments in `tests/vecenv.rs` |
| `dd012d9` | | this section, first version |
| `938c154` | | D147 and D148: the sides that follow from what was said (criteria 10, 14, 15); criterion 53 added to the spec |
| `c651848` | | the review pass: tests only, no engine code |
| `6bce5d8` | | this section, second version, with the wrong rules run again |
| `5c77360` | | D150: a seat shown as an opponent by its own kontra (criteria 15, 16, 36) |
| `d2e4c1b` | | this section, third version |
| `208857a` | | D151: the public sides by the standing rule (criteria 15, 16) |
| `1e8292f` | | the second review: tests only (the four-world search, the next deal of a table, the private conclusion) |
| the commit after it | | this section, fourth version, with every wrong rule run again on the final code |

Tests were written from the criteria before the code of each stage. Where the old code could compile them they were run against it first: 14 of the 15 tests of `scoring_b.rs` failed before the scoring change (the fifteenth, `c24`, tests a silent valat, which ignores what is said before and after).

### What was built

- `src/announce.rs` (new): `Bonus`, `Target`, and `Round`, which holds what was said and whose turn it is.
- `src/action.rs`: `Action::Announce` (slots 77 to 81), `Action::Kontra` (82 to 87), `Action::Done` (88). Slot 89 has no action.
- `src/game.rs`: `Phase::Announcements` between the last discard and the first lead. `round_actions` gives the legal set from what the speaker knows. `side_known_to` is the one place that says whether a seat knows another seat's side (criterion 15); the legal set and block D1 both use it. `PublicView::announced`, `PublicView::kontra_level`, `PrivateView::knows_side_of`. `MAX_ACTIONS` is 192 (was 96).
- `src/score.rs`: announced bonuses, the announced valat, the kontra multipliers. The eight parts keep their order.
- `src/encode/`: tokens 146 to 156, phase entry 5, block D1 and D2.
- `src/record.rs`: `RecordError::OtherEngine` and the replay of a record of another version (D135).
- `src/vecenv.rs`: `Tables::step_hold_into`, `VecEnv::step_hold`. `src/train/tarok_env/src/lib.rs`: `step_into` takes an optional last argument `hold`.
- The crate version is 0.3.0.

### Criteria

| # | Criterion | Status | Test |
|---|---|---|---|
| 1 | The round follows the last discard; the declarer first, then in order of play | met | `c01_after_the_last_discard_the_declarer_opens_the_round` |
| 2 | A turn: actions, then "done"; "done" always legal | met | `c02_a_turn_is_any_number_of_actions_and_then_done` |
| 3 | The end of the round | met | `c03_the_round_ends_after_three_passes_once_every_seat_had_a_turn` |
| 4 | Forehand leads, also after an announced valat | met | `c04_forehand_leads_the_first_trick_also_after_an_announced_valat` |
| 5 | All pass: the deal of flow A | met | `c05_a_deal_in_which_all_pass_plays_and_scores_as_in_flow_a`; the tests of specs 005 and 013 pass with the four passes in the helpers (list below) |
| 6 | Trula, kralji, valat by every seat | met | `c06_trula_kralji_and_valat_may_be_announced_by_every_seat` |
| 7 | Kralj ultimo by the holder of the called king | met | `c07_kralj_ultimo_only_by_the_holder_of_the_called_king` |
| 8 | Pagat ultimo by the holder of the pagat | met | `c08_pagat_ultimo_only_by_the_holder_of_the_pagat` |
| 9 | Each bonus once | met | `c09_each_bonus_can_be_announced_once_in_a_deal` |
| 10 | An announcement does not show the side; kralj ultimo does (D147: also the declarer's) | met | `c10_an_announcement_does_not_show_the_side_except_kralj_ultimo` |
| 11 | Six targets, four levels | met | `c11_six_targets_four_levels_one_action_raises_one_level` |
| 12 | Who says which level | met | `c12_kontra_and_sub_by_the_other_side_re_and_mort_by_the_owner` |
| 13 | Not announced: no kontra; mort is the top | met | `c13_a_bonus_not_announced_cannot_be_kontrad_and_mort_cannot_be_raised` |
| 14 | The game's ladder; a partner shown by re or mort (D148: also on a bonus of the declarer) | met | `c14_the_game_ladder_and_a_partner_shown_by_re_or_mort`; what shows nothing: `c14_what_is_said_and_shows_no_partner` |
| 15 | A bonus is raised only by a seat that knows the side (D148: such a raise can show the sides) | met | `c15_a_bonus_is_raised_only_by_a_seat_that_knows_the_announcers_side`; a seat shown as an opponent (D150): `c15_a_seat_shown_as_an_opponent_by_its_own_kontra`; the pairs of deals for a bonus of such a seat, and that a private conclusion is not used (D151): `c15_the_pairs_of_deals_for_a_bonus_of_a_seat_shown_as_an_opponent` |
| 16 | The legal set depends only on what the seat knows | met | `c16_legal_actions_depend_only_on_what_the_seat_knows`: the two deals of the criterion, and 400 random deals against a second legal set written from the public and the private view. D150: `c16_what_all_know_about_the_sides_follows_from_what_was_said`, 2,000 random deals against a second reading of the public events and against the truth of the deal. D151, the standing rule: `c16_the_engine_shows_what_no_pair_of_deals_can_tell_apart`, 5,000 random deals, every other possible holder of the called king tried. The second review: the four-world search, three `c16_four_worlds_…` tests in `tests/worlds.rs`, 1.79 million nodes |
| 17 | No duty to keep an announced card | met | `c17_an_announced_ultimo_card_may_be_played_when_the_rules_of_play_allow` |
| 18 | Announced trula | met | `c18_announced_trula_scores_20_for_or_against_its_owner` |
| 19 | Announced kralji | met | `c19_announced_kralji_scores_20_for_or_against_its_owner` |
| 20 | Announced replaces silent | met | `c20_an_announced_bonus_replaces_the_silent_one_of_its_kind` |
| 21 | Announced kralj ultimo | met | `c21_announced_kralj_ultimo` |
| 22 | Announced pagat ultimo follows the side | met | `c22_announced_pagat_ultimo_follows_the_side_that_takes_the_last_trick` |
| 23 | Announced valat | met | `c23_announced_valat_scores_500_and_nothing_else` |
| 24 | Silent valat next to announced bonuses | met | `c24_a_silent_valat_next_to_announced_bonuses_scores_250_and_nothing_else` |
| 25 | Not announced: as in flow A | met | `c25_a_bonus_nobody_announced_is_scored_as_in_flow_a` |
| 26 | Kontra on the game | met | `c26_kontra_on_the_game_multiplies_game_and_difference_only` |
| 27 | Kontra on a bonus | met | `c27_kontra_on_a_bonus_multiplies_that_announced_bonus_only` |
| 28 | Silent bonuses and the mondfang are never multiplied | met | `c28_silent_bonuses_and_the_mondfang_are_never_multiplied` |
| 29 | Kontra and a valat | met | `c29_with_a_valat_only_the_kontra_on_the_announced_valat_counts` |
| 30 | The kontra examples | met | `c30_kontra_examples_on_whole_deals` |
| 31 | The three examples of the source | met | `c31_the_three_examples_of_the_source` |
| 32 | The parts add up to the entry | met | `c32_the_ten_parts_of_a_seat_add_up_to_its_entry` |
| 33 | Three written deals | met | `c33_three_written_deals_with_announcements_and_kontra` |
| 34 | The slots 77 to 88 | met | `c34_the_round_uses_the_slots_77_to_88` |
| 35 | One token per announcement and kontra, none for "done" | met | `c35_every_announcement_and_kontra_writes_one_token_with_the_acting_seat` |
| 36 | State block D | met | `c36_state_block_d_holds_the_announcements_by_side_and_the_kontra_levels`, with a seat shown as an opponent (D150) |
| 37 | At most 128 tokens | met | `c37_no_legal_deal_has_more_than_128_tokens` |
| 38 | Hidden sides stay hidden | met | `c38_hidden_sides_stay_hidden_in_the_observation` (the built pair) and `c38_no_leak_with_announcements_and_kontra` (120,000 random cases, half of them with the called king in another hand) |
| 39 | The record with the round | met | `c39_a_record_with_announcements_and_kontras_replays_to_the_same_entries` |
| 40 | D135 | met | `c40_a_record_of_another_engine_version_keeps_going_and_names_both_versions` |
| 41 | Every deal ends; the round is bounded | met | `c41_every_deal_of_flow_b_reaches_over_and_the_round_is_bounded`; the bound of 91 turns is reached in `c41_the_longest_round_has_91_turns` |
| 42 | The second scorer agrees | met | `c42_the_second_scorer_agrees_on_random_deals_of_flow_b` |
| 43 | Declarer and partner equal; an opponent 0 or −20 | met | `c43_declarer_and_partner_write_the_same_and_an_opponent_0_or_minus_20` |
| 44 | Coverage with fixed seeds | met | `c44_fixed_seeds_cover_the_rules_of_flow_b` |
| 45 | The module builds, pytest, the smoke config | **not done** | waits for the module build |
| 46 | Reported numbers | part | the counts are measured (below); the deals per second were measured next to other jobs and are to be measured again |
| 47 | fmt, clippy, no `unsafe`, workflows green | part | local: met for the engine and the binding crate. `engine` on the branch: green on `6bce5d8` (run 45403601333), on `5f198bc` (run 45368097231) and on `42a2001` before it. The run on the D150 commits is reported by the session after the push. `train` runs on `main` and on pull requests only |
| 48 | Every earlier test passes; changes listed | met | list below |
| 49 | At least 30 wrong rules | met | 75 planted, 75 caught, all of them run on the final code; table below |
| 50 | A held table does not move | met | `c50_a_held_table_does_not_move` |
| 51 | Holding never changes what a table plays | met | `c51_holding_never_changes_what_a_table_plays` |
| 52 | The binding passes the flag through | part | the binding's Rust takes `hold` and compiles (`cargo clippy`, `cargo test` of the crate). `EngineEnv.step(actions, hold)` in `env.py` and its test are not written: they cannot be run before the module is built |
| 53 | A guard by engine version for checkpoints (D148) | **not done** | stage (d): Python tests first, then the trainer and the evaluation; waits for the module build |

The tests are in `src/engine/tests/`: `announcements.rs` (c01 to c10, c17), `kontra.rs` (c11 to c16), `scoring_b.rs` (c18 to c32), `written_deals.rs` (c33), `encode.rs` (c34 to c38), `no_leak.rs` (c38), `record.rs` (c39, c40), `whole_games.rs` (c41 to c44), `vecenv.rs` (c50, c51). 53 named tests: the 47 of the first execution, `c14_what_is_said_and_shows_no_partner` and `c41_the_longest_round_has_91_turns` from the review pass, and `c15_a_seat_shown_as_an_opponent_by_its_own_kontra` and `c16_what_all_know_about_the_sides_follows_from_what_was_said` from D150, and `c15_the_pairs_of_deals_for_a_bonus_of_a_seat_shown_as_an_opponent` and `c16_the_engine_shows_what_no_pair_of_deals_can_tell_apart` from D151. `tests/common/sides.rs` is the second reading of what all know. From the second review: the three `c16_four_worlds_…` tests of `tests/worlds.rs` and `c17_every_decision_of_a_table_is_that_of_a_fresh_game_of_its_deal` in `tests/vecenv.rs` (numbered by spec 006, criterion 17). The engine has 241 tests (182 before this spec).

### Earlier tests that had to change (criterion 48)

Every test of specs 002, 005, 006 and 013 passes. These had to change, each for the reason given. No expected entry, count of card points or trick winner of an earlier test changed.

**Because the round now lies between the last discard and the first card**

- `tests/common/flow.rs`: `exchange`, `exchange_back`, `run_until`, `finish`, `play_scripted` and `play_random` say "done" for every seat in the round (criterion 5). A pass uses none of the random choices, so every seeded deal of flow A has the cards and the plays it had.
- `tests/talon.rs` `c19_…`, `c20_…` and `tests/views.rs` `hidden_discards_and_the_unshown_talon_do_not_show_in_the_public_view`: the phase after the last discard is "announcements"; after four passes it is "play".
- `tests/game_play.rs` `c28_called_king_played_makes_the_partner_public`: its loop took the lowest legal action. In the round that is an announcement, and kralj ultimo shows the partner. It passes in the round now.
- `tests/state_features.rs` `block_b_through_the_phases`: the phase entry 5 exists and is seen; the loop passes in the round.
- `tests/whole_games.rs`: the order of the phases has the new one; `c46` allows four more actions; `c50` has the slot range 77 to 88 for the round.
- `tests/tokens.rs`: `tokens_name_the_actions_that_were_taken` expects no token for "done"; `longest_possible_sequence_of_this_slice` counts the four passes among the actions.
- `tests/golden/record_seed_42.json`: the version is 0.3.0 and the record has four more decisions, the passes, with their legal slots. `examples/record_deal.rs` and the example in `src/lib.rs` pass in the round.

**Because slots, limits and features are in use that were reserved**

- `tests/actions.rs` `every_slot_is_one_action_or_reserved`: twelve more slots have an action; 89 and 144 to 159 are left.
- `src/game.rs`, unit test `the_action_history_panics_when_it_is_full`: the limit in the message is 192.
- `tests/state_features.rs` `block_d_e_and_g_of_the_scripted_position`: block D2 holds "none" for each of the six targets, where it was all zero. `SAME_FOR_ALL` no longer lists D1: an announcement shows on one side or on both, depending on who looks.

**Because a random deal is longer**

- `tests/vecenv.rs`: a random deal has about 88 decisions, not about 57, so a fixed number of steps finishes fewer deals. Lower bounds: `c15` 900 to 500 deals, `c18` 400 to 300, `random_runs_do_not_depend_on_the_thread_count` 400 to 250. `c17` takes more steps for its two runs per table.

**Because an entry can be thousands of points**

- `tests/vecenv.rs` `c18_rewards_and_parts_are_delivered_once_per_deal`: "the four rewards sum to zero" and "the ten parts sum to the reward" held within 1e-6. Rewards are 32-bit numbers; with a valat under kontra a reward is above 100 and its last place is about 1e-5. The room is now 1e-6 times the largest reward of the deal, and 1e-6 for rewards up to 1. See question 1.

**Moved, not changed**

- `tests/scoring.rs`: its built deals moved to `tests/common/built.rs`, so `scoring_b.rs` uses the same deals. No test body changed.
- `tests/no_leak.rs`: the body of `c06` is a function now, which runs once on deals of flow A (`c06`, the same checks) and once on deals of flow B (`c38`). The seeds of a property test depend on the line of its call, so `c06` runs other cases than before: 16,620 compared.
- `tests/common/second_scorer.rs`: it reads what was said and scores it by criteria 18 to 29, written a second time.

**Tests of this spec that changed after the first execution**

- D147 and D148 changed what four tests expect: `c10`, `c14`, `c15` and `c36`. What changed in each is listed under "The sides that follow from what was said" below.
- The review pass: `c18`, `c19`, `c21` and `c22` have more built deals and none of their earlier cases changed. `c38` in `tests/no_leak.rs` runs 120,000 cases with a fifth kind of change and has higher lower bounds; `c06` (flow A) runs the kinds it ran. `c44` counts announced pagat ultimos won and lost apart (it had counted both as "made").
- D150 changed `c16`, `c36` and `c38`. What changed in each is listed under "A seat shown as an opponent" below.
- D151 changed `c15_a_seat_shown_as_an_opponent_by_its_own_kontra` and the second reading in `tests/common/sides.rs`; see "The public sides by the standing rule" below.
- The second review added a block by hand to `c36` (the private conclusion is not used) and changed no expectation.

### Wrong rules (criterion 49)

**75 wrong rules, each put into the final code alone. 75 caught. None was missed.** Every rule fails a test of the criterion it breaks.

How: a copy of `src/engine` outside the repo (with `docs/encoding.md` next to it and its own target folder); one rule at a time, as text replacements; the whole suite; the files restored after. Two copies ran side by side. Both clean copies pass all 237 tests before the first and after the last rule.

This is the fourth run. The first (52 rules) was on the code of the first execution, the second (16) after D147, D148 and the review, the third (19) after D150. D150 and D151 rewrote the code that says who may say what and what an action shows, so a rule planted there earlier was no longer a statement about the final code. For this run every rule of the list was put into the code after D151: 50 as they were, 21 written again for the new code, and three new ones. Rule 75 is from the second review; it and rule 74 were run after the review's tests were in, on 241 tests.

- Rules 1 to 52: the first execution. Rules 53 to 55: wrong readings of D147. 56 to 60: of D148. 61 to 64: the review's points 2 and 3 (`score.rs`). 65 to 71: of D150.
- New with D151: 72 is the rule before D151 (re or mort on a bonus of a seat shown as an opponent shows nothing). 73 switches reading 10 off. 74 uses a private conclusion, which item 2 of D151 rules out.
- New with the second review: 75, the environment keeps the seats shown as opponents for the next deal of a table. It fails `c17_every_decision_of_a_table_is_that_of_a_fresh_game_of_its_deal` and no other test: before the review nothing caught it.
- A rule about what an action shows is planted as an exception at the head of `Game::shows`, since the function itself has no cases any more.

The column names every test of this spec that failed, by its number; the last column counts the failing tests of earlier specs and the unit tests. Rules 1 to 73 ran before the tests of the second review were added, so the four-world search and the fresh-game test stand in the rows of 74 and 75 only.

| # | Criterion | Wrong rule | Tests of this spec that failed | Other tests that failed |
|---|---|---|---|---|
| 1 | 1 | Forehand opens the round, not the declarer | `c01`, `c04`, `c05`, `c33` | 11 |
| 2 | 1 | The turns go against the order of play | `c01`, `c02`, `c03`, `c04`, `c06`, `c09`, `c10`, `c11`, `c12`, `c13`, `c14`, `c14` (no partner), `c15`, `c15` (D150), `c15` (pairs), `c16`, `c17`, `c35`, `c36`, `c37`, `c38`, `c41` (91 turns) | 1 |
| 3 | 3 | Three passes end the round also before every seat had a turn | `c01`, `c03`, `c05`, `c06`, `c07`, `c08`, `c10`, `c12`, `c13`, `c14`, `c18`, `c19`, `c22`, `c23`, `c41`, `c41` (91 turns) | 2 |
| 4 | 3 | The round ends after four turns, whatever was said | `c03`, `c09`, `c11`, `c12`, `c13`, `c14`, `c14` (no partner), `c15`, `c15` (D150), `c15` (pairs), `c16`, `c16` (pairs of deals), `c16` (what all know), `c26`, `c27`, `c28`, `c29`, `c30`, `c32`, `c34`, `c35`, `c36`, `c37`, `c38`, `c39`, `c41` (91 turns), `c44` | 1 |
| 5 | 3 | A kontra does not count as speaking: the turn is a pass | `c03`, `c11`, `c12`, `c13`, `c14`, `c14` (no partner), `c15`, `c15` (D150), `c15` (pairs), `c16`, `c16` (pairs of deals), `c26`, `c27`, `c28`, `c29`, `c30`, `c32`, `c36`, `c37`, `c38`, `c41` (91 turns) | 0 |
| 6 | 4 | The declarer leads the first trick | `c04`, `c05`, `c31`, `c33` | 11 |
| 7 | 6 | Trula, kralji and valat only by the declarer's side | `c02`, `c03`, `c04`, `c06`, `c09`, `c12`, `c14` (no partner), `c15`, `c15` (D150), `c15` (pairs), `c16`, `c16` (pairs of deals), `c18`, `c19`, `c20`, `c23`, `c24`, `c25`, `c29`, `c33`, `c36`, `c38`, `c41` (91 turns), `c44` | 1 |
| 8 | 7 | Kralj ultimo by every seat when a king is called | `c07`, `c09`, `c10`, `c14` (no partner), `c15`, `c16`, `c16` (pairs of deals), `c16` (what all know), `c27`, `c36`, `c41` (91 turns), `c42`, `c44` | 1 |
| 9 | 7 | Kralj ultimo by the partner only, not by a declarer who holds the king | `c07`, `c10`, `c15`, `c16`, `c16` (pairs of deals), `c20`, `c21`, `c36`, `c44` | 0 |
| 10 | 8 | Pagat ultimo by every seat | `c08`, `c09`, `c16` | 1 |
| 11 | 9 | A bonus can be announced again by another seat | `c09`, `c16`, `c16` (pairs of deals), `c16` (what all know), `c34`, `c35`, `c36`, `c37`, `c38`, `c39`, `c41`, `c41` (91 turns), `c42`, `c43`, `c44`, `c50`, `c51` | 10 |
| 12 | 10 | Kralj ultimo by another seat does not show the partner | `c10`, `c14` (no partner), `c15`, `c16`, `c16` (pairs of deals), `c16` (what all know), `c27`, `c36`, `c41` (91 turns) | 0 |
| 13 | 10 | Every announcement by the partner shows the partner | `c10`, `c14` (no partner), `c15` (pairs), `c16`, `c16` (pairs of deals), `c16` (what all know), `c36`, `c38` | 1 |
| 14 | 12 | Every level may be said by either side | `c12`, `c14`, `c14` (no partner), `c15`, `c15` (D150), `c15` (pairs), `c16`, `c16` (pairs of deals), `c16` (what all know), `c36`, `c38` | 1 |
| 15 | 12 | Kontra and sub by the owning side, re and mort by the other | `c02`, `c03`, `c11`, `c12`, `c13`, `c14`, `c14` (no partner), `c15`, `c15` (D150), `c15` (pairs), `c16`, `c16` (pairs of deals), `c16` (what all know), `c21`, `c24`, `c26`, `c27`, `c28`, `c29`, `c30`, `c32`, `c33`, `c35`, `c36`, `c37`, `c38`, `c41` (91 turns) | 1 |
| 16 | 13 | Mort can be raised again | `c13`, `c14`, `c16`, `c16` (pairs of deals), `c16` (what all know), `c34`, `c35`, `c36`, `c37`, `c38`, `c39`, `c41`, `c41` (91 turns), `c42`, `c43`, `c44`, `c50`, `c51` | 10 |
| 17 | 13 | A bonus nobody announced can be kontra'd, as the declarer's | `c13`, `c16`, `c16` (what all know), `c36` | 1 |
| 18 | 14 | Only mort shows the partner, re does not | `c14`, `c14` (no partner), `c16`, `c16` (pairs of deals), `c16` (what all know), `c36` | 0 |
| 19 | 14 | Every kontra on the game or on a bonus of the declarer by another seat shows a partner | `c14`, `c14` (no partner), `c15`, `c15` (D150), `c15` (pairs), `c16`, `c16` (pairs of deals), `c16` (what all know), `c36`, `c38` | 2 |
| 20 | 15 | A seat may raise a bonus without knowing the announcer's side | `c14` (no partner), `c15`, `c15` (pairs), `c16`, `c16` (pairs of deals), `c16` (what all know), `c36`, `c38` | 1 |
| 21 | 15 | A seat that holds the called king does not know the sides | `c14` (no partner), `c15`, `c16`, `c16` (pairs of deals), `c36` | 0 |
| 22 | 15 | The announcer may not raise its own bonus while the partner is hidden | `c15` (pairs), `c16`, `c36` | 0 |
| 23 | 17 | The announced pagat may not be played before the last trick while another card is legal | `c17`, `c22`, `c23`, `c31` | 0 |
| 24 | 18 | An announced trula is worth 10 | `c18`, `c20`, `c25`, `c27`, `c28`, `c30`, `c31`, `c32`, `c33`, `c42` | 0 |
| 25 | 19 | An announced kralji is checked against the declarer's pile, whoever owns it | `c18`, `c19`, `c20`, `c25`, `c33`, `c42` | 0 |
| 26 | 20 | A failed announced trula costs the silent bonus on top | `c18`, `c20`, `c28`, `c42` | 0 |
| 27 | 18 | A failed announced bonus costs nothing | `c18`, `c19`, `c20`, `c21`, `c22`, `c23`, `c25`, `c27`, `c28`, `c29`, `c30`, `c31`, `c33`, `c41` (91 turns), `c42`, `c44` | 0 |
| 28 | 18 | An opponent's announced bonus is written with the owner's sign | `c18`, `c19`, `c20`, `c22`, `c23`, `c25`, `c27`, `c29`, `c32`, `c33`, `c41` (91 turns), `c42` | 0 |
| 29 | 21 | An announced kralj ultimo played before the last trick scores nothing | `c21`, `c42` | 0 |
| 30 | 22 | The announced pagat ultimo needs the pagat itself to take the trick | `c22`, `c42` | 0 |
| 31 | 22 | An announced pagat played before the last trick scores nothing | `c22`, `c31`, `c42`, `c44` | 0 |
| 32 | 23 | An announced valat that fails costs 250 | `c23`, `c29`, `c33`, `c41` (91 turns), `c42` | 0 |
| 33 | 23 | An opponent's announced valat is made when the declarer takes all twelve | `c23`, `c29`, `c42`, `c44` | 0 |
| 34 | 23 | An announced valat that fails still scores the game | `c23`, `c29`, `c33`, `c41` (91 turns), `c42` | 0 |
| 35 | 24 | A silent valat scores an announced trula next to it | `c24`, `c29`, `c42` | 0 |
| 36 | 26 | A kontra on the game does not multiply the difference | `c26`, `c28`, `c30`, `c32`, `c33`, `c42` | 0 |
| 37 | 26 | A kontra on the game multiplies the kralji too | `c26`, `c28`, `c33`, `c42` | 0 |
| 38 | 26 | A level adds one instead of doubling: 2, 3, 4, 5 | `c26`, `c27`, `c28`, `c29`, `c30`, `c32`, `c41` (91 turns), `c42` | 1 |
| 39 | 27 | A kontra on a bonus is not counted | `c21`, `c27`, `c28`, `c29`, `c30`, `c32`, `c33`, `c41` (91 turns), `c42` | 0 |
| 40 | 28 | The mondfang is multiplied by the kontra on the game | `c28`, `c33`, `c42`, `c43` | 0 |
| 41 | 29 | A silent valat is multiplied by the kontra on the game | `c29`, `c42` | 0 |
| 42 | 34 | "Done" lies on slot 89 | `c02`, `c34`, `c39`, `c40`, `c50` | 3 |
| 43 | 35 | A kontra writes no token | `c16`, `c16` (what all know), `c35`, `c36`, `c37` | 0 |
| 44 | 35 | The token of an announcement carries no seat | `c16`, `c16` (what all know), `c35`, `c36` | 0 |
| 45 | 35 | The kontra tokens start at id 150 | `c35` | 1 |
| 46 | 36 | D1 shows the true side of every announcement to every seat | `c36`, `c38` | 0 |
| 47 | 36 | D1 sets no bit for an announcement of unknown side on the declarer's half | `c36`, `c38` | 0 |
| 48 | 36 | D2 stays at "none" | `c36` | 0 |
| 49 | 40 | The legal slots of a record of another version must fit too | `c40` | 0 |
| 50 | 40 | The error of a record of another version does not name the versions | `c40` | 0 |
| 51 | 50 | The action of a held table must be legal | `c50`, `c51` | 0 |
| 52 | 50 | A held table takes its action | `c50` | 0 |
| 53 | 10 | A declarer's kralj ultimo shows nothing (the rule before D147) | `c10`, `c15`, `c16`, `c16` (pairs of deals), `c16` (what all know), `c36` | 0 |
| 54 | 10 | Every announcement of the declarer shows the sides | `c10`, `c14`, `c14` (no partner), `c15` (D150), `c16`, `c16` (pairs of deals), `c16` (what all know), `c36`, `c38` | 2 |
| 55 | 15 | A declarer who is publicly alone does not make the sides known | `c12`, `c15`, `c15` (D150), `c15` (pairs), `c16`, `c16` (pairs of deals), `c16` (what all know), `c27`, `c29`, `c36` | 0 |
| 56 | 14 | Re or mort on a bonus of the declarer does not show the partner (D148 ii off) | `c14`, `c14` (no partner), `c16` (pairs of deals), `c16` (what all know), `c36` | 0 |
| 57 | 15 | A raise of a bonus of a seat of unknown side by another seat than the declarer shows nothing (D148 i off) | `c14` (no partner), `c15`, `c16` (pairs of deals), `c16` (what all know), `c36` | 0 |
| 58 | 15 | The declarer's raise of a bonus of a seat of unknown side shows nothing (D148 iii off) | `c15`, `c16` (pairs of deals), `c16` (what all know), `c36` | 0 |
| 59 | 14 | The declarer's own re or mort shows the sides | `c14`, `c14` (no partner), `c15` (D150), `c16`, `c16` (pairs of deals), `c16` (what all know), `c36`, `c38` | 2 |
| 60 | 14 | A raise of a bonus of the declarer is taken as a raise of a bonus of unknown side | `c14`, `c14` (no partner), `c15` (D150), `c16`, `c16` (pairs of deals), `c16` (what all know), `c36`, `c38` | 2 |
| 61 | 21 | An announced kralj ultimo is won when the declarer's side takes the last trick (the king may be played earlier) | `c21`, `c42` | 0 |
| 62 | 22 | An announced pagat ultimo is won when the owner's side takes the last trick (the pagat may be played earlier) | `c22`, `c31`, `c42` | 0 |
| 63 | 18 | An opponent's announced trula or kralji is counted without the packet the declarer did not take | `c18`, `c19`, `c42` | 0 |
| 64 | 18 | The declarer's announced trula or kralji counts the packet it did not take | `c18`, `c19`, `c42` | 0 |
| 65 | 15 | Kontra or sub on the game or on a bonus of the declarer shows nothing (D150 off) | `c15` (D150), `c15` (pairs), `c16`, `c16` (pairs of deals), `c16` (what all know), `c36`, `c38` | 0 |
| 66 | 15 | A seat shown as an opponent is not known to the other seats | `c15` (D150), `c15` (pairs), `c16`, `c16` (pairs of deals), `c16` (what all know), `c36`, `c38` | 0 |
| 67 | 15 | Two seats shown as opponents make the sides public | `c15` (D150), `c15` (pairs), `c16`, `c16` (pairs of deals), `c16` (what all know), `c36` | 0 |
| 68 | 15 | All three other seats shown as opponents: the partner stays not known | `c15` (D150), `c15` (pairs), `c16` (pairs of deals), `c16` (what all know), `c36` | 0 |
| 69 | 15 | The declarer's kontra on a bonus of a seat shown as an opponent shows that it holds the called king | `c15` (D150), `c15` (pairs), `c16` (pairs of deals), `c16` (what all know), `c38` | 1 |
| 70 | 15 | Re or mort on a bonus of a seat shown as an opponent shows the seat as the partner | `c15` (D150), `c15` (pairs), `c16` (pairs of deals), `c16` (what all know), `c38` | 1 |
| 71 | 14 | Every raise of the game or of a bonus of the declarer by another seat shows an opponent, re and mort too | `c14`, `c14` (no partner), `c16`, `c16` (pairs of deals), `c16` (what all know), `c36` | 0 |
| 72 | 15 | Re or mort on a bonus of a seat shown as an opponent shows nothing (the rule before D151) | `c15` (D150), `c15` (pairs), `c16` (pairs of deals), `c16` (what all know), `c36` | 0 |
| 73 | 15 | Kontra or sub on a bonus of a seat shown as an opponent, by another seat than the declarer, shows nothing | `c15` (D150), `c15` (pairs), `c16`, `c16` (pairs of deals), `c16` (what all know), `c36` | 0 |
| 74 | 15 | A private conclusion is used: a declarer without the called king who sees two seats shown as opponents knows the third is its partner | `c15` (pairs), `c16` (four worlds), `c36` | 0 |
| 75 | 16 | The environment keeps the seats shown as opponents for the next deal of the table | none of this spec's numbered tests; `c17` (fresh game) in `tests/vecenv.rs` | 1 |

Rules caught by one test only: 48 (`c36`), 49 (`c40`), 50 (`c40`), 52 (`c50`), and 75 (the fresh-game test of `tests/vecenv.rs`). Each of the first four tests has the case written out by hand.

The scripts and their outputs are in the session's scratch folder: this run `mutate009_d151.py`, `mutations009_d151_0.txt`, `mutations009_d151_1.txt`, and for rules 74 and 75 `mutate009_review2.py`, `mutations009_review2.txt`; the earlier runs `mutate009.py`, `mutate009_final.py`, `mutate009_d150.py` with their outputs.

### Measured

- **Tokens.** The longest of 10,000 random deals of flow B has 94 tokens (seed 66). The built round with everything said (5 announcements, 24 kontras) gives a deal of 91, in the fewest turns (`c37`) and in the most (`c41_the_longest_round_has_91_turns`: 43 events before the first card). No deal can have more than 26 + 29 + 48 = 103. The limit is 128.
- **Actions.** The longest of those random deals has 130 actions. The built round of 91 turns gives a deal of 178: 10 before the round, 120 in it, 48 cards. The bound is 187; `MAX_ACTIONS` is 192.
- **Counts of `bench_game`** (200,000 deals, every action drawn uniformly from the legal ones), on the final code: 87.6 decisions per deal (largest 123), 21.9 per seat and deal (largest 41), 79.2 tokens per deal (largest 94), 35.1 % of the decisions in the round. The means did not move with D150. Before D147 and D148: 87.5 (largest 125), 79.1 tokens, 35.0 %. In flow A a random deal had 56.8.
- **Deals per second:** 164,918 on one thread on the final code, in one run after the wrong-rule run had ended; the training session may have used the machine. Earlier: 148,895 after D150, 140,695 after D148 and 192,231 in the first execution, all next to other jobs. None is a clean number: to be measured again on a quiet machine (criterion 46). Spec 005 measured about 350,000 for flow A, with 56.8 actions per deal.
- **No leak.** 120,000 cases of flow B; 20,439 pairs of worlds compared: two hands 13,750, a hand and the hidden discards 2,613, a hand and the unshown talon 415, which card the declarer hid 1,838, the called king in another hand 1,823. 16,052 of them after something was said, 2,776 with the partner still hidden. In 1,903 the partner is another seat in the other world (or there is none there): the 1,823 of the fifth kind, 1,088 of them after something was said, and 80 by chance. 4,004 changes of the fifth kind were not a legal deal: what was said is not legal with the king in the other hand. 1,524 pairs have a seat shown as an opponent while the partner is hidden, 516 of them of the fifth kind. No difference in tokens, state or mask, in the public partner or in the public side of a seat. The first execution compared 9,352 pairs, 50 of them with another partner.
- **Coverage of the 10,000 seeded deals (`c44`), on the final code.** Announced by the declarer, the partner, an opponent: trula 3,608 / 547 / 4,562; kralji 3,652 / 517 / 4,531; kralj ultimo 757 / 1,949 / 0; pagat ultimo 1,526 / 376 / 3,601; valat 1,195 / 326 / 2,548. The game at kontra, re, sub, mort: 3,560 / 736 / 1,339 / 2,055. Announced valats won 97, lost 3,972. An announced pagat played before the last trick 5,354 times, an announced king 2,367 times. Announced pagat ultimos won 48, lost 2,159; announced kralj ultimos won 70. The numbers move a little with each of D147, D148, D150 and D151: more kontras are legal, so the same choices give other deals.
- **What all know, on 2,000 random deals (`c16`, D150).** 42,462 decisions in the round: 3,466 with a seat shown as an opponent and the partner hidden, 689 of them with two such seats, 529 with a raised bonus of such a seat; 1,658 with the declarer alone because all three other seats are shown (1,190 before D151).
- **The pairs of deals, on 5,000 random deals (`c16`, D151).** 56,440 decisions with a called king in a hand, 20,316 with the sides hidden, 9,600 of them with a seat shown as an opponent. The table of what each kind of action showed is in "The public sides by the standing rule" below.
- **The four-world search (`tests/worlds.rs`, the second review).** Every order of four things said with all five bonuses: 527,190 nodes, 97,805 of them with the sides hidden and a seat shown as an opponent. Six things with trula and kralj ultimo: 1,009,555 nodes. A declarer who is not forehand (B, dve) and another contract (C, ena), four things with three bonuses: 128,000 nodes each. About 2 seconds in all.
- **The next deal of a table (`c17`, the second review).** 525 deals at four tables, every decision compared with a fresh game; 1,003 decisions with a hidden partner in a deal that follows a deal with a seat shown as an opponent.
- **Seeds used in `c30` and `c31`,** found by trying seeds, with what they must give written by hand: 167 (tri, 43 points), 367 (tri, 21 points), 476 (dve, 35 points, the declarer plays its pagat to trick 7), 3185 (solo tri, 51 points, the trula with the declarer). The third example of the source is built by hand (`source_example_ena`): 43 points.

### Command output (trimmed)

Engine, run from `src\engine` in the worktree, on the code of the second review's commit:

```
> cargo --version
cargo 1.94.1 (29ea6fb6a 2026-03-24)
> cargo fmt --check
(no output)
> cargo clippy --locked --all-targets -- -D warnings
    Finished `dev` profile [optimized + debuginfo] target(s)
> cargo test --locked
     Running unittests src\lib.rs            test result: ok. 22 passed; 0 failed
     Running tests\actions.rs                test result: ok. 6 passed; 0 failed
     Running tests\announcements.rs          test result: ok. 11 passed; 0 failed
     Running tests\bidding.rs                test result: ok. 13 passed; 0 failed
     Running tests\cards_and_counting.rs     test result: ok. 7 passed; 0 failed
     Running tests\cardset.rs                test result: ok. 5 passed; 0 failed
     Running tests\critic_targets_reward.rs  test result: ok. 3 passed; 0 failed
     Running tests\deal.rs                   test result: ok. 5 passed; 0 failed
     Running tests\encode.rs                 test result: ok. 5 passed; 0 failed
     Running tests\encoding_contract.rs      test result: ok. 1 passed; 0 failed
     Running tests\game_play.rs              test result: ok. 3 passed; 0 failed
     Running tests\inference.rs              test result: ok. 8 passed; 0 failed
     Running tests\king_call.rs              test result: ok. 5 passed; 0 failed
     Running tests\kontra.rs                 test result: ok. 11 passed; 0 failed
     Running tests\legal_cards.rs            test result: ok. 7 passed; 0 failed
     Running tests\no_leak.rs                test result: ok. 2 passed; 0 failed
     Running tests\record.rs                 test result: ok. 6 passed; 0 failed
     Running tests\scoring.rs                test result: ok. 28 passed; 0 failed
     Running tests\scoring_b.rs              test result: ok. 15 passed; 0 failed
     Running tests\state_features.rs         test result: ok. 9 passed; 0 failed
     Running tests\talon.rs                  test result: ok. 9 passed; 0 failed
     Running tests\tokens.rs                 test result: ok. 7 passed; 0 failed
     Running tests\trick_winner.rs           test result: ok. 10 passed; 0 failed
     Running tests\vecenv.rs                 test result: ok. 14 passed; 0 failed
     Running tests\views.rs                  test result: ok. 3 passed; 0 failed
     Running tests\whole_deals.rs            test result: ok. 4 passed; 0 failed
     Running tests\whole_games.rs            test result: ok. 12 passed; 0 failed
     Running tests\worlds.rs                 test result: ok. 3 passed; 0 failed
     Running tests\written_deals.rs          test result: ok. 4 passed; 0 failed
   Doc-tests tarok_engine                    test result: ok. 3 passed; 0 failed
(241 tests, 0 failed; 182 before this spec)
> cargo test --locked --no-default-features
(227 tests, 0 failed; tests\vecenv.rs is left out)
> cargo run --release --bin bench_game -- 200000
deals:              200000
actions:            17510286
seconds:            1.213
deals per second:   164918
actions per second: 14438769

counted on 200000 deals, not timed:
decisions per deal:      mean 87.6, largest 123
decisions per seat-deal: mean 21.9, largest 41
tokens per deal:         mean 79.2, largest 94 (the limit is 128)
decisions in the round of announcements: 35.1 % of all
```

Binding crate, run from `src\train\tarok_env` in the worktree. Nothing was installed and no module was built:

```
> cargo fmt --check
(no output)
> cargo clippy --locked --all-targets -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s)
> cargo test --locked
test result: ok. 0 passed; 0 failed
```

### Readings and deviations

1. **D2 is a one-hot from the first decision of a deal.** Each of the six targets has its "none" entry set before the round too, and for a bonus nobody announced. So six state features that were 0 in flow A are 1 now, at every decision. A model trained before this spec sees an input it never saw. `docs/encoding.md` section 4b says it.
2. **A declarer who announces kralj ultimo with its own king is shown as alone (D147).** The first execution did not show it: reading 3 of the spec named four cases and this was not one. The user decided on 2026-11-24. See "The sides that follow from what was said" below.
3. **A turn can be a pass after the seat spoke earlier.** A seat that passed may speak in a later turn; the round ends only on three passes in a row after every seat had a turn. That is criterion 3 as written.
4. **`MAX_ACTIONS` is 192.** A round has at most 29 announcements and kontras and at most 91 turns, so a deal has at most 187 actions. The array of the environment's state that the binding hands to Python has 192 columns, not 96: a state saved by the trainer before this spec does not fit.
5. **D135, "keeps going".** A record of another version is not held to its lists of legal slots, only to the seat and to a legal chosen slot. A record of this engine's own version is checked as before. Any text is taken as a version.
6. **The random deals of the tests announce a valat one time in eight** of what uniform choice would give. Drawn uniformly, a valat is announced in nearly every deal and nothing else is scored. The environment's own random policy (`run_random`, `bench_game`) is uniform and unchanged.
7. **Block F is unchanged.** An announced pagat ultimo tells every seat who holds the pagat, and kralj ultimo who holds the called king. Block F3 ("certainly holds") does not use that. It stays sound; it knows less than it could. Question 3.
8. **Stage (d) is split.** Its property tests are Rust only and are committed. Criterion 49 needs them.
9. **The built deals moved** to `tests/common/built.rs`, and the helpers can speak a script in the round of a built or seeded deal (`with_round`). So the scoring tests of flow B are the forced deals of flow A with something said, and the play is the same card for card.
10. **Case (i) of D148 is kept on a bonus of a seat shown as an opponent (D150).** D150 does not say what a kontra on such a bonus shows. A seat other than the declarer that says kontra or sub on it acts for the declarer's side, so it is the partner: the sentence of D148 (i) is still true there, and the engine shows the partner. The declarer's own kontra there shows nothing, because case (iii) is no longer true there. Both are in criterion 15 as reworded, and in `c15_a_seat_shown_as_an_opponent_by_its_own_kontra`. D151 let it stand, with the pair-of-deals check written down; see "The public sides by the standing rule" below.

### The sides that follow from what was said (D147, D148)

Built after the first execution, on the user's answers of 2026-11-24. Order of work: the criterion text and reading 3 first, then the tests, then the code. Against the code of D147 alone, `c14` and `c15` fail; against the code before D147, `c10`, `c15` and `c36` fail too.

The engine now shows the sides as public in eight cases (reading 3). The four new ones are in one function, `shows_the_king_holder` in `src/game.rs`. It is asked before each announcement and kontra is applied, so it reads the state the action was legal in. D150 later rewrote that function (it is `Game::shows` now) and reworded cases (i) and (iii); see "A seat shown as an opponent" below.

**The three cases of D148, each checked against who may say what** (criteria 12, 14 and 15, `round_actions` and `side_known_to`). The condition of D148 was: build a case only if it follows strictly from the public history.

| Case | What is said | Why only a holder of the called king, or only the declarer's side, can say it | Follows strictly |
|---|---|---|---|
| (i) | A seat Y that is not the declarer raises a bonus of a seat X that is not the declarer, X not Y, while the partner is hidden | Y may raise only if it knows X's side (criterion 15). X is not the declarer, X is not Y, the partner is not public: the one way left is that Y holds the called king. Y is not the declarer, so Y is the partner. | yes |
| (ii) | A seat that is not the declarer says re or mort on a bonus the declarer announced | The bonus is the declarer's side's. Re and mort belong to the owner's side (criterion 12). A seat on that side that is not the declarer is the partner. Every seat knows the declarer's side, so no other knowledge is needed. | yes |
| (iii) | The declarer raises a bonus of a seat X that is not the declarer while the partner is hidden | The declarer may raise only if it knows X's side. The partner is not public, so the declarer holds the called king itself: it plays alone. | yes |

Two things came out of the check.

- **Case (i) covers kontra and sub only, as the user wrote it.** A re or a mort by such a seat Y cannot happen while the partner is hidden: the kontra before it was a raise of X's bonus by another seat than X, which is case (i) or (iii), so the sides were public already.
- **The code has no test for "another seat than the announcer" and none for "while the partner is hidden".** Neither can change the answer. The first raise of a bonus is a kontra, and kontra belongs to the other side than the announcer's, so it is never the announcer's own. From that raise on the sides are public and the flag is set already. In a solo, and with the called king in the talon, the flag is not read. A condition that cannot change the answer cannot be pinned by a test, so it is left out and the comment in the code says why.

**What follows from the four cases, without more code.** The public partner is read in one place. Once the sides are public, every seat may raise every announced bonus for its side (criterion 15), block D1 shows every announcement on one side, and block B6 or "declarer is publicly alone" in B7 is set. `c15` and `c36` check each of these after each kind of case.

**Tests of this spec that changed** (named again under criterion 48):

- `c10`: its last part pinned the first reading (a declarer's kralj ultimo shows nothing). It now expects "alone" after it, and "not known" after a declarer's trula or valat.
- `c14`: a block for case (ii): the declarer announces trula; an opponent's kontra shows nothing; the partner's re shows the partner; the declarer's own re and an opponent's sub show nothing; the partner's mort shows the partner.
- `c15`: part 2 had "D is on B's side but does not know it: no re". After C's kontra on B's bonus C is public (case i), so D may say re. Part 4 had the same for C after the declarer's kontra (case iii). A fifth part has the declarer's kralj ultimo (D147) and what the other seats may do after it.
- `c36`: a block for D147 in the state: D1, B6 and B7 after a declarer's kralj ultimo. One lower bound goes from 5,000 to 3,000: fewer observations have an announcement of unknown side.

**A further case of the same kind (question 4). Not built in that commit; built since, with D150.** A seat that says kontra or sub on the game, or on a bonus the declarer announced, is an opponent: those levels belong to the other side than the declarer's (criterion 12). Every seat can see that. The engine does not use it: the public state has "the partner is this seat", "the declarer is alone" and "not known", and nothing for "this seat is an opponent, the partner is still hidden". Two things would follow from it. A seat could raise a bonus that such a seat announced (criterion 15 lets it only when it knows the announcer's side). And when all three other seats are shown as opponents in this way, the declarer plays alone, which the public state can say.

### The review pass over `719a968..5f198bc`

The review found no rule defect and four weak places in the tests. Each point was checked before anything changed. One commit holds the tests; no code changed for them, because no new test failed on the code as it was.

1. **The hidden partner.** Checked: true. The wrong rule "re or mort on any target by another seat than the declarer shows the partner" passed every named test of the first execution.
   - *Where I disagree, after D148:* that rule is not a wrong rule any more. On a bonus of the declarer it is case (ii). On a bonus of another seat a re cannot come before the sides are public (see above). So it cannot be planted. The rule of the same kind that is still wrong shows too much from the other side: "every kontra or sub on the game or on a bonus of the declarer shows a partner". It is rule 19 of the list, planted again on the new code, with rules 59 and 60.
   - New named test `c14_what_is_said_and_shows_no_partner` in `tests/kontra.rs`. One round, written by hand, in two worlds that differ in who the partner is (C or B). Four announcements by three seats, kontra and sub by an opponent on the game and on the declarer's trula, re and mort by the declarer on both. After each of the 24 actions the public partner is "not known" in both worlds, and at the end the public view prints the same in both. Its second half is the other side: the same things said in both worlds up to C's turn, then one action of C (kralj ultimo, re on the game, re on the declarer's trula, kontra on B's kralji). Each shows C as the partner where C holds the king, and is not legal for C where B holds it.
   - `tests/no_leak.rs` has a fifth kind of change, on deals of flow B only: the called king goes to another hand the observer does not see. The partner is another seat in the other world, or there is none, or there is one now. If the same actions are legal there, the observer's tokens, state and mask must be the same. This is the test of D148 at scale: if a case showed a "partner" that the actions do not prove, the other world would show another seat and the observations would differ.
2. **An early king and an early pagat while the announcer's side takes the last trick.** Checked: no case written by hand had it. On the final code the two wrong rules (61 and 62) fail the new cases of `c21` and `c22`, and `c42`. Rule 62 also fails `c31`, a test of the first execution: its second example (seed 476) is a declarer who plays the announced pagat to trick 7. So for the pagat the rule did not pass every named test, but it was caught by a seed found by trying, not by a built deal. Three built deals, in `c21` and `c22`: the declarer's called king played to trick 11 and the declarer takes trick 12 (−20; −40 under kontra); the declarer's pagat played to trick 11 and the declarer takes trick 12 (−50); an opponent's pagat played to trick 2 and that opponent takes the last trick (+50 for the declarer).
3. **A king or a trula card in the packet not taken, next to an announcement.** Checked: no case written by hand had it. Two wrong rules for it, 63 (the opponents' side) and 64 (the declarer's side), fail the new cases of `c18` and `c19`, and `c42`. Four built deals, in `c18` and `c19`, for both sides: the opponents have škis and mond in their tricks and the pagat lies in the talon (an opponent announces trula: −20 for the declarer; the declarer announces: −20); the declarer has škis and mond and the pagat lies in the talon (the declarer announces: −20; an opponent: +20); the same two for kralji with the srce kralj in the talon.
4. **The second scorer has the shape of `score.rs`.** True: it was written from the same criteria and follows the same order. A mistake in the reading of a criterion would be in both. What it does prove is that the two texts agree on 10,000 random deals, and that the piles are collected in two ways. **What rests on `c42` alone:** before the review, rules 61, 63 and 64: no test with numbers written by hand failed on them, only the comparison of the two scorers. Now none of the 64 planted rules: every one fails a test with the expected numbers written by hand. Combinations without a hand-written case, which only `c42` (and the sums of `c32` and `c43`) see:
   - sub and mort on a bonus other than the trula (`c27` takes the trula up all four levels and the others to kontra or re);
   - an announced trula of the opponents in a deal with a partner (the cases by hand are solos, and one kralji case on written deal 1);
   - several announced bonuses at different levels in one deal with a kontra on the game (`c32` has such deals but checks sums, not amounts);
   - an announced bonus of the partner other than kralj ultimo and pagat ultimo under kontra.
   These use the same multiplier and the same owner sign as the cases by hand, so a wrong rule there would need to depend on the combination.
5. **Criterion 41, the longest round in turns.** Built: `c41_the_longest_round_has_91_turns` in `tests/whole_games.rs`. Three passes, then each of the 29 things in a turn of its own with two passes before the next, then three passes: 91 turns, the bound.
6. **Small.** `docs/encoding.md` pointed to section 4a for "done"; it is 4b. `c44` counted an announced pagat ultimo as "made" when the part was 50 or more in either direction; it now counts won and lost apart.

### A seat shown as an opponent (D150)

Built after the user's answer to question 4 (D150, Batch 19, item 1), on the branch before the merge. Order of work: the spec text first (criteria 15, 16 and 36, reading 3), then the tests, then the code. On the code of `6bce5d8` five tests fail: `c15_a_seat_shown_as_an_opponent_by_its_own_kontra`, both `c16` tests, `c36` and `c38_no_leak_with_announcements_and_kontra`. (For that run the old code got the new accessor `PublicView::side_of` with what the old engine knew, so the tests compile.)

**What is built**

- The engine keeps, for each seat, "shown as an opponent". `PublicView::side_of(seat)` gives what all know about a seat's side: the declarer's, the opponents', or not known.
- A seat is shown so when it says kontra or sub on the game or on a bonus the declarer announced.
- `side_known_to` reads it. So a seat may raise a bonus of such a seat for its side, and block D1 shows that seat's announcements on the opponents' side for every observer.
- When all three other seats are shown as opponents, the declarer is publicly alone ("declarer is publicly alone" in B7).
- The encoding has no new feature. No reason was found to add one: see "The encoding" below.

**The check of each consequence against who may say what** (criteria 12, 14 and 15)

| What follows | Why it follows from the public history alone | Strict |
|---|---|---|
| A seat that says kontra or sub on the game is an opponent | The game is the declarer's side's. Kontra and sub are said by the other side (criterion 12), and every seat knows its own side. No knowledge of another seat is needed. | yes |
| The same on a bonus the declarer announced | The bonus is the declarer's side's, and all know the declarer's side. | yes |
| Every seat may raise a bonus of a seat shown as an opponent, for its side | Criterion 15 asks that the seat knows the announcer's side. It does: all do. | yes |
| D1 shows such a seat's announcements on the opponents' side | Criterion 36 uses the rule of criterion 15. | yes |
| All three other seats shown as opponents: the declarer plays alone | No seat is left to be the partner. | yes |
| Two seats shown as opponents: the third is the partner | **No.** The declarer may hold the called king itself, and that is not public. Not built. `c15` (part 2) has the two worlds: the same things said, C the partner in one and the declarer alone in the other, and the public view prints the same. | no |

**What D150 changes in the cases of D148.** The check showed that two of them were no longer true as written, so criterion 15 is reworded.

- Case (iii), "the declarer raises a bonus of another seat while the partner is hidden: it holds the called king". False when the announcer is shown as an opponent: the declarer knows that side from what was said, not from its hand. With D150 added to the code of `6bce5d8` and nothing else, such a kontra would have made the true sides public: a leak. Now: on a bonus of a seat shown as an opponent the declarer's raise shows nothing. On a bonus of a seat whose side is not public, case (iii) stands.
- Case (i), "another seat than the declarer raises a bonus of another seat while the partner is hidden: it is the partner". For kontra and sub it still holds, also on a bonus of a seat shown as an opponent, for another reason: the bonus is publicly the opponents', kontra and sub are said against the owner, so the seat acts for the declarer's side. Kept, and `c15` (part 1) has it. For re and mort it no longer holds: after the declarer's kontra on such a bonus, another opponent may say re. The engine shows nothing for that re.

So the function that says what an action shows (`Game::shows` in `src/game.rs`) now reads: a raise of a target whose side all know shows the side of the seat that raises (for the declarer's side: the partner; against it: an opponent; the declarer's own raise: nothing). A raise of a bonus whose side is not public shows a holder of the called king.

**What else follows, said and not built**

1. **Re or mort on a bonus of a seat shown as an opponent, by another seat, shows that seat as an opponent too.** It acts for the opponents' side. D150 names kontra and sub on the game and on a bonus of the declarer only, so the engine does not record it. `c15` pins that it shows nothing, and above all no partner. Question 5. **Built since, with D151.**
2. **The declarer's own knowledge.** A declarer who does not hold the called king and sees two seats shown as opponents knows that the third seat is its partner: the king is not in the talon (that would be public) and not with an opponent. That is private knowledge, not public: the other seats cannot tell whether the declarer holds the king. The engine does not use it. ~~So such a declarer may not raise its partner's bonus. If it were built, case (iii) would need one more condition.~~ **Corrected (second review, point 1): that was wrong.** A hidden partner's bonus stays at level 0 until the sides are public, because no opponent knows whose it is and kontra is for the other side. So the conclusion would change no legal set and case (iii) would stay as it is; it would only put the partner's announcements on the declarer's side in the declarer's own block D1. Question 5. Decided with D151: private conclusions are not used; the main session asks the user again, since this wrong reason was in front of them.
3. **An opponent that is not shown learns nothing more** from another seat being shown: the third seat can be the partner or an opponent.

**Can "shown as an opponent" contradict the nine cases?** No. Each rule shows a fact that is true of the deal: a seat is shown as an opponent only by an action the declarer's side may not take, and a seat is shown as holder of the called king only by an action that only the holder, or only the declarer's side, may take. Two true facts about one deal do not contradict. It is also tested: `c16_what_all_know_about_the_sides_follows_from_what_was_said` plays 2,000 random deals of flow B and checks at every decision before the called king is played that (a) the public partner and the public side of every seat are what a second reading of the public events gives (`tests/common/sides.rs`, which looks at no hand), and (b) both are true of the deal. 42,467 decisions in the round: 3,484 with a seat shown as an opponent and the partner hidden, 672 of them with two such seats, 547 with a raised bonus of such a seat, and 1,190 with the declarer alone because all three are shown.

**No leak.** `tests/no_leak.rs` now also compares the public partner and the public side of every seat in the two worlds of each case. 1,526 compared pairs have a seat shown as an opponent while the partner is hidden, 515 of them with the called king in another hand. What a seat is shown as is the same in both worlds every time.

**The encoding.** No feature holds "this seat is shown as an opponent" by itself. It reaches the network in three ways: D1 (that seat's announcements are on the opponents' side), the legal mask, and the tokens (a kontra token carries the seat and the target, so the fact can be read from the sequence). "Declarer is publicly alone" in B7 is set in the ninth case. A model that reads only the state vector cannot see a shown opponent that has announced nothing. That is the same as before D150 for everything else the tokens carry and the state does not, so no feature was added and `encoding_version` stays 1. `docs/encoding.md` section 4b says where it shows.

**Tests of this spec that changed with D150**

- `c16_legal_actions_depend_only_on_what_the_seat_knows`: in its pair of deals D now also announces the valat (C did). D is shown as an opponent in both deals, and the declarer may say kontra on D's valat in both. Its second legal set, written from the public and the private view, reads the shown opponents from the public events.
- `c36`: a block for D150 (D1 before and after B's kontra; B7 with all three shown). Its own second reading of D1 (`d1_bits`) reads the shown opponents from the public events. The lower bound for "observations with an announcement of unknown side" goes from 3,000 to 1,200: 1,896 are left.
- `c38` in `tests/no_leak.rs`: the comparison of the public partner and the public sides, and two counts.
- New: `c15_a_seat_shown_as_an_opponent_by_its_own_kontra` (by hand, three parts) and `c16_what_all_know_about_the_sides_follows_from_what_was_said`.

### The public sides by the standing rule (D151)

Built after the user's answer to question 5 (D151, Batch 20). Order of work: the spec text first (reading 3, criteria 15 and 16), then the tests, then the code. On the code of `d2e4c1b` five tests fail: `c15_a_seat_shown_as_an_opponent_by_its_own_kontra` (it had pinned the old behaviour), `c15_the_pairs_of_deals_for_a_bonus_of_a_seat_shown_as_an_opponent`, both `c16` tests on what all know, and `c36`.

**The rule, and how the engine applies it.** The engine shows every fact about the sides that every seat can work out strictly from what was said and from who may say what, and no other. Every seat knows the rules of the round and the public state. So for an action of a seat every seat can ask two questions: could the seat take it if it held the called king, and could it if it did not?

- Only if it held the king: the seat holds it. Another seat than the declarer is the partner; a declarer plays alone.
- Only if it did not: the seat is an opponent.
- In both cases: the action shows nothing.

`Game::shows` in `src/game.rs` asks exactly these two questions. The legal set of the round is one function of "holds the called king" (`round_actions_of`), so the questions use the rules the legal set uses, and no list of cases is kept in the code. The public state is, as before, "who holds the king is shown" and "which seats are shown as opponents".

**The pair of deals, as a test.** `c16_the_engine_shows_what_no_pair_of_deals_can_tell_apart` does the check of the rule for every decision of the round on 5,000 random deals. For the public history so far it puts the called king into each other seat's hand in turn (changed over with any one card of that seat) and takes the same actions again. A seat is a possible holder if all of them are legal in such a deal. Then:

- the engine shows a seat as the partner, or the declarer as alone, exactly when one holder is possible;
- the engine shows a seat as an opponent exactly when it is not a possible holder;
- while the sides are hidden the declarer is always a possible holder.

56,440 decisions were checked, 20,316 of them with the sides hidden and 9,600 of those with a seat shown as an opponent. "Exactly" is both directions: nothing is shown that a pair of deals can tell apart, and nothing that follows is left out.

**The full table.** Every kind of action of the round while the sides are hidden: who takes it, on whose target, at which level, and what it shows. The last column is how often the test above saw it; each kind showed the same thing every time. "Cannot be said" rows are not legal for any seat in that state, by criteria 12 and 15.

| # | Action | A holder of the king may | A seat without it may | What all conclude | Built | Seen |
|---|---|---|---|---|---|---|
| 1 | Any seat announces trula, kralji or valat | yes | yes | nothing | nothing to build | 3,757 |
| 2 | A seat announces pagat ultimo | if it has the pagat | if it has the pagat | nothing about the sides (it shows who holds the pagat) | nothing to build | 666 |
| 3 | A seat announces kralj ultimo | yes | no | it holds the king: another seat than the declarer is the partner, the declarer plays alone | yes: criterion 10, D147 | 822 |
| 4 | A seat says "done", also as a pass | yes | yes | nothing: not saying a thing shows nothing | nothing to build | 10,565 |
| 5 | Kontra or sub on the game or on a bonus of the declarer, by a seat not yet shown | no | yes | it is an opponent | yes: D150 | 1,503 |
| 6 | The same by a seat already shown as an opponent | no | yes | nothing new | — | 664 |
| 7 | Re or mort on the game or on a bonus of the declarer, by another seat than the declarer | yes | no | it is the partner | yes: criterion 14, D148 (ii) | 232 |
| 8 | Re or mort there by the declarer | yes | yes | nothing | yes: it shows nothing | 530 |
| 9 | Kontra or sub there by the declarer | no | no | cannot be said | — | 0 |
| 10 | Kontra or sub on a bonus of a seat shown as an opponent, by the declarer | yes | yes | nothing: the declarer knows that side from what was said | yes: D150 (case iii held back there) | 311 |
| 11 | Kontra or sub on such a bonus by another seat | yes | no | it is the partner | yes: reading 10, D148 (i) | 143 |
| 12 | Re or mort on such a bonus by another seat, not yet shown | no | yes | it is an opponent | **yes: D151** | 43 |
| 13 | Re or mort on such a bonus by its announcer, or by another seat already shown | no | yes | nothing new | — | 99 |
| 14 | Re or mort on such a bonus by the declarer | no | no | cannot be said | — | 0 |
| 15 | Kontra on a bonus of a seat whose side is not public, by another seat than the declarer | yes | no: it does not know whose the bonus is | it is the partner, and all sides are public | yes: D148 (i) | 134 |
| 16 | Kontra on such a bonus by the declarer | yes | no | the declarer holds the king and plays alone | yes: D148 (iii) | 74 |
| 17 | Kontra on such a bonus by its announcer | no | no | cannot be said: kontra is for the other side | — | 0 |
| 18 | Re, sub or mort on such a bonus | — | — | cannot come up while the sides are hidden: the kontra before it made them public (rows 15, 16) | — | 0 |

What follows from the state, not from one action:

| # | State | What all conclude | Built |
|---|---|---|---|
| 19 | All three other seats are shown as opponents | the declarer plays alone | yes: D150 |
| 20 | Two seats are shown as opponents | nothing about the third. A pair of deals exists: the third seat holds the king, or the declarer does (`c15`, parts 2 and 3) | not built: does not follow |
| 21 | One, two or no seat shown | never "the declarer has a partner": the declarer is a possible holder in every hidden state (asserted by the test) | nothing to build |
| 22 | Two seats shown, and the declarer does not hold the king | the declarer alone can work out that the third seat is its partner | not built: a private conclusion (D151, item 2). It would change no legal set, only the declarer's own block D1 (second review, point 1). `c15_the_pairs_of_deals…`, `c36` and the four-world search pin that the engine gives the declarer "not known" there, and rule 74 of the wrong rules plants it |

Outside the round nothing more follows from what was said: any seat may bid, the declarer may choose any contract it won, and it may call a king it holds. The two cases that come from cards all have seen stay as they were: the called king in the talon, and the called king played.

**The pair-of-deals check of reading 10** (row 11), written down as asked. B says kontra on the game and announces trula. Then the same history is set up in three deals that differ only in who holds the called king: C, the declarer A, or D. The public view prints the same in all three. C's kontra on B's trula is legal where C holds the king and not legal in the other two. A deal in which B holds the king does not have this history: B's kontra on the game is not legal there. So no deal with this history and this kontra has another holder than C: C is the partner. The test is `c15_the_pairs_of_deals_for_a_bonus_of_a_seat_shown_as_an_opponent`. Reading 10 stands.

**The case built with D151** (row 12), in the same test. After the declarer's kontra on B's trula (legal in all three deals: it shows nothing), D's re is legal where D is an opponent (C holds the king, or A does) and not legal where D holds it. So D is shown as an opponent, and the partner stays hidden. Then C's mort is legal only where C is an opponent; with it all three are shown and the declarer is alone.

**Private conclusions are not used (item 2).** What a seat knows about another seat's side is the public state, its own side, and for a holder of the called king the fact that every other seat but the declarer is an opponent. `side_known_to` reads nothing else; the legal set and block D1 both go through it.

**Cost.** For each announcement and each kontra the engine now works out the legal set of the speaker twice more. "Done" costs nothing more. One run after the wrong-rule run had ended gave 164,918 deals per second, against 148,895 after D150. Both are noisy, so no slowdown can be read from them. A clean number belongs to criterion 46.

**Tests of this spec that changed with D151**

- `c15_a_seat_shown_as_an_opponent_by_its_own_kontra`: after D's re on B's trula it expected "D not shown". It expects D shown as an opponent.
- `tests/common/sides.rs`, the second reading of the public events: one rule for a target whose side all know (a raise by another seat than the declarer shows that seat's side), where it had left out re and mort on a bonus of a shown opponent. `c16_what_all_know…`, `c16_legal_actions…` and `c36` read it.
- New: `c15_the_pairs_of_deals_for_a_bonus_of_a_seat_shown_as_an_opponent` and `c16_the_engine_shows_what_no_pair_of_deals_can_tell_apart`.

### The second review pass over `dd012d9..d2e4c1b`

The pass looked at the code that works out the public sides. It found no defect in the engine, one wrong statement in these Results, three gaps in the tests and one note. Each point was checked before anything changed. Its own check was a search through four worlds at once, exhaustive to eight things said; it also found that the engine of `d2e4c1b` plus the first item of question 5 shows exactly what is strictly public. That is what D151 built.

1. **A wrong statement in these Results. Checked: it was wrong, and it was mine.** Question 5, item 2, and the section on D150 said that a declarer who works out its partner privately "may not raise its partner's bonus", and that with it built case (iii) of D148 would need one more condition. Neither is so. The first level on a bonus is kontra, which only the other side says. No opponent knows the side of a hidden partner, so a bonus of the partner stays at level 0 until the sides are public, and the declarer's re on it cannot be reached. With the private conclusion built, no legal set changes and case (iii) stays as it is. The only effect is the declarer's own block D1: its partner's announcements would show on the declarer's side for the declarer alone. The reviewer measured 81 decisions in 9 of 2,000 seeded deals. The two places are corrected below, each with a note; `docs/encoding.md` had the same sentence and is corrected. The user decided "private conclusions are not used" (D151, item 2) with that wrong reason in front of them; the main session asks again. **Until then it is not built**, and the tests pin that (point 3).
2. **Test gap: state carried into the next deal of a table. Checked: true.** A table starts each deal as a fresh game (`Table::deal_out`), so the engine as written cannot carry anything over. But no test would have caught a version that did: the environment tests draw every action uniformly, and then nearly every auction ends in a solo. In 547 such deals not one decision had a hidden partner. New test `c17_every_decision_of_a_table_is_that_of_a_fresh_game_of_its_deal` in `tests/vecenv.rs`: at every step and every table, the tokens, the state, the mask and the critic's features are those of a fresh game of that deal's seed and dealer that takes the deal's actions. Its auctions pass three times out of four, as the other random deals of the tests do. 525 deals at four tables; 1,003 decisions have a hidden partner in a deal that follows a deal with a seat shown as an opponent. Wrong rule 75 plants the reviewer's version.
3. **Test gap: nothing pinned that the private conclusion is not built. Checked: true at `d2e4c1b`.** Now pinned three times: by hand in `c15_the_pairs_of_deals_for_a_bonus_of_a_seat_shown_as_an_opponent` (`knows_side_of` for the third seat is "not known" to the declarer and to both shown opponents) and in `c36` (block D1 of the declarer shows the partner's kralji on both sides); and at every node of the four-world search (point 4), which holds what each seat knows to "the public state, its own side, and everything for a holder of the king". Wrong rule 74 builds the conclusion and fails these.
4. **The second readings follow the engine. Checked: true as the code stood.** `known_from_events` had the same arms as `Game::shows`, and `legal_from_what_the_seat_sees` and `d1_bits` take the public partner from the engine. Two things are different now.
   - Since D151 `Game::shows` has no arms: it asks two questions about the legal set. `known_from_events` is still a list of cases, so the two are no longer the same text. They can still share a wrong reading of the rules.
   - Two checks in the repo do not read the engine's rules at all, only which actions are legal in which deal. **The four-world search** is in the tests: `tests/worlds.rs`, taken from the reviewer's file, with the equality asserted (what the engine shows is exactly what is true in all alive worlds) and with what each seat knows held to the list of criterion 15. Its sizes: every order of four things said with all five bonuses, 527,190 nodes; six things with trula and kralj ultimo, 1,009,555 nodes; and, widened as asked, a declarer who is not forehand (B plays dve) and another contract (C plays ena), four things with three bonuses, 128,000 nodes each. 1.79 million nodes in about 2 seconds. **The pairs of deals on random deals** (`c16_the_engine_shows_what_no_pair_of_deals_can_tell_apart`, D151) covers what the search does not: random hands, every contract that calls a king, every dealer, the pagat with any seat. Limits that remain: the search uses one deal (`round_deal`) with the pagat at B in every world; both checks stop at the first lead.
5. **Note: the version stayed 0.3.0 while D148, D150 and D151 changed which kontras are legal.** True. No module was built from the branch and nothing was installed, so no record and no checkpoint was written by a build between those commits. The first build of 0.3.0 is the one of stage (d), from the final code. No change.

Small: the "Kontra" bullet of `src/engine/README.md` listed who knows a side without "shown as an opponent". Corrected.

Not looked at by this pass, as its report says: the scoring tests of `c651848`, the play after the round, dve and ena, the Python side.

### Open issues

1. **Criteria 45, 46 and 53 and the Python side of 52 are not done.** They need the binding module built from these sources, which waits for the main session's word that the runs of spec 017 have ended (D146). What that stage has to do, as far as it can be seen from the Rust side:
   - Build the module from the worktree (`maturin develop --release` in `src	rain	arok_env`). The module in the shared `.venv` then is engine 0.3.0, and the check of the shared checkout against its own engine sources (0.2.0) will refuse it. So this cannot happen while the runs of specs 015 and 017 use that environment, or it needs a virtual environment of its own for the worktree.
   - `EngineEnv.step(actions, hold=None)` in `tarok_train/env.py`, and a test in `src/train/tests` that a held table does not move and that `step(actions)` works as before (criterion 52). The binding's `step_into` already takes `hold` as an optional last argument.
   - `test_spec006.py`: one message names the shape `(96, 4)`, which is `(192, 4)` now; two checks use the absolute 1e-6 of question 1. Any Python test that expects block D to be zero, the phase entry 5 never to be set, or the slots 77 to 88 never to be legal has to change, and is to be listed here.
   - The environment state the trainer saves has 192 columns of actions, not 96: a checkpoint of an earlier run does not load its environment state (reading 4).
   - `pytest` on the CPU and with the GPU tests; a run of `smoke-a-ci.toml` with the round in play; the entropy of the phase "announcements" in its log (criterion 45).
   - `bench_game` and the environment benchmark (`python -m tarok_train.env`) on a quiet machine (criterion 46).
   - Criterion 53 (D148, B17-2 a): the trainer writes the engine version into every checkpoint; resuming a checkpoint of another engine version, or of none, is refused with an error that names both versions; the evaluation takes such a checkpoint as agent or as opponent only with a flag and prints a warning. Tests first, in the Python tests of this spec. `encoding_version` stays 1.
   - The `train` workflow, which runs on a pull request or on `main`.
2. **The `train` workflow has not run on this branch.** It runs on `main` and on pull requests.
3. **Block F on deals of flow B has no soundness test of its own.** `tests/inference.rs` plays deals in which all pass. Block F does not read the round, so nothing changed for it; a test would be needed with question 3.
4. **A reward can be above 100.** Mort on an announced valat is 8,000 points, a reward of 160 for the declarer. The scale is unchanged (reading 4 of the spec). The training spec for flow B has to look at it.
5. **The deals per second are not a clean number yet** (see "Measured").
6. **The private conclusion of the declarer (D151, item 2) waits for the user's second answer.** It is not built. If the answer is to build it, the change is one line in `side_known_as`, the three tests that pin it and the note in `docs/encoding.md`.

### Questions for the user

Questions 1 to 3 were answered on 2026-11-24 (D147): 1 (a); 2 (b), built; 3 not in this spec. Question 4 was answered the same day (D150): (b) with (c), built. Question 5 was answered on 2026-11-25 (D151): (a) for both, as a standing rule; built. They stay below as they were asked. **No question of mine is open. One answer is asked for again by the main session: item 2 of question 5, because the reason I gave there was wrong (correction below the question).**

1. **Rewards as 32-bit numbers.** Spec 006 asked that the ten parts of a seat sum to its reward within 1e-6. With kontra that cannot hold for large rewards. Options: (a) the room grows with the reward: 1e-6 times the largest reward of the deal, as the engine's tests now have it; (b) the rewards become 64-bit numbers in the arrays, which changes the contract and the binding; (c) the reward scale grows with flow B so rewards stay small. Recommended: (a). The parts and the reward are exact as whole points in the record; only the divided numbers round.
2. **A declarer who announces kralj ultimo with its own king.** Options: (a) as built: the engine shows nothing; the other seats may not kontra each other's announcements until the king falls; (b) the declarer is shown as alone from that announcement on, as a player at the table would conclude. Recommended: (b). It is a fifth case next to the four of reading 3, and one line in `PublicView::partner`.
3. **Block F and the announcements.** Options: (a) as built: block F does not use them; (b) F3 holds the pagat for the seat that announced pagat ultimo and the called king for the seat that announced kralj ultimo, until the card is played, with a soundness test on deals of flow B. Recommended: (b), with the training spec for flow B. The network can also read it from the token, which carries the seat.
4. **A seat shown as an opponent by its own kontra.** Found while checking the cases of D148; not built, as the condition of D148 asks. A seat that says kontra or sub on the game, or on a bonus the declarer announced, is an opponent: those levels belong to the other side than the declarer's. Every seat sees it. The engine does not use it. Example: D says kontra on the game and announces trula. The declarer does not hold the called king and the partner is hidden. At the table the declarer knows D's trula is the opponents' and may say kontra on it. In the engine it may not: it "does not know D's side". Options: (a) leave it: the engine shows only what reading 3 lists; (b) the engine keeps, for each seat, "shown as an opponent", and `side_known_to` uses it. A seat may then raise a bonus of such a seat, and block D1 shows that seat's announcements on the opponents' side. Criteria 15, 16 and 36 and reading 3 change; the encoding does not need a new feature; (c) with (b), or alone: when all three other seats are shown as opponents, the declarer is publicly alone. That is a ninth case of reading 3 and the public state can say it today. Recommended: (b) with (c), in a spec change of its own after stage (d), because it changes what is legal and so the second legal set of `c16`. Reason: it is the same kind of conclusion as D147 and D148, and without it the engine again forbids a kontra that a player at the table may say.
5. **Two more things that follow, found while checking D150; neither is built.** (1) A seat other than the announcer that says re or mort on a bonus of a seat shown as an opponent is an opponent too: it acts for the opponents' side. The engine records "shown as an opponent" only for kontra and sub on the game or on a bonus of the declarer, as D150 says. (2) A declarer who does not hold the called king and sees two seats shown as opponents knows that the third seat is its partner. That is the declarer's own knowledge, not public. The engine does not use it, so that declarer may not raise its partner's bonus. Options: (a) leave both; (b) build (1): the rule becomes "on a target whose side all know, a raise by another seat than the declarer shows that seat's side", with no exception. Criterion 15 gets one sentence; the encoding does not change; (c) build (1) and (2). With (2), a declarer's raise of the third seat's bonus no longer proves that it holds the called king, so case (iii) of D148 gets one more condition, and what the declarer may do then depends on knowledge the other seats cannot check. Recommended: (b). Reason: (1) is the same conclusion as D150 from the other side and closes the last exception in the rule. (2) is rare, private, and makes case (iii) harder to state; a player at the table would use it, but the gain is one kontra in few deals.

**Correction to question 5, item (2) (second review, 2026-11-25).** Three sentences above are wrong: "so that declarer may not raise its partner's bonus", "case (iii) of D148 gets one more condition", and "the gain is one kontra in few deals". A bonus of a hidden partner cannot be raised by anyone before the sides are public: its first level is kontra, which only the other side says, and no opponent knows whose the bonus is. So with the private conclusion built no legal set changes, case (iii) stays as it is, and nothing the declarer may do depends on it. The one effect: in the declarer's own block D1 its partner's announcements would stand on the declarer's side, not on both. The reviewer built it to measure: 81 decisions in 9 of 2,000 seeded deals. Options as they should have read: (a) leave it out: the state shows a seat only what all know, what it holds and its own side; (b) build it: block D1 of that declarer is more exact in those decisions, and the rule "what a seat knows" gets one more line that only the declarer can use. It is not built; three tests and wrong rule 74 pin that.
