# 008 — Pipeline smoke run on deal flow A

Status: **approved** (2026-11-12, D124; open points A–E answered as recommended; D136: entropy targets and readings 2 and 3). Spec 006 is committed (D123). **Executed on 2026-11-12 in the `train-env` session: code, tests, review fixes and the three long runs. Not committed. Waiting for the user's read of the diff and for CI after a push.** See Results.

Build-plan row 008. Tool and model: Claude Code 2.1.306, `claude-fable-5-2`.

## Goal

Run the whole training loop once on the real engine: Rust tables, encoding, Python learner, GPU. Flow A only: six normal contracts, forehand plays tri if all pass. Both networks.

**It is a test of the pipeline, not a result.** It answers four things early:

1. Does the loop run to the end without an illegal action, an error or a NaN?
2. Are seats and rewards wired correctly? The trained agent must beat random play, and a run with the rewards given to the wrong seat must not.
3. How many steps per second does the whole loop reach (G3), and where does the time go?
4. Do the monitors of D107 work on real deals: bidding shares, entropy per phase?

## In scope

- `src/train/tarok_train/train.py`: the environment kind `"tarok"` (the wrapper `env.py` of spec 006) next to `"mock"`.
- Run configs: `src/train/configs/smoke-a-mlp.toml`, `smoke-a-transformer.toml`, and a small `smoke-a-ci.toml` for the tests.
- Monitors, logged per update to TensorBoard: policy entropy per phase (bidding, king call, talon, discards, card play); share of each contract and of forehand's forced tri; share of deals the declarer's side wins; decisions per seat-deal; steps per second played and trained; share of time in the environment, in acting, in bookkeeping and in the update. The losses, gradient norms, approximate KL, clip fraction and explained variance of spec 007 stay.
- `src/train/tarok_train/evaluate.py`: a trained checkpoint against random legal play on seeded deals, with a 95% interval.
- A fault switch for the control run: rewards handed to the next seat. Test-only; the config key is refused unless the run name starts with `control-`.
- The two runs and the control run on this PC with the GPU free. A run log in `docs/progress.md` (D79). Checkpoints stay in `src/train/runs/`, untracked: nothing is released.
- The entropy controller, with its own tests (criteria 16 and 17; D127).
- The stale-module check (D130, B12-15): the module carries a hash of the engine's sources from build time, and `EngineEnv` refuses to start when the files on disk differ. One test.

## Out of scope

Any claim about playing strength. Flows B and C. The rule bot and the arena (spec 011). Tuning beyond the settings named here. The Rust forward pass. Released weights and `MODEL.md`.

## Settings of the run (to be confirmed, open points A–C)

| Setting | Value | From |
|---|---|---|
| Tables | 512 | spec 007: the acting cache of 1,024 tables does not fit in 6 GB |
| Seat-deals per update | 2,048 (one deal of every table) | open point B; the mock wasted 61% of the play with this ratio, real deals are measured in criterion 9 |
| Updates per run | 200 | open point A; about 5 to 6 million trained decisions, **estimated** from 56.8 actions per deal (spec 005) |
| Time limit per run | 90 minutes | a run that hits it is reported as not finished |
| Value-head weight, policy first scale, entropy controller | 0.1, 0.3, on | D127 |
| Everything else | as `configs/mock.toml` | spec 007, D119 |

## Acceptance criteria

The user approves these before tests are written (D73).

**The loop**
1. `python -m tarok_train.train --config configs/smoke-a-mlp.toml` and the transformer config each run 200 updates on the real environment and end with exit code 0.
2. No illegal action: at every stored decision the chosen slot is legal in the mask, checked in the rollout; the environment refuses an illegal slot with an error. Both runs end with zero refusals.
3. No NaN and no infinity in any logged loss, gradient norm or reward.
4. Every deal's four rewards add up to zero within 1e-5 in the learner's storage, and the ten parts of each seat add up to its reward.
5. Every trained decision was played under the current weights (`rollout/decision_age_max` is 0 at every update).
6. A run stopped after a checkpoint and resumed with `--resume` reaches the same update count. The optimiser state, the update counter and the opponent pool come from the checkpoint; the tables start fresh deals. Tested on the small config.

**Wiring**
7. **The agent beats random.** After each run: 2,000 seeded deals, each played four times with the agent in a different seat and random legal play in the other three. The agent's mean reward per deal is above zero, and so is the lower end of the 95% interval (interval over the 2,000 deals, each deal's value the mean of its four seatings). The same numbers for the untrained network are reported next to it.
8. **The control fails.** A run of 200 updates with the MLP and the rewards handed to the next seat does not pass criterion 7: its interval is not wholly above zero. If it passes, criterion 7 proves nothing and the spec is not done.

**Measurements** (reported in `docs/progress.md`; no thresholds except where said)
9. End to end, per network, over the whole run: decisions played per second, decisions trained per second, share of played decisions that were trained on. Compared with the old G3 (10,000); the new figure of G3 is proposed from these numbers (D128, C13). GPU state before and after from `nvidia-smi`.
10. Time split per network: environment, acting, bookkeeping, update. The four shares add up to between 95% and 100% of the wall time.
11. Peak GPU memory per run. No out-of-memory error with 512 tables.
12. Decisions and tokens per seat-deal: mean and largest. The largest token count is at most 128.
13. Monitors at the first and the last update: entropy per phase, contract shares, share of forehand's forced tri, share of declarer wins. A phase whose entropy is under 0.05 at the end is listed as a collapse.
14. Accuracy of the partner head and the card-location head at the first and the last update, next to the accuracy of guessing uniformly over the places still possible.

**Record**
15. 100 of the evaluation deals of each run are exported as game records (spec 006) and replay to the same entries.

**Entropy controller** (D127)
16. With the controller on, the entropy of each phase stays within a band around its target on the mock task in `tokens` mode over 200 updates; with it off, the same run falls under the band (the collapse of spec 007). The band and targets are in the config.
17. The controller changes only the entropy weight. A unit test fixes its update rule on hand-computed numbers.

**Quality gates**
18. `pytest` passes here with the new tests; the `train` workflow is green. The CI test builds the environment module and runs two updates of `smoke-a-ci.toml` on the CPU.
19. Every setting of the three configs is in the file with its origin, as in `mock.toml`. The run log names the commit, the configs, the duration and what failed.

## Checks

Criteria 1–6, 16–18: pytest (the full runs of criterion 1 are the runs themselves; pytest covers the small config). Criteria 7, 8, 15: `python -m tarok_train.evaluate`, output in Results. Criteria 9–14: read from the TensorBoard logs by a small script and written into Results. Criterion 19: by reading.

## Files expected to change

New: `src/train/configs/smoke-a-mlp.toml`, `smoke-a-transformer.toml`, `smoke-a-ci.toml`; `src/train/tarok_train/evaluate.py`; `src/train/tests/test_spec008.py`.
Changed: `src/train/tarok_train/train.py`, `rollout.py`, `ppo.py` (monitors, the fault switch, the controller if chosen), `src/train/README.md`, `.github/workflows/train.yml` if the build step needs it, `docs/progress.md`, this spec.

One session (`train-env`), after spec 006 is committed. The three runs need the GPU free for about three hours in all.

## Commits

(a) the `"tarok"` environment kind, monitors, small config, tests 1–6 on the small config; (b) `evaluate.py`, the fault switch, tests for 7, 8, 15; (c) the controller, if chosen; (d) the run log and Results. Each with `Spec: docs/specs/008-pipeline-smoke-run.md`.

## Open points for the user

- **A. Length of each run.** (a) 200 updates, about 5 to 6 million trained decisions, recommended: by the mock's measured speed that is under an hour for the transformer, and 34 updates (one million decisions) is few for PPO to show anything. (b) one million trained decisions, as the build plan first said. (c) a fixed hour per network.
- **B. Seat-deals per update.** (a) 2,048 with 512 tables, one deal of every table, recommended to start: the share of wasted play is then measured (criterion 9) and the first real run is set from it. (b) 4,096: fewer, larger updates, less waste.
- **C. Which networks.** (a) both, recommended: the MLP is the control (D18) and the pair gives the G3 comparison of B11-7. (b) the MLP only, the transformer later.
- **D. The control run (criterion 8).** (a) keep it, recommended: the review of spec 007 showed a learning test that ten planted bugs passed. It costs one more MLP run. (b) drop it.
- **E. The opponent in criterion 7.** (a) random legal play in three seats, recommended: no bot exists before spec 011. (b) also the untrained network as an opponent.

