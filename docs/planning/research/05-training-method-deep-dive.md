# 05: Training method deep dive (decision-grade pass)

Date: 2026-11-09. Scope: planning only. No code.
Decision already taken: a self-play-trained agent is the core of the project. A search bot (PIMC/ISMCTS) exists only as a benchmark.
Game: 4-player Slovenian tarok with a called king (54 cards, bidding, hidden partner, talon, announcements, 12 tricks, point scoring, berač and klop as special contracts).
Training PC: Ryzen 5 5600 (6c/12t), 32 GB RAM, GTX 1660 SUPER 6 GB (Turing, sm_75), Windows 11. Target runtime: phone browser.
Deadline: 11 Jan 2027 (9 weeks from today).

Reading guide:
- "Full" = the full text was opened and read on 2026-11-09. "Abstract" = abstract or landing page only. "README" = GitHub README only. "Snippet" = seen only in a search result.
- "Inferred" = my arithmetic or scaling. Every such number is marked.
- "Memory" = not checked in a source today.
- Source keys here are T1–T62. Keys S1–S49 refer to the first-pass survey (03-bot-methods-and-training-cost.md).
- Part of the reading was done by three parallel research agents under my direction. I re-opened T1, T2, T30 and T57 myself to check their numbers. Those checks matched.

---

## (a) Sources

All accessed 2026-11-09.

### Hidden-partner games (Doppelkopf, Schafkopf, Sheepshead, Königrufen)

| Key | Source | URL | Depth |
|---|---|---|---|
| T1 | Emrich, SchafkopfRL (PPO, PIMC, HP-PIMC, imitation), GitHub 2020 | https://github.com/tobiasemrich/SchafkopfRL | README (full, re-checked by me) |
| T2 | adamanthil/sheepsheadai (PPO league + ISMCTS teacher), GitHub 2025–26 | https://github.com/adamanthil/sheepsheadai | README (full, re-checked by me) |
| T3 | Obenaus 2017, Doppelkopf AI using UCT and an LSTM, BSc FU Berlin | https://www.mi.fu-berlin.de/inf/groups/ag-ki/Theses/Completed-theses/Bachelor-theses/2017/Obenaus/BA-Obenaus.pdf | Full |
| T4 | Dockhorn et al. 2017, decision heuristic for MCTS Doppelkopf agents, IEEE SSCI | DOI 10.1109/SSCI.2017.8285181 | Abstract |
| T5 | jlittek/Doppelkopf; mafn/doppelkopf-training; Templeton 2011 (CS229) | GitHub / Stanford | README / short report |
| T6 | Schafkopf GitHub projects: ttbrunner/alphasheep, 33Jakob33/schafkopf-ai, clauszitzelsberger/Schafkopf_RL, Bonifatius94/Schafkopf.AI | GitHub | README |
| T44 | Sievers, Helmert 2015, Doppelkopf UCT, KI 2015 | https://ai.dmi.unibas.ch/papers/sievers-helmert-ki2015.pdf | Full |

Königrufen: no AI work of any kind found (searches in German and English).

### Tarok

| Key | Source | URL | Depth |
|---|---|---|---|
| T7 | Požrl 2021, Reinforcement learning in Tarock, MSc UL FRI | https://repozitorij.uni-lj.si/Dokument.php?id=150850 | Full |

### Skat

| Key | Source | URL | Depth |
|---|---|---|---|
| T8 | Rebstock, Solinas, Buro 2019, Learning Policies from Human Data for Skat, IEEE CoG | https://arxiv.org/abs/1905.10907 | Full |
| T9 | Solinas, Rebstock, Buro 2019, card-location inference, AAAI | https://arxiv.org/abs/1903.09604 | Full |
| T10 | Rebstock, Solinas, Buro 2019, Policy Based Inference | https://arxiv.org/abs/1905.10911 | Full |
| T11 | Rebstock et al. 2024, GO-MCTS (transformer planning; Hearts, Skat, The Crew) | https://arxiv.org/abs/2404.13150 | Full |
| T12 | Edelkamp 2025, Outer-learning framework for Skat | https://arxiv.org/abs/2512.15435 | Full |
| T45 | Buro et al. 2009, Kermit, IJCAI | https://cs.du.edu/~sturtevant/papers/skat.pdf | Full |

### Bridge

| Key | Source | URL | Depth |
|---|---|---|---|
| T13 | Yeh, Lin 2016, bridge bidding DRL, ECAI | https://arxiv.org/abs/1607.03290 | Abstract |
| T14 | Rong, Qin, An 2019, Competitive Bridge Bidding with DNNs, AAMAS | https://arxiv.org/abs/1903.00900 | Full |
| T15 | Gong, Jiang, Tian 2019/20, Simple is Better (end-to-end bridge bidding) | https://realworld-sdm.github.io/paper/42.pdf | Full (PDF) |
| T16 | Tian et al. 2020, Joint Policy Search, NeurIPS | https://arxiv.org/abs/2008.06495 | Full |
| T17 | Lockhart et al. 2020, Human-Agent Cooperation in Bridge Bidding | https://arxiv.org/abs/2011.14124 | Full |
| T18 | Kita et al. 2024, A Simple, Solid, and Reproducible Baseline for Bridge Bidding AI, IEEE CoG | https://arxiv.org/abs/2406.10306 | Full |

### Other trick-taking games

| Key | Source | URL | Depth |
|---|---|---|---|
| T19 | Shi, Li, Sun 2021, ScrofaZero (Gongzhu) | https://arxiv.org/abs/2102.07495 | Full |
| T20 | Schumacher, Pleines 2022, DQN for Wizard, IEEE CoG | https://arxiv.org/abs/2205.13834 | Full |
| T21 | Niklaus et al. 2020, Jass MCTS vs DNN, AAAI poster | https://zenodo.org/records/5769551 | Full (poster) |
| T22 | Bürli 2022, Learning Approximate Nash Equilibria for Jass (MuZero, CFR), HSLU | https://zenodo.org/records/7733443 | Full |
| T23 | Wyss 2024, RL for Tichu: DQN vs PPO, BSc Bern | https://prg.inf.unibe.ch/wp-content/uploads/2024/08/BA_NicolasWyss.pdf | Full |
| T24 | Elzayn, Hayhoe et al., Four Hundred (Whist family), UPenn project | https://www.seas.upenn.edu/~mferey/publication/ai-400/ai-400.pdf | Full |
| T25 | Hearts: Demirdöver et al. 2022 (TJEECS 30(6)); Culhane 2021 (CS230); Wei 2022 AlphaHearts Zero (Yale) | journal page; Stanford CS230; https://csec.yale.edu/node/60 | Abstract / Full / Abstract |
| T26 | Pugh 2020, Euchre with model-free RL (CS238) | Stanford CS238 report | Full |
| T27 | Oeschger 2022, RL for Cego (tarot-family game), RLCard fork | https://github.com/PhilippOesch/rlcard | README |
| T28 | Von York 2020, Catch Five PPO (CS230) | http://cs230.stanford.edu/projects_winter_2020/reports/32625681.pdf | Full |

### Reference systems (non-trick-taking)

| Key | Source | URL | Depth |
|---|---|---|---|
| T29 | Zha et al. 2021, DouZero, ICML; code | https://arxiv.org/abs/2106.06135 ; https://github.com/kwai/DouZero | Full; code read (dmc.py, arguments.py) |
| T30 | Yang et al. 2022, PerfectDou | https://arxiv.org/abs/2203.16406 | Full (re-checked by me) |
| T31 | Li et al. 2020, Suphx | https://arxiv.org/abs/2003.13590 | Full |
| T32 | Charlesworth 2018, Big 2 PPO | https://arxiv.org/abs/1808.10442 | Full |
| T33 | Patwa 2026, self-play RL in Big 2 | https://arxiv.org/abs/2605.28863 | Full |
| T34 | Fan, Farina 2026, GAE falls short in imperfect-information self-play; Lei, Lei 2024, AlphaDou | https://arxiv.org/abs/2605.19235 ; https://arxiv.org/abs/2407.10279 | Abstract / Abstract |

### Methods, self-play theory, failure modes

