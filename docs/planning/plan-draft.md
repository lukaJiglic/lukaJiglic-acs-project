# Plan draft

Draft 0, 2026-11-06. Nothing here is chosen unless `decisions.md` says so. Sections fill in as batches get answered.

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

## 0b. Research so far (first pass, 2026-11-06)

Done by four research subagents. Claude did not re-read the sources itself. Each file lists what was read directly and what is inferred.

- **Rules** (`research/01`). An official ruleset exists for 3 players only: TARZS "Pravila tekmovalnega taroka", 2010, amended 2017, 2018, 2022. No official 4-player ruleset; pagat.com and the Tarok.net rulebook are the references. 55 variation points found. Neighbouring games (Königrufen, tarokk, taroky) are separate games.
- **Existing software** (`research/02`). Play-against-bots exists (TAROK.net, Tarock Palčka, Tarok Counter). No app teaches Slovenian tarok. No app advises at a real table. Only KubiTarock (Königrufen) has "ask the bot why". OpenSpiel has a tarok engine (C++/Python, Apache-2.0, lightly tested, single round). StockŠkis is copyleft (GPL/AGPL). Prior academic work: Silicon Tarokist 2003 (3-player, sampling + alpha-beta), Marinčič 2006 (bidding), Lapajne 2011 (MCTS), Požrl 2021 (tabular Q-learning). No deep-RL tarok result found.
- **Bots** (`research/03`). Shortlist: (1) rule-based bots plus determinized search, no training, explanations from search statistics; (2) a self-play network (Deep Monte Carlo or PPO with a full-information critic); (3) a hybrid as a stretch. Training estimates on this PC range from a few hours to several days and depend mostly on engine speed. All estimates are inferred by scaling published numbers.
- **Artwork** (`research/04`). Recommended: own index-only SVG cards. Optional: public-domain Piatnik scans from Volkskundemuseum Wien as a "classic" skin, though they show a different pattern type from the one Slovenian players know. The modern Piatnik deck is assumed protected.

## 1. Product

Decided in Batch 3 (D31–D42). Screen specs and the Figma wireframe are the next product step.

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

Not defined. Proposed scheme (to be confirmed): Tier 0 = core that must work for the demo; Tier 1 = planned; Tier 2 = stretch; Tier 3 = after the course.

## 2. Architecture and technology

Not decided. Questions to settle: language of the rules engine; whether one engine serves both the app and training; where bots run (in the browser or on a server); UI framework; hosting; storage; how trained models ship to the browser.

## 3. Bots

Decided: the trained agent is the core (D1). A search bot is a benchmark only (D10). Explanations come from ML explanation methods on the trained agent (D1).

Design decided in Batch 2 (D14–D26; sources `research/05`, `06`, `07`; `rule-switches.md` for which toggles touch play):

- **Method (D14).** PPO with a full-information critic, self-play, 20% past checkpoints. Deep Monte Carlo run on the same pipeline as control and fallback.
- **Network (D18, D15, D19).** Transformer over the sequence of play and bid tokens, at least 4 layers, 256 wide. One state token carries the flat card-aligned state (~620 features from research/06) and the exact inference bits. One network for all phases, phase one-hot, ~140-way masked action vector. Heads: policy, observation-only Q per action, decomposed value, partner-seat and unseen-card-location auxiliaries. Full-information critic separate, training only. Control run: flat MLP (512-512-256) on the same pipeline. The transformer is the primary model and is not dropped at the first setback.
- **Token sequence (D21).** Bid tokens appear from stage 1, produced by a rule-based bidder until bidding is learned. Then king call, talon, announcement and card-play tokens as the curriculum adds them.
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

Not decided. The course wants numbered, checkable goals (README and report). Candidate kinds of measures:

- Rules correctness: number of rule scenarios passing, per rule switch.
- Bot strength: score difference over N mirrored deals with a confidence interval; ladder between bots.
- Speed: time per bot move on a named phone; app load size.
- Explanations: share of moves for which the app gives a reason; a check that the reason matches the bot's numbers.
- Teaching: completion of a tutorial by a test user; before/after puzzle score.
- Process: share of commits traceable to a logged prompt and model.

## 5. Working with agents day to day

Not decided. To cover: how tasks are split, which files carry state, how output is verified before each commit, how sessions are logged, how commits map to prompts and models (course requirement R14).

## 6. Course compliance

See `course-requirements.md`. A mapping from each requirement (R1–R20) to the plan item that satisfies it will be added here once the plan has content.

## 7. Timeline

Fixed points: 18 Nov (proposal, Figma wireframe, pitch slide), 23 Nov (repo live, pitch), 13–14 Dec (sprint-1 milestone, checkpoint), 11 Jan (everything). Sprint contents not decided.
