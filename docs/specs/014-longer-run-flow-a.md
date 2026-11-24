# 014 — Longer run on deal flow A

Status: **approved** (2026-11-23) by the user's instruction "go for a longer run" (D138). The user gave no settings: every setting below is Claude's choice and open to correction. Executed on 2026-11-23 and 24 in the `train-env` session; committed on the user's go-ahead of 2026-11-24 (D139). **Done** once the workflows are green on that commit.

Tool and model: Claude Code 2.1.306, `claude-fable-5-2`.

## Goal

Train both networks ten times longer than the smoke run of spec 008, on the same rules, and see what the longer run does: to the result against random play, to the bidding, to the entropies, to speed and memory.

**It is a run on deal flow A only**: six normal contracts, silent bonuses, no announcements, no kontra, no klop, berač or valat as a bid. A model from it does not play full tarok. It is not the first full run of the build plan (spec 012), which needs flows B and C.

## In scope

- Two run configs: `src/train/configs/long-a-mlp.toml` and `long-a-transformer.toml`. Each is the smoke config of spec 008 with the settings of the table below changed, and the origin of every value in the file.
- The two runs on this PC with the GPU free, the MLP first.
- Evaluation of five checkpoints of each run against random legal play (`evaluate.py` of spec 008).
- The numbers of the criteria in this spec's Results; the run log in `docs/progress.md` (D79, written by the main session).

## Out of scope

Any change of the code. Released weights and a `MODEL.md` (the user decides that separately). Evaluation of one checkpoint against another: `evaluate.py` cannot do it yet (open point C). Tuning.

## Settings

| Setting | Value | From |
|---|---|---|
| Updates per run | 2,000 | Claude's choice: ten times the smoke run, about 60 million trained decisions per run |
| Estimated duration | MLP about 1 hour, transformer about 5.2 hours | from spec 008: 1.76 s and 9.41 s per update |
| Time limit | MLP 120 minutes, transformer 480 minutes | Claude's choice: the estimate plus a margin |
| Checkpoint every | 100 updates | Claude's choice: a stop loses at most 16 minutes |
| Seed | 14 | Claude's choice: another seed than the smoke run's 8 |
| Run name | `long-a` | |
| Everything else | as `smoke-a-*.toml` | D124, D127, D136: 512 tables, 2,048 seat-deals per update, value-head weight 0.1, policy scale 0.3, the entropy controller on with the targets of D136 and the mean over all decisions (C16 is open) |

## Acceptance criteria

**The runs**
1. Each run reaches 2,000 updates. A run that stops early (time limit, out of memory, the PC) is resumed from its last checkpoint with `--resume`; every stop is named in the run log with its cause.
2. No illegal action, no logged number that is not finite, every deal's rewards add up to zero, every trained decision was played under the current weights, over the whole run (the checks of spec 008, criteria 2 to 5).
3. The runs start only with the GPU free (under 10% busy, under 700 MiB in use) and nothing else running on the PC. The GPU state before each run is in the run log.

**Measurements** (no thresholds except where said)
4. Against random legal play, 2,000 deals in four seatings, the same deals for every checkpoint: the checkpoints of updates 200, 500, 1,000, 1,500 and 2,000 of each run. Mean reward per deal and the 95% interval. The interval of the last checkpoint is wholly above zero. 100 records of the last checkpoint replay to the same entries.
5. Monitors at updates 1, 200, 500, 1,000, 1,500 and 2,000: entropy per phase (over all decisions and over decisions with a choice), the share of forced decisions, the controller's weight per phase, the contract shares, the share of deals the declarer's side wins, the accuracy of the two auxiliary heads next to the uniform guess, the critic's explained variance.
6. A phase whose entropy is under the lower edge of its band at the last update is listed as a collapse. Every phase whose weight reached the controller's upper limit is listed.
7. Speed end to end per run: decisions played and trained per second, over the whole run and per quarter of the run. The time split.
8. GPU memory per run: the tensors' peak, the allocator's peak and the whole card (nvidia-smi) at updates 200, 1,000 and 2,000. Growth during the run is reported.
9. Length of a seat's deal: decisions and tokens, mean and largest. The largest token count is at most 128.

**Quality gates**
10. `pytest -m "not slow"` passes with the two new configs in the folder.
11. Every setting of the two configs is in the file with its origin. The run log names the commit, the configs, the duration, the state of the tree and what failed.
12. No claim about playing strength beyond "against random legal play" is made from these runs.

## Checks

Criteria 1 to 3, 7 to 9: `summary.json`, `run-info.json` and the TensorBoard logs of each run, read with `smoke_report.py` or a few lines of Python; the numbers go into Results. Criteria 4 and 5: `python -m tarok_train.evaluate` and the logs. Criterion 10: pytest output. Criteria 11 and 12: by reading.

## Files expected to change

New: `src/train/configs/long-a-mlp.toml`, `src/train/configs/long-a-transformer.toml`. Changed: this spec (Results); `src/train/README.md` if the commands are listed there. `docs/progress.md` by the main session. Run folders under `src/train/runs/`, untracked.

## Commits

One commit by the main session after the user's read of the diff: `train(configs): the longer run on flow A (spec 014)`, with the two configs and the Results.

## Open points for the user

