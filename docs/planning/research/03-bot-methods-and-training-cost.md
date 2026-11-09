# 03: Bot methods and training cost (first-pass survey)

Date: 2026-11-06. Scope: planning only. No code.
Target: tarok bots that range from rule-based to self-play trained, explain their moves, and run in a phone browser.
Training PC: Ryzen 5 5600 (6c/12t), 32 GB RAM, GTX 1660 SUPER 6 GB, Windows 11.

Reading guide:
- "Verified" means I opened the source on 2026-11-06 and read the number there.
- "Inferred" means I derived it by arithmetic or scaling. Every such number is marked.
- "Memory" means I did not check a source today.

---

## (a) Sources checked

All accessed 2026-11-06. "Depth" says how much of the source I read.

| Key | Source | URL | Depth |
|---|---|---|---|
| S1 | Cowling, Powley, Whitehouse 2012, Information Set MCTS, IEEE TCIAIG | https://eprints.whiterose.ac.uk/75048/1/CowlingPowleyWhitehouse2012.pdf | Full text |
| S2 | Buro, Long, Furtak, Sturtevant 2009, skat program Kermit, IJCAI | https://cs.du.edu/~sturtevant/papers/skat.pdf | Full text |
| S3 | Solinas, Rebstock, Buro 2019, card-location inference for skat | https://arxiv.org/abs/1903.09604 | Full text (ar5iv) |
| S4 | Rebstock, Solinas, Buro, Sturtevant 2019, Policy Based Inference | https://arxiv.org/abs/1905.10911 | Full text (ar5iv) |
| S5 | Long, Sturtevant, Buro, Furtak 2010, why PIMC works, AAAI | https://ojs.aaai.org/index.php/AAAI/article/view/7562 | Abstract and summaries only |
| S6 | Ginsberg 2001, GIB (bridge), JAIR | https://arxiv.org/abs/1106.0669 | Abstract only (PDF text unreadable) |
| S7 | Sievers, Helmert 2015, doppelkopf player based on UCT | https://ai.dmi.unibas.ch/papers/sievers-helmert-ki2015.pdf | Full text |
| S8 | Whitehouse, Cowling, Powley, Rollason 2013, MCTS in AI Factory Spades (mobile), AIIDE | https://ojs.aaai.org/index.php/AIIDE/article/view/12679 | Full text |
| S9 | Zha et al. 2021, DouZero, ICML; and project README | https://arxiv.org/abs/2106.06135 ; https://github.com/kwai/DouZero | Full text (ar5iv, first 100k chars); README |
| S10 | Yang et al. 2022, PerfectDou | https://arxiv.org/abs/2203.16406 | Full text (ar5iv) |
| S11 | Jiang et al. 2019, DeltaDou, IJCAI | https://www.ijcai.org/proceedings/2019/176 | Paper page |
| S12 | Charlesworth 2018, PPO self-play for Big 2 | https://arxiv.org/abs/1808.10442 | Full text (ar5iv) |
| S13 | Patwa 2026, self-play RL in Big 2 | https://arxiv.org/abs/2605.28863 | Full text (HTML) |
| S14 | Li et al. 2020, Suphx (mahjong) | https://arxiv.org/abs/2003.13590 | Full text (ar5iv) |
| S15 | Heinrich, Silver 2016, NFSP | https://arxiv.org/abs/1603.01121 | Full text (ar5iv) |
| S16 | Brown, Lerer, Gross, Sandholm 2019, Deep CFR | https://arxiv.org/abs/1811.00164 | Full text (ar5iv) |
| S17 | Brown, Bakhtin, Lerer, Gong 2020, ReBeL | https://arxiv.org/abs/2007.13544 | Full text (ar5iv, first 100k chars) |
| S18 | Schmid et al. 2021/2023, Student of Games | https://arxiv.org/abs/2112.03178 | Full text (ar5iv, first 100k chars) |
| S19 | Blüml, Czech, Kersting 2023, AlphaZe** | https://www.frontiersin.org/articles/10.3389/frai.2023.1014561 | Search summary only |
| S20 | Niklaus et al. 2020, Jass (MCTS vs DNN) | https://zenodo.org/records/5769551 | Abstract and poster text |
| S21 | Aaldering 2023, ISMCTS + AlphaZero ideas in Klaverjas (BSc thesis, Leiden) | https://theses.liacs.nl/pdf/2022-2023-AalderingBBart.pdf | Abstract and parts |
| S22 | Emrich, SchafkopfRL (GitHub) | https://github.com/tobiasemrich/SchafkopfRL | README |
| S23 | Schumacher et al. 2022, DQN for Wizard | https://arxiv.org/abs/2205.13834 | Full text (ar5iv) |
| S24 | AlphaHearts Zero (Yale student project, 2022) | https://csec.yale.edu/node/60 | Abstract |
| S25 | Rebstock et al. 2024, GO-MCTS with transformers (Hearts, Skat, The Crew) | https://arxiv.org/abs/2404.13150 | Abstract |
| S26 | Edelkamp 2025, outer-learning framework for skat | https://arxiv.org/abs/2512.15435 | Abstract |
| S27 | Požrl 2021, Reinforcement learning in Tarock (MSc thesis, UL FRI) | https://repozitorij.uni-lj.si/IzpisGradiva.php?id=133070&lang=eng | Record page; PDF text searched |
| S28 | Oblak 2020, tarok bots (BSc thesis, UL FRI) | https://repozitorij.uni-lj.si/IzpisGradiva.php?id=121522 | Record page |
| S29 | Gams, Luštrek 2003, Intelligent System for Playing Tarok | https://hrcak.srce.hr/44748 | Record page |
| S30 | Marinčič, Gams, Luštrek 2006, bidding in four-player tarok (Tarok7) | https://dlib.si/stream/URN:NBN:SI:DOC-L4HX57BX/781f7188-4f97-4d3a-96bd-7d0a9cfed9d2/PDF | Not opened; known via search result and S27 |
| S31 | OpenSpiel game list and tarok source headers | https://github.com/google-deepmind/open_spiel/blob/master/docs/games.md | Game list; tarok.h; contracts.h |
| S32 | mytja/tarok (Palčka app, StockŠkis engine) | https://github.com/mytja/tarok | README pages |
| S33 | Zha et al. 2019, RLCard toolkit | https://arxiv.org/abs/1910.04376 | Full text (ar5iv) |
| S34 | Koyamada et al. 2023, Pgx simulators | https://arxiv.org/abs/2303.17503 | Full text (ar5iv) |
| S35 | ONNX Runtime Web docs (env flags; performance diagnosis) | https://onnxruntime.ai/docs/tutorials/web/env-flags-and-session-options.html ; https://onnxruntime.ai/docs/tutorials/web/performance-diagnosis.html | Both pages |
| S36 | TensorFlow.js WASM backend README (benchmarks) | https://github.com/tensorflow/tfjs/blob/master/tfjs-backend-wasm/README.md | Benchmark tables |
| S37 | Jangda et al. 2019, WebAssembly vs native speed | https://arxiv.org/abs/1901.09056 | Abstract |
| S38 | web.dev, cross-origin isolation (COOP/COEP) | https://web.dev/articles/coop-coep | Page |
| S39 | Mxplainer 2025, explaining Mahjong agents | https://arxiv.org/abs/2506.14246 | Abstract |
| S40 | Instructive AI for Hanabi (JHU/APL) 2021 | https://arxiv.org/abs/2111.01726 | Abstract |
| S41 | Lu et al. 2026, template-free explanations of MCTS with LLMs | https://arxiv.org/abs/2605.16524 | Abstract |
| S42 | NooK bridge AI, "white box" (news summary) | https://www.deeplearning.ai/the-batch/bridge-to-explainable-ai | Search snippet only |
| S43 | Burch et al. 2018, AIVAT variance reduction | https://arxiv.org/abs/1612.06915 | Full text (ar5iv) |
| S44 | Pluribus evaluation (secondary report) | https://syncedreview.com/2019/07/11/an-absolute-monster-bluffer-facebook-cmu-ai-bot-beats-poker-pros/amp/ | Search snippet only |
| S45 | Microsoft Research, TrueSkill | https://www.microsoft.com/en-us/research/project/trueskill-ranking-system/ | Page |
| S46 | Tian et al. 2020, Joint Policy Search (bridge bidding) | https://arxiv.org/abs/2008.06495 | Abstract |
| S47 | Kelidari et al. 2026, lightweight game agents (Gin Rummy) | https://www.alphaxiv.org/abs/2607.06854 | Abstract |
| S48 | Bridge double-dummy solver speed (Lead Solver docs) | https://lajollabridge.com/Software/Lead-Solver/Lead-Solver-About.htm | Search snippet only |
| S49 | RL-tarot (French tarot, GitHub) | https://github.com/sjoerdapp/RL-tarot | Repo page (no results published) |