## Answers to the open points (D124, 2026-11-12)

A: 200 updates per run. B: 2,048 seat-deals per update with 512 tables. C: both networks. D: the control run stays. E: random legal play in three seats.

Added with the approval: the settings of D127 (value-head weight 0.1, policy scale 0.3, the entropy controller) and the stale-module check of D130. The engine's rules change under spec 013 while this spec is built; the three long runs start only after spec 013 is verified, and the run log names the commit and any uncommitted diff the tree carried.

## Results

**State on 2026-11-12: everything of this spec is built and run.** The code, the configs and the tests; the fixes of the review pass; the two runs of 200 updates and the control run, their evaluations and reports. Both trained agents beat random play and the control run does not. Nothing is committed. Open: CI (criterion 18) after a push, and the copy of the run log into `docs/progress.md`, which is the main session's file.

### Criteria

The numbers of the runs are in "The three runs". The tests named here run on the small config `configs/smoke-a-ci.toml` (the settings of the smoke run with 16 tables and 32 seat-deals per update, on the CPU); they are in `src/train/tests/test_spec008.py`. The tests added after the review are listed under "Review fixes".

| # | State | Evidence |
|---|---|---|
| 1 | **met**: both runs ended after 200 updates with exit code 0 | `test_c01_c18_two_updates_of_the_ci_config_run_on_the_engine_and_end_with_exit_code_0` [mlp, transformer]; `test_c01_the_smoke_configs_carry_the_settings_of_the_spec`; `test_c01_the_environment_kind_decides_which_env_keys_a_config_needs`; `test_c01_a_run_that_hits_its_time_limit_is_reported_as_not_finished` |
| 2 | **met**: 0 refusals in each of the three runs | `test_c02_the_rollout_refuses_a_stored_action_that_its_mask_forbids`; `test_c02_the_engine_refuses_an_illegal_slot_and_a_run_ends_with_zero_refusals`. The count is logged as `env/illegal_actions` and written to `summary.json`. |
| 3 | **met**: 0 of 18,802 logged numbers of each run are not finite | `test_c03_every_logged_number_of_a_run_on_the_engine_is_finite`; `test_c03_a_logged_number_that_is_not_finite_stops_the_run` [nan, inf]. A run stops with an error at the first logged number that is not finite. |
| 4 | **met**: largest sum of a deal's rewards 1.2e-07 in each run | `test_c04_the_storage_refuses_a_deal_whose_four_rewards_do_not_add_up_to_zero`; `test_c04_on_the_engine_the_four_rewards_add_up_to_zero_and_the_ten_parts_to_the_reward`. On the engine the storage refuses a deal whose rewards do not add up to zero within 1e-5, so a run that ends has passed the check at every deal. The largest sum is also logged (`reward/deal_sum_abs_max`). |
| 5 | **met**: age 0 at every update of each run | `test_c05_the_decision_age_is_zero_at_every_update_also_with_a_past_checkpoint_in_play` |
| 6 | met; reading 2 accepted by the user (D136) | `test_c06_a_resumed_run_reaches_the_same_update_count_from_the_checkpoints_state` |
| 7 | **met**: MLP +0.598 (+0.590 to +0.606), transformer +0.605 (+0.597 to +0.613) | `test_c07_the_interval_is_the_mean_and_1_96_standard_errors_on_each_side`; `test_c07_each_deal_is_played_four_times_with_the_agent_in_another_seat`; `test_c07_the_agent_gets_the_reward_of_its_own_seat`; `test_c07_the_network_agent_samples_legal_actions_for_its_seat` [mlp, transformer]; `test_c07_c15_a_checkpoint_and_the_untrained_network_are_evaluated_from_the_command_line` |
| 8 | **met**: the control run's interval is −0.239 to −0.184, wholly under zero | `test_c08_the_fault_key_is_refused_unless_the_run_name_starts_with_control`; `test_c08_the_fault_hands_the_reward_of_every_seat_to_the_next_seat`; `test_c08_the_control_run_is_started_from_the_command_line` |
| 9 | reported below: MLP 34,743 played and 17,050 trained decisions per second, transformer 6,506 and 3,194 | `test_c09_played_and_trained_decisions_per_second_are_logged`; `test_c09_to_c14_the_report_script_reads_the_numbers_from_a_run_folder` |
| 10 | **met**: the four shares add up to 99.4% (MLP) and 99.9% (transformer) | `test_c10_the_four_time_shares_add_up_to_95_to_100_percent_of_the_wall_time` |
| 11 | **met**, no out-of-memory error; the transformer run is close to the card's limit (below) | `test_c11_the_peak_gpu_memory_is_in_the_summary` (on the CPU the entry is empty; the GPU number comes with the runs) |
| 12 | **met**: largest token count 67 | `test_c12_decisions_and_tokens_per_seat_deal_mean_and_largest` |
| 13 | reported below; by the fixed limit one phase is listed (MLP, contract choice, which sits on its approved target) | `test_c13_the_names_of_the_bids_are_those_of_the_contract`; `test_c13_deal_monitors_of_a_hand_built_batch`; `test_c13_the_monitors_of_d107_are_logged_on_real_deals` |
| 14 | reported below | `test_c14_the_guessing_level_of_hand_built_states`; `test_c14_on_real_deals_the_true_place_is_always_among_the_possible_ones`; `test_c14_the_accuracy_of_both_heads_is_logged_next_to_the_guessing_level` |
| 15 | **met**: 100 of 100 records of each of the three runs replay to the same entries | `test_c15_exported_records_replay_to_the_same_entries`; the command-line test of criterion 7 |
| 16 | met in the reading of point 3 (from update 61 on), accepted by the user (D136) | `test_c16_with_the_controller_the_entropy_stays_in_its_band_and_without_it_falls_under` (marked `slow`) |
| 17 | met | `test_c17_the_update_rule_on_hand_computed_numbers`; `test_c17_a_controller_that_is_off_never_changes_a_weight`; `test_c17_the_controller_changes_only_the_entropy_weight`; `test_c17_the_config_names_a_target_for_every_phase_and_the_band` |
| 18 | met locally; **CI waits for a push** | pytest output below. `train.yml` is unchanged: it already builds the module and runs `python -m pytest -m "not slow"`, which now includes the two updates of `smoke-a-ci.toml` for both networks. |
| 19 | configs met; the run log is in "The three runs" below, for the main session to copy into `docs/progress.md` | `test_c19_every_setting_of_the_three_configs_has_its_origin_in_the_file`; `test_c19_the_run_folder_names_the_commit_the_state_of_the_tree_and_the_config`. `docs/progress.md` is the main session's file; the numbers for it will be in this section. |
| D130 | met | `test_d130_a_module_built_from_other_engine_sources_is_refused` |

### What was built

- **`train.py`.** The config's `[env]` table names the environment: `kind = "mock"` or `kind = "engine"`, each with its own keys. New keys in `[run]`: `time_limit_minutes`, `fault`. New table `[entropy_control]`. The time of an update is split over four clocks. A run from the command line writes `run-info.json` at its start (commit from `git rev-parse HEAD`, `git status --short`, the config as it ran, the fingerprint of the engine's sources) and `summary.json` at its end, also when it fails. New options: `--name`, `--fault`, `--time-limit-minutes`.
- **`rollout.py`.** `add` refuses a decision whose action is not legal in its mask. With `zero_sum` (the engine) `finish` refuses a deal whose four rewards do not add up to zero within 1e-5.
- **`ppo.py`.** The monitors (`deal_stats`, the lengths of seat-deals, `possible_places`, `possible_partners`, the accuracy of the card-location head) and the `EntropyController`.
- **`evaluate.py`** (new). A checkpoint or the untrained network against random legal play; the 95% interval; game records exported and replayed.
- **`smoke_report.py`** (new). Reads a run folder (TensorBoard scalars, `run-info.json`, `summary.json`, `evaluation-*.json`) and prints the numbers of criteria 2 to 5 and 9 to 14 as Markdown. The spec's "small script".
- **`env.py` and `tarok_env/build.rs`** (new). The stale-module check of D130, and three flags on `EngineEnv` that the trainer reads (`zero_sum`, `real_deals`, `reward_is_score`).
- **Configs.** `smoke-a-mlp.toml`, `smoke-a-transformer.toml` (they differ in the network only), `smoke-a-ci.toml`. `mock.toml` got the new keys; its values of spec 007 are unchanged (value-head weight 0.5, policy scale 1.0, controller off).
- **`contract.py`.** The names of the thirteen bids, the four roles of block B4 and the blocks the monitors read. A test compares the bid names with `docs/encoding.md`.
- **`README.md`** of `src/train`: the commands of the smoke run, the evaluation and the report, the run files, the monitors, the controller, the stale-module check.

