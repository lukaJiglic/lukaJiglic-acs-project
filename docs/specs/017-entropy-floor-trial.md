# 017 — Trial: the entropy controller's lower limit at 0.001

Status: **draft** (2026-11-24). Waits for the user's approval of the acceptance criteria (D73) and for the commit of spec 015.

Tool and model: Claude Code, claude-fable-5-2. Drafted by the main session; executed in the `train-env` session.

## Goal

Find out before the full run whether a lower limit of 0.001 on the entropy weight lets the play firm up, and whether the agent is stronger for it (D142, B14-2 a). The spec measures and reports. The user decides the value for later runs.

## Why

Measured in spec 014: the entropy of the play ends at 0.46 (MLP) and 0.51 (transformer) against a target of 0.25. The weight of the play sits at the lower limit 0.01 from start to end. The controller wants less entropy and cannot ask for it.

Checked in the config: `[entropy_control] min_weight` is one value for all phases. Lowering it lowers the limit of every phase, not only of the play. Criterion 9 reports which phases use the new room.

## Needs

- Spec 015 committed: a checkpoint as the opponent in the evaluation, and the transformer's memory remedy.
- The installed engine module of version 0.2.0 (deal flow A). The trial is compared with the run of spec 014, which was played on it. See open point A.

## In scope

- Run configs for two arms: `base` (lower limit 0.01) and `trial` (lower limit 0.001). Both are the configs of spec 014 with 500 updates.
- MLP: three seeds per arm (14, 15, 16), six runs of about 15 minutes.
- Transformer: one new run, the trial arm with seed 14. Its base arm is the run of spec 014, checkpoint 500. About 80 to 105 minutes.
- The comparison of the update-500 checkpoints, one against three, with the evaluation of spec 015.

GPU time, estimated from the speeds of spec 014: about three and a half hours with the comparisons.

## Out of scope

- A lower limit per phase. That is a code change and its own spec if the trial points to it.
- Other values than 0.001. Other targets (C16 stays open).
- The aligned collection (spec 016). Both arms use the collection of spec 014.

## Acceptance criteria

**Configs**
1. `trial-floor-base-mlp.toml` differs from `long-a-mlp.toml` only in `name`, `updates` (500) and `time_limit_minutes`. `trial-floor-low-mlp.toml` and `trial-floor-low-transformer.toml` differ from their `long-a` files in those three and in `min_weight` (0.001). A test checks this key by key.
2. The seed comes from the command line for the seeds 15 and 16, or from one config per seed. In both cases the run folder holds the config as it ran, with its seed.

**Runs**
3. The seven runs finish with 500 updates each. A run that hits its time limit is resumed and that is said in the Results.
4. The loop checks of spec 008 hold in every run: no illegal action, the rewards of a deal add up to zero, no logged number is not finite.
5. Before each run the card is read with `nvidia-smi`. A run during which another program held more than 300 MiB of dedicated memory is marked as disturbed. Its speed is not used. Its checkpoint is used.
6. The base arm of the transformer is the checkpoint 500 of `long-a-transformer-20261124-011623`. The Results say that it was made before the changes of spec 015, which change no training setting.

**What is reported**
7. Per run, from the logged tags: the entropy of the play and its weight at the updates 100, 200, 300, 400 and 500; the lowest weight the play reached; the update at which the weight first went under 0.01.
8. Per run: the entropy of every phase at update 500 beside its target, and the number of phases outside the band.
9. Per run of the trial arm: every phase whose weight went under 0.01, with its lowest value.
10. Per run: the contract shares and the share of deals the declarer's side won at update 500. This shows a collapse if there is one.
11. Each update-500 checkpoint against random legal play, 2,000 deals in four seatings, seed 2008, with the 95% interval.

**The comparison** (evaluation of spec 015, 2,000 deals in four seatings, seed 5151, mean reward per deal of the single seat with the 95% interval)
12. For each MLP seed and for the transformer: one `trial` seat among three `base` seats, and one `base` seat among three `trial` seats. Eight comparisons.
13. The size of chance between runs: the three MLP `base` checkpoints against each other, one seat among three, in both directions. Six comparisons. They show how far two runs with the same settings and other seeds lie apart.
14. The Results give a reading by this rule, fixed before the runs. "Trial ahead" for a network: in both directions the interval lies on the trial's side of zero. For the MLP this must hold for at least two of the three seeds, and the differences must be larger than the largest one of criterion 13. "Trial behind" is the mirror. Everything else is "no difference shown".
15. The spec changes no setting of later runs. Claude's recommendation follows the reading: 0.001 if the trial is ahead for the transformer and not behind for the MLP; otherwise 0.01 stays. The user decides.

**Quality gates**
16. `pytest` in `src\train` passes on the CPU and with the GPU tests.
17. The run log is in `docs/progress.md` (D79): configs, commit, duration, the numbers of criteria 7 to 13, what failed.
18. `SOURCE_HASH` does not change.

## Checks

- Criteria 1 and 2: a test in `src/train/tests/test_spec017.py`, written before the runs.
- Criteria 3 to 13: the run folders and the evaluation files. The main session reads the numbers from the files again.
- Criterion 14: the main session applies the rule on its own and compares.

## Files expected to change

Three or more new configs in `src/train/configs/`, `src/train/tests/test_spec017.py`, a script that reads the tags of criteria 7 to 10 from the run folders (if `smoke_report.py` does not already), `src/train/README.md`.

## Commits

1. `train(configs): the trial of the lower entropy limit (spec 017)`
2. `docs(progress): run log of spec 017`

## Open points for the user

- **A. When the trial runs.** After spec 009 the engine plays the round of announcements in every deal. A network trained after that plays another game than the run of spec 014.
  - (a) Run the trial right after spec 015, on the installed engine 0.2.0, before the binding is rebuilt for spec 009. Recommended: it compares with the checkpoints that exist, and the card is free in that time. Spec 009 still has its wrong-rule run, its Results and the review pass ahead. If spec 009 is ready first, its rebuild waits until the trial ends, at most about three and a half hours.
  - Reported by the engine session for the branch `spec-009`: six state features that were 0 in flow A are 1 from the first decision, and the saved environment state of an old checkpoint no longer fits. A checkpoint of spec 014 would see inputs it was not trained on.
  - (b) Run it after the merge of spec 009, on flow B. Both arms of the transformer are then new runs (about twice the time, and deals are longer), and the first look at flow B is mixed into the trial.
- **B. The seeds.** (a) Three MLP seeds per arm and one transformer seed. Recommended: the MLP is cheap and shows the size of chance; a second transformer pair costs about three more hours. (b) One seed everywhere. (c) Two transformer seeds per arm as well.
- **C. The rule of criterion 14 and the recommendation of criterion 15.** (a) As written. Recommended: with no difference shown, nothing changes. (b) With no difference shown, take 0.001 if the entropy of the play came closer to its target.

## Results

Not executed.