Searches that returned nothing useful: Königrufen RL work; Sheepshead PPO results; a Hearts PPO paper; a published games-per-second benchmark for a Rust trick-taking engine.

---

## (b) Method survey

Each entry: how it works, published cost, reported strength, fit for tarok (3–4 players, hidden partner, bidding, many contracts).

### B1. Rule-based heuristics

- How: ordered rules pick a card. Oblak's tarok bots choose from a "hierarchically sorted set of actions" (S27 describing S28).
- Cost: no training. Developer time only.
- Strength: weak to medium. Oblak's advanced bot scored almost twice the points of a random bot (S28). XSkat (rule-based) lost clearly to Kermit (S2). A rule-based Gin Rummy expert still beat every lightweight RL agent 70–99% of the time (S47).
- Fit: good for easy levels and as rollout policy and evaluation yardstick. Rules must be written per contract (berač and klop need their own).

### B2. PIMC (determinization + perfect-information solver)

- How: sample complete deals consistent with what you know. Solve each as an open-card game. Play the move that wins most often (S2). GIB did this for bridge (S6). Kermit does it for skat and biases the samples by inference from bids (S2).
- Cost: no training for the search itself. Kermit's bidding evaluator was fitted on 22 million human games (S2). Kermit got 60 CPU seconds per game (S2). With 320 sampled worlds a move took 0.093 s (S3).
- Strength: Kermit played "roughly at the level of human experts" (S2). GIB was rated "approximately expert caliber" (S6). In Schafkopf, PIMC ranked above the PPO agents (S22).
- Known flaws: strategy fusion and non-locality (S1, S5). PIMC works best when hidden information is revealed quickly, which holds for trick-taking games (S5). Open-card solving "almost always" predicts a loss for the skat null-game soloist (S2). Expect the same problem for berač.
- Fit: good. A sampled deal fixes who holds the called king, so the hidden partner needs no special handling in search. Three- and four-player open-card search needs a multi-player rule (paranoid or max-n). The skat work is 2-team after bidding, like most tarok contracts. Klop is all-against-all and needs max-n or rollouts.

