# Task specs

Every task is one file here (D70). Claude drafts it, the user approves the acceptance criteria, an agent executes it, verification runs, the commit references it.

## Rules

- File name `NNN-short-name.md`. NNN is the next free number, three digits. Numbers are never reused and specs are never deleted.
- The status line at the top is one of: draft, approved, done, dropped. It carries the date of the last change. A spec is "done" only after its changes are committed, which needs the user's read of the diff (D72).
- "Checks" says how each criterion is verified.
- Tests are written from the acceptance criteria before the code (D73).
- An executed spec gets a "Results" section: what was built, the output of the checks, measured numbers, what failed.

## Template

```
# NNN — Title

Status: **draft** (YYYY-MM-DD).

Tool and model: ...

## Goal
## In scope
## Out of scope
## Acceptance criteria
## Checks
## Files expected to change
## Commits
## Open points for the user
## Results
```

## List

| # | Spec | Status |
|---|------|--------|
| 001 | [Repo skeleton](001-repo-skeleton.md) | done 2026-11-10 |
| 002 | [Engine core](002-engine-core.md) | done 2026-11-10 |
| 003 | [Training environment and benchmark](003-training-environment.md) | done 2026-11-11 (GPU-free run added) |
| 004 | [Encoding contract](004-encoding-contract.md) | done 2026-11-11 |
| 005 | [Deal flow A](005-deal-flow-a.md) | done 2026-11-11 |
| 006 | [Encoding, environment, bindings, record](006-bindings-and-environment.md) | done 2026-11-12 |
| 007 | [Model and PPO learner](007-model-and-learner.md) | done 2026-11-11 |
| 008 | [Pipeline smoke run](008-pipeline-smoke-run.md) | done 2026-11-23 |
| 009 | [Deal flow B: announcements and kontra](009-deal-flow-b.md) | approved 2026-11-24 (D143); in execution (session `engine-core`, branch `spec-009`) |
| 013 | [Three rulings of 12 November](013-rulings-of-12-nov.md) | done 2026-11-12 |
| 014 | [Longer run on deal flow A](014-longer-run-flow-a.md) | done 2026-11-24 |
| 015 | [Findings of the longer run](015-findings-of-the-longer-run.md) | approved 2026-11-24 (D140); in execution (session `train-env`) |
| 016 | [Aligned updates: every played deal is trained on](016-aligned-updates.md) | draft 2026-11-24; waits for approval (Batch 15) and for spec 009 |
| 017 | [Trial: the entropy controller's lower limit at 0.001](017-entropy-floor-trial.md) | draft 2026-11-24; waits for approval (Batch 15) and for spec 015 |
