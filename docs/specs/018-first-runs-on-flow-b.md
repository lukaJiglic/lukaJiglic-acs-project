# 018 — First runs on deal flow B

Status: **done** (2026-11-27). Code `41c6f29`, configs `453885c`. Two review passes on Opus 5.5 (code; Results against the run files); their points are in. CI after the push.

Tool and model: Claude Code. Drafted by the main session with claude-fable-5-2; executed in the `train-env` session with claude-opus-5-5 (D152, D154).

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

Executed in the `train-env` session with Claude Code, model claude-opus-5-5 (D152, D154), 2026-11-25 to 2026-11-27. Nothing committed (D109). Engine 0.3.0, `tarok_env.SOURCE_HASH` `fe9b13e811e50d2b`. The runs ran on the working tree of this spec over `main` at 09ce2a1 (the MLP run), 52ecd40 (the MLP evaluations) and 977ba50 (the transformer runs and evaluations); each `run-info.json` holds the commit and `git status`, and `runs/spec018/runs.log` the commit of each step.

### What ran

| Run | Folder (under `src/train/runs/`) | Status |
|---|---|---|
| MLP | `smoke-b-mlp-20261125-212012` | 200 updates, 419 s, not disturbed. Used. |
| Transformer, first | `smoke-b-transformer-20261125-212719` | Out of memory in update 3 (criterion 13). Stopped and reported. Not used. |
| Transformer, after D155 (a) | `smoke-b-transformer-20261127-011341` | Disturbed: a video render of another project of the user's held 635 to 1,477 MiB from about 14 s in. Stopped by hand after 54 updates (TensorBoard; the last checkpoint is 50) (criterion 11). Kept as the memory record of D155. Not used. |
| Transformer, after D155 (a) | `smoke-b-transformer-20261127-022207` | 200 updates, 2,062 s, not disturbed. Used. |

### The transformer's memory (criterion 13, D155)

- **The first run** stopped in update 3 with an out-of-memory error. The error came in the per-term gradient diagnostic of `ppo.update`, asking for 132 MiB more. PyTorch held 4.69 of its 4.88 GiB.
  - The peak of the tensors was 4,782 MiB in updates 1 and 2, and 4,815 MiB at the failure; reserved, 4,992 MiB.
  - The card was free and no build ran. The limit and the number of tables were not changed (criterion 13).
- **The cause:** the acting cache. It holds 2,048 slots × 4 layers × keys and values × 4 heads × 64 × float32, about 16 MiB per position.
  - Flow A's longest seat-deal had 66 tokens, so its cache stopped at 68 positions, about 1,040 MiB. On flow B a seat-deal reaches 92 tokens in the clean run (about 22 decisions on average at update 1, up to 41), so the cache grows to its cap of 130 positions: 2,080 MiB. The cap still covers the longest seat-deal.
  - `Actor.reset` only set the counters to zero, so the cache stayed allocated through the update step.
- **D155 (Batch 22), the user's answer:** free the cache before the update step; if that is not enough, `update_batch` 128 (b), then a limit of 5,300 MiB (c).
  - Only step (a) was taken. `KVCache.release` and `Actor.release` (`model.py`) free the keys and values. `Trainer.update` (`train.py`) calls `Actor.release` after the reading "after acting" and before `ppo.update`. The cache grows again from its first size in the next acting phase.
  - Steps (b) and (c) were not needed. The transformer keeps the same `update_batch` as flow A's runs, so its number of optimizer steps is unchanged.
- **The test, written first:** `test_d155_freeing_the_acting_cache_before_the_update_changes_no_number`. Three CPU updates of the transformer give the same numbers with and without freeing. The update step runs with the cache freed, and every acting phase grows it again. `test_d155_a_freed_cache_grows_again_from_its_first_size` covers the regrowth. Both failed before the code and pass now.
  - On the card, update 1 of the clean run gives loss 49.4266, the same as the first run.
- **The result:** the peak of the tensors is now 2,871 MiB (before: 4,782 MiB), and PyTorch reserves at most 4,766 MiB, under the limit of 5,000 MiB. "Allocated after the update step" falls from about 2,190 MiB to 109 MiB. The disturbed run `…-011341` shows the same peak, 2,871 MiB, over its 54 updates.
- **The cost:** one regrowth (34 → 68 → 130 positions, 512 tables) takes 22.2 ms on the card (median of 10 repetitions after the first, 22.0 to 23.3 ms). That is about 0.2% of an update of about 10.9 s. It was measured on a clean card; the output is in `runs/spec018/regrow.txt`.