### B3. ISMCTS (Cowling, Powley, Whitehouse)

- How: one search tree over information sets. Each iteration samples a new deal and runs a normal MCTS pass restricted to moves legal in that deal (S1).
- Cost: no training. The authors expect 10,000 iterations to take about one second in an efficient implementation (S1). ISMCTS runs 2–4 times fewer iterations per second than determinized UCT (S1).
- Strength: in Dou Dizhu it was on par with determinized UCT (42.3% vs 43.6% win rate; a cheating player got 56.5%) (S1). In AI Factory Spades it beat the old knowledge-based AI and shipped in a mobile app (S8). In Jass, ISMCTS lost to determinized MCTS and to a DNN (S20, poster). In doppelkopf, a new deal per rollout (close to ISMCTS) did far worse than 10 fixed deals × 1,000 rollouts (S7).
- Fit: good for 3–4 players and hidden teams; doppelkopf has the same unknown-partner feature (S7). Results are mixed versus plain determinization, so test both.

### B4. AlphaZero-style with determinization

- How: a policy/value network guides tree search on sampled deals. Self-play games train the network. DeltaDou uses per-player information-set trees plus Bayesian inference of hidden hands (S11). AlphaZe** swaps MCTS for PIMC (S19).
- Cost: DeltaDou trained about 2 months on 68 CPUs for about 640,000 games, and needed 5–8 s per move on 8 cores (S11). Pgx AlphaZero baselines for small perfect-information board games took 6–18 hours on one GPU (S34).
- Strength: DeltaDou was roughly level with expert humans over 100 games (S11). In Klaverjas, ISMCTS with random rollouts (+55 points vs a rule bot) beat the self-play network version (+45.8) (S21). AlphaHearts Zero reports human-level play without numbers (S24).
- Fit: possible but expensive. Evidence at student scale says the learned value did not beat plain rollouts (S21).

### B5. Deep Monte Carlo (DouZero)

- How: no search. A Q-network scores every legal action. Targets are the final game return (plain Monte Carlo). Many parallel actors generate self-play games (S9).
- Cost: 48 CPU cores and four 1080 Ti GPUs, 45 actors, 30 days (S9). It passed heuristic bots in about half a day, the supervised agent in about 2 days (~5×10^8 steps), and DeltaDou in about 10 days (~5×10^9 steps) (S9). Network: LSTM over the last 15 moves plus 6 MLP layers of 512 (S9). Inference about 2 ms per decision on one CPU core (S10).
- Strength: first of 344 bots on Botzone; 58.6% win rate vs DeltaDou (S9).
- Fit: good shape for tarok. It is simple, needs no search at play time, and outputs a number per legal card. Bidding was a separate supervised network in DouZero (S9). Hidden partner is handled only implicitly through the history input.

### B6. PPO / actor-critic self-play

- How: a policy network and value network are trained by policy gradient on self-play games.
- Cost: Big 2 (4 players): one PC with 4 cores and a GPU, about 2 days, 150 million steps, about 3 million games (S12). A 2026 Big 2 study: 6-core i7 laptop, 7–13 hours, 320,000 games (S13). PerfectDou: 880 CPU cores and 8 GPUs, 2.5×10^9 steps, value network sees all cards during training (S10). SchafkopfRL: still improving after 15 days (S22).
- Strength: Big 2 agent beat amateur humans over 664 games (S12). The 2026 agent won 37–43% vs a heuristic bot in a 4-player game where 25% is par (S13). PerfectDou beat DouZero with about 10× fewer samples (S10). In Schafkopf, PPO ranked below PIMC and below an imitation agent (S22). No Hearts or Spades PPO paper found; Hearts work found is TD-learning and UCT (cited in S2).
- Fit: feasible on the student PC at Big 2 scale. Giving the value network full information during training (PerfectDou, S10) is cheap to add and fits hidden-card games.

### B7. NFSP

- How: each agent keeps a best-response Q-network and an average-policy network, and mixes them (S15).
- Cost: no hardware or wall-clock reported. The authors tried 99 configurations for limit hold'em (S15).
- Strength: near top-half ACPC 2014 bots in limit hold'em (S15). Deep CFR later beat it (S16).
- Fit: poor. Theory covers two-player zero-sum. Heavy tuning. No trick-taking results found.

### B8. Deep CFR

- How: neural networks replace CFR regret tables. External-sampling traversals generate training data (S16).
- Cost: hardware not reported. 10,000 traversals per iteration; thousands of SGD steps with batch 10,000–20,000 per iteration (S16).
- Strength: beat NFSP in poker (S16).
- Fit: poor. Guarantees are two-player zero-sum only (S16). Tarok trees are deep and 3–4 player.

### B9. ReBeL and Student of Games

