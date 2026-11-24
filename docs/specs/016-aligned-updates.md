# 016 — Aligned updates: every played deal is trained on

Status: **approved** (2026-11-24, D146). Waits for the merge of spec 009.

Tool and model: Claude Code, claude-fable-5-2. Drafted by the main session; executed in the `train-env` session.

## Goal

The trainer plays one deal per table for an update and trains on all of them. A table that has finished its deal waits (D142, B14-1 a). The engine's part, the hold flag, is criteria 50 to 52 of spec 009. This spec is the trainer's part.

## Why

Checked in the code (`src/train/tarok_train/train.py`, `collect`, `step`, `update`):

- The tables are never in step. A collection ends when 2,048 trained seat-deals have finished. Every deal that is running at that moment is dropped, because its first decisions would be one update old.
- After an update only the tables that stand at a deal's first decision are stored from the start. The others first play out their dropped deal.

Measured in spec 014: 48.1% of the played decisions are trained on. Acting is 47% of the transformer's time.

Inferred from the code, not measured: the trained deals are not a fair sample of the played deals. A deal is stored only when it starts and ends inside one collection, so short deals are stored more often than long ones. Klop and deals with a short bidding are short. Criterion 17 measures this.

## Needs

- Spec 009 merged into `main` and the binding rebuilt: `step(actions, hold)`.
- Spec 015 committed: the memory tags and a checkpoint as the opponent in the evaluation.

## In scope

- A config key `[ppo] collection` with the values `"rolling"` (the collection as it is now) and `"aligned"`.
- The aligned collection in the trainer.
- The hold flag in the mock environment and in the wrapper `tarok_train/env.py`.
- Two run configs and a measurement of both modes, 200 updates each, both networks.

## Out of scope

- Judging the training on deal flow B. After spec 009 the engine plays the round of announcements in every deal, so the runs here are the first on flow B. They are compared with each other only. The reward size under kontra and the first look at flow B are their own spec.
- The number of tables and `seat_deals_per_update`. Both stay 512 and 2,048.
- The lower limit of the entropy controller (spec 017).
- The evaluation's own play loop. It does not collect and does not change.

## How the aligned collection works

1. At the start of a collection every table stands at a deal's first decision.
2. A collection is one or more rounds. In a round each playing table plays exactly one deal. When its deal ends it is held until the round ends.
3. A round plays only as many tables as are needed. The tables are taken in order of their number until their trained seats reach what is still missing of `seat_deals_per_update`. Without seats of a past checkpoint the first round is all 512 tables and there is no second round. With such seats a short second round fills the gap.
4. A held table gets no row in the network's batch.
5. Nothing is dropped at an update. Every table then stands at a first decision again.

## Acceptance criteria

**The config key**
1. `[ppo] collection` is `"rolling"` or `"aligned"`. A missing key or another value stops the run with an error that names the key. Every config in `src/train/configs/` has the key. The configs that exist today get `"rolling"`, so each still describes the run that was made with it.
2. With `"rolling"` the trainer does what it does today. Test: a run of three updates on the mock environment on the CPU with a fixed seed gives the same statistics, apart from the time tags, as the code before this spec. The test holds numbers that were produced before the change.

**The aligned collection**
3. At the start of every collection each table stands at a deal's first decision.
4. In a round a playing table plays exactly one deal. From the step after its deal ends to the end of the round it is held: its deal counter and its observation do not change.
5. A held table has no row in the actor's batch. Test: the rows that go through the network in an update equal the decisions played in it.
6. A round plays the lowest-numbered tables whose trained seats reach what is still missing. The trained seat-deals of an update are at least `seat_deals_per_update` and at most three more.
7. Every decision of a trained seat is trained on: `rollout/trained_share_own_seats` is 1.0 in every update. The storage drops no row at an update.
8. Every trained decision was played under the current weights: the age of every stored row is 0 (criterion 5 of spec 008).
9. Seats of a past checkpoint are drawn at the start of each deal, as today. Over 200 updates the share of deals with such a seat is within two percentage points of `past_share`.
10. The same config and seed give the same statistics twice, apart from the time tags (mock environment, CPU).
11. A run that is stopped and resumed from a checkpoint gives the same statistics as the run that was not stopped (mock environment, CPU). A checkpoint resumes in the mode of its own config.

