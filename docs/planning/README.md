# Planning state — tarok app (ACS individual project)

This folder holds the full state of planning. A fresh session should read it and continue without the chat history.

## Read in this order

1. `course-requirements.md` — what the course demands. Facts taken from the instructions PDF and the report template.
2. `decisions.md` — what is decided, by whom, and when. Also the givens from the user's brief.
3. `open-questions.md` — the current batch waiting for the user, then the backlog of later batches.
4. `plan.md` — the plan. Every item points at a decision number; the idea pool is kept for the record.
5. `screen-spec.md` — the T0 screens (draft 2). The Figma wireframe built from it: https://www.figma.com/design/XVrAGNVsQuisOd79HYbRBg (overview image in `docs/wireframe/`). `pitch.md` — the one-slide pitch text, draft.
6. `rule-switches.md` — the rule toggles, presets, left-out list and decided defaults.
7. `research/` — research notes. Each file ends with a "Verified vs inferred" section.
8. `ai-log.md` — log of AI use. Needed for the graded AI-use declaration.

Outside this folder: `CLAUDE.md` at the repo root (standing instructions), `docs/build-plan.md` (order of the specs), `docs/specs/` (tasks), `docs/progress.md` (one entry per session), `docs/architecture.md`.

## Rules for any session working on the plan

- Planning only until the user says building starts.
- Ask the user instead of assuming. Every decision goes to the user unless they said otherwise.
- Ask in batches. For each decision give the options, a recommendation and the reason.
- Nothing is locked early. Scope stays open until the user locks it.
- Question each step, including the original idea.
- Bring new ideas, not only the ones already raised.
- For research: say what was checked and what is inferred.
- After each session: update `decisions.md`, `open-questions.md`, `plan.md` and add a row to `ai-log.md`.
- Commands shown to the user must be Windows cmd syntax.
- Prose style: short, direct sentences.
- Commits follow D74: `type(scope): subject`, body, trailers `Produced-by: Claude Code <version>, model <tag>`, `Session: ai-log row N`, and a `Co-Authored-By:` line naming the model in use (Claude Fable 5.2 since D89). Use type `plan` while planning. In use since commit 0d5045d.
- Large multi-file edits: write a Python script to the scratchpad and run it with `python -I`; a heredoc with a long script fails in the Bash tool.
- `project/` (course PDF, template originals) is gitignored and must stay untracked (D28). The report source is in `report/`.

## Status

See the top of `open-questions.md` for where planning stands. As of 2026-11-10: the eight plan batches are answered (D1–D88). The model changed to Claude Fable 5.2 (D89). Batch 9 is answered (D90–D102) and the user said work can begin (D103). C8–C10 are answered (D104–D106). Building has started in the sense of D106: specs first, training as early as possible. Batch 10 is answered (D107–D113): the first run trains every phase; specs 001–003 are approved. Specs 004–007 approved (D117–D120), ten score parts (D121), specs 005 and 007 committed (D122). Open: C11 and Batch 11. `plan.md` is the plan. Two provisional choices (P2, P5 in `decisions.md`) are in use but never explicitly confirmed by the user.

## What a fresh session does next

1. Read `docs/build-plan.md` (outside this folder), draft 2. It orders specs 001–012 up to the first full training run.
2. Check `docs/specs/`: each spec's status line says draft, approved, or done, and approved specs carry a "Results" section once executed. Execute only approved specs (D70, D73). Since D144 (2026-11-24) the agent commits without asking, after the tests and a separate review pass, and reports each commit.
3. As of 2026-11-23: specs 001–008 and 013 are done. Spec 014 (a longer run on deal flow A: both networks, 2,000 updates each; D138) is in execution in the `train-env` session; it waits for a free GPU and then needs about seven hours. When it reports: verify the numbers from the run files, write the run log into `docs/progress.md`, ask the user to read the two configs, commit. On 2026-11-24 at 00:04 the GPU was free and the run started. Spec 009 (flow B) is drafted on the same day; it waits for the user's approval of its criteria and for Batch 13, and its execution must not touch an engine source file before the run of spec 014 and its evaluations are finished (the stale-module check). The run ended on 2026-11-24 at 08:12; spec 014 is committed (D139). The user ordered an analysis, fixes of the problems found and to go on (D140). The analysis is at the top of spec 015; spec 015 (GPU reading, transformer memory, a checkpoint as opponent in the evaluation) is in execution in the `train-env` session. When it reports: verify, review pass, the user's read of the diff, three commits. Later on 2026-11-24 the user answered Batches 13 and 14 (D141, D142), approved spec 009 (D143), dropped the question before every commit (D144) and had the repo made public (D145). Spec 009 is in execution in the `engine-core` session on the branch `spec-009` (worktree `.worktrees/spec-009`); the binding module is not rebuilt before the GPU work of spec 015 is finished. After both: merge `spec-009`, then spec 016 (the trainer's side of the aligned updates; D142, B14-1) and spec 017 (the trial of the lower entropy limit; B14-2). Both are approved (D146). Order of the GPU work: spec 015, then spec 017 on the installed engine 0.2.0, then the rebuild of the binding for spec 009 and the merge of `spec-009`, then spec 016. Batch 16 is answered too (D147): a fifth case of the public partner goes into spec 009 on the branch. The review pass over spec 009 found no rule defect (progress entry of 2026-11-24); Batch 17 is answered (D148): three more such cases and a guard by engine version go into spec 009, and spec 018 (first runs on flow B; a draft, Batch 18) comes before the measurement of spec 016. After spec 009 the engine plays the round of announcements in every deal; there is no switch back to flow A, which is why Batch 15 asks whether spec 017 runs before the binding is rebuilt. Both side sessions stopped at the usage limit on 2026-11-24 and were restarted; their state is in the progress entry of that afternoon. Open with the user: Batch 18, C13, C16, the open points A, B and D of spec 014; later C14. Next in the build plan: specs 009 (flow B), 010 (flow C), 011 (bots and arena); D135 (replay names both engine versions) is built with spec 009.
4. Still owed for Wed 18 Nov: the proposal text (format unknown; ask), corrections to the wireframe (frames A-3 and A-5 need the D104 step) and to `pitch.md`.
5. User-only actions: confirm the topic with the lecturer on Mon 16 Nov (R2); the repo is public since 2026-11-24 (D145, R5); share the Figma file with the lecturer.
