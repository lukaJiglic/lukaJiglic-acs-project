# 014 — Longer run on deal flow A

Status: **approved** (2026-11-23) by the user's instruction "go for a longer run" (D138). The user gave no settings: every setting below is Claude's choice and open to correction. In execution in the `train-env` session.

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

(empty until executed)
