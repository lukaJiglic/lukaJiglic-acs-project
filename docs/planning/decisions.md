# Decision log

Status values: **given** (stated by the user in the brief), **decided** (user chose after options were shown), **provisional** (Claude chose to keep work moving; needs user confirmation), **reversed**.

## Givens from the user's brief (2026-11-06)

| ID | Statement | By | Status |
|----|-----------|----|--------|
| G1 | Session 1 is planning only. Nothing gets built until the user says so. | User | given |
| G2 | Claude asks instead of assuming, for every decision unless told otherwise. | User | given |
| G3 | Decisions come in batches the user can answer in one go. Each has options, a recommendation and a reason. The user decides. | User | given |
| G4 | Facts get researched properly. Claude says what was checked and what is inferred. | User | given |
| G5 | Training methods and architectures must be researched thoroughly before building starts. | User | given |
| G6 | Planning state lives in files. A fresh session must be able to continue from the files alone. | User | given |
| G7 | Planning sessions are logged as AI use: date, tool, exact model, what was done. | User | given |
| G8 | Card artwork must be the user's own or under a licence that allows a public repo. | User | given |
| G9 | The plan must satisfy every course requirement for the repo and the report. | User | given |
| G10 | Scope is not locked early. Every step may be questioned. Claude adds ideas. | User | given |
| G11 | The finished plan covers: product; architecture and technology; bots and training time; measurement; day-to-day agent workflow. | User | given |

## Facts from the user (2026-11-09, Batch 1)

| ID | Fact |
|----|------|
| F1 | Topic approval is the user's concern, not part of the plan. |
| F2 | The user plays 4-player tarok with a called king. |
| F3 | Time is sufficient. No blackout dates given. |
| F4 | This PC is the training machine: AMD Ryzen 5 5600 (6c/12t), 32 GB RAM, GTX 1660 SUPER 6 GB, Windows 11. |
| F5 | The user can review TypeScript/Vue, Python, Rust and C++. |
| F6 | Primary tool: Claude Code with Claude Fable 5.1. Local models may be tried if time allows. |
| F7 | Tarok Counter (gajbrus) is not related. Everything is implemented by this project. |

## Decisions