### Readings: where I read the spec differently, or had to choose

Reading 1 is confirmed by the main session. Readings 2 and 3 are accepted by the user (D136). The others stand as written; none was objected to in the review.

1. **The environment kind is `"engine"`, not `"tarok"`.** Spec 006 built and committed `env.make` with the kinds `"mock"` and `"engine"`, and its tests and README use that name. I kept it. Confirmed by the main session.
2. **Criterion 6, "the tables start fresh deals".** Not built that way. `EngineEnv` saves and restores its state (spec 006), and spec 006's own test asserts that a resumed trainer on the engine equals an unbroken one. The resume restores the tables exactly where they stood. Everything else of the criterion holds and is tested: the same update count, the optimiser state, the update counter and the opponent pool come from the checkpoint. On the CPU the resumed run gives the same weights as the unbroken one. If fresh deals are wanted (for example to resume after a rule change), it is a small change. **Accepted by the user (D136).**
3. **Criterion 16, "stays within a band over 200 updates".** An untrained policy is near uniform (entropy ln 4 = 1.39 on the mock), far above the band, and the controller needs some updates to find the weight each phase needs. The test reads the criterion as: from update 61 to update 200 the entropy of every phase is inside its band at every update. The first 60 updates are not inside it (numbers below). **Accepted by the user (D136).**
4. **The fault key.** The code has no defaults, so `fault` is a key of `[run]` in every config, with the value `"none"`. Any other value is refused unless the run's name starts with `control-`. The control run is the MLP config started with `--name control-smoke-a --fault rewards_to_next_seat`; there is no fourth config file.
5. **"Forehand's forced tri."** Two numbers are logged. `deals/forehand_forced`: the three other seats passed before forehand spoke (read from the first three bid tokens). `deals/forehand_forced_tri`: the contract played is tri. In flow A tri can only be played by a forced forehand, so this is the share the spec asks for. A forced forehand may also name a higher contract.
6. **"The declarer's side wins."** Read from the score parts: the declarer side's `game` part is positive, or its `valat` part where a valat replaces the game. Counted once per deal.
7. **"Guessing uniformly over the places still possible" (criterion 14).** Card location: a seat is possible unless block F2 says it cannot hold the card. "Out of play" is possible before the talon is turned up, and in the play for a seat that is not the declarer if the card is a suit card below the king (a discarded tarok is shown, kings are never discarded). Partner seat: the called partner has one possible seat (the declarer), an opponent the seats that are not the declarer's, the declarer all three, and once the partner is public (B6) everyone has one. A test checks on real deals that the true place is always among the possible ones (more than 100,000 unseen cards and more than 300 decisions with a partner in each test run; the test asserts those counts).
8. **The evaluation.** The agent samples from its policy, as in training; it does not take the most likely action. The 2,000 deals are played in chunks of 500 tables (the acting cache of the transformer needs about 1.2 GB for 500); chunk c uses the engine seed `seed + c`. No chunk may use the run's seed, or its deals would be training deals; the code refuses such a call (review fix A1). The records are the first 100 deals with the agent in seat 0.
9. **The time limit.** A run that passes `time_limit_minutes` stops after the update in hand, writes a checkpoint and `summary.json` with `finished: false`, prints "not finished" and ends with exit code 3.
10. **Criterion 4 in a run.** There is no separate pass over the storage at the end of a run. The storage refuses every deal that fails the check, so the run itself is the check; the largest sum is also logged per update.
11. **Old checkpoints.** A checkpoint of spec 007 made before this change cannot be resumed: its config lacks the new keys. The runs of spec 007 are finished, so nothing is lost.

### Settings: what is decided and what is my proposal

- **The target entropy of each phase on the real game: approved for the smoke run by the user (D136)**, as proposed: bidding 0.2, contract choice 0.03, king call 0.35, talon pick 0.45, discard 0.6, play 0.25, each about a quarter of the entropy of random legal play in that phase (measured on the engine, 35,832 deals: 0.87, 0.11, 1.39, 1.74, 2.43, 0.95 nats). The configs name D136 as the origin. The target of the announcements (0.25, as play) is my choice: flow A has no announcements, but every phase needs a value. What the targets do in a run: the lower limit of a weight is the start weight 0.01, so while a phase's entropy is above its target the run is the run without a controller. Only when a phase falls under its target does its weight rise.
- **The controller's rule and its numbers** (gain 0.3, proportional factor 1.5, weights between 0.01 and 1.0, band 75% of the target), tuned on the mock (below). The spec names a controller; the rule is my design.
- **Engine threads: 6**, one per core. **Checkpoints every 50 updates.** **Seed 8.** Speed and bookkeeping only.

### The entropy controller (criteria 16 and 17)

Rule, after every update, for each phase that had decisions in it:

```
error    = (target - entropy) / target, cut to [-1, 1]
integral <- integral * exp(gain * error),         kept inside [min_weight, max_weight]
weight    = integral * exp(proportional * error), kept inside [min_weight, max_weight]
```

One weight per phase; every weight and integral starts at `[ppo] entropy_weight`. The integral moves slowly and keeps the level a phase needs. The proportional part answers at once and fades when the entropy is back. In the loss each decision's entropy counts with the weight of its phase. Nothing else changes: with equal weights the loss is the loss without a controller (tested), and with `enabled = false` the code path of spec 007 runs unchanged. The weights are in the checkpoint.

**How the rule was found.** All runs: the transformer on the mock in mode `tokens`, target 0.4 nats in every phase (a uniform policy over the four legal slots has 1.39), GPU.

| Rule and numbers | Run | What the entropy per phase did |
|---|---|---|
| Integral only (`weight <- weight * exp(gain * error)`), gain 0.3, weights from 0.001 to 1.0 | 60 updates, 64 tables | Swings. The weight fell to 0.001 while the entropy was high, the entropy then dropped to 0.04, the weight climbed too late and overshot: entropy between 0.04 and 1.03. Dropped. |
| Integral and proportional part, gain 0.3, proportional 1.5, weights from 0.01 | 200 updates, 64 tables | No swing. Updates 61 to 200: mean 0.39 in every phase, single updates from 0.18 to 0.67. |
| The same | 200 updates, 128 tables (in the test) | One value of 840 at 0.724. |
| Gain 0.3, proportional 1.0 | 200 updates, 128 tables | Updates 61 to 200: 0.23 to 0.62. |
| **Gain 0.2, proportional 1.0** (in the configs) | 200 updates, 128 tables, three runs | Updates 61 to 200: 0.25 to 0.61, 0.23 to 0.59 and the run below. |

The band is set from this spread: 75% of the target, so 0.10 to 0.70 for a target of 0.4. A band of 50% (0.20 to 0.60) would hold the means but not every single update.

**The test of criterion 16**, last run (2026-11-12, GPU free, 128 tables, 512 seat-deals per update, 200 updates with the controller and 200 without):

| Phase | With the controller, updates 61 to 200 | Without, last 50 updates |
|---|---|---|
| bidding | 0.252 to 0.532, mean 0.390 | mean 0.005, at most 0.033 |
| king call | 0.253 to 0.563, mean 0.387 | mean 0.012, at most 0.043 |
| talon pick | 0.244 to 0.609, mean 0.395 | mean 0.008, at most 0.023 |
| discard | 0.275 to 0.526, mean 0.389 | mean 0.008, at most 0.060 |
| announcements | 0.273 to 0.523, mean 0.390 | mean 0.012, at most 0.030 |
| play | 0.275 to 0.493, mean 0.389 | mean 0.003, at most 0.016 |

The test asserts: with the controller every value of updates 61 to 200 is inside the band and the mean is within 10% of the target; without it the mean of the last 50 updates is under half the band's lower edge and nine of ten single updates are under the edge (in the run before this one a single update without the controller reached 0.091).

**What the first 60 updates look like** (gain 0.2, proportional 1.0): the entropy falls from 1.3 to about 0.1 by update 21 (0.06 to 0.15 by phase), is back between 0.2 and 0.3 at update 31 and near 0.4 from update 41 on. So for about 15 updates the entropy is under the band before the weights have risen from 0.01 to the 0.1 to 0.3 that this task needs.

