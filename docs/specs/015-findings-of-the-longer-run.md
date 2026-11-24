# 015 — Findings of the longer run: GPU reading, transformer memory, checkpoint against checkpoint

Status: **approved** (2026-11-24) by the user's instruction "analyze the training and suggest improvements, fix any problems you found and continue with development" (D140). The user named no criteria: the criteria below are Claude's and open to correction. Parts A and B are the two defects of the run of spec 014. Part C is open point C (a) of spec 014; Claude took it under the same instruction, because the analysis the user ordered needs it. Say so if part C should wait. Executed on 2026-11-24 in the `train-env` session: not committed, waiting for the user's read of the diff (see Results).

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

Executed on 2026-11-24 in the `train-env` session; the fixes of the review of the same day are in (section "Review fixes" at the end). Not committed. Waiting for the user's read of the diff.

### Summary

- **A.** `gpu_snapshot` reads a temperature with decimals. An empty reading is said in one line by the trainer and printed as "not read" by the report.
- **B.** The cause is measured: the excess of PyTorch's allocator sits in shared GPU memory. The remedy is a limit on the allocator, the new key `[run] gpu_memory_limit_mib`. With 5,000 MiB the 200-update run is 15% faster than the smoke run of spec 008 and the allocator's peak is 4,910 MiB. No training setting changed.
- **C.** `evaluate.py --opponent <checkpoint>` is in. The twelve pairs of the analysis, from the committed code with seed 5151, all overlap the scratch numbers.
- **Review.** One real defect was found by the separate review, not by my tests: the sampling noise was reused in every chunk and seating, so intervals could be far too narrow. It is fixed. The pairs and the evaluation of criterion 8 were run again from the fixed code; both sets of numbers are below.
- The criteria are met, with one reservation under criterion 17: a test of spec 006 fails on the idle PC, as it did in spec 008. Criterion 10 did not apply. What did not go by the book is under "What failed".

### Criteria