### Criteria

| # | Met | Evidence |
|---|---|---|
| 1 | yes | `ppo.round_stats`, logged per update. Tests c01: built token arrays, and engine deals compared with the actions taken. |
| 2 | yes | The five levels add up to 1 (c02, also on the Trainer's log); `round/bonus_kontra` (c02). The level is the number of kontra tokens on the game. |
| 3 | yes | `reward/abs_mean`, `abs_max`, `share_above_10` (c03). |
| 4 | yes | `round/decision_share`, `round/decisions_per_seat_deal` (c04). Padding rows are not decisions. |
| 5 | yes | A deal is counted once through its key, whatever number of its seats is trained (c05). |
| 6 | yes | `--opponent silent-random` (`evaluate.SilentRandom`). Over 500 deals among four silent seats, no announcement or kontra token is written and no action is illegal. Outside the round it draws the same numbers as random legal play (c06). |
| 7 | yes | The file is `evaluation-<update>-vs-silent-random.json`, and the JSON has `opponent_kind`. `random` keeps its old file name; a checkpoint as opponent works as before (c07, spec 015's tests). |
| 8 | yes | 32,000 deals (seeds from 3018), the mean of ln(number of legal slots) over the round's decisions, the method of D136. It gives back the six D136 values, and 0.9150 nats for the round (978,679 decisions; 22.9% have one legal slot). The number is in both configs' comments. |
| 9 | yes | A quarter is 0.229. Rounded to 0.05 as D136 did, it is 0.25, equal to the placeholder. The test checks the configs against `flowb_report.target_of`. |
| 10 | yes, with one exception | `smoke-b-<network>.toml` is `smoke-a-<network>.toml` key by key, except the name and the target (c10). The target came out equal to the old value. The one exception is the transformer's `gpu_memory_limit_mib = 5000`: criterion 13 runs the transformer under spec 015's limit, and D155 builds on it (its step c raises it to 5,300). The MLP keeps 0. |
| 11 | yes | The card was read before each run: free under 10% busy and under 700 MiB, by `measured_run.sh`. The clean transformer run also waited for two free readings 5 minutes apart with no render, and had a watcher every 10 s. In the used runs' samples (MLP 60, transformer 293, evaluations 5 and 19) no render, no build and no other program over 300 MiB was seen. One counter was set aside: the window manager (dwm) shows 4,245 MiB of dedicated memory in every sample of the MLP's run and evaluations, and 1,865 to 1,876 MiB in the transformer's. It does not change during a run, and at the same moments `nvidia-smi` read the whole card at 208 to 338 MiB (at the start of each run). So it cannot be memory held on this card; where the counter's number comes from is not known. The disturbed run `…-011341` was stopped and is not used. |
| 12 | yes | Both used runs reached 200 updates. Illegal actions 0. The largest sum of a deal's four rewards was 3.83e-6 at a largest reward of 160.4, within D147's tolerance of 1e-6 × 160.4 = 1.6e-4. No logged number was not finite. The first transformer run stopped with its reason in the log (criterion 13). |
| 13 | yes | Stopped and reported. The remedy went to the user (Batch 22, D155); see above. |
| 14 | yes | The tables below. Seconds per update are in `time/seconds`. From D155 the transformer's `gpu/allocated_after_update_mib` no longer holds the acting cache, so it is not comparable with flow A readings. The regrowth costs about 0.2% of an update. |
| 15 | yes | The valat stays under 50% / 20% / 5% from update MLP 5 / 6 / 8 and transformer 7 / 8 / 14. |
| 16 | yes | The evaluation table below. |
| 17 | yes | Below, by both measures of D153. |
| 18 | yes | The evaluation table below. |
| 19 | yes, both | Over updates 151 to 200: MLP, valat in 0.1% of deals and a mean reward size of 1.213; transformer, 0.3% and 1.373. The limits are 5% and 2.0. The flood ends by itself for both networks. |
| 20 | yes | Criterion 19 holds for both, so no remedy is proposed (D148). No setting was changed. |
| 21 | locally yes (both suites after D155); CI after the push | "Suite" at the end. The `train` workflow runs after the main session's push. |
| 22 | yes | 14 of 14 wrong changes caught, each by a numbered test (list below). |
| 23 | draft | The README describes the monitors, the silent opponent, the report script and the memory change of D155. The run log for `docs/progress.md` is drafted below; this session does not write that file (D109). |
| 24 | yes | `SOURCE_HASH` is `fe9b13e811e50d2b` before and after; `src/engine` and the binding are untouched. |

### Evaluations (criteria 16 to 18)

2,000 deals each, four seatings per deal.

| Network | Evaluation | Seed | Mean | 95% interval | SD of a deal | Deals for ±0.1 |
|---|---|---|---|---|---|---|
| MLP | 200 against silent-random | 2008 | +0.874 | +0.863 to +0.885 | 0.250 | 24 |
| MLP | 200 against random | 2008 | +8.13 | +7.36 to +8.89 | 17.41 | 116,483 |
| MLP | untrained against silent-random | 2008 | −2.37 | −2.45 to −2.30 | 1.709 | 1,123 |
| Transformer | 200 against silent-random | 2008 | +0.713 | +0.703 to +0.723 | 0.224 | 20 |
| Transformer | 200 against random | 2008 | +8.79 | +7.98 to +9.59 | 18.30 | 128,600 |
| Transformer | untrained against silent-random | 2008 | −2.43 | −2.51 to −2.35 | 1.713 | 1,128 |

- **The width on flow B:** "Deals for ±0.1" is (1.96 × SD / 0.1)², rounded up: the number of deals that would give an interval of ±0.1. Against random play the rewards are large: about 116,000 to 129,000 deals would be needed. Against silent-random, 20 to 24 deals.
- **Game records:** 20 of 20 replay to the same entries, in every evaluation that exported them (both networks).
- **Not a judgement of strength:** both trained networks are far ahead of both fixed opponents, and both untrained networks are far behind silent-random. The two networks are not compared with each other.

### Progress inside a run (criterion 17), seed 5151

| Network | 200 in one seat among three of 100 | 100 in one seat among three of 200 | Half the difference, paired | Correlation | The old rule |
|---|---|---|---|---|---|
| MLP | +0.0786 (+0.0371 to +0.1200) | −0.0837 (−0.1347 to −0.0327) | +0.0811 (+0.0464 to +0.1159) | −0.122 | update 200 ahead |
| Transformer | +0.0660 (−0.0043 to +0.1363) | −0.0974 (−0.1975 to +0.0027) | +0.0817 (+0.0196 to +0.1438) | −0.034 | no difference shown |

- **D153:** both measures are reported, and the old rule decides. For the transformer the two measures disagree. Each interval of its own crosses zero, so the old rule shows no difference. Half the difference, paired over the deals, lies above zero.
- **Within one run:** both checkpoints come from the same run, so the handicap between two runs does not apply here.

### Tests (criteria 21 and 22)

- **The new tests:** `test_spec018.py` holds 61 tests, written before the code: criteria 1 to 7, 9, 10, 12, 14 to 19, and D155.
- **The suite:** see the last line below. The suite with the GPU, before D155: 324 passed, 2 failed (both fixed below), 7 deselected; so it was not the full suite. After D155: the 142 tests of specs 6, 7 and 15 pass on the CPU, and the full suite passes (CPU part and GPU part, see "Suite").
- **Earlier tests that had to change:**
  - `test_spec009.py` line 400, `test_r2_no_config_says_that_the_engine_has_no_announcements`. It counted exactly 12 engine configs, and this spec adds two. It now asserts `>= 12`, with a comment: an exact count breaks with every new config. This is the only change to that file (spec 009 is done).
  - `test_spec015.py` c09 wants "MiB" in every limit comment. No test changed: the transformer config's comment was fixed instead.
- **The 14 wrong changes (criterion 22):**
  - a monitor reads the wrong token id (c01)
  - a deal is counted once per trained seat (c05)
  - the silent opponent announces (c06)
  - the game's level is read from the first kontra token, not the last (c02)
  - tokens after the seat's last visible token are read (c01)
  - a kontra on the game counts as one on a bonus (c02)
  - a reward of 10 counts as above 10 (c03)
  - the decision share reads the phase next to the round (c04)
  - padding counts as decisions (c04)
  - silent-random writes the file of random play (c07)
  - the silent opponent draws numbers only for tables outside the round (c06)
  - criterion 19 is read over updates 150 to 199 (c19)
  - a mean reward size of exactly 2 passes (c19)
  - "stays under" is taken as the first update under the level (c15)
- **Two more wrong changes for D155, both caught:** the cache is not freed before the update step (the first D155 test), and a freed cache grows again from 0 instead of from its first size (the second D155 test). Each made one test fail; both files were put back byte for byte. The output is in `runs/spec018/mutants-d155.log`.

### Files

- Code: `ppo.py` (`round_stats`, called by `deal_stats`), `evaluate.py` (`SilentRandom`, `--opponent silent-random`, `opponent_kind`), `model.py` and `train.py` (D155), and the new `flowb_report.py`.
- Configs: `configs/smoke-b-mlp.toml` and `smoke-b-transformer.toml`.
- Tests: `tests/test_spec018.py`, and line 400 of `tests/test_spec009.py`.
- `src/train/README.md`.
- The report: `python -m tarok_train.flowb_report <run folders>`, and `--random-entropy` for criterion 8.

### Run log draft for `docs/progress.md` (D79)

- 2026-11-25 21:20 to 21:27, `smoke-b-mlp.toml`, engine 0.3.0, on the GPU: 200 updates in 419 s, 32,900 decisions played per second. Not disturbed. Folder `smoke-b-mlp-20261125-212012`. The flood ends by itself (0.1% valat, mean reward size 1.21).
- 2026-11-25 21:27, `smoke-b-transformer.toml`: out of memory in update 3 (the acting cache, 2,080 MiB, stayed allocated through the update step). Stopped (criterion 13). Batch 22, D155 (a).
- 2026-11-27 01:13 to 01:26, the same config after D155 (a): disturbed by a video render, stopped after 54 updates. Peak 2,871 MiB. Kept as the memory record.
- 2026-11-27 02:22 to 02:56, the same config: 200 updates in 2,062 s, 6,845 decisions played per second. Not disturbed. Peak 2,871 MiB, reserved 4,766 MiB. Folder `smoke-b-transformer-20261127-022207`. The flood ends by itself (0.3% valat, mean reward size 1.37).

### Tables (criterion 14)

From `flowb_report` over the two used runs.

#### smoke-b-mlp (`smoke-b-mlp-20261125-212012`), 200 updates

Loop checks: illegal actions 0; largest sum of a deal's four rewards 3.83e-06 with a largest reward of 160.4 (within the tolerance of D147); finished True, failed None, stopped by the time limit False.

| Tag | 1 | 5 | 10 | 25 | 50 | 100 | 150 | 200 |
|---|---|---|---|---|---|---|---|---|
| reward/abs_max | 160.400 | 160.400 | 28.600 | 6.000 | 5.000 | 10.400 | 24.000 | 11.733 |
| reward/abs_mean | 42.898 | 11.901 | 1.858 | 0.522 | 0.513 | 0.834 | 1.369 | 0.890 |
| reward/share_above_10 | 0.737 | 0.257 | 0.017 | 0.000 | 0.000 | 0.000 | 0.017 | 0.002 |
| round/announced_kralj_ultimo | 0.002 | 0.008 | 0.058 | 0.005 | 0.002 | 0.009 | 0.005 | 0.000 |
| round/announced_kralji | 0.994 | 0.922 | 0.669 | 0.043 | 0.009 | 0.011 | 0.026 | 0.007 |
| round/announced_pagat_ultimo | 0.773 | 0.562 | 0.161 | 0.016 | 0.004 | 0.180 | 0.229 | 0.156 |
| round/announced_trula | 0.992 | 0.930 | 0.661 | 0.064 | 0.032 | 0.099 | 0.357 | 0.213 |
| round/announced_valat | 0.992 | 0.354 | 0.018 | 0.000 | 0.000 | 0.000 | 0.000 | 0.000 |
| round/bonus_kontra | 0.953 | 0.803 | 0.418 | 0.033 | 0.011 | 0.071 | 0.187 | 0.073 |
| round/decision_share | 0.348 | 0.270 | 0.174 | 0.091 | 0.084 | 0.095 | 0.128 | 0.098 |
| round/decisions_per_seat_deal | 7.588 | 5.253 | 2.979 | 1.446 | 1.318 | 1.504 | 2.130 | 1.564 |
| round/game_kontra | 0.275 | 0.465 | 0.453 | 0.314 | 0.265 | 0.131 | 0.120 | 0.180 |
| round/game_mort | 0.428 | 0.176 | 0.018 | 0.007 | 0.005 | 0.014 | 0.097 | 0.012 |
| round/game_none | 0.016 | 0.127 | 0.412 | 0.627 | 0.671 | 0.771 | 0.681 | 0.715 |
| round/game_re | 0.020 | 0.057 | 0.060 | 0.033 | 0.051 | 0.081 | 0.095 | 0.080 |
| round/game_sub | 0.262 | 0.176 | 0.056 | 0.019 | 0.007 | 0.004 | 0.007 | 0.014 |
| deals/contract_dve | 0.002 | 0.002 | 0.016 | 0.000 | 0.135 | 0.898 | 0.891 | 0.948 |
| deals/contract_ena | 0.000 | 0.006 | 0.267 | 0.019 | 0.011 | 0.021 | 0.016 | 0.012 |
| deals/contract_solo_dve | 0.072 | 0.105 | 0.286 | 0.002 | 0.000 | 0.002 | 0.002 | 0.002 |
| deals/contract_solo_ena | 0.910 | 0.879 | 0.195 | 0.000 | 0.002 | 0.000 | 0.000 | 0.000 |
| deals/contract_solo_tri | 0.016 | 0.008 | 0.187 | 0.000 | 0.002 | 0.000 | 0.002 | 0.000 |
| deals/contract_tri | 0.000 | 0.000 | 0.051 | 0.979 | 0.851 | 0.079 | 0.090 | 0.038 |
| deals/declarer_wins | 0.455 | 0.287 | 0.171 | 0.494 | 0.525 | 0.690 | 0.679 | 0.704 |
| entropy/announcements | 0.916 | 0.929 | 0.818 | 0.397 | 0.224 | 0.308 | 0.309 | 0.260 |
| entropy/bidding | 0.860 | 0.915 | 0.871 | 0.035 | 0.145 | 0.223 | 0.183 | 0.165 |
| entropy/contract choice | 0.121 | 0.151 | 0.811 | 0.047 | 0.141 | 0.033 | 0.028 | 0.028 |
| entropy/discard | 2.427 | 2.313 | 2.285 | 1.890 | 1.345 | 0.893 | 0.832 | 0.827 |
| entropy/king call | 1.386 | 1.238 | 1.226 | 1.180 | 0.449 | 0.342 | 0.419 | 0.260 |
| entropy/play | 0.953 | 0.939 | 0.918 | 0.832 | 0.713 | 0.576 | 0.552 | 0.521 |
| entropy/talon pick | 1.723 | 1.670 | 1.184 | 0.664 | 0.637 | 0.592 | 0.536 | 1.002 |
| entropy_weight/announcements | 0.010 | 0.010 | 0.010 | 0.010 | 0.012 | 0.055 | 0.030 | 0.022 |
| entropy_weight/bidding | 0.010 | 0.010 | 0.010 | 0.081 | 0.159 | 0.010 | 0.020 | 0.024 |
| entropy_weight/contract choice | 0.010 | 0.010 | 0.010 | 0.010 | 0.010 | 0.010 | 0.028 | 0.012 |
| entropy_weight/discard | 0.010 | 0.010 | 0.010 | 0.010 | 0.010 | 0.010 | 0.010 | 0.010 |
| entropy_weight/king call | 0.010 | 0.010 | 0.010 | 0.010 | 0.010 | 0.081 | 0.043 | 0.084 |
| entropy_weight/play | 0.010 | 0.010 | 0.010 | 0.010 | 0.010 | 0.010 | 0.010 | 0.010 |
| entropy_weight/talon pick | 0.010 | 0.010 | 0.010 | 0.010 | 0.010 | 0.010 | 0.010 | 0.010 |
| gpu/allocated_after_acting_mib | 22 | 59 | 60 | 59 | 60 | 59 | 60 | 59 |
| gpu/allocated_after_update_mib | 59 | 60 | 60 | 59 | 59 | 59 | 60 | 59 |
| gpu/reserved_after_acting_mib | 50 | 566 | 566 | 566 | 568 | 568 | 568 | 568 |
| gpu/reserved_after_update_mib | 500 | 566 | 566 | 566 | 568 | 568 | 568 | 568 |
| time/seconds | 2.61 | 3.69 | 3.24 | 2.02 | 1.88 | 2.02 | 2.15 | 2.04 |

Valat announced in under 50%, 20%, 5% of deals from update on (criterion 15): 50%: 5, 20%: 6, 5%: 8.
Criterion 19, the flood ends by itself: holds. updates 151 to 200: valat announced in 0.1% of deals (limit 5%), mean reward size 1.213 (limit 2.0).

#### smoke-b-transformer (`smoke-b-transformer-20261127-022207`), 200 updates

Loop checks: illegal actions 0; largest sum of a deal's four rewards 3.83e-06 with a largest reward of 160.4 (within the tolerance of D147); finished True, failed None, stopped by the time limit False.

| Tag | 1 | 5 | 10 | 25 | 50 | 100 | 150 | 200 |
|---|---|---|---|---|---|---|---|---|
| reward/abs_max | 160.400 | 160.400 | 160.400 | 106.667 | 8.533 | 13.333 | 106.667 | 106.800 |
| reward/abs_mean | 48.230 | 29.447 | 5.032 | 3.285 | 0.613 | 1.048 | 1.064 | 1.620 |
| reward/share_above_10 | 0.783 | 0.515 | 0.090 | 0.069 | 0.000 | 0.005 | 0.010 | 0.021 |
| round/announced_kralj_ultimo | 0.002 | 0.000 | 0.000 | 0.239 | 0.000 | 0.012 | 0.025 | 0.005 |
| round/announced_kralji | 0.998 | 0.925 | 0.640 | 0.549 | 0.007 | 0.016 | 0.053 | 0.014 |
| round/announced_pagat_ultimo | 0.846 | 0.718 | 0.449 | 0.096 | 0.000 | 0.005 | 0.077 | 0.202 |
| round/announced_trula | 0.988 | 0.925 | 0.499 | 0.625 | 0.011 | 0.284 | 0.285 | 0.216 |
| round/announced_valat | 0.990 | 0.750 | 0.094 | 0.004 | 0.000 | 0.005 | 0.004 | 0.003 |
| round/bonus_kontra | 0.957 | 0.859 | 0.484 | 0.554 | 0.009 | 0.142 | 0.114 | 0.142 |
| round/decision_share | 0.366 | 0.317 | 0.194 | 0.244 | 0.082 | 0.112 | 0.098 | 0.111 |
| round/decisions_per_seat_deal | 8.176 | 6.591 | 3.433 | 4.657 | 1.285 | 1.837 | 1.555 | 1.796 |
| round/game_kontra | 0.209 | 0.342 | 0.372 | 0.379 | 0.212 | 0.311 | 0.073 | 0.034 |
| round/game_mort | 0.572 | 0.313 | 0.087 | 0.315 | 0.009 | 0.009 | 0.018 | 0.039 |
| round/game_none | 0.014 | 0.079 | 0.408 | 0.119 | 0.715 | 0.548 | 0.895 | 0.844 |
| round/game_re | 0.014 | 0.054 | 0.048 | 0.039 | 0.057 | 0.111 | 0.011 | 0.072 |
| round/game_sub | 0.191 | 0.211 | 0.085 | 0.149 | 0.007 | 0.021 | 0.004 | 0.010 |
| deals/contract_dve | 0.002 | 0.000 | 0.000 | 0.115 | 0.195 | 0.870 | 0.913 | 0.925 |
| deals/contract_ena | 0.000 | 0.000 | 0.000 | 0.065 | 0.060 | 0.078 | 0.011 | 0.015 |
| deals/contract_solo_dve | 0.027 | 0.025 | 0.021 | 0.009 | 0.002 | 0.005 | 0.000 | 0.002 |
| deals/contract_solo_ena | 0.961 | 0.973 | 0.971 | 0.014 | 0.004 | 0.003 | 0.002 | 0.000 |
| deals/contract_solo_tri | 0.010 | 0.002 | 0.008 | 0.037 | 0.000 | 0.007 | 0.000 | 0.003 |
| deals/contract_tri | 0.000 | 0.000 | 0.000 | 0.759 | 0.740 | 0.036 | 0.075 | 0.055 |
| deals/declarer_wins | 0.488 | 0.493 | 0.081 | 0.398 | 0.473 | 0.637 | 0.727 | 0.702 |
| entropy/announcements | 0.894 | 0.961 | 0.868 | 0.674 | 0.207 | 0.277 | 0.304 | 0.305 |
| entropy/bidding | 0.798 | 0.792 | 0.773 | 0.135 | 0.261 | 0.191 | 0.207 | 0.215 |
| entropy/contract choice | 0.068 | 0.048 | 0.046 | 0.432 | 0.258 | 0.036 | 0.031 | 0.049 |
| entropy/discard | 2.410 | 2.399 | 2.317 | 2.039 | 1.392 | 1.019 | 0.978 | 1.113 |
| entropy/king call | 1.373 | - | - | 1.311 | 0.556 | 0.324 | 0.686 | 0.419 |
| entropy/play | 0.949 | 0.947 | 0.913 | 0.900 | 0.787 | 0.723 | 0.785 | 0.777 |
| entropy/talon pick | 1.753 | 1.755 | 1.554 | 0.386 | 0.301 | 0.826 | 0.456 | 0.937 |
| entropy_weight/announcements | 0.010 | 0.010 | 0.010 | 0.010 | 0.014 | 0.034 | 0.013 | 0.010 |
| entropy_weight/bidding | 0.010 | 0.010 | 0.010 | 0.017 | 0.090 | 0.014 | 0.024 | 0.017 |
| entropy_weight/contract choice | 0.010 | 0.010 | 0.010 | 0.010 | 0.010 | 0.010 | 0.034 | 0.070 |
| entropy_weight/discard | 0.010 | 0.010 | 0.010 | 0.010 | 0.010 | 0.010 | 0.010 | 0.010 |
| entropy_weight/king call | 0.010 | 0.010 | 0.010 | 0.010 | 0.010 | 0.069 | 0.010 | 0.013 |
| entropy_weight/play | 0.010 | 0.010 | 0.010 | 0.010 | 0.010 | 0.010 | 0.010 | 0.010 |
| entropy_weight/talon pick | 0.010 | 0.010 | 0.010 | 0.013 | 0.031 | 0.010 | 0.010 | 0.010 |
| gpu/allocated_after_acting_mib | 2122 | 2189 | 2188 | 2189 | 2188 | 2189 | 2188 | 2189 |
| gpu/allocated_after_update_mib | 108 | 109 | 109 | 109 | 109 | 109 | 109 | 109 |
| gpu/reserved_after_acting_mib | 3914 | 4506 | 4506 | 4506 | 4766 | 4766 | 4766 | 4766 |
| gpu/reserved_after_update_mib | 3982 | 4506 | 4506 | 4506 | 4766 | 4766 | 4766 | 4766 |
| time/seconds | 10.50 | 11.66 | 10.62 | 12.01 | 9.14 | 10.33 | 10.82 | 10.86 |

Valat announced in under 50%, 20%, 5% of deals from update on (criterion 15): 50%: 7, 20%: 8, 5%: 14.
Criterion 19, the flood ends by itself: holds. updates 151 to 200: valat announced in 0.3% of deals (limit 5%), mean reward size 1.373 (limit 2.0).

### Suite

- CPU, GPU hidden (`CUDA_VISIBLE_DEVICES=-1` and `TAROK_REQUIRE_ENV=1`), after D155: 321 passed, 14 skipped (the GPU tests, CUDA not available) in 1:03:15. Output: `runs/spec018/suite-cpu-3.txt`. An earlier run with an empty `CUDA_VISIBLE_DEVICES` left the card visible, so it is not used.
- GPU tests (`-m gpu`, `TAROK_REQUIRE_ENV=1`), after D155, on a clean card (two free readings 5 minutes apart, no render): 14 passed, 321 deselected in 3:19. No render after the tests. Output: `runs/spec018/suite-gpu-3.txt`.
- Together: 335 tests, all pass.