- How: search and learning over public belief states, with CFR-style search (S17, S18).
- Cost: ReBeL poker used up to 90 machines with 8 GPUs each for data generation (S17). SoG used TPU resources similar to 3,500 actors (S18).
- Strength: superhuman poker (S17); strong in chess, Go, poker, Scotland Yard (S18).
- Fit: none for this project. Both are two-player zero-sum (S17, S18). SoG must enumerate information states per public state, which the authors call prohibitive in some games (S18). Card deals in tarok make that set enormous.

### B10. Suphx (mahjong) and oracle guiding

- How: supervised start, then RL. "Oracle guiding" first trains with hidden tiles visible, then drops those inputs out gradually (S14).
- Cost: 44 GPUs for 2 days per agent; 1.5–2.5 million self-play games; evaluation alone used 20 GPUs for 2 days (S14).
- Strength: above 99.99% of ranked Tenhou players (S14).
- Fit: the full system is out of reach. The oracle idea is cheap and transferable (see PerfectDou, B6).

### B11. Inference-based sampling in skat (Buro group)

- How: do not sample deals uniformly. Kermit weights deals by bids using tables learned from human games (S2). Solinas et al. predict each card's location with a network (S3). Rebstock et al. weight each sampled deal by the probability a policy network gives to the moves actually played (S4).
- Cost: networks trained on about 20 million human games (S3). Policy inference is about 5× slower per move than card-location inference (S4).
- Strength: each step added tournament points per game over the previous one (S3, S4). Adding inference made Kermit "significantly stronger" (S2).
- Fit: the mechanism fits tarok well (bids, king call, talon pick and who follows suit all carry information). No human game corpus is assumed for tarok, so the policy for weighting must come from the bots themselves. Void-suit tracking is free and should be used from day one.

### B12. Jass and Klaverjas

- Jass: determinized MCTS was on par with a DNN; ISMCTS lost; rule-based trump choice was almost as good as a DNN (S20, poster). I could not read the paper body, so move-time budgets are not verified.
- Klaverjas: see B4 (S21).
- Fit: both are fixed 2v2 partnership games. They support "determinized search first".

### B13. Schafkopf, Doppelkopf, Sheepshead, Wizard, French Tarot

- Schafkopf: ranking HP-PIMC > imitation > PIMC > PPO (LSTM) > PPO (linear) > rules > random; author says "still not close to human level" (S22).
- Doppelkopf: UCT with 10,000 rollouts per decision; best setup 10 deals × 1,000 rollouts; announcements raised the score; only 48 games vs a human (S7).
- Sheepshead: no RL results found.
- Wizard: DQN beat rule-based and random bots; adding tree search with sampled states did not beat plain DQN (S23).
- French Tarot: one GitHub project (S49), no results published.

### B14. Tarok and Königrufen

- Gams and Luštrek 2003: three-player tarok, alpha-beta with Monte Carlo sampling of hidden cards (S29). This is PIMC. Known as Silicon Tarokist (S27).
- Marinčič et al. 2006: bidding for four-player tarok by Bayesian network and by simulation (S30, not opened).
- Oblak 2020: three rule-based bots (S28).
- Požrl 2021: tabular Q-learning for three-player tarok. Klop and berač excluded. Models trained on 100,000 to 2 million games. Best model beat Oblak's bots but was on average worse than Silicon Tarokist (S27).
- OpenSpiel ships a "Slovenian Tarok" environment: 3–4 players, 12 contracts including klop, beggar, solo without and valat. It is marked lightly tested. Announcements, kontra and radli are not in the state (S31). No trained agents are published with it.
- StockŠkis (Palčka app): open-source Dart engine, GPL, "early alpha" by its own README (S32). Decision method not documented on the pages read.
- Königrufen: no RL or search paper found.
- Conclusion: no published deep-RL result for tarok exists that I could find. The strongest documented tarok bot is a PIMC program from 2003.

---

## (c) Training budget estimates

### C1. Published throughput numbers

| Source | Setup | Number | Derived rate |
|---|---|---|---|
| S33 RLCard (Python) | Dou Dizhu, 1M random games, 1 process | 87,271 s | 11.5 games/s |
| S33 | same, 16 processes | 7,276 s | 137 games/s |
| S33 | UNO, 1 process | 4,953 s | 202 games/s |
| S12 Big 2 PPO | 4 cores + GPU, 2 days | 150M steps, ~3M games | ~870 steps/s, ~17 games/s |
| S13 Big 2 2026 | 6-core laptop, 7–13 h | 320,000 games | 7–13 games/s |
| S9 DouZero | 48 cores + 4×1080 Ti | ~5×10^9 steps in ~10 days | ~5,800 steps/s |
| S11 DeltaDou | 68 CPUs, ~2 months | 640,000 games | ~0.12 games/s |
| S14 Suphx | 44 GPUs, 2 days | 1.5M games | 88 GPU-days per agent |
| S34 Pgx (JAX, GPU) | one A100 | slowest game ≥ ~10^5 steps/s | — |
| S7 Doppelkopf UCT (C++) | 1,000 games, 2 UCT players, 10,000 rollouts per decision | 19.5 h | ≥ ~3,400 rollouts/s |
| S1 ISMCTS | authors' expectation | 10,000 iterations ≈ 1 s | ~10,000 it/s |
| S8 Spades on a phone | Samsung Galaxy S II (2011) | 2,500 iterations < 0.25 s | > 10,000 it/s |
| S3 Skat PIMC | 320 worlds per move | 0.093 s | ~3,400 solved worlds/s |
| S48 Bridge double-dummy | 2,000 deals in 38 s (anecdotal) | 0.02 s/deal | ~50 deals/s |