| Key | Source | URL | Depth |
|---|---|---|---|
| T35 | Yu et al. 2021, MAPPO | https://arxiv.org/abs/2103.01955 | Full |
| T36 | Pinto et al. 2017, Asymmetric Actor Critic | https://arxiv.org/abs/1710.06542 | Full |
| T37 | Vinyals et al. 2019, AlphaStar, Nature (unformatted manuscript) | https://storage.googleapis.com/deepmind-media/research/alphastar/AlphaStar_unformatted.pdf | Full |
| T38 | Hanabi: Other-Play (2003.02979), SAD (1912.02288), Off-Belief Learning (2103.04000), Learned Belief Search (2106.09086) | arXiv | Full |
| T39 | Huang, Ontañón 2020, invalid action masking | https://arxiv.org/abs/2006.14171 | Full |
| T40 | Balduzzi et al. 2019 (1901.08106); Lanctot et al. 2017 PSRO (1711.00832); Czarnecki et al. 2020 spinning tops (2004.09468) | arXiv | Full |
| T41 | OpenAI Five 2019 | https://arxiv.org/abs/1912.06680 | Full |
| T42 | Heinrich, Silver 2016, NFSP | https://arxiv.org/abs/1603.01121 | Full |
| T43 | Kujur 2026, collapse in self-play (poker, dice) | https://arxiv.org/abs/2605.16315 | Full |
| T46 | Burch et al. 2018, AIVAT | https://arxiv.org/abs/1612.06915 | Full |
| T47 | Pluribus evaluation (secondary summary) | https://www.greaterwrong.com/posts/6qtq6KDvj86DXqfp6/let-s-read-superhuman-ai-for-multiplayer-poker | Secondary |
| T48 | NIST e-Handbook 7.2.2.2, sample size for a mean | https://www.itl.nist.gov/div898/handbook/prc/section2/prc222.htm | Page |
| T49 | XLand (2107.12808); Soemers et al. 2021, Ludii transfer (2102.12375) | arXiv | Partial / Full |

### Pipeline, hardware, software

| Key | Source | URL | Depth |
|---|---|---|---|
| T50 | Zha et al. 2019, RLCard | https://arxiv.org/abs/1910.04376 | Full |
| T51 | Koyamada et al. 2023, Pgx | https://arxiv.org/abs/2303.17503 | Full (PDF) |
| T52 | EnvPool benchmark page | https://envpool.readthedocs.io/en/latest/pages/benchmark.html | Page |
| T53 | Petrenko et al. 2020, Sample Factory | https://arxiv.org/abs/2006.11751 | Full |
| T54 | PufferLib, RLC 2025 paper; blog | https://rlj.cs.umass.edu/2025/papers/RLJ_RLC_2025_151.pdf ; https://puffer.ai/blog.html | Full / Snippet |
| T55 | pons-dds README (Rust DDS port benchmarks) | https://docs.rs/crate/pons-dds/0.1.1/source/README.md | Page |
| T56 | PyO3: issue 3787 (call overhead); performance guide; parallelism guide | https://github.com/PyO3/pyo3/issues/3787 ; https://pyo3.rs/latest/performance.html ; https://pyo3.rs/latest/parallelism.html | Pages |
| T57 | PyTorch 2.8.0 release notes; dev-discuss CUDA 13.2 (release 2.12); dev-discuss CUDA 12.6 wheels end at 2.15 | https://github.com/pytorch/pytorch/releases/tag/v2.8.0 ; https://dev-discuss.pytorch.org/t/introducing-cuda-13-2-and-deprecating-cuda-12-8-release-2-12/3337 ; https://dev-discuss.pytorch.org/t/notice-cuda-12-6-wheels-will-no-longer-be-published-from-pytorch-2-15-drops-maxwell-pascal-volta/3432 | Pages (last one re-checked by me) |
| T58 | NVIDIA CUDA 13.0 release notes | https://docs.nvidia.com/cuda/archive/13.0.0/cuda-toolkit-release-notes/index.html | Page |
| T59 | GTX 1660 SUPER specs (flopper.io); Wikipedia GeForce 16 series | https://flopper.io/gpu/nvidia-geforce-gtx-1660-super-6gb | Page |
| T60 | PyTorch Inductor-on-Windows tutorial; triton-lang/triton-windows README | https://docs.pytorch.org/tutorials/unstable/inductor_windows.html ; https://github.com/triton-lang/triton-windows | Pages |
| T61 | RLCard issue 329 (DouZero port fps on CPU) | https://github.com/datamllab/rlcard/issues/329 | Page |
| T62 | OpenNN CPU MLP benchmark; rlox NN-backend benchmark | opennn.net ; riserally.github.io/rlox | Snippet only (pages 404) |

Searches that returned nothing: Königrufen AI of any kind; a Doppelkopf or Schafkopf thesis with neural self-play; Sheepshead RL beyond T2; a Skat self-play RL repository; HSLU Jass DQN/PPO theses 2021–24; Spades neural self-play; Belote, The Crew (beyond T11), Klaverjas neural RL; a Rust or C++ trick-taking engine with a published games-per-second figure; any paper conditioning one card-game policy on a rule-variant vector.

---

## (b) Closest published analogues

### B1. Summary table

Column "Hidden partner" says whether the game has a partner called by a card. Column "vs search" says how the neural agent compared with a PIMC/ISMCTS/UCT bot in the same paper.

