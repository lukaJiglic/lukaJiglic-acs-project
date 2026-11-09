# Open questions

## Where planning stands

- 2026-11-06, session 1: course requirements extracted, first research pass done (4 files), state files created.
- 2026-11-09, session 2: Batch 1 answered (see `decisions.md` D1–D13, F1–F7). Git started, private repo `lukaJiglic/lukaJiglic-acs-project` pushed. Deep research on training, network/browser inference and explanations started (`research/05–07`). Rule-switch list drafted (`rule-switches.md`).
- **Batch 2 (bots) is written and waiting for answers.** Clarifications C1–C4 are also open (C1 is answered by B2-3).
- The proposal, Figma wireframe and pitch slide are due Wed 18 Nov. The product batch must follow soon after the bots batch.

## Clarifications asked on 2026-11-09 (unanswered)

- **C1. Explanation methods.** D1 says "ML explaining methods". Which kinds do you have in mind: per-move expected scores from the network's own outputs; attribution (which cards drove the choice); counterfactuals ("why not X"); concept probes (e.g. who the agent thinks the partner is); natural-language narration over those signals? Research 07 will give options; an early steer helps.
- **C2. Library reuse.** D9 bans code from other tarok engines. General libraries (PyTorch, ONNX Runtime, a UI framework, RL helper code) are assumed allowed. Correct?
- **C3. `project/` folder.** The course PDF and the report template are not committed yet. Commit them, or keep only the report source (`.tex`, `.sty`) in the repo?
- **C4. Learn first.** D2 puts Learn before Play. Does that mean the first screens built are the tutorial and review screens, with free play behind them, or only that Learn gets priority when time is short?

## Batch 2 — bots (asked 2026-11-09, unanswered)

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

**Batch 3: product core and screens.** Target user. Phone-first or desktop-first. Screen list for the Figma wireframe (due 18 Nov). What "teaching as you go" means in practice. The live adviser's input flow. Tier scheme and Tier 0 contents. Languages (SL/EN) and when.

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
