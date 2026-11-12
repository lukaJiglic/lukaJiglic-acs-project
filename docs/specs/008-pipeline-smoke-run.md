# 008 — Pipeline smoke run on deal flow A

Status: **approved** (2026-11-12, D124; open points A–E answered as recommended). Spec 006 is committed (D123). In execution in the `train-env` session.

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

(empty until executed)
