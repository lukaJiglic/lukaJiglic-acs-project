# Planning state — tarok app (ACS individual project)

This folder holds the full state of planning. A fresh session should read it and continue without the chat history.

## Read in this order

1. `course-requirements.md` — what the course demands. Facts taken from the instructions PDF and the report template.
2. `decisions.md` — what is decided, by whom, and when. Also the givens from the user's brief.
3. `open-questions.md` — the current batch waiting for the user, then the backlog of later batches.
4. `plan.md` — the plan. Every item points at a decision number; the idea pool is kept for the record.
5. `research/` — research notes. Each file ends with a "Verified vs inferred" section.
6. `ai-log.md` — log of AI use. Needed for the graded AI-use declaration.

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
- Commits follow D74: `type(scope): subject`, body, trailers `Produced-by: Claude Code <version>, model <tag>`, `Session: ai-log row N`, `Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>`. Use type `plan` while planning. In use since commit 0d5045d.
- Large multi-file edits: write a Python script to the scratchpad and run it with `python -I`; a heredoc with a long script fails in the Bash tool.
- `project/` (course PDF, template originals) is gitignored and must stay untracked (D28). The report source is in `report/`.

## Status

See the top of `open-questions.md` for where planning stands. As of 2026-11-10: all eight decision batches answered (D1–D88), no open batch, no open clarification. `plan.md` is the plan. Two provisional choices (P2, P5 in `decisions.md`) are in use but never explicitly confirmed by the user.

## What a fresh session does next

1. D50: write the screen spec for the T0 screens (`plan.md` section 1, D33–D37), get it approved, then the Figma wireframe through the Figma connector, then the pitch slide text. Due Wed 18 Nov with the proposal.
2. Derive the proposal text from `plan.md` (D87). Format of the proposal is not stated in the course text; ask the user.
3. Remind the user of the two actions only they can take: confirm the topic with the lecturer on Mon 16 Nov (R2); make the repo public or add the instructor by Mon 23 Nov (R5).
4. Building starts only when the user says so. The first building step is order-of-work step 0 (D82): README with goals, root `CLAUDE.md`, `docs/progress.md`, CI, licences.
