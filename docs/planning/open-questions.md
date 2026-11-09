# Open questions

## Where planning stands

- 2026-11-06, session 1: course requirements extracted, first research pass done (4 files), state files created.
- 2026-11-09, session 2: Batch 1 answered (see `decisions.md` D1–D13, F1–F7). Git started, private repo `lukaJiglic/lukaJiglic-acs-project` pushed. Deep research on training, network/browser inference and explanations started (`research/05–07`). Rule-switch list drafted (`rule-switches.md`).
- 2026-11-09, later: Batch 2 answered (D14–D26), C1–C4 answered (D27–D29). Report source copied to `report/`.
- **Batch 3 (product and screens) is written and waiting for answers.** Clarification C5 is open.
- The proposal, Figma wireframe and pitch slide are due Wed 18 Nov. The product batch must follow soon after the bots batch.

## Clarifications

Answered 2026-11-09: C1 (by D16), C2 (D27), C3 (D28), C4 (D29).

Open:

- **C5. "Exact inference bits" (D18).** Claude reads this as the facts known for certain from play: which seats are void in which suit, the revealed partner seat, the cards already accounted for, the talon cards seen, whether the called king has fallen. Is that the meaning, or do you also mean derived counts (trumps still out per seat, points so far)?

## Batch 3 — product and screens (asked 2026-11-09, unanswered)

Context: Learn → Play → Table (D2), Learn screens built first (D29), live adviser in the core (D3), adviser asks for the learner's card first (D17), explanations layered (D16), own SVG cards (D11), 4-player (D4). Sources: `research/02` (existing apps, borrowable features), `research/07` (explanation display). No new research was run for this batch; the options are Claude's design proposals.

**B3-1. Primary user.**
- a) A player who knows the rules and wants to play better. Beginners get a rules layer on top.
- b) A beginner who has never played tarok.
- c) Both as equals.
- Recommendation: **a**. The explanation signals (expected score, partner belief, attributions) are about strategy. A beginner needs rules first, which is cheap to add as rule cards. Designing for the beginner first would push the agent's output out of the main screens.

**B3-2. Form factor.**
- a) Phone-first web app (portrait), installable as a PWA, usable on desktop.
- b) Desktop-first, phone later.
- c) Both from the start, separate layouts.
- Recommendation: **a**. The table adviser is used on a phone. One responsive layout keeps the Figma work small.

**B3-3. Screen list for Tier 0** (the Figma wireframe).
- a) Home · Learn hub (lessons, progress) · Lesson deal (table with teaching overlay) · Play table (game screen with explanation panel) · Post-game review (trick by trick, divergence, score loss) · Adviser (hand entry → trick-by-trick entry → learner's card → contrast) · Rules and presets (toggles, incompatibility warnings, share code) · Levels (bot ladder with measured strength) · Settings (language, card style).
- b) Same, but Lesson deal, Play table and Adviser are one table component in three modes (one screen in Figma, three states).
- c) A trimmed list: Home, Table (3 modes), Review, Rules. Levels and Settings folded into Home.
- Recommendation: **b**. One table component with modes is less to build and keeps the explanation panel identical everywhere. Levels and Settings stay separate screens because they carry report results (strength ladder) and the rule toggles.

**B3-4. What "teaching as you go" means.**
- a) Scripted lessons: each lesson is a seeded deal chosen to trigger one concept (e.g. partner inference, pagat safety, when to kontra); the explanation layer narrates; the learner plays it.
- b) Coached free play: the learner plays any deal; the app interrupts when the learner's intended card loses more than a threshold against the agent's choice, and shows the contrast.
- c) Both: lessons first, then coached play with an adjustable interrupt threshold.
- Recommendation: **c**. Lessons give structure for a beginner-to-intermediate path; coached play scales without writing content. Lesson deals can be mined from self-play by the key-moment method (research/07, method 13).

**B3-5. Lesson content and who writes it.**
- a) Claude drafts the lesson list and texts from the rules and the agent's signals; the user, who plays, corrects them.
- b) The user writes them.
- Recommendation: **a**. Template sentences are bound to signals (D16), so most "teaching" text is generated from numbers. Strategy framing needs a player's judgement, which is the user's review step.

