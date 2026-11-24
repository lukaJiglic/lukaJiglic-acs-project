# 015 — Findings of the longer run: GPU reading, transformer memory, checkpoint against checkpoint

Status: **approved** (2026-11-24) by the user's instruction "analyze the training and suggest improvements, fix any problems you found and continue with development" (D140). The user named no criteria: the criteria below are Claude's and open to correction. Parts A and B are the two defects of the run of spec 014. Part C is open point C (a) of spec 014; Claude took it under the same instruction, because the analysis the user ordered needs it. Say so if part C should wait. In execution in the `train-env` session.

Tool and model: Claude Code 2.1.306, `claude-fable-5-2`.

## Goal

Fix what the longer run showed to be broken in the training code, and make the evaluation able to show progress that random play no longer shows.

## Analysis behind this spec (main session, 2026-11-24)

From the run files of spec 014 and from a scratch script of the main session (not committed code) that seats one checkpoint against three seats of another, on 2,000 seeded deals in four seatings. Mean reward per deal of the single seat, 95% interval. Reward 1 is 50 points of the paper score. Flow A only.

| One seat | Three seats | Result |
|---|---|---|
| MLP 2,000 | MLP 2,000 (control) | +0.002 (−0.007 to +0.012) |
| MLP 2,000 | MLP 200 | +0.059 (+0.048 to +0.071) |
| MLP 200 | MLP 2,000 | −0.074 (−0.085 to −0.062) |
| MLP 2,000 | MLP 1,000 | +0.016 (+0.004 to +0.029) |
| MLP 2,000 | MLP 1,500 | +0.012 (+0.001 to +0.023) |
| Transformer 2,000 | Transformer 2,000 (control) | −0.002 (−0.011 to +0.007) |
| Transformer 2,000 | Transformer 200 | +0.118 (+0.102 to +0.135) |
| Transformer 200 | Transformer 2,000 | −0.103 (−0.113 to −0.092) |
| Transformer 2,000 | Transformer 1,000 | +0.043 (+0.032 to +0.054) |
| Transformer 2,000 | Transformer 1,500 | +0.025 (+0.015 to +0.036) |
| Transformer 2,000 | MLP 2,000 | +0.010 (−0.008 to +0.027) |
| MLP 2,000 | Transformer 2,000 | −0.063 (−0.074 to −0.052) |

Seeds 5151 (MLP rows but the last) and 5252. What it says:

1. **Both networks were still improving at update 2,000.** Against random legal play that was not visible (spec 014, criterion 4). Against its own earlier checkpoints the transformer gains +0.025 over the last 500 updates, the MLP +0.012.
2. **The transformer improves about twice as fast as the MLP** against its own checkpoint of update 200 (+0.118 against +0.059).
3. **The transformer is ahead of the MLP at update 2,000.** An MLP seat among three transformer seats loses 0.063 per deal. A transformer seat among three MLP seats is not measurably ahead. First evidence for the choice of D18, on flow A only.
4. **Random legal play is no longer a useful opponent** beyond the wiring test. It stays as that.
5. **Speed.** The transformer run was 24% slower than the smoke run. The update step slows at the update where the allocator's peak passes the card's size (spec 014, "What failed", item 3).
6. **Half of the acting is thrown away.** 48.1% of the played decisions are trained on. Each update trains on one deal per table and plays about two, because tables are not aligned with the updates and a deal cut by an update is dropped. Acting is 47% of the transformer's time. This is Batch 14, item 1; it is not fixed here.
7. **Play stays looser than its target.** The entropy of play is 0.46 and 0.51 against a target of 0.25, with the weight at the controller's lower limit for the whole run. Batch 14, item 2, and C16.

These numbers come from a scratch script. Part C puts the same measurement into the repo; criterion 16 repeats the table from committed code.

## In scope

- **A.** `gpu_snapshot` reads nvidia-smi output with a decimal temperature. A failed reading is said, not silent.
- **B.** The cause of the transformer's allocator peak above the card's size: measured, then removed without changing what is trained.
- **C.** `evaluate.py --opponent <checkpoint>`: the three other seats are played by another checkpoint.
- Tests for each part, written from the criteria before the code. README. The Results of this spec.

## Out of scope

Any change of the engine or the binding crate (the installed module stays valid). Any change of what is trained: tables, seat-deals per update, the update's batch size, the model, the loss. The straddling deals and the entropy of play (Batch 14). A new long run.

## Acceptance criteria

**A. The GPU reading**
1. `gpu_snapshot` returns the four numbers for the line `0, 188, 6144, 54.30` and for `0, 188, 6144, 54`. Test with a faked nvidia-smi output; no GPU needed.
2. For output it cannot read it returns an empty reading, and the trainer prints one line that says the GPU state could not be read. Test.
3. On this PC `run-info.json` and `summary.json` of a GPU run hold `gpu_before` and `gpu_after` with the four numbers. GPU test.

