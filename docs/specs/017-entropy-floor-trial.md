# 017 — Trial: the entropy controller's lower limit at 0.001

Status: **done** (2026-11-25). Committed as `931d7d4` after the main session's check of the numbers and a separate review pass, whose fourteen points are fixed (see "Review fixes"). Approved 2026-11-24 (D146); executed on 2026-11-24 and 25 in the `train-env` session on the installed engine 0.2.0, before the binding was rebuilt for spec 009. The user took the recommendation of criterion 15: 0.01 stays (D153).

Tool and model: Claude Code. Drafted by the main session with claude-fable-5-2; executed in the `train-env` session with claude-fable-5-2; the review fixes of 2026-11-25 with claude-opus-5-5 (D152).

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
1. `trial-floor-base-mlp.toml` differs from `long-a-mlp.toml` only in `name`, `updates` (500) and `time_limit_minutes`. `trial-floor-low-mlp.toml` and `trial-floor-low-transformer.toml` differ from their `long-a` files in those three and in `min_weight` (0.001). `trial-floor-low-transformer.toml` has one further difference: `gpu_memory_limit_mib = 5000` (changed with D150; it is a setting of the machine and changes no number of the training). The MLP configs keep 0. A test checks this key by key.
2. The seed comes from the command line for the seeds 15 and 16, or from one config per seed. In both cases the run folder holds the config as it ran, with its seed.

**Runs**
3. The seven runs finish with 500 updates each. A run that hits its time limit is resumed and that is said in the Results.
4. The loop checks of spec 008 hold in every run: no illegal action, the rewards of a deal add up to zero, no logged number is not finite.
5. Before each run the card is read with `nvidia-smi`. A run during which another program held more than 300 MiB of dedicated memory is marked as disturbed. Its speed is not used. Its checkpoint is used.
6. The base arm of the transformer is the checkpoint 500 of `long-a-transformer-20261124-011623`. The Results say that it was made before the changes of spec 015, which change no training setting, and that it ran without the limit on GPU memory that the trial run has (D150).

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

## Answers to the open points (D146)

- A. (a) The trial runs right after spec 015, before the binding is rebuilt.
- B. (a) Three MLP seeds per arm, one transformer seed.
- C. (a) The rule of criterion 14 and the recommendation of criterion 15 as written.
- D150 (Batch 19, item 2): the trial's transformer config sets `gpu_memory_limit_mib = 5000`; the MLP configs keep 0. Criteria 1 and 6 were changed for it.

## Results

Executed on 2026-11-24 and 25 in the `train-env` session; the fixes of the review of 25 November are in (section "Review fixes" at the end). Runs and first Results: `claude-fable-5-2`. Review fixes: `claude-opus-5-5` (D152). Not committed. Waiting for the user's read of the diff.

### Summary

- The seven runs reached 500 updates each. No run stopped, none hit its time limit, none was resumed. The loop checks hold in all of them.
- **The lower limit lowers the entropy of the play.** Over updates 401 to 500 it is 0.266, 0.247 and 0.235 in the MLP's trial arm against 0.453, 0.420 and 0.473 in the base arm; the target is 0.25, so the MLP reaches it. The transformer's play goes from 0.519 to 0.364 and stops there, 46% above the target, with its weight at the new limit 0.001 in every one of the last 300 updates. That is the picture of spec 014 one step lower.
- **By the rule of criterion 14: no difference shown**, for the MLP and for the transformer. So the recommendation of criterion 15 is: 0.01 stays. The user decides.
- **All seven new runs count as disturbed** by criterion 5: a video render of another project of the user's took the card in bursts. Their speed is not used; their checkpoints are.
- **An addition the criteria did not ask for:** the 14 comparisons were run once more with a second seed (5252). The second set reads the same by the rule, and its numbers repeat those of the first.
- **A finding about the measure, in its own section:** between two different runs the single seat tends to lose. In six of the seven pairings the mean of the two directions is 0.03 to 0.06 under zero. The stronger run's seat loses little or nothing (+0.004 to -0.025), the weaker run's seat 0.04 to 0.10. It touches how every one-against-three number so far is read.
- **GPU runs are not repeatable to the bit** (spec 007 says so). A same-seed replicate of the MLP's base arm exists from spec 014: it is equal to update 15 and then goes its own way, with an "ena" share of 72.6% against 56.6% at the end. Pairing the arms by seed carries no information after the first updates.
- A side effect of the lower limit: early in the run the entropies of the bidding and of the king call dip deeper and longer. They recover; no phase is under its band at update 500.

### Criteria

| Criterion | Result |
|---|---|
| 1 | Met. `tests/test_spec017.py` compares the files key by key: `test_c01_...` (five tests). The transformer file has the one further difference of D150 |
| 2 | Met with one config per seed: `train.py` has no option for the seed, and the spec lists no code change for one. `test_c02_...` (three tests, six with their parameters); one runs a config of seed 16 and reads the seed from the run folder |
| 3 | Met. Seven runs, 500 updates each, no time limit reached, no resume |
| 4 | Met in all seven, and in the first 500 updates of the base arm of the transformer |
| 5 | Done. The card was read before each run. All seven new runs are marked as disturbed |
| 6 | Met. The base arm of the transformer is checkpoint 500 of `long-a-transformer-20261124-011623`. It was made before the changes of spec 015 and ran without the limit on GPU memory that the trial run has (D150). Why the differences should not matter, and what cannot be checked: under "The runs" |
| 7 to 10 | Reported below. Every number from `tarok_train/trial_report.py` |
| 11 | Reported. Eight checkpoints, all from the evaluation code after the review fix of spec 015 |
| 12, 13 | Reported. Eight and six comparisons. Every number from the evaluation files, read with `tarok_train/trial_compare.py` |
| 14 | No difference shown, MLP and transformer. Applied with `trial_compare.reading` |
| 15 | No setting of a later run is changed. Recommendation by the rule: 0.01 stays |
| 16 | Met. On 2026-11-25 after the review fixes, in `src\train`: `tests/test_spec017.py` 23 passed; `pytest -m "not slow"` with the GPU 245 passed; with `CUDA_VISIBLE_DEVICES=-1` 231 passed, 13 skipped (the GPU tests) and one failed: `test_c20` of spec 006, the timing test that fails on the idle PC (spec 008, spec 015). Before the review the whole suite with the slow tests: 243 passed with the GPU, 229 passed and 14 skipped without |
| 17 | The numbers for the run log are in these Results; `docs/progress.md` is written by the main session |
| 18 | Met. `tarok_env.SOURCE_HASH` is `64a6157fb05f43c4` before the runs, in every evaluation file and after the tests |

**Which code made the tables.** Every number of a run comes from `tarok_train.trial_report.read` and every number of a comparison from `tarok_train.trial_compare` (both committed, with tests in `tests/test_spec017.py`). An untracked script, `src/train/runs/spec017/make_tables.py`, calls them and puts the numbers into the tables below; it computes nothing of its own but three columns of the first table: the other program's memory, the number of samples and the process's shared memory, which it reads from the files of the sampler. The sampler is a scratch tool of this PC, not training code, and stays untracked.