**B3-6. Learn features and tiers.** Proposed: post-game review (T0); coached play (T0); scripted lessons (T0, small set; grows in T1); bidding adviser from the Q over bids (T1); "what if" replays (T1); puzzles generated from key moments (T1); progress tracking (T1); card-counting trainer (T2); puzzles from own mistakes with spaced repetition (T2).
- a) Accept. b) Move the bidding adviser to T0. c) Other changes.
- Recommendation: **a**. The bidding adviser is cheap once bidding is learned, but bidding is learned in stage 2, after card play.

**B3-7. Adviser input flow (T0).**
- a) Hand entry on a 54-card grid (tap 12 cards). Then bidding entry (what each seat bid), king call, talon shown if the learner is declarer. Then trick by trick: tap the cards others played in order; the grid hides cards already seen. Before each of the learner's plays: tap the intended card → agent's card and contrast.
- b) As a, but play phase only in T0 (bids and talon entered as a summary: contract, declarer, called king).
- c) Voice or camera entry.
- Recommendation: **b** for T0, **a** in T1. The agent needs the contract and the called king to reason; it does not need the full bid sequence to give card-play advice. Entering every bid is friction at a real table. Camera entry is out (high risk, no gain for the core).

**B3-8. Play features and tiers.** Proposed: play vs bots (T0); levels (T0); daily deal with comparison to the bots (T1); share a deal by code (T1); beat-the-bot duplicate mode (T2); online play with friends (out of the course scope: needs a server and adds nothing to the agent core).
- a) Accept. b) Keep online play as T3 (after the course). c) Other.
- Recommendation: **b**. Listing it as T3 records the idea without planning for it.

**B3-9. Table tools and tiers.** Proposed: scoring calculator (T1; the rules engine already computes it); house-rules share code (T1; the preset is a short string); score sheet for a session (T2); rules arbiter (T2).
- a) Accept. b) Other.
- Recommendation: **a**.

**B3-10. Languages.**
- a) i18n keys from the first screen, English text first, Slovenian added as a translation pass. Tarok terms stay Slovenian in both.
- b) Slovenian first.
- c) English only during the course.
- Recommendation: **a**. The demo and report are in English; the users are Slovenian; keys from the start cost little, retrofitting costs a lot.

**B3-11. Figma wireframe.**
- a) Claude writes a screen-by-screen spec (layout, elements, states); the user draws it in Figma.
- b) Claude also builds a clickable HTML wireframe from the spec; the user screenshots it into Figma or redraws it.
- c) Use a Figma connector if one is available in the user's Claude setup.
- Recommendation: **b**. The HTML wireframe becomes the first UI code later; the spec is needed either way.

**B3-12. Tier scheme.** T0 = must work for the demo; T1 = planned; T2 = stretch; T3 = after the course.
- a) Accept. b) Other.
- Recommendation: **a**.

## Batch 2 — bots (asked 2026-11-09, answered 2026-11-09; see D14–D26)

Sources: `research/05` (training method), `research/06` (network, browser), `research/07` (explanations). Claude read the method, failure-mode, evaluation and recommendation sections of all three; the subagents read the papers. Key numbers that are inferred, not measured: pipeline step rates, training hours, tarok's per-deal score spread. Nothing in the literature trains a rule-conditioned card-game policy, and no user study of explanations exists for any card game.

**B2-1. Training method.**
- a) PPO actor-critic with a full-information critic (critic sees all hands, talon and true partner during training; actor sees only its own view), self-play with 20% past checkpoints. Plus a Deep Monte Carlo (DouZero-style) run on the same pipeline as a control and fallback.
- b) Deep Monte Carlo only.
- c) PPO with a behaviour-cloning warm start from a search bot from day one.
- d) AlphaZero-style with determinisation.
- Recommendation: **a**. PerfectDou's ablation (read by the subagent): vanilla PPO against DouZero reached win rate 0.509; the same PPO with a full-information critic reached 0.738. The hidden partner is exactly what the critic can be told. DMC needs 10^9–10^10 steps in Dou Dizhu. AlphaZero-style lost at student scale without supervised pretraining (Jass, Klaverjas). c is kept as a fallback if the agent loses badly to the benchmark.

**B2-2. Heads on the network (what explanations are built from).**
- a) Actor trunk with: policy head (masked logits), an observation-only Q-per-action head regressed on returns (usable in the browser, gives "expected score per card"), a decomposed value head (trick points / king / pagat / announcements / penalties), auxiliary heads for partner seat and unseen-card location at low loss weight. Full-information critic is a separate network, training only.
- b) Policy and value only. Explanations post hoc (probes, perturbation).
- c) Q-per-action only (DMC style).
- Recommendation: **a**. The PPO critic needs hidden information, so it cannot produce expected scores on the phone; an observation-only Q head fills that gap. Research 07's methods 1, 4, 11 and 12 need these heads. Cost at inference is negligible. Risk: auxiliary losses can weaken play; mitigation is to measure strength with and without and keep weights ≤ 0.1.

