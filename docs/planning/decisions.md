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
| D30 | 2026-11-09 | **State token = flat features + exact inference bits.** Exact inference bits mean both the facts known for certain (void suits per seat, revealed partner, cards accounted for, talon seen, called king fallen) and derived counts (trumps still out per seat, points so far). **Learned positional encoding includes trick number and position within the trick.** | — | User (C5, plus spec) | Refines D18. |
| D31 | 2026-11-09 | **Primary user: a player who knows the rules and wants to play better.** Beginners get a rules layer on top. | B3-1 a–c | User (a) | |
| D32 | 2026-11-09 | **Phone-first web app**, portrait, installable as a PWA, usable on desktop. | B3-2 a–c | User (a) | |
| D33 | 2026-11-09 | **Tier 0 screens:** Home · Learn hub · Table component in three modes (Lesson, Play, Adviser) with the explanation panel · Post-game review · Rules and presets · Levels · Settings. | B3-3 a–c | User (b) | |
| D34 | 2026-11-09 | **Teaching as you go: coached free play first** (interrupt when the learner's intended card loses more than a threshold against the agent), **then scripted lessons** on seeded deals. | B3-4 a–c | User (b first, then a) | |
| D35 | 2026-11-09 | **Lesson and teaching text: Claude drafts** from the rules and the agent's signals; **the user corrects.** | B3-5 a–b | User (a) | |
| D36 | 2026-11-09 | **Learn tiers:** review, coached play, small lesson set = T0; bidding adviser, what-if replays, puzzles from key moments, progress tracking = T1; card-counting trainer, spaced-repetition puzzles from own mistakes = T2. | B3-6 | User (accept) | |
| D37 | 2026-11-09 | **Adviser input in T0 is full entry:** hand on a 54-card grid, every bid per seat, king call, talon if declarer, then trick by trick with the grid hiding seen cards; intended card before each own play. | B3-7 a–c | User (a) | Claude had recommended b for T0. |
| D38 | 2026-11-09 | **Play tiers:** play vs bots, levels = T0; daily deal, share deal by code = T1; beat-the-bot duplicate = T2; **online play with friends = T3** (after the course). | B3-8 a–c | User (b) | |
| D39 | 2026-11-09 | **Table tools:** scoring calculator, house-rules share code = T1; session score sheet, rules arbiter = T2. | B3-9 a–b | User (a) | |
| D40 | 2026-11-09 | **Slovenian first.** i18n keys from the first screen; English as a translation pass. Tarok terms stay Slovenian in both. | B3-10 a–c | User (b) | Claude had recommended a (English first). The report and demo are in English; the UI is Slovenian first. |
| D41 | 2026-11-09 | **Figma wireframe through the Figma connector** available in the user's Claude setup. Claude writes the screen spec, then builds the wireframe in Figma. | B3-11 a–c | User (c) | Figma MCP tools appeared in the session on 2026-11-09. Timing of the wireframe work is open. |
| D42 | 2026-11-09 | **Tier scheme:** T0 = must work for the demo; T1 = planned; T2 = stretch; T3 = after the course. | B3-12 | User (a) | |
| D43 | 2026-11-09 | **Default preset = the user's table's rules**, saved as preset "Default". The first model trains on it. pagat.com and Majority stay as named presets. | B4-1 a–c | User (a) | |
| D44 | 2026-11-09 | **Default values:** S14 taroks may be discarded, shown (a). S25 rounding to nearest 5 (a). S29 mondfang −20 (a). S28 klop full at 35 or more (b). S26 partner writes the same (a). S33 radelc cancelled only on a win (a). S34 unused radelci **−70 each** (user's table; not in any source). S35 session **open-ended**; unused radelci settle when the session is closed in the app. S6 odprti berač **pinned off** until T1 (b). S7 barvni valat pinned off until T1 (a). S1 no compulsory klop at score 0 (b). S18 four kontra levels, ×16 (a). | B4-2 | User | Claude's interpretation of "open-ended + −70": the app has an explicit "end session" action that settles leftovers. To confirm if wrong. |
| D45 | 2026-11-09 | **Toggle list and tiers accepted for now** (39 toggles; T0 12). May change later. | B4-3 | User (accept for now) | |
| D46 | 2026-11-09 | **Left-out list accepted.** | B4-4 | User (a) | |
| D47 | 2026-11-09 | **Scoring-only T0 toggles are network inputs**, randomised in self-play, so one model covers their values. | B4-5 a–b | User (a) | |
| D48 | 2026-11-09 | **Incompatibilities enforced in the UI:** dependent toggles greyed out and forced; presets set every toggle; the share code encodes all toggle values; a warning lists what a preset leaves unspecified. | B4-6 a–b | User (a) | |
| D49 | 2026-11-09 | **Klop always on in T0 and T1.** S4 (klop off) moves to T2. | B4-7 a–b | User (b) | |
| D50 | 2026-11-09 | **Next session: screen spec, then the Figma wireframe through the connector, then the pitch slide text.** | C6 a–b | User (a) | |
| D51 | 2026-11-09 | **One Rust crate `tarok-engine`**: rules, legal moves, scoring, toggles, seeding, game records, state encoding, transformer forward pass. Built native (PyO3 actors, CLI) and WASM (wasm-bindgen, SIMD). | B5-1 a–b | User (a) | |
| D52 | 2026-11-09 | **App: Vue 3 + Pinia + Vite, TypeScript, custom CSS variables, VueUse, PWA via vite-plugin-pwa.** | B5-2 a–c | User (a) | |
| D53 | 2026-11-09 | **Model runs in the browser only**, inside the WASM module. Weights as a versioned f16 file, cached by the service worker. No server. | B5-3 a–c | User (a) | |
| D54 | 2026-11-09 | **Transformer forward pass hand-written in Rust** in the engine crate; golden test against PyTorch on 10,000 random states at every weight export; ONNX export in CI as a second cross-check. tract-onnx in WASM is the fallback. | B5-4 a–c | User (a) | |
| D55 | 2026-11-09 | **Training code: Python package** (PyTorch learner, configs, checkpoints, evaluation runner, export). Rust actors via PyO3/maturin. Native Windows, CUDA wheel with Turing support, no WSL2. Versions pinned. | B5-5 a–b | User (a) | |
| D56 | 2026-11-09 | **Repo layout:** `src/engine`, `src/app`, `src/train`, `src/assets/cards`, `docs`, `report`, `models`, `.github/workflows`. | B5-6 a–b | User (a) | |
| D57 | 2026-11-09 | **Hosting: GitHub Pages**, deployed by GitHub Actions on push to `main`. | B5-7 a–c | User (a) | |
| D58 | 2026-11-09 | **Storage: IndexedDB (games, sessions) and localStorage (settings)** via VueUse; JSON export/import. No backend. | B5-8 a–b | User (a) | |
| D59 | 2026-11-09 | **One JSON game-record schema** for self-play logs, app games, adviser sessions and lessons: seed, preset and toggles, model version, seats, full deal, and per decision the acting seat, its observation, legal actions, chosen action, agent outputs. Defined and validated in the engine crate. | B5-9 a–b | User (a) | |
| D60 | 2026-11-09 | **Cards: index-only SVGs generated by a script from one template**, colour-blind-safe suits, styled in the user's craft palette. Licensed CC BY 4.0, separate from the code licence. | B5-10 a–b | User (a) | |
| D61 | 2026-11-09 | **Licences: MIT for code, CC BY 4.0 for cards and texts.** | B5-11 a–c | User (a) | |
| D62 | 2026-11-09 | **Engine tests:** rule scenarios per toggle, property tests (card conservation, 70 points, legal-move sets), golden games from records, native↔WASM↔PyTorch parity, Python binding tests. | B5-12 a–b | User (a) | |
| D63 | 2026-11-09 | **CI on GitHub Actions:** cargo test and clippy, WASM build, app lint and unit tests, Python tests, golden forward-pass test when weights change, Pages deploy. | B5-13 a–b | User (a) | |
| D64 | 2026-11-09 | **Offline PWA:** service worker caches app and weights; weights refetched silently if evicted; UI never blocks on cache presence. | B5-14 a–b | User (a) | |

## Provisional choices by Claude (need confirmation)

| ID | Choice | Why | Status |
|----|--------|-----|--------|
| P1 | Planning files in `planning/` at root. | — | superseded by D6 (`docs/planning/`) |
| P2 | Research notes: one file per topic in `docs/planning/research/`, each ending with "Verified vs inferred". | Matches G4. | open |
| P3 | Commit messages carry a trailer naming the tool and model that produced the change. | Course R14: commit log must show which prompt and model produced which part. | open, to be settled in the workflow batch |
| P4 | The `project/` folder (course PDF and template) is not committed until the user says so. | — | superseded by D28 |
| P5 | Report source lives in `report/` (copied from `project/`, which stays untracked). | D28 says only the source is committed; `report/` keeps it apart from `src/` and `docs/`. | open |