### The runs (criteria 3 to 6)

In the tables `base` is the base arm (lower limit 0.01), `low` the trial arm (lower limit 0.001) and `s14` seed 14. `transformer-base-s14` is the run of spec 014, read up to update 500 (`trial_report --at 500`).

| Run | Folder | Commit | Started | Seconds | Updates | Card before (trainer's reading) | Other program: largest dedicated MiB | Samples with it over 300 MiB | Disturbed | Process shared memory, largest MiB | Allocator's peak MiB |
|---|---|---|---|---|---|---|---|---|---|---|---|
| mlp-base-s14 | `trial-floor-base-s14-mlp-20261124-191405` | `579e3e6` | 19:14:08 | 859.7 | 500 | 21%, 4141 MiB | 4959 | 15 of 123 | yes | 82 | 438 |
| mlp-low-s14 | `trial-floor-low-s14-mlp-20261124-192834` | `579e3e6` | 19:28:36 | 868.3 | 500 | 0%, 233 MiB | 5006 | 15 of 124 | yes | 82 | 436 |
| mlp-base-s15 | `trial-floor-base-s15-mlp-20261124-194315` | `579e3e6` | 19:43:17 | 879.6 | 500 | 0%, 237 MiB | 4868 | 28 of 125 | yes | 82 | 502 |
| mlp-low-s15 | `trial-floor-low-s15-mlp-20261124-195802` | `579e3e6` | 19:58:03 | 852.0 | 500 | 0%, 247 MiB | 4748 | 14 of 121 | yes | 82 | 454 |
| mlp-base-s16 | `trial-floor-base-s16-mlp-20261124-201220` | `579e3e6` | 20:12:21 | 881.4 | 500 | 0%, 218 MiB | 5102 | 15 of 125 | yes | 82 | 380 |
| mlp-low-s16 | `trial-floor-low-s16-mlp-20261124-202707` | `c8c9b56` | 20:27:08 | 1086.7 | 500 | 0%, 246 MiB | 4953 | 52 of 153 | yes | 82 | 466 |
| transformer-base-s14 | `long-a-transformer-20261124-011623` | `67c425b` | 01:16:25 | 24883.5 (all 2,000 updates) | 500 of 2,000 read | not read | - | - | - | - | 6484 |
| transformer-low-s14 | `trial-floor-low-s14-transformer-20261124-204712` | `c8c9b56` | 20:47:14 | 4295.3 | 500 | 0%, 221 MiB | 909 | 57 of 608 | yes | 120 | 4992 |

- The runs were started one after the other by a script: the MLP runs from 19:14 to 20:45 with the two arms in turn per seed, the transformer run from 20:47 to 21:58. The evaluations of criteria 11 to 13 ran at 20:45 (MLP, 100 s) and at 21:58 (transformer, 65 s).
- The card before each run, read by the script with nvidia-smi: 1% busy and 655 MiB before the first run (after ten minutes of waiting: 100% busy and 1,155 MiB at 19:03, free at 19:08 and 19:13), then 0% and 218 to 247 MiB before each of the others. The table shows the trainer's own reading, taken some seconds later; before the first run the render had already started in between.
- **What disturbed the runs.** A video render of another project of the user's. It holds 3 to 5 GB of dedicated GPU memory for about 100 seconds at a time. During the MLP runs it showed in 14 to 52 of about 125 samples per run (one sample every seven seconds). The MLP holds about 0.5 GB, so nothing was pushed off the card: the process's shared memory stayed at 82 MiB. The last MLP run (trial arm, seed 16) had the most of it and took 1,087 s against 852 to 881 s for the other five.
- During the transformer run the render showed from 21:15 to 21:25, updates 208 to 261, with at most 909 MiB. The process's shared memory rose from 82 to 120 MiB.
- "Disturbed" is read here as: a program other than the training process and the desktop compositor held more than 300 MiB of dedicated memory in at least one sample. The compositor's counter shows about 4,000 MiB in every run and in the undisturbed runs of spec 015, so it says nothing.
- No speed of these runs is used (criterion 5). The seconds in the table are only the log.
- The base arm of the transformer: 500 of its 2,000 updates are read. Its seconds are those of the whole run of spec 014.
- Commits: the runs started on `579e3e6` and `c8c9b56`, each with this spec's uncommitted files. The main session's commits in between touch `docs/` only.

**GPU runs do not repeat.** Spec 007 says that runs on the GPU are not bit-reproducible. The files show what that means here. The run of spec 014 (`long-a-mlp-20261124-000454`) has the same seed and the same training settings as the MLP's base arm of seed 14; its config differs only in `name`, `updates`, `time_limit_minutes` and `gpu_memory_limit_mib` (0 in both). Read to update 500 it is a same-seed replicate of that base run:

| Mean over updates 401 to 500 | The run of spec 014, read to 500 | The base run of this spec, seed 14 | Base, seed 15 | Base, seed 16 |
|---|---|---|---|---|
| Share of "ena" | 72.6% | 56.6% | 87.7% | 63.9% |
| Share of "dve" | 27.4% | 43.0% | 12.2% | 36.0% |
| Entropy of the play | 0.437 | 0.453 | 0.420 | 0.473 |
| Entropy of the talon pick | 1.229 | 0.794 | 0.935 | 1.080 |
| Declarer's side wins | 62.4% | 63.5% | 60.5% | 62.8% |
| Card-location head, accuracy | 0.3715 | 0.3717 | 0.3739 | 0.3737 |

- The two are equal in `loss/total` up to update 15 and differ from update 16. By update 500 their "ena" share is 16 points apart, as far as two seeds.
- What follows: a run repeated with the same seed gives another checkpoint. Pairing the two arms by seed ties them together only in the first updates; after that the seed is one draw among others. The three seeds are three draws, not three matched pairs.
- **Criterion 6, the base arm of the transformer.** It ran on commit `67c425b`; the trial run on `c8c9b56`. Between the two, `train.py` gained the GPU reading of spec 015, the four memory tags, the limit on the allocator and the reading of old checkpoints. None of these touches the loss, the sampling, the collection or a setting of the training. Whether the limit changes a number of the training cannot be checked bit by bit on the GPU, for the reason above. The limit changes where PyTorch keeps freed memory, not what is computed.

### Criterion 7: the play

The weight in a column is the one the controller set after that update, from that update's entropy; the next update trains with it.

| Run | Lower limit | Entropy at 100 | 200 | 300 | 400 | 500 | Weight set after 100 | 200 | 300 | 400 | 500 | Lowest weight | A weight under 0.01 first set after update |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| mlp-base-s14 | 0.01 | 0.509 | 0.515 | 0.481 | 0.469 | 0.485 | 0.0100 | 0.0100 | 0.0100 | 0.0100 | 0.0100 | 0.0100 | never |
| mlp-low-s14 | 0.001 | 0.458 | 0.368 | 0.322 | 0.276 | 0.265 | 0.0010 | 0.0010 | 0.0010 | 0.0010 | 0.0010 | 0.0010 | 1 |
| mlp-base-s15 | 0.01 | 0.535 | 0.435 | 0.362 | 0.420 | 0.421 | 0.0100 | 0.0100 | 0.0100 | 0.0100 | 0.0100 | 0.0100 | never |
| mlp-low-s15 | 0.001 | 0.430 | 0.284 | 0.289 | 0.261 | 0.247 | 0.0010 | 0.0010 | 0.0010 | 0.0010 | 0.0019 | 0.0010 | 1 |
| mlp-base-s16 | 0.01 | 0.517 | 0.474 | 0.480 | 0.478 | 0.486 | 0.0100 | 0.0100 | 0.0100 | 0.0100 | 0.0100 | 0.0100 | never |
| mlp-low-s16 | 0.001 | 0.425 | 0.328 | 0.282 | 0.247 | 0.264 | 0.0010 | 0.0010 | 0.0010 | 0.0012 | 0.0036 | 0.0010 | 1 |
| transformer-base-s14 | 0.01 | 0.628 | 0.568 | 0.525 | 0.521 | 0.515 | 0.0100 | 0.0100 | 0.0100 | 0.0100 | 0.0100 | 0.0100 | never |
| transformer-low-s14 | 0.001 | 0.515 | 0.439 | 0.388 | 0.368 | 0.373 | 0.0010 | 0.0010 | 0.0010 | 0.0010 | 0.0010 | 0.0010 | 1 |
| mlp-base-s14, replicate (the run of spec 014 read to update 500) | 0.01 | 0.560 | 0.518 | 0.467 | 0.446 | 0.444 | 0.0100 | 0.0100 | 0.0100 | 0.0100 | 0.0100 | 0.0100 | never |

- In the base arm the weight of the play sits at 0.01 from start to end, as in spec 014.
- In the trial arm a weight under 0.01 is set after update 1 and used from update 2, and the weight is at 0.001 for most of the run. That says little by itself: at the start every entropy is far above its target, so every weight falls to the limit at once, in both arms.
- The MLP's play is at its target at the end in all three seeds (0.266, 0.247 and 0.235 over updates 401 to 500), and the controller then starts to raise the weight again (seeds 15 and 16: 0.0019 and 0.0036 at update 500).
- The transformer's play: 0.407, 0.380 and 0.364 as means over updates 201 to 300, 301 to 400 and 401 to 500, and 0.364 in each half of the last 100. The weight is at 0.001 in all of the last 300 updates. It has stopped 46% above its target while the controller still asks for less.

### Criterion 8: every phase at update 500

| Run | bidding (0.2) | contract choice (0.03) | king call (0.35) | talon pick (0.45) | discard (0.6) | play (0.25) | Phases outside the band |
|---|---|---|---|---|---|---|---|
| mlp-base-s14 | 0.215 | 0.042 | 0.197 | 0.698 | 0.755 | 0.485 | 1 |
| mlp-low-s14 | 0.179 | 0.041 | 0.251 | 0.588 | 0.625 | 0.265 | 0 |
| mlp-base-s15 | 0.183 | 0.016 | 0.426 | 1.054 | 0.713 | 0.421 | 1 |
| mlp-low-s15 | 0.281 | 0.060 | 0.466 | 0.471 | 0.669 | 0.247 | 1 |
| mlp-base-s16 | 0.226 | 0.062 | 0.296 | 0.797 | 0.589 | 0.486 | 3 |
| mlp-low-s16 | 0.221 | 0.029 | 0.330 | 0.388 | 0.675 | 0.264 | 0 |
| transformer-base-s14 | 0.204 | 0.008 | 0.247 | 0.840 | 0.632 | 0.515 | 2 |
| transformer-low-s14 | 0.231 | 0.018 | 0.357 | 0.361 | 0.653 | 0.373 | 0 |

The weights at update 500:

| Run | bidding | contract choice | king call | talon pick | discard | play |
|---|---|---|---|---|---|---|
| mlp-base-s14 | 0.0218 | 0.0100 | 0.1366 | 0.0100 | 0.0181 | 0.0100 |
| mlp-low-s14 | 0.0550 | 0.0010 | 0.1080 | 0.0010 | 0.0144 | 0.0010 |
| mlp-base-s15 | 0.0702 | 0.2851 | 0.1313 | 0.0100 | 0.0174 | 0.0100 |
| mlp-low-s15 | 0.0061 | 0.0010 | 0.0832 | 0.0063 | 0.0285 | 0.0019 |
| mlp-base-s16 | 0.0243 | 0.0100 | 0.0427 | 0.0100 | 0.0267 | 0.0100 |
| mlp-low-s16 | 0.0181 | 0.0395 | 0.1412 | 0.0023 | 0.0180 | 0.0036 |
| transformer-base-s14 | 0.0324 | 0.1388 | 0.0868 | 0.0100 | 0.0452 | 0.0100 |
| transformer-low-s14 | 0.0564 | 0.0246 | 0.1834 | 0.0078 | 0.0298 | 0.0010 |

The same over the last 100 updates, which swings less than one update. Entropy, mean over updates 401 to 500:

| Run | bidding (0.2) | contract choice (0.03) | king call (0.35) | talon pick (0.45) | discard (0.6) | play (0.25) | Phases outside the band, mean |
|---|---|---|---|---|---|---|---|
| mlp-base-s14 | 0.209 | 0.033 | 0.348 | 0.794 | 0.585 | 0.453 | 1.50 |
| mlp-low-s14 | 0.199 | 0.036 | 0.357 | 0.463 | 0.635 | 0.266 | 0.22 |
| mlp-base-s15 | 0.193 | 0.037 | 0.340 | 0.935 | 0.608 | 0.420 | 1.33 |
| mlp-low-s15 | 0.211 | 0.033 | 0.359 | 0.483 | 0.589 | 0.247 | 0.03 |
| mlp-base-s16 | 0.203 | 0.048 | 0.373 | 1.080 | 0.576 | 0.473 | 2.16 |
| mlp-low-s16 | 0.206 | 0.028 | 0.328 | 0.489 | 0.600 | 0.235 | 0.01 |
| transformer-base-s14 | 0.193 | 0.031 | 0.361 | 0.706 | 0.556 | 0.519 | 1.54 |
| transformer-low-s14 | 0.186 | 0.046 | 0.351 | 0.471 | 0.586 | 0.364 | 0.33 |
| mlp-base-s14, replicate (the run of spec 014 read to update 500) | 0.184 | 0.032 | 0.316 | 1.229 | 0.598 | 0.437 | 1.64 |

- The talon pick moves with the play: 0.46 to 0.49 in the MLP's trial arm and 0.47 in the transformer's, against 0.71 to 1.08 in the base arm; its target is 0.45.
- Bidding, contract choice, king call and discard lie in their bands in all runs. Bidding, king call and discard are close to their targets. The contract choice is 0.028 to 0.048 against a target of 0.03; in two runs (MLP base of seed 16, transformer trial) it is 50 to 60% above the target, still in the band.
- Phases outside the band, mean over updates 401 to 500: 1.3 to 2.2 in the base arm, 0.0 to 0.3 in the trial arm. The count at update 500 is made from the entropies and checked against the count the trainer logged; the two agree in every run.

### Criterion 9: phases whose weight went under 0.01

In all four runs of the trial arm the weight of every one of the six phases went under 0.01, and the lowest value of each is the limit, 0.0010. That is the start of the run again: all weights fall to the limit in the first updates.

Which phases use the new room at the end of the run. Per phase over updates 401 to 500: mean weight; share of updates with the weight under 0.01; share of updates with the weight at the run's lower limit.

| Run | bidding | contract choice | king call | talon pick | discard | play |
|---|---|---|---|---|---|---|
| mlp-base-s14 | 0.0369; 0%; 0% | 0.0504; 0%; 14% | 0.0709; 0%; 0% | 0.0100; 0%; 99% | 0.0202; 0%; 19% | 0.0100; 0%; 100% |
| mlp-low-s14 | 0.0395; 0%; 0% | 0.0431; 26%; 12% | 0.0591; 0%; 0% | 0.0134; 62%; 17% | 0.0276; 0%; 0% | 0.0010; 100%; 98% |
| mlp-base-s15 | 0.0273; 0%; 28% | 0.1155; 0%; 36% | 0.1849; 0%; 0% | 0.0100; 0%; 100% | 0.0307; 0%; 0% | 0.0100; 0%; 100% |
| mlp-low-s15 | 0.0254; 6%; 0% | 0.0015; 100%; 35% | 0.1240; 0%; 0% | 0.0030; 100%; 29% | 0.0244; 0%; 0% | 0.0013; 100%; 35% |
| mlp-base-s16 | 0.0466; 0%; 0% | 0.0101; 0%; 91% | 0.1466; 0%; 0% | 0.0100; 0%; 100% | 0.0194; 0%; 0% | 0.0100; 0%; 100% |
| mlp-low-s16 | 0.0333; 0%; 0% | 0.0684; 3%; 0% | 0.0777; 0%; 0% | 0.0018; 100%; 40% | 0.0186; 0%; 0% | 0.0026; 100%; 0% |
| transformer-base-s14 | 0.0211; 0%; 1% | 0.1105; 0%; 19% | 0.0662; 0%; 0% | 0.0100; 0%; 100% | 0.0261; 0%; 12% | 0.0100; 0%; 100% |
| transformer-low-s14 | 0.0463; 0%; 0% | 0.2711; 10%; 0% | 0.1143; 0%; 0% | 0.0239; 42%; 0% | 0.0314; 0%; 0% | 0.0010; 100%; 100% |

- The play: under 0.01 in all 100 updates in all four trial runs.
- The talon pick: under 0.01 in 42% to 100% of them.
- The contract choice: under 0.01 in 3% to 100%; it swings between the two limits as in spec 014.
- The bidding: under 0.01 in 6% of the updates of one run. King call and discard: never at the end.

### Criterion 10: contracts and the declarer's side

At update 500, as the criterion asks. One update is about 575 deals (the last column), so a share of one update moves by a few points from update to update.

| Run | dve | ena | solo dve | solo ena | solo tri | tri | Declarer's side wins | Deals in that update |
|---|---|---|---|---|---|---|---|---|
| mlp-base-s14 | 53.3% | 44.9% | 0.0% | 0.0% | 0.0% | 1.8% | 65.4% | 563 |
| mlp-low-s14 | 44.1% | 55.7% | 0.2% | 0.0% | 0.0% | 0.0% | 63.3% | 578 |
| mlp-base-s15 | 3.7% | 96.2% | 0.2% | 0.0% | 0.0% | 0.0% | 60.3% | 572 |
| mlp-low-s15 | 14.1% | 84.0% | 0.0% | 0.0% | 0.0% | 1.9% | 62.7% | 568 |
| mlp-base-s16 | 23.6% | 76.4% | 0.0% | 0.0% | 0.0% | 0.0% | 64.2% | 573 |
| mlp-low-s16 | 35.8% | 64.2% | 0.0% | 0.0% | 0.0% | 0.0% | 66.1% | 572 |
| transformer-base-s14 | 0.9% | 99.1% | 0.0% | 0.0% | 0.0% | 0.0% | 58.3% | 564 |
| transformer-low-s14 | 0.0% | 99.3% | 0.0% | 0.2% | 0.4% | 0.2% | 56.6% | 567 |

The same as means over updates 401 to 500, with the same-seed replicate of the MLP's base arm:

| Run | dve | ena | solo dve | solo ena | solo tri | tri | Declarer's side wins | Deals per update, mean |
|---|---|---|---|---|---|---|---|---|
| mlp-base-s14 | 43.0% | 56.6% | 0.0% | 0.0% | 0.0% | 0.3% | 63.5% | 574 |
| mlp-low-s14 | 38.3% | 61.3% | 0.0% | 0.0% | 0.0% | 0.3% | 62.2% | 576 |
| mlp-base-s15 | 12.2% | 87.7% | 0.0% | 0.0% | 0.0% | 0.0% | 60.5% | 574 |
| mlp-low-s15 | 20.3% | 79.5% | 0.0% | 0.0% | 0.0% | 0.2% | 61.6% | 573 |
| mlp-base-s16 | 36.0% | 63.9% | 0.0% | 0.0% | 0.0% | 0.0% | 62.8% | 574 |
| mlp-low-s16 | 32.4% | 67.5% | 0.0% | 0.0% | 0.0% | 0.1% | 63.7% | 574 |
| transformer-base-s14 | 12.4% | 86.8% | 0.0% | 0.1% | 0.0% | 0.7% | 60.6% | 575 |
| transformer-low-s14 | 5.6% | 92.9% | 0.1% | 0.1% | 1.3% | 0.1% | 61.6% | 575 |
| mlp-base-s14, replicate (the run of spec 014 read to update 500) | 27.4% | 72.6% | 0.0% | 0.0% | 0.0% | 0.0% | 62.4% | 575 |

- No collapse: the declarer's side wins 60% to 64% of the deals over the last 100 updates, and no run bids one contract only.
- The MLP bids "ena" and "dve" in every run (dve 12% to 43%).
- The transformer bids almost only "ena" in both arms. Its trial run bid no "dve" at all in update 500 (0 of 567 deals) and 5.6% over the last 100 updates; its base arm 12.4%.
- The contract mix differs more between seeds, and between the two same-seed runs, than between the arms.

### Criterion 11: against random legal play

2,000 deals in four seatings, seed 2008, mean reward per deal with the 95% interval.

| Run | Mean reward per deal (95% interval) |
|---|---|
| mlp-base-s14 | +0.6068 (+0.5985 to +0.6152) |
| mlp-low-s14 | +0.6014 (+0.5931 to +0.6097) |
| mlp-base-s15 | +0.6008 (+0.5923 to +0.6093) |
| mlp-low-s15 | +0.6056 (+0.5973 to +0.6138) |
| mlp-base-s16 | +0.6024 (+0.5941 to +0.6108) |
| mlp-low-s16 | +0.6033 (+0.5952 to +0.6113) |
| transformer-base-s14 | +0.6031 (+0.5943 to +0.6118) |
| transformer-low-s14 | +0.5953 (+0.5872 to +0.6033) |

All eight lie between +0.595 and +0.607, and every interval overlaps every other one. As in spec 014, this number does not tell these checkpoints apart. The base arm of the transformer was evaluated again with the code after the review fix of spec 015; the file of spec 014 (+0.6044, +0.5961 to +0.6127) is kept in `before-review\` of its run folder.

### Criterion 13: the size of chance between runs

The three base checkpoints of the MLP against each other. 2,000 deals in four seatings, seed 5151, mean reward per deal of the single seat with the 95% interval.

| One seat | Three seats | Mean reward per deal (95% interval) |
|---|---|---|
| base, seed 14 | base, seed 15 | +0.0036 (-0.0085 to +0.0156) |
| base, seed 15 | base, seed 14 | -0.1027 (-0.1205 to -0.0848) |
| base, seed 14 | base, seed 16 | -0.0160 (-0.0282 to -0.0039) |
| base, seed 16 | base, seed 14 | -0.0733 (-0.0878 to -0.0589) |
| base, seed 15 | base, seed 16 | -0.0612 (-0.0759 to -0.0464) |
| base, seed 16 | base, seed 15 | -0.0179 (-0.0303 to -0.0055) |

The largest difference is 0.1027. In five of the six rows the single seat's interval lies below zero, in one it includes zero, in none it lies above.

### Criterion 12: trial against base

The same evaluation.

| Network, seed | One trial seat among three base seats | One base seat among three trial seats | Both intervals on the trial's side | Both on the base's side | Both differences larger than the largest of criterion 13 |
|---|---|---|---|---|---|
| MLP, seed 14 | -0.0652 (-0.0796 to -0.0509) | -0.0037 (-0.0148 to +0.0074) | no | no | no |
| MLP, seed 15 | -0.0198 (-0.0321 to -0.0075) | -0.0536 (-0.0688 to -0.0384) | no | no | no |
| MLP, seed 16 | -0.0033 (-0.0145 to +0.0080) | +0.0019 (-0.0097 to +0.0135) | no | no | no |
| Transformer, seed 14 | -0.0408 (-0.0546 to -0.0269) | -0.0254 (-0.0388 to -0.0119) | no | no | no |

In five of the eight comparisons the single seat's interval lies below zero, in three it includes zero, in none it lies above.

### Criterion 14: the reading by the rule

- **MLP: no difference shown.** In no seed do both intervals lie on the trial's side of zero, and in none on the base's side. No difference is larger than the largest of criterion 13 (0.1027).
- **Transformer: no difference shown.** One trial seat among three base seats: -0.0408, below zero. One base seat among three trial seats: -0.0254, also below zero. The two do not lie on the same arm's side.
- The condition "larger than the largest one of criterion 13" is read as belonging to the MLP's sentence of the rule. For the transformer the reading is the same either way.

### Criterion 15: recommendation

By the rule the trial is not ahead for the transformer, so **0.01 stays**. This spec changes no setting of a later run. The user decides.

### Addition, not asked by the criteria: the 14 comparisons with a second seed

Run on 2026-11-25 at 16:06 (card before: 0% busy, 233 MiB; 151 s), on the main session's word, because the checkpoints of flow A cannot be evaluated once the engine changes. Seed 5252; no chunk of it holds a training seed. The rule of criterion 14 was fixed for the first set and is applied to the first set only. The files of seed 5151 stay in place; those of seed 5252 are in `seed-5252\` of each run folder, and a copy of the first set is in `seed-5151\`.

| One seat | Three seats | Seed 5151 (the criterion's) | Seed 5252 (addition) |
|---|---|---|---|
| base, seed 14 | base, seed 15 | +0.0036 (-0.0085 to +0.0156) | -0.0090 (-0.0212 to +0.0032) |
| base, seed 15 | base, seed 14 | -0.1027 (-0.1205 to -0.0848) | -0.1109 (-0.1289 to -0.0930) |
| base, seed 14 | base, seed 16 | -0.0160 (-0.0282 to -0.0039) | -0.0167 (-0.0291 to -0.0043) |
| base, seed 16 | base, seed 14 | -0.0733 (-0.0878 to -0.0589) | -0.0716 (-0.0864 to -0.0569) |
| base, seed 15 | base, seed 16 | -0.0612 (-0.0759 to -0.0464) | -0.0597 (-0.0742 to -0.0453) |
| base, seed 16 | base, seed 15 | -0.0179 (-0.0303 to -0.0055) | -0.0129 (-0.0252 to -0.0006) |

| Network, seed | Direction | Seed 5151 (the criterion's) | Seed 5252 (addition) |
|---|---|---|---|
| MLP, seed 14 | one trial seat among three base seats | -0.0652 (-0.0796 to -0.0509) | -0.0533 (-0.0665 to -0.0400) |
| MLP, seed 14 | one base seat among three trial seats | -0.0037 (-0.0148 to +0.0074) | -0.0214 (-0.0330 to -0.0097) |
| MLP, seed 15 | one trial seat among three base seats | -0.0198 (-0.0321 to -0.0075) | -0.0151 (-0.0276 to -0.0026) |
| MLP, seed 15 | one base seat among three trial seats | -0.0536 (-0.0688 to -0.0384) | -0.0565 (-0.0717 to -0.0413) |
| MLP, seed 16 | one trial seat among three base seats | -0.0033 (-0.0145 to +0.0080) | -0.0054 (-0.0163 to +0.0056) |
| MLP, seed 16 | one base seat among three trial seats | +0.0019 (-0.0097 to +0.0135) | -0.0075 (-0.0188 to +0.0039) |
| Transformer, seed 14 | one trial seat among three base seats | -0.0408 (-0.0546 to -0.0269) | -0.0340 (-0.0480 to -0.0199) |
| Transformer, seed 14 | one base seat among three trial seats | -0.0254 (-0.0388 to -0.0119) | -0.0279 (-0.0408 to -0.0150) |

- **The second set would read the same:** no difference shown, for the MLP (no seed has both intervals on one arm's side) and for the transformer.
- The numbers repeat. In each of the 14 comparisons the intervals of the two seeds overlap. The largest difference between two base runs is 0.1027 with seed 5151 and 0.1109 with seed 5252, both for the same pair and direction.
- Of the 14 single seats, the interval lies below zero for 10 with seed 5151 and for 11 with seed 5252. It lies above zero for none.

### The single seat's handicap between two runs

**What the comparison assumes.** If run X is ahead of run Y, "one X among three Y" should be above zero and "one Y among three X" below zero, by about the same amount. The rule of criterion 14 is built on that.

**What is measured.** Two numbers per pair, taken deal by deal over the same 2,000 deals: half the sum of the two directions, which is what the single seat gets on average whichever of the two runs it comes from, and half the difference. Their intervals are paired over the deals (`trial_compare.paired`): the two directions play the same deals, and their per-deal values correlate at -0.11 to -0.40, so they are not independent. Both seeds.

| Pair (X against Y) | X among three Y, seed 5151 | Y among three X, seed 5151 | Half the sum, seed 5151 | Half the sum, seed 5252 | Half the difference, seed 5151 | Half the difference, seed 5252 | Share of "ena" over updates 401 to 500: X; Y |
|---|---|---|---|---|---|---|---|
| trial against base, MLP, seed 14 | -0.0652 (-0.0796 to -0.0509) | -0.0037 (-0.0148 to +0.0074) | -0.0345 (-0.0424 to -0.0265) | -0.0373 (-0.0448 to -0.0298) | -0.0308 (-0.0408 to -0.0207) | -0.0160 (-0.0259 to -0.0060) | 61.3%; 56.6% |
| trial against base, MLP, seed 15 | -0.0198 (-0.0321 to -0.0075) | -0.0536 (-0.0688 to -0.0384) | -0.0367 (-0.0448 to -0.0286) | -0.0358 (-0.0441 to -0.0275) | +0.0169 (+0.0057 to +0.0281) | +0.0207 (+0.0096 to +0.0318) | 79.5%; 87.7% |
| trial against base, MLP, seed 16 | -0.0033 (-0.0145 to +0.0080) | +0.0019 (-0.0097 to +0.0135) | -0.0007 (-0.0083 to +0.0069) | -0.0064 (-0.0139 to +0.0010) | -0.0026 (-0.0111 to +0.0059) | +0.0010 (-0.0073 to +0.0094) | 67.5%; 63.9% |
| trial against base, Transformer, seed 14 | -0.0408 (-0.0546 to -0.0269) | -0.0254 (-0.0388 to -0.0119) | -0.0331 (-0.0415 to -0.0247) | -0.0310 (-0.0392 to -0.0228) | -0.0077 (-0.0185 to +0.0031) | -0.0030 (-0.0137 to +0.0077) | 92.9%; 86.8% |
| base seed 14 against base seed 15, MLP | +0.0036 (-0.0085 to +0.0156) | -0.1027 (-0.1205 to -0.0848) | -0.0496 (-0.0583 to -0.0409) | -0.0599 (-0.0686 to -0.0513) | +0.0531 (+0.0406 to +0.0656) | +0.0510 (+0.0383 to +0.0637) | 56.6%; 87.7% |
| base seed 14 against base seed 16, MLP | -0.0160 (-0.0282 to -0.0039) | -0.0733 (-0.0878 to -0.0589) | -0.0447 (-0.0525 to -0.0369) | -0.0442 (-0.0521 to -0.0362) | +0.0286 (+0.0178 to +0.0395) | +0.0275 (+0.0164 to +0.0385) | 56.6%; 63.9% |
| base seed 15 against base seed 16, MLP | -0.0612 (-0.0759 to -0.0464) | -0.0179 (-0.0303 to -0.0055) | -0.0395 (-0.0476 to -0.0314) | -0.0363 (-0.0443 to -0.0284) | -0.0216 (-0.0326 to -0.0107) | -0.0234 (-0.0343 to -0.0126) | 87.7%; 63.9% |

- **Half the sum.** In six of the seven pairings it lies under zero with both seeds: -0.031 to -0.060 per deal, 1.6 to 3.0 points of the paper score. The seventh, trial against base of seed 16, shows none (-0.001 and -0.006, both intervals include zero).
- **It is not a loss in each direction.** Both intervals lie below zero in four of the six pairings with a handicap (seed 5151), five of seven with seed 5252. The seat of the stronger run loses little or nothing: +0.004 to -0.025. The seat of the weaker run loses 0.041 to 0.103.
- A checkpoint against itself gives zero (the controls of spec 015: +0.006 and -0.002). So this is not something the evaluation adds; it comes from the two runs being different.
- **Half the difference is a lead only under an assumption:** that the handicap is the same in both directions. Nothing here shows that it is. Read it as a summary of the two numbers.

**The one-against-three numbers of spec 015**, where both directions exist (evaluation code after the review fix, seed 5151, paired intervals):

| Pair (X against Y) | X among three Y | Y among three X | Half the sum | Half the difference |
|---|---|---|---|---|
| MLP update 2,000 against MLP update 200, one run | +0.0629 (+0.0513 to +0.0745) | -0.0741 (-0.0861 to -0.0620) | -0.0056 (-0.0134 to +0.0022) | +0.0685 (+0.0596 to +0.0773) |
| Transformer update 2,000 against transformer update 200, one run | +0.1205 (+0.1032 to +0.1378) | -0.0982 (-0.1094 to -0.0870) | +0.0112 (+0.0025 to +0.0198) | +0.1093 (+0.0976 to +0.1211) |
| Transformer update 2,000 against MLP update 2,000, two runs | +0.0166 (-0.0013 to +0.0345) | -0.0596 (-0.0709 to -0.0483) | -0.0215 (-0.0307 to -0.0123) | +0.0381 (+0.0263 to +0.0499) |

- MLP update 2,000 against update 200 of one run: half the sum -0.006, interval includes zero.
- Transformer update 2,000 against update 200 of one run: half the sum +0.011 (+0.003 to +0.020), a small bonus for the single seat, not a handicap.
- The transformer against the MLP, two runs: half the sum -0.022.

**What it means for the numbers so far.**

1. *Checkpoints of one run against each other* (ten of the twelve pairs of spec 015, the two controls among them). Where both directions exist the handicap is about zero, or a small bonus of 0.011 for the transformer. "Both networks were still improving at update 2,000" stands. The pairs measured in one direction only carry an unknown part of that kind: transformer 2,000 among three of 1,500 is +0.016; with a bonus of 0.011 like the one above it would be about +0.005. The MLP's +0.019 and +0.015, and the transformer's +0.051 against 1,000, are larger than that.
2. *The transformer against the MLP* (two of the twelve pairs). Spec 015 read "+0.017, not measurably ahead" and "-0.060, the MLP seat loses". Half the sum of the two is -0.022 and half the difference +0.038 (+0.026 to +0.050). If the handicap were the same in both directions, the transformer would be ahead by about 0.038. Here an equal handicap is least likely of all pairs: at update 2,000 the two runs play different games (transformer: 51.5% "ena", 47.9% "solo tri"; MLP: 84.1% "ena", 15.5% "dve", no solo), and the per-deal spread of the two directions differs (0.41 and 0.26).
3. *One run of each network cannot show that a network is ahead.* The only measure of chance between runs is the MLP base runs at update 500: 0.053, 0.029 and 0.022 apart by half the difference. That is MLP against MLP, at another update; it gives no size for two transformer runs or for update 2,000. So the finding touches the evidence that spec 015 quoted for the transformer, not D18: D18 was decided on 2026-11-09, before any run.
4. *Trial against base* (this spec): by half the difference the trial arm is behind in seed 14 (-0.031 and -0.016), ahead in seed 15 (+0.017 and +0.021), level in seed 16 (-0.003 and +0.001, both intervals include zero), and not shown for the transformer (-0.008, -0.019 to +0.003; -0.003 with seed 5252). The arms differ in both directions and by less than the seeds do. This agrees with "no difference shown".
5. *Any rule of the form "both intervals on one side of zero"* needs the weaker run's seat to lose and the stronger run's seat to gain. Where the stronger seat also loses, the rule cannot say "ahead" however many deals are played.

**Explanations, none measured.**

- *Off the training distribution.* Each run trains only against itself and its own past checkpoints (`past_share` 0.2). A lone seat among three seats of another run meets opponents and a partner it never trained with. Two checkpoints of one run are closer to each other's training, which would fit their missing handicap.
- *Shared habits.* Three seats of one run agree on things like which contract to bid or call; the fourth does not. The contract mix is not that habit: two checkpoints of one run differ more in contract mix than two runs do (the MLP of spec 014 bid 56% "dve" over updates 101 to 200 and 15.5% over 1,901 to 2,000) and show no handicap, and seed 14's two arms differ in "ena" share as little as seed 16's (4.7 and 3.6 points) while only seed 14 has a handicap.
- *For the part that differs between the directions* (half the difference): in all six pairs other than seed 16's, the run with the lower "ena" share over updates 401 to 500 has the higher half difference; for the transformer its interval includes zero. This is an observation over pairs that share runs, not a test.
- What none of these explains: why seed 16's two arms show no handicap. Nothing was built to test any of them.

### Not asked by the criteria: the run itself

**1. The lower limit lets the early dips go deeper.** Per phase over the 500 updates: lowest entropy; updates under the lower edge of the band (a quarter of the target); the last such update.

| Run | bidding | contract choice | king call | talon pick | discard | play |
|---|---|---|---|---|---|---|
| mlp-base-s14 | 0.025; 12; 27 | 0.007; 3; 247 | 0.124; 0; - | 0.433; 0; - | 0.455; 0; - | 0.434; 0; - |
| mlp-low-s14 | 0.012; 24; 40 | 0.003; 12; 102 | 0.013; 26; 96 | 0.164; 0; - | 0.398; 0; - | 0.247; 0; - |
| mlp-base-s15 | 0.023; 13; 29 | 0.003; 16; 498 | 0.054; 5; 74 | 0.182; 0; - | 0.475; 0; - | 0.362; 0; - |
| mlp-low-s15 | 0.010; 22; 38 | 0.002; 30; 307 | 0.029; 20; 88 | 0.135; 0; - | 0.378; 0; - | 0.217; 0; - |
| mlp-base-s16 | 0.026; 11; 27 | 0.003; 5; 96 | 0.052; 6; 94 | 0.303; 0; - | 0.447; 0; - | 0.408; 0; - |
| mlp-low-s16 | 0.014; 25; 47 | 0.005; 9; 114 | 0.025; 15; 103 | 0.165; 0; - | 0.392; 0; - | 0.218; 0; - |
| transformer-base-s14 | 0.012; 15; 31 | 0.004; 8; 409 | 0.121; 0; - | 0.338; 0; - | 0.478; 0; - | 0.509; 0; - |
| transformer-low-s14 | 0.006; 24; 38 | 0.002; 11; 450 | 0.041; 8; 62 | 0.185; 0; - | 0.465; 0; - | 0.351; 0; - |

- Bidding: lowest 0.006 to 0.014 in the trial arm against 0.012 to 0.026 in the base arm; 22 to 25 updates under the edge against 11 to 15. Over by update 47 in every run.
- King call: lowest 0.013 to 0.041 against 0.052 to 0.124; 8 to 26 updates under the edge against 0 to 6. Over by update 103.
- The contract choice dips under its edge later in the run too, in both arms, as in spec 014.
- The play and the talon pick never go under their lower edge in either arm.

**2. Other monitors over updates 401 to 500.**

| Run | Entropy of the play, decisions with a choice | Forced share of the play | Critic, explained variance | Card-location head, accuracy | Approximate KL |
|---|---|---|---|---|---|
| mlp-base-s14 | 0.640 | 0.293 | 0.619 | 0.3717 | 0.0023 |
| mlp-low-s14 | 0.377 | 0.293 | 0.642 | 0.3765 | 0.0023 |
| mlp-base-s15 | 0.594 | 0.292 | 0.639 | 0.3739 | 0.0025 |
| mlp-low-s15 | 0.349 | 0.291 | 0.633 | 0.3766 | 0.0021 |
| mlp-base-s16 | 0.669 | 0.293 | 0.640 | 0.3737 | 0.0021 |
| mlp-low-s16 | 0.332 | 0.292 | 0.642 | 0.3809 | 0.0020 |
| transformer-base-s14 | 0.736 | 0.294 | 0.612 | 0.3865 | 0.0077 |
| transformer-low-s14 | 0.516 | 0.294 | 0.644 | 0.3965 | 0.0070 |
| mlp-base-s14, replicate (the run of spec 014 read to update 500) | 0.618 | 0.293 | 0.624 | 0.3715 | 0.0024 |

- The entropy of the play over the decisions with a choice is 0.33 to 0.38 in the MLP's trial arm against 0.59 to 0.67 in the base arm. The share of forced decisions is 29% everywhere.
- The card-location head is a little more accurate in the trial arm: every trial value (0.3765, 0.3766, 0.3809) lies above every base value (0.3717, 0.3739, 0.3737) and the replicate's (0.3715); transformer 0.3965 against 0.3865. A difference of half to one point.
- The declarer's wins and the critic do not differ between the arms beyond what the seeds do.

### Readings: where I chose

1. **One config per seed** (criterion 2), seven files in all. The run name holds the arm and the seed (`trial-floor-low-s15`), so a run folder says what it is.
2. **Time limits:** 45 minutes for the MLP and 180 for the transformer, about three times and two and a half times what the runs took.
3. **The order:** per seed the base run and then the trial run, so that a change in the PC's state over the evening touches both arms alike.
4. **Disturbed** as written above: the desktop compositor is left out, any other program over 300 MiB counts.
5. **Criterion 11 for the base arm of the transformer** was run again with today's evaluation code, so that all eight numbers come from the same code. The file of spec 014 is kept.
6. **The scripts.** `tarok_train/trial_report.py` reads a run folder (criteria 7 to 10, the means over the last updates, the dips); `tarok_train/trial_compare.py` reads the two evaluation files of a pair (criteria 12 to 14, half the sum and half the difference). `smoke_report.py` prints the first and the last update only.

### What failed, and what did not go by the book

1. **Every new run is disturbed.** Nothing in this project can stop the render. The first version of these Results said the checkpoints do not depend on the load by the design of the trainer. That was wrong: GPU runs do not repeat even without load (the replicate above), so whether the load changed a checkpoint cannot be told from the files.
2. **The session was stopped by a usage limit** from about 21:00 on 24 November to 15:00 on 25 November, and again during the review fixes. The transformer run and its evaluations ran to their end without it; their logs, samples and files were read afterwards.
3. **One small test run during a training run:** at 20:16 I ran `tests/test_spec017.py` (8 s on the CPU) while the base run of seed 16 was going. The runs are not timed for a criterion.
4. **The second set of comparisons ran while the test suite was running on the CPU** (16:06 on 25 November). Evaluations are not timed, and their numbers do not depend on the load.

### Open issues

1. **The rule and the measure.** Where the stronger run's seat loses too, "both intervals on the trial's side" cannot be met. For the user, before a next trial of this kind: (a) keep the rule; a result then needs a clear lead, recommended as long as nothing better is tested; (b) read half the difference of the two directions with its paired interval of about ±0.01, knowing that it is a lead only if the handicap is equal in both directions; (c) a seating of two against two, which needs a change of `evaluate.py`. This also matters for the arena of spec 011. Nothing was built for it here.
2. **Does a firmer play help?** Not shown at 500 updates. The MLP's play reached its target and the trial arm is behind, ahead and level in the three seeds by half the difference. One run per arm of the transformer cannot settle it either.
3. **The transformer's play stopped above its target** with 0.001: 0.364 over the last 100 updates, flat, with the weight at the limit. These runs do not show whether a lower limit or more updates would move it.
4. **The early dips** (item 1 of "the run itself") are the price of one limit for all phases. A limit per phase is out of scope here (it is a code change).
5. **Nothing more must be measured before the engine changes.** The 22 evaluation files of the criteria are complete and from one code, and the 14 comparisons exist with a second seed. What could no longer be measured after the rebuild, if it were wanted: more seeds of these comparisons, or the comparisons of spec 015 in the direction that is missing (three earlier checkpoints as the single seat among three of update 2,000). I see no need for either.
6. **The evidence for the transformer over the MLP** in spec 015 rests on one run of each. For the user: whether a comparison with several seeds of each network is wanted, on the flow the full run will use. D18 itself is not touched.
7. **Disk:** the seven new run folders hold five checkpoints each.

### Review fixes (2026-11-25, after the D72 step 4 review)

The separate review recomputed every number of the tables from the files and found them right. It found fourteen points in the prose, the intervals of the handicap section and the process. I checked each in the files before changing it; all fourteen hold. No run was repeated and no evaluation was run again. The tests for the new code were written first and failed on the code as it was (11 failing tests). These fixes were made with `claude-opus-5-5` (D152).

| Point | What it led to |
|---|---|
| 1. "Both directions are negative" | Wrong as written. Both intervals lie below zero in four of the six pairs with a handicap (seed 5151). The handicap section and the summary now say: half the sum is 0.03 to 0.06 under zero; the stronger run's seat loses +0.004 to -0.025, the weaker run's 0.041 to 0.103. |
| 2. The transformer against the MLP | Half the difference is a lead only if the handicap is equal in both directions; said, with why it is least likely for this pair (other contract mix, other spread). The claim "no larger than chance" is dropped: 0.038 is larger than two of the three MLP gaps, and those are MLP against MLP at update 500. Kept: one run of each cannot show that a network is ahead. The finding touches the evidence quoted in spec 015, not D18 (decided 2026-11-09, before any run). |
| 3. The intervals of half the sum and half the difference | Now paired over the deals (`trial_compare.paired`). The per-deal values of the two directions correlate at -0.11 to -0.40. Half the difference ±0.008 to ±0.013, half the sum ±0.007 to ±0.009; transformer against MLP +0.038 (+0.026 to +0.050). No sign changes, no reading changes. |
| 4. "The checkpoints do not depend on the load" | Wrong; withdrawn. GPU runs do not repeat (spec 007). The run of spec 014 is a same-seed replicate of the MLP's base arm of seed 14: equal to update 15, then apart (ena 72.6% against 56.6% at the end). Its consequences are under "The runs"; its numbers are in the tables of criteria 8 and 10 and of the monitors. Nothing new was run for it. |
| 5. The "habits" explanation | Rewritten as explanations, none measured. Contract mix is ruled out as the mechanism by the files; the training distribution (each run plays only itself and its past) is named; the observation about the lower "ena" share and half the difference is given as an observation over pairs that share runs. |
| 6. A speed of a disturbed run was used | The conclusion about the update step is dropped. No speed of these runs is used. |
| 7. "Two checkpoints of one run show no handicap" | The transformer's pair has a small bonus (+0.011, +0.003 to +0.020). The one-direction numbers of spec 015 get the caveat: transformer 2,000 among 1,500 is +0.016, about +0.005 with such a bonus. |
| 8. "Every run bids ena and dve" | Wrong for the transformer's trial run at update 500 (no "dve" in 567 deals). Contract shares are now also given over the last 100 updates, with the deals per update (about 575). |
| 9. Card-location accuracy | It differs between the arms, a little: every trial value above every base value, by half to one point. Reported. |
| 10. "Does what it was meant to do to the entropy" | Overstated for the transformer. The summary now says: the MLP reaches the target; the transformer stops 46% above it, flat over the last 100 updates, with the weight at 0.001 in the last 300. Open issue 3 no longer says that more updates would show where it settles. |
| 11. "Level for the transformer" | Now "not shown" (-0.008, paired -0.019 to +0.003). |
| 12. Code of the tables | All numbers now come from two committed modules with tests: `tarok_train/trial_report.py` (extended) and `tarok_train/trial_compare.py` (new). The untracked `make_tables.py` only puts them into tables, but for three columns of the first table that read the sampler's files. Said under "Criteria". |
| 13. `trial_report.py`, small | The logged weight is the one set after the update: the report says so, and `play_first_under_old_limit` gives the update after which it was set and the one that used it. `--at` has a test. `fake_run` writes each tag once per update. The phases outside the band are counted from the entropies and checked against the logged count (they agree in all runs). |
| 14. "At their targets" | Loose. Now: in their bands; the contract choice 50 to 60% above its target in two runs. |
| Criterion 6 | Argued now under "The runs": what changed in `train.py` between the two commits, and that it cannot be checked bit by bit on the GPU (point 4). |

Where I differ from the review:

- **Point 1, the wording.** The review says the weaker run's seat loses "0.05 to 0.11". By the files it is 0.041 to 0.103 (seed 5151), the lower end being the transformer's trial seat. I use the files' numbers.
- **Point 5, the observation on "ena".** It holds in six of the seven pairs, not all: seed 16's pair has a half difference of -0.003 with an interval that includes zero. I give it that way.
- **Point 12.** The columns read from the sampler's files stay in the untracked script. The sampler is a tool of this PC that writes process names; it does not belong in the training code.

New tests in `tests/test_spec017.py` (23 in all, from 14): five for `trial_report.py` (the window of the last updates, the dips, `--at`, the band counted two ways, what a logged weight is) and five for `trial_compare.py` (the paired numbers, refusing two files of different deals, the side of zero, the rule of criterion 14, reading a pair of run folders with a second set in a subfolder).