**B2-3. Explanation methods and order** (this also answers C1).
- a) In order: (1) intrinsic signals (Q per card, margin, reward components) rendered through templates bound to thresholds; (2) belief and concept readouts (partner estimate, trumps out, pagat safety, final score) via auxiliary heads and post-hoc probes; (3) perturbation attribution by card swaps and single-edit counterfactuals; (4) review replays and key-moment summaries. LLM narration optional and last, local Ollama model, review mode only, with a hallucination check.
- b) Only (1) and (2).
- c) LLM narration as a core feature.
- Recommendation: **a**. Each method has a numbered faithfulness check (research 07, D1–D10), which feeds the course's checkable goals. Evidence from chess: a good action-specific attribution raised puzzle accuracy from 57% to 72%; two older saliency methods lowered it. LLM commentary has documented hallucination even with strong models and does not run on phones.
- Sub-decision: in adviser mode the learner enters their intended card first, then sees the agent's card and the contrast. Reason: studies on human-AI decisions show people copy the AI otherwise. Recommendation: yes.

**B2-4. Network and encoding.**
- a) Flat encoding of about 620 features (own hand, cards played per seat, current trick, talon seen, discards, contract, called king, partner seat if revealed, announcements, counts, phase, rule vector), seat-relative. MLP 620→512→512→256, about 0.8 M parameters (1.5 MB fp16). GRU over the play history as a measured ablation later.
- b) GRU over history from the start.
- c) Transformer over card tokens.
- Recommendation: **a**. Every credible trick-taking net is a shallow wide MLP of 1–7 M parameters; the Skat imitation net at 2.4 M ran at 2.5 ms per move on one CPU thread. A card-aligned flat encoding keeps attributions readable per card. Start simple, measure, add order information only if the ablation says so.

**B2-5. Phases and action space.**
- a) One network for all phases. Phase one-hot in the input. One masked action vector of about 140 logits (bids, king call, talon choice, single-card discards in sequence, announcements and kontra one at a time, 54 cards).
- b) Separate networks per phase.
- Recommendation: **a**. Masking is a valid policy gradient (Huang and Ontañón). Separate heads are an optimisation to try only if bidding underperforms.

**B2-6. Rule switches inside the agent.**
- a) One network with a rule vector, trained on a mixture of presets from the start.
- b) One model per preset.
- c) First model on one default preset with play-changing toggles fixed; rule vector and preset mixture added in a later stage (research 05 stage 3). Scoring-only toggles can be inputs from the start.
- Recommendation: **c**. Rule-conditioned card-game policies have no published precedent, so start without that risk and add it once the basic loop is proven. The rule list and the default preset come in Batch 4.

**B2-7. Curriculum.**
- a) Phase by phase: card play with a fixed contract and a rule-called king → talon exchange → bidding → announcements and kontra → klop and berač last. Each phase has monitors (declare rate, pass rate, announcement rate, partner-prediction accuracy, legal-move violations).
- b) Whole game from the start.
- Recommendation: **a**. Bidding collapse ("never bid") is documented three times in this game family; kontra must exist before bidding is trained or the agent over-bids.

**B2-8. Training pipeline.**
- a) Rust engine with the forward pass in Rust; self-play actors on 12 CPU threads; PyO3 hands batches to a PyTorch learner on the GPU. Gate: at least 20,000 steps/s end to end, measured in week 1.
- b) Rust engine, Python actors with batched GPU inference.
- c) Python environment.
- Recommendation: **a**, decided by measurement. Research 05 and 06 differ on where inference runs during self-play (GPU batches vs CPU in Rust); both agree the engine must be Rust. A 10^9-step run is estimated at 3–9 h with a fast pipeline and 2–6 days in Python. These rates are inferred; no Rust trick-taking engine publishes a benchmark. PyTorch still ships CUDA wheels for this GPU (Turing). torch.compile is not usable on it under Windows; eager mode is fine. 6 GB VRAM is not a constraint for a 0.8 M-parameter net.

