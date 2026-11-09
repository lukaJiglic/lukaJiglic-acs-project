# Open questions

## Where planning stands

- 2026-11-06, session 1: course requirements extracted, first research pass done (4 files), state files created.
- **Batch 1 is waiting for the user's answers.** Nothing is decided yet.
- Next after Batch 1: Batch 2 (product core and screens), because the proposal and Figma wireframe are due Wed 18 Nov.

## Batch 1 — foundations (asked 2026-11-06, unanswered)

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

**Batch 2: product core and screens.** Target user. Phone-first or desktop-first. Screen list for the Figma wireframe. What "teaching as you go" means in practice. Tier scheme and what Tier 0 contains. Languages (SL/EN) and when.

**Batch 3: rules.** Which of the 55 variation points become switches, and in which tier. Presets. Whether a switch may change play or scoring only. Reference source per rule. Direction of play, radelci, klop, kontra depth, rounding.

**Batch 4: architecture.** Engine language (one Rust engine compiled to WASM and used natively for training, vs a TypeScript engine plus a separate training engine, vs reusing OpenSpiel's tarok for training). UI framework. Where bots run. Hosting. Storage. Card artwork route (own index-only SVG cards vs public-domain scans as a skin).

**Batch 5: bots.** Bot list. Search bot settings. Whether a trained bot is in Tier 0 or Tier 1. Training method (Deep Monte Carlo vs PPO with a full-information critic vs distilling the search bot). Training budget. Explanation format. Difficulty levels and styles.

**Batch 6: measurement.** The numbered goals. Deals per comparison. Rules test counts. Phone speed targets. How explanations are checked. Whether to test with real learners.

**Batch 7: agent workflow.** Task splitting. Spec and progress files. Verification before each commit. Commit message convention that records prompt and model. Session logging. Which tools and models do what.

**Batch 8: tiers and timeline.** Final tier of every feature. Sprint contents for 23 Nov – 13 Dec and 14 Dec – 11 Jan. What the 10-minute demo shows.

## Loose questions noticed along the way

- The `project/` folder holds course material (instructions PDF, template). Should it go into the public repo, or only the report source?
- Who draws the Figma wireframe, and with what help?
- OpenSpiel's tarok engine (Apache-2.0) is marked "lightly tested" and may lack announcements. Worth a closer look before Batch 4.
- Tarok's per-deal score variance is unknown. It sets how many deals a strength comparison needs. Can be measured once an engine exists.
