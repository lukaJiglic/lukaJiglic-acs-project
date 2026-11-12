# Plan

Started 2026-11-06 as a draft; all eight decision batches answered by 2026-11-10 (D1–D88). Every item here points at its decision number; anything without one is a note by Claude, not a choice. The proposal, `README.md` and the report derive from this file (D87).

## 0. The starting idea (from the user, open to challenge)

An app for the card game tarok that works well on a phone or in a browser. Three uses:

1. Play against bots.
2. Learn the game by playing, with the app teaching as you go.
3. Use it during a real-life game: enter your cards and the cards played; it explains the thinking behind the best move.

Bots range from simple to self-play-trained on the user's PC. The app can show why a bot chose a move. Each commonly varying rule is a player-settable switch.

### Challenges raised by Claude, and their outcome (2026-11-09)

- **Size.** Closed by D8: the user says it is doable. Not raised again.
- **One engine, three faces.** Accepted in spirit by D1 and D13: the trained agent is the core; the three uses are built around it.
- **Real-life adviser.** Resolved by D3: it is in the core, and its purpose is learning (the game, the thinking, the strategies).
- **Rule switches multiply work.** Still true. Handled by D12: a toggle list with tiers and incompatibilities (`rule-switches.md`). Toggles that change play affect the agent's training; toggles that change scoring only are cheap.
- **Trained bots vs deadline.** Resolved by D1/D10: the trained agent is the core; a search bot is only a benchmark. Risk is managed in the bots batch.

### Direction after Batch 1

- 4-player tarok with a called king. 3-player later if time allows (D4).
- Uses in priority order: Learn → Play → Table (D2).
- Explanations come from ML explanation methods applied to the trained agent (D1). See `research/07`.
- No code from other tarok engines (D9). Own simple SVG cards (D11).

## 0b. Research so far

First pass 2026-11-06 by four research subagents (`research/01–04`), summarised below. Second pass 2026-11-09 (`research/05` training method, `06` network and browser inference, `07` explaining the trained agent) is summarised in sections 2–3 through the decisions it informed. Claude did not re-read the sources itself. Each file lists what was read directly and what is inferred.

- **Rules** (`research/01`). An official ruleset exists for 3 players only: TARZS "Pravila tekmovalnega taroka", 2010, amended 2017, 2018, 2022. No official 4-player ruleset; pagat.com and the Tarok.net rulebook are the references. 55 variation points found. Neighbouring games (Königrufen, tarokk, taroky) are separate games.
- **Existing software** (`research/02`). Play-against-bots exists (TAROK.net, Tarock Palčka, Tarok Counter). No app teaches Slovenian tarok. No app advises at a real table. Only KubiTarock (Königrufen) has "ask the bot why". OpenSpiel has a tarok engine (C++/Python, Apache-2.0, lightly tested, single round). StockŠkis is copyleft (GPL/AGPL). Prior academic work: Silicon Tarokist 2003 (3-player, sampling + alpha-beta), Marinčič 2006 (bidding), Lapajne 2011 (MCTS), Požrl 2021 (tabular Q-learning). No deep-RL tarok result found.
- **Bots** (`research/03`). Shortlist: (1) rule-based bots plus determinized search, no training, explanations from search statistics; (2) a self-play network (Deep Monte Carlo or PPO with a full-information critic); (3) a hybrid as a stretch. Training estimates on this PC range from a few hours to several days and depend mostly on engine speed. All estimates are inferred by scaling published numbers.
- **Artwork** (`research/04`). Recommended: own index-only SVG cards. Optional: public-domain Piatnik scans from Volkskundemuseum Wien as a "classic" skin, though they show a different pattern type from the one Slovenian players know. The modern Piatnik deck is assumed protected.

## 1. Product

