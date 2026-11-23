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
2. Check `docs/specs/`: each spec's status line says draft, approved, or done, and approved specs carry a "Results" section once executed. Execute only approved specs (D70, D73). Code changes are committed only after the user has read the diff (D72).
3. As of 2026-11-23: specs 001–008 and 013 are done. Spec 014 (a longer run on deal flow A: both networks, 2,000 updates each; D138) is in execution in the `train-env` session; it waits for a free GPU and then needs about seven hours. When it reports: verify the numbers from the run files, write the run log into `docs/progress.md`, ask the user to read the two configs, commit. Open with the user: C13, C16, the open points A–D of spec 014; later C14. Next in the build plan: specs 009 (flow B), 010 (flow C), 011 (bots and arena); D135 (replay names both engine versions) is built with spec 009.
4. Still owed for Wed 18 Nov: the proposal text (format unknown; ask), corrections to the wireframe (frames A-3 and A-5 need the D104 step) and to `pitch.md`.
5. User-only actions: confirm the topic with the lecturer on Mon 16 Nov (R2); make the repo public or add the instructor by Mon 23 Nov (R5); share the Figma file with the lecturer.