| Key | Game | Hidden partner | Method | Network | Compute | Evaluation | Strength | Depth |
|---|---|---|---|---|---|---|---|---|
| T1 | Schafkopf (long) | Yes (called ace) | PPO self-play; also PIMC, HP-PIMC, imitation | Linear MLP or LSTM over history; 55-d info vector; 43 actions | Hardware not given; "15 days" and still slowly improving | 2 × 1000 same hands per pairing, cents/game | HP-PIMC > imitation > PIMC > PPO-LSTM > PPO-linear > rules. PIMC beat PPO-LSTM by 4.6 cents/game | README |
| T2 | Sheepshead | Yes (called card) | PPO: shaped bootstrap, privileged critic, PFSP league, ISMCTS teacher | "perceiver" encoders; separate pick/partner/bury/play heads; 4 aux heads incl. secret partner | 10-core M1 Max, CPU workers; 400k ep. bootstrap 6–8 h; 1M ep./generation ≈ 2 days; 30M-episode model | Duplicate deals, 2,000–8,000 per check; adopt if edge + 2·SE ≥ 0 | Only vs own checkpoints (edges of +0.003 to +0.024). No number vs rules, ISMCTS or humans | README |
| T3 | Doppelkopf | Yes (queens of clubs) | Supervised LSTM next-card prior inside UCT. No RL | 2 × 64 LSTM; 62-char game string | 0.5M online games; 2 days CPU | 4000 games, seat rotation every 1000 | LSTM alone: −0.36 ± 0.13/game vs UCT; UCT+LSTM: +0.04 ± 0.24 vs UCT (no gain) | Full |
| T4 | Doppelkopf | Yes | NN move classifier for MCTS playouts, human data | not read | not read | not read | not read | Abstract |
| T5 | Doppelkopf | Yes | jlittek: CNN + terminal reward, ε-greedy; mafn: PPO, self-declared broken | not documented | 24,000 games/day (jlittek) | plots only | none usable | README |
| T7 | Tarok (3-player) | No | Tabular Q-learning; bidding and talon hand-coded; klop, berač excluded | none | 100k–2M games | 15,000 games per matchup; 1,500 ten-game sessions | Best model beat Oblak's rule bots; worse than Silicon Tarokist (PIMC); "on many occasions beat an average player" | Full |
| T8 | Skat | No (soloist vs 2) | Supervised imitation, human data. No RL | 5 FC layers; 2.3–2.4M weights per net | 1 × 1080 Ti; 8 h (bidding) + 16 h (cardplay); up to 289M samples | 5,000 matches × 6 seatings vs Kermit, same deals | Best bidding + Kermit play: +1.17 TP/game; imitation cardplay alone: −2.1 to −4.1 TP/game | Full |
| T11 | Skat, Hearts, The Crew | No | GPT-2 (8 layers, 256 emb) trained by population-based fictitious self-play; MCTS over observations | ~? (8 layers, 8 heads) | Skat: 4M seed games + 20 iterations; hardware not given | 3,000-match tournaments, seat permutations, Wilcoxon | Hearts: beat xinxin by 1.74 points. Skat: lost to Kermit by 9.84 TP/game; 42 s per move | Full |
| T12 | Skat | No | Table-based self-play (no NN) for bidding | none | EPYC 16 cores; 1M games ≈ 1.5 days; 30M games > 1 month | 83,844 human deals; server play 1,000 deals rotated | Bidding accuracy 84.78% vs open-card optimum (humans 76.7%) | Full |
| T14 | Bridge bidding | No (fixed partner) | SL then REINFORCE self-play | ENN: 8 × 1500, predicts partner's 52 cards; PNN: 10 × 1200, 38 actions | 1M games SL; 2M deals RL; hardware not given | 10k deals net vs net; 64 boards vs WBridge5 | +0.25 IMP/board vs WBridge5 (64 boards, no CI); ENN added +1.09 IMP/b over PNN alone | Full |
| T15 | Bridge bidding | No | A3C pure self-play | small MLP, 38 actions | "4–5 hours ... on a single GPU" | duplicate tables, IMPs | +0.41 IMP/b vs WBridge5 (64 boards). Belief loss hurt: 2.31 → 1.22 IMP/b. Not reproduced by T18 | Full |
| T16 | Bridge bidding | No | A2C self-play + Joint Policy Search | 5 FC layers of 200; 267-bit input | 2 GPUs, 2 days (baseline); 14 days (JPS) | 1,000 boards vs WBridge5 | +0.63 ± 0.22 IMP/b; A2C alone +0.29 ± 0.22 | Full |
| T17 | Bridge bidding | No | Imitation (WBridge5 data) → search → policy iteration | 4 × 1024 MLP; 480-d input | ~1M deals; hardware not given | 10,000 deals; agent in each seat | +0.85 ± 0.05 IMP/deal; imitation alone −0.12 ± 0.05 | Full |
| T18 | Bridge bidding | No | SL pretrain → PPO + fictitious self-play (uniform past checkpoints) | 4 × 1024 MLP, 38 actions, action masking | 8192 envs, 10^4 updates; hardware not given | 1,000 boards vs WBridge5 | +1.24 ± 0.19 IMP/b. Without SL: cannot beat WBridge5. FSP alone: ineffective | Full |
| T19 | Gongzhu (Hearts-like) | No | AlphaZero-style on perfect information; sampled hands at test | FC with skips, 11.4M params | 2 × 2080 Ti; total not given | 1024 × 2 games vs 3 rule bots | +73.9 ± 3.3 points vs greedy bot with search; raw policy net ≈ 1 ± 16 (useless alone) | Full |
| T20 | Wizard | No | DQN self-play; one net per round for bidding, one for play | 3 hidden FC layers | not given | 10,000 rounds × 5 seeds | 80% win vs 3 rule bots. LSTM history and sampled tree search did not help | Full |
| T21 | Jass | No (fixed teams) | Supervised DNN vs DMCTS/ISMCTS | not read | 1.8M online rounds | tournaments | DMCTS on par with DNN; ISMCTS beaten | Poster |
| T22 | Jass | No | MuZero (from scratch; SL-pretrained + RL); CFR | 5 res blocks, 256 filters, 4–5.4M params | i7-9700K + 2 × RTX Titan + collectors; 2 × A6000 for eval | 1,000–20,000 rounds | From scratch: 45–46% points vs DMCTS (loses). SL + RL fine-tune: 53.2% vs DMCTS (p = 0.002) | Full |
| T23 | Tichu | No (fixed teams) | DQN, PPO, with/without behaviour cloning; 80/20 opponent sampling | 85 features; 1,415 masked actions | 32 CPUs, 150M steps | 10,000 duplicate rounds (hands shifted one seat) | PPO + BC: +90 pts/round vs rule bot; PPO alone: −210; DQN alone: −197 | Full |
| T24 | Four Hundred | No | Supervised bid regression + fitted-Q play | small nets | 100k games per bet net | 200 games | Beat heuristic 89.5%; conservative bidding | Full |
| T25 | Hearts | No | DQN (Culhane); modular RL (Demirdöver); PIMC+MCTS+DQN (Wei) | 8-layer MLP (Culhane) | not given | small | Culhane: ~30% wins (25% par); "human level ... wasn't achieved" | Full / Abstract |
| T26 | Euchre | No | RLCard DQN, NFSP | RLCard defaults | not given | 1,000 games | "just over 50%" vs smart rule bots | Full |
| T27 | Cego (tarot family) | No | RLCard DMC, DQN, NFSP | RLCard defaults | not read | not read | not read (thesis PDF in repo not opened) | README |
| T28 | Catch Five | No | PPO, one action vector across bid/suit/card | small MLP | 20k iterations | self-play | Early agents "rarely bidded"; fixed by tanh + data augmentation | Full |

### B2. Notes on the hidden-partner analogues

- Only two systems train a neural agent by self-play on a called-partner game: T1 and T2. Both are GitHub projects. Neither is peer reviewed. Neither beats a tuned search bot in its own tests. T2 reports no external baseline at all.
- T1 encodes the partner badly. The "teams" bits are set only after the called ace has been played. There is no input for which ace was called and no partner-belief output. The author lists "prediction head ... for prediction of teams" as future work. The PPO agent "sometimes plays higher trump than teammate" (T1).
- T2 does the opposite. It has a privileged (full-information) critic, a "secret partner" auxiliary head, and gates the league phase on "secret partner ≥ 99.5%" accuracy (T2). This is the closest design to what tarok needs. Its strength is unmeasured.
- T3 shows that a supervised next-card prior did not improve UCT in Doppelkopf. UCT value estimates "differed by less than 5 card points" between moves, so the prior had nothing to steer (T3).
- The only published partner-belief head with a measured effect is Rong et al.'s ENN in bridge: +1.09 IMP/board over the same policy net without it (T14, 10k self-play deals). Gong et al. found the opposite in their system: a supervised partner-hand belief loss lowered the score from 2.31 to 1.22 IMP/b as its weight rose (T15). Kita et al. could not reproduce Gong et al.'s headline result (T18). Treat the belief-head question as open.
- The bridge line is the best-documented evidence on self-play for bidding: pure self-play from scratch works (T15, T16) but every result above +0.6 IMP/b used supervised pretraining or search (T17, T18). T18: without SL pretraining the PPO agent "cannot beat WBridge5"; fictitious self-play "is ineffective on its own" but helps after SL.
- Across all games read: every deep self-play agent compared against a tuned PIMC/ISMCTS/DMCTS bot lost or tied (T1, T3, T11, T22 from-scratch, T26), unless it was combined with supervised pretraining (T22 fine-tuned, T17, T18). The exceptions are games without a strong search baseline (T20 Wizard, T32 Big 2).
- Compute of working systems on one machine: T32: 4 cores + GPU, 2 days, 150M steps. T33: 6-core laptop, 7–13 h, 320k episodes. T2: 10-core M1 Max, roughly 1–3 weeks for a 30M-episode model. T1: 15 days, unspecified CPU, 5–15M PPO steps. T23: 32 CPUs, 150M steps. T22: multi-GPU, from-scratch MuZero still lost.

### B3. Tarok prior work

- T7 is the only tarok RL work. Tabular, 3 players, no called king, bidding hand-coded, klop and berač excluded. 15,000-game matchups vs a cheating agent and vs rule bots. It beat the rule bots and lost to the PIMC program. It did not reach the hidden-partner problem at all.
- No neural tarok agent exists in the literature I could find. This project would be the first documented one.

---

## (c) Method comparison for tarok

Facts first, with keys. My judgement is marked "(my view)".

### C1. Candidate methods

**Deep Monte Carlo (DouZero style, T29).**
- Mechanism: a Q-network takes (state, action) and regresses the final return by every-visit Monte Carlo. ε-greedy actors (ε = 0.01), γ = 1, no bootstrapping, one net per position. LSTM over the last 15 moves + 6 × 512 MLP. 45 actors on 3 GPUs + 1 learner GPU, 30 days (T29).
- Evidence: beat rule bots in about 0.5 day, the supervised agent in about 2 days, DeltaDou in about 10 days (T29). Derived from the paper's x-axis: ~1.65 × 10^10 timesteps in 30 days ≈ 6,400 steps/s (inferred from Fig. 16–19).
- Bidding: not learned. Trained on random decks "without the bidding network"; a separate supervised bidding net was used only for evaluation (T29). AlphaDou (T34, abstract) later extended DMC to bidding plus card play.
- Fit to tarok: the (state, action) input makes heterogeneous phases easy. Every legal action, whether a bid, a king, a talon choice or a card, is one forward pass. Legal-action masking is implicit. Weakness: MC targets in a 4-player game with a terminal reward are high variance. DouZero needed 10^9–10^10 steps. No full-information trick is possible for the Q-net itself, since the same net acts at play time.