**B2-9. Other bots.**
- a) Random bot and a rule-based bot (per-contract heuristics) in stage 0; PIMC/ISMCTS search bot in stage 3 as the benchmark and, if needed, a behaviour-cloning teacher; a cheating full-information version of the agent as an upper bound; past checkpoints as yardsticks.
- b) Search bot early (stage 0–1).
- Recommendation: **a**. The search bot is only needed when there is something to benchmark (D10). Build it early only if bidding collapse needs seeding with non-passive games.

**B2-10. Difficulty levels and styles.**
- a) Levels = random bot, rule bot, and checkpoints of the trained agent at measured strengths; styles (e.g. aggressive bidder, cautious) = fine-tuned copies with small reward changes, later tier.
- b) Levels by sampling temperature only.
- c) Separate training runs per level.
- Recommendation: **a**. Checkpoints give measured strength for free from the evaluation harness. Temperature alone produces random blunders rather than weaker play.

**B2-11. Evaluation protocol.**
- a) Fixed seeded deal files. Each deal played in 8 seatings (A in each seat with B elsewhere, and the reverse). Mean, standard error, paired t-test. 2,000 deals for routine checks, 10,000 for release claims, recomputed once the per-deal spread is measured. Behaviour monitors from day one. Human sessions of 50–200 deals reported as anecdote with standard error.
- b) Simpler: 4-seat rotation, 1,000 deals.
- Recommendation: **a**. Sample-size formula n = (1.96·σ/d)². With an assumed σ of 50 points per deal, ±5 points needs about 400 deals, ±2 points about 2,400. σ for tarok is unknown and must be measured.

**B2-12. Stages and budget** (research 05, G2; weeks counted from the start of building).
- Stage 0 (wk 1–2): engine, rules tests, harness, random and rule bots, pipeline speed gate.
- Stage 1 (wk 2–3): card play only, 10^7 then 10^8 steps. Gate: beats random widely by 10^7; beats the rule bot at card play by 10^8.
- Stage 2 (wk 3–5): talon, bidding, announcements, kontra, then klop and berač.
- Stage 3 (wk 5–6): search benchmark, tournament, rule vector on a preset mixture; BC warm start if the agent loses badly.
- Stage 4 (wk 6–8): one or two 10^9 runs overnight, export to WASM, phone test, human sessions.
- Stage 5 (wk 8–9): evaluation tables, ablations, writing.
- Options: a) accept as the draft schedule; b) compress stages 1–2; c) other.
- Recommendation: **a** as a draft; the sprint plan in Batch 8 will align it with 23 Nov, 13 Dec and 11 Jan. Note: 66 days from 2026-11-06 is about 9.4 weeks including planning, so the stages overlap the proposal and checkpoint dates.

## Batch 1 — foundations (asked 2026-11-06, answered 2026-11-09; kept for the record)

### Facts only the user knows

- **F1. Topic approval.** Is a tarok app on the Moodle topic list (e.g. as "browser game"), already approved, or still to be approved at consultation? Own topics need approval before 18 Nov.
- **F2. Your tarok.** Which form do you play (3 players, 4 players with a called king, both)? How well do you play? Which house rules does your table use?
- **F3. Time.** Hours per week until 11 Jan. Dates you cannot work. (The exam is 4 Jan.)
- **F4. Compute.** Is this PC (Ryzen 5 5600, 32 GB, GTX 1660 SUPER 6 GB) the training machine? Can it run overnight or for several days? Any other compute: the course's dingo server, a cloud budget?
- **F5. Review skills.** Which languages can you read well enough to catch an agent's mistakes: TypeScript/Vue, Python, Rust, C++?
- **F6. Agent tools.** Which tools and plans do you have (Claude plan and its limits, the course's pi + dingo, local Ollama)? Does the course expect local or dingo models to be used?
- **F7. Tarok Counter.** Research found "Tarok Counter" (GitHub user gajbrus; web + Android, closed source, updated 2026-11-04): score sheet with a rule engine, heuristic and Monte Carlo bots, online play, replays. Do you know it? Is it a classmate's or a friend's?

### Decisions

**D1. Centrepiece — what must be excellent, with the rest thinner.**
- a) The explaining bot. One rules engine, one bot that scores every legal move, and an explanation built from those scores. Play, learn and table use are layers on it.
- b) Self-play-trained bots and their measured strength.
- c) The teaching experience (tutorial, review, puzzles).
- d) The real-life table tools.
- Recommendation: **a**. All three uses need it. It gives checkable numbers for the report. Research found only one tarock app with "ask the bot why" (KubiTarock, for Königrufen) and none for Slovenian tarok.

