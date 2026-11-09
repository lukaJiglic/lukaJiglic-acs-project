# Course requirements

Source: `project/project_instructions.pdf` and `project/project_report.tex` (+ `erk.sty`). Read on 2026-11-06.
Course: Agentic Computer Science, UL FRI, 2026/27, doc. dr. Blaž Meden. The project is 70% of the grade.

## Deliverables and dates

| ID | Deliverable | Due | Weight |
|----|-------------|-----|--------|
| D1 | Proposal + Figma wireframe + one-slide pitch (3 min) | Wed 18 Nov 2026 | 10% |
| D2 | Checkpoint: 5-min lightning talk + written sprint deliverable | Mon 14 Dec 2026 | 10% |
| D3 | Report: conference paper in ERK format, English | Mon 11 Jan 2027 | 25% |
| D4 | Codebase: working software + demo + repository | Mon 11 Jan 2027 | 25% |

Late penalty: 10% of that deliverable's points per calendar day, capped at −50%.

## Calendar

| Date | Event |
|------|-------|
| Mon 16 Nov | Project scoping + Figma session |
| Wed 18 Nov | D1 due |
| Mon 23 Nov | Pitches (3 min, one slide). Repo must be live. Sprint 1 starts. |
| 30 Nov – 13 Dec | No sessions (lecturer on leave). Work independently. |
| Sun 13 Dec | Sprint-1 milestone self-submitted |
| Mon 14 Dec | Checkpoint: lightning talks + written sprint deliverable |
| Mon 21 Dec | Sprint review. Winter break after. |
| Mon 4 Jan | Exam (30%, lectures only, not the project) |
| Mon 11 Jan | Demo Day, 10-minute slots. Entire project due. |

From 2026-11-06: 12 days to D1, 38 days to D2, 66 days to D3/D4.

## Topic

- R1. One topic per project.
- R2. Topic comes from the Moodle list (browser game, dashboard, tracker, organizer, generator, …) **or** is approved at consultation before the proposal deadline (18 Nov).
- R3. A good project is "small enough to finish, mostly buildable by agents, demoable in 10 minutes".
- R4. Graded on the quality of the process and of the results. Not on polish a large model produces when asked nicely.

## Repository

- R5. Name: `<username>-acs-project` on GitHub. Public or accessible to the instructor.
- R6. Required from pitch day (23 Nov).
- R7. `README.md` must state: what the project is; goals and objectives (numbered, checkable); the agentic stack used (every tool, every pinned model tag); how to install, run and demo.
- R8. `src/` holds source code. `docs/` holds screenshots and architecture notes.
- R9. Commit history is graded. Frequent commits, real messages, no bulk dump at the end.

## Report (ERK two-column template, English)

Sections forced by the template:

- R10. Abstract, max 200 words: what the software does, who it is for, what the agents did, one result.
- R11. Introduction: the problem and the user. Max two short paragraphs.
- R12. Goals and objectives: numbered (G1, G2, …), each testable by a reader.
- R13. Agentic stack table: Tool | Model (pinned tag) | What it did. "As used, not as advertised."
- R14. Method: how work was split, what files carried state (specs, progress files), how each agent output was verified before committing. The commit log is the replication record: which prompt and which model produced which part.
- R15. Results: what works, how you know (tests, demo, counts). Failures are reported too.
- R16. Limitations and next steps: product and process.
- R17. AI-use declaration: what agents produced vs what the student produced; every model with exact pinned tag and date of use; what was auto-generated at release (model defaults). Graded.
- R18. Bibliography entry for the repository with the branch/tag used for the demo.

Other:

- R19. Nothing is banned, everything is declared: local Ollama, the shared dingo server, any cloud model. Name it, pin its version, say what it did.
- R20. Best reports may be submitted to ERK 2027 (Portorož). Recognition only, not graded.

## Consequences for planning (inferred, not stated by the course)

- The proposal and Figma wireframe are due in 12 days. Product scope and the main screens must be decided first.
- R14 means every commit should be traceable to a prompt and a model. The workflow plan needs a commit-message or log convention that records this.
- R17 means planning sessions are declared too. `ai-log.md` covers this.
- R3 conflicts with a very wide scope. The plan needs tiers and a small core that is safe to finish.