Arithmetic for derived rates:
- RLCard: 1,000,000 / 87,271 = 11.5; 1,000,000 / 7,276 = 137; 1,000,000 / 4,953 = 202.
- Big 2: 150,000,000 / (2 × 86,400) = 868 steps/s; 3,000,000 / 172,800 = 17.4 games/s.
- Big 2 2026: 320,000 / (13 × 3,600) = 6.8; 320,000 / (7 × 3,600) = 12.7.
- DouZero: 5×10^9 / (10 × 86,400) = 5,787 steps/s. The paper's step counts are read from a plot, so treat as ±50%.
- Doppelkopf: 19.5 h = 70,200 s for 1,000 games = 70 s/game. Two UCT players × 12 card plays × 10,000 = 240,000 rollouts per game at least. 240,000 / 70.2 = 3,419/s. Lower bound, since bidding and announcement queries add rollouts.
- Spades: 2,500 / 0.25 = 10,000.

### C2. Simulator speed: Python vs Rust/C++

- Python, published: 11–200 full games per second per process for card games (S33).
- C++/Rust, published: I found no games-per-second benchmark for a trick-taking engine. The nearest anchors are search iterations, each of which plays a (partial) game to the end: 3,400/s (S7, 2015 desktop, tree-building UCT), ~10,000/s (S1 expectation, 2012), >10,000/s (S8, 2011 phone).
- Inferred: a bitmask-based Rust tarok engine should reach 10^4 to 10^5 search iterations per second per desktop core, and more for bare random playouts. This is an order-of-magnitude guess. It must be measured in week 1.
- Inferred consequence: with a compiled engine, the simulator is not the bottleneck for RL. Network inference and the Python training loop are.

### C3. Game size assumption

A four-player tarok deal has 48 card plays (12 tricks × 4). Add bidding, king call, talon choice, discards and announcements. I assume 55 decisions per deal. Inferred.

### C4. Scaled estimates for the student PC (all inferred)

Assumed pipeline speeds on this PC:
- P-slow: Python actor loop, Big 2 style. 868 steps/s × 1.5 (6 cores vs 4) = ~1,300 steps/s. With faster cores, say up to 2,000.
- P-fast: Rust engine, batched GPU inference, small MLP. Assumed 10,000–30,000 steps/s. Not measured anywhere; this is my engineering estimate.

| Scenario | Work | P-slow (1,300–2,000 steps/s) | P-fast (10,000–30,000 steps/s) |
|---|---|---|---|
| E1. Big 2 scale: 3M deals × 55 = 165M steps | PPO or DMC, small MLP | 23–35 h | 1.5–4.6 h |
| E2. DouZero "beats supervised agent": 5×10^8 steps | DMC | 2.9–4.5 days | 4.6–14 h |
| E3. DouZero "beats DeltaDou": 5×10^9 steps | DMC | 29–45 days | 1.9–5.8 days |

Arithmetic: 165×10^6 / 1,300 = 126,900 s = 35 h; / 2,000 = 23 h; / 10,000 = 4.6 h; / 30,000 = 1.5 h. 5×10^8 / 1,300 = 4.5 days; / 2,000 = 2.9 days; / 10,000 = 13.9 h; / 30,000 = 4.6 h. 5×10^9 is ten times each.

Cross-check by hardware ratio: DouZero used 48 cores and 4 GPUs. The student PC has 6 cores and one GPU that I believe is under half a 1080 Ti (memory). That is roughly 1/8 to 1/10 of the hardware. DouZero's "half a day to beat heuristic bots" then becomes about 4–5 days with their Python code. Inferred.

Caveats:
- Step counts do not transfer between games. Tarok has at most 16 legal cards per move; Dou Dizhu has thousands of combinations. Tarok may need fewer steps. It also has hidden teams and about a dozen contracts, which may need more. Unknown.
- What E1 buys, by analogy: "beats amateurs" in Big 2 (S12); 37–43% vs a heuristic in 4-player Big 2 at a tenth of that budget (S13).

Search-based work (no network), all inferred:
- E4. Tournament of 10,000 four-player deals, search bots in all seats, 1,000 iterations per move: 48 × 1,000 × 10,000 = 4.8×10^8 iterations. At 20,000 it/s per core on 6 cores: 4,000 s ≈ 1.1 h. At the published low of 3,400 it/s per core: 6.5 h.
- E5. Distillation set of 1 million decisions labelled by a 1,000-iteration search: 10^9 iterations. 2.3 h at 120,000 it/s total; 13.6 h at 20,400 it/s total. Training a small MLP on 1M samples then takes minutes on the GPU (memory-level estimate).
- E6. AlphaZero-style with search in the loop: 200 simulations × 48 moves = 9,600 network calls per deal. DeltaDou's 640,000 games would need 6.1×10^9 calls. At an assumed 50,000 batched calls/s: 34 h. At 10,000 calls/s: 7.1 days. One run would use a large share of the calendar.

### C5. Budget reading