Decided in Batch 3 (D31–D42). The screen spec is `screen-spec.md` (draft 2, 2026-11-10, Batch 9 answers D90–D102 included). App name: "Tarok trener" (D102). Play runs clockwise on screen (D91). The coach interrupts in every phase (D93). Review uses five grade classes (D95). Six lessons and eleven rule cards are T0 (D100, D101). The Figma wireframe (28 greyscale frames, one styled sample, D99) is at https://www.figma.com/design/XVrAGNVsQuisOd79HYbRBg; overview image in `docs/wireframe/wireframe-overview.png`. Pitch text draft: `pitch.md`. The adviser contrasts every phase in T0, not only card play (D104).

- **User (D31).** A player who knows the rules and wants to play better. Beginners get a rules layer on top (rule cards when a rule triggers).
- **Form (D32).** Phone-first web app, portrait, installable PWA, usable on desktop. Slovenian first, English as a translation pass, i18n keys from the first screen (D40). Own simple SVG cards (D11).
- **Tier 0 screens (D33).** Home · Learn hub · Table component in three modes (Lesson, Play, Adviser) with the layered explanation panel (D16) · Post-game review · Rules and presets · Levels · Settings.
- **Learn (D34–D36).** Coached free play first: the learner plays; when the intended card loses more than a threshold against the agent's choice, the app shows the contrast. Scripted lessons on seeded deals come second. T0: review, coached play, a small lesson set. T1: bidding adviser, what-if replays, puzzles from key moments, progress tracking. T2: card-counting trainer, spaced-repetition puzzles from own mistakes. Teaching text: Claude drafts from rules and agent signals, the user corrects.
- **Play (D38).** T0: play vs bots, levels (random, rule bot, measured checkpoints; D24). T1: daily deal, share a deal by code. T2: beat-the-bot duplicate mode. T3: online play with friends.
- **Table / adviser (D3, D17, D37).** T0: full entry: hand on a 54-card grid, every bid per seat, king call, talon if declarer, trick by trick with the grid hiding seen cards; the learner taps the intended card before each own play, then sees the agent's card and the contrast. T1: scoring calculator, house-rules share code (D39). T2: session score sheet, rules arbiter.
- **Rules (D12, D20).** Toggles per `rule-switches.md`; one default preset for the first model; presets and share code.
- **Tiers (D42).** T0 demo, T1 planned, T2 stretch, T3 after the course.

### Idea pool (kept for the record; tiers above supersede the "none chosen" status)

Origin: U = user, C = Claude.

**Play**
- U: Play against bots.
- U: Difficulty settings; levels with measured strength; bots with different playing styles.
- U: Playing with friends.
- C: Daily deal. Everyone gets the same seeded deal; compare your result with the bots' and with friends. Multiplayer feel with no server.
- C: Share a deal by code or link (seed + rule set).
- C: "Beat the bot" duplicate mode: you and the strong bot play the same hand; scores compared.
- C: Bot ladder page showing each bot's measured strength. Doubles as a report result.
- C: Short session formats (e.g. 4 deals) for phone use.

**Learn**
- U: Learn by playing, with teaching as you go.
- U: Post-game review marking good moves and mistakes.
- U: "What if" replays.
- U: Puzzles.
- U: Card-counting trainer; bidding trainer.
- U: Progress tracking.
- C: Rule cards that appear the first time a rule triggers (e.g. first forced trump, first pagat ultimo chance).
- C: Assist levels that fade: legal-move highlight → counting overlay → hint on request → nothing.
- C: Open-cards mode: all hands visible, for seeing why a move works.
- C: Belief overlay: where the bot thinks the unseen cards are.
- C: Bid adviser: for a given hand, the simulated score distribution of each contract.
- C: Puzzles generated from the player's own mistakes, repeated on a spaced schedule.
- C: Take-back that shows what the move would have cost.
- C: Glossary of tarok terms, Slovenian and English.

**Real-life table**
- U: Adviser: enter your cards and the cards played, get the best move with reasons.
- U: Score sheet for real-life sessions.
- U: Share a table's house rules by code.
- U: Faster ways of entering cards.
- C: Scoring calculator: enter contract, announcements and card points; get the score under the table's rules. Small, useful on its own.
- C: Rules arbiter: describe a disputed situation, get the ruling under the table's rule set.
- C: Rule-set diff: show how two house-rule sets differ.
- C: Entry by elimination: the grid only shows cards not yet seen; one tap per card.
- C: Camera recognition of cards. High risk; listed for completeness.
- C: After-the-hand replay: enter the hand afterwards and review it, instead of advice during play.
- C: Session stats and export of the score sheet.

