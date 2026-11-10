# 001 — Repo skeleton

Status: **approved** 2026-11-10 (D112); executed 2026-11-10; the user read the diff and approved the commit on 2026-11-10 (D114). Done once CI is green (criterion 8). Answers to the open points: A force LF; B holder as written; C `tools/` accepted.

Order-of-work step 0 (D82). Tool and model: Claude Code 2.1.306, `claude-fable-5-2`.

## Goal

Make the repository meet the course's requirements for pitch day (R5–R9) and set up the files every later session relies on (D71). No product code.

## Acceptance criteria

The user approves these before anything is written (D73).

1. **`README.md` at the root** has these sections: what the project is (three sentences); goals G1–G19, numbered, copied from `docs/planning/plan.md` section 4; the agentic stack table with tool, pinned model tag, date range, what it did (two model rows: `claude-fable-5-1` 2026-11-06 to 2026-11-10, `claude-fable-5-2` from 2026-11-10); install, run and demo sections, each marked "not yet available" with the planned command; not in scope (D86); licences; a link to `docs/planning/`.
2. **`CLAUDE.md` at the root** holds the standing instructions: session routine (D78), verification checklist (D72), commit convention (D74), "ask, do not assume" and the batch rule, cmd syntax for user-facing commands, no code from other tarok engines (D9), no LLM narration (D16), where the state files are.
3. **`docs/progress.md`** exists with the entry format (date, session, done, measurements, broken, next) and one entry per past session (2026-11-06, 2026-11-09, 2026-11-10), written from `ai-log.md`.
4. **`docs/architecture.md`** exists with the planned architecture from `plan.md` section 2, marked "planned, not built", and one diagram in text form (engine → native/WASM/PyO3 → app/training).
5. **`docs/specs/README.md`** exists with the spec template (status line, goal, acceptance criteria, files, out of scope, tool and model, verification) and the numbering rule.
6. **Licences:** `LICENSE` (MIT, code) and `LICENSE-assets` (CC BY 4.0, cards and texts), holder "Luka Jelenko Iglic", year 2026 (D61).
7. **Directory layout (D56):** `src/engine`, `src/app`, `src/train`, `src/assets/cards`, `models`, each with a one-paragraph `README.md` saying what will live there. `.github/workflows` exists.
8. **CI (D63, first slice):** `.github/workflows/ci.yml` runs on every push to `main`. One job checks that every commit since `0d5045d` has a `Produced-by:` trailer (the measurable half of G19). The job passes on the current history. Each later track adds its own workflow file (`engine.yml`, `train.yml`), so parallel sessions never edit the same file (D109).
9. **Line endings:** `.gitattributes` forces LF for text files (`* text=auto eol=lf`), so the CRLF warnings stop. See open point A.
10. **Nothing else changes.** `docs/planning/` content is untouched except `README.md` (pointer to the new files) and `ai-log.md` (row for the session).

## Checks (how each criterion is verified)

- 1–7, 9: a script `tools/check_skeleton.py` asserts that each file exists and contains its required headings; it prints one line per criterion. Run locally before the commit.
- 1: the goals in `README.md` are compared to `plan.md` section 4 by the same script (same count, same IDs).
- 8: the workflow run is green on GitHub after the push.
- The user reads the diff (D72 step 3). A separate agent review pass reads the diff against this spec (D72 step 4).

## Files expected to change

New: `README.md`, `CLAUDE.md`, `LICENSE`, `LICENSE-assets`, `docs/progress.md`, `docs/architecture.md`, `docs/specs/README.md`, `src/engine/README.md`, `src/app/README.md`, `src/train/README.md`, `src/assets/cards/README.md`, `models/README.md`, `.github/workflows/ci.yml`, `tools/check_skeleton.py`, `tools/check_trailers.py` (added during execution; criterion 8 needs it).
Changed: `.gitattributes`, `docs/planning/README.md`, `docs/planning/ai-log.md`.

## Out of scope

- Any engine, app or training code.
- GitHub Pages deployment (comes with the app shell).
- The report skeleton in `report/` (spec 002, D85).
- Making the repository public. That is the user's action (R5, by 23 Nov).

## Commits

One commit per logical step, each with `Spec: docs/specs/001-repo-skeleton.md`: (a) README, licences, layout; (b) CLAUDE.md, progress, architecture, spec template; (c) CI and `.gitattributes`.

## Open points for the user

- **A. Line endings.** Force LF in the repo (recommended; Rust, Python and the web tools all expect it, and Git Bash is the agent's shell), or keep `text=auto`?
- **B. Licence holder name** as written above, or another form?
- **C. `tools/` directory** for small check scripts is not in D56. Accept it, or put the script under `src/`?

## Results

Executed 2026-11-10 by Claude Code 2.1.306, `claude-fable-5-2`. **Not committed: waiting for the user's read of the diff (D72 step 3).**

Order of work: the two check scripts were written first and failed on the empty tree (0 of 10), then the files were written.

| Criterion | Result | Evidence |
|-----------|--------|----------|
| 1 README | met | Sections present; goals G1–G19 taken from `plan.md` section 4 by script, aspirations left out; stack table with both model tags |
| 2 CLAUDE.md | met | Routine, checklist, commit convention, hard rules |
| 3 progress.md | met | Format and three dated entries from the AI log |
| 4 architecture.md | met | Marked "planned, not built"; text diagram |
| 5 specs/README.md | met | Template, numbering rule, list |
| 6 Licences | met | MIT full text. CC BY 4.0 as a notice with a link to the legal text, not a copy. Scope: cards and app texts only |
| 7 Layout | met | Five README files; `.github/workflows` exists |
| 8 CI | **not yet verifiable** | `ci.yml` and `tools/check_trailers.py` exist; the script passes locally (7 of 7 commits). The run on GitHub happens only after the commit and push |
| 9 Line endings | met | `* text=auto eol=lf`; CRLF kept for `.bat` and `.cmd`; binary types listed |
| 10 Nothing else changes | **deviation** | One file outside the list: `tools/check_trailers.py`. Needs the user's acceptance |

Local check output:

```
python tools/check_skeleton.py   -> 10 of 10 checks passed (criteria 1-9)
python tools/check_trailers.py   -> 7 of 7 commits since 0d5045d carry a Produced-by trailer
```

Review pass (D72 step 4): a separate subagent read the diff against this spec and reported 13 defects. Fixed: the trailer check failed on pull-request merge commits; a false claim in the README that every commit carries the trailer; a missing "Checks" heading in the spec template; a wrong session label in the progress log; missing planned commands in the README; the licence scope reached into `docs/` and `report/`; batch files and weight formats missing from `.gitattributes`; a short commit id and a loose trailer match in the check script; two wrong decision numbers; one long sentence. Left for the user: the deviation in criterion 10.

Not verified: the GitHub run; whether the CRLF warnings stop once the files are committed under the new `.gitattributes`.
