# Progress log

One entry per session (D71). Newest first. Measurements are numbers that were actually measured; estimates are marked as such.

## Entry format

```
## YYYY-MM-DD — session N (tool, model)
**Done:** what was finished.
**Measurements:** numbers measured this session, with how.
**Broken:** what does not work or was dropped.
**Next:** the next step.
```

## 2026-11-24 — spec 015 verified; its review pass; spec 009 at `6bce5d8` (session 3, Claude Code 2.1.306, claude-fable-5-2; ai-log row 8)

**Done:** the `train-env` session reported spec 015 executed. The main session read the numbers of criteria 7 and 8 from the run files (`src/train/runs/spec015/c7-smoke-limit5000-transformer`, not tracked): median 8.347 s per update over the updates 101 to 200 (smoke run of spec 008: 9.818), update step 4.086 s (4.477), allocator's peak 4,910 MiB (5,156), no illegal action, +0.6015 (+0.5931 to +0.6099) against random legal play (smoke run: +0.6049, +0.5968 to +0.6130). The not-slow suite with the GPU, run by the main session: 204 passed, 1 failed; the failure is `test_c20` of spec 006 under load from the review agent's own test run, and it passes alone. A separate agent reviewed the uncommitted diff; its CPU run of the not-slow suite: 194 passed, 11 skipped.
**Review findings, sent to the `train-env` session; nothing is committed before they are fixed:**
- A defect in the evaluation with a checkpoint as opponent: the sampling noise of both networks is seeded anew, with the same seed, in every seating and chunk. For a policy that reads the cards weakly the deal values are then not independent and the 95% interval is too narrow. The reviewer measured the recipe of `test_c15` on 20 seeds: the control's interval excludes zero in 8 of 20. In the files of the twelve pairs and of all trained checkpoints the effect is within ±0.03 correlation, so those intervals stand; for the untrained network against random play (spec 008) the standard error was understated 1.3 to 1.4 times. The seeding of the agent was there before spec 015.
- No test has the two checkpoints of different kinds or model configs; two wrong changes survive. The code is right by the reviewer's hand check.
- Evaluation files of the same pair with another seed, or of two opponents with one run name, overwrite each other.
- Lesser: the limit is set after a resumed checkpoint is on the card; a limit of 0 does not lift an earlier limit in one process; a limit equal to the card's size may be refused; `nan` passes as a temperature; a wrong path ends in a traceback.
**Not covered by the review:** nothing ran on the GPU; the measured numbers of the Results were not derived again by the reviewer (the main session read those of criteria 7 and 8).
**Spec 009:** the branch `spec-009` is at `6bce5d8`: D147, D148 and the tests the review asked for; 233 engine tests, 64 planted wrong rules caught, the `engine` workflow green (session's report and the workflow list). The new code gets a review pass with stage (d). Its question 4 is Batch 19, item 1.
**Broken or open:** both sessions and the review agent were stopped once more by the usage limit and resumed.
**Next:** the fixes of spec 015, then its commits; then spec 017. The user's answers to Batch 19.

## 2026-11-24 — Batches 17 and 18 answered; spec 018 approved (session 3, Claude Code 2.1.306, claude-fable-5-2; ai-log row 8)

**Done:** the user answered Batch 17 (D148) and Batch 18 (D149) with a for every item. Spec 018 (first runs on flow B: monitors of the round, a silent random opponent, two runs of 200 updates, a rule fixed beforehand for when the flood of announced valats counts as ended) is drafted and approved. The three more cases of the public partner and the guard by engine version (criterion 53) went to the `engine-core` session for spec 009.
**Broken or open:** nothing of specs 009 and 015 is verified or merged yet. The timed 200-update run of spec 015 started at about 13:40.
**Next, in the order of the GPU work:** spec 015 (report, verification, review pass, commits), spec 017, the rebuild of the binding and the merge of spec 009, spec 018, spec 016.

## 2026-11-24 — review pass over spec 009 (separate agent; ordered and read by the main session; ai-log row 8)

**Done:** a separate agent reviewed `719a968..5f198bc` of the branch `spec-009` in an export of its own. Result: no rule defect against the spec in criteria 1 to 44, 50 and 51. It ran `cargo test` (231 passed), `fmt` and `clippy`; computed the examples of criteria 30 and 31 by hand; ran seven scoring cases of its own with hand-computed results (all match); planted 16 wrong rules. The findings went to the `engine-core` session.
**Findings:**
- Tests, hidden partner: a wrong rule that makes the true partner public on a non-declarer's re on a bonus fails no named test, only a coverage counter. The no-leak test changes the partner in 50 of 9,352 cases.
- Tests, scoring: an announced king or pagat played early, and a talon card next to an opponent's announced trula or kralji, are held only by the second scorer on random deals. The second scorer has the same shape as `score.rs`.
- The bound of 187 actions per deal is argued, not walked by a test.
- A checkpoint of the engine 0.2.0 passes the trainer's guard on 0.3.0 (Batch 17, item 2).
- With uniform random legal actions a valat is announced in 19,806 of 20,000 deals; the mean entry of the declarer is 4,462 points (Batch 17, item 3).
- Three more cases in which the sides follow from what was said (Batch 17, item 1).
**Not covered by the review:** Python, the binding build, the GPU, the workflows; pagat.com was not read again; 16 wrong rules, not the 30 of criterion 49 (the session's own run has 52).
**Broken or open:** the review agent's `cargo test` ran while the `train-env` session had a timed run going. The engine session now reads the card before every build.
**Next:** the engine session closes the weak tests and builds the fifth case of D147. The user's answers to Batch 17.

## 2026-11-24 — reports on specs 009 and 015; Batches 15 and 16 answered (session 3, Claude Code 2.1.306, claude-fable-5-2; ai-log row 8)

**Done:** the `engine-core` session reported spec 009 built as far as Rust goes on the branch `spec-009` (last commit `dd012d9`). Checked by the main session from the branch: a test exists for each of the criteria 1 to 44, 50 and 51; the `engine` workflow is green on the branch tip; no `unsafe`; the version is 0.3.0. From the session's Results, not checked again: 231 engine tests, 52 of 52 planted wrong rules caught, 87.5 decisions per random deal (56.8 in flow A), 35% of them in the round. The user answered Batches 15 and 16 with a for every item (D146, D147): specs 016 and 017 are approved.
**Spec 015, from the session's interim notes, not verified:** the twelve comparisons ran from committed code and agree with the scratch numbers. With a limit of 5,200 MiB on the allocator nothing spills into shared memory and the update step stays at 4.1 to 4.3 s (long run: 5.2 to 5.8 s). A trial with 5,000 MiB was running at 13:12.
**Broken or open:**
- The review agent's `cargo test` ran at 13:12 to 13:20, at the start of that trial. Its first updates are disturbed in timing. The main session runs no build during a timed run from now on.
- Spec 009: criteria 45, 46 and the Python side of 52 need the module build. The review pass over its code is not back.
- Flow B and rewards: a lost announced valat at mort is a reward of −160. Random play announces valat in about 40% of deals. For the training spec of flow B.
**Next:** the review findings to the `engine-core` session, with the fifth case of D147. Spec 015: report, verification, review pass, commits. Then spec 017, then the rebuild and the merge of spec 009, then spec 016.

## 2026-11-24 — after the usage limit: sessions restarted; specs 016 and 017 drafted (session 3, Claude Code 2.1.306, claude-fable-5-2; ai-log row 8)

**Done:** both side sessions had stopped at the usage limit; both were started again after its reset. A review pass over the engine code of spec 009 (commits `719a968..5f198bc` of the branch `spec-009`) was ordered from a separate agent. Specs 016 and 017 drafted from D142 and asked as Batch 15.
**State of the side sessions, from their notes, not verified:**
- Spec 009: stages (a), (b), (c), (e) and the property tests of (d) are on the branch `spec-009`, seven commits, 231 engine tests. The wrong-rule run stood at 41 of 52 and the Results section was not in the spec.
- Spec 015: part A done; part C done on the CPU, its twelve comparisons queued for the card; part B: the excess of the transformer's allocator sits in shared GPU memory and starts at about 5,400 MiB of allocator size, at update 3. About 2 GB of the allocator is cache. Another program of the user's (Blender) took about 500 MiB of the card from update 29 of that measurement; four updates then took 24 to 503 s.
**Found while drafting (checked in `train.py`):** the trained deals are not a fair sample of the played deals. A deal is stored only when it starts and ends inside one collection, so short deals are stored more often. Not measured; criterion 17 of spec 016 measures it.
**Broken or open:** after spec 009 the engine plays the round of announcements in every deal. There is no switch back to flow A. Runs after the merge are not comparable with the runs of specs 008 and 014. This sets the order of spec 017 (Batch 15, item 5).
**Next:** the reports of both sessions; verification, review pass, commits (D144). The user's answers to Batch 15.

## 2026-11-24 — Batches 13 and 14 answered; spec 009 approved and handed out; the repo public (session 3, Claude Code 2.1.306, claude-fable-5-2; ai-log row 8)

**Done:** the user answered Batch 13 (D141) and Batch 14 (D142), approved spec 009 (D143), dropped the question before every commit (D144) and ordered the repo public (D145). Spec 009 carries the answers: criterion 17 is now free play (B13-3 b, against the recommendation), and criteria 50 to 52 add the hold flag for aligned updates. The repo was made public with the GitHub CLI after a check for secrets. Spec 009 went to the `engine-core` session on the branch `spec-009` in its own worktree. CI is green on `bb917ce`; spec 014 is done (`7206f9d`).
**Measurements:** none.
**Broken or open:**
- Spec 015 is still in execution in the `train-env` session.
- The binding module cannot be rebuilt for spec 009 before the GPU work of spec 015 ends; stage (d) of spec 009 waits.
- Open with the user: C13, C16, the open points A, B and D of spec 014.
**Next:** the reports of both sessions; verification and review of each; commits without a question (D144); merge of `spec-009`.

## 2026-11-24 — spec 014 committed; analysis of the run; spec 015 handed out (session 3, Claude Code 2.1.306, claude-fable-5-2; ai-log row 8)

**Done:** spec 014 committed on the user's go-ahead (D139) as `e1a325f`, notes as `bb917ce`, pushed. The user ordered an analysis of the training, proposals, fixes of the problems found and to go on (D140). Analysis by the main session: one checkpoint in one seat against another checkpoint in the three other seats, with a scratch script built on `evaluate.py`. Spec 015 written from it and handed to the `train-env` session: the GPU reading, the transformer's memory, a checkpoint as opponent. The improvements that are not fixes are Batch 14.
**Measurements** (scratch script, 2,000 deals in four seatings, seeds 5151 and 5252, mean reward per deal of the single seat with the 95% interval; flow A):
- Control, a checkpoint against itself: MLP +0.002 (−0.007 to +0.012); transformer −0.002 (−0.011 to +0.007).
- Update 2,000 against its own update 200: MLP +0.059 (+0.048 to +0.071); transformer +0.118 (+0.102 to +0.135).
- Update 2,000 against its own update 1,500: MLP +0.012 (+0.001 to +0.023); transformer +0.025 (+0.015 to +0.036). Both were still improving at the end. Against random legal play this was not visible.
- MLP 2,000 among three transformer 2,000 seats: −0.063 (−0.074 to −0.052). Transformer 2,000 among three MLP 2,000 seats: +0.010 (−0.008 to +0.027).
**Broken or open:**
- These numbers come from a script that is not in the repo. Criterion 16 of spec 015 repeats them from committed code.
- Batch 13 (announcements and kontra) and the criteria of spec 009 are not answered. Flow B cannot start before that.
- Batch 14 is asked: the dropped half of the acting, the entropy of play, the length of the full run.
**Next:** the report of the `train-env` session on spec 015; verification, review pass, the user's read of the diff. The user's answers to Batches 13 and 14.

## 2026-11-24 — spec 014, the longer run on flow A: run log (session `train-env`, claude-fable-5-2; verified by the main session, ai-log rows 8 and 8a)

**This is a run on deal flow A only**: six normal contracts, no announcements, no kontra, no klop. The opponent in the evaluation is random legal play. It says nothing more about playing strength. No weights are released.

**Done:** both networks trained for 2,000 updates, ten times the smoke run of spec 008, with the same settings and seed 14. Five checkpoints of each run were evaluated on the same deals. The numbers are in the Results of `docs/specs/014-longer-run-flow-a.md`.
**Run log (D79).** Engine 0.2.0, source fingerprint `64a6157fb05f43c4`. 512 tables, 2,048 seat-deals per update, a checkpoint every 100 updates. Tree at both starts: three uncommitted paths (the two configs and `src/train/README.md`). Folders in `src/train/runs/`, untracked.

| | MLP | Transformer |
|---|---|---|
| Config | `long-a-mlp.toml` | `long-a-transformer.toml` |
| Run folder | `long-a-mlp-20261124-000454` | `long-a-transformer-20261124-011623` |
| Commit | `9351c6b` | `67c425b` (docs only between the two) |
| GPU before (whole card) | 0% busy, 536 MiB | 0% busy, 372 MiB |
| Updates, exit code | 2,000 of 2,000, 0 | 2,000 of 2,000, 0 |
| Duration | 3,643.6 s (60.7 min) | 24,883.5 s (414.7 min; estimate 312) |
| Stops and resumes | none | none |
| What failed | another program used the card at times | the allocator's peak is above the card's size; 24% slower than the smoke run |

**Measurements** (from each run's `summary.json`, `evaluation-*.json` and the TensorBoard files; recomputed by the main session from the files):
- Loop: 0 illegal actions, 0 of 188,002 logged numbers not finite, largest sum of a deal's four rewards 1.3e-07, oldest trained decision 0 updates, in both runs.
- Against random legal play, 2,000 deals in four seatings, seed 2008, checkpoints 200, 500, 1,000, 1,500, 2,000. MLP: +0.595, +0.606, +0.613, +0.614, +0.619 (last interval +0.611 to +0.627). Transformer: +0.601, +0.604, +0.611, +0.622, +0.617 (last interval +0.609 to +0.625). Reward 1 is 50 points of the paper score. 100 of 100 exported records of each last checkpoint replay to the same entries.
- Paired on the same deals, update 2,000 minus 200: MLP +0.024 (+0.016 to +0.031), transformer +0.016 (+0.006 to +0.026). Update 2,000 minus 1,000: +0.007 (−0.001 to +0.014) and +0.006 (−0.002 to +0.014). Transformer minus MLP at 2,000: −0.002 (−0.012 to +0.008). Ten times the training moved this number by about one point of the paper score per deal. Update 2,000 is not measurably above update 1,000 or 1,500 in either run. Two of the 25 pairs after update 1,000 exclude zero by a small margin (transformer 1,500 minus 1,000: +0.011, +0.001 to +0.021; transformer minus MLP at 1,500: +0.008, +0.001 to +0.016).
- The same checkpoints evaluated again by the main session on other deals (seed 4141, copies in a scratch folder): MLP +0.587 at update 200 and +0.614 at 2,000; transformer +0.596 and +0.616.
- Speed end to end: MLP 33,848 decisions played per second and 16,286 trained (smoke run 34,743); transformer 4,948 played and 2,381 trained (smoke run 6,506). 48.1% of the played decisions are trained on. Seconds per update: 1.82 and 12.44.
- GPU memory, transformer: tensors' peak 4,364 MiB, allocator's peak 6,484 MiB against a card of 6,144 MiB, whole card 5,531 MiB (median of 413 samples). Both peaks are reached by update 32; no growth after that. MLP: 299 and 438 MiB.
- Seat-deal length: 14.3 decisions on average, 20 at most; 66 tokens at most (limit 128).
- Contracts at the last update: MLP ena 86.4%, dve 13.3%; transformer ena 52.9%, solo tri 46.3%. The declarer's side wins 60.6% and 64.2%. Two different mixes with the same result against random legal play.
- Entropy: no phase is under its band at the last update. The bidding dipped under it in updates 16 to 27 (MLP) and 17 to 31 (transformer) and came back. The weight of the contract choice reached the upper limit 1.0 in both runs (MLP 5 updates, transformer 51). Play stayed above its band for the whole run with its weight at the lower limit (0.46 and 0.51 against a target of 0.25).
- Heads at the last update: card location 39.1% and 40.0% (uniform guess 30.5% and 30.8%), the transformer's flat since update 1,000. Critic's explained variance 0.67 and 0.66.
- Tests with the two configs in the folder: 153 passed, 6 skipped (GPU), 6 deselected (slow), run by the session before the runs.
**Review pass (D72 step 4):** a separate subagent read the diff and recomputed the parts the main session had not. It found no fault in the runs. Two defects in the text: the README gave five to six hours for the transformer (measured 6.9), and the Results said that nothing is gained after update 1,000 although two pairs of checkpoints exclude zero. Twelve lesser points: counts and cuts in the GPU samples, statements read from single updates, two config comments, the name of the uniform-guess rows, the base commit of a diff. The `train-env` session fixed all of them and recomputed each number; the main session checked the fixes in the files. The list is in the spec under "Review fixes".
**Broken or open:**
- The transformer's allocator reserved more than the card holds. No error was raised. The run was 24% slower than the smoke run. The update step slows from update 21 on, where the allocator's peak passes the card's size (median 4.1 s before, 6.2 to 6.7 s from update 81 on; 4.4 to 4.5 s in the smoke run). The cause (the Windows driver moving the excess to shared system memory) is inferred, not measured. Flow B has longer sequences, so this gets worse before the full run.
- `gpu_snapshot` in `bench_model.py` returns an empty reading when nvidia-smi prints the temperature with decimals. `run-info.json` and `summary.json` of both runs hold no GPU state. The GPU state in this log comes from the pipeline's own nvidia-smi calls. A small fix with a test, as its own change.
- Another program used the card during the MLP run (11 of 60 one-minute samples at 3,118 to 5,634 MiB). At most 337 s of 3,602 s are affected. Criterion 3 ("nothing else running") did not hold for that run.
- Against random legal play update 2,000 is not measurably above update 1,000 in either run. Whether the networks still improve needs one checkpoint against another (open point C of the spec).
- Play and, for the MLP, talon pick sit above their bands with the weight at the lower limit. The controller cannot push an entropy down. Whether that is a finding belongs to C16.
- The two run folders take 5.8 GB.
**Next:** the user's read of the diff of spec 014; commit; push; CI. Then the answers to Batch 13 and spec 009.

## 2026-11-24 — the longer run started; spec 009 drafted (session 3, Claude Code 2.1.306, claude-fable-5-2; ai-log row 8)

**Done:** the GPU was free at 00:04 and the pipeline of spec 014 started the MLP run (`long-a-mlp-20261124-000454`, commit `9351c6b`, seed 14). Spec 009 (deal flow B: announcements and kontra) is drafted with 49 acceptance criteria. pagat.com was read again for it on this day; the spec lists what the page says and what it leaves open. Seven open points went to the user as Batch 13.
**Measurements:**
- GPU before the MLP run: 0% busy, 536 MiB in use (criterion 3 of spec 014: under 10% and under 700 MiB). The session waited 2 h 55 min for it.
- The run's own log at update 200: 38,432 decisions played per second, 48% of them trained on.
**Broken or open:**
- The run and its evaluations were not finished when this entry was written. Its results go into the run log of spec 014.
- Spec 009 is a draft. No test and no code of it exists.
**Next:** the report of the `train-env` session; verification of the run; the user's answers to Batch 13 and the approval of spec 009.

## 2026-11-23 — spec 008 committed, CI green; the longer run of spec 014 handed out (session 3, Claude Code 2.1.306, claude-fable-5-2; ai-log row 7)

**Done:** the user read the diff of spec 008 and gave the go-ahead (D137); committed as `124b2e9` and pushed; spec 008 is done. The user ordered a longer run (D138). Spec 014 holds it: both networks, 2,000 updates each on deal flow A, the MLP first, everything else as the smoke configs; every setting is a choice of Claude, written down for correction. The run went to the `train-env` session.
**Measurements:**
- CI on `124b2e9`: `ci` success; `engine` success; `train` success in 6 min 43 s with 150 passed, 9 skipped (GPU), 6 deselected (slow). This closes criterion 18 of spec 008.
**Broken or open:**
- The longer run has not started: a game was running on the GPU when it was handed out (79 to 96% busy, about 1,300 MiB). The session waits for a free GPU and gives up after six hours.
- C13 (the new figure of G3) and C16 (the entropy mean of the controller) were asked with the commit question and not answered. The longer run uses the controller as built.
- The repo was private on the day it had to be public or shared with the instructor (R5). The files hold no record that the proposal, the wireframe and the pitch slide of 18 Nov were handed in.
**Next:** the two runs of spec 014 and their evaluations; verification by the main session; the user's read of the two configs; commit. Then specs 009 to 011.

## 2026-11-12 — spec 008, pipeline smoke run on the real engine: run log (session `train-env`, claude-fable-5-2; verified by the main session, ai-log rows 6 and 6e)

**This is a test of the pipeline, not a result about playing strength.** The opponent in the evaluation is random legal play.

**Done:** the trainer runs on the real engine (`kind = "engine"`); monitors per phase; `evaluate.py` (a checkpoint against random legal play, 95% interval, records exported and replayed); the control fault `rewards_to_next_seat`; the entropy controller with a target per phase (D127, targets of D136); the stale-module check (D130); three configs. A review subagent found no defect that corrupts training or evaluation, seven lesser defects in reporting and checks, and 20 planted bugs that the tests let through; all fixed, 26 planted bugs caught after the fixes (session's count). Then the three runs. Not committed: waiting for the user's read of the diff.
**Run log (D79).** Commit `c7d1fa3` plus the uncommitted diff of spec 008 (16 paths, listed in each `run-info.json`). Engine 0.2.0, source fingerprint `64a6157fb05f43c4`. 512 tables, 2,048 seat-deals per update, 200 updates, seed 8. GPU before: 0% busy, 518 MiB. Folders in `src/train/runs/`, untracked; nothing is released.

| | MLP | Transformer | Control (MLP, rewards handed to the next seat) |
|---|---|---|---|
| Config | `smoke-a-mlp.toml` | `smoke-a-transformer.toml` | `smoke-a-mlp.toml`, `--name control-smoke-a --fault rewards_to_next_seat` |
| Run folder | `smoke-a-mlp-20261112-170937` | `smoke-a-transformer-20261112-171542` | `control-smoke-a-mlp-20261112-174730` |
| Updates, exit code | 200 of 200, 0 | 200 of 200, 0 | 200 of 200, 0 |
| Duration | 351.8 s | 1,881.7 s | 342.6 s |
| What failed | nothing | nothing | nothing in the run; it fails criterion 7, as it must |

**Measurements** (from each run's `summary.json`, `evaluation-*.json` and the report script; read again by the main session from the files):
- Loop: 0 illegal actions, 0 non-finite logged numbers, largest sum of a deal's four rewards 1.2e-07, oldest trained decision 0 updates, in all three runs.
- Against random legal play, 2,000 deals in four seatings, seed 2008: MLP +0.598 per deal (95% interval +0.590 to +0.606); transformer +0.605 (+0.597 to +0.613); untrained MLP −0.005 (−0.029 to +0.019); untrained transformer −0.124 (−0.148 to −0.099); control −0.211 (−0.239 to −0.184). Reward 1 is 50 points of the paper score. 100 of 100 exported records replay to the same entries in each run.
- The same three checkpoints evaluated again by the main session on other deals (seed 4040, copies in a scratch folder): MLP +0.595 (+0.587 to +0.603), transformer +0.605 (+0.597 to +0.612), control −0.184 (−0.211 to −0.157).
- Speed end to end: MLP 34,743 decisions played per second and 17,050 trained; transformer 6,506 played and 3,194 trained. 49.1% of the played decisions are trained on. Against the old G3 of 10,000: the MLP is 3.5 times above, the transformer at 65% played and 32% trained.
- Time split (environment, acting, bookkeeping, update): MLP 1.9%, 24.3%, 30.7%, 42.5%; transformer 0.4%, 46.2%, 5.8%, 47.5%.
- GPU memory, transformer: tensors' peak 4,441 MiB, allocator 5,156 MiB, whole card at the end 5,350 of 6,144 MiB. MLP: 294, 506, 1,115 MiB.
- Seat-deal length: 14.4 decisions on average, 20 at most; 67 tokens at most (limit 128).
- Bidding: the untrained policy bids solo ena in over 90% of the deals and the declarer's side wins 3.7%. After 200 updates: MLP dve 34.7%, ena 65.3%; transformer tri 6.8%, dve 48.8%, ena 44.1%; the declarer's side wins 65.7% and 66.9%.
- Entropy: the bidding collapsed around update 24 in both runs (0.012 to 0.016 against a target of 0.2); the controller raised that phase's weight from 0.01 to 0.38 and 0.59 and the entropy was back at 0.21 and 0.24 at the end. Play, discard and talon pick never came down to their targets.
- Heads at the last update: partner seat 95.6% and 92.4% (uniform guess over the possible seats 90.4% and 85.8%); card location 36.3% and 37.1% (guess 30.9% and 30.8%). Critic's explained variance 0.59 and 0.61.
- Tests on the final tree, re-run by the main session: 159 passed with the GPU tests, 6 slow left out. Three bugs planted by the main session in a scratch copy (the fault always on; the controller never steps; the interval over seat-deals) each fail a test.
**Broken or open:**
- The transformer run leaves about 800 MiB of the card free. 512 tables are the limit on this card; longer sequences (flow B) may not fit.
- Half of the play is not trained on (deals that straddle an update are dropped; 8.7% are a past checkpoint's seats).
- The controller steers on a phase mean that includes forced decisions; their share moved from 28% to 0% in the bidding during the runs. In the control run a phase with 99% forced decisions drove its weight to the upper limit without effect. Both means are logged; the user decides which one steers (C16).
- No run without the controller on real deals, so its effect on the bidding collapse is not measured against a fixed weight.
- The untrained transformer is under zero against random play (−0.12); the session's reading (first scale 0.3 gives fixed preferences) is not checked.
- The control corrupts about half the signal (two of three opponents of a solo keep their reward); it still failed clearly.
- `test_c20` of spec 006 fails on an idle PC (18.5% against 20%) and passes under load. Cause not established.
- CI is unverified for this code until the push.
**Next:** the user's read of the diff of spec 008; commit; push; CI. Then the new figure of G3 (C13), the controller's mean (C16), and specs 009 to 011.

## 2026-11-12 — spec 013, three rulings and D133 in the engine (session `engine-core`, claude-fable-5-2; verified by the main session, ai-log rows 6 and 6d)

**Done:** the trick winner and the score follow the user's table: the pagat wins a trula trick only when played after the škis and the mond (D131, D132); pagat ultimo is signed by the side that takes the last trick (D126); a valat keeps a mondfang lost in a trick (D125); in the declarer's valat the packets not taken are the declarer's, so a mond in them costs nothing (D133, ruled during the review). Crate version 0.2.0. The shared random-case helper of the tests has fixed seeds now. A review subagent found no defect in the code; it found two wrong rules that the suite let through, a clock-seeded test that almost never drew a trula trick, and two statements in Results that held only in some runs. All fixed. Not committed: waiting for the user's read of the diff.
**Measurements:**
- Re-run by the main session on the final tree: `cargo fmt --check` and clippy clean; `cargo test --locked` 182 tests, twice with the same result; 171 with `--no-default-features`; no `pyo3` in the engine's tree.
- Planted wrong rules: 4 old rules by the author (all caught), 24 by the reviewer (2 not caught before the fixes, 2 that cannot change behaviour), 9 by the author after the fixes (all caught). The main session planted three in its own scratch copy: the declarer's valat does not win the talon rest; every valat wins it, also the valat against; no pagat ultimo in a trula trick. Each fails at least one test.
- `winner_matches_the_reference_on_random_tricks` now builds a quarter of its 10,000 tricks with the whole trula: 2,519 trula tricks (session's count), before about two.
**Broken or open:**
- The module in `.venv` is still built from engine 0.1.0; the `train-env` session builds it again when its jobs end. The Python parity tests passed only against a wheel in a scratch folder (26 of 26, session's run).
- Replay does not look at the engine version: a 0.1.0 record can replay to other entries under 0.2.0. A question for the user in the spec.
- CI is unverified for this code until the push.
**Committed** on the user's go-ahead (D134) as `ce175f6`. CI on it: `ci` success; `engine` success in 3 min 2 s; `train` success in 7 min 9 s with 88 passed, 8 skipped, 5 deselected, the module built from engine 0.2.0 on Linux.
**Next:** verify and review the code of spec 008 before its three long runs.

## 2026-11-12 — spec 006 committed, the user's answers recorded, specs 008 and 013 started (session 3, Claude Code 2.1.306, claude-fable-5-2; ai-log row 6)

**Done:** the user answered everything open ("Specs accepted, continue"; D123–D131). Spec 006 is committed in two commits (`8f24dbb` engine, `8cb2bb0` bindings) and pushed. Batch 11, Batch 12 and C11 are recorded. `docs/encoding.md` has section 4a with the fifteen readings; the version stays 1. Spec 008 is approved and carries the settings of D127 and the stale-module check. Spec 013 is new: three rulings that change the engine (a valat keeps the mondfang; pagat ultimo by side; the pagat wins a trula trick only if it falls last). The local compiler pin file is deleted (D129). Spec 013 went to the `engine-core` session and spec 008 to the `train-env` session.
**Measurements:**
- CI on `8cb2bb0`: `ci` success; `engine` success in 3 min 1 s; `train` success in 4 min 45 s. The `train` log shows the wheel of `tarok_env` built on Linux in 27 s and pytest with 88 passed, 8 skipped (GPU), 5 deselected (slow). The binding crate has no Rust tests of its own ("0 passed"), so `cargo test` there only proves that it links.
- After the amendment of `docs/encoding.md`: `tools/check_encoding.py` 3 of 3, the engine's `c01` contract test and the learner's three `c01` tests pass.
**Broken or open:**
- The engine plays the old readings of the three rulings until spec 013 is in.
- C12 is open: what "falls last" means exactly. Spec 013 builds reading (a) in one marked place.
- G3 has no figure now (D128); it is proposed after the smoke run (C13).
- Open issues of spec 006 stay as listed in its entry below.
**Next:** verify and review the output of spec 013 and of spec 008's code; the user's read of each diff; then the three smoke runs (about three hours with the GPU free).

## 2026-11-11 — spec 006, encoding, vectorised environment, bindings, record (session `engine-core`, claude-fable-5-2; verified by the main session, ai-log rows 6 and 6c)

**Done:** in `src/engine`: an event log on `Game`; the module `encode` (tokens, the 1,382 state features with the exact inference bits, mask, critic features, targets, reward with ten parts); `vecenv` (N tables, runs of 24 deals, own thread pool, behind a feature so a build without threads stays possible); `record` (JSON, replay). A separate binding crate `src/train/tarok_env` (PyO3 0.30.1, no `unsafe`, no rule in it) and `src/train/tarok_train/env.py` with the mock's interface. A review subagent found no leak, no unsound bit and no wrong reward in the code. It found six planted wrong rules that the suite let through (one a leak), loose array checks in the binding, a contract reading nobody had listed (C4) and three overclaims; 19 items, all fixed. Not committed: waiting for the user's read of the diff.
**Measurements:**
- Re-run by the main session on the fixed tree: `cargo fmt --check` and clippy clean for both crates; `cargo test --locked` 180 tests; `--no-default-features` 169; no `pyo3` in the engine's dependency tree. Full pytest with the GPU and the slow tests: 101 passed in 253 s.
- Environment alone, 1,024 tables, random legal play, a full observation written at every step (session's run after the fixes, median of 3): through Python 396,822 steps/s with 1 thread and 976,682 with 12; inside the engine 564,587 and 3,213,112. The main session measured 374,161 and 968,028 through Python before the fixes. With 12 threads the single Python thread is the limit. The environment does not limit G3.
- Longest token sequence: 66 in 100,000 random deals; 71 is the longest possible in this slice. The limit is 128.
- No-leak test, last run: 15,415 cases compared in four kinds, 1,287 of them the kind the review added (which card the declarer put down).
- Soundness: 10,000 deals, four observers at every decision; 4.2 million void bits, 223 million "cannot hold" bits and 1.0 million "certainly holds" bits checked against the true hands, none wrong.
- Planted wrong rules: 26 by the author (24 caught; one changes nothing in this slice, one was dead code and is removed), 6 by the reviewer (0 caught before the fixes), 22 for the review items after the fixes (all caught). The main session planted the reviewer's leak again in a scratch copy: the no-leak test fails, the soundness test passes, as it should.
**Broken or open:**
- The two workflows are unverified for this code until the push: the build of the module on Linux, and `cargo test` of the binding crate there.
- A thread that holds the observation arrays can read half-written rows while another thread steps. `EngineEnv` has a lock for its own calls and the rule is written down; the code cannot enforce the rest without copying every observation.
- A stale installed module is not detected: after a change in `src/engine` the module must be built again (`maturin develop --release` in `src\train\tarok_env`).
- The trainer still starts the mock; the switch to the engine is in spec 008.
- The declarer's hidden discards give repeated token sequences; only the state token differs between them.
- Fifteen points wait for the user in Batch 12.
**Next:** the user's read of the diff of spec 006; commit; push; the two workflows. Then spec 008 once approved.

## 2026-11-11 — commit of specs 005 and 007, CI (main session, ai-log row 6; Claude Code 2.1.306, claude-fable-5-2)

**Done:** five commits pushed, `2bd5f21` to `819220d`: the engine's deal flow A (spec 005), the GPU-free benchmark results (spec 003), the encoding check script (spec 004), the model and learner (spec 007), and D122. The user's go-ahead was the word "Continue" in answer to the commit question; D122 records it as said. Specs 003, 004, 005 and 007 are done. Spec 006 is handed to the `engine-core` session. The open rulings are written down as Batch 11 in `docs/planning/open-questions.md`.
**Measurements:**
- Before the commits, on this PC: `cargo fmt --check` and clippy clean, 130 engine tests pass; the full training suite, with the GPU and the slow tests: 75 passed in 1,100 s. The CI set without a GPU (`-m "not slow"`, `CUDA_VISIBLE_DEVICES=-1`): 65 passed, 5 skipped, 5 deselected in 285 s.
- CI on `819220d`: `ci` success; `engine` success (format, clippy, 130 tests, the deal against the Python reference); `train` success: 62 passed, 8 skipped, 5 deselected in 148 s on the runner. The three more skips than here are the PyO3 probe tests; the probe is not built on the runner.
**Broken:** nothing new. The open points of the entries below stand: G3 and the transformer, the plateau on the token task, the value head's gradient, the acting cache, the valat and mond ruling.
**Next:** spec 006 (encoding in Rust, vectorised environment, bindings, record). Spec 008 (pipeline smoke run) to be drafted for approval. Batch 11 for the user.

## 2026-11-10 — session 2 continued and session 3, ai-log rows 4–5 (Claude Code 2.1.306, claude-fable-5-1 then claude-fable-5-2)

**Done:** Batches 7–10 answered; decisions D70–D113. Plan finalised as `docs/planning/plan.md`. Screen spec draft 2. Figma wireframe: 28 greyscale frames and one styled sample (`docs/wireframe/wireframe-overview.png`). Pitch text draft. Build plan with twelve specs to the first full training run. Specs 001–003 approved; spec 001 (this skeleton) executed.
**Measurements:** none of the product. PC check, read-only: GTX 1660 SUPER, driver 610.88, 6144 MiB with 2836 MiB in use by the desktop; global PyTorch is 2.12.0+cpu; Rust 1.94.1 on the `x86_64-pc-windows-gnu` toolchain; maturin and wasm-pack not installed. `python tools/check_skeleton.py`: all 10 checks pass; they cover criteria 1–9 of spec 001 (criterion 10 and the CI run are checked by hand). `python tools/check_trailers.py`: all commits since 0d5045d carry the trailer.
**Broken:** nothing built yet. The proposal format is unknown. Wireframe frames A-3 and A-5 do not show the D104 step.
**Next:** specs 002 (engine core) and 003 (training environment and benchmark) in parallel; spec 004 (encoding contract) for approval.

## 2026-11-11 — GPU-free benchmark (spec 003, open point B) and spec 007, model and PPO learner (session `train-env`, claude-fable-5-2)

**Done:** the benchmark of spec 003 ran once with the GPU idle. Spec 007 is written: the transformer in layout L2, the MLP control, the full-information critic, PPO with ten-part decomposed values (D121), whole-deal collection, past-checkpoint seats, checkpoints with resume, TensorBoard logs, a mock environment. A review subagent found no bug in the PPO math or the model, but showed that the first learning test let ten planted bugs through; thirteen findings were fixed. Not committed: waiting for the user's read of the diff.

**Measurements, GPU free** (nvidia-smi before and after: 0% busy, 234 MiB used; 3 repeats per row, medians, fp32; repeats agree within 1%):

| Network | Case | Acting decisions/s | Update samples/s |
|---|---|---|---|
| MLP 512-512-256 | | 1,546,968 | 640,075 |
| transformer | L1 bucketed, acting uncached | 7,673 | 2,532 |
| transformer | L2, acting cached, ragged, action copied to host | 34,911 | 19,864 |

- The free GPU is 4.0 to 4.8 times faster than under the game, in every row.
- Update, L2 against L1 bucketed: 7.8 times. The layout choice (D116) now stands on free-GPU numbers.
- Half precision is 2.6 to 4.5 times slower. fp32 stays.
- End-to-end estimate, **derived**, learner alone, the benchmark's synthetic shape: L2 gives 7,732 steps/s with 2 epochs, 4,348 with 4, 12,660 with 1. MLP 265,177 with 2 epochs.

**Measurements, spec 007** (session's numbers; tests re-run by the main session):

- Parameters: transformer 3,767,076; MLP control 1,242,916; critic 1,244,424.
- Mock task that can only be solved from the observation (random play 0.25, best 1.0), CPU test, 30 updates: transformer 0.28 → 0.80 in 49 s (the rewarded slot is named by a token); MLP 0.25 → 0.94 in 12 s (named by a state feature). Controls: reward to the wrong seat stays at 0.25; the MLP on the token task stays under 0.52.
- Ten planted bugs (mask left out of the update, stored log-probability zeroed, critic never trained, advantages reversed, no bootstrapping, reward averaged or swapped across deals, state from another seat-deal, decision points zeroed, cache never reset): each now fails at least one test. With the first version of the test all ten passed.
- 200 updates on the free GPU, 1,024 tables: MLP plays 19,604 decisions per second (median), transformer 5,621. Only whole deals under the current weights are trained on, so 39% of the decisions played are trained on with 2,048 seat-deals per update. Decision age in every update: 0. Peak GPU memory of the transformer run: 3,621 MiB.
- Gradient norms on the trunk at the transformer's first update: policy 1.33, value head 19.9, Q head 2.21; whole actor 22.2 against a clip of 0.5.

**Broken or open:**
- G3 (10,000 steps/s): not reached by the transformer in any measured or derived number with 2 or more epochs, before the engine is counted. The MLP is far above it.
- The transformer stops at 0.81 on the token task: it follows the token (1.000 right) and ignores the state hint (0.248 right), with policy entropy near zero from update 20. The cause is not established; an early entropy collapse is the guess.
- The policy head's first scale had to go from 0.01 to 1.0 for anything to learn on the stronger task; with 1.0 the first policy is not uniform.
- The acting cache needs about 8 KB per token per table and seat: 1,024 tables with 128-token deals do not fit on the 6 GB card next to the update.
- Whole-deal collection wastes the deal that straddles each update.
- The `train` workflow is unverified for this code until the push.

**Next:** the user's answers on loss weights, first scale and the plateau; the read of the diff; commit; then spec 006.

## 2026-11-10 — spec 005, deal flow A (session `engine-core`, claude-fable-5-2)

**Done:** in `src/engine`: bidding with priority, the declarer's contract choice, king call, talon packets and discards, the called king in the talon, play, and the score of the six normal contracts with silent bonuses and mondfang. Actions map to the 160 slots of the encoding contract. Public, private and full-information views are separate types. A review subagent found one bug (the debug text of the public view printed hidden cards) and test gaps in scoring and bidding; all are fixed. Not committed: waiting for the user's read of the diff.
**Measurements:**
- `cargo test --locked`: 130 tests pass (re-run by the main session); 47 of them are spec 002's. Format and clippy clean.
- Benchmark, `cargo run --release --bin bench_game`, one thread, CPU shared with a training run: 349,000 to 362,000 whole random deals per second, about 20 million actions per second, 56.8 actions per deal on average. The reviewer measured 316,000.
- Auctions: 1,297 possible in this slice, all walked for each dealer against a model written from the criteria. The longest has 13 calls. For the full ladder of the Default preset the longest has 19 calls (36,182 auctions, by the same model), so the 128-token limit of the encoding holds with room.
- A second scorer written only from the criteria agrees with the engine on 10,000 random deals.
- Mutation checks: 32 wrong rules by the author, 4 more scoring rules and 4 leaks after the review; each fails at least one test.
**Broken:**
- Open ruling: what a valat does to a lost mond (three cases). The current behaviour (the valat cancels every mondfang) is pinned by five tests.
- Random test auctions are thin (85% have one bid); the exhaustive walk covers the bidding instead.
- No rules parameter yet for score values, phase order and legality; flows B and C need one. No ordered action history yet; spec 006 adds the event log.
- The `engine` workflow is unverified for this code until the push.
**Next:** the user's read of the diff; commit; then spec 006.

## 2026-11-10 — first CI runs after the commit of specs 001–003

**Done:** five commits pushed (`011e288` to `3dbe257`).
**Measurements:** workflow `ci` (trailers, skeleton): success. Workflow `train` (CPU tests of `src/train`): success. Workflow `engine`: **failure**.
**Broken:** the `engine` run used the runner's newest stable compiler (clippy 1.99), because the toolchain file is switched off. That clippy has a lint the pinned 1.94.1 does not (`chunks_exact_to_as_chunks`, `src/engine/src/deal.rs:81`) and `-D warnings` turns it into an error. Tests did not run. The fix pins 1.94.1 inside `engine.yml`; it waits for the user's read of the diff.
**Next:** commit the workflow fix, re-run, then mark specs 001 and 002 done.

Update, same day: the fix is commit `56f4caf` (D115). All three workflows are green on it. In `engine`: format, clippy, 47 tests, and the deal compared with the Python reference on five seeds all passed on Rust 1.94.1. Specs 001 and 002 are done. Spec 003 waits only for the GPU-free benchmark run.

## 2026-11-10 — spec 003, training environment and benchmark (parallel session `train-env`, claude-fable-5-2)

**Done:** `.venv` with pinned packages (`src/train/requirements.txt`), `envcheck`, a PyO3 probe built with maturin on the GNU toolchain, a throughput benchmark of the stand-in transformer (3.49 M parameters) and the MLP control (0.83 M) in both sequence layouts. A review subagent found no functional bug, four method gaps, five overclaims and two test gaps; all were fixed and the benchmark was re-run with the better method. Not committed: waiting for the user's read of the diff.

**Measurements.** Every benchmark run had a game on the GPU (75 to 97% busy). The absolute numbers measure a loaded GPU. **The GPU-free run is pending.**

`python -m tarok_train.envcheck` (exit code 0, re-run by the main session):

```
torch: 2.14.1+cu130
torch cuda build: 13.0
numpy: 2.5.4
cuda available: True
device: NVIDIA GeForce GTX 1660 SUPER
capability: (7, 5)
cudnn: 92400
vram total: 6143 MiB
vram free: 5138 MiB
gpu matmul: ok
```

- `pytest` in `src/train`: 23 tests pass (re-run by the main session).
- Download 2,045 MB; `.venv` takes 3.2 GB on disk.
- PyO3 probe: works on `stable-x86_64-pc-windows-gnu` with `PYO3_USE_RAW_DYLIB=0` (pyo3 0.30.1, maturin 1.16.0). Without it the build fails: `dlltool` finds no assembler on this PC. No MSVC needed.

Benchmark run 3 (3 repeats per row, median, fp32, loaded GPU):

| Network | Case | Acting decisions/s | Update samples/s |
|---|---|---|---|
| MLP 512-512-256 | | 358,828 | 142,829 |
| transformer | L1 bucketed (recompute per decision), acting uncached | 1,593 | 592 |
| transformer | L2 (one pass per seat and deal), acting cached, deals at different points, action copied to host | 7,935 | 4,909 |

- Update, L2 against L1 bucketed: 6.8 to 8.3 times over four runs. The ratio follows from the synthetic shape (85 tokens for 15 decisions against about 47 padded tokens per decision).
- State width 1,382 as in the encoding draft changes little: L2 update 4,707, L1 bucketed 681.
- Half precision is slower on this card: a raw 2048 × 2048 matmul takes 24 ms in fp32 and 122 ms in fp16. Use fp32.
- Single-thread CPU, batch 1: MLP 0.16 ms, transformer 2.9 ms (cached) to 5.1 ms per decision. Mostly PyTorch overhead; a weak hint for G14.

End-to-end estimate, **derived**: `steps/s = 1 / (1/A + E/U)` with A acting decisions/s, U update samples/s, E PPO epochs. The engine, data transfer, GAE and the critic are not counted.

| Case (fp32, loaded GPU) | 4 epochs | 2 epochs |
|---|---|---|
| MLP | 32,476 | 59,561 |
| transformer L1 bucketed, acting cached | 145 | 285 |
| transformer L2, acting cached | 1,063 | 1,875 |

G3 (10,000 steps/s): the MLP is above it even on the loaded GPU. For the transformer it cannot be answered until the GPU-free run; under load both layouts are below it, L2 by less.

**Broken:** no GPU-free run yet. CI not run. An unknown Python 3.14 process used the GPU during run 1 and ended on its own. The first import of torch after the install took over two minutes; cause unknown.
**Next:** the user runs the GPU-free benchmark; the layout choice (D110) follows; spec 007 (model and learner) is drafted on that choice.

## 2026-11-10 — spec 002, engine core (parallel session `engine-core`, claude-fable-5-2)

**Done:** `src/engine`, crate `tarok-engine`: cards, card sets, card points, seeded deal, trick winner, legal cards for normal contracts, card-play state machine, benchmark binary. Tests were written from the 33 criteria before the code. A review subagent found no rule bug and seven test or API gaps; all seven were fixed. Not committed: waiting for the user's read of the diff.
**Measurements:**
- `cargo test --locked`: 47 tests pass (re-run by the main session). `cargo fmt --check` and `cargo clippy --locked --all-targets -- -D warnings`: clean.
- Property tests: 10,000 cases each for criteria 6, 7, 27–30.
- Benchmark, `cargo run --release --bin bench_play`, one thread, Ryzen 5 5600: 35.8 to 36.6 million card plays per second (about 750,000 deals per second), three runs. The reviewer measured 34.4 million with the CPU shared. Card play only: no bidding, no encoding, no network. research/05 D4 had inferred 10^5 to 10^6.
- Shuffle check by the reviewer: chi-square 2742 for 2809 degrees of freedom on 540,000 decks. The Rust deal equals a separate pure-Python derivation on five seeds.
- Mutation check by the author: seven wrong rules put into the code one at a time; each made a numbered test fail.
**Broken:**
- `rust-toolchain.toml` at the root makes plain `cargo` fail on this PC: rustup's default host is MSVC and that toolchain cannot link here. Runs used `RUSTUP_TOOLCHAIN=stable-x86_64-pc-windows-gnu`. The user chooses the fix.
- Incident: a review agent ran `rustup toolchain list` in the repository; the toolchain file made rustup install `1.94.1-x86_64-pc-windows-msvc` (588 MB) without anyone asking. The main session uninstalled it and renamed the file to `rust-toolchain.toml.disabled`. The two toolchains that were there before (stable GNU, stable MSVC) are untouched.
- proptest is built without its `std` feature (no `dlltool` assembler on this PC), so the tests seed their own runner; regression files and `PROPTEST_*` settings are lost.
- Criterion 32 (CI green) is unverified until the first push.
**Next:** spec 005 (deal flow A). Notes for it: legality and trick rules need a rules parameter for klop and berač; `PlayState` needs a constructor for positions inside a deal; which seat is forehand is not defined yet.

## 2026-11-09 — session 2 (Claude Code 2.1.306, claude-fable-5-1)

**Done:** Batches 1–6 answered; decisions D1–D69. Git repository and private GitHub repo. Deep research on training methods, network and browser inference, and explanation methods (`docs/planning/research/05`–`07`). Rule-switch list with 39 toggles. Goals G1–G19.
**Measurements:** none.
**Broken:** nothing.
**Next:** agent workflow batch.

## 2026-11-06 — session 1 (Claude Code 2.1.306, claude-fable-5-1)

**Done:** Course requirements extracted (R1–R20). First research pass (`docs/planning/research/01`–`04`): rules and variants, existing software, bot methods, card artwork licensing. Planning files created.
**Measurements:** none.
**Broken:** nothing.
**Next:** first decision batch.