- **A. Length.** 2,000 updates per run is Claude's choice. (a) keep it, recommended: about 6 hours in all, one evening; (b) 5,000 updates, about 16 hours; (c) another number.
- **B. Networks.** (a) both, the MLP first, recommended: the MLP costs one hour and is the control (D18); (b) the transformer only.
- **C. Checkpoint against checkpoint.** Against random play both networks are at +0.60 after 200 updates, so that number will say little about further progress. (a) add an option to `evaluate.py` that seats another checkpoint as the opponents, as its own small change with tests and a review, recommended; (b) wait for the arena of spec 011.
- **D. Weights.** Nothing is released from this run. Say so if a checkpoint should go into `models/` with a `MODEL.md` (D79), for example for a first look in the app.

## Results

Executed on 2026-11-23 and 24 in the `train-env` session. Verified by the main session, reviewed by a separate pass, committed on the user's go-ahead (D139).

### Summary

- Both runs reached 2,000 updates in one part. No stop, no resume, no out of memory.
- Against random legal play the last checkpoints are at +0.6190 (MLP) and +0.6170 (transformer). Both intervals are wholly above zero.
- From update 200 to 2,000 the result against random legal play rose by +0.024 (MLP) and +0.016 (transformer). Update 2,000 is not measurably above update 1,000 or 1,500 in either run. Every pair of checkpoints is in the table under criterion 4.
- No phase is collapsed at the last update. The weight of the contract choice reached the upper limit in both runs.
- Four things did not go as planned. They are under "What failed".

| Criterion | Result |
|---|---|
| 1 | Met. 2,000 of 2,000 updates in each run, no stop. |
| 2 | Met in both runs. Numbers below. |
| 3 | The GPU was free before each run. "Nothing else running" did not hold during the MLP run: another program used the card at times. See "What failed", item 1. |
| 4 | Met. Table below. 100 of 100 records of each last checkpoint replay to the same entries. |
| 5 | Reported. Tables below. |
| 6 | Reported. No collapse at the last update. Contract choice reached the upper limit in both runs. |
| 7 | Reported. |
| 8 | Reported. The transformer's allocator peak is above the card's size. See "What failed", item 3. |
| 9 | Met. Largest token count 66, limit 128. |
| 10 | Met. 153 passed, 6 skipped, 6 deselected in 203 s, with the two configs in the folder. The log is from 2026-11-23 21:09, before the runs. The six skipped are the GPU tests: that call ran with CUDA hidden, because the game held the card. No test reads the two long configs; `validate_config` passed at the start of each run. |
| 11 | Met for the configs. The run log is below. The trainer's own GPU reading is empty in `run-info.json` and `summary.json`. See "What failed", item 2. |
| 12 | Held. Every result below is against random legal play, on flow A. |

### Run log (criterion 11)

| | MLP | Transformer |
|---|---|---|
| Config | `src/train/configs/long-a-mlp.toml` | `src/train/configs/long-a-transformer.toml` |
| Run folder under `src/train/runs` | `long-a-mlp-20261124-000454` | `long-a-transformer-20261124-011623` |
| Commit | `9351c6b` | `67c425b` |
| Tree | 3 uncommitted paths: the two configs (new) and `src/train/README.md` (changed) | the same 3 paths |
| Module | engine 0.2.0, fingerprint `64a6157fb05f43c4`, equal to the files on disk | the same |
| GPU before the run (nvidia-smi, whole card) | 0% busy, 536 MiB | 0% busy, 372 MiB |
| Started | 2026-11-24 00:04:55 | 2026-11-24 01:16:25 |
| Duration | 3,643.6 s (60.7 minutes) | 24,883.5 s (414.7 minutes) |
| Seconds per update | 1.82 (estimate 1.76) | 12.44 (estimate 9.41) |
| Updates | 2,000 of 2,000 | 2,000 of 2,000 |
| Stops and resumes | none | none |
| Time limit | 120 minutes, not reached | 480 minutes, not reached |
| GPU after the run (nvidia-smi, whole card) | 9% busy, 3,177 MiB | 75% busy, 187 MiB |
| Size of the run folder | 1.6 GB | 4.2 GB |

Notes on the run log:

- The commit differs between the two runs. The main session committed `67c425b` in between. `git diff --stat 9351c6b 67c425b` names six files, all under `docs/`. The code and the module's fingerprint are the same for both runs.
- The wait for the GPU: the pipeline checked nvidia-smi every five minutes from 21:09. The game held the card until 23:54 (34 checks, 5% to 97% busy, 1,297 to 1,675 MiB). The checks of 23:59 and 00:04 were free (0%, 679 MiB and 0%, 536 MiB). Before the transformer run the checks of 01:11 and 01:16 were free (0%, 372 MiB both).
- The GPU numbers of this log come from the pipeline's own nvidia-smi calls, not from the trainer. See "What failed", item 2.
- The 3,177 MiB after the MLP run is not the run: its allocator held 438 MiB at most. See "What failed", item 1.
- The runs were started by a script that waits, checks the fingerprint, runs, resumes on a stop, evaluates and writes the reports. The script, its log, one nvidia-smi sample per minute and the printed output of every step are in `src/train/runs/long-a-pipeline` (untracked).
- A checkpoint every 100 updates: measured, 100 updates took 20.7 minutes on average in the transformer run (slowest 28.7) and 3.0 minutes in the MLP run (slowest 3.8). The 16 minutes of the settings table were the estimate for the transformer. A stop by the time limit loses nothing: the trainer saves a checkpoint before it stops.
- Evaluations: seed 2008 for all, engine seeds 2008 to 2011, the run's seed is 14. Each evaluation writes `evaluation-NNNNNN.json` named by the checkpoint's update, so no output overwrote another. The records are in `records-002000` of each run folder.

