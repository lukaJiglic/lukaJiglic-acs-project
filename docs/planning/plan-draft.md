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

Not decided. To cover: every mode and screen, rule switches, difficulty, each expansion with its tier.

### Idea pool (none chosen)

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

Draft design, pending Batch 2 answers (sources: `research/05`, `06`, `07`; `rule-switches.md` for which toggles touch play):

- **Method.** PPO with a full-information critic, self-play, 20% past checkpoints. Deep Monte Carlo run as control and fallback.
- **Network.** Flat card-aligned encoding (~620 features), MLP 512-512-256, ~0.8 M parameters. One network for all phases, phase one-hot, ~140-way masked action vector. Heads: policy, observation-only Q per action, decomposed value, partner-seat and card-location auxiliaries. Full-information critic separate, training only.
- **Curriculum.** Card play → talon → bidding → announcements and kontra → klop and berač. Rule vector and preset mixture in stage 3.
- **Pipeline.** Rust engine and forward pass, self-play actors on CPU, PyO3 to a PyTorch learner on the GPU. Gate: ≥ 20,000 steps/s measured in week 1. Same Rust crate compiles to WASM for the browser; weights shipped as a ~1.5 MB f16 file; no ONNX runtime in the browser.
- **Explanations.** Layered: default = chosen card, expected score, margin, belief strip; tap = Q bars, reward components, top-3 perturbation attributions; review = contrastive vs the learner's move, one counterfactual, concept readouts, replay. Faithfulness checks D1–D10 from research 07 become numbered goals. Adviser asks for the learner's move first.
- **Other bots.** Random, rule-based, PIMC/ISMCTS benchmark (stage 3), cheating full-information agent as upper bound, checkpoints.
- **Levels and styles.** Levels = random, rule bot, measured checkpoints. Styles = fine-tuned copies, later tier.
- **Evaluation.** Seeded deal files, 8-seating duplicate scheme, 2,000 routine / 10,000 release deals, paired t-test, behaviour monitors, per-deal spread measured early.
- **Training time (inferred, to be measured).** With the Rust pipeline: 10^8 steps in 17 min–0.9 h, 10^9 in 3–9 h. With a Python environment: 10^9 in 2–6 days. Budget: 5–10 runs of 10^8, 2–3 runs of 10^9.

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