**B. The transformer's memory**
4. The trainer logs per update the allocator's current size and the tensors' current size, after acting and after the update step, next to the two peaks. Four new tags. Test on the CPU that the tags exist on a GPU run only, or are absent on the CPU without an error.
5. A measurement before any remedy: 60 updates of `long-a-transformer.toml` with the GPU free. Reported per update: seconds of the update step and of acting, the four new numbers, the whole card (nvidia-smi), and the shared GPU memory of the process if Windows gives it (a performance counter). The Results say whether the excess sat in shared memory: measured, or still inferred.
6. A remedy that changes no training setting. Candidates, in the order to try: a limit on the allocator below the card's size; releasing the allocator's cache between acting and the update; another allocator setting. The Results name what was tried and what each did.
7. With the remedy, 200 updates of `smoke-a-transformer.toml` (seed 8, as in spec 008): the allocator's peak stays at or under 5,600 MiB, no out-of-memory error, and the median seconds per update over updates 101 to 200 are at most 3% above the smoke run's for the same updates. The update step's median over updates 101 to 200 is at most 4.7 s (smoke run 4.41 to 4.51; the long run 6.2 to 6.7).
8. The remedy does not change the training. The loop checks of spec 008 hold over the 200 updates (no illegal action, no number that is not finite, rewards add to zero, decision age 0). The checkpoint of update 200 against random legal play, 2,000 deals, seed 2008: its interval overlaps the smoke run's (+0.597 to +0.613).
9. If the remedy is a setting, it is a key of the config with its origin, present in every config file; the files of finished runs get the value those runs had, with a comment that the key was added later.
10. If no remedy meets criterion 7: nothing of part B is changed beyond the logging; the Results give the measurements and the options for the user.

**C. Checkpoint against checkpoint**
11. `python -m tarok_train.evaluate --checkpoint A --opponent B --deals N --seed S`: checkpoint A plays one seat, checkpoint B the three others, each deal in four seatings. Both sample from their policy, each with its own seeded generator. Without `--opponent` nothing changes: random legal play.
12. The result goes to `evaluation-NNNNNN-vs-<run of B>-MMMMMM.json` next to A. It names both checkpoints. It never overwrites the file of the evaluation against random play.
13. A's seat is played by A only and the other three by B only. Test with two networks that can be told apart by their actions.
14. Refused with a clear message: an opponent of another encoding version; an evaluation seed range that holds the training seed of A or of B.
15. A checkpoint against itself on 2,000 deals has an interval that includes zero. Slow GPU test. Records can still be exported with an opponent, and replay to the same entries.
16. The table of the analysis above, from the committed code: the twelve pairs on 2,000 deals, seed 5151. Reported next to the scratch numbers. A pair whose interval does not overlap the scratch interval is named.

**Quality gates**
17. `pytest -m "not slow"` passes on the CPU (`CUDA_VISIBLE_DEVICES=-1`) and with the GPU tests. The slow tests pass on the GPU.
18. Check of the tests: at least eight wrong changes, put in one at a time, each fail a test. Among them: the temperature read as a whole number; the opponent seated in the agent's seat; the agent seated in all four seats; the evaluation against random play overwritten; the opponent's seed check left out; the remedy switched off.
19. No engine source file changes. `tarok_env.SOURCE_HASH` is `64a6157fb05f43c4` before and after.
20. README: the new flag with a cmd example, the new config key if there is one, the new tags.

## Checks

1 to 4, 11 to 15: named tests in `src/train/tests/test_spec015.py`. 5 to 10, 16: runs on this PC with the GPU free (under 10% busy, under 700 MiB), numbers in the Results. 17 to 20: command output and the table of wrong changes in the Results.

## Files expected to change

`src/train/tarok_train/bench_model.py`, `train.py`, `evaluate.py`, `smoke_report.py` if it prints the GPU state; `src/train/tests/test_spec015.py` (new); `src/train/README.md`; the config files only under criterion 9; this spec (Results). `docs/progress.md` by the main session. Run folders under `src/train/runs/`, untracked.

## Commits

By the main session after the user's read of the diff, one per part: `fix(train): the GPU reading with a decimal temperature (spec 015)`, `fix(train): the transformer's allocator stays on the card (spec 015)`, `feat(train): a checkpoint as the opponent in the evaluation (spec 015)`.

## Open points for the user

- **Part C** was taken without a separate answer (see Status). Stop it if it should wait for the arena of spec 011.
- The improvements that are not fixes are Batch 14 in `docs/planning/open-questions.md`.

## Results

(empty until executed)