### What failed

1. **Another program used the card during the MLP run.** The run's allocator held at most 438 MiB. In 48 of the 60 one-minute samples the whole card shows 910 to 1,007 MiB. In one it shows 1,152 MiB. In the other 11 it shows 3,118 to 5,634 MiB. The same holds right after the run (3,177 MiB) and at 01:06 (3,855 MiB). At 01:11 the card was back at 372 MiB. What the program was is not known: only numbers were recorded. 123 updates took over 1.5 times the median time; they took 337 s of 3,602 s and lie in the same stretches (most between updates 488 and 1,000). Effect: the speed numbers of the MLP run are a little low. The load should not change what is trained. That is inferred from the design (seeded generators; the clock enters only the time limit and the logged times), not measured. In the transformer run 41 of 413 samples lie 252 to 415 MiB above the median, all before update 520, and no other sample lies more than 50 MiB above it; 47 of its 50 slow updates lie there too. Whether that was the same program or the run itself is not known.
2. **The trainer's own GPU reading is empty.** `gpu_before` and `gpu_after` are `{}` in `run-info.json` and `summary.json` of both runs, and the report prints "None MiB". Cause, checked: `gpu_snapshot` in `bench_model.py` reads four whole numbers from nvidia-smi. nvidia-smi now prints the temperature with decimals ("49.60"), the conversion fails, and the function returns an empty reading. In the smoke run of 2026-11-12 the temperature was a whole number. No code was changed (out of scope). The GPU state before and after each run is in the run log above, from the pipeline.
3. **The transformer's allocator peak is above the card's size.** `max_memory_reserved` is 6,484 MiB; the card has 6,144 MiB. No out-of-memory error was raised. The whole card (nvidia-smi) has a median of 5,531 MiB: exactly that in 147 of 413 samples, within 10 MiB of it in 302. In the smoke run the allocator's peak was 5,156 MiB with the same settings and another seed. Measured: the run took 12.44 s per update, the smoke run 9.41 s. Inferred, not measured: the Windows driver placed the part above the card's size in shared system memory, which is slower. The current (not peak) allocator size is not logged. The time split shows a slower update step from update 21 to the end; see "Where the transformer's time went".
4. **The transformer run took 6.9 hours, not the estimated 5.2.** It stayed under the time limit of 480 minutes. The overrun against the estimate (2,000 times 9.41 s) is about 6,060 s. The 50 slow updates add 732 s over the median, about 12% of the overrun. The rest is a steady slowdown; see "Where the transformer's time went".

### Where the transformer's time went (to items 3 and 4 of "What failed")

From the logged time split. Median seconds per update, next to the smoke run of spec 008 (same settings, 200 updates).

| Updates | Whole update, this run | Whole update, smoke run | Update step, this run | Update step, smoke run | Acting, this run | Acting, smoke run |
|---|---|---|---|---|---|---|
| 1 to 20 | 7.61 | 7.70 | 4.13 | 4.41 | 3.01 | 2.79 |
| 21 to 40 | 9.20 | 9.00 | 4.76 | 4.45 | 3.62 | 3.89 |
| 41 to 80 | 10.17 | 9.19 | 5.37 | 4.44 | 4.19 | 4.15 |
| 81 to 150 | 11.92 | 9.70 | 6.18 | 4.48 | 5.10 | 4.66 |
| 151 to 200 | 12.17 | 9.97 | 6.52 | 4.48 | 5.17 | 4.90 |
| 201 to 250 | 12.34 | | 6.55 | | 5.31 | |
| 251 to 500 | 13.10 | | 6.70 | | 5.93 | |
| 501 to 1,000 | 12.51 | | 6.59 | | 5.71 | |
| 1,001 to 1,500 | 12.44 | | 6.55 | | 5.72 | |
| 1,501 to 2,000 | 12.43 | | 6.48 | | 5.72 | |

- The update step is as fast as in the smoke run for the first 20 updates. It slows from update 21 on, the update where the allocator's peak passes 6,144 MiB, and stays at 6.2 to 6.7 s from update 81 to the end. In the smoke run it is 4.41 to 4.48 s throughout. This fits the shared-memory explanation of item 3. The cause stays inferred: nothing in the logs shows where the memory was.
- Acting rose in the smoke run too, from 2.8 s to 4.9 s over its 200 updates. So the estimate of 9.41 s per update was a mean over a rising curve; the smoke run's last 50 updates have a median of 9.97 s. In this run acting went on to 5.7 s and is flat from about update 500. Why acting rises was not examined.
- Against the smoke run's last 50 updates, the steady state of this run has about 2.0 s more in the update step (6.5 against 4.5) and about 0.8 s more in acting (5.7 against 4.9).
- The 50 slow updates (over 1.5 times the median) took 1,354 s, 732 s more than 50 median updates. That is about 12% of the overrun of 6,060 s.
- The MLP run has no steady slowdown. Its median is 1.66 s per update, against 1.74 s in the smoke run. Its 123 slow updates add 132 s over the median; its overrun against the estimate (2,000 times 1.76 s) is 124 s.

### Criterion 2: checks over the whole run

| Check | MLP | Transformer |
|---|---|---|
| Illegal actions refused by the engine | 0 | 0 |
| Logged numbers that are not finite | 0 of 188,002 | 0 of 188,002 |
| Largest sum of a deal's four rewards | 1.3e-07 | 1.3e-07 |
| Oldest trained decision, in updates | 0 | 0 |
| Largest difference between the stored log-probability of an action and the one the current weights give, first minibatch | 1.0e-05 | 1.8e-05 |