**PPO / IMPALA actor-critic with a full-information critic (MAPPO, asymmetric actor-critic, PerfectDou).**
- Mechanism: the critic sees the full state during training; the actor sees only its observation (T36). PerfectDou calls this "perfect-information distillation": the value network sees the other players' cards, the policy does not (T30). MAPPO adds value normalisation (PopArt), 5–15 epochs, few minibatches, small clip (T35). AlphaStar also fed the opponent's observations to the value function only: 82% vs 22% win rate in the ablation (T37).
- Evidence (T30, Table 4, vs DouZero at 10^9 steps, 100k decks): PerfectDou WP 0.732 / ADP 1.270; ImperfectDou (imperfect critic) 0.717 / 1.180; RewardlessDou (no shaped reward) 0.738 / 0.490; vanilla PPO (neither) 0.509 / −0.307. Reading: the perfect-information critic plus the dense oracle reward together turn PPO from useless into the best system. Removing either one keeps most of the win rate. Removing both kills it. PerfectDou at 10^9 steps roughly tied DouZero at 10^10 (WP 0.524), the basis of the "10× sample efficiency" claim (T30).
- Compute in that paper: 880 CPU cores, 8 GPUs, 2.5 × 10^9 steps (T30). Big 2 PPO without any of this: 4 cores + GPU, 2 days, beat amateurs (T32).
- Fit to tarok: the critic can be given all four hands, the talon and the true partner identity. That removes the hidden-partner variance from the value target. The actor still has to infer the partner. Cost: more hyperparameters (clip, epochs, entropy, GAE λ, value normalisation). Fan and Farina (T34, abstract) report that GAE variance grows in equilibrium self-play with imperfect information and propose a centralised action-value critic instead.

**Suphx-style oracle guiding and global reward prediction (T31).**
- Oracle guiding: train with perfect features visible, then drop them out with a Bernoulli mask whose keep-rate decays from 1 to 0. Plain distillation from an oracle "does not work well" (T31). No isolated effect size is reported.
- Global reward predictor: a GRU predicts the final multi-round game result; per-round reward is the difference of successive predictions. Rationale: a negative round score "may not necessarily mean a poor policy" (T31). It is for multi-round games with ranking rewards.
- Fit to tarok: tarok deals are scored independently, so the global reward predictor has no role. Oracle guiding on the actor is riskier than an oracle critic (Suphx itself lists "an oracle critic" as future work). PerfectDou's result shows the critic route works.

**AlphaZero-style with determinisation (T19, T22, S11, S21).**
- ScrofaZero trained on perfect information and sampled hidden hands at test time. Its raw policy net alone scored ≈ 1 ± 16 points; only with search did it reach +73.9 (T19). Bürli's from-scratch MuZero lost to DMCTS (45–46% of points); only SL-pretrained MuZero won (53.2%) (T22). DeltaDou: 68 CPUs for 2 months (S11).
- Fit: search in the training loop multiplies network calls by the simulation count (S-doc E6 estimate: 34 h to 7 days per DeltaDou-sized run on this PC). Evidence at student scale says the learned value did not beat plain rollouts (S21, T22). Not recommended as the core here.

**Distillation from a search bot (behaviour cloning as warm start, then RL).**
- Evidence that warm starts matter: T18 (SL then PPO: +1.24 IMP/b; no SL: cannot beat WBridge5), T22 (SL + RL beat DMCTS; from scratch lost), T23 (PPO + BC +90 vs rule bot; PPO alone −210), T17 (imitation then policy iteration). T11 seeded Skat self-play with XSkat games after random bootstrapping collapsed to never bidding.
- Evidence that imitation alone is weak: imitation cardplay was 2–4 TP/game below Kermit (T8); imitation bidding alone was −0.12 IMP/deal (T17).
- Fit: here the teacher would be the project's own PIMC/ISMCTS benchmark bot, not human data. Distillation data cost (S-doc E5): about 10^9 search iterations for 1M labelled decisions, 2–14 h. The student has decided the search bot is a benchmark only; a distillation warm start is compatible with that, since the final agent is still trained by self-play.

**NFSP / Deep CFR (T42, S16).**
- NFSP: a DQN best response plus a supervised average policy with anticipatory mixing η = 0.1; converged to exploitability 0.06 in Leduc (T42). Deep CFR: regret networks, external sampling (S16). Both have two-player zero-sum guarantees only. Euchre NFSP "converges faster but no higher ceiling" than DQN (T26). PSRO/DCH converge faster than NFSP early, NFSP lower exploitability late, in Leduc (T40).
- Fit: tarok is 4-player, partly cooperative, with a very deep tree. No guarantee applies. Trick-taking results with NFSP are weak (T26). Not recommended.

### C2. Handling the hidden partner

Facts:
- Input encoding of the king call: the called king's suit is public. Only T2 documents a design that uses it (separate "partner" head for the call itself, and the secret-partner auxiliary target). T1 has no input for it.
- Partner reveal: when the called king falls, team membership is public. T1 sets "teams" bits at that moment (T1). The same is natural here.
- Belief auxiliary head: evidence for (T14 ENN +1.09 IMP/b; T2 gates on 99.5% partner accuracy; Hanabi SAD+AUX cross-play 17.65 vs 2.52 with the auxiliary card-status task, T38). Evidence against (T15: belief loss hurt; T38 SAD: the auxiliary task "drastically hurts performance for 3–5 players").
- Full-information critic: the simplest way to use the partner identity. The critic input includes a 4-way one-hot of the true partner seat and all hands. The actor never sees it (T30, T36).
- Team reward: in Schafkopf and tarok, the deal score already pays declarer and partner the same sign. T1 pays each player their game payment and nothing else. OpenAI Five's "team spirit" interpolates individual and team reward and anneals toward the team reward; low team spirit "reduces gradient variance in early training" (T41).

My view: encode the called king as a 4-way one-hot (suit) plus a 4-way "revealed partner seat" one-hot (all zero until revealed) plus a bit "I hold the called king". Give the critic the true partner seat and all hands. Add a 4-way partner-seat prediction head on the actor trunk with a small loss weight (0.1 or lower) and log its accuracy. Keep it switchable. The accuracy curve is a cheap diagnostic even if the loss weight is set to zero.

### C3. Multi-phase decisions

Facts:
- Separate networks per phase: DouZero (bidding net separate, supervised), Suphx (five models), Wizard (one bid net and one play net per round, T20). PerfectDou omitted bidding (T30).
- One network, one action vector, masking: T1 (43-d: 9 game types + 2 kontra + 32 cards), T28 (bids + suits + cards in one vector), T18 (38 bridge actions with masking). Huang and Ontañón prove that masking logits "corresponds to a valid policy gradient" and that it makes "the gradient corresponding to the logits of the invalid action to zero" (T39). Masking scales better than penalties. "Naive masking" (sample masked, update unmasked) is unstable (T39).
- Phase token: no paper names it. It is the standard practice of putting the phase one-hot in the observation (T17 includes "phase" in its 480 features).

Tarok action inventory (my count, inferred from the rules in 01-rules-and-variants.md): bids ≈ 10 (pass, tri, dve, ena, solo tri, solo dve, solo ena, berač, solo without, open berač, valat, klop; depends on preset), king call 4 (plus a joker option in some presets), talon pick: choose one of 6 cards (tri), one of 3 pairs (dve), one of 2 triples (ena) → at most 6 options if encoded as "pick group index", discard 54 (one card at a time, repeated up to 3 times), announcements ≈ 6 kinds × kontra levels (encode one at a time plus "done"), card play 54. Total about 140 logits. With masking, at most ~16 are legal at any moment.

My view: one network. Observation includes a phase one-hot. One 140-way logits vector with masking. Discards as sequential single-card decisions. For DMC the same holds automatically, since each legal action is one (state, action) pass. Separate heads per phase are an optimisation, not a necessity; T2 uses them, T1 and T28 do not. Start with one head.

### C4. Reward design