**Cross-cutting**
- U: Slovenian and English versions.
- U: Rule switches for every commonly varying rule.
- C: Installable offline (PWA).
- C: Readable cards on small screens: large indices, colour-blind-safe suits.
- C: Named rule presets (e.g. tournament rules, common home rules) on top of the switches.

### Tiers

T0 = must work for the demo, T1 = planned, T2 = stretch, T3 = after the course (D42). The T0 list is final for the proposal (D80):

- **Engine:** 4-player rules on the Default preset, the 12 T0 toggles, scoring, seeded deals, game records, CLI. Goals G1–G2.
- **Bots:** random, rule-based, trained transformer through stages 1–2 (card play, talon, bidding, announcements, kontra, klop, berač). Levels from checkpoints (D24). The MLP control (G7) and the search benchmark (G6) are measured and reported, not demoed.
- **App:** Home, Learn hub, Table in Lesson / Play / Adviser modes, Review, Rules and presets, Levels, Settings. Offline PWA on GitHub Pages.
- **Learn:** coached play, review, at least 3 lessons and 8 rule cards (G17).
- **Adviser:** full entry of a real deal (G16), with the end-session action that settles radelci (D88).
- **Explanations:** default and tap layers (D16), review contrast with one counterfactual, faithfulness goals G8–G13.
- **Process:** CI, specs, progress file, AI log, commit trailers (G19).

T1–T3 per feature are listed in the bullets above (D34–D39). The app always ships the best measured checkpoint, whatever its strength (D81).

## 2. Architecture and technology

Decided in Batch 5 (D51–D64), building on D22:

- One Rust crate `tarok-engine`: rules, toggles, scoring, seeding, game records, state encoding, transformer forward pass. Built as native (training actors via PyO3, CLI for tests and tournaments) and WASM (browser, SIMD).
- App: Vue 3 + Pinia + Vite, TypeScript, custom CSS, PWA. Model runs in the browser inside the WASM module; weights as a versioned f16 file cached by the service worker. No server.
- Training: Python package with the PyTorch learner, Rust actors through PyO3/maturin, native Windows, CUDA wheel with Turing support. Pinned versions.
- Layout: `src/engine`, `src/app`, `src/train`, `src/assets/cards`, `docs`, `report`, `models`, `.github/workflows`.
- Hosting: GitHub Pages by Actions. Storage: IndexedDB and localStorage, JSON export/import.
- One JSON game-record schema for self-play logs, app games, adviser sessions and lessons, with per-decision observations.
- Cards: script-generated index-only SVGs, CC BY 4.0; code MIT.
- Tests: rule scenarios per toggle, property tests, golden games, native↔WASM↔PyTorch parity. CI on GitHub Actions.

## 3. Bots

Decided: the trained agent is the core (D1). A search bot is a benchmark only (D10). Explanations come from ML explanation methods on the trained agent (D1).

Design decided in Batch 2 (D14–D26; sources `research/05`, `06`, `07`; `rule-switches.md` for which toggles touch play):