| Criterion | Result | Shown by |
|---|---|---|
| 1 | Met | `test_c01_the_reading_holds_the_four_numbers_for_a_whole_and_a_decimal_temperature` (four lines, with and without a line end) |
| 2 | Met | `test_c02_output_that_cannot_be_read_gives_an_empty_reading` (7 outputs), `test_c02_a_missing_or_hanging_nvidia_smi_gives_an_empty_reading`, `test_c02_the_trainer_says_in_one_line_that_the_gpu_state_could_not_be_read`, `test_c02_a_gpu_run_whose_state_cannot_be_read_says_so_twice_and_still_runs` (GPU), `test_c02_the_report_says_not_read_for_an_empty_reading` |
| 3 | Met | `test_c03_on_this_pc_run_info_and_summary_hold_the_four_numbers_before_and_after` (GPU). Every measured run below has the four numbers in `run-info.json` and `summary.json` |
| 4 | Met | `test_c04_on_the_cpu_the_memory_tags_are_absent_and_nothing_fails`, `test_c04_the_four_tags_hold_the_reading_before_and_the_reading_after_the_update_step`, `test_c04_on_the_gpu_the_four_tags_are_logged_next_to_the_two_peaks` (GPU) |
| 5 | Met. The excess sat in shared memory: **measured** | "Part B, the measurement" |
| 6 | Met with the first candidate, the limit. Tried: 5,200 MiB and 5,000 MiB. The two other candidates were not tried | "Part B, the remedy"; `test_c06_...` (six tests, two on the GPU) |
| 7 | Met | "Part B, criterion 7" |
| 8 | Met | "Part B, criterion 8" |
| 9 | Met | `test_c09_...` (five tests); the six config files |
| 10 | Did not apply: a remedy met criterion 7 | |
| 11 | Met, after the review fix of the sampling noise. "Without `--opponent` nothing changes" holds for the files, the opponents and the stream of random legal play; the agent's own random numbers changed with the fix, so the digits of an evaluation against random play are not those of before | `test_c11_c13_...` (both networks), `test_c11_without_an_opponent_random_legal_play_is_asked_at_every_table_as_before`, `test_c11_without_the_flag_the_evaluation_is_against_random_legal_play`, `test_c11_both_sample_from_their_policy_each_with_its_own_seeded_generator`, `test_c11_the_function_wants_random_play_or_an_opponent_not_both_and_not_neither`, the three `test_r1_...` |
| 12 | Met. `<run of B>` is B's run folder since the review | `test_c12_the_result_names_both_checkpoints_and_never_overwrites_the_one_against_random_play`, `test_c12_the_report_names_the_opponent_and_does_not_say_beats_random`, the two `test_r3_...` |
| 13 | Met | `test_c11_c13_the_agents_seat_is_played_by_a_only_and_the_other_three_by_b_only` (MLP and transformer), `test_c13_with_an_opponent_every_table_is_asked_once_per_step_and_the_agent_only_in_its_seat`, `test_r2_two_networks_of_different_kinds_and_sizes_each_play_with_their_own_config` |
| 14 | Met | `test_c14_an_opponent_of_another_encoding_version_is_refused`, `test_c14_a_seed_range_that_holds_the_training_seed_of_a_or_of_b_is_refused`; also refused since the review: `test_r8_...` (two tests) |
| 15 | Met, after the review fix. The test of the first hand-over passed by its seed: on the code of that time the interval excluded zero for 8 of 20 seeds (the reviewer's count) | `test_c15_a_checkpoint_against_itself_on_2000_deals_has_an_interval_that_includes_zero` (slow, GPU): 20 seeds, at most 3 intervals may exclude zero; 20 records replay. Records with an opponent on the CPU: in the test of criterion 12 |
| 16 | Met, from the fixed code and from the code before the fix. No pair lies apart from its scratch interval in either | "Part C, the twelve pairs" |
| 17 | Met for every test of specs 007, 008 and 015. One test of spec 006 fails on the idle PC, as reported in spec 008; its files are not touched by this diff. CPU: 208 passed, 13 skipped, 1 failed. With the GPU: 221 passed, 1 failed. Slow tests on the GPU: 7 passed | "Tests" |
| 18 | Met. 41 of 41 wrong changes fail a test, the six named ones among them | "Wrong changes" |
| 19 | Met. `tarok_env.SOURCE_HASH` is `64a6157fb05f43c4` before and after; `git diff --stat -- src/engine src/train/tarok_env` is empty | "Tests" |
| 20 | Met | `src/train/README.md`: the `--opponent` paragraph with a cmd example, "The limit on GPU memory", the four tags in the table of monitors |

### The measured runs and the state of the card

Every run was started by a script that reads nvidia-smi first and waits when the card is not free (under 10% busy, under 700 MiB). During each run a sampler wrote nvidia-smi and two Windows performance counters ("GPU Process Memory", "GPU Adapter Memory") every 5 to 7 seconds. Scripts, logs and samples are in `src/train/runs/spec015` (untracked).

| Run | What | Started | Card before | Seconds | Another program on the card during it |
|---|---|---|---|---|---|
| M1 | 60 updates of `long-a-transformer.toml`, no remedy (criterion 5) | 09:32 | 0%, 380 MiB | 1,939.8 | Yes, from update 29 on |
| Pairs | The twelve evaluations of criterion 16 | 10:36 | 0%, 308 MiB | 215 | No |
| T1 | 100 updates of `long-a-transformer.toml`, limit 5,200 MiB | 10:40 | 0%, 236 MiB | 867.3 | No |
| T3 | The same, limit 5,000 MiB | 13:12 | 0%, 268 MiB | 884.6 | No. CPU load from a test run of the main session until about 13:20 |
| C7 | 200 updates of `smoke-a-transformer.toml`, limit 5,000 MiB (criteria 7 and 8) | 13:33 | 0%, 304 MiB | 1,647.2 | No |
| C8 | Its checkpoint of update 200 against random legal play | 14:01 | 1%, 306 MiB | under 30 | No |
| Pairs, again | The twelve evaluations from the fixed code (review, point 1) | 18:27 | 0%, 313 MiB | 172 | No |
| C8, again | The same evaluation from the fixed code | 18:30 | 0%, 322 MiB | under 30 | No |

Waiting for the card: M1 waited 10 minutes (09:22: 100% busy, 1,817 MiB; free at 09:27 and 09:32). The pairs waited 30 minutes (three of seven checks showed 12% to 22% busy and 3,318 to 3,872 MiB). T1, T3 and C7 found the card free at their first check. T2 does not exist: the name was kept for a trial of the second candidate that was not needed.

Commits at the start: M1 `7206f9d`, pairs, T1 and T3 `719a968`, C7 `144f824`, each with this spec's uncommitted changes. The main session's commits in between touch `docs/` only. M1, T1 and T3 ran the trainer before the key existed; the limit of T1 and T3 was set by a scratch launcher (`try_remedy.py`) that calls `torch.cuda.set_per_process_memory_fraction` and then the unchanged trainer. C7 ran the committed code path with `--gpu-memory-limit-mib 5000`.

### Part A: the GPU reading

- `gpu_snapshot` (`bench_model.py`) reads load, memory used and memory total as whole numbers and the temperature as a number with or without decimals. `temperature_c` is now a float in the JSON files (54.3, or 54.0).
- Anything else gives the empty reading `{}`: no output, three or five fields, a field that is not a number, nvidia-smi missing or hanging.
- `train.gpu_state("before" | "after")` wraps it and prints one line for an empty reading: "the GPU state before the run could not be read: nvidia-smi gave no output this code can read". `train.main` uses it for both readings.
- `smoke_report.py` prints "not read" in place of `{}` and "None MiB".
- On this PC today: `gpu_before` of C7 is 0%, 309 MiB of 6,144 MiB, 50.5 degrees; `gpu_after` is 92%, 5,242 MiB, 66.7 degrees.
- Not changed: the benchmark of spec 003 (`bench_model.main`) prints its "GPU before the run" line only when the reading worked. It was so before. See "Open issues".

### Part B, the measurement (criteria 4 and 5)

Four new tags per update on a GPU run: `gpu/reserved_after_acting_mib`, `gpu/allocated_after_acting_mib`, `gpu/reserved_after_update_mib`, `gpu/allocated_after_update_mib`. "Reserved" is what PyTorch's allocator holds from the card at that moment, "allocated" the tensors in it. They are read in `Trainer.update`, right before and right after `ppo.update`.

M1: 60 updates of `long-a-transformer.toml`, no remedy. MiB. "Process dedicated" and "process shared" are the Windows counters of the training process, the largest sample inside the update.

| Update | Seconds | Update step s | Acting s | Allocator after acting | Tensors after acting | Allocator after the update | Tensors after the update | Whole card (nvidia-smi) | Process dedicated | Process shared |
|---|---|---|---|---|---|---|---|---|---|---|
| 1 | 6.21 | 4.33 | 1.41 | 1,898 | 1,130 | 3,762 | 1,199 | 3,452 | 3,853 | 82 |
| 2 | 6.82 | 4.11 | 2.28 | 3,762 | 1,199 | 3,764 | 1,198 | 4,228 | 3,855 | 82 |
| 3 | 7.24 | 4.10 | 2.71 | 5,844 | 2,190 | 5,844 | 2,191 | 5,835 | 5,488 | 530 |
| 5 | 7.47 | 4.16 | 2.82 | 5,844 | 2,191 | 5,940 | 2,190 | 5,828 | 5,488 | 626 |
| 8 | 8.01 | 4.66 | 2.86 | 6,040 | 2,190 | 6,144 | 2,191 | 5,829 | 5,490 | 724 |
| 20 | 7.69 | 3.99 | 3.14 | 6,144 | 2,191 | 6,144 | 2,190 | 5,837 | 5,490 | 828 |
| 21 | 9.02 | 4.71 | 3.69 | 6,144 | 2,190 | 6,252 | 2,191 | 5,837 | 5,490 | 936 |
| 24 | 8.97 | 4.79 | 3.64 | 6,366 | 2,191 | 6,366 | 2,190 | 5,833 | 5,491 | 1,050 |
| 28 | 8.85 | 4.04 | 4.20 | 6,366 | 2,190 | 6,366 | 2,190 | 5,828 | 5,491 | 1,050 |
| 29 | 23.80 | 12.92 | 10.30 | 6,366 | 2,190 | 6,366 | 2,190 | 5,926 | 5,347 | 1,194 |
| 30 | 418.28 | 406.08 | 11.59 | 6,366 | 2,190 | 6,366 | 2,190 | 5,956 | 5,347 | 1,194 |
| 31 | 502.86 | 316.19 | 185.95 | 6,366 | 2,190 | 6,366 | 2,191 | 5,957 | 5,347 | 1,194 |
| 32 | 311.85 | 4.96 | 306.34 | 6,366 | 2,191 | 6,366 | 2,190 | 5,956 | 5,347 | 1,194 |
| 33 | 10.72 | 6.06 | 4.16 | 6,366 | 2,190 | 6,366 | 2,191 | 5,472 | 5,347 | 1,194 |
| 60 | 10.22 | 5.01 | 4.35 | 6,366 | 2,191 | 6,366 | 2,191 | 5,516 | 5,347 | 1,194 |

What it shows:

1. **The excess is in shared memory. Measured.** The process's shared GPU memory is 82 MiB while the allocator holds 3,762 MiB. It is 530 MiB at 5,844, 828 MiB at 6,144 and 1,050 MiB at 6,366. Its dedicated memory stops at about 5,490 MiB. Every 100 MiB the allocator takes above that go to shared memory.
2. **The spill starts at update 3, at an allocator size of about 5,400 MiB**, not at the card's 6,144 MiB. The card had 380 MiB of other programs before the run, the process's own overhead is about 90 MiB, and Windows did not hand out the last 270 MiB or so.
3. **The allocator holds far more than the tensors need.** Between the phases the tensors are 2,190 MiB; their peak inside the update step is 4,270 MiB (4,441 MiB in the smoke config). The allocator holds 6,366 MiB. About 2 GB are freed blocks it keeps as a cache.
4. **Cold cache in shared memory costs little by itself.** Updates 3 to 28 ran with 530 to 1,050 MiB in shared memory and an update step of 4.0 to 5.4 s.
5. **It becomes costly when the card is contested.** From update 29 another program took about 500 MiB of dedicated memory. The training's dedicated share fell to 5,347 MiB, its shared share rose to 1,194 MiB, and updates 29 to 32 took 24, 418, 503 and 312 s. Updates 36 to 44, 54 and 56 were slow again (12 to 77 s) while that program was active. The timing of M1 after update 28 is not used anywhere below.
6. The run repeats the long run of spec 014: loss, trained decisions and both peaks agree to five decimals up to update 23, and the allocator's peak takes the same steps at the same updates (3,762, 5,844, 5,940, 6,040, 6,144, 6,252, 6,366). Differences of 1e-7 from update 1 grow until the sampled actions differ, as in every repeat on the GPU.

The slow update step of the long run (5.2 to 6.7 s) is therefore explained in two parts. Measured: the allocator's excess is in shared memory, and with the limit the update step stays at 4.1 s (next section). Inferred: in the long run the slowdown grew over the first 100 updates because more of the memory in use, not only cold cache, ended up in shared memory; no counter was recorded during that run.

### Part B, the remedy (criterion 6)

**A limit on PyTorch's allocator.** `torch.cuda.set_per_process_memory_fraction` caps what the allocator may take from the card. When a new block would pass the cap, the allocator frees its cached blocks and tries again; only if the tensors themselves do not fit does it raise an out-of-memory error. The second half of that sentence is from the function's documentation in the installed PyTorch 2.14.1 ("If trying to allocate more than the allowed value in a process, will raise an out of memory error in allocator"). The first half is tested here: `test_c06_on_the_gpu_the_allocator_frees_its_cache_instead_of_passing_the_limit`, and the runs below.

Tried on 100 updates of `long-a-transformer.toml`, next to the same updates of the long run of spec 014. Medians.

| | Long run of spec 014, no limit | T1: limit 5,200 MiB | T3: limit 5,000 MiB |
|---|---|---|---|
| Update step, updates 1 to 20 | 4.13 s | 4.23 s | 4.11 s |
| Update step, updates 21 to 60 | 5.23 s | 4.19 s | 4.09 s |
| Update step, updates 61 to 100 | 5.82 s | 4.10 s | 4.14 s |
| Whole update, updates 61 to 100 | 10.33 s | 8.26 s | 8.49 s |
| Allocator's peak | 6,484 MiB | 5,184 MiB | 4,992 MiB |
| Allocator after the update, updates 61 to 100 | not logged then | 4,746 MiB | 4,776 MiB |
| Tensors' peak | 4,364 MiB | 4,369 MiB | 4,335 MiB |
| Process shared memory, largest sample | not recorded (M1: 1,194 MiB) | 82 MiB | 82 MiB |
| Out-of-memory error | no | no | no |
| Decisions played per second, whole trial | | 6,989 | 6,854 |

- Both limits remove the slowdown. The update step stays at 4.1 to 4.2 s for all 100 updates.
- T3 ran under CPU load of a test run of the main session until about 13:20, about its first 50 updates. The environment's time per update is 0.04 to 0.05 s there, as in the rest of the run and in T1. No trace of the load is seen; the memory readings do not depend on it.
- **Chosen: 5,000 MiB.** Reason: the spill started at about 5,400 MiB with 380 MiB of other programs on the card. A card counts as free up to 700 MiB of other programs; then the spill starts at about 5,080 MiB. 5,000 stays under that, 5,200 does not. The tensors' peak is 4,335 to 4,441 MiB, so 5,000 leaves about 560 MiB for the allocator's cache.
- **Not tried:** releasing the cache between acting and the update (`torch.cuda.empty_cache`), and another allocator setting (`PYTORCH_CUDA_ALLOC_CONF`). The first candidate met the criteria. The launcher `try_remedy.py` can run both if they are wanted.

**The code.**

- `train.limit_gpu_memory(limit_mib, device)` sets the limit and returns the fraction of the card it set. The CPU sets nothing. 0 sets nothing in a process that never set a limit, and sets the card back to its whole size in one that did (review, point 5). A limit more than 1% above the card's memory is refused (review, point 6).
- The `Trainer` applies `[run] gpu_memory_limit_mib` first, before anything is put on the card; on a resume the checkpoint is read to the CPU for that (review, point 4). The limit holds for the whole process. A GPU run prints it at the start ("gpu memory: PyTorch's allocator is limited to 5,000 MiB").
- `--gpu-memory-limit-mib` overrides the key, also together with `--resume`: the limit belongs to the machine, not to the training.
- `train.LATER_KEYS` and `train.with_later_keys`: a checkpoint made before the key existed is read with 0, the value its run had. Without this the checkpoints of specs 008 and 014 could not be resumed. A config file without the key is still refused.

### Part B, criterion 7: 200 updates of `smoke-a-transformer.toml` with the limit

Seed 8, as the smoke run of spec 008. Command: `python -m tarok_train.train --config src\train\configs\smoke-a-transformer.toml --gpu-memory-limit-mib 5000 --name c7-smoke-limit5000 --out-dir src\train\runs\spec015`. Card before: 0% busy, 309 MiB. No other program on the card during the run.

| | Asked | With the limit (C7) | Smoke run of spec 008 |
|---|---|---|---|
| Allocator's peak | at most 5,600 MiB | 4,910 MiB | 5,156 MiB |
| Out-of-memory error | none | none | none |
| Median seconds per update, updates 101 to 200 | at most 3% above the smoke run's | 8.35 s (15.0% below) | 9.82 s |
| Median of the update step, updates 101 to 200 | at most 4.7 s | 4.09 s | 4.48 s |
| Median of acting, updates 101 to 200 | | 3.70 s | 4.79 s |
| Tensors' peak | | 4,441 MiB | 4,441 MiB |
| Process shared memory, largest sample | | 82 MiB | not recorded |
| Whole run | | 1,647.2 s, 7,437 played and 3,645 trained decisions per second | 1,881.7 s, 6,506 and 3,194 |

The run with the limit is faster than the smoke run in every block of updates, in the update step and in acting. The likely reason, inferred: the smoke run's allocator held 5,156 MiB with 518 MiB of other programs on the card, which is at the spill point measured above, so the smoke run of spec 008 was already slowed a little. One number of that run fits: nvidia-smi showed 5,350 MiB on the whole card at its end, less than the allocator's 5,156 MiB plus the 518 MiB of other programs. No counter was recorded then.

### Part B, criterion 8: the remedy does not change the training

| Check over the 200 updates | Value |
|---|---|
| Illegal actions refused by the engine | 0 |
| Logged numbers that are not finite | 0 of 19,602 |
| Largest sum of a deal's four rewards | 1.2e-07 |
| Oldest trained decision, in updates | 0 |
| Largest difference between the stored log-probability and the one the weights give | 6.1e-06 |
| Checkpoint of update 200 against random legal play, 2,000 deals, seed 2008, from the fixed code | +0.5997 (+0.5914 to +0.6080) |
| The same from the code before the review fix | +0.6015 (+0.5931 to +0.6099) |
| The smoke run's interval (code before the fix) | +0.6049 (+0.5968 to +0.6130): both overlap it |

The file of the code before the fix is kept in `before-review\` of the run folder; the new one has the old name. The two differ only by the agent's random numbers (review, point 1).

Not asked by the criterion: over updates 1 to 20 the run with the limit agrees with the smoke run of spec 008 to 1.5e-06 in all 86 tags that are not times or memory. After that the two drift apart as two repeats on the GPU do (see M1, point 6).

### Part B, criterion 9: the key

`gpu_memory_limit_mib` is a key of `[run]` in the schema of `train.py`: a whole number of MiB, 0 for no limit, negative values refused. It stands in all six config files with its origin. All six have 0:

| File | Value | Why |
|---|---|---|
| `mock.toml`, `smoke-a-ci.toml` | 0 | Small runs; the test config runs on the CPU |
| `smoke-a-mlp.toml`, `smoke-a-transformer.toml` | 0 | The finished runs of spec 008 had no limit; the comment says the key was added later |
| `long-a-mlp.toml`, `long-a-transformer.toml` | 0 | The finished runs of spec 014 had no limit; the comment says so |

So no config file carries 5,000 yet. The two transformer files name the value for a new run in their comment. The next run config must set it; see "Open issues".

### Part C: a checkpoint as the opponent (criteria 11 to 15)

- `python -m tarok_train.evaluate --checkpoint A --opponent B --deals N --seed S`. A plays one seat, B the three others, each deal in four seatings. Each network plays with the model config of its own checkpoint.
- A samples from a generator seeded with S, B from one seeded with S + 1. Each generator serves the whole evaluation: no chunk and no seating gets the random numbers of another one (review, point 1).
- The opponent is asked only for the tables the agent does not play at that step. Random legal play is still asked for every table at every step, so its stream of random numbers is the one of spec 008.
- The result goes to `evaluation-NNNNNN-vs-<run folder of B>-MMMMMM.json` next to A, records to a folder `records-...` of the same name. The run folder carries date and time, so two runs of one name give two files. The file has `opponent` (checkpoint, run, updates, run folder), `opponents` (a sentence), the seed, the deals, `tables` and `threads`. It has `above_zero` and `below_zero` in place of `beats_random`.
- The same pair with another seed, other deals or other tables has the same file name. Such a call is refused with one line that names the file, unless `--overwrite` is given; then the records of the replaced call are removed too.
- Refused with exit code 2 and one line: a path that is no file or no checkpoint; a checkpoint of a run on the mock environment; another encoding version; a seed whose chunks hold the training seed of A or of B. `evaluate.evaluate` refuses the seed too (`opponent_training_seed`).
- The two networks may be of different kinds and sizes.
- `smoke_report.py` prints such a file with its opponent and "The whole interval is above zero" (or below, or "includes zero"), not "Beats random".

### Part C, the twelve pairs (criterion 16)

2,000 deals, each in four seatings, seed 5151 for all twelve. Mean reward per deal of the single seat with the 95% interval. Reward 1 is 50 points of the paper score. Flow A only. Run twice: at 10:36 from the code of the first hand-over (card before: 0% busy, 308 MiB; 215 s), and at 18:27 from the code after the review fix of the sampling noise (0%, 313 MiB; 172 s). The files of the first run keep their names (`...-vs-long-a-mlp-000200.json`); the new ones carry the opponent's run folder (`...-vs-long-a-mlp-20261124-000454-000200.json`).

| One seat | Three seats | Fixed code, seed 5151 | Code before the review fix, seed 5151 | Scratch script | Scratch seed |
|---|---|---|---|---|---|
| MLP 2,000 | MLP 2,000 (control) | +0.0058 (-0.0056 to +0.0171) | -0.0024 (-0.0133 to +0.0085) | +0.0023 (-0.0074 to +0.0120) | 5151 |
| MLP 2,000 | MLP 200 | +0.0629 (+0.0513 to +0.0745) | +0.0575 (+0.0453 to +0.0697) | +0.0593 (+0.0480 to +0.0705) | 5151 |
| MLP 200 | MLP 2,000 | -0.0741 (-0.0861 to -0.0620) | -0.0712 (-0.0835 to -0.0589) | -0.0739 (-0.0853 to -0.0624) | 5151 |
| MLP 2,000 | MLP 1,000 | +0.0188 (+0.0062 to +0.0315) | +0.0174 (+0.0048 to +0.0300) | +0.0163 (+0.0041 to +0.0285) | 5151 |
| MLP 2,000 | MLP 1,500 | +0.0149 (+0.0036 to +0.0262) | +0.0223 (+0.0105 to +0.0342) | +0.0118 (+0.0008 to +0.0229) | 5252 |
| Transformer 2,000 | Transformer 2,000 (control) | -0.0024 (-0.0124 to +0.0076) | -0.0013 (-0.0110 to +0.0084) | -0.0019 (-0.0110 to +0.0072) | 5252 |
| Transformer 2,000 | Transformer 200 | +0.1205 (+0.1032 to +0.1378) | +0.1018 (+0.0848 to +0.1188) | +0.1182 (+0.1018 to +0.1345) | 5252 |
| Transformer 200 | Transformer 2,000 | -0.0982 (-0.1094 to -0.0870) | -0.1011 (-0.1124 to -0.0898) | -0.1025 (-0.1134 to -0.0916) | 5252 |
| Transformer 2,000 | Transformer 1,000 | +0.0508 (+0.0396 to +0.0621) | +0.0501 (+0.0389 to +0.0613) | +0.0433 (+0.0323 to +0.0542) | 5252 |
| Transformer 2,000 | Transformer 1,500 | +0.0164 (+0.0051 to +0.0276) | +0.0135 (+0.0019 to +0.0251) | +0.0254 (+0.0148 to +0.0361) | 5252 |
| Transformer 2,000 | MLP 2,000 | +0.0166 (-0.0013 to +0.0345) | +0.0022 (-0.0155 to +0.0199) | +0.0096 (-0.0076 to +0.0268) | 5252 |
| MLP 2,000 | Transformer 2,000 | -0.0596 (-0.0709 to -0.0483) | -0.0557 (-0.0670 to -0.0443) | -0.0632 (-0.0741 to -0.0523) | 5252 |

- No pair's interval from the fixed code lies apart from its scratch interval, and none lies apart from the interval of the code before the fix.
- The three columns use different random numbers for the sampling (the scratch script: seed + 7 for the opponent, and both from the same numbers in every chunk; the code before the fix: seed + 1, the same numbers in every chunk; the fixed code: seed + 1, numbers never reused). So no row is the same number twice.
- Why the intervals of the first run stand: the tie between the deal values of one table index across the four chunks is at most 0.025 in its twelve files, and at most 0.042 in the new ones. A tie of that size comes from chance with 500 tables. For these trained checkpoints the reused noise did not matter; for a network that depends little on the cards it did (see "Review fixes", point 1).
- Both controls include zero in all three columns.
- Every sign and every "interval excludes zero" of the analysis holds in all three columns: the later checkpoint is ahead of the earlier one in all eight such pairs, one MLP seat among three transformer seats loses, and one transformer seat among three MLP seats is not measurably ahead (fixed code: +0.0166, -0.0013 to +0.0345).
- One statement of the analysis does not hold across the columns: the size of the gain over the last 500 updates. Scratch: transformer +0.025, MLP +0.012. Code before the fix: +0.0135 and +0.0223. Fixed code: +0.0164 and +0.0149. All six intervals are above zero, so "both were still improving at update 2,000" stands; which network gained more over the last 500 updates is not settled by these numbers.
- These are results on flow A between checkpoints of two runs. They say nothing about playing strength in full tarok.

### Tests (criterion 17)

`src/train/tests/test_spec015.py`: 46 test functions, 64 tests with their parameters; 8 need the GPU, one of them is slow.

The runs of 18:32 to 18:55, from `src\train`, after the last change to the code (the review fixes). Card before: 0% busy, 313 MiB.

| Run | Result |
|---|---|
| `pytest -m "not slow"` with `CUDA_VISIBLE_DEVICES=-1` | 208 passed, 13 skipped, 1 failed, 7 deselected in 158 s. The 13 skipped are the GPU tests, seven of them of this spec. The one failure is `test_c20_a_python_thread_keeps_running_during_a_long_step` of spec 006 |
| `pytest -m "not slow"` with the GPU | 221 passed, 1 failed, 7 deselected in 179 s. The same one failure |
| `pytest -m slow` with the GPU | 7 passed, 222 deselected in 612 s. Six are the slow tests of specs 007 and 008, one is criterion 15 (20 seeds) |
| `pytest tests\test_spec015.py` with the GPU, slow test included | 64 passed in 110 s |

The runs before the review (14:06 to 14:20, 47 tests of this spec): CPU 194 passed, 11 skipped; with the GPU 205 passed; slow 7 passed.

`tarok_env.SOURCE_HASH` printed before and after these runs: `64a6157fb05f43c4`. `git diff --stat -- src/engine src/train/tarok_env` prints nothing.

The tests of specs 007 and 008 needed no change. Their configs come from the config files, which now hold the new key.

### Wrong changes (criterion 18)

Each wrong change was put alone into a copy of the tree and the tests of spec 015 were run on the copy (`src/train/runs/spec015/mutants015.py`, with the GPU). The six the criterion names are A1, C1, C2, C3, C4 and B3. R1 to R8 came with the review: R1 is the defect of point 1 put back, R2a and R2b are the two wrong changes the reviewer found alive.

| Wrong change | Caught by | Outcome |
|---|---|---|
| A1 the temperature read as a whole number | `test_c01_the_reading_holds_the_four_numbers_for_a_whole_and_a_decimal_temperature` | fails, as it must |
| A2 an empty reading is not said | `test_c02_the_trainer_says_in_one_line_that_the_gpu_state_could_not_be_read` | fails, as it must |
| A3 the report prints an empty reading as it is | `test_c02_the_report_says_not_read_for_an_empty_reading` | fails, as it must |
| B1 the readings after acting and after the update step are swapped | `test_c04_the_four_tags_hold_the_reading_before_and_the_reading_after_the_update_step` | fails, as it must |
| B2 reserved and allocated are swapped | `test_c04_on_the_gpu_the_four_tags_are_logged_next_to_the_two_peaks` | fails, as it must |
| C1 the opponent seated in the agent's seat | `test_c11_c13_the_agents_seat_is_played_by_a_only_and_the_other_three_by_b_only` | fails, as it must |
| C2 the agent seated in all four seats | `test_c11_c13_the_agents_seat_is_played_by_a_only_and_the_other_three_by_b_only` | fails, as it must |
| C3 the evaluation against random play overwritten | `test_c12_the_result_names_both_checkpoints_and_never_overwrites_the_one_against_random_play` | fails, as it must |
| C4 the opponent's seed check left out (command line and function call) | `test_c14_a_seed_range_that_holds_the_training_seed_of_a_or_of_b_is_refused` | fails, as it must |
| C5 the function's own check of the opponent's seed left out | `test_c14_a_seed_range_that_holds_the_training_seed_of_a_or_of_b_is_refused` | fails, as it must |
| C6 the opponent's encoding version is not checked | `test_c14_an_opponent_of_another_encoding_version_is_refused` | fails, as it must |
| C7 agent and opponent share one seed | `test_c11_both_sample_from_their_policy_each_with_its_own_seeded_generator` | fails, as it must |
| C8 the opponent is asked at the agent's tables too | `test_c13_with_an_opponent_every_table_is_asked_once_per_step_and_the_agent_only_in_its_seat` | fails, as it must |
| C9 A's network also plays the three other seats | `test_c11_c13_the_agents_seat_is_played_by_a_only_and_the_other_three_by_b_only` | fails, as it must |
| C10 the opponent's weights are not loaded | `test_c11_c13_the_agents_seat_is_played_by_a_only_and_the_other_three_by_b_only` | fails, as it must |
| C11 with an opponent the result still says beats_random | `test_c12_the_result_names_both_checkpoints_and_never_overwrites_the_one_against_random_play` | fails, as it must |
| C12 records with an opponent go into the folder of the records against random play | `test_c12_the_result_names_both_checkpoints_and_never_overwrites_the_one_against_random_play` | fails, as it must |
| B3 the remedy switched off: the limit is never set | `test_c06_the_limit_is_set_as_that_share_of_the_cards_memory` | fails, as it must |
| B4 the trainer does not apply the limit of its config | `test_c06_the_trainer_applies_the_limit_of_its_config_before_it_builds_anything` | fails, as it must |
| B5 the limit is taken in MB, not MiB | `test_c06_the_limit_is_set_as_that_share_of_the_cards_memory` | fails, as it must |
| B6 a checkpoint from before the key is refused | `test_c09_a_checkpoint_from_before_the_key_is_resumed_with_the_value_its_run_had` | fails, as it must |
| B7 the limit of the command line is ignored on resume | `test_c09_the_limit_can_be_set_from_the_command_line_also_on_resume` | fails, as it must |
| B8 a negative limit is accepted | `test_c09_the_key_must_be_a_whole_number_of_mib_and_not_negative` | fails, as it must |
| B9 a config file without the key gets no limit instead of a refusal | `test_c09_a_config_file_without_the_key_is_refused` | fails, as it must |
| B10 a limit above the card's memory is passed on | `test_c06_a_limit_above_the_cards_memory_is_refused` | fails, as it must |
| R1 every environment starts a new generator from the same seed (the defect of point 1) | `test_r1_two_environments_of_one_evaluation_do_not_get_the_same_random_numbers` | fails, as it must |
| R2a the opponent's actor gets A's model config (point 2) | `test_r2_two_networks_of_different_kinds_and_sizes_each_play_with_their_own_config` | fails, as it must |
| R2b the opponent's network is built with A's kind and config, B's weights are loaded (point 2) | `test_r2_two_networks_of_different_kinds_and_sizes_each_play_with_their_own_config` | fails, as it must |
| R3a an existing result is replaced without --overwrite | `test_r3_an_existing_result_is_not_replaced_without_overwrite` | fails, as it must |
| R3b the file is named by the opponent's run name, not by its run folder | `test_c12_the_result_names_both_checkpoints_and_never_overwrites_the_one_against_random_play` | fails, as it must |
| R3c records of the replaced call stay behind | `test_r3_an_existing_result_is_not_replaced_without_overwrite` | fails, as it must |
| R3d tables and threads are not written into the result | `test_c11_without_the_flag_the_evaluation_is_against_random_legal_play` | fails, as it must |
| R4 on resume the checkpoint is put on the card before the limit is set | `test_r4_on_resume_the_limit_is_set_before_anything_of_the_checkpoint_is_on_the_card` | fails, as it must |
| R5a no limit does not lift a limit set earlier in the process | `test_r5_no_limit_lifts_a_limit_set_earlier_in_the_same_process` | fails, as it must |
| R5b after lifting, the run says "no limit" | `test_r5_the_line_at_the_start_of_a_run_says_what_holds` | fails, as it must |
| R6a a limit of the card's nominal size is refused when PyTorch reports a little less | `test_r6_a_limit_of_the_cards_nominal_size_is_taken_when_pytorch_reports_a_little_less` | fails, as it must |
| R6b a fraction above 1 is passed on to PyTorch | `test_r6_a_limit_of_the_cards_nominal_size_is_taken_when_pytorch_reports_a_little_less` | fails, as it must |
| R7 a temperature that is not a number is taken | `test_c02_output_that_cannot_be_read_gives_an_empty_reading` | fails, as it must |
| R8a a file that is no checkpoint ends in a traceback | `test_r8_a_path_that_is_no_checkpoint_is_refused_in_one_line` | fails, as it must |
| R8b a checkpoint of the mock environment is evaluated | `test_r8_a_checkpoint_of_the_mock_environment_is_refused_as_agent_and_as_opponent` | fails, as it must |
| R8c a missing file is not said before the unpickler runs, and a folder ends in a traceback | `test_r8_a_path_that_is_no_checkpoint_is_refused_in_one_line` | fails, as it must |

41 of 41 wrong changes are caught.

### Readings: where I chose

1. **Temperature as a float.** `temperature_c` was a whole number; it is a float now, also when nvidia-smi prints "54". Nothing in the repo reads it as a whole number.
2. **Where the memory is read.** "After acting": right before `ppo.update`, when the acting cache is at its largest. "After the update step": right after `ppo.update` and the wait for the card.
3. **The limit is applied by the `Trainer`**, not only by `train.main`, so a trainer made in code gets it too. It holds for the whole process and is not taken back when the trainer is closed.
4. **0 means "set nothing"** in a new process, not "set the limit to the card's size": the files of the finished runs then mean exactly what those runs did. Only a process that set a limit before sets the card back to its whole size (review, point 5).
5. **Checkpoints from before the key** are read with 0 (`LATER_KEYS`). This is the one place where the code supplies a value a file does not have. It is limited to keys listed there with the value their runs had.
6. **Criterion 6 asks for candidates "in the order to try".** I read it as: stop at the first that meets criterion 7. I tried two values of the first candidate and none of the others.
7. **The opponent's seed is `--seed` + 1**, as random legal play uses `--seed` + 1 for its numbers. Since the review each network has one generator for the whole evaluation.
8. **`<run of B>` in the file name** is B's run folder (`long-a-mlp-20261124-000454`) since the review; at the first hand-over it was the run's name and network, and two runs of one name gave one file.
8a. **`--overwrite` belongs to `--opponent` only.** The file of the evaluation against random play is replaced by the next call, as before spec 015 (criterion 11, and the tests of spec 008 rely on it). See "Open issues".
9. **`beats_random` is left out with an opponent** and `above_zero` and `below_zero` are added. Reading the old key for such a file is an error, which is intended.
10. **Criterion 3 on "this PC".** The GPU test runs the small config for one update on the card. The measured runs are the second evidence.

### What failed, and what did not go by the book

1. **The order of D73 slipped once.** The tests of parts A, C and of the memory tags were written before their code. For the key of the remedy I wrote the code about ten minutes before its tests, while the second trial was running; nothing was run in between. The tests then found a defect in that code: `limit_gpu_memory` passed a device without a number to two PyTorch calls that want the card's number. Fixed, with the test's own wrong assumption about that.
2. **M1 was disturbed from update 29 on** by another program on the card. Its memory readings are used, its timing after update 28 is not.
3. **T3 ran partly under CPU load** from the main session's test run. No trace in its numbers; said in the table.
4. **The pairs and T1 ran while this session was stopped** by a usage limit (10:06 to 10:55). The script checked the card before each; the logs and samples were read afterwards.
5. **My tests did not find the defect of the review's point 1.** The pattern was there before (a new generator from the same seed for every environment) and I copied it for the opponent. My test of criterion 15 used one seed and passed. The separate review found it by counting over 20 seeds.
6. **`test_c20_a_python_thread_keeps_running_during_a_long_step` of spec 006 fails at times.** It depends on what else the PC does, in the direction spec 008 reported: it tends to fail on the idle PC. Today: failed once in a suite run at 09:25 and passed alone right after; passed in the three suite runs of 14:06 to 14:20; failed in both suite runs of 18:40 to 18:48 and in 3 of 3 runs alone at 18:55, with the PC at 0.5% CPU load. The main session saw it fail in a suite run and pass alone. The test, `tarok_train/env.py`, the binding crate and the engine are unchanged by this diff (`git diff --stat` for them is empty), and the module's fingerprint is the same. Cause not established; see spec 008.

### Open issues

1. **No config file carries the limit.** The six files keep 0 (criterion 9). The next transformer run on this card needs `gpu_memory_limit_mib = 5000` in its config, or `--gpu-memory-limit-mib 5000`. For the user: (a) the next run config sets it, recommended; (b) also change `long-a-transformer.toml` and `smoke-a-transformer.toml` to 5,000, which makes them differ from the runs they describe.
2. **The value 5,000 is measured for these sizes only**: 512 tables, 2,048 seat-deals per update, this card, under 700 MiB of other programs. A run whose tensors need more stops with an out-of-memory error. It can be resumed with a higher limit. The tensors' peak was 4,441 MiB over 200 updates and 4,364 MiB over the 2,000 updates of the long run.
3. **The limit does not protect against another program on the card.** In M1 a program that took 500 MiB made four updates take 24 to 503 s. With the limit the training holds about 500 MiB less than without it, so it is less exposed, but a program that takes more than the card has left will still push it into shared memory. A training run needs the card to itself.
4. **The acting cache is probably twice as large as it needs to be.** Computed from its shape: one position costs 16 MiB with 512 tables; it starts at 34 positions and doubles (68, then 130). The tensors between the phases grow by 991 MiB at update 3, which is 62 positions: the step from 68 to 130. Inferred: a sequence of 67 or more tokens was met (the longest among the trained seat-deals is 66; seats of a past checkpoint and dropped deals are not in that number). A check on the CPU with 64 tables did not reach it in three updates (longest sequence 65 tokens, 68 positions). If the cache grew by what is needed and not by doubling, the tensors would be about 0.9 GB smaller between the phases and at their peak. That is a change in `model.py`, outside this spec. For the user, with Batch 14.
5. **The benchmark of spec 003** (`bench_model.main`) still prints nothing when its GPU reading is empty. Its result file then has `gpu_before: {}`. One line would say it; not done, the criteria name the trainer.
6. **The numbers of the analysis in this spec** (the section "Analysis behind this spec") come from the scratch script. The table of criterion 16 is the version from committed code; item 1 of the analysis quotes the gains of the last 500 updates, which differ between the three columns (see above).
7. **The smoke run of spec 008 was probably slowed by the same spill**, by about 15% (inferred from C7). The speed numbers of spec 008's Results (6,506 played decisions per second for the transformer) are then low for a card with the limit: 7,437 here.
8. **Evaluations of the untrained network in spec 008 have intervals that are too narrow.** Measured on its files: the tie across chunks is 0.23 to 0.30 for the three `evaluation-untrained.json` (smoke MLP, smoke transformer, control), which understates the standard error by about 1.3 to 1.4 times. For the trained checkpoints of specs 008 and 014 the tie is 0.03 at most, and their intervals stand. The Results of spec 008 are not changed here; for the main session.
9. **An evaluation against random play no longer repeats the digits of the files of specs 008 and 014.** Same deals, same random legal play, other random numbers for the agent. Example: the checkpoint of criterion 8, +0.6015 before and +0.5997 now, on the same deals.
10. **`--overwrite` and the evaluation against random play.** Its file is still replaced without a question, also by a call with another seed. For the user: (a) leave it, as criterion 11 says, recommended for now; (b) the same rule as with `--opponent`, which changes spec 008's behaviour and at least one of its tests.
11. **Seed, deals and tables are not in the file name.** Two seeds for one pair need two calls with `--overwrite` and a copy in between, or a longer file name. Not changed: criterion 12 fixes the form of the name.

### Review fixes (2026-11-24, after the D72 step 4 review)

The separate review found one real defect in part C and seven lesser points. I checked each in the code before changing it; all eight hold. The tests for each were written first and failed on the code as it was (23 failing tests at that moment). Tests named `test_r<n>_...` belong to point n. No training run was repeated.

| Point | Checked | What it led to | Tests |
|---|---|---|---|
| 1. The sampling noise is reused | Holds. Measured on the code as it was (a network of the small config after two updates against itself, 1,000 deals in four chunks, CPU): the deal values of one table index correlate at +0.72 across chunks, and the control interval excludes zero for 4 of 6 seeds | One generator per network for the whole evaluation (`network_agent` makes it once; every environment draws on from it). After the fix on the same recipe: correlation -0.005, 0 of 6 intervals exclude zero. The pairs and the evaluation of criterion 8 were run again | `test_r1_two_environments_of_one_evaluation_do_not_get_the_same_random_numbers`, `test_r1_the_deal_values_of_one_table_index_are_not_tied_from_chunk_to_chunk`, `test_r1_the_result_does_not_depend_on_the_number_of_threads`; `test_c15` now runs 20 seeds |
| 2. Two wrong changes survive | Holds: every test used two checkpoints of one kind from one config | A test with an MLP and a real transformer of different model sizes, both ways round. It compares the rewards with a reference written out in the test (a fresh actor for every call, its own two generators), and it records with which network and which model config every actor is built | `test_r2_two_networks_of_different_kinds_and_sizes_each_play_with_their_own_config`; wrong changes R2a and R2b |
| 3. Evaluations overwrite each other | Holds | The opponent's run folder in the file name; `tables` and `threads` in the file; an existing file is refused without `--overwrite`, in one line that names it; with `--overwrite` the records of the replaced call go too | `test_r3_an_existing_result_is_not_replaced_without_overwrite`, `test_r3_two_opponents_with_one_run_name_from_two_run_folders_give_two_files` |
| 4. Resume puts the checkpoint on the card before the limit | Holds | `Trainer.load` reads the checkpoint to the CPU; the trainer then sets the limit and moves the weights | `test_r4_on_resume_the_limit_is_set_before_anything_of_the_checkpoint_is_on_the_card` (GPU) |
| 5. A limit of 0 does not lift an earlier limit | Holds | A process remembers the cards it has limited. A trainer without a limit sets such a card back to the whole card and the run says "lifted", not "no limit" | `test_r5_...` (three tests, one on the GPU) |
| 6. A limit of the card's nominal size may be refused | Holds on this PC: PyTorch reports 6,143.7 MiB for the card of 6,144 MiB, so 6,144 was refused | A limit up to 1% above what PyTorch reports is read as the whole card (fraction 1.0); more is refused | `test_r6_a_limit_of_the_cards_nominal_size_is_taken_when_pytorch_reports_a_little_less` |
| 7. `nan` as a temperature | Holds | A temperature that is not finite gives an empty reading. Two cards: left as it is, said in the README and held by a test | `test_c02_output_that_cannot_be_read_gives_an_empty_reading` (four new outputs) |
| 8. Paths and mock checkpoints | Holds. A mock checkpoint was evaluated on the engine without a word | One line and exit code 2 for: no such file, a folder, a file that is no checkpoint, a checkpoint of a run on the mock environment; for the agent and for the opponent | `test_r8_a_path_that_is_no_checkpoint_is_refused_in_one_line`, `test_r8_a_checkpoint_of_the_mock_environment_is_refused_as_agent_and_as_opponent` |

Where I differ from the review, or went another way:

- **Point 1, the form of the fix.** I took "one generator per network for the whole evaluation", not a seed per chunk and seating. It keeps the interface of `evaluate.evaluate` and needs no rule for deriving seeds. The price: the result depends on the order in which the environments are played, which is fixed (chunks in order, seats 0 to 3), and on `--tables`. It does not depend on `--threads` (tested).
- **Point 2, what the reference can and cannot see.** The reference catches a network in the wrong seat and wrong weights. It cannot catch the opponent's actor getting A's model config: the two keys the actor reads from it only change the speed, not the actions. That wrong change is caught by the record of how every actor was built.
- **Point 3, `--overwrite`.** The refusal holds with `--opponent` only. Without it the file of the evaluation against random play is replaced as before spec 015; criterion 11 asks that nothing changes there, and a test of spec 008 calls the evaluation twice for one checkpoint. Open issue 10 puts the choice to the user.
- **Point 5, "0 really means no limit".** PyTorch has no call that takes a limit back. The widest setting is the whole card (fraction 1.0), and that is what a trainer without a limit now sets when the process limited the card before. It is not the same as a process that never set a limit: there the driver may hand out more than the card has. In a new process a limit of 0 still sets nothing, so the finished runs' configs mean what those runs did. The line at the start says which of the two holds.
- **Point 6, the tolerance** of 1% is my choice.

Consequences of point 1 for numbers that exist:

- The twelve pairs: run again, table under criterion 16. Nothing changes in what they say.
- Criterion 8: +0.5997 (+0.5914 to +0.6080) from the fixed code, +0.6015 (+0.5931 to +0.6099) before. Both overlap the smoke run's interval.
- The evaluations of trained checkpoints in specs 008 and 014: tie across chunks 0.03 at most; their intervals stand. Their digits cannot be repeated with the fixed code.
- The evaluations of the untrained network in spec 008: tie 0.23 to 0.30; their intervals are too narrow (open issue 8).

One existing test changed with point 6: `test_c06_a_limit_above_the_cards_memory_is_refused` asked that 6,145 MiB be refused on a card of 6,144; it now asks that for 6,300.