**The hold flag on the Python side**
12. The mock environment takes `step(actions, hold)` with the meaning of criterion 50 of spec 009: a held table does not move, gives no reward, is not done, and its action is ignored.
13. The wrapper `tarok_train/env.py` passes the flag to the binding. `step(actions)` works as before.
14. Holding does not change what a table plays. Test on the mock and on the engine: 64 tables, once without holds and once with random holds; each table's sequence of deals and rewards is the same.

**The measurement** (GPU, the engine as merged with spec 009, seed 8, 200 updates)
15. Four runs finish: `rolling` and `aligned`, MLP and transformer. The configs are `smoke-a-<network>.toml` for `rolling` and two new files `aligned-<network>.toml`, which differ from them in the name and the collection only. A test checks that.
16. The loop checks of spec 008 hold in all four runs: no illegal action, the rewards of a deal add up to zero, no logged number is not finite.
17. Reported for both modes, no threshold: decisions played and trained per update, environment steps per update, the mean and the longest number of decisions of a trained seat-deal, the contract shares of the trained deals in update 1 (the weights are the same in both modes there, so a difference is the selection of the rolling mode), the four time shares, the memory tags of spec 015.
18. Speed of the transformer: the median of the seconds per update over the updates 101 to 200 is at least 10% lower with `aligned` than with `rolling`. The MLP's figure is reported. Estimated before the measurement, not measured: 14% to 23%.
19. Strength: the update-200 checkpoint of `aligned` is not behind that of `rolling`, for each network. Measured with the evaluation of spec 015, 2,000 deals in four seatings, seed 5151: the interval of one `aligned` seat among three `rolling` seats is not wholly under zero, and the interval of one `rolling` seat among three `aligned` seats is not wholly above zero. Both are reported.
20. If criterion 18 or 19 fails, or a run stops for a reason that is not the collection: stop and report. The user decides what happens with the mode. No setting is changed to pass.

**Quality gates**
21. `pytest` in `src\train` passes on the CPU and with the GPU tests. The `train` workflow is green.
22. Check of the tests: at least eight wrong changes, put into the code one at a time, each fail a numbered test. Among them: a held table gets a network row; a held table's action is applied; a table plays two deals in a round; the second round plays every table; rows are dropped at an update; the hold flag is inverted. The list is in the Results.
23. `SOURCE_HASH` does not change. This spec changes no engine source.
24. `src/train/README.md` describes both modes. `docs/architecture.md` describes the collection as built.

## Checks

- Criteria 1 to 14: `src/train/tests/test_spec016.py`, written before the code.
- Criteria 15 to 19: the run folders, read by a script that is committed; the numbers in the Results come from files, not from the console.
- Criterion 22: the list of wrong changes with the test each one failed.
- The main session verifies the numbers from the files and orders a separate review pass over the diff.

## Files expected to change

`src/train/tarok_train/train.py`, `mock_env.py`, `env.py`, `rollout.py` (if the storage needs it), `src/train/configs/*.toml` (the new key), two new configs, `src/train/tests/test_spec016.py`, `src/train/README.md`, `docs/architecture.md`.

## Commits

1. `feat(train): the hold flag in the mock environment and the wrapper (spec 016)`
2. `feat(train): the aligned collection (spec 016)`
3. `train(configs): both collections measured on 200 updates (spec 016)`

## Open points for the user

- **A. The old collection.** (a) Keep `rolling` in the code behind the config key. Recommended: the configs of specs 008 and 014 can still be run as they were, and the two modes can be compared. (b) Remove it. Less code, but the old configs describe runs that cannot be repeated.
- **B. The speed gate of criterion 18.** (a) 10%. Recommended: under that the change does not pay for a second code path. (b) No gate, only the report.
- **C. Which mode later runs use.** Decided by the user from the Results. Proposed rule: `aligned` if criteria 18 and 19 hold.

## Answers to the open points (D146)

- A. (a) `rolling` stays in the code behind the config key.
- B. (a) The gate of criterion 18 is 10%.
- C. Not decided. The user decides from the Results.

## Results

Not executed.