- The deadline allows a handful of training runs of up to about one day each. Inferred from 9 weeks part-time.
- E1-size runs fit. E3-size runs do not fit unless the P-fast pipeline is real.
- Suphx, PerfectDou, ReBeL and SoG budgets are 2–4 orders of magnitude beyond this PC (S14, S10, S17, S18).

---

## (d) Explainability

What each family can show for "why this move":

| Family | Native explanation data | Notes |
|---|---|---|
| Rule-based | The rule that fired, and the rules that were skipped | Exact and cheap. Text can be written by hand per rule. |
| PIMC | Per card: share of sampled deals where it was best; mean score per card over deals | Kermit records "which move was best" per world (S2). Numbers assume open cards, so they are optimistic. |
| ISMCTS / UCT | Per card: visit count and mean reward at the root; most-visited continuation | Standard MCTS statistics (S1). |
| DMC (DouZero style) | One Q-value per legal card = expected final score | Direct "expected points if you play X" list (S9). |
| PPO / policy-value | Policy probability per card; value before and after a candidate move | Value differences need one extra forward pass per card. |
| Any | Counterfactual rollouts: replay the same sampled deals with card A vs card B | Same deals for both cards removes most noise. |
| Any search with sampling | Belief display: for each unseen card, the share of sampled deals in which each player holds it | Skat networks output exactly this per-card location table (S3). SchafkopfRL has a 32×4 hand-prediction output (S22). |

Published work on explaining game AI to people:
- Mxplainer: fits an interpretable parameterized search to a black-box Mahjong agent and explains single moves (S39).
- Instructive AI in Hanabi: turns the gap between a human's and an AI's strategy into short advice; reports improved human play (S40).
- NooK (bridge): described as a "white box" system that applies rules people understand (S42, snippet only).
- LLM explanations from MCTS traces using visit counts and value estimates (S41). Not card-specific.
- AI Factory Spades: users judge a bot by visible blunders, not win rate. Plain MCTS made such blunders; adding knowledge reduced them (S8). This matters for a teaching app.

Gap: I found no study that tests search-statistic explanations with learners of a trick-taking game.

Design notes (inferred):
- Search bots give the richest explanations at no extra cost. Pure policy networks give the thinnest.
- Show "average over N guessed deals", not "the truth". PIMC numbers ignore that opponents cannot see cards (S1, S2, S5).
- A belief display for the called king ("who is probably the partner") comes free from the sampled deals.

---

## (e) Running bots in a phone browser

### Runtimes

- ONNX Runtime Web: WASM is the CPU backend and is suggested for very small models. WebGPU is for devices with a capable GPU (S35).
- WASM multi-threading only turns on when the page is cross-origin isolated (S35). That needs COOP and COEP response headers (S38). Some static hosts cannot set headers (memory, not checked). Plan for one thread.
- ORT's proxy option runs inference in a Web Worker to keep the UI responsive; it does not make inference faster, and it does not work with WebGPU (S35).
- ORT docs advise uint8-quantized models and warn that float16 is slow on CPU (S35).
- TensorFlow.js WASM backend: comparable to WebGL for "lite" models of about 20–60M multiply-adds; several times slower for 100–500M (S36).
- Hand-written inference: an MLP is a few matrix-vector products. Writing it in the same Rust/WASM module as the engine avoids a runtime dependency. Inferred.

### Published latency numbers

| Model | Device | Backend | Time |
|---|---|---|---|
| MobileNet v2 | Pixel 4 | WASM | 182 ms |
| MobileNet v2 | Pixel 4 | WASM + SIMD | 82 ms |
| Face detector (lite) | Pixel 4 | WASM | 28 ms |
| Face detector (lite) | Pixel 4 | WASM + SIMD | 15.9 ms |
| Face detector (lite) | iPhone XS | WASM | 21.4 ms |

Source S36, measured in Chrome 79 (2020). Old, but it is the only mobile table I could verify.

Other verified facts:
- WASM runs SPEC CPU about 45% (Firefox) to 55% (Chrome) slower than native, with peaks of 2–2.5× (S37).
- A native ISMCTS did 2,500 iterations in under 0.25 s on a 2011 phone and shipped that way; weaker handsets got fewer iterations automatically (S8).
- DouZero's network needs about 2 ms per decision on one server CPU core (S10).

### Scaled estimates (inferred)

- A tarok MLP with 3–4 hidden layers of 256 has roughly 0.2–0.3M weights, so about 0.3M multiply-adds per forward pass. The lite face detector is in the 20–60M class (S36). Scaling 15.9 ms by 0.3/20 gives about 0.25 ms. Call overhead will dominate. Expect well under 1 ms per forward pass on a mid-range phone, if hand-written or run through ORT WASM.
- Model size: 0.3M float32 weights = 1.2 MB; 0.3 MB as int8.
- Search speed in a phone browser: take >10,000 it/s native on a 2011 phone (S8). A current mid-range phone core is several times faster (memory). WASM costs 1.5–2.5× (S37). Net: about 10^4 iterations per second or more is plausible. Must be measured on a real phone.
- So within about 1 s per move:
  - Plain PIMC or ISMCTS with a few thousand iterations: feasible.
  - Policy network alone: feasible with large margin.
  - Search plus a small network: feasible only if network calls are limited to hundreds or low thousands per move. Example: 30 sampled deals × up to 12 legal cards = 360 value calls, about 0.1–0.4 s. A network call at every tree node is not feasible at 10,000 iterations.