- **Method (D14).** PPO with a full-information critic, self-play, 20% past checkpoints. Deep Monte Carlo run on the same pipeline as control and fallback.
- **Network (D18, D15, D19).** Transformer over the sequence of play and bid tokens, at least 4 layers, 256 wide. One state token carries the flat card-aligned state (~620 features from research/06) and the exact inference bits. One network for all phases, phase one-hot, ~140-way masked action vector. Heads: policy, observation-only Q per action, decomposed value, partner-seat and unseen-card-location auxiliaries. Full-information critic separate, training only. Control run: flat MLP (512-512-256) on the same pipeline. The transformer is the primary model and is not dropped at the first setback.
- **Token sequence (D21).** Bid tokens appear from stage 1, produced by a rule-based bidder until bidding is learned. Then king call, talon, announcement and card-play tokens as the curriculum adds them.
- **Sequence layout (D116, 2026-11-10).** L2: tokens computed once per seat and deal, a state token per decision that reads them through a mask. Chosen after the benchmark of spec 003.
- **First run (D107, 2026-11-10).** The user chose to train every phase from the start, in all Default-preset contracts. The phase-by-phase curriculum below is kept as the fallback if the monitors show the documented failures.
- **Curriculum (D21).** Card play → talon → bidding → announcements and kontra → klop and berač, each with monitors (declare rate, pass rate, announcement rate, partner-prediction accuracy, legal-move violations).
- **Rule switches (D20).** First model on one default preset; play-changing toggles fixed. Rule vector and preset mixture later. Scoring-only toggles may be inputs from the start.
- **Pipeline (D22).** Rust engine and forward pass, self-play actors on CPU threads, PyO3 to a PyTorch learner on the GPU. Gate: ≥ 20,000 steps/s end to end, measured in stage 0. The same Rust crate compiles to WASM for the browser. Weights shipped as a small f16 file (size to be measured for the transformer; the MLP estimate was 1.5 MB). No ONNX runtime in the browser; ONNX export kept only as a cross-check of the Rust forward pass.
- **Explanations (D16, D17).** Layered: default = chosen card, expected score, margin, belief strip; tap = Q bars, reward components, top-3 perturbation attributions; review = contrast with the learner's move, one counterfactual, concept readouts, replay. No LLM narration. Faithfulness checks from research/07 (D1–D8, D10) become numbered goals. Adviser asks for the learner's card first.
- **Other bots (D23).** Random, rule-based, PIMC/ISMCTS benchmark (stage 3), cheating full-information agent as upper bound, checkpoints.
- **Levels and styles (D24).** Levels = random, rule bot, measured checkpoints. Styles = fine-tuned copies, later tier.
- **Evaluation (D25).** Seeded deal files, 8-seating duplicate scheme, 2,000 routine / 10,000 release deals, paired t-test, behaviour monitors, per-deal spread measured early.
- **Stages (D26).** 0 engine, tests, harness, speed gate → 1 card play (with rule-based bid tokens) → 2 talon, bidding, announcements, kontra, klop, berač → 3 benchmark, tournament, rule vector → 4 long runs, WASM export, phone test, human sessions → 5 evaluation tables, ablations. No calendar; progress recorded thoroughly.
- **Training time (inferred, to be measured).** MLP figures: 10^8 steps in 17 min–0.9 h, 10^9 in 3–9 h with the Rust pipeline. The transformer costs more per step; expect several times the MLP figures. No published number for a 4-layer, 256-wide transformer on this GPU at this batch size; measure in stage 0.
- **Open for the transformer (Claude's notes, not decided).** Token vocabulary (card × seat, bid × seat, king call, announcement, kontra, phase markers); maximum sequence length (~48 plays + ~15 bids/announcements + state token); whether a causal mask is needed (no, the sequence is fully observed up to now); positional encoding (learned, by index); whether past checkpoints' sequences stay compatible when the vocabulary grows across stages (keep the vocabulary fixed from stage 1).

Hardware (confirmed as the training machine, F4): AMD Ryzen 5 5600 (6 cores / 12 threads), 32 GB RAM, NVIDIA GTX 1660 SUPER 6 GB (Turing, sm_75, supported by current PyTorch CUDA wheels), 1.2 TB free disk, Windows 11.

## 4. Measurement

Decided in Batch 6 (D65–D69). Thresholds are realistic targets; aspirations in brackets are not goals. Numbers may change until the proposal is submitted (D69). These go into `README.md` and the report.

**Rules and engine**
- G1. Rule-scenario suite of at least 150 tests covering the baseline and every T0 toggle value, plus property tests (card conservation, 70 points per deal, legal-move sets) on 10,000 random deals. `cargo test` in CI. [aspiration 200 / 100,000]
- G2. Native, WASM and PyTorch agree on legal-move sets and scores for 1,000 random games; forward-pass outputs within 1e-4 fp32 / 1e-2 fp16 on 1,000 random states. Parity tests in CI. [aspiration 10,000]

**Pipeline**
- G3. End-to-end self-play throughput of at least 10,000 steps/s on the training PC. Stage-0 benchmark, logged in `docs/progress.md`. [aspiration 20,000; D22's gate becomes 10,000] [D128, 2026-11-12: lowered to a figure the transformer can reach; the number is set from the measurement of spec 008 and confirmed by the user (C13)]

**Agent strength** (8-seating duplicate protocol, D25)
- G4. Transformer agent beats the rule bot: positive mean score per deal, 95% CI excluding zero, at least 1,000 deals. [aspiration +5 points per deal over 2,000 deals]
- G5. Zero illegal moves in 10,000 self-play deals; declare, pass and announcement rates per contract logged and within ranges set in stage 2.
- G6. Agent vs the search benchmark at a fixed iteration budget: result with 95% CI over at least 1,000 deals, reported. No pass/fail.
- G7. Transformer vs MLP control, same pipeline and step budget: difference with 95% CI over at least 1,000 deals, reported.

**Explanations**
- G8. Partner-prediction accuracy before the called king falls at least 50% on 5,000 held-out deals (chance about 33%). [aspiration 60%]
- G9. Belief head Brier score at least 10% better than the uniform prior on held-out deals; calibration error reported. [aspiration 20%]
- G10. Q-head calibration: mean absolute error between predicted and realised deal score reported on held-out deals.
- G11. Attribution sanity: rank correlation between attributions from the trained and a weight-randomised model below 0.3; deletion-curve AUC better than random ordering on at least 500 decisions. [aspiration 0.2 / 1,000]
- G12. Counterfactual consistency: the model plays the stated card in at least 95% of shown counterfactuals. [aspiration 99%]
- G13. Template audit: 100% of rendered explanation clauses backed by their signal and threshold on a sample of 500 decisions.

**Phone and app**
- G14. Agent move latency at most 500 ms on a Honor 200; app shell plus weights at most 15 MB; first load at most 5 s on 4G. Measured on the Pages build; DevTools CPU throttling ×4 as the CI proxy. [aspiration 300 ms / 10 MB / 3 s]
- G15. Coached play, review and adviser each complete a full deal end to end in the T0 app (manual test script in `docs/`).
- G16. Adviser: a full deal entered by the user in at most 6 minutes, timed. [aspiration 4]
- G17. At least 3 scripted lessons and at least 8 rule cards in Slovenian, each tied to a seeded deal or rule. [aspiration 6 / 11, the planned set of D100 and D101]
- G18 (T1). At least 30 deals played against the agent by at least 2 people, results and feedback recorded, reported with standard error as anecdote. [aspiration 50 / 3]

**Process**
- G19. 100% of commits on `main` carry the tool and model trailer; `ai-log.md` has a row for every session; `README.md` lists every tool with a pinned model tag.

## 5. Working with agents day to day

Decided in Batch 7 (D70–D79). This section is the answer to course requirement R14 and feeds the stack table (R13) and the AI-use declaration (R17).

- **Unit of work (D70).** A spec file `docs/specs/NNN-short-name.md`: goal, acceptance criteria, files expected to change, out of scope, tool and model. Status at the top (draft / approved / done / dropped). Numbered, never deleted. Flow: Claude drafts the spec, the user approves it (criteria first, D73), an agent executes, verification runs, the commit references the spec.
- **State files (D71).** `docs/planning/` (decisions, open questions, plan, research, AI log) stays. `docs/progress.md`: one entry per session with date, what was done, measurements, what is broken. `docs/specs/`: the tasks. `docs/architecture.md`: the architecture as built, kept current. `CLAUDE.md` at the repo root: standing instructions for every agent session (routine, checklist, conventions).
- **Verification before each commit (D72).** (1) Acceptance criteria met and named in the commit. (2) Tests pass locally (`cargo test`, app tests, `pytest`, as applicable). (3) The user reads the diff. (4) A separate agent review pass over the diff. (5) CI green after push. Weights also pass the golden forward-pass test. Rule changes also pass the scenario tests for that toggle.
- **Commits (D74).** `type(scope): subject`; types feat, fix, test, docs, plan, chore, train, model. Body says what and why. Trailers: `Spec: docs/specs/NNN-name.md`, `Produced-by: Claude Code <version>, model <tag>` or `Produced-by: user`, `Session: <ai-log row>`, `Co-Authored-By`. One commit per task or logical step. The commit log is the replication record.
- **Session logging (D75).** One `ai-log.md` row per session: date, tool and version, model tag, what the agent did, what the user did, outputs. Prompts summarised; a prompt is stored verbatim in `docs/prompts/` only when reused. Subagents get a sub-row with their model.
- **Stack (D76, D89).** Claude Code for planning, code, tests, research subagents and review passes. Model: Claude Fable 5.1 (`claude-fable-5-1`) from 2026-11-06 to 2026-11-10; Claude Fable 5.2 (`claude-fable-5-2`) from 2026-11-10 on. Figma connector for the wireframe. GitHub Actions for CI. A local Ollama model only later, with one declared role, if time allows. Each model has its own stack-table row with pinned tag and date range of use.
- **Branching (D77).** Trunk-based on `main`. A branch only for a long or risky change, merged with a merge commit.
- **Session routine (D78).** Start: read `docs/planning/README.md`, `docs/progress.md`, the open specs; work in spec order. End: update `docs/progress.md`, spec status, `ai-log.md`; commit; push.
- **Training runs (D79).** Config per run in `src/train/configs/`. Run log in `docs/progress.md` (steps, hours, metrics, checkpoint hash). Released weights in `models/` with a `MODEL.md` (config, data, date, metrics), one `model:` commit each. Intermediate checkpoints stay out of git.
- **Build plan (D106).** Training comes first: `docs/build-plan.md` lists specs 001–009 up to the first stage-1 run and what is deferred until then. It refines the order below.
- **Order of work (D82).** A sequence, not a calendar: (0) repo skeleton: README with goals, `CLAUDE.md`, `docs/progress.md`, CI, licences; (1) engine stage 0: rules, Default preset, T0 toggles, tests, CLI, records, random and rule bots, seeded deals, speed harness and gate; (2) training pipeline: PyO3 actors, learner, stage-1 card-play run started; (3) app shell while training runs: Table component, Play vs rule bot, cards, PWA, Pages; (4) WASM export of engine and forward pass, first checkpoint in the browser, levels; (5) stage-2 training with monitors; (6) explanations: signals, templates, aux-head readouts, attributions, counterfactuals; (7) Learn: coached play, review, lessons, rule cards; (8) adviser entry; (9) evaluation: benchmarks, MLP control, faithfulness checks, phone measurements; (10) report and demo script. Training runs are long and unattended, so app work fills the GPU time.
- **Report cadence (D85).** Skeleton in `report/` from the start of building. Stack table and method updated at the end of each session that changes them. Results filled as measurements land in `docs/progress.md`. Final pass at the end.
- **Goal G19** measures this section: every commit on `main` carries the trailer, every session has an AI-log row, `README.md` lists every tool with a pinned tag.

## 6. Course compliance

Mapping from each requirement in `course-requirements.md` to the plan item that satisfies it. "Action" marks items that still need a step by the user.

| Req | What it asks | Satisfied by | Status |
|-----|--------------|--------------|--------|
| R1 | One topic | Trained tarok agent with a platform around it (D1, D13) | done |
| R2 | Topic from the Moodle list or approved at consultation before 18 Nov | "Browser game" fits the list; confirm at the 16 Nov scoping session | **Action: user confirms the topic with the lecturer on 16 Nov** |
| R3 | Small enough, agent-buildable, demoable in 10 minutes | Tiers (D80), demo script (D83), fallback (D81) | planned |
| R4 | Process and results graded, not polish | Section 5; G19; honest results (D81) | planned |
| R5 | Repo `lukaJiglic-acs-project`, accessible to the instructor | Exists, private (D6) | **Action: make public, or add the instructor, by 23 Nov** |
| R6 | Repo live from 23 Nov | D6; order-of-work step 0 (D82) | planned |
| R7 | README: what, numbered goals, stack with pinned tags, install/run/demo | Goals G1–G19 (section 4), stack (D76), `docs/demo.md` (D83), not-in-scope (D86) | planned |
| R8 | `src/` for code, `docs/` for screenshots and architecture | Layout D56; `docs/architecture.md` (D71) | planned |
| R9 | Commit history graded; frequent, real messages | D74, D77; one commit per task (D70) | in use since 0d5045d |
| R10 | Abstract ≤ 200 words | Report skeleton (D85); one result from section 4 | later |
| R11 | Introduction: problem and user | D31 (user), section 0 | later |
| R12 | Goals numbered and testable | Section 4, G1–G19 (D65) | done; frozen at the proposal (D69) |
| R13 | Stack table: tool, pinned model, what it did | D76; `ai-log.md` rows (D75) | in use |
| R14 | Method: task split, state files, verification, commit log as replication record | Section 5 (D70–D79) | decided |
| R15 | Results: what works, how known, failures too | Goals table with measured values in `docs/progress.md` (D71, D84); D81 | planned |
| R16 | Limitations and next steps | Not in scope (section 8, D86); T1–T3 lists | planned |
| R17 | AI-use declaration: agents vs student, every model with tag and dates | `ai-log.md` (D75), trailers (D74), stack rows with date ranges (D76) | in use |
| R18 | Bibliography entry for the repo with the demo branch or tag | A git tag on the demo commit, named when tagging (step 10, D82) | later |
| R19 | Everything declared, including local models | D76 (Ollama only with a declared role), `ai-log.md` | in use |
| R20 | Optional ERK 2027 submission | No action | — |

## 7. Fixed points, demo and checkpoint

No calendar is planned against (D26). The course fixes these points:

| Date | Course event | What the plan brings |
|------|--------------|----------------------|
| Mon 16 Nov | Scoping and Figma session | Topic confirmation (R2); wireframe questions |
| Wed 18 Nov | Proposal, Figma wireframe, one-slide pitch (10%) | Derived from this file (D87); screen spec → wireframe → pitch text (D50) |
| Mon 23 Nov | Pitch; repo live; sprint 1 starts | Repo public (R5); order-of-work step 0 (D82) |
| Sun 13 Dec / Mon 14 Dec | Sprint-1 milestone; lightning talk and written deliverable (10%) | `docs/sprint-1.md` (D84): goals table with measured values, what was built, stage-0 measurements, what broke, next sprint, AI-log excerpt. Aim: steps 0–3, shown as it stands |
| Mon 21 Dec | Sprint review | `docs/progress.md` |
| Mon 11 Jan | Report (25%), code and 10-minute demo (25%) | Demo per D83; report per D85; tag for R18 |

**Demo (D83).** Scripted on a seeded deal, phone in hand, laptop as fallback. Intro 1 min; a deal against the agent with the explanation layers 3 min; coached play contrast and one counterfactual 2 min; adviser on a real deal with physical cards 2 min; numbers (goals table, tests, CI, commit log, AI log) 2 min. Script in `docs/demo.md`. The seed is fixed so the demo repeats.

## 8. Not in scope (D86)

Stated in `README.md` and the report so the goals do not over-claim:

- 3-player tarok: after T0, only if time allows (D4).
- Online multiplayer: T3, after the course (D38).
- LLM narration of explanations: never (D16).
- Code from other tarok engines: never (D9). General libraries are allowed (D27).
- Rule variants in the left-out list of `rule-switches.md` (D12).
- Play styles as fine-tuned copies: T2 (D24).