Facts:
- Terminal reward only: DouZero (WP ±1 or ADP), Big 2 (sum of opponents' remaining cards), Wizard ([0,1]-normalised round points), ScrofaZero, Tichu (T29, T32, T20, T19, T23).
- Dense shaping: PerfectDou's oracle "distance to win" reward, scale 50, annealed away in the final stage; it raised ADP from 0.49 to 1.27 at equal WP (T30). OpenAI Five decays shaped rewards by 0.6 every 10 minutes of game time (T41). No trick-taking paper documents a per-trick shaping exploit; none documents a per-trick vs terminal ablation either.
- Objective choice matters: DouZero-WP and DouZero-ADP are different agents with different bidding tendencies (T29).

My view: use the signed deal score per seat as the reward, divided by a fixed constant so typical values are in ±1 to ±3. Declarer and partner get the same number by the rules, so the team reward is free. No per-trick shaping in version 1. If learning stalls in the opening tricks, add a small oracle-style shaping term (card points taken by my team minus the opponents', available to the critic) and anneal it to zero as PerfectDou did. Log bid rates and announcement rates per contract from day one.

### C5. Opponent population and self-play variant

Facts:
- Pure latest-policy self-play worked for DouZero, PerfectDou, Big 2, and T15 (bridge: a 20-model zoo and Nash averaging were both worse than latest-model self-play; "mixed strategy can mislead both opponents and partners") (T29, T30, T32, T15). Patwa: checkpoint self-play slightly worse than latest-policy (T33).
- Past-checkpoint mixing worked for OpenAI Five (80% latest, 20% past, quality-weighted) to avoid "strategy collapse" (T41), for T23 (80/20), for T18 (uniform fictitious self-play helps after SL; T18 also shows plain self-play models "struggle against earlier ones").
- Leagues: AlphaStar main agents trained 35% self-play, 50% PFSP against the league, 15% against forgotten players; "naive self-play has high Elo, but is more forgetful" (min win rate vs past versions 46% for SP vs 71% for PFSP+SP) (T37). Balduzzi: self-play "cycles through agents without improving overall agent strength" in cyclic games (T40). Czarnecki: non-transitivity is largest in the middle of the strength range and shrinks near the top (T40).
- Cooperative caveat: Hanabi self-play agents form arbitrary conventions; cross-play with a separately trained copy drops from 23.97 to 2.52 (T38). In a called-partner game the partner is always a copy of the same network, so this is not a problem at play time, but it is a problem if the human is the partner.

My view: latest-policy self-play for all four seats, plus 20% of games with one or more seats filled by a past checkpoint sampled uniformly from the last N snapshots. Keep a fixed rule bot and a fixed random bot in the evaluation set, not in training. No league before the basic loop is proven.

### C6. Rule switches

Facts:
- No card-game paper trains one policy conditioned on a rule-variant vector. XLand conditions on a goal vector and needs dynamic task generation to generalise (T49). Ludii transfer between game variants shows "a high degree of negative transfer" in some pairs and near-perfect transfer in others (T49).
- OpenSpiel's tarok takes game parameters at load time (S31) but no agent conditioned on them is published.

My view: one network with a rule vector (one bit or small one-hot per switch) in the observation, trained on deals sampled from a mixture of presets. Evaluate per preset. If one preset lags, fine-tune a copy on that preset. This is the cheapest path and gives one model for the phone. It is untested in the literature; treat it as a project risk (see (g)).

### C7. Comparison table (my judgement)

| Method | Sample efficiency | Hyperparameter risk | Hidden partner | Multi-phase | Explainability | Evidence at this scale | Verdict |
|---|---|---|---|---|---|---|---|
| DMC (DouZero) | Low (10^9–10^10 in Dou Dizhu) | Low | History only; aux head possible | Natural via (s,a) input | Q per action = expected score | T29; T27 (Cego, results unread) | Strong fallback / control run |
| PPO + full-info critic | High (≈10× DMC in T30) | Medium | Critic sees partner; aux head on actor | One head + masking | Policy probs + value | T30, T32, T33, T2 | Primary |
| Suphx oracle guiding | Unknown alone | High | Oracle features dropped out | Separate models in Suphx | Policy probs | T31 only | Not as core; critic route instead |
| AlphaZero + determinisation | High per game, very high per network call | High | Via sampled deals | Natural | MCTS stats | T19, T22 lost without SL | No |
| BC from search bot, then RL | Warm start cuts early steps | Low | Teacher handles it | Natural | Same as student | T18, T22, T23 positive | Optional stage 2 add-on |
| NFSP / Deep CFR | Low | High | Not addressed | Natural | Average policy | T26 weak; theory 2p only | No |

---

## (d) Throughput and training-time arithmetic

### D1. Published anchors

| Source | Setup | Number | Meaning of a unit |
|---|---|---|---|
| T50 RLCard | Python, 1 process, Dou Dizhu, random agents | 87,271 s per 10^6 games; 1.4 × 10^-3 s/step | one agent action |
| T50 | UNO, 1 process | 4,953 s per 10^6 games | ≈ 200 games/s |
| T29 DouZero | 45 actors on 3 × 1080 Ti + 48 cores | ≈ 1.65 × 10^10 timesteps in 30 days (read from plot axis) | one action; ≈ 6,400 steps/s aggregate (inferred) |
| T61 DouZero port | 5 CPU actors, M1 Pro, PyTorch inference per step | "@ 5116.7 fps", alternating ≈ 3,837 | one (state, action) sample into the learner |
| T32 Big 2 PPO | 4 cores + GPU, Python env | 150M steps in ~2 days ≈ 870 steps/s (inferred) | one action |
| T33 Big 2 PPO | 6-core laptop, Python | 320k episodes in 7–13 h | episodes |
| T53 Sample Factory | 10-core i9-7900X + 1080 Ti, C++ env, CNN policy, full APPO loop | Atari 48,016 FPS = ~12,000 samples/s; VizDoom 59,525 FPS | frames (×4 frameskip) |
| T52 EnvPool | 12-thread i7-8750H laptop, C++ Atari, random actions, no NN | 49,439 FPS async | emulator frames |
| T54 PufferLib | C environments | ">1M steps/second on a single CPU core"; NetHack 39k, Atari 1.2k per core | env steps |
| T54 PufferLib | 6-core i7-10750H + RTX 3070, full PPO, 150k–1M-param MLP-LSTM, trivial C envs | 110k–200k steps/s | env steps through PPO |
| T54 | 24-core i9-14900K + RTX 4090 | 300k–1.2M steps/s | same |
| T51 Pgx | 1 × A100, 1024 vectorised envs, bridge bidding, JAX | ≈ 10^5 steps/s (read from bar chart) | env steps |
| T51 | OpenSpiel via Python, 256 cores | ≈ 10^4 steps/s for chess and Go | env steps |
| T55 pons-dds | Rust DDS port, 16c/32t | 149 ms per deal (1 thread); 13.2 ms (32 threads) | full double-dummy table (inferred meaning) |
| T56 PyO3 | no-op call overhead | Python 19 ns vs PyO3 25 ns per call (issue 3787); removed in 0.22+ | one call |
| T62 OpenNN (snippet) | 28-1024-1024-1 MLP, batch 1024, 8 P-cores i9-12900K | PyTorch 364,220 samples/s inference | one sample |
| T62 rlox (snippet) | tiny DQN, batch 1, CPU | 4 µs (Candle) vs 12 µs (PyTorch) per forward | one forward |

No Rust or C++ trick-taking engine publishes a games-per-second number (T55 is the nearest: a double-dummy solver, which is far heavier than a rules engine).

### D2. Step rate for tarok (inferred)

Decisions per deal: 48 card plays + bidding (4 players, usually 1–2 rounds: ~6) + king call (1) + talon choice (1) + discards (1–3, as single-card decisions: ~2) + announcements (4 players: ~4). Total ≈ 62. I use 60 steps per deal. In self-play all four seats are the agent, so 60 training samples per deal.

Steps per budget: 10^7 steps = 167k deals; 10^8 = 1.67M deals; 10^9 = 16.7M deals.

Reference: Big 2 reached "beats amateurs" at 150M steps ≈ 3M games (T32). DouZero passed rule bots at ≈ 0.5 day ≈ 2.8 × 10^8 steps and the supervised agent at ≈ 10^9 (inferred from 6,400 steps/s). PerfectDou at 10^9 steps ≈ DouZero at 10^10 (T30).

### D3. Pipeline (a): Python-only environment

Anchors: 700 steps/s per process of pure game logic in RLCard Dou Dizhu (T50); 870 steps/s full PPO loop on 4 cores (T32); 3,800–5,100 samples/s with 5 CPU actors and PyTorch inference per step (T61).

Estimate for this PC (inferred): a Python tarok engine with legal-move generation will be similar to RLCard's Dou Dizhu per step. 6–10 actor processes with per-step CPU inference: 2,000 steps/s (pessimistic, Python overhead and small batches) to 5,000 steps/s (optimistic, T61-like).

### D4. Pipeline (b): Rust engine with batched GPU inference

Engine: a bitmask-based Rust tarok engine should do 10^5–10^6 steps/s per core (inferred; PufferLib's C environments do >10^6 for trivial games and 39k for NetHack, T54). On 6 cores the engine alone is ≥ 5 × 10^5 steps/s. It is not the bottleneck.

Inference: actor networks of 0.2–1M parameters. Forward cost ≈ 2 × parameters FLOP per sample ≈ 1.6 MFLOP for a 0.8M-param MLP (inferred). GTX 1660 SUPER FP32 ≈ 5.0 TFLOPS peak (T59). At 30–50% utilisation and batch 4096: ≈ 1–1.4M forwards/s ceiling (inferred). In practice a Python-driven loop adds ~1–2 ms per inference call (kernel launches, transfers), so 100k–300k inferences/s is realistic (inferred). CPU-only inference in Rust (tch-rs, candle, ort) at ~20–50 µs per batch-1 forward of this net ≈ 20k–50k/s per core, 100k–300k/s on 6 cores (inferred from the 4–12 µs rlox snippet for a smaller net, T62). So actors do not need the GPU.

Learner: forward + backward ≈ 3× forward ≈ 4.9 MFLOP/sample; ceiling ≈ 350k–450k samples/s on the GPU (inferred). PPO reuses each sample 3–10 times (T35: 5–15 epochs). PufferLib's full PPO loop on a 6-core laptop with an RTX 3070 (roughly 2–3× this GPU, memory) does 110k–200k steps/s on trivial environments (T54).

Estimate for this PC (inferred): 30,000 steps/s (conservative: Python learner, 4 PPO epochs, synchronisation overhead) to 100,000 steps/s (optimistic: Rust actors with CPU inference, learner only on GPU, few epochs).

### D5. Training-time table

Arithmetic: time = steps / rate. 1 h = 3,600 s; 1 day = 86,400 s.

| Budget | Deals | (a) Python, 2,000 steps/s | (a) Python, 5,000 steps/s | (b) Rust, 30,000 steps/s | (b) Rust, 100,000 steps/s |
|---|---|---|---|---|---|
| Quick 10^7 | 1.7 × 10^5 | 5,000 s = 1.4 h | 2,000 s = 0.6 h | 333 s = 5.6 min | 100 s = 1.7 min |
| Medium 10^8 | 1.7 × 10^6 | 50,000 s = 13.9 h | 20,000 s = 5.6 h | 3,333 s = 0.9 h | 1,000 s = 17 min |
| Large 10^9 | 1.7 × 10^7 | 500,000 s = 5.8 days | 200,000 s = 2.3 days | 33,333 s = 9.3 h | 10,000 s = 2.8 h |

Reading: with pipeline (a), one 10^9 run uses a large part of a week and leaves no room for repeats. With pipeline (b), a 10^9 run is overnight, and 10^8 runs are hourly. The pipeline decision is worth more than the algorithm decision. All rates are inferred and must be measured in the first week.

What each budget buys, by analogy (all inferred): 10^7 ≈ a tenth of Patwa's budget (T33); expect "beats random, loses to rules". 10^8 ≈ Big 2's 150M (T32) and ≈ DouZero's "beats rule bots" mark; expect to beat a rule bot in card play. 10^9 ≈ DouZero's "beats the supervised agent" and PerfectDou's full run; the realistic ceiling of this project.

### D6. GPU and software facts (verified)

- PyTorch support for sm_75: PyTorch 2.8.0 dropped sm_50–sm_60 (Maxwell, Pascal) from the CUDA 12.8/12.9 wheels and kept Turing (T57). The 2.12 release ships CUDA 12.6.3 (arch list includes 7.5), 13.0.x (7.5, 8.0, 8.6, 9.0, 10.0, 12.0) and 13.2.x (T57). The 2026-09-04 notice says "PyTorch 2.14 is the last release to ship any CUDA 12.x wheels" and "CUDA 13.x builds continue and cover Turing (7.5) through Blackwell" (T57). NVIDIA: "CUDA 13.0 supports all NVIDIA architectures from Turing through Grace Blackwell" (T58). Conclusion: the GTX 1660 SUPER is supported by every current wheel line. It is now the oldest supported architecture. Pin the PyTorch version in the project from day one.
- torch.compile on Windows with CUDA: the official Inductor-on-Windows tutorial covers CPU and XPU only (T60). triton-windows provides Triton for Windows, but its table says Turing support "was dropped since Triton 3.3", so Inductor-on-CUDA for this GPU would need PyTorch ≤ 2.6 (T60). Plan for eager mode. The networks are small; eager is fine.
- GTX 1660 SUPER: 1,408 CUDA cores, ≈ 5.03 TFLOPS FP32, 10.05 TFLOPS FP16 via plain shader FP16 (no tensor cores), 336 GB/s, 6 GB GDDR6 (T59). Autocast FP16 gives at most 2× on matmuls. Not needed for these nets.
- VRAM (arithmetic, inferred): a 500 → 3 × 512 → 140 MLP has ≈ 0.85M parameters = 3.4 MB; with Adam and gradients ≈ 14 MB. Activations for batch 4,096 ≈ 4,096 × (500 + 3 × 512) × 4 B ≈ 33 MB, double it for backward ≈ 70 MB. Working set ≈ 100–150 MB plus a 300–500 MB CUDA context. 6 GB is not a constraint. A replay buffer of 10^6 samples × 500 float32 features = 2 GB belongs in host RAM (32 GB), or as uint8 = 0.5 GB.
- PyO3: call overhead is nanoseconds (T56). The real cost is argument conversion and the GIL. `Python::detach` (formerly `allow_threads`) lets Rust threads use all cores while Python waits (T56). Pattern: one Python call per batch of N deals, Rust plays them with rayon, returns numpy arrays.

---

## (e) Known failure modes and fixes

| # | Failure | Evidence | Fix used |
|---|---|---|---|
| 1 | Passive bidding; never declaring | GO-MCTS Skat from random bootstrap: "the learning process quickly collapsed as the players learned to not bid at all" (T11). sheepsheadai: "leaster watchdog against the all-PASS attractor"; bootstrap gate "leaster rate must be below 50%" (T2). Catch Five PPO: early agents "rarely bidded" and found "bidding was dangerous" (T28). Imitation Skat: soloist rate 24–26% vs Kermit 38% (T8). | Seed self-play with games from a non-passive policy (T11 used XSkat). Watchdog on pass rate with forced declaring (T2). Activation change (tanh) and data augmentation (T28). Monitor declare rate per contract from the start. |
| 1b | Over-bidding | SchafkopfRL: "every player always wants to play" (no kontra implemented) (T1). Skat max-value declaration: 96% hand games, −75.59 TP/G (T8). | Implement kontra early so over-bidding is punished (T1). Require a minimum action probability before trusting a value estimate (T8). |
| 2 | Never announcing; risk aversion | No RL paper documents it. Doppelkopf UCT: players "profit from being allowed to make announcements"; a baseline that never announces unless provoked was weak (T44). Four Hundred: conservative bidding (T24). | Same monitors as #1. Give announcements enough exploration (entropy, ε) and a critic that sees the full hands so the value of a correct announcement is learned. |
| 3 | Exploiting or ignoring the partner | SchafkopfRL: "the team concept is not well understood: Agent sometimes plays higher trump than teammate" (T1). JPS: cooperative self-play "often converge[s] to sub-optimal Nash equilibrium" (T16). Hanabi arbitrary conventions: cross-play 2.52 vs self-play 23.97 (T38). Opponent pools "mislead both opponents and partners" (T15). | Partner identity to the critic; partner-prediction auxiliary head (T2, T14). Keep partners as copies of the same policy in training (T15). Convention battery tests (T2: partner trump lead rate). For human partners: other-play / OBL ideas (T38), out of scope here. |
| 4 | Non-transitive strength; cycling | AlphaStar: self-play "may chase cycles in strategy space" (T37). Balduzzi: self-play cycles in the cyclic component of a game (T40). Kita: self-play models "struggle against earlier ones"; FSP checkpoints fixed it (T18). | Past-checkpoint sampling (T41 80/20; T18 uniform FSP; T23 80/20). League with PFSP only if needed (T37). Evaluate against fixed yardsticks, never only against the previous checkpoint. |
| 5 | Forgetting | AlphaStar: min win rate vs past versions 46% (SP) vs 71% (PFSP+SP); "naive self-play has high Elo, but is more forgetful" (T37). OpenAI Five: "strategy collapse" rationale for 20% past opponents (T41). Tichu DQN "effective against its own strategies but failed ... against other, stronger opponents" (T23). | Same as #4. Keep a fixed opponent set in evaluation (random, rule bot, early checkpoint). |
| 6 | Reward hacking via dense shaping | No documented per-trick exploit in trick-taking RL. PerfectDou anneals its shaped reward away; OpenAI Five decays shaping by 0.6 per 10 game minutes (T30, T41). | Terminal reward by default. If shaping is added, anneal to zero. |
| 7 | Entropy collapse / dead actions | Big 2: policy "sampled its top action with 90+% probability" on the first trick; β = 0.05 best, too much hurts (T33). Catch Five: "actions that would always have probability zero after some training time" (dead ReLUs) (T28). Suphx adapts the entropy coefficient to a target (T31). Kujur: entropy regularisation "has no measurable effect on collapse severity" in poker/dice; PSRO "mitigates but does not eliminate" (T43). | Target-entropy controller (T31). Log per-phase entropy. Avoid ReLU dead units in small nets (T28 used tanh). Fixed ε-greedy in DMC sidesteps this (T29). |
| 8 | Vanilla PPO fails outright | PerfectDou ablation: vanilla PPO WP 0.509 / ADP −0.307 vs DouZero; with a perfect critic 0.738 (T30). PPO alone lost to a rule bot in Tichu (T23); PPO ranked below PIMC and imitation in Schafkopf (T1). | Full-information critic (T30). Behaviour-cloning warm start (T23, T18). Large batches and value normalisation (T35). |
| 9 | Search in the loop is too slow or does not help | Wizard: sampled tree search and LSTM "do not surpass the performance of the basic DQN agent" (T20). Doppelkopf UCT+LSTM: +0.04 ± 0.24 (T3). Klaverjas ISMCTS beat the self-play network (S21). | Keep search out of the training loop. Use it as benchmark and optional teacher. |
| 10 | Pipeline bugs that look like learning failure | mafn/doppelkopf-training: "GAE crosses trajectories", "team-label coordinate defects", pipeline marked "Do not use" (T5). | Unit tests on seat-relative encodings, episode boundaries and reward sign per seat before any long run. |

---

## (f) Evaluation protocol

### F1. What the papers do

- Duplicate deals with role or seat swaps: DouZero 10,000 decks, each played twice with sides swapped (T29). Kermit: 800 deals × 6 seatings for 3 players (ABB, BAB, BBA, AAB, ABA, BAA) = 4,800 games (T45). Doppelkopf: 1,000 deals in all 24 seat permutations = 24,000 games; 95% CI half-widths ±0.06–0.13 on mean scores of 1.4–2.2 points (T44). Skat inference: 5,000 matches × 2 role-reversed games, paired t-tests at p = 0.05 (T10). SchafkopfRL: 2 × 1,000 same hands per pairing (T1). sheepsheadai: paired deals, adoption rule "edge + 2·SE ≥ 0" (T2). Tichu: 10,000 rounds, hands shifted one seat (T23). Bridge: duplicate tables, IMP difference as the reward; "a margin of 0.1 IMPs per board is significant" (T15); SE 0.05 IMP/deal at 10,000 deals (T17), 0.19 at 1,000 (T18).
- Variance reduction: AIVAT is unbiased and cut per-hand standard deviation by about 68% in no-limit poker and by "a little less than 99.9%" in Leduc self-play; it "reduce[s] the number of hands needed ... by more than a factor of 10" (T46). MIVAT corrects chance only. Pluribus used AIVAT: 48 ± 25 mbb/game over 10,000 hands, p = 0.028 (T47, secondary).
- Human tests are small and noisy: Doppelkopf 48 games (S7); Big 2 664 games, −0.96 ± 0.38 per game for the humans (T32); bridge 32 deals × 8, +0.97 ± 0.76 (not significant) (T17); Kermit 408 hands vs world-class players (T45).
- Fixed yardsticks: random, rule bot, cheating (full-information) agent as an upper bound (S1, T7, T20: omniscient Wizard play "up to 99%" accuracy).
- Ratings: TrueSkill handles teams and any number of players but uses rank order only (S45). Elo between two sides in DouZero (T29).

### F2. Sample size formula

For a mean score difference with per-deal standard deviation σ and desired 95% half-width d (T48):

n = (1.96 × σ / d)²

For a win rate near 50%: σ = 0.5, so n = (0.98 / d)²; ±2 points needs 2,401 deals, ±1 point needs 9,604.

Per-deal σ for tarok is unknown. Skat gives σ ≈ 70–100 tournament points per game (S-doc inference). Doppelkopf gives σ ≈ 8 score points per game when mean scores are ≈ 2 (inferred from SE ≈ 0.05 at n = 24,000 in T44). Tarok deal scores range roughly ±10 to ±100 with bonuses (rules doc; memory for the spread). Assume σ ≈ 50 points per deal for one seat until measured.

Example with σ = 50: d = 10 → n = (1.96 × 50 / 10)² = 96 deals; d = 5 → 384; d = 2 → 2,401; d = 1 → 9,604.

Duplicate play lowers σ of the paired difference. Doppelkopf's 24 permutations and Kermit's 6 seatings are the two published designs. How much it helps in tarok must be measured.

### F3. Recommended protocol (my view)

- Build the harness before the first training run. Fixed deal files, seeded. Each deal replayed in several seatings.
- Two-bot comparison, A vs B, 4 seats: play each deal in 8 seatings: A in each of the 4 seats with B in the other three, and B in each seat with A in the other three. Score = mean of A's points in the "1 A" games minus mean of B's points in the "1 B" games, per deal. This isolates the single-seat effect. It is my adaptation of Kermit's 6-seating scheme (T45) to 4 players. The 24-permutation scheme (T44) is the exhaustive alternative for 2 A + 2 B.
- Report mean, standard error, deal count, and a paired t-test (T10). Target 2,000 deals for coarse checks and 10,000 for release-grade claims, then recompute from the measured σ.
- Yardsticks, always present: random bot, rule bot (per contract), the search benchmark (PIMC/ISMCTS with fixed iteration budget), a cheating version of the agent (full-information actor) as an upper bound, and the agent's own earlier checkpoints. A search bot at 1,000 iterations on 10,000 deals × 8 seatings costs about 8 × 10^9 iterations ≈ 10–30 h on 6 cores at 10^5–10^6 it/s total (inferred from S-doc E4); use 2,000 deals for routine checks.
- Behaviour monitors: declare rate per contract, pass rate, announcement rate, kontra rate, partner-prediction accuracy, mean score as declarer / partner / defender, legal-move violations (must be zero). Collapse to passive bidding shows here within minutes (T2, T11).
- Variance reduction: subtract the rule bot's score on the same deal and seat as a control variate (inferred, simple MIVAT-like). Full AIVAT needs a value function; the critic can provide one later.
- Humans: 50–200 deals with 3–6 players is what the deadline allows. Report as anecdote with SE, not as a strength claim (T17's 32-deal test was not significant).

---

## (g) Recommendation (my view, not fact)

### G1. Method

Primary: PPO actor-critic with a full-information critic, trained by self-play.
- Actor input: own hand (54 bits), cards played per trick in order (history), current trick, void-suit inference bits, bids so far, called king (4), revealed partner seat (4), "I hold the called king" (1), talon seen (54), contract one-hot, announcements and kontra state, phase one-hot, seat-relative encodings, rule vector.
- Critic input: actor input plus all four hands, the talon, and the true partner seat (PerfectDou pattern, T30; AlphaStar's value ablation, T37).
- One trunk (MLP 3 × 512, or a small transformer over card tokens later), one 140-way masked logits head, one value head, one 4-way partner-prediction auxiliary head with weight ≤ 0.1.
- Reward: signed deal score per seat divided by a constant; terminal only.
- Self-play: latest policy in all seats, 20% of games with past checkpoints (T41, T18, T23).
- Hyperparameters from MAPPO and Big 2: clip 0.1–0.2, 4–10 epochs, large batches, value normalisation, entropy target controller (T35, T32, T31).

Control run: Deep Monte Carlo (T29) on the same Rust pipeline, same observation, Q(s,a) with the action as input, ε = 0.01. The learner is ~100 lines different. If PPO misbehaves, DMC is the known-stable fallback and gives a per-action expected score for the "why this move" display.

Why this and not the others: PerfectDou's ablation is the single most useful experiment for this game. Vanilla PPO failed (WP 0.509); the full-information critic alone rescued it (0.738) (T30). The hidden partner is the main source of value variance in tarok, and the critic can simply be told the answer. DMC cannot use that trick. AlphaZero-style search in the loop costs 10–100× more network calls and lost without supervised pretraining in Jass (T22) and Klaverjas (S21). NFSP and Deep CFR have no multi-player guarantees and weak trick-taking results (T26). Suphx's global reward predictor does not apply to independent deals (T31).

### G2. Training plan in stages

Stage 0 (week 1–2): Rust engine, rules tests, duplicate-deal harness, random and rule bots. Measure engine steps/s and the end-to-end pipeline steps/s. Decision gate: ≥ 20,000 steps/s end-to-end; if not, fix the pipeline before training anything.

Stage 1 (week 2–3): card play only. Fix the contract (e.g. "tri" with a called king), skip bidding and announcements, let the engine deal and call the king by a rule. Train 10^7 then 10^8 steps. Detect that it works: beats random by a wide margin within 10^7; beats the rule bot in card play by 10^8 (Big 2 and DouZero analogies, inferred). Partner-prediction accuracy should rise well above 33% (the chance level for three unknown seats) before the king falls.

Stage 2 (week 3–5): add talon exchange, then bidding, then announcements and kontra, one phase at a time, each with its monitors. Watch for the all-pass attractor (T2, T11). If it appears: force a minimum declare rate by a watchdog (T2) or seed with rule-bot games (T11). Add klop and berač last; they change the reward sign structure.

Stage 3 (week 5–6): build the PIMC/ISMCTS benchmark bot. It is only needed now. Run the 8-seating tournament: agent vs search bot, 2,000–10,000 deals. If the agent loses by a wide margin, use the search bot as a behaviour-cloning teacher for a warm start and continue RL from it (T18, T22, T23 pattern). Rule-vector training starts here, on a mixture of presets.

Stage 4 (week 6–8): one or two 10^9 runs overnight with the best configuration. Export to ONNX or hand-written WASM inference. Phone test. Human sessions.

Stage 5 (week 8–9): writing, evaluation tables, ablations if time allows (critic with vs without partner identity; aux head on vs off; one preset vs mixture).

### G3. Time budget

Engineering is the main cost, not GPU time. With a Rust pipeline a 10^9-step run is 3–9 h (section D5). The plan allows about 5–10 runs of 10^8 and 2–3 runs of 10^9. With a Python-only environment the same plan allows one 10^9 run and little else; I would not start it.

### G4. Top 3 risks and mitigations

1. Bidding collapse or over-bidding. The best-documented failure in this family (T1, T2, T8, T11, T28). Mitigation: phase-by-phase curriculum, declare-rate watchdog, kontra implemented before bidding is trained, seeding from the rule bot if needed, monitors from day one.
2. The agent does not beat the search benchmark. Every small-scale comparison in the literature went to the search bot (T1, T3, T11, T22). Mitigation: this is acceptable for the course deliverable if measured honestly. Fallback: behaviour-cloning warm start from the benchmark, then RL. Keep the critic's full information and the 10^9 budget in reserve for this.
3. Pipeline speed or correctness. Reaching 30k+ steps/s needs a working Rust↔PyTorch loop; mafn's repository shows what a broken GAE/seat pipeline looks like (T5). Mitigation: measure in week 1, unit-test seat-relative encodings and episode boundaries, run DMC as a control (fewer moving parts), keep PyTorch pinned to a version with sm_75 wheels (T57).

Secondary risk: rule-vector conditioning is untested in the literature (C6). Mitigation: evaluate per preset; fine-tune per preset if the mixture lags.

---

## (h) Verified vs inferred

Verified in a source today (full text or page):
- SchafkopfRL ranking, cents/game table, 55-d info vector, teams bits, hyperparameters, 15 days, failure notes, future team head (T1).
- sheepsheadai phases, hours, M1 Max, aux heads incl. secret partner, 99.5% gate, all-PASS watchdog, adoption rule, absence of external baselines (T2).
- Obenaus LSTM/UCT numbers (T3). Požrl method and results (T7). Skat imitation nets, hours, TP/game results, over-declaration failure (T8). Card-location and policy inference gains (T9, T10). GO-MCTS results, bidding collapse, 42 s/move (T11). Edelkamp compute and accuracy (T12).
- Bridge: ENN/PNN design and +1.09 IMP/b (T14); Gong belief-loss table, 4–5 h on one GPU, latest-model self-play best (T15); JPS compute and +0.63 ± 0.22 (T16); Lockhart +0.85 ± 0.05 and human test (T17); Kita +1.24 ± 0.19, SL necessity, FSP finding (T18).
- ScrofaZero, Wizard, Jass (poster and Bürli), Tichu, Four Hundred, Hearts CS230, Euchre, Catch Five numbers (T19–T26, T28).
- DouZero design, compute, bidding handled separately, timesteps axis (T29). PerfectDou ablation table, 880 cores / 8 GPUs, bidding omitted (T30, re-checked by me). Suphx oracle dropout, GRU reward predictor, entropy control, 44 GPUs (T31). Big 2 PPO compute and human result (T32). Patwa budgets and entropy finding (T33).
- MAPPO suggestions (T35). Asymmetric actor-critic definition (T36). AlphaStar league mix, PFSP functions, forgetting ablation, value-function ablation (T37). Hanabi cross-play numbers and auxiliary-task effects (T38). Masking gradient validity (T39). Balduzzi, PSRO, spinning-tops statements (T40). OpenAI Five 80/20 and team spirit (T41). NFSP (T42). Kujur entropy result (T43). Doppelkopf 24 permutations and CI widths (T44). Kermit seatings and series SDs (T45). AIVAT reductions (T46). NIST formula (T48). XLand and Ludii statements (T49).
- RLCard timings (T50). Pgx statements and approximate bar values (T51). EnvPool table (T52). Sample Factory Table A.2 (T53). PufferLib per-core and PPO rates (T54). pons-dds timings (T55). PyO3 overhead and detach semantics (T56). PyTorch sm_75 support statements and CUDA 12.x wheel end at 2.14 (T57, re-checked by me). CUDA 13.0 Turing statement (T58). 1660 SUPER specs (T59). Inductor Windows scope and Triton Turing drop (T60). DouZero port 5,116 fps (T61).

Seen as abstract, README, snippet or secondary only:
- Dockhorn et al. (T4). jlittek and mafn repos (T5). Schafkopf misc repos (T6). Yeh & Lin (T13). Demirdöver, Wei (T25). Oeschger Cego thesis (T27, README only). Fan & Farina; AlphaDou (T34). Pluribus numbers (T47). OpenNN and rlox benchmarks (T62). PufferLib 3.0 blog (T54 snippet). Pgx bridge-bidding exact rate (bar chart read-off).

Inferred (my arithmetic or scaling):
- 60 decisions per deal and all deal counts per budget.
- DouZero ≈ 6,400 steps/s (from a plot axis). Big 2 ≈ 870 steps/s.
- Pipeline rates: Python 2,000–5,000 steps/s; Rust + GPU 30,000–100,000 steps/s. Every entry of the training-time table.
- Rust engine 10^5–10^6 steps/s per core. GPU inference and learner ceilings. CPU inference 20–50 µs per forward.
- VRAM working set ≈ 100–150 MB.
- 140-way action vector count.
- 8-seating tournament design. σ ≈ 50 points per deal and the deal counts derived from it. Doppelkopf σ ≈ 8 from its CI.
- Search-bot tournament hours.
- "What each budget buys" analogies.

From memory, not checked today:
- RTX 3070 is roughly 2–3× a GTX 1660 SUPER.
- Spread of tarok deal scores (±10 to ±100).
- 9 weeks of part-time work allows 5–10 medium runs.

Could not verify:
- Any games-per-second figure for a Rust or C++ trick-taking engine.
- Hardware or hours for T1, T14, T17, T18, T20, T9.
- Strength of sheepsheadai against any external opponent.
- Whether a rule-conditioned policy works for card games (no source exists).
- Whether CUDA graphs work on Windows for this GPU (no run performed).
- Per-deal σ for Slovenian tarok. Must be measured.
- Whether the DouZero and Big 2 step counts transfer to a game with hidden partners and 12 contracts.