### Criterion 4: against random legal play

2,000 deals, each in four seatings, the same deals for every checkpoint. Mean reward per deal with the 95% interval. Reward 1 is 50 points of the paper score. The agent samples from its policy.

| Checkpoint of update | MLP | Transformer |
|---|---|---|
| 200 | +0.5954 (+0.5871 to +0.6037) | +0.6007 (+0.5924 to +0.6090) |
| 500 | +0.6062 (+0.5975 to +0.6148) | +0.6044 (+0.5961 to +0.6127) |
| 1,000 | +0.6125 (+0.6040 to +0.6211) | +0.6108 (+0.6024 to +0.6191) |
| 1,500 | +0.6138 (+0.6057 to +0.6219) | +0.6220 (+0.6136 to +0.6304) |
| 2,000 | +0.6190 (+0.6108 to +0.6273) | +0.6170 (+0.6087 to +0.6252) |

The interval of the last checkpoint is wholly above zero in both runs. Records of the last checkpoint: 100 exported, 100 replay to the same entries, in both runs.

Not asked by the criterion: the difference per deal between two checkpoints on the same deals, with its 95% interval. Computed from the per-deal values in the evaluation files. All pairs are listed. A star marks an interval that excludes zero.

| Difference | MLP | Transformer |
|---|---|---|
| Update 500 minus 200 | +0.0108 (+0.0037 to +0.0179) * | +0.0037 (-0.0038 to +0.0112) |
| Update 1,000 minus 200 | +0.0172 (+0.0096 to +0.0247) * | +0.0100 (+0.0000 to +0.0200) * |
| Update 1,500 minus 200 | +0.0184 (+0.0116 to +0.0252) * | +0.0213 (+0.0128 to +0.0298) * |
| Update 2,000 minus 200 | +0.0237 (+0.0163 to +0.0310) * | +0.0163 (+0.0062 to +0.0264) * |
| Update 1,000 minus 500 | +0.0064 (-0.0010 to +0.0138) | +0.0063 (-0.0037 to +0.0164) |
| Update 1,500 minus 500 | +0.0076 (+0.0004 to +0.0149) * | +0.0176 (+0.0091 to +0.0261) * |
| Update 2,000 minus 500 | +0.0129 (+0.0056 to +0.0202) * | +0.0126 (+0.0026 to +0.0225) * |
| Update 1,500 minus 1,000 | +0.0013 (-0.0061 to +0.0086) | +0.0112 (+0.0014 to +0.0211) * |
| Update 2,000 minus 1,000 | +0.0065 (-0.0006 to +0.0136) | +0.0062 (-0.0016 to +0.0141) |
| Update 2,000 minus 1,500 | +0.0052 (-0.0017 to +0.0122) | -0.0050 (-0.0149 to +0.0049) |

| Checkpoint of update | Transformer minus MLP |
|---|---|
| 200 | +0.0053 (-0.0028 to +0.0135) |
| 500 | -0.0017 (-0.0096 to +0.0062) |
| 1,000 | -0.0018 (-0.0121 to +0.0086) |
| 1,500 | +0.0082 (+0.0006 to +0.0158) * |
| 2,000 | -0.0021 (-0.0122 to +0.0081) |

Reading:

- Both runs gained from update 200: every later checkpoint but one is above it (the transformer's update 500 is not).
- Update 2,000 is not measurably above update 1,000 or 1,500 in either run. Those four intervals include zero.
- Two pairs after update 1,000 exclude zero: the transformer's update 1,500 minus 1,000, and transformer minus MLP at update 1,500. Both are borderline (lower edges +0.0014 and +0.0006). The transformer's update 1,000 minus 200 has its lower edge at zero to four decimals.
- With 25 intervals in the two tables, one or two borderline ones are expected by chance alone. A borderline interval is weak evidence here.
- At update 2,000 no difference between the two networks is measured.
- Ten times the training moved this number by about one point of the paper score per deal. It says little about further progress (open point C).

By the main session, not recomputed here: a second evaluation on other deals (seed 4141, copies of the checkpoints) gives MLP +0.5874 at update 200 and +0.6144 at update 2,000; transformer +0.5958 and +0.6159.

### Criterion 5: monitors

The auxiliary heads: accuracy next to what a uniform guess over the places still possible gets, both over the same cases (as criterion 14 of spec 008). The guess is not one in three: for each case it counts the seats or places the state still leaves possible.

Each column is one single update. Single updates swing; the statements under the tables use means over 100 updates.

MLP:

| Monitor at update | 1 | 200 | 500 | 1,000 | 1,500 | 2,000 |
|---|---|---|---|---|---|---|
| Entropy, bidding | 0.871 | 0.196 | 0.192 | 0.199 | 0.224 | 0.205 |
| Entropy with a choice, bidding | 1.207 | 0.196 | 0.192 | 0.199 | 0.224 | 0.205 |
| Forced share, bidding | 27.8% | 0.0% | 0.0% | 0.0% | 0.0% | 0.0% |
| Weight, bidding | 0.0100 | 0.0169 | 0.0495 | 0.0130 | 0.0151 | 0.0509 |
| Entropy, contract choice | 0.111 | 0.035 | 0.031 | 0.031 | 0.015 | 0.027 |
| Entropy with a choice, contract choice | 0.877 | 0.035 | 0.031 | 0.031 | 0.015 | 0.027 |
| Forced share, contract choice | 87.3% | 0.0% | 0.0% | 0.0% | 0.0% | 0.0% |
| Weight, contract choice | 0.0100 | 0.0987 | 0.0181 | 0.0141 | 0.0265 | 0.0244 |
| Entropy, king call | 1.386 | 0.373 | 0.340 | 0.321 | 0.303 | 0.435 |
| Entropy with a choice, king call | 1.386 | 0.373 | 0.340 | 0.321 | 0.303 | 0.435 |
| Forced share, king call | 0.0% | 0.0% | 0.0% | 0.0% | 0.0% | 0.0% |
| Weight, king call | 0.0100 | 0.0644 | 0.1287 | 0.0981 | 0.1291 | 0.1012 |
| Entropy, talon pick | 1.743 | 0.838 | 1.112 | 1.138 | 0.830 | 0.824 |
| Entropy with a choice, talon pick | 1.743 | 0.838 | 1.112 | 1.138 | 0.830 | 0.824 |
| Forced share, talon pick | 0.0% | 0.0% | 0.0% | 0.0% | 0.0% | 0.0% |
| Weight, talon pick | 0.0100 | 0.0100 | 0.0100 | 0.0100 | 0.0100 | 0.0100 |
| Entropy, discard | 2.423 | 0.659 | 0.641 | 0.603 | 0.592 | 0.607 |
| Entropy with a choice, discard | 2.423 | 0.659 | 0.641 | 0.603 | 0.592 | 0.607 |
| Forced share, discard | 0.0% | 0.0% | 0.0% | 0.0% | 0.0% | 0.0% |
| Weight, discard | 0.0100 | 0.0100 | 0.0126 | 0.0333 | 0.0103 | 0.0152 |
| Entropy, play | 0.951 | 0.518 | 0.444 | 0.493 | 0.503 | 0.457 |
| Entropy with a choice, play | 1.333 | 0.735 | 0.630 | 0.696 | 0.711 | 0.643 |
| Forced share, play | 28.6% | 29.5% | 29.5% | 29.1% | 29.3% | 28.9% |
| Weight, play | 0.0100 | 0.0100 | 0.0100 | 0.0100 | 0.0100 | 0.0100 |
| Contract dve | 0.0% | 40.5% | 31.8% | 23.5% | 17.4% | 13.3% |
| Contract ena | 0.4% | 56.9% | 68.2% | 76.4% | 81.8% | 86.4% |
| Contract solo dve | 5.5% | 0.2% | 0.0% | 0.0% | 0.0% | 0.0% |
| Contract solo ena | 93.2% | 0.2% | 0.0% | 0.0% | 0.0% | 0.0% |
| Contract solo tri | 0.8% | 0.4% | 0.0% | 0.0% | 0.0% | 0.0% |
| Contract tri | 0.2% | 1.9% | 0.0% | 0.2% | 0.9% | 0.4% |
| Three passes before forehand | 0.2% | 3.3% | 0.2% | 2.6% | 1.1% | 0.4% |
| Declarer's side wins | 4.9% | 65.7% | 62.3% | 64.8% | 60.4% | 60.6% |
| Partner head, accuracy | 86.0% | 93.8% | 94.3% | 91.5% | 88.2% | 88.5% |
| Partner, uniform guess over the seats still possible | 73.5% | 90.1% | 88.2% | 80.8% | 78.0% | 75.5% |
| Partner head, partner not public | 90.5% | 71.5% | 79.0% | 81.0% | 77.1% | 80.0% |
| Card-location head, accuracy | 26.6% | 35.9% | 37.5% | 39.1% | 39.2% | 39.1% |
| Card location, uniform guess over the places still possible | 30.3% | 30.8% | 30.6% | 30.4% | 30.3% | 30.5% |
| Critic, explained variance | -0.001 | 0.606 | 0.621 | 0.670 | 0.654 | 0.670 |
| Approximate KL | 0.0011 | 0.0014 | 0.0021 | 0.0034 | 0.0026 | 0.0030 |
| Clip fraction | 0.012 | 0.019 | 0.031 | 0.038 | 0.026 | 0.038 |

Transformer:

| Monitor at update | 1 | 200 | 500 | 1,000 | 1,500 | 2,000 |
|---|---|---|---|---|---|---|
| Entropy, bidding | 0.843 | 0.247 | 0.204 | 0.176 | 0.221 | 0.217 |
| Entropy with a choice, bidding | 1.192 | 0.247 | 0.204 | 0.176 | 0.221 | 0.217 |
| Forced share, bidding | 29.3% | 0.0% | 0.0% | 0.0% | 0.0% | 0.0% |
| Weight, bidding | 0.0100 | 0.0149 | 0.0324 | 0.0209 | 0.0295 | 0.0268 |
| Entropy, contract choice | 0.098 | 0.047 | 0.008 | 0.014 | 0.026 | 0.052 |
| Entropy with a choice, contract choice | 0.867 | 0.047 | 0.008 | 0.014 | 0.026 | 0.052 |
| Forced share, contract choice | 88.7% | 0.0% | 0.0% | 0.0% | 0.0% | 0.0% |
| Weight, contract choice | 0.0100 | 0.0176 | 0.1388 | 0.2893 | 0.1400 | 0.1621 |
| Entropy, king call | 1.360 | 0.287 | 0.247 | 0.334 | 0.405 | 0.372 |
| Entropy with a choice, king call | 1.360 | 0.287 | 0.247 | 0.334 | 0.405 | 0.372 |
| Forced share, king call | 0.0% | 0.0% | 0.0% | 0.0% | 0.0% | 0.0% |
| Weight, king call | 0.0100 | 0.0534 | 0.0868 | 0.1640 | 0.0512 | 0.1048 |
| Entropy, talon pick | 1.734 | 0.564 | 0.840 | 0.451 | 0.549 | 0.477 |
| Entropy with a choice, talon pick | 1.734 | 0.564 | 0.840 | 0.451 | 0.549 | 0.477 |
| Forced share, talon pick | 0.0% | 0.0% | 0.0% | 0.0% | 0.0% | 0.0% |
| Weight, talon pick | 0.0100 | 0.0100 | 0.0100 | 0.0364 | 0.1123 | 0.0989 |
| Entropy, discard | 2.406 | 0.554 | 0.632 | 0.743 | 0.577 | 0.640 |
| Entropy with a choice, discard | 2.406 | 0.554 | 0.632 | 0.743 | 0.577 | 0.640 |
| Forced share, discard | 0.0% | 0.0% | 0.0% | 0.0% | 0.0% | 0.0% |
| Weight, discard | 0.0100 | 0.0122 | 0.0452 | 0.0100 | 0.0105 | 0.0100 |
| Entropy, play | 0.943 | 0.568 | 0.515 | 0.525 | 0.502 | 0.508 |
| Entropy with a choice, play | 1.321 | 0.804 | 0.729 | 0.734 | 0.703 | 0.707 |
| Forced share, play | 28.6% | 29.3% | 29.4% | 28.5% | 28.5% | 28.1% |
| Weight, play | 0.0100 | 0.0100 | 0.0100 | 0.0100 | 0.0100 | 0.0100 |
| Contract dve | 0.0% | 32.9% | 0.9% | 0.0% | 0.0% | 0.0% |
| Contract ena | 0.4% | 61.8% | 99.1% | 64.2% | 60.2% | 52.9% |
| Contract solo dve | 4.3% | 0.0% | 0.0% | 0.0% | 0.0% | 0.9% |
| Contract solo ena | 93.9% | 0.0% | 0.0% | 0.0% | 0.0% | 0.0% |
| Contract solo tri | 1.2% | 0.0% | 0.0% | 35.6% | 39.6% | 46.3% |
| Contract tri | 0.2% | 5.3% | 0.0% | 0.2% | 0.2% | 0.0% |
| Three passes before forehand | 0.2% | 5.5% | 0.0% | 0.4% | 0.4% | 0.3% |
| Declarer's side wins | 4.7% | 63.5% | 58.3% | 67.0% | 66.1% | 64.2% |
| Partner head, accuracy | 88.0% | 92.8% | 94.7% | 93.8% | 92.8% | 92.2% |
| Partner, uniform guess over the seats still possible | 81.2% | 86.8% | 83.0% | 80.8% | 83.7% | 78.4% |
| Partner head, partner not public | 79.1% | 75.5% | 86.6% | 86.0% | 80.9% | 84.7% |
| Card-location head, accuracy | 26.1% | 37.5% | 39.2% | 40.2% | 40.1% | 40.0% |
| Card location, uniform guess over the places still possible | 30.3% | 30.8% | 30.4% | 30.6% | 30.7% | 30.8% |
| Critic, explained variance | -0.001 | 0.568 | 0.640 | 0.593 | 0.637 | 0.662 |
| Approximate KL | 0.0068 | 0.0056 | 0.0078 | 0.0089 | 0.0094 | 0.0097 |
| Clip fraction | 0.123 | 0.062 | 0.080 | 0.090 | 0.086 | 0.090 |

What the logs show, without a threshold. Means over 100 updates unless said otherwise.

- Contracts, MLP. "ena" is the main contract: 84.1% over updates 1,901 to 2,000. The share of "dve" swings and has no steady trend: 21.0%, 43.9%, 10.3%, 31.3% and 15.5% over updates 301 to 400, 801 to 900, 1,001 to 1,100, 1,501 to 1,600 and 1,901 to 2,000.
- Contracts, transformer. "dve" is gone from about update 600 (1.5% over updates 501 to 600, 0.2% over 601 to 700). From about update 600 it bids "solo tri": 16.6% over updates 601 to 700, 33.4% over 801 to 900, 47.9% over 1,901 to 2,000, next to "ena" at 51.5%.
- The two networks ended with different mixes. Against random legal play no difference between them is measured at update 2,000.
- The declarer's side wins 62.1% (MLP) and 65.7% (transformer) of the deals over the last 100 updates.
- The card-location head, over updates 901 to 1,000, 1,401 to 1,500 and 1,901 to 2,000: MLP 38.8%, 39.3%, 39.5%, still rising a little; transformer 40.1%, 40.2%, 40.1%, flat. The guess is 30.5% and 30.9% over the last 100 updates.
- The critic's explained variance over the last 100 updates is 0.672 (MLP) and 0.644 (transformer).
- The transformer's approximate KL rose from 0.0038 over updates 1 to 100 to about 0.009 from update 500 on (0.0089 over the last 100); its clip fraction is 8.3% there. The MLP is at 0.0035 and 4.0% over the last 100.

### Criterion 6: collapses and weights at the upper limit

Collapse at the last update (entropy under the lower edge of the band): **none**, in both runs.

Weight at the upper limit (1.0): **contract choice, in both runs**. MLP: 5 updates, 125 to 129. Transformer: 51 updates between 514 and 607. No other phase reached it.

| Phase | Target | Band | MLP: mean entropy, updates 1,001 to 2,000 | MLP: updates under the lower edge | MLP: largest weight | Transformer: mean entropy, updates 1,001 to 2,000 | Transformer: updates under the lower edge | Transformer: largest weight |
|---|---|---|---|---|---|---|---|---|
| Bidding | 0.20 | 0.050 to 0.350 | 0.199 | 12 (updates 16 to 27) | 0.308 | 0.200 | 15 (updates 17 to 31) | 0.597 |
| Contract choice | 0.03 | 0.0075 to 0.0525 | 0.032 | 11 (between 113 and 949) | 1.000 | 0.030 | 51 (between 309 and 1,692) | 1.000 |
| King call | 0.35 | 0.0875 to 0.6125 | 0.349 | 0 | 0.289 | 0.351 | 0 | 0.288 |
| Talon pick | 0.45 | 0.1125 to 0.7875 | 0.861 | 0 | 0.0105 | 0.448 | 0 | 0.253 |
| Discard | 0.60 | 0.150 to 1.050 | 0.623 | 0 | 0.052 | 0.616 | 0 | 0.057 |
| Play | 0.25 | 0.0625 to 0.4375 | 0.459 | 0 | 0.0100 | 0.506 | 0 | 0.0100 |

- Lowest entropies of the run: bidding 0.025 (MLP, update 20) and 0.012 (transformer, update 27); contract choice 0.003 (MLP, update 125) and 0.002 (transformer, update 585). The controller brought both back each time.
- The contract choice is the phase the controller works hardest on. Its entropy was under the lower edge in short runs of updates late in the run: 945, 946, 947 and 949 (MLP); 1,688 to 1,692 (transformer).
- Above the upper edge at the last update: play in both runs (0.457 and 0.508), talon pick in the MLP run (0.824). This is not a collapse. The weight of play sat at the lower limit 0.01 in all 2,000 updates of both runs, and the entropy stayed above the upper edge in 1,767 (MLP) and 2,000 (transformer) updates. The controller cannot lower an entropy further than the lower limit of the weight allows. The same holds for the MLP's talon pick (weight at the lower limit in 1,998 updates).
- 29% of the play decisions are forced. Over the decisions with a choice the play entropy is 0.64 (MLP) and 0.71 (transformer). This belongs to C16.

### Criterion 7: speed end to end

| | MLP | Transformer |
|---|---|---|
| Decisions played, whole run | 123,330,048 | 123,130,880 |
| Decisions trained on, whole run | 59,340,082 | 59,240,041 |
| Played per second, whole run | 33,848 | 4,948 |
| Trained per second, whole run | 16,286 | 2,381 |
| Share of played decisions trained on (all seats count as played) | 48.1% | 48.1% |
| The same against the current network's seats only | 53.4% | 53.4% |
| Updates 1 to 500: played and trained per second | 34,171 and 16,580 | 4,625 and 2,240 |
| Updates 501 to 1,000 | 31,218 and 14,979 | 4,973 and 2,390 |
| Updates 1,001 to 1,500 | 37,716 and 18,104 | 5,121 and 2,458 |
| Updates 1,501 to 2,000 | 34,474 and 16,532 | 5,141 and 2,466 |
| Time split: environment, acting, bookkeeping, update | 2.7%, 26.1%, 32.8%, 37.9% (together 99.4%) | 0.3%, 46.7%, 4.4%, 48.6% (together 99.9%) |
| Seconds per update: median and largest | 1.66 and 3.8 | 12.43 and 80.3 |
| Smoke run of spec 008, played per second | 34,743 | 6,506 |

The quarters count the time inside the updates (3,602 s and 24,842 s); the whole-run lines count the whole wall time. The MLP is 3% slower than in the smoke run, the transformer 24% slower. The MLP's second quarter is its slowest; it holds most of the stretches of item 1 of "What failed". The transformer's first quarter is its slowest.

### Criterion 8: GPU memory

The tensors' peak is `max_memory_allocated`, the allocator's peak is `max_memory_reserved`; both are peaks since the start of the run and cannot fall. The whole card is the nvidia-smi sample nearest to the update (one sample per minute, all programs).

| Update | MLP: tensors' peak | MLP: allocator's peak | MLP: whole card | Transformer: tensors' peak | Transformer: allocator's peak | Transformer: whole card |
|---|---|---|---|---|---|---|
| 200 | 299 MiB | 438 MiB | 921 MiB | 4,364 MiB | 6,484 MiB | 5,521 MiB |
| 1,000 | 299 MiB | 438 MiB | 969 MiB | 4,364 MiB | 6,484 MiB | 5,529 MiB |
| 2,000 | 299 MiB | 438 MiB | 1,152 MiB | 4,364 MiB | 6,484 MiB | 5,531 MiB |

Growth:

- MLP: the tensors' peak went from 225 MiB at update 1 to 299 MiB at update 13, the allocator's from 266 to 438 MiB. No new peak in the 1,987 updates after that.
- Transformer: the tensors' peak went from 2,878 MiB at update 1 to 4,364 MiB at update 32, the allocator's from 3,762 to 6,484 MiB. It stood at 6,144 MiB, the card's size, from update 8 and passed it at update 21. No new peak in the 1,968 updates after update 32.
- Whole card during the transformer run: 5,489 to 5,946 MiB, median 5,531 MiB, 413 samples. Before the run the card held 372 MiB of other programs.
- Whole card during the MLP run: see item 1 of "What failed". The sample for update 2,000 was taken 39 s before the last update.
- No growth over the length of the run is seen: both peaks are reached in the first 32 updates.

### Criterion 9: length of a seat's deal

| | MLP | Transformer |
|---|---|---|
| Decisions: mean and largest | 14.3 and 20 | 14.3 and 20 |
| Tokens: mean and largest | 60.6 and 66 | 60.1 and 66 |

The largest token count is 66; the limit is 128.

### Open issues

1. **The GPU reading of the trainer.** `gpu_snapshot` fails on a temperature with decimals. A small fix with a test, as its own change. Until then `run-info.json` has no GPU state.
2. **The transformer's memory.** The allocator's peak is above the card's size and the run is 24% slower than the smoke run. The update step slows at the update where the peak passes the card's size. For the user, before the full run (spec 012): (a) measure first: log the allocator's current size and repeat 50 updates, recommended, because the cause is inferred and not measured; (b) lower a setting of the transformer's update so the allocator stays under the card's size; (c) leave it.
3. **What ran on the card between 00:05 and 01:06.** If the MLP's speed numbers matter, the run can be repeated in one hour. Not recommended: at most 337 s of 3,602 s are affected and the training is not.
4. **The entropy of play stays above its band.** Target 0.25, measured 0.46 and 0.51, weight at the lower limit in all 2,000 updates. The MLP's talon pick is alike: weight at the lower limit in 1,998 of 2,000 updates. A question for the user together with C16: is a phase above the upper edge a finding, or is the band only a guard against collapse.
5. **The contract choice.** Its weight reached the upper limit in both runs and its entropy still dips under the lower edge late in the run. The target 0.03 (D136) is held on average (0.032 and 0.030).
6. **Open point C now has numbers.** Against random legal play update 2,000 is not measurably above update 1,000 or 1,500 in either run, and at update 2,000 no difference between the networks is measured. Two borderline pairs exclude zero (criterion 4). Whether the networks still improve needs checkpoint against checkpoint.
7. **Durations for planning.** 1.82 s per update for the MLP and 12.44 s for the transformer on this card with these settings.
8. **Disk.** The two run folders take 5.8 GB (20 checkpoints each). They are untracked.

### Review fixes (2026-11-24, after the D72 step 4 review)

The main session's review recomputed and confirmed the tables, and found 16 items to fix. Each number below was recomputed from the run files before it was written. No run was repeated and no code was changed.

| Item | What was wrong | Fix |
|---|---|---|
| 1 | README: "five to six hours for the transformer" | The measured durations with the date: 60.7 and 414.7 minutes. |
| 2 | Overclaim in three places: "from update 1,000 on, no gain is measured" and "nothing between the two networks" | All 25 pairs are in the tables of criterion 4. The summary, the reading and open issue 6 now say what holds, name the two borderline pairs that exclude zero, and say why a borderline interval is weak evidence. |
| 3 | Summary said three things failed; the list has four | "Four". |
| 4 | Run log: the diff was taken from `f4c08c3`, not from the MLP run's commit | `git diff --stat 9351c6b 67c425b`: six files, all under `docs/`. |
| 5 | Criterion 5: the rows "uniform guess" read like one in three | Rows named as in `smoke_report.py`; lead sentence as in spec 008. |
| 6 | "5,531 MiB for most of the run" | Median 5,531 MiB: exactly that in 147 of 413 samples, within 10 MiB in 302. |
| 7 | The cut at 300 MiB left out four samples | 41 of 413 samples lie 252 to 415 MiB above the median, all before update 520. Three of the four are near updates 29 to 41, one near update 515. |
| 8 | Items 3 and 4 of "What failed" did not use the time split | New section "Where the transformer's time went". |
| 9 | Three statements read from six single updates | Rewritten with means over 100 updates: the transformer bids "solo tri" from about update 600; the MLP's "dve" swings; the MLP's card-location head still rose a little. The other statements of that list use means too. |
| 10 | Criterion 6: "single updates" | Short runs: 945, 946, 947, 949 (MLP); 1,688 to 1,692 (transformer). |
| 11 | Configs: "a run that hits it is resumed" | The trainer saves a checkpoint and stops with exit code 3; going on needs `--resume`. |
| 12 | Configs: "a stop loses at most 16 minutes" | The measured figures in both configs and in the run log notes. The settings table of this spec still holds the estimate. |
| 13 | Open issue 4: talon pick "the same" | 1,998 of 2,000 updates. |
| 14 | Criterion 10: no word on when and how pytest ran | Date of the log, why six tests were skipped, that no test reads the long configs. |
| 15 | "The load does not change what is trained" stood as a fact | Marked as inferred from the design, not measured. |
| 16 | README: the resume read as exercised | These runs did not need a resume; it is covered only by the small test on the CPU. |

Differences to the reviewer's numbers after recomputing: the update step from update 81 on is 6.2 to 6.7 s by my blocks (the review: 6.0 to 6.7); acting adds about 0.8 s against the smoke run's last 50 updates (the review: 0.9); the overrun is about 6,060 s (the review: 6,070); acting is at 5.9 s over updates 251 to 500 and at 5.7 s from update 501 on (the review: 5.7 s from update 250 on); of the four samples of item 7, three are in the first minutes and one is near update 515 (the review: the first minutes). The reviewer's other numbers came out the same.
