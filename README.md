# Tarok trener

Individual project for the course Agentic Computer Science (UL FRI, 2026/27).

## What this is

A trained agent that plays four-player Slovenian tarok, and a phone-first web app built around it. The agent learns by self-play on one PC. The app uses it three ways. You learn by playing with a coach that explains the agent's choice. You play against bots of measured strength. You get advice at a real table by entering the cards.

Explanations come from the model's own signals (values, beliefs about hidden cards, attributions, counterfactuals). No language model writes them.

**State on 2026-11-10: planning is finished and building has started. No product code exists yet.** The plan is in [docs/planning/plan.md](docs/planning/plan.md), the order of work in [docs/build-plan.md](docs/build-plan.md), the progress log in [docs/progress.md](docs/progress.md).

## Goals

Numbered and checkable. Thresholds are the realistic targets; higher aspirations are listed in the plan and are not goals. The numbers may change until the proposal is submitted and are frozen after that.

**Rules and engine**

- G1. Rule-scenario suite of at least 150 tests covering the baseline and every T0 toggle value, plus property tests (card conservation, 70 points per deal, legal-move sets) on 10,000 random deals. `cargo test` in CI.
- G2. Native, WASM and PyTorch agree on legal-move sets and scores for 1,000 random games; forward-pass outputs within 1e-4 fp32 / 1e-2 fp16 on 1,000 random states. Parity tests in CI.

**Pipeline**

- G3. End-to-end self-play throughput of at least 10,000 steps/s on the training PC. Stage-0 benchmark, logged in `docs/progress.md`.

**Agent strength** (seeded duplicate deals, eight seatings)

- G4. Transformer agent beats the rule bot: positive mean score per deal, 95% CI excluding zero, at least 1,000 deals.
- G5. Zero illegal moves in 10,000 self-play deals; declare, pass and announcement rates per contract logged and within ranges set in stage 2.
- G6. Agent vs the search benchmark at a fixed iteration budget: result with 95% CI over at least 1,000 deals, reported. No pass/fail.
- G7. Transformer vs MLP control, same pipeline and step budget: difference with 95% CI over at least 1,000 deals, reported.

**Explanations**

- G8. Partner-prediction accuracy before the called king falls at least 50% on 5,000 held-out deals (chance about 33%).
- G9. Belief head Brier score at least 10% better than the uniform prior on held-out deals; calibration error reported.
- G10. Q-head calibration: mean absolute error between predicted and realised deal score reported on held-out deals.
- G11. Attribution sanity: rank correlation between attributions from the trained and a weight-randomised model below 0.3; deletion-curve AUC better than random ordering on at least 500 decisions.
- G12. Counterfactual consistency: the model plays the stated card in at least 95% of shown counterfactuals.
- G13. Template audit: 100% of rendered explanation clauses backed by their signal and threshold on a sample of 500 decisions.

**Phone and app**

- G14. Agent move latency at most 500 ms on a Honor 200; app shell plus weights at most 15 MB; first load at most 5 s on 4G. Measured on the Pages build; DevTools CPU throttling ×4 as the CI proxy.
- G15. Coached play, review and adviser each complete a full deal end to end in the T0 app (manual test script in `docs/`).
- G16. Adviser: a full deal entered by the user in at most 6 minutes, timed.
- G17. At least 3 scripted lessons and at least 8 rule cards in Slovenian, each tied to a seeded deal or rule.
- G18 (T1). At least 30 deals played against the agent by at least 2 people, results and feedback recorded, reported with standard error as anecdote.

**Process**

- G19. 100% of commits on `main` carry the tool and model trailer; `ai-log.md` has a row for every session; `README.md` lists every tool with a pinned model tag.

## Agentic stack

As used, not as advertised. One row per tool and model. The full log is [docs/planning/ai-log.md](docs/planning/ai-log.md).

| Tool | Model (pinned tag) | Dates | What it did |
|------|--------------------|-------|-------------|
| Claude Code 2.1.306 (Claude desktop app, Windows 11) | `claude-fable-5-1` | 2026-11-06 to 2026-11-10 | Planning: read the course material, ran research subagents, prepared decision batches, kept the planning files |
| Claude Code 2.1.306 | `claude-fable-5-2` | from 2026-11-10 | Screen spec, wireframe scripts, pitch draft, build plan, task specs, and all code from spec 001 on |
| Claude Code subagents (general-purpose) | same as the parent session (`claude-fable-5-1`); not confirmed per subagent | 2026-11-06, 2026-11-09 | Web research written to `docs/planning/research/` |
| Claude Code subagent (general-purpose) | `claude-fable-5-2`, as the subagent reported | 2026-11-10 | Review pass over the output of spec 001 |
| Claude Code parallel sessions | `claude-fable-5-2` | from 2026-11-10 | Execution of specs 002 and 003 |
| Figma connector (Figma MCP server) | none; the agent wrote Plugin API scripts | 2026-11-10 | Drew the wireframe frames |
| GitHub Actions | none | from 2026-11-10 | CI: repository checks, later tests and builds |

Every commit since `0d5045d` (2026-11-10) names the tool and model that produced it in a `Produced-by:` trailer. The eight planning commits before it do not; they are covered by the AI log.

## Install

Not yet available. Planned: a Rust toolchain for the engine, Python 3.12 with a pinned PyTorch CUDA build for training, Node for the app. Planned commands (Windows cmd), valid once specs 002 and 003 are committed:

```bat
py -3.12 -m venv .venv
.venv\Scripts\activate
pip install -r src\train\requirements.txt
```

## Run

Not yet available. Planned commands (Windows cmd):

```bat
cargo test --manifest-path src\engine\Cargo.toml
.venv\Scripts\activate
python -m tarok_train.envcheck
```

## Demo

Not yet available. Planned: open the GitHub Pages address of this repository on a phone, install the app, and follow the script in `docs/demo.md` on a seeded deal.

## Repository layout

| Path | Content |
|------|---------|
| `src/engine` | Rust crate `tarok-engine`: rules, scoring, seeding, records, encoding |
| `src/train` | Python training code: learner, configs, evaluation, export |
| `src/app` | Vue 3 app (PWA) |
| `src/assets/cards` | Script-generated SVG cards |
| `models` | Released weight files with a `MODEL.md` each |
| `docs` | Planning, specs, progress, architecture, wireframe |
| `report` | Report source (ERK template) |
| `tools` | Small check scripts |

## Not in scope

- Three-player tarok. Only after the four-player core, if time allows.
- Online multiplayer. After the course.
- Text written by a language model at run time. Never.
- Code from other tarok engines. Never. General libraries are used.
- Rule variants on the left-out list in [docs/planning/rule-switches.md](docs/planning/rule-switches.md).
- Playing styles as fine-tuned copies. Stretch.

## Licences

Code: MIT, see [LICENSE](LICENSE). Cards and texts: CC BY 4.0, see [LICENSE-assets](LICENSE-assets).