**D2. Order of the three uses if time runs short.**
- a) Play → Learn → Table.
- b) Learn → Play → Table.
- c) Table first.
- Recommendation: **a**. Play is the test bench for the engine and bots. Learn is Play plus explanations. Table needs its own input screens. Note: play-against-bots already exists (TAROK.net, Tarock Palčka), so Play alone is not the pitch. The pitch is the explanation.

**D3. The real-life use.**
- a) Live adviser in the core, as in the brief.
- b) Adviser in Tier 1, on top of the play core. Input designed to work during a hand (with the table's consent) and after it.
- c) Score sheet and scoring calculator first, adviser later.
- d) Leave all real-life features for after the course.
- Recommendation: **b**. Research found no adviser for real tables, so it is the new part. Score sheets already exist. The adviser is only as good as the bot behind it, so it comes after the bot. Live advice is cheating unless the table agrees; after-the-hand entry avoids that.

**D4. Which form of the game is the baseline.**
- a) 4 players with a called king. No official ruleset exists; reference would be pagat.com plus the Tarok.net rulebook.
- b) 3 players. Official TARZS tournament rules exist (2010, amended to 2022). No hidden partner, so bots and explanations are simpler. Earlier academic work (Silicon Tarokist 2003) is 3-player, which gives a comparison.
- c) Both from the start.
- Recommendation: **the form you play most**, since you have to catch rule errors. If you play both equally: **b** for the core and 4 players as the first expansion. The engine should be designed for both either way.

**D5. What "played differently everywhere" covers.**
- a) Slovenian tarok and its house rules only.
- b) Also the neighbouring games (Königrufen, Hungarian tarokk, Czech taroky).
- Recommendation: **a**. The neighbouring games differ in deck, partner call or scoring unit. They are separate games, not settings. Research found 55 variation points inside Slovenian tarok alone; which become switches is a later batch.

**D6. Repository and planning files.**
- a) Start git now. Commit the planning files. Create `lukaJiglic-acs-project` on GitHub as private now, public by 23 Nov.
- b) Local git now, GitHub later.
- c) No git until building starts.
- Sub-question: planning files stay in `planning/` at the root, or move to `docs/planning/`?
- Recommendation: **a**, with `docs/planning/`. The commit history is graded as the log of how you worked, and planning is part of that. The course names `docs/` as the place for notes.

**D7. Order of the planning itself.**
- a) Product core and screens → architecture and agent workflow → bots → measurement → tiers and timeline. A deeper bot-research pass runs before the bots batch.
- b) Bots first, since they carry the most risk.
- c) Workflow first, since the repo goes live on 23 Nov.
- Recommendation: **a**. The proposal and Figma wireframe are due in 12 days and need the product and screens.

## Backlog — later batches (not asked yet)

**Batch 2: bots (next).** Training method. Network and input encoding. How the hidden partner and the phases are handled. Rule-set conditioning. Training budget and stages. Explanation methods (per D1). Benchmark bots (rule-based, search). Difficulty levels and styles from one trained model. Evaluation protocol.

**Batch 4: rules.** Confirm the toggle list in `rule-switches.md`: defaults, tiers, presets, the left-out list.

**Batch 5: architecture.** Engine language (one Rust engine compiled to WASM and used natively for training, vs TypeScript engine plus a separate training engine). UI framework. Where the model runs. Hosting. Storage. Model export path.

**Batch 6: measurement.** The numbered goals. Deals per comparison. Rules test counts. Phone speed targets. How explanations are checked. Whether to test with real learners.

**Batch 7: agent workflow.** Task splitting. Spec and progress files. Verification before each commit. Commit message convention that records prompt and model. Session logging. Which tools and models do what.

**Batch 8: tiers and timeline.** Final tier of every feature. Sprint contents for 23 Nov – 13 Dec and 14 Dec – 11 Jan. What the 10-minute demo shows.

## Loose questions noticed along the way

- The `project/` folder holds course material (instructions PDF, template). Should it go into the public repo, or only the report source?
- Who draws the Figma wireframe, and with what help?
- OpenSpiel's tarok engine (Apache-2.0) is marked "lightly tested" and may lack announcements. Worth a closer look before Batch 4.
- Tarok's per-deal score variance is unknown. It sets how many deals a strength comparison needs. Can be measured once an engine exists.