**A finding beside the criterion: the controller lifts the plateau of spec 007 (Finding 2).** Task score, mean of the last 20 updates: 0.896 with the controller, 0.808 without (the plateau at 0.81 where the transformer follows the cue token and ignores the state hint). With the entropy held near 0.4 the policy keeps trying the other slots at the hint decisions and learns them. It was the same in all three runs with these numbers (0.894, 0.894, 0.896). The score is still rising at update 200.

**Limits.** Tuned on one task, with four legal slots at every decision and a large advantage signal. On the real game the weight a phase needs is unknown, a phase such as the king call has few decisions per update, and many decisions have one legal slot (entropy 0). The smoke run will show the weights and the entropies per phase (criterion 13); the numbers here may need changing after it.

### The stale-module check (D130)

`tarok_env/build.rs` writes a fingerprint of the sources at build time: one line per file (path, size, CRC-32) for every `.rs` file under `src` of the engine crate and of the binding crate, both `Cargo.toml`, the binding's `Cargo.lock` and `build.rs`; and an FNV-1a hash of those lines. The module exposes them as `SOURCE_MANIFEST` and `SOURCE_HASH`. `EngineEnv` computes the same text from the files on disk when it starts and raises `StaleModuleError` if the hash differs. The message names the changed, new and missing files and says how to build again. Where the sources are not on disk nothing is compared. The engine's tests, benches and `target` are not counted.

The build script uses the standard library only, has no `unsafe` and knows no rule. `cargo fmt --check` and `cargo clippy --locked --all-targets -- -D warnings` pass for the binding crate with Rust 1.94.1.

The check fired twice during this session, both times rightly. The engine session changed sources for spec 013 while I worked; a run of the suite then failed in every engine test, and later the pre-flight refused to start with "changed: engine/src/game.rs, engine/src/record.rs, engine/src/score.rs". After `maturin develop --release` both ran.

### Review fixes (2026-11-12, after the D72 step 4 review)

The review found no defect that corrupts training or evaluation, seven lesser defects in reporting and checks (A), and 20 of 47 planted bugs that `test_spec008.py` let through (B). It also asked for two more monitors (C). All items are worked in. The numbers of the three runs below were made after these fixes.

**A. Defects**

1. **Evaluation seed.** The chunks of an evaluation use the engine seeds `seed`, `seed + 1`, and so on (`evaluate.chunk_seeds`). The check was on the first seed only. Now no chunk may use the seed of the training run: `evaluate.evaluate` raises and the command ends with exit code 2 and a message. With run seed 8, `--seed 5 --deals 2000` (chunks 5 to 8) is refused; the runs below use `--seed 2008` (chunks 2008 to 2011).
2. **Collapse in the report.** `smoke_report` prints two lists, each named: phases under the spec's fixed 0.05 at the last update, and phases under the lower edge of their own band (target times 0.25 with the band of 75%). The approved target of "contract choice" is 0.03, under the fixed limit, so that phase can be on the first list while it sits on its target.
3. **One definition of the trained share.** `rollout/trained_share` and `trained_share` in `summary.json` are trained decisions / played decisions, where played counts every seat, also the seats of a past checkpoint. This is the criterion's "share of played decisions that were trained on". The other number is logged under its own name: `rollout/trained_share_own_seats`, trained / decisions of the current network's seats. `rollout/played` and `rollout/played_own_seats` are logged too. The README names both. Spec 007's 39% on the mock was the second kind.
4. **Deal monitors in the control run.** The storage keeps what the environment paid (`true_parts`) next to the parts the learner gets. `deals/declarer_wins` and `reward/deal_sum_abs_max` read the true parts, so they are right under the fault.
5. **Summary of a failed run.** `updates` counts the updates that were logged; an update that fails the check of its logged numbers is not counted, and neither are its decisions. Played, trained and the shares of `summary.json` come from one function (`train.run_summary`) over the logged updates.
6. **A module built before D130** raises `StaleModuleError` with the rebuild hint, not an `AttributeError`.
7. **GPU memory, three numbers.** `peak_gpu_memory_mib`: the largest sum of the run's tensors (`torch.cuda.max_memory_allocated`). `peak_gpu_memory_reserved_mib`: the most PyTorch's allocator held from the card (`max_memory_reserved`); this is the one that must fit. `gpu_after.memory_used_mib`: what `nvidia-smi` shows for the whole card, every program on it, while the run still holds its memory. All three are in `summary.json` and in the report; the first two are also logged per update.

One more change, for item M11 of part B: **the version of a stored decision is now the number of updates done when its deal began**, not when the decision was taken. `rollout/decision_age_max` then shows 1 for every decision of a deal that began under older weights, which the earlier stamp could not show: the broken loop stores only the later decisions of such a deal, and those were taken under the new weights. The checkpoint holds the number per table.

**B. Tests**