- Run search in a Web Worker so the UI does not freeze. Scale iterations to the device, as AI Factory did (S8).

Not verified: current ORT Web WASM bundle size; WebGPU availability on current Android and iOS browsers; any 2025–2026 phone benchmark for tiny MLPs.

---

## (f) Measuring bot strength

How the cited papers did it:

| Paper | Design |
|---|---|
| S9 DouZero | 10,000 random decks, each played twice with sides swapped. Metrics: win percentage and average point difference. Elo from 100,000 decks, 5 seeds. |
| S10 PerfectDou | Same 10,000-deck swap design; 100,000 decks for ablations; 1,260 games vs humans. |
| S2 Kermit | 800 deals. Card-play test: each deal played twice with roles swapped (1,600 games). Full-game test: all 6 seatings of two programs in 3 seats (4,800 games). Tournament scoring (Fabian-Seeger). Reported mean, standard deviation and game count. |
| S3, S4 skat inference | 2,500–5,000 matches, each two games with roles reversed. Tournament points per game. Paired t-test at p = 0.05. Some differences were not significant even at 5,000 matches. |
| S7 Doppelkopf | 1,000 deals, repeated in every seat permutation. Average score with 95% confidence interval. |
| S1 ISMCTS | Fixed set of 1,000 deals reused across experiments. A "cheating" player that sees all cards as an upper bound. |
| S12 Big 2 | 664 games vs humans; mean score ± error. |
| S14 Suphx | 1 million games vs fixed agents; bootstrap resampling for rank variance; 5,000+ games on Tenhou. |
| S46 Bridge bidding | About 1,000 boards vs WBridge5, IMPs per board, double-dummy scoring of the play. |
| S44 Pluribus | About 10,000 hands vs pros with AIVAT variance reduction. |

Techniques:
- Duplicate deals with seat rotation. This is the standard in every card paper above. For four players and two bot types, play each deal in several seatings (one A among three B, and the reverse, rotated through seats). Inferred from the skat 6-seating scheme (S2).
- Confidence intervals on mean score per deal. Use paired differences on the same deal.
- Control variates: AIVAT cut the standard deviation about 3×, equal to about 10× fewer hands, in poker (S43). It needs a value estimate for chance events; a simpler version for tarok is to subtract a baseline bot's score on the same deal and seat. Inferred.
- TrueSkill handles more than two players and teams by ranking outcomes; skill is a mean and an uncertainty (S45). It uses rank order only, not point margins, so it wastes information in a point-scored game. Inferred.
- Elo: DouZero computed Elo between two sides (S9). For hidden, changing partnerships, mean points per deal against a fixed reference pool is simpler and is what the skat papers report (S2, S3, S4).
- Exploitability proxies: (1) a cheating opponent or cheating version as upper bound (S1; S27 used a cheating agent too); (2) train a fresh RL agent against the frozen bot and see how much it wins (known technique; no source checked today).
- Use a strong fixed yardstick. Random opponents are too weak to rank agents, and self-play copies only tie (S47).

How many deals (arithmetic; inferred where marked):
- Win-rate metric near 50%: standard error = 0.5/√n. A 95% interval of ±2 points needs n = (1.96 × 0.5 / 0.02)² = 2,401 deals. ±1 point needs 9,604. DouZero's 20,000 games give about ±0.7 points.
- Score metric: n = (1.96 × σ / d)², with σ the per-deal standard deviation of the score difference and d the difference to detect.
- σ for tarok is unknown. Skat gives a guide: a standard error of 1.0–1.4 points per game over 5,000 games (S3) implies σ ≈ 1.0 × √5,000 ≈ 70–100 points per game. Inferred.
- With σ = 70: d = 5 points per deal needs 753 deals; d = 2 needs 4,706; d = 1 needs 18,824.
- Plan on 5,000–20,000 deals per comparison. With search bots this is hours of PC time (see E4 in section c). Measure tarok's real σ early and recompute.

---

## (g) Shortlist — recommendation (opinion, not fact)

Constraints applied: 9 weeks part-time; one mid-range PC; phone-browser inference; "why this move"; a range from simple to self-play trained.

### 1. Rule-based bots plus determinized search (PIMC with rollouts, or ISMCTS) in a Rust engine

- Why: no training. Runs the same code natively and as WASM. Best published track record in trick-taking games (S2, S7, S8, S20, S22). The only strong documented tarok bot used it (S29). A sampled deal fixes the hidden partner. Explanations come free from search statistics and from the sampled deals (belief display). A 2011 phone already ran it (S8).
- Role: easy levels = rules; strong level = search with rule-based rollouts; difficulty = iteration count.
- Main risk: open-card reasoning misplays berač, klop and deceptive situations (S2, S1), and can produce visible blunders with a partner (S8). Bidding needs its own evaluator. Browser iteration speed is unmeasured.

### 2. Self-play network without search: Deep Monte Carlo (DouZero style) or PPO with a full-information critic

