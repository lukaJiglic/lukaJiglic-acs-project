# 018 — First runs on deal flow B

Status: **approved** (2026-11-24, D149). Waits for the merge of spec 009.

Tool and model: Claude Code, claude-fable-5-2. Drafted by the main session; executed in the `train-env` session.

## Goal

Train both networks for 200 updates on the engine with the round of announcements and kontra, and measure what the round does to the training. The spec measures and reports. Remedies, if any are needed, are proposed from its numbers and decided by the user (D148, B17-3 a).

## Why

Measured by the review of spec 009, on 20,000 deals with uniform random legal actions, which is about what an untrained network plays:

- A valat is announced in 19,806 deals. Every seat may announce it at every turn, and a deal with an announced valat scores nothing else.
- The game is under kontra in 19,552 deals.
- The mean entry of the declarer is 4,462 points in size. With the reward scale of 50 that is a reward of about 89. In flow A a reward was under 1 in size on average.

Not known: how fast the network drops the valat. A lost valat is a strong signal. This spec measures it.

Random legal play as the fixed opponent of the evaluation does the same in every deal. So the evaluation needs a second fixed opponent that says nothing in the round.

## Needs

- Spec 009 merged into `main`, the binding rebuilt (engine 0.3.0).
- Spec 015 committed: a checkpoint as the opponent, the memory limit of the transformer.
- Before the measurement of spec 016 (D148). The `rolling` runs of spec 016 are these runs' configs.

## In scope

- Monitors of the round, logged per update from the tokens of the trained deals.
- A fixed opponent `silent-random` in the evaluation.
- The entropy of uniform random legal play in the round, measured on the engine, and from it the target of the phase "announcements".
- Two configs `smoke-b-mlp.toml` and `smoke-b-transformer.toml`, two runs of 200 updates, their evaluations.

## Out of scope

- Any remedy: a start bias toward "done", another reward scale, a cut of large rewards. Each is a decision after the numbers.
- The aligned collection (spec 016) and the lower entropy limit (spec 017).
- Block F and the announced ultimo cards (D147, B16-3). It gets its own spec with the training on flow B that follows this one.
- A judgement of the playing strength on flow B.

## Acceptance criteria

**Monitors** (computed from the tokens 146 to 156 of the trained deals; tests on built token arrays with known contents)
1. Per update, the share of trained deals in which each of the five bonuses was announced: `round/announced_trula`, `_kralji`, `_kralj_ultimo`, `_pagat_ultimo`, `_valat`.
2. Per update, the share of trained deals with the game at each level at the end of the round: `round/game_none`, `_kontra`, `_re`, `_sub`, `_mort`. The five shares add up to 1. And `round/bonus_kontra`: the share of deals with a kontra on any bonus.
3. Per update: `reward/abs_mean`, `reward/abs_max` and `reward/share_above_10`, over the trained seat-deals.
4. Per update: `round/decision_share`, the share of the trained decisions that fall in the round, and `round/decisions_per_seat_deal`.
5. A deal is counted once, whatever number of its seats is trained.

**The silent opponent**
6. `--opponent silent-random` in the evaluation: a uniform choice among the legal actions outside the round, and "done" at every turn in the round. Test: over 500 deals among four such seats no announcement or kontra token is written and no action is illegal.
7. The evaluation's file name and its JSON name the opponent. `--opponent random` (the default) and a checkpoint as opponent work as before.

**The target of the phase**
8. The policy entropy of uniform random legal play in the round is measured on at least 30,000 deals of the engine, the way the six values of D136 were measured. The number is in the Results and in the configs' comments.
9. The target of "announcements" in the two configs follows the rule of open point A.