| ID | Date | Decision | Options shown | Decided by | Notes |
|----|------|----------|---------------|-----------|-------|
| D1 | 2026-11-09 | **The trained agent is the core.** The three uses are built around it. Explanations come from ML explanation methods applied to the trained agent, not from a separate bot. | a) explaining search bot; b) trained bots; c) teaching; d) table tools | User (b, with correction) | Claude had recommended a. The user corrected the framing: explanation = XAI on the trained model. |
| D2 | 2026-11-09 | **Order of uses: Learn → Play → Table.** | a) Play→Learn→Table; b) Learn→Play→Table; c) Table first | User (b) | Claude had recommended a. |
| D3 | 2026-11-09 | **Live real-life adviser is in the core.** Its purpose is learning: the game, the thinking behind moves, and strategies. | a) live adviser in core; b) Tier 1; c) score sheet first; d) after course | User (a) | Claude had recommended b. |
| D4 | 2026-11-09 | **4-player with called king first.** 3-player if time allows. | a) 4-player; b) 3-player; c) both | User (a) | Matches F2. |
| D5 | 2026-11-09 | **Slovenian tarok house rules only.** Neighbouring games are out. | a) Slovenian only; b) also neighbours | User (a) | |
| D6 | 2026-11-09 | **Git now. Planning files committed. Private GitHub repo `lukaJiglic-acs-project` now, public by 23 Nov.** Planning files live in `docs/planning/`. | a) git + private repo now; b) local git; c) later | User (a) | P1 superseded. |
| D7 | 2026-11-09 | **Planning order: bots first**, then the rest. | a) product first; b) bots first; c) workflow first | User (b) | Claude had recommended a. A deep research pass on training and explanation methods precedes the bots batch. |
| D8 | 2026-11-09 | **Project size is not a concern.** Claude stops raising it. | — | User | Stated in response to Claude's size challenge. |
| D9 | 2026-11-09 | **No code reuse from other tarok engines** (OpenSpiel tarok, StockŠkis, etc.). Everything tarok-specific is written in this project. | — | User | General libraries (ML frameworks, UI frameworks) are assumed allowed; confirmation asked. |
| D10 | 2026-11-09 | **A search bot is an addition and a benchmark**, not a core part. | — | User | |
| D11 | 2026-11-09 | **Card artwork: own simple SVG cards.** | own SVG; PD scans; adapted open sets | User | Matches research/04 recommendation. |
| D12 | 2026-11-09 | **Rule variations become toggles**, described concisely. Some toggles may be incompatible with each other; the list must say which. Claude produces a list of proposed toggles and a list of left-out variations with reasons. | — | User | Output: `rule-switches.md`. |
| D13 | 2026-11-09 | **The aim is not novelty.** The end result is a trained agent that plays tarok, with a platform around it that showcases its uses. | — | User | |
| D14 | 2026-11-09 | **Training method: PPO actor-critic with a full-information critic**, self-play with 20% past checkpoints. A Deep Monte Carlo run on the same pipeline as control and fallback. | B2-1 a–d | User (a) | Source: research/05. |
| D15 | 2026-11-09 | **Network heads:** policy (masked logits), observation-only Q per action (regressed on returns, usable in the browser), decomposed value (trick points / king / pagat / announcements / penalties), auxiliary partner-seat and unseen-card-location heads at low loss weight. The full-information critic is a separate training-only network. | B2-2 a–c | User (a) | |
| D16 | 2026-11-09 | **Explanation methods, in order:** (1) intrinsic signals through threshold-bound templates; (2) belief and concept readouts via auxiliary heads and post-hoc probes; (3) perturbation attribution by card swaps and single-edit counterfactuals; (4) review replays and key-moment summaries. **No LLM narration, ever.** | B2-3 a–c | User (a, LLM removed) | Faithfulness checks D1–D8, D10 of research/07 apply; D9 (LLM claim check) is dropped. |
| D17 | 2026-11-09 | **Adviser asks for the learner's intended card first**, then shows the agent's card and the contrast. | yes / no | User (yes) | |
| D18 | 2026-11-09 | **Network: a transformer over play and bid tokens**, at least 4 layers, 256 wide. The flat state vector and the exact inference bits form one state token. Purpose: understanding of decisions across several moves and more complex inference. **A flat MLP is the control run** on the same pipeline. The transformer is not abandoned quickly if it does not work at once. | B2-4 a–c | User (c, with specification) | Claude had recommended a. The flat encoding from research/06 becomes the state token. "Exact inference bits" to be confirmed (C5). |
| D19 | 2026-11-09 | **One network for all phases**, phase one-hot, one masked action vector (~140 logits). | B2-5 a–b | User (a) | |
| D20 | 2026-11-09 | **First model on one default preset** with play-changing toggles fixed. Rule vector and preset mixture added in a later stage. Scoring-only toggles may be inputs from the start. | B2-6 a–c | User (c) | Default preset chosen in Batch 4. |
| D21 | 2026-11-09 | **Curriculum phase by phase** (card play → talon → bidding → announcements and kontra → klop and berač), with monitors. **Bid tokens are present in the sequence from stage 1**, produced by a rule-based bidder until bidding is learned. | B2-7 a–b | User (a, plus bid tokens) | |
| D22 | 2026-11-09 | **Pipeline:** Rust engine with the forward pass in Rust, self-play actors on CPU threads, PyO3 to a PyTorch learner on the GPU. Gate ≥ 20,000 steps/s end to end, measured in stage 0. | B2-8 a–c | User (a) | Where inference runs during self-play is settled by measurement. |
| D23 | 2026-11-09 | **Other bots:** random and rule-based in stage 0; PIMC/ISMCTS benchmark in stage 3 (also a possible behaviour-cloning teacher); cheating full-information agent as upper bound; checkpoints as yardsticks. | B2-9 a–b | User (a) | |
| D24 | 2026-11-09 | **Levels** = random, rule bot, measured checkpoints. **Styles** = fine-tuned copies with small reward changes, later tier. | B2-10 a–c | User (a) | |
| D25 | 2026-11-09 | **Evaluation:** seeded deal files, 8-seating duplicate scheme, 2,000 deals routine / 10,000 release, paired t-test, behaviour monitors from day one, human sessions reported as anecdote with standard error. | B2-11 a–b | User (a) | Per-deal σ to be measured early. |
| D26 | 2026-11-09 | **Stages accepted as a sequence, not a calendar.** Dates are not planned against. **Progress is recorded thoroughly** instead. | B2-12 a–c | User | A progress file is part of the workflow batch. |
| D27 | 2026-11-09 | **General libraries are allowed** (PyTorch, UI framework, RL helpers, etc.). Only code from other tarok engines is banned (D9). | — | User (C2) | |
| D28 | 2026-11-09 | **Only the report source is committed** (`project_report.tex`, `erk.sty`), copied to `report/`. The course PDFs stay out of the repo. | — | User (C3) | `project/` is gitignored. |
| D29 | 2026-11-09 | **Learn screens are built first** (tutorial and review), free play behind them. | — | User (C4) | |

## Provisional choices by Claude (need confirmation)

| ID | Choice | Why | Status |
|----|--------|-----|--------|
| P1 | Planning files in `planning/` at root. | — | superseded by D6 (`docs/planning/`) |
| P2 | Research notes: one file per topic in `docs/planning/research/`, each ending with "Verified vs inferred". | Matches G4. | open |
| P3 | Commit messages carry a trailer naming the tool and model that produced the change. | Course R14: commit log must show which prompt and model produced which part. | open, to be settled in the workflow batch |
| P4 | The `project/` folder (course PDF and template) is not committed until the user says so. | — | superseded by D28 |
| P5 | Report source lives in `report/` (copied from `project/`, which stays untracked). | D28 says only the source is committed; `report/` keeps it apart from `src/` and `docs/`. | open |