- Why: this is the "trained by self-play on my own PC" requirement. Big 2 scale fits the hardware: hours to about a day per run (section c). Inference is under 1 ms on a phone. DMC gives a per-card expected score, which is a ready explanation. The same self-play data can train a bidding value model.
- Role: a fast "intuition" bot, and a mid-level opponent.
- Main risk: it may not beat the search bot or even good rules within the budget. Small-scale evidence points that way (S22, S21, S23, S47). Many contracts and hidden teams multiply what must be learned. Few tuning runs fit before the deadline.

### 3. Hybrid: learned network inside the search (stretch goal)

- Why: use the network from option 2 as rollout policy or as value at a shallow depth, and to weight sampled deals by how well they explain past plays (S3, S4). A cheap variant is supervised distillation of the search bot into a network (E5 in section c), which yields a strong instant-move bot.
- Role: top difficulty, only if options 1 and 2 are done.
- Main risk: time. Also the phone budget allows only hundreds to low thousands of network calls per move (section e). A learned value may not beat plain rollouts (S21).

Not recommended: NFSP, Deep CFR, ReBeL, Student of Games, full AlphaZero-style training, Suphx-style pipelines. Reasons: two-player zero-sum theory, or compute 2–4 orders of magnitude above this PC (S15–S18, S11, S14).

Suggested order (opinion): build 1 first, since 2 and 3 need its engine, its bots as opponents, and its tournament harness. Start one option-2 training run as soon as the engine is fast and correct.

---

## (h) Verified vs inferred

Verified in a source today:
- ISMCTS definitions, the 10,000-iterations-per-second expectation, Dou Dizhu win rates, 2–4× iteration slowdown (S1).
- Kermit: PIMC description, 22 million human games, 60 CPU s per game, tournament design, expert-level claim, null-game weakness of open-card solving (S2).
- Skat inference: 20 million games, 320 worlds, 0.093 s and 0.286 s per move, tournament sizes, points-per-game gains (S3, S4).
- Doppelkopf: 10,000 rollouts, 10×1,000 best, 19.5 h per 1,000 games, human test size (S7).
- Spades: 2,500 iterations under 0.25 s on Galaxy S II; blunder finding (S8).
- DouZero hardware, 30 days, actor count, network shape, evaluation design, results, time-to-beat figures (S9). PerfectDou hardware, steps, 2 ms / 6 ms inference (S10). DeltaDou 68 CPUs, 2 months, 640,000 games, 5–8 s per move (S11).
- Big 2: 4 cores + GPU, 2 days, 150M steps, ~3M games, human result (S12); 6-core laptop, 7–13 h, 320,000 games, win rates (S13).
- Suphx compute and game counts (S14). NFSP, Deep CFR, ReBeL, SoG scope and compute statements (S15–S18).
- Klaverjas scores (S21). SchafkopfRL ranking and 15-day note (S22). Wizard results (S23).
- Tarok prior work: Požrl (tabular Q-learning, 3 players, 100k–2M games, no klop/berač, result vs Silicon Tarokist), Oblak, Gams and Luštrek (S27–S29). OpenSpiel tarok exists with 12 contracts (S31).
- RLCard timings (S33). Pgx ≥10^5 steps/s and AlphaZero baseline hours (S34).
- ORT Web threading and backend guidance (S35). tfjs WASM phone benchmarks (S36). WASM 45–55% slower than native (S37). COOP/COEP requirement (S38).
- AIVAT 3× standard-deviation reduction (S43). TrueSkill description (S45).

Seen only as abstract, snippet or secondary summary:
- Long et al. 2010 on PIMC properties (S5). GIB details beyond the abstract (S6). AlphaZe** (S19). Jass paper body (S20; poster text only). GO-MCTS and Edelkamp results (S25, S26). NooK (S42). Pluribus evaluation numbers (S44). Bridge solver speed (S48). Marinčič et al. tarok bidding (S30).

Inferred (my arithmetic or scaling):
- All "derived rate" values in C1.
- 55 decisions per tarok deal.
- Every time estimate in C4 (E1–E6), and both pipeline speeds P-slow and P-fast.
- Rust engine speed of 10^4–10^5 iterations per second per core.
- Phone figures: under 1 ms per MLP forward pass; ~10^4 search iterations per second in WASM; 360 value calls in 0.1–0.4 s.
- Per-deal score standard deviation of 70–100 points (from skat), and all deal counts that use it.
- Seating scheme for four-player tests; baseline-subtraction control variate.

From memory, not checked today:
- GTX 1660 SUPER is under half the speed of a GTX 1080 Ti.
- Some static hosts cannot set COOP/COEP headers.
- Current phone cores are several times faster than a 2011 Galaxy S II.
- Training a fresh RL agent against a frozen bot as an exploitability proxy.
- Training a small MLP on 1M samples takes minutes on this GPU.
- First-author names in the source table for S10, S11, S14, S18, S34 and S46. The URLs are the ones I opened.

Could not verify:
- Any games-per-second benchmark for a Rust or C++ trick-taking engine.
- Any Königrufen bot research; any Sheepshead RL results; a Hearts or Spades PPO paper.
- Network sizes for the skat inference models.
- Per-move time budgets in the Jass study.
- How StockŠkis chooses moves.
- Current WebGPU support on phones and current ORT Web bundle size.
- Whether DouZero-style step counts transfer to tarok. No tarok deep-RL result exists to compare against.