New tests, each aimed at the planted bugs named with it (the letters and numbers are the review's):

| Test | Planted bugs it is for |
|---|---|
| `test_c04_c08_in_a_normal_run_every_stored_seat_has_its_own_decisions_and_the_reward_the_engine_paid_it` | N07, M02, M12, M01, M11. A spy on the engine records every action, seat and payment; the storage must hold exactly that, with past checkpoints in play and deals that straddle an update. |
| `test_c02_one_illegal_row_among_legal_ones_is_refused` | M06 |
| `test_c05_a_deal_that_began_under_older_weights_shows_an_age_above_zero` | M11: the criterion's monitor can now fail |
| `test_c07_the_interval_is_taken_over_the_deals_not_over_the_single_seatings` | M14 |
| `test_c07_the_trained_evaluation_plays_the_checkpoints_weights_and_untrained_the_first_weights` | M16 |
| `test_c07_random_legal_play_is_uniform_over_the_legal_slots` | N01 |
| `test_c07_no_chunk_of_the_evaluation_may_use_the_seed_of_the_training_run` | M18, defect A1 |
| `test_c17_the_controller_is_connected_to_the_trainer` | M23, N02, N22. Fast (three updates on the mock): the weights the loss gets are the weights the controller holds, they follow the rule from the logged entropy, and with the controller off the loss gets none. |
| `test_c06_the_weights_of_the_entropy_controller_come_back_from_a_checkpoint` | M24: a checkpoint whose weights differ from the start |
| `test_c01_the_time_limit_of_the_config_reaches_the_loop_in_seconds` | N14 |
| `test_c09_played_trained_and_the_two_shares_are_hand_counted_with_a_past_checkpoint_in_play` | M28, M35: counted from the spy, one decision per table and step |
| `test_c09_the_summary_of_a_run_on_hand_computed_numbers` | N27, M29 |
| `test_c09_the_summary_of_a_failed_run_counts_only_the_updates_that_were_logged` | defect A5 |
| `test_c10_each_clock_takes_the_time_of_its_own_work` | M30: a slow environment, a slow actor, a slow storage and a slow update; each clock must hold its own delay and nothing near another's |
| `test_c14_at_the_talon_pick_and_the_discard_no_unseen_card_is_out_of_play` | M32 |
| `test_c14_the_guessing_level_counts_the_seats_still_possible_for_each_decision` | M33 |
| `test_c08_under_the_fault_the_deal_monitors_still_read_the_true_rewards` | defect A4 |
| `test_c13_entropy_over_the_decisions_with_a_choice_and_the_share_of_forced_decisions` | part C |
| `test_c13_the_report_names_a_collapse_by_the_fixed_limit_and_one_relative_to_the_target` | defect A2 |
| `test_c11_on_the_gpu_the_summary_holds_the_peak_of_the_tensors_the_reserved_peak_and_nvidia_smi` (GPU) | defect A7 |
| `test_d130_a_module_built_before_the_fingerprint_existed_is_refused` | defect A6 |

No existing test was loosened. Three existing tests were extended (the summary's keys and its own arithmetic, the report's lines, the TensorBoard tags).

**Planted bugs.** Each edit was made in a scratch copy of the tree (the repo was not touched), the short tests of `test_spec008.py` were run against the copy (`-m "not slow"`, GPU hidden), and the copy was restored. Without a bug the copy passes: 62 passed, 1 skipped (the GPU test). 26 planted, 26 caught. The 22 the review named are first; the last four put back defects of part A and a wrong count in part C.

| Bug | Edit | Tests of `test_spec008.py` that fail |
|---|---|---|
| N07 | The fault is always on: rewards and parts are rolled in a normal run (`train.py`) | `test_c08_the_fault_hands_the_reward_of_every_seat_to_the_next_seat`; `test_c04_c08_in_a_normal_run_every_stored_seat_has_its_own_decisions_and_the_reward_the_engine_paid_it`; `test_c08_under_the_fault_the_deal_monitors_still_read_the_true_rewards` |
| M02 | Decisions are stored under the next seat (`train.py`) | `test_c04_c08_in_a_normal_run_every_stored_seat_has_its_own_decisions_and_the_reward_the_engine_paid_it`; `test_c09_played_trained_and_the_two_shares_are_hand_counted_with_a_past_checkpoint_in_play` |
| M12 | Seats of the past checkpoint are stored (`train.py`) | `test_c04_c08_in_a_normal_run_every_stored_seat_has_its_own_decisions_and_the_reward_the_engine_paid_it` |
| M06 | The storage refuses a batch only if every row is illegal (`rollout.py`) | `test_c02_one_illegal_row_among_legal_ones_is_refused` |
| M01 | A seat-deal gets the next seat's reward and parts (`rollout.py`) | `test_c04_c08_in_a_normal_run_every_stored_seat_has_its_own_decisions_and_the_reward_the_engine_paid_it`; `test_c08_under_the_fault_the_deal_monitors_still_read_the_true_rewards` |
| M11 | After an update every table counts as fresh (`train.py`) | `test_c01_c18_two_updates_of_the_ci_config_run_on_the_engine_and_end_with_exit_code_0[mlp]`; `test_c01_c18_two_updates_of_the_ci_config_run_on_the_engine_and_end_with_exit_code_0[transformer]`; `test_c05_the_decision_age_is_zero_at_every_update_also_with_a_past_checkpoint_in_play`; `test_c09_to_c14_the_report_script_reads_the_numbers_from_a_run_folder`; `test_c12_decisions_and_tokens_per_seat_deal_mean_and_largest`; `test_c13_the_monitors_of_d107_are_logged_on_real_deals`; `test_c04_c08_in_a_normal_run_every_stored_seat_has_its_own_decisions_and_the_reward_the_engine_paid_it`; `test_c05_a_deal_that_began_under_older_weights_shows_an_age_above_zero`; `test_c07_the_trained_evaluation_plays_the_checkpoints_weights_and_untrained_the_first_weights`; `test_c09_played_trained_and_the_two_shares_are_hand_counted_with_a_past_checkpoint_in_play`; `test_c13_entropy_over_the_decisions_with_a_choice_and_the_share_of_forced_decisions` |
| M14 | The interval is taken over the single seatings (`evaluate.py`) | `test_c07_the_interval_is_taken_over_the_deals_not_over_the_single_seatings` |
| M16 | The trained evaluation plays untrained weights and the other way round (`evaluate.py`) | `test_c07_the_trained_evaluation_plays_the_checkpoints_weights_and_untrained_the_first_weights` |
| N01 | The opponents play the first legal slot, not a random one (`evaluate.py`) | `test_c07_random_legal_play_is_uniform_over_the_legal_slots` |
| M18 | The check of the evaluation seed is off (`evaluate.py`) | `test_c07_no_chunk_of_the_evaluation_may_use_the_seed_of_the_training_run` |
| M23 | The loss always gets the start weights of the entropy (`train.py`) | `test_c17_the_controller_is_connected_to_the_trainer` |
| N02 | The controller is never stepped (`train.py`) | `test_c06_the_weights_of_the_entropy_controller_come_back_from_a_checkpoint`; `test_c17_the_controller_is_connected_to_the_trainer` |
| N22 | The controller hands the integral to the loss, not the weight (`ppo.py`) | `test_c17_the_controller_is_connected_to_the_trainer` |
| M24 | A checkpoint's controller state is not loaded (`train.py`) | `test_c06_the_weights_of_the_entropy_controller_come_back_from_a_checkpoint` |
| N14 | The time limit is read as seconds (`train.py`) | `test_c01_the_time_limit_of_the_config_reaches_the_loop_in_seconds` |
| M28 | The trained share of an update divides by the own seats' decisions (`train.py`) | `test_c09_played_trained_and_the_two_shares_are_hand_counted_with_a_past_checkpoint_in_play` |
| N27 | The trained share of the summary divides by the own seats' decisions (`train.py`) | `test_c09_the_summary_of_a_run_on_hand_computed_numbers` |
| M29 | Played per second of the summary is trained per second (`train.py`) | `test_c11_the_peak_gpu_memory_is_in_the_summary`; `test_c09_the_summary_of_a_run_on_hand_computed_numbers` |
| M35 | Decisions of the past checkpoint are counted twice (`train.py`) | `test_c09_played_trained_and_the_two_shares_are_hand_counted_with_a_past_checkpoint_in_play` |
| M30 | The acting clock runs until after the environment's step (`train.py`) | `test_c10_each_clock_takes_the_time_of_its_own_work` |
| M32 | Out of play counts as possible at the talon pick (`ppo.py`) | `test_c14_at_the_talon_pick_and_the_discard_no_unseen_card_is_out_of_play` |
| M33 | The partner's guessing level counts one possible seat everywhere (`ppo.py`) | `test_c14_the_guessing_level_counts_the_seats_still_possible_for_each_decision` |
| A4 | The deal monitors read the parts after the fault (`ppo.py`) | `test_c08_under_the_fault_the_deal_monitors_still_read_the_true_rewards` |
| A5 | The summary counts an update that failed (`train.py`) | `test_c09_the_summary_of_a_failed_run_counts_only_the_updates_that_were_logged` |
| A6 | A module without a fingerprint is not handled (`env.py`) | `test_d130_a_module_built_before_the_fingerprint_existed_is_refused` |
| C1 | Forced decisions count as decisions with a choice (`ppo.py`) | `test_c13_entropy_over_the_decisions_with_a_choice_and_the_share_of_forced_decisions` |

The other planted bugs of the review, which the first version of the tests caught, were not planted again.

**C. Monitors for a later decision** (the controller's rule is unchanged)

- Logged per phase next to `entropy/<phase>`: `entropy_choice/<phase>`, the same mean over the decisions with two or more legal slots, and `forced_share/<phase>`, the share of decisions with one legal slot. A forced decision has entropy 0, so entropy = entropy_choice x (1 - forced share); a test checks that identity on real deals. The report prints all three at the first and the last update. The controller still steers on `entropy/<phase>`.
- **Open issues from the review, noted here and in "Open issues":** the king call is rare under random play (about 22 decisions in 6,912 random deals; in the two runs every update had king calls once the policy stopped bidding solo); under the fault two of the three opponents of a solo keep their own reward, so the control corrupts about half the signal, not all of it; `possible_places` ignores block F3 (a card another seat certainly holds), which makes the guessing level a little too low.

**Test counts after the review fixes.** `test_spec008.py`: 64 tests (62 not slow and not GPU, 1 GPU, 1 slow). Whole suite, `python -m pytest -m "not slow"`, GPU hidden: 153 passed, 6 skipped (GPU), 6 deselected (slow), 152 s. `python -m pytest -m "slow or gpu" -s`, GPU free: 12 passed, 689 s. The spec 006 test that failed on the idle PC before passed in both of these runs. The controller's numbers of the last slow run: entropy from update 61 on between 0.20 and 0.60 in every phase (means 0.385 to 0.395), task score 0.893 with the controller and 0.808 without.

### Pre-flight of the real configs (not one of the three runs)

Three updates of each smoke config with 512 tables, to see that they start, fit in memory and how long 200 updates will take. 2026-11-12, GPU free before the run (0% busy, 518 MiB), engine sources as they stood in the tree with spec 013 unfinished. The folders are in my scratch folder, not in `runs`.

| | Transformer | MLP |
|---|---|---|
| Decisions played per second, updates 2 and 3 | 7,868 and 7,504 | 35,915 and 33,537 |
| Decisions played per second, all three updates (the first includes start-up) | 6,735 | 27,531 |
| Decisions trained per second, all three | 3,961 | 16,454 |
| Share of played decisions trained on, updates 2 and 3 | 49%, 50% | 50%, 51% |
| Time split: environment, acting, bookkeeping, update | 0%, 30%, 6%, 63% | 1%, 18%, 31%, 49% |
| The four shares together | 99.9% | 99.4% |
| Peak GPU memory | 3,918 MiB | 254 MiB |
| Illegal actions, oldest trained decision | 0, age 0 | 0, age 0 |

Read with care: three updates, the first update of a run trains on 93 to 95% of its play (every table starts a deal at once), and the policy is untrained. The numbers of criterion 9 come from the 200-update runs. As a first sign for G3 (D128): the transformer is near 7,500 played decisions per second end to end, under the old 10,000; the MLP is far above it.

### pytest output

The runs below are those of the first hand-over, before the review. The counts after the review fixes are at the end of "Review fixes".

All on 2026-11-12, in `src\train`, with `TAROK_REQUIRE_ENV=1`.

**1. On the final engine sources of spec 013.** The module was built again after the main session reported spec 013 verified: engine 0.2.0, fingerprint `64a6157fb05f43c4`, equal to the files on disk; HEAD was `3347cb6` with the work of spec 013 still uncommitted; it was committed right after as `ce175f6`, and the fingerprint still equals the files on disk. `python -m pytest tests\test_spec006.py tests\test_spec008.py -m "not slow" -v`, GPU hidden. The lines of this spec and the total:

```
tests/test_spec008.py::test_c01_c18_two_updates_of_the_ci_config_run_on_the_engine_and_end_with_exit_code_0[mlp] PASSED
tests/test_spec008.py::test_c01_c18_two_updates_of_the_ci_config_run_on_the_engine_and_end_with_exit_code_0[transformer] PASSED
tests/test_spec008.py::test_c01_the_smoke_configs_carry_the_settings_of_the_spec PASSED
tests/test_spec008.py::test_c01_the_environment_kind_decides_which_env_keys_a_config_needs PASSED
tests/test_spec008.py::test_c01_a_run_that_hits_its_time_limit_is_reported_as_not_finished PASSED
tests/test_spec008.py::test_c02_the_rollout_refuses_a_stored_action_that_its_mask_forbids PASSED
tests/test_spec008.py::test_c02_the_engine_refuses_an_illegal_slot_and_a_run_ends_with_zero_refusals PASSED
tests/test_spec008.py::test_c03_every_logged_number_of_a_run_on_the_engine_is_finite PASSED
tests/test_spec008.py::test_c03_a_logged_number_that_is_not_finite_stops_the_run[nan] PASSED
tests/test_spec008.py::test_c03_a_logged_number_that_is_not_finite_stops_the_run[inf] PASSED
tests/test_spec008.py::test_c04_the_storage_refuses_a_deal_whose_four_rewards_do_not_add_up_to_zero PASSED
tests/test_spec008.py::test_c04_on_the_engine_the_four_rewards_add_up_to_zero_and_the_ten_parts_to_the_reward PASSED
tests/test_spec008.py::test_c05_the_decision_age_is_zero_at_every_update_also_with_a_past_checkpoint_in_play PASSED
tests/test_spec008.py::test_c06_a_resumed_run_reaches_the_same_update_count_from_the_checkpoints_state PASSED
tests/test_spec008.py::test_c07_the_interval_is_the_mean_and_1_96_standard_errors_on_each_side PASSED
tests/test_spec008.py::test_c07_each_deal_is_played_four_times_with_the_agent_in_another_seat PASSED
tests/test_spec008.py::test_c07_the_agent_gets_the_reward_of_its_own_seat PASSED
tests/test_spec008.py::test_c07_c15_a_checkpoint_and_the_untrained_network_are_evaluated_from_the_command_line PASSED
tests/test_spec008.py::test_c07_the_network_agent_samples_legal_actions_for_its_seat[mlp] PASSED
tests/test_spec008.py::test_c07_the_network_agent_samples_legal_actions_for_its_seat[transformer] PASSED
tests/test_spec008.py::test_c08_the_fault_key_is_refused_unless_the_run_name_starts_with_control PASSED
tests/test_spec008.py::test_c08_the_fault_hands_the_reward_of_every_seat_to_the_next_seat PASSED
tests/test_spec008.py::test_c08_the_control_run_is_started_from_the_command_line PASSED
tests/test_spec008.py::test_c09_played_and_trained_decisions_per_second_are_logged PASSED
tests/test_spec008.py::test_c10_the_four_time_shares_add_up_to_95_to_100_percent_of_the_wall_time PASSED
tests/test_spec008.py::test_c11_the_peak_gpu_memory_is_in_the_summary PASSED
tests/test_spec008.py::test_c09_to_c14_the_report_script_reads_the_numbers_from_a_run_folder PASSED
tests/test_spec008.py::test_c12_decisions_and_tokens_per_seat_deal_mean_and_largest PASSED
tests/test_spec008.py::test_c13_the_names_of_the_bids_are_those_of_the_contract PASSED
tests/test_spec008.py::test_c13_deal_monitors_of_a_hand_built_batch PASSED
tests/test_spec008.py::test_c13_the_monitors_of_d107_are_logged_on_real_deals PASSED
tests/test_spec008.py::test_c14_the_guessing_level_of_hand_built_states PASSED
tests/test_spec008.py::test_c14_on_real_deals_the_true_place_is_always_among_the_possible_ones PASSED
tests/test_spec008.py::test_c14_the_accuracy_of_both_heads_is_logged_next_to_the_guessing_level PASSED
tests/test_spec008.py::test_c15_exported_records_replay_to_the_same_entries PASSED
tests/test_spec008.py::test_c17_the_update_rule_on_hand_computed_numbers PASSED
tests/test_spec008.py::test_c17_a_controller_that_is_off_never_changes_a_weight PASSED
tests/test_spec008.py::test_c17_the_controller_changes_only_the_entropy_weight PASSED
tests/test_spec008.py::test_c17_the_config_names_a_target_for_every_phase_and_the_band PASSED
tests/test_spec008.py::test_c19_every_setting_of_the_three_configs_has_its_origin_in_the_file PASSED
tests/test_spec008.py::test_c19_the_run_folder_names_the_commit_the_state_of_the_tree_and_the_config PASSED
tests/test_spec008.py::test_d130_a_module_built_from_other_engine_sources_is_refused PASSED

================= 1 failed, 67 passed, 1 deselected in 38.85s =================
```

`test_spec008.py`: 42 passed, 1 left out (`slow`). `test_spec006.py`: 25 of 26 passed.

**The one failure is `test_c20_a_python_thread_keeps_running_during_a_long_step` of spec 006, and it depends on the load of the PC.** The test asks that a counting thread keeps at least 20% of its lone speed while the engine runs. On the idle PC (4% CPU load) it failed in 4 of 4 runs with 18.5% (30.0 million counts in 3.49 s against 46.6 million per second alone). With four cores kept busy by another process it failed once and passed twice. It passed in all three runs of the whole suite earlier the same day, when other programs loaded the PC. My diff does not touch `run_random` or the release of the GIL. I did not change the test: it is spec 006's. Cause not established; my guess is that on an idle PC Windows keeps the three threads on few cores, and the counter gives up its time slice at every `time.sleep(0)`. Whether the engine of spec 013 plays a part was not checked.

**2. The whole suite as the `train` workflow runs it**, `python -m pytest -m "not slow" -v`, GPU hidden, on the tree before the last rebuild (the engine's sources as they were then): 133 passed, 5 skipped (the GPU tests), 6 deselected (`slow`), 200 s. 43 tests are this spec's (42 not slow and one slow); the other 91 not-slow tests are those of specs 003, 006 and 007. `test_spec008.py` alone takes 23 s on the idle PC.

**3. The slow tests and the GPU tests**, `python -m pytest -m "slow or gpu" -v -s`, GPU free: 11 passed in 733 s (the five slow tests of spec 007, the five GPU tests, and the test of criterion 16, which takes 624 s). Its printed numbers are in the table above.

**4. The binding crate** (`src\train\tarok_env`, Rust 1.94.1): `cargo fmt --check` clean, `cargo clippy --locked --all-targets -- -D warnings` clean, `cargo test --locked` 0 tests.

The first run of `test_spec008.py` found three wrong assertions in my tests (a count that a solo makes exactly half, not more than half; two in the controller tests) and no defect in the code. The tests were corrected, not loosened: each still states the criterion.

### The three runs (2026-11-12)

**This is a test of the pipeline, not a result about playing strength.** The opponent in the evaluation is random legal play.

Made after the review fixes, one after the other, with nothing else running on the PC. Commit `c7d1fa3` plus the uncommitted work of this spec (16 paths in `git status --short`, all of this spec; the list is in each `run-info.json`). Engine 0.2.0, fingerprint `64a6157fb05f43c4`, equal to the files on disk; the module was not rebuilt after the review fixes (`build.rs` and `lib.rs` did not change). GPU before the first run: 0% busy, 518 MiB of 6,144 in use. Folders under `src\train\runs\`, untracked.

Commands, from the repo root with the venv active:

```bat
python -m tarok_train.train --config src\train\configs\smoke-a-mlp.toml
python -m tarok_train.train --config src\train\configs\smoke-a-transformer.toml
python -m tarok_train.train --config src\train\configs\smoke-a-mlp.toml --name control-smoke-a --fault rewards_to_next_seat
python -m tarok_train.evaluate --checkpoint <run folder>\checkpoint-000200.pt --deals 2000 --seed 2008 --records 100
python -m tarok_train.evaluate --checkpoint <run folder>\checkpoint-000200.pt --deals 2000 --seed 2008 --untrained
python -m tarok_train.smoke_report <run folder>
```

**The run log (criterion 19).** For `docs/progress.md`.

| | MLP | Transformer | Control: MLP, rewards handed to the next seat |
|---|---|---|---|
| Run folder | `smoke-a-mlp-20261112-170937` | `smoke-a-transformer-20261112-171542` | `control-smoke-a-mlp-20261112-174730` |
| Config | `smoke-a-mlp.toml` | `smoke-a-transformer.toml` | `smoke-a-mlp.toml`, `--name control-smoke-a --fault rewards_to_next_seat` |
| Started | 17:09:38 | 17:15:44 | 17:47:32 |
| Updates, exit code | 200 of 200, 0 | 200 of 200, 0 | 200 of 200, 0 |
| Duration | 351.8 s (5.9 minutes) | 1,881.7 s (31.4 minutes) | 342.6 s (5.7 minutes) |
| What failed | nothing | nothing | nothing (it is meant to fail criterion 7, and does) |
| Parameters | MLP 1,243,430; critic 1,244,938 | transformer 3,767,590; critic 1,244,938 | as the MLP |

**The loop (criteria 1 to 5).**

| | MLP | Transformer | Control |
|---|---|---|---|
| 1. 200 updates, exit code 0 | yes | yes | yes |
| 2. Illegal actions refused by the engine | 0 | 0 | 0 |
| 3. Logged numbers that are not finite | 0 of 18,802 | 0 of 18,802 | 0 of 18,021 |
| 4. Largest sum of a deal's four rewards over the run | 1.2e-07 | 1.2e-07 | 1.2e-07 |
| 5. Oldest trained decision, in updates | 0 | 0 | 0 |
| Largest gap between stored and recomputed log-probability (`ppo/behaviour_mismatch`) | 6.0e-06 | 6.0e-06 | 4.8e-06 |

**Wiring (criteria 7, 8 and 15).** 2,000 deals with the engine seeds 2008 to 2011, each in four seatings, the agent sampling from its policy, random legal play in the other three seats. Reward 1 is 50 points of the paper score.

| Agent | Mean reward per deal | 95% interval | Beats random | Records |
|---|---|---|---|---|
| MLP after 200 updates | +0.5979 | +0.5896 to +0.6062 | **yes** | 100 of 100 replay to the same entries |
| MLP, untrained | −0.0049 | −0.0292 to +0.0194 | no | |
| Transformer after 200 updates | +0.6049 | +0.5968 to +0.6130 | **yes** | 100 of 100 |
| Transformer, untrained | −0.1236 | −0.1481 to −0.0991 | no | |
| Control after 200 updates | −0.2115 | −0.2394 to −0.1836 | **no** | 100 of 100 |

Criterion 7 is met for both networks. Criterion 8 is met: the control run, the same MLP with every seat's reward handed to the next seat, does not beat random; its whole interval is under zero. So the test of criterion 7 can fail, and the reward of each seat reaches that seat.

Two things to read with care. The untrained transformer is clearly under zero (−0.12): my reading, not checked, is that with a first scale of 0.3 its untrained policy has fixed preferences and those lose against uniform play. The same evaluation puts the untrained MLP at zero. And the control agent is worse than random, not equal to it: it learned something, namely to bid solo ena in 99% of the deals and lose (the declarer's side wins 1.4% of them). Under the fault that pays: the declarer gets the reward of the seat before it, an opponent who gains when the solo is lost.

**Speed and G3 (criterion 9).** End to end over the whole run: wall time from the first step to the last update, with logging and four checkpoints. Played counts the decisions of all four seats.

| | MLP | Transformer |
|---|---|---|
| Decisions played | 12,222,464 | 12,242,944 |
| Decisions trained on | 5,998,226 | 6,009,736 |
| Decisions played per second | 34,743 | 6,506 |
| Decisions trained on per second | 17,050 | 3,194 |
| Per update: median played per second (lowest to highest) | 34,928 (15,940 to 42,205) | 6,426 (4,646 to 7,959) |
| Share of played decisions trained on | 49.1% | 49.1% |
| The same against the decisions of the current network's seats only | 53.8% | 53.8% |
| GPU before (nvidia-smi) | 0% busy, 518 MiB | 0% busy, 518 MiB |
| GPU after, the run still holding its memory | 27% busy, 1,115 MiB | 92% busy, 5,350 MiB |

Against the old G3 of 10,000 decisions per second: the MLP is 3.5 times above it, the transformer reaches 65% of it played and 32% of it trained. The control run was as fast as the MLP run (35,118 played, 17,137 trained per second). For the new figure of G3 (D128, C13) the transformer's numbers are the ones to set it from: 6,500 played and 3,200 trained decisions per second on this card, with 512 tables, 2,048 seat-deals per update and two epochs. I propose no figure here; that is the main session's step.

Half of the play is not trained on: 49.1% is. A deal that straddles an update is played to its end and dropped, and 8.7% of the decisions are those of a past checkpoint's seats. The share of the first update is 93 to 95% (every table starts a deal at once); from the second on it is between 38% and 54%, with a median of 50%.

**Time split (criterion 10).**

| | Environment | Acting | Bookkeeping | Update | Together |
|---|---|---|---|---|---|
| MLP | 1.9% | 24.3% | 30.7% | 42.5% | 99.4% |
| Transformer | 0.4% | 46.2% | 5.8% | 47.5% | 99.9% |
| Control | 1.8% | 24.6% | 31.2% | 41.7% | 99.4% |

The engine is 0.4 to 2% of the time. For the transformer, acting and the update share the rest evenly. For the MLP the Python bookkeeping (copies into the storage, building the padded batches) is 31%: that is the cheapest place to gain speed for the control network, and it is 6% for the transformer.

**GPU memory (criterion 11).** No out-of-memory error with 512 tables. Three different numbers:

| | MLP | Transformer | Control |
|---|---|---|---|
| Largest sum of the run's tensors (`torch.cuda.max_memory_allocated`) | 294 MiB | 4,441 MiB | 283 MiB |
| Most held by PyTorch's allocator (`max_memory_reserved`) | 506 MiB | 5,156 MiB | 452 MiB |
| In use on the whole card at the end, all programs (nvidia-smi) | 1,115 MiB | 5,350 MiB | 653 MiB |

**The transformer run is close to the card's limit**: 5,350 of 6,144 MiB at the end, with 518 MiB used by the desktop before the run. The tensors' peak grew during the run, from 3,841 MiB at the first update to 4,441 MiB (the pre-flight of three updates showed 3,918). 512 tables fit; there is no room for more tables, a longer sequence or another program on the card.

**Length of a seat's deal (criterion 12).**

| | MLP | Transformer | Control |
|---|---|---|---|
| Decisions per seat-deal: mean, largest | 14.4, 19 | 14.4, 20 | 14.1, 19 |
| Tokens per seat-deal at its last decision: mean, largest | 60.0, 67 | 60.1, 67 | 60.9, 67 |

The largest token count is 67, under the limit of 128.

**Monitors at the first and the last update (criterion 13).** Entropy in nats. "Choice" is the same mean over the decisions with two or more legal slots; "forced" is the share of decisions with one legal slot (part C of the review).

| Phase (target) | MLP: entropy first, last | choice last | forced first, last | Transformer: entropy first, last | choice last | forced first, last |
|---|---|---|---|---|---|---|
| bidding (0.2) | 0.860, 0.212 | 0.212 | 28.4%, 0.0% | 0.795, 0.240 | 0.240 | 34.2%, 0.0% |
| contract choice (0.03) | 0.121, 0.029 | 0.029 | 85.7%, 0.0% | 0.068, 0.072 | 0.072 | 92.0%, 0.0% |
| king call (0.35) | 1.386, 0.412 | 0.412 | 0%, 0% | 1.367, 0.355 | 0.355 | 0%, 0% |
| talon pick (0.45) | 1.723, 0.717 | 0.717 | 0%, 0% | 1.739, 0.487 | 0.487 | 0%, 0% |
| discard (0.6) | 2.426, 0.595 | 0.595 | 0%, 0% | 2.409, 0.700 | 0.700 | 0%, 0% |
| play (0.25) | 0.953, 0.467 | 0.657 | 28.7%, 29.0% | 0.950, 0.579 | 0.815 | 28.4%, 29.0% |

| | MLP: first, last | Transformer: first, last | Control: first, last |
|---|---|---|---|
| Contract tri (forehand's forced tri) | 0.0%, 0.0% | 0.0%, 6.8% | 0.0%, 0.0% |
| Contract dve | 0.2%, 34.7% | 0.2%, 48.8% | 0.2%, 0.0% |
| Contract ena | 0.0%, 65.3% | 0.0%, 44.1% | 0.0%, 0.0% |
| Contract solo tri | 1.6%, 0.0% | 1.0%, 0.0% | 1.6%, 0.0% |
| Contract solo dve | 7.2%, 0.0% | 2.7%, 0.0% | 7.2%, 1.0% |
| Contract solo ena | 91.0%, 0.0% | 96.1%, 0.3% | 91.0%, 99.0% |
| Three passes before forehand | 0.8%, 0.4% | 0.6%, 7.5% | 0.8%, 0.0% |
| Declarer's side wins | 3.7%, 65.7% | 3.7%, 66.9% | 3.7%, 1.4% |

What the monitors show:

- **The bidding changes completely.** An untrained policy bids itself up to solo ena in more than nine deals of ten and the declarer wins 4% of them. After 200 updates both networks play dve and ena with a called king, and the declarer's side wins two deals of three. Tri and the solo contracts are almost gone. Whether this bidding is good cannot be read from a run against itself and random play.
- **The share of forced decisions moves with the policy**, as the review expected: in the bidding from 28% to 0%, in the contract choice from 86 to 92% to 0% (nobody is left with a solo ena as the only choice). In these two phases the phase mean rose for a reason that has nothing to do with how firmly the policy chooses. In the play the share stays at 29%.
- **Collapse, by the spec's fixed limit of 0.05 at the last update**: MLP: contract choice (0.029); transformer: none; control: contract choice (0.005). **Relative to the phase's target** (under the lower edge of its band, a quarter of the target): MLP none, transformer none, control: contract choice (0.005 under 0.0075). The MLP's contract choice is on its approved target of 0.03, so the fixed limit names a phase that is where D136 wants it.

**The entropy controller on real deals.** It acted as a floor, which is what its lower limit of 0.01 makes it. Lowest entropy of a phase during the run, and the largest weight the controller gave it:

| Phase (target) | MLP: lowest entropy (update), largest weight, last weight | Transformer: lowest entropy (update), largest weight, last weight |
|---|---|---|
| bidding (0.2) | 0.016 (24), 0.382, 0.051 | 0.012 (24), 0.592, 0.062 |
| contract choice (0.03) | 0.003 (98), 1.000, 0.223 | 0.013 (103), 0.170, 0.010 |
| king call (0.35) | 0.087 (76), 0.248, 0.097 | 0.051 (61), 0.231, 0.082 |
| talon pick (0.45) | 0.375 (29), 0.012, 0.010 | 0.225 (28), 0.034, 0.010 |
| discard (0.6) | 0.479 (191), 0.030, 0.026 | 0.690 (195), 0.010, 0.010 |
| play (0.25) | 0.440 (198), 0.010, 0.010 | 0.553 (163), 0.010, 0.010 |

- **The bidding did collapse, around update 24, in both runs** (entropy 0.012 to 0.016 against a target of 0.2): the failure D107 warns of. The controller raised the weight of that phase from 0.01 to 0.38 and 0.59, and the entropy came back to 0.21 and 0.24 by the end. Without the controller there is no run to compare with; what a fixed weight of 0.01 would have done on real deals was not measured.
- The play, the discard and the talon pick never came down to their targets in 200 updates. Their weights stayed at the lower limit, so for them the run is the run without a controller.
- In every update at least one phase was outside its band (`entropy/phases_outside_band`): early because all phases are far above their targets, later because the play stays above its band. The band says little on a run this short.
- In the MLP run the weight of the contract choice reached the upper limit of 1.0, and in the control run it stayed there: with 99% of those decisions forced, the phase mean cannot reach its target whatever the weight is (entropy 0.005 while the decisions with a choice have 0.53). This is the case of part C in practice.

**The auxiliary heads (criterion 14).** Accuracy next to what a uniform guess over the places still possible gets, both over the same cases.

| | MLP: first, last | Transformer: first, last |
|---|---|---|
| Partner head, accuracy | 0.0%, 95.6% | 23.5%, 92.4% |
| Partner, uniform guess | 0.0%, 90.4% | 69.9%, 85.8% |
| Partner head, accuracy while the partner is not public | 0.0%, 79.7% | 25.0%, 76.3% |
| Card-location head, accuracy | 28.5%, 36.3% | 26.9%, 37.1% |
| Card location, uniform guess | 30.4%, 30.9% | 30.4%, 30.8% |

- At the first update the partner numbers rest on almost no decisions (the MLP's first update had none: solo games have no partner). At the last update both heads are above the guessing level, by 5 to 7 points. The guessing level is high because most decisions with a partner come after the partner is public, where one seat is left.
- While the partner is not public the heads are right in 76 to 80% of the decisions. There is no guessing level for that subset alone in the logs; for an opponent it would be one of two seats, for the declarer one of three.
- The card-location head rises from the guessing level to 5 to 6 points above it.

**The critic.** Explained variance of its value against the realised return: from 0.00 at the first update to 0.59 (MLP) and 0.61 (transformer) at the last.

### Open issues

1. **CI is open** (criterion 18): nothing is pushed. The `train` workflow needs no change.
2. **The transformer run leaves about 800 MiB of the card free** (5,350 of 6,144 MiB in use at the end). 512 tables are the limit on this card with this cache; more tables, longer sequences (announcements, spec 009) or a second program on the GPU will not fit. The acting cache is the large part.
3. **Half of the play is not trained on** (49.1%). 2,048 seat-deals per update with 512 tables means one deal of every table, and the deals that straddle an update are dropped. Open point B named 4,096 as the other option.
4. **The controller steers on a phase mean that includes forced decisions** (part C of the review). In the runs the share of forced decisions went from 28% to 0% in the bidding and from about 90% to 0% in the contract choice, and in the control run a phase with 99% forced decisions drove its weight to the upper limit without effect. `entropy_choice/<phase>` and `forced_share/<phase>` are logged for the user's decision which mean the controller should use.
5. **The play, the discard and the talon pick did not reach their targets in 200 updates.** The controller did nothing for them; whether their targets are right cannot be said from this run. The band of 75% says little on a run this short: in every update at least one phase was outside it.
6. **The control corrupts about half the signal, not all of it.** In a solo two of the three opponents get the reward they would have got anyway. The control still failed criterion 7 clearly (−0.21).
7. **The untrained transformer is under zero against random play** (−0.12, interval −0.15 to −0.10), the untrained MLP is not (−0.005). My reading, not checked: with a first scale of 0.3 the untrained policy has fixed preferences, and those lose against uniform play. It does not touch criterion 7, which is about the trained agent.
8. **`aux/partner_*` rests on few decisions early in a run** (none at the MLP's first update: solo games have no partner). The king call is rare under random play: about 22 decisions in 6,912 random deals.
9. **The guessing level of the card-location head uses rules of flow A** (which cards can be hidden discards) and ignores block F3. It must be looked at again with flows B and C.
10. **The fingerprint counts `src/train/tarok_env/Cargo.lock`.** A change of the engine's version must be committed together with this lock file, or a fresh checkout builds a module with another fingerprint than the one tested.
11. **A test of spec 006 depends on the load of the PC**: `test_c20_a_python_thread_keeps_running_during_a_long_step` failed on the idle PC at the first hand-over (18.5% against the 20% it asks) and passed in the runs after the review. Details under "pytest output". Not this spec's test; not changed.
12. **The first 60 updates of the controller on the mock are outside the band** (reading 3, accepted by D136).
13. **Entropy per phase and the accuracies are measured during the first epoch of an update** (as in spec 007): later minibatches of that epoch see slightly changed weights. The controller steers on these numbers.
14. **No comparison run without the controller on real deals.** The bidding collapsed around update 24 and came back with the controller; what a fixed weight would have done was not measured. One more run of 31 minutes would show it.
