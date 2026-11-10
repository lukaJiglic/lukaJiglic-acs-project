# Standing instructions for agent sessions

Project: Tarok trener, a self-play-trained tarok agent with a web app around it. Course project; the process is graded. The user decides; agents propose and execute.

## Where the state is

- `docs/planning/README.md`: read order for the planning files and what to do next. Read it first.
- `docs/planning/decisions.md`: every decision with its number (D…), date and who made it. Do not re-ask decided items.
- `docs/planning/open-questions.md`: open batches and clarifications.
- `docs/build-plan.md`: order of the specs.
- `docs/specs/`: one file per task. `docs/progress.md`: one entry per session. `docs/architecture.md`: the architecture as built.
- `docs/planning/ai-log.md`: one row per session. Needed for the graded AI-use declaration.

## Session routine (D78)

Start: read `docs/planning/README.md`, `docs/progress.md` and the open specs. Work in spec order.
End: update `docs/progress.md`, the spec status and `docs/planning/ai-log.md`; commit; push. Exception: a parallel track session commits nothing and updates only its own spec file (see Hard rules).

## Ask, do not assume

- Every decision goes to the user. Ask in batches; give options, one recommendation and the reason for each item.
- Record answers in `decisions.md` with the next D number, then update the plan and the open questions.
- For research, say what was checked in a source and what is inferred.
- A spec is executed only after the user approved its acceptance criteria (D70, D73). Tests are written from the criteria before the code.

## Verification before each commit (D72)

1. The spec's acceptance criteria are met and named in the commit.
2. Tests pass locally (`cargo test`, `pytest`, app tests, `python tools/check_skeleton.py`, as applicable).
3. The user reads the diff. Code is not committed before that.
4. A separate agent review pass over the diff, apart from the writing pass.
5. CI is green after the push.

Weights also pass the golden forward-pass test. Rule changes also pass the scenario tests for that rule.

## Commit convention (D74)

`type(scope): subject` with types feat, fix, test, docs, plan, chore, train, model. The body says what and why. Trailers:

```
Spec: docs/specs/NNN-name.md            (when a spec applies)
Produced-by: Claude Code <version>, model <tag>     (or: Produced-by: user)
Session: ai-log row N
Co-Authored-By: <model name> <noreply@anthropic.com>
```

One commit per task or logical step. Trunk-based on `main`; a branch only for a long or risky change (D77). `python tools/check_trailers.py` checks the trailer.

## Hard rules

- No code from other tarok engines (D9). General libraries are allowed (D27).
- No LLM narration in the product, ever (D16). Explanation text is templates bound to model signals.
- Commands shown to the user are Windows cmd syntax. Prose is short, direct sentences.
- `project/` holds course material and stays untracked (D28).
- Parallel sessions share one checkout: touch only your spec's files, commit nothing, write results into your spec's "Results" section (D109).
- Training runs: a config per run in `src/train/configs/`, a run log in `docs/progress.md`, released weights in `models/` with a `MODEL.md` (D79).