**The runs** (GPU, seed 8, 200 updates)
10. `smoke-b-<network>.toml` differs from `smoke-a-<network>.toml` only in the name and the target of "announcements". A test checks that key by key.
11. Before each run the card is read. A run during which another program held more than 300 MiB of dedicated memory, or during which a build ran on the PC, is marked as disturbed; its timing is not used.
12. Each run reaches 200 updates, or stops with its reason in the log. The loop checks hold: no illegal action; the rewards of a deal add up to zero within the tolerance of D147; no logged number is not finite.
13. If the transformer run stops with an out-of-memory error under the limit of spec 015: stop and report. The limit and the number of tables are not changed to get through.
14. Reported per run at the updates 1, 5, 10, 25, 50, 100, 150 and 200: the tags of criteria 1 to 4; the entropy and the entropy weight of every phase; the contract shares; the share of deals the declarer's side won; the seconds per update; the memory tags of spec 015.
15. Reported: the first update from which the share of deals with an announced valat stays under 50%, under 20% and under 5%.

**The evaluations** (2,000 deals in four seatings, 95% interval)
16. The update-200 checkpoint of each run against `silent-random` and against `random`, seed 2008.
17. Progress inside a run: update 200 in one seat against update 100 in three seats, and the reverse, seed 5151.
18. The untrained network (update 0) against `silent-random`, seed 2008: the starting point.

**The reading** (rule fixed before the runs)
19. "The flood ends by itself" for a network: over the updates 151 to 200 the mean share of deals with an announced valat is under 5%, and the mean of `reward/abs_mean` is under 2.
20. If this holds for both networks, the Results propose no remedy. If not, the Results name the remedies that fit the numbers, with what each would change, as a batch for the user. The spec changes no setting.

**Quality gates**
21. `pytest` in `src\train` passes on the CPU and with the GPU tests. The `train` workflow is green.
22. Check of the tests: at least six wrong changes, put into the code one at a time, each fail a numbered test. Among them: a monitor reads the wrong token id; a deal is counted once per trained seat; the silent opponent announces; the game's level is read from the first kontra token, not the last.
23. The run log is in `docs/progress.md` (D79). `src/train/README.md` describes the monitors and the silent opponent.
24. `SOURCE_HASH` is that of `main` after the merge of spec 009 and does not change.

## Checks

- Criteria 1 to 7 and 10: `src/train/tests/test_spec018.py`, written before the code.
- Criteria 8, 14 to 19: from the run folders and evaluation files, read by a committed script. The main session reads the numbers from the files again and applies the rule of criterion 19 on its own.
- A separate review pass over the diff.

## Files expected to change

`src/train/tarok_train/ppo.py` (the monitors), `evaluate.py` (the opponent), `smoke_report.py` or a new report script, two new configs, `src/train/tests/test_spec018.py`, `src/train/README.md`.

## Commits

1. `feat(train): monitors of the round of announcements (spec 018)`
2. `feat(train): a silent random opponent in the evaluation (spec 018)`
3. `train(configs): the first runs on flow B (spec 018)`
4. `docs(progress): run log of spec 018`

## Open points for the user

- **A. The target entropy of the phase "announcements".** Today it is 0.25, a placeholder of Claude's, because flow A had no such phase.
  - (a) A quarter of the measured entropy of random legal play in the round. Recommended: this is the rule you approved for the six other phases (D136).
  - (b) 0.25 stays.
- **B. The thresholds of criterion 19.** (a) Under 5% of deals with an announced valat and a mean reward size under 2. Recommended: a good player announces a valat in far fewer than one deal in twenty, and a mean reward size of 2 is twice that of flow A. (b) Other numbers.
- **C. The length.** (a) 200 updates per network, as the smoke run of spec 008. Recommended: about ten minutes for the MLP and under an hour for the transformer, estimated from the longer deals. (b) 500 updates.

## Answers to the open points (D149)

- A. (a) The target is a quarter of the measured entropy of random legal play in the round.
- B. (a) Under 5% of deals with an announced valat and a mean reward size under 2, over the updates 151 to 200.
- C. (a) 200 updates per network.

## Results

Not executed.
