# 02 — Existing tarok software and prior work

Research date: 2026-11-06. All URLs were accessed on 2026-11-06.
Scope: Slovenian tarok first. Königrufen and other tarock games second. Other card games only as feature models.

Legend used in tables:
- **Read** = I fetched and read the page or PDF.
- **Listing** = seen only in a search result or repository listing, page itself not read.
- **Not stated** = the page was read and does not say.

---

## (a) Sources checked

### Apps and sites

| Source | URL | Result |
|---|---|---|
| TAROK.net home | https://www.tarok.net/ | Read |
| TAROK.net program page | https://www.tarok.net/tarok.php | Read |
| TAROK.net FAQ, ordering | https://www.tarok.net/faq.php , https://www.tarok.net/narocilo.php | Read, pages returned only navigation |
| TAROK.net web client | https://igraj.tarok.net | Read, only loading screen |
| tarok.si | https://tarok.si | Failed, TLS certificate mismatch |
| Valat.si home | https://valat.si | Read |
| Valat.si Android listing | https://play.google.com/store/apps/details?id=si.valat.tarok | Failed, page not readable by the fetch tool |
| Palčka site | https://palcka.si | Read, only the title was returned |
| Palčka on F-Droid | https://f-droid.org/en/packages/si.palcka.tarok/ | Read |
| Palčka on Google Play | https://play.google.com/store/apps/details?id=si.palcka.tarok | Failed, page not readable |
| Tarok Counter showcase | https://github.com/gajbrus/tarok-counter-showcase | Read |
| Tarok Counter live app | https://tarokcounter-6a9f8.web.app | Read, only login button visible |
| Briskula.si tarok | https://briskula.si/tarok | Read, only the title was returned |
| Briskula.si on App Store | https://apps.apple.com/us/app/briskula-si/id1507003492 | Read |
| Silicon Tarokist site | http://tarok.bocosoft.com | Read, frameset shell only |
| KubiTarock on App Store | https://apps.apple.com/us/app/kubitarock/id1623335943 | Read |
| KubiTarock on Microsoft Store | https://apps.microsoft.com/detail/9wzdncrdmc7v | Read, title only |
| Tarock – Königrufen on App Store | https://apps.apple.com/us/app/tarock-k%C3%B6nigrufen/id1512961466 | Read |
| Pagat, Slovenian Tarok | https://www.pagat.com/tarot/sltarok.html | Read |
| Board Game Arena news 386 | https://sl.boardgamearena.com/news?id=386 | Read, French Tarot only |

### Open source

| Source | URL | Result |
|---|---|---|
| mytja/Tarok (Palčka) | https://github.com/mytja/tarok | Read |
| mytja/Tarok API record | https://api.github.com/repos/mytja/Tarok | Read |
| StockŠkis folder | https://github.com/mytja/tarok/tree/master/stockskis | Read |
| Palčka asset credits | https://github.com/mytja/Tarok/blob/master/tarok/README.md | Read |
| semanticweights/tarok | https://github.com/semanticweights/tarok | Read |
| OpenSpiel tarok header | https://raw.githubusercontent.com/google-deepmind/open_spiel/master/open_spiel/games/tarok/tarok.h | Read |
| OpenSpiel games list | https://github.com/google-deepmind/open_spiel/blob/master/docs/games.md | Read |
| GitHub topic "tarok" | https://github.com/topics/tarok | Read |
| GitHub topic "tarock" | https://github.com/topics/tarock | Read |
| GitHub API search, tarok | https://api.github.com/search/repositories?q=tarok+in:name,description,topics&per_page=30&sort=stars | Read, first part only |
| GitHub API search, tarock/königrufen | https://api.github.com/search/repositories?q=tarock+OR+k%C3%B6nigrufen+OR+koenigrufen+in:name,description,topics&per_page=30&sort=stars | Read, first part only |
| markrc27/Tarok | https://github.com/markrc27/Tarok | Read |
| Blarc/tarok | https://github.com/Blarc/tarok | Read |
| jureslak/tarok | https://github.com/jureslak/tarok | Read |
| janmerhar/tarock | https://github.com/janmerhar/tarock | Read |
| KlemenKl/tarok | https://github.com/KlemenKl/tarok | Read |
| smatkovi/harbour-tarock | https://github.com/smatkovi/harbour-tarock | Read |
| coder0987/Taroky | https://github.com/coder0987/Taroky | Read |
| sjoerdapp/RL-tarot | https://github.com/sjoerdapp/RL-tarot | Read |
| dds-bridge/dds | https://github.com/dds-bridge/dds | Read |

### Academic

| Source | URL | Result |
|---|---|---|
| Luštrek BSc thesis 2002 (PDF) | https://dis.ijs.si/mitjal/documents/Racunalnisko_igranje_iger_s_kartami-BSc_thesis-02.pdf | Read (summary, rules, algorithms overview, evaluation, conclusion) |
| Luštrek, Gams, Bratko, ICGA 2003 (PDF) | https://dis.ijs.si/mitjal/documents/A_Program_for_Playing_Tarok-ICGA-03.pdf | Read in full |
| ICGA record at IOS Press | https://content.iospress.com/articles/icga-journal/icg26305 | Failed, HTTP 403 |
| Gams and Luštrek, CIT 2003 record | https://hrcak.srce.hr/en/44748 | Read (abstract) |
| Marinčič, Gams, Luštrek, Informatica 2006 (PDF) | https://dis.ijs.si/mitjal/documents/Marincic-Knowledge_vs_Simulation_for_Bidding_in_Tarok-Informatica-06.pdf | Read in full |
| Luštrek research page | https://dis.ijs.si/mitjal/research-other.htm | Read |
| RUL search "tarok" | https://repozitorij.uni-lj.si/Iskanje.php?type=enostavno&lang=eng&niz=tarok&vir=dk | Read |
| RUL search "tarock" | https://repozitorij.uni-lj.si/Iskanje.php?type=enostavno&lang=eng&niz=tarock&vir=dk | Read |
| Požrl MSc 2021 record | https://repozitorij.uni-lj.si/IzpisGradiva.php?id=133070&lang=eng | Read |
| Požrl MSc 2021 PDF | https://repozitorij.uni-lj.si/Dokument.php?id=150850&lang=eng | Read (abstract, related work, problem, learning process, testing, conclusion) |
| Lapajne BSc 2011 record | https://repozitorij.uni-lj.si/IzpisGradiva.php?id=157187&lang=eng | Read |
| Lapajne BSc 2011 PDF | https://repozitorij.uni-lj.si/Dokument.php?id=184655&lang=eng | Read (abstract, prior work, method, results, conclusion) |
| Oblak BSc 2020 record | https://repozitorij.uni-lj.si/IzpisGradiva.php?id=121522&lang=eng | Read (abstract only) |
| "Mobilna aplikacija za tarok" BSc 2025 | https://repozitorij.uni-lj.si/IzpisGradiva.php?id=170528&lang=eng | Read (abstract only) |
| Hoffman Jeglič MSc 2026 | https://repozitorij.uni-lj.si/IzpisGradiva.php?id=189590&lang=eng | Read (abstract only) |
| Čampelj BSc 2022 (FS) | https://repozitorij.uni-lj.si/IzpisGradiva.php?id=140070&lang=eng | Read (abstract only) |
| COBISS record for Požrl | https://plus.cobiss.net/cobiss/si/en/bib/87302915 | Failed, bot check |
| eprints.fri.uni-lj.si | — | Not reached. Lapajne's thesis was found in RUL instead |

### Feature models from other games

| Source | URL | Result |
|---|---|---|
| Chess.com move classification | https://support.chess.com/en/articles/8572705-how-are-moves-classified-what-is-a-blunder-or-brilliant-etc | Read |
| Lichess features | https://lichess.org/features | Read |
| Trickster Cards, for beginners | https://euchre.trickstercards.com/features/for-beginners/ | Read |
| GNU Backgammon manual, Tutor mode | https://www.gnu.org/software/gnubg/manual/html_node/Tutor-mode.html | Failed twice (HTTP 429). Used search-result excerpt of the same page |
| BBO forum threads on the GIB button | https://www.bridgebase.com/forums/topic/83311-gib-notations , https://www.bridgebase.com/forums/topic/84587-gib-makes-dd-error | Search-result excerpts only |
| Skat app listings (Isar Interactive Skat, Skat Coach) | https://apps.apple.com/de/app/skat/id403198590 , https://appgoblin.info/apps/com.isarinteractive.skatcoach.android | Search-result excerpts only |

---

## (b) Existing apps and sites

### Slovenian tarok

| Name | Platform | Play mode | Bots | Teaching / hints | Post-game review | Rule settings | Real-life score sheet | Move adviser | Price / licence |
|---|---|---|---|---|---|---|---|---|---|
| **TAROK.net** ("TAROK.net 2026") | Windows 10/11 and Linux desktop program. Browser client at igraj.tarok.net. | Solo vs computer. Online vs people. Variants: three, three+1, four. League, web top list, "e-Turnir" (everyone gets the same cards, plays vs computers). | Yes. Site calls them "inteligentne računalniške igralke". No levels or method stated. | Not stated. Rules page and a demo video only. | Not stated | Not stated | No | No | Site says free ("zastonj"). Registration and club membership exist, price not shown. Closed source. Copyright 1996–2026. |
| **Valat.si** | Web. Android app `si.valat.tarok` (listing only). | Online vs people. Lobby, rated tables, private tables, "Top 300". | Home page does not mention bots. Požrl (2021) says Oblak (2020) tested his bots "against the bots on the website valat.si". | Not stated | Not stated | Table options: number of rounds, speed, rating limit | No | No. Požrl notes the platform shows a recommended game at bidding. | Daily free credits plus paid membership. Price not shown. Closed source. |
| **Tarock Palčka** | Android (Google Play, F-Droid). Flutter client, Go backend. | Online vs people (WebSocket). Offline vs bots on device. | Yes, "StockŠkis" engine. Its README calls it early alpha and weaker than closed engines. Method not documented. | Not stated | Not stated | "Some modifications" to Slovenian rules, not detailed | No | No | Free. AGPL-3.0-or-later. StockŠkis is GPL-3.0-or-later. v1.0.7 on F-Droid, 2025-02-01. |
| **Tarok Counter / Tarok Točkovnik** (gajbrus) | Web (Firebase Hosting), Android. Flutter + Supabase. | Score sheet. Solo vs bots. Online 4-player with bots filling seats. Public lobby, Elo-style rating. | "Normal": heuristics from a tarok strategy manual. "Hard": perfect-information Monte Carlo over sampled hands. | Not stated | Round-by-round replays. No move grading stated. | Custom scoring. 3–5 players on the score sheet. | **Yes.** Scores every contract, announcements, mond penalty, dealer rotation, statistics, live sharing. | No | Closed source. Showcase repo has no code and no licence. Active, last push 2026-11-04. |
| **Briskula.si** | Web. iOS/iPadOS/macOS app. Pagat also lists Android. | Vs people or vs a bot. Tournament leaderboard. Tarok is one of several games. | Yes, "proti robotu". No details. | Not stated | Not stated | Not stated | No | No | Free, with advertising. Closed source. |
| **Silicon Tarokist / Silicijasti tarokist** | Windows desktop (written in Delphi, per Požrl). | Solo, three-player only. | Yes. Alpha-beta search plus Monte Carlo sampling. See section (d). | No | No | No | No | No | Free download in 2002–2003. Site still answers, download not confirmed. Closed source. |
| **markrc27 "Slovenian Tarok"** | Web at tarok.pages.dev (Cloudflare). | 4-player. Details not stated. | Not stated | Not stated | Not stated | Not stated | No | No | Source public, no licence file. |
| **UL FRI thesis apps** (2025 Expo app; 2026 Flutter client) | Mobile, cross-platform. | Online vs people. 2026 client: computer takes over when a player leaves. | Replacement bot only (2026). | Not stated | Not stated | Not stated | No | No | Thesis projects. Public release not confirmed. |
| **Score-sheet-only apps** (KlemenKl/tarok and several small GitHub projects) | Android APK, web. | None. Record keeping only. | No | No | No | No | **Yes**, manual entry with radelci. | No | KlemenKl/tarok is GPL-3.0. Others have no licence. |

### Königrufen and other tarock games

| Name | Game | Platform | Play mode | Bots | Teaching / hints | Review | Price / licence |
|---|---|---|---|---|---|---|---|
| **KubiTarock** (Paul Kubitscheck) | Königrufen, 4 players | iPhone, iPad, Mac. Windows (Microsoft Store). Android per search listing. | Offline vs "Tarobot". Online groups of up to 4. | Yes. Tarobot sees only the human's visible cards. | **Yes.** Learning mode: ask Tarobot why it proposes a move, it explains its numeric reasoning. Quiz mode asks game-state questions and tracks progress. | Undo and redo of all actions offline. Score export to Excel. | Free download. Offline free. Online: $2.99 per 3 months or $9.99 per year. Closed source. |
| **Tarock – Königrufen** (Bernd Nussbaumer) | Königrufen, Zwanzigerrufen | iOS. Android per search listing. | Solo vs computer. Online tables and tournaments. | Yes, with an "expert level". | Not stated | Save export and import | Free. Closed source. |
| **harbour-tarock** (smatkovi) | Königrufen now. Hungarian tarokk, Tapp, Strohmandeln planned. | Sailfish OS, Android (Qt/QML). | Solo vs computer. LAN planned. | Heuristic. | Design goal: the same evaluation that picks the bot's card also produces hints for the player. Tests folder mentions "lessons". | Not stated | GPL-3.0-or-later. Early stage, 35 commits. |
| **machtarok.com** (coder0987/Taroky) | Taroky (Czech-style) | Web | Online rooms, 1–4 players | Not stated | Not stated | Not stated | Source public, no licence. |

Not found: any Slovenian tarok on Board Game Arena. The BGA news item checked is about French Tarot.

---

## (c) Open-source projects

| Project | URL | Language | Licence | Last activity | What it does | Reusable? |
|---|---|---|---|---|---|---|
| **OpenSpiel, game `tarok`** | https://github.com/google-deepmind/open_spiel (folder `open_spiel/games/tarok`) | C++ with Python API | Apache-2.0 | OpenSpiel is maintained. Tarok status in docs: "implemented but lightly tested". | Slovenian tarok environment, 3–4 players. Phases: deal, bidding, king calling, talon exchange, tricks, scoring. 12 contracts incl. klop and negative contracts. Single round only: radelci are left to the caller. | **Yes, best candidate for bot training.** Plugs into OpenSpiel's MCTS and RL algorithms. Permissive licence. Check announcements and kontra before relying on it (see section g). |
| **semanticweights/tarok** | https://github.com/semanticweights/tarok | C++17, Python bindings | MIT | Last push 2020-09-28. Unmaintained. | Original source of the OpenSpiel tarok game. Has C++ tests and CI. | Superseded by the OpenSpiel copy. Useful as a readable reference and for its tests. |
| **mytja/Tarok (Palčka)** | https://github.com/mytja/tarok | Dart (Flutter), Go | AGPL-3.0-or-later | Last push 2025-01-28. 424 commits, 25 stars. | Full app: online server, Flutter client, offline bots. | Reference for a complete product. AGPL applies to any derived app. |
| **StockŠkis** (inside mytja/Tarok) | https://github.com/mytja/tarok/tree/master/stockskis | Dart, with a CLI. Go wrapper calls the CLI. | GPL-3.0-or-later | Same repo | Engine: shuffling, game modes, bot logic. Files: `cards`, `gamemodes`, `kombinatorika`, `stockskis_base`, `configuration`, `types`. | Partly. Rules plus bots in Dart. No test suite visible. GPL applies to derived work. Self-described early alpha. |
| **Palčka card assets** | https://github.com/mytja/Tarok/blob/master/tarok/README.md | Images | Mixed | — | README credits a c.1910 public-domain tarock deck, plus custom Škis and Pagat art under CC BY-SA 4.0. | Possibly, for card faces. Verify each file's licence before use. |
| **gajbrus/tarok-counter-showcase** | https://github.com/gajbrus/tarok-counter-showcase | — | None | 2026-11-04 | Description only. Claims a Dart rule engine, ~800 tests, PIMC bots. | No. Code is private. |
| **markrc27/Tarok** | https://github.com/markrc27/Tarok | TypeScript (Vite, Cloudflare Workers) | None stated | 2026-10-27, 94 commits | Web 4-player Slovenian tarok with a `tests` folder (Vitest). | Not without permission. No licence means all rights reserved. |
| **janmerhar/tarock** | https://github.com/janmerhar/tarock | Go | None stated | 2022-09-04 | "Slovenian Tarock made with Go". Contents not inspected. | Unknown. No licence. |
| **jureslak/tarok** | https://github.com/jureslak/tarok | Python | MIT | 2014-05-15 | Play against three AI players with a GUI. A PROG2 course project. | Low. Old, small, no tests. Licence is permissive. |
| **Blarc/tarok** | https://github.com/Blarc/tarok | TypeScript (boardgame.io, React) | None shown | 2022-05-09, 10 commits | Web tarok prototype. | No. Too early. |
| **KlemenKl/tarok** | https://github.com/KlemenKl/tarok | Dart (Flutter) | GPL-3.0 | 2025-01-01 | Score sheet app. Author will only maintain it. | Reference for score-sheet UI only. |
| **smatkovi/harbour-tarock** | https://github.com/smatkovi/harbour-tarock | C++, QML | GPL-3.0-or-later | 2026, 35 commits | Königrufen with heuristic bots and hints from the same evaluator. | Design reference for "hint = bot's evaluation". Different game. |
| **dds-bridge/dds** | https://github.com/dds-bridge/dds | C++ with Python, .NET, Java, WASM bindings | Apache-2.0 | v3.1 current | Double-dummy solver for bridge. | Not for tarok rules. A model for a perfect-information solver API. |
| **sjoerdapp/RL-tarot** (fork of GautierDulac/RL-tarot) | https://github.com/sjoerdapp/RL-tarot | Not shown (has an `rlcard` folder) | Not shown | Not shown | RL agent for French tarot. | Different game. Method reference only. |
| Other listings | brodul/tarok, naceee/tarok, KrozekGimVic/tarok (Python); Denys-gcd/Tarok (Java); lukaaj-commits/tarok_v01, nejcludvik/Tarok-Tockovalnik, epodbreg/tarok-scorekeeper-android (score sheets) | — | None shown | Various | Listing only. | Not assessed. |

No public repository was found that contains a trained RL agent for Slovenian tarok.

---

## (d) Academic and thesis work

### Silicon Tarokist line (Jožef Stefan Institute / UL FRI)

**Luštrek, M. (2002). *Računalniško igranje iger s kartami*. Diploma thesis, UL FRI. Mentor: Ivan Bratko.**
- Game: three-player tarok, Tarok Association rules.
- Method: alpha-beta with transposition table, history heuristic, minimal window, and knowledge-based move pruning. Monte Carlo sampling of opponents' hands.
- Settings: sample size 20, search depth 9 at the start. Constants tuned so no decision takes more than about 10 seconds on an Athlon XP 1800+ with 512 MB.
- Search result: all four enhancements cut nodes from 1,598,924 to 8,667 (184 times fewer) and time 86 times.
- Strength: two good players judged it "competent, not top level". Weak points: klop, long-term plans such as forcing out taroks.
- Machine learning was used only in a simple form for bidding and announcements.
- Prior programs named: "Tarok" (1995, no computer bidding) and "Tarok_MC" (three players, author declined to cooperate).

**Luštrek, M., Gams, M., Bratko, I. (2003). A Program for Playing Tarok. *ICGA Journal* 26(3), 190–197.**
- Complexity, three-player: bidding etc. ~10^4, play ~10^28, together ~10^32 on average. Worst case 1.3×10^47. Bridge for comparison: 2.3×10^24 average.
- Sampling: hierarchical clustering picks representative deals from 300 random ones. Graded by an expert: +3.8 per 100 moves over plain Monte Carlo at 30 samples, +1 at 60 samples.
- Search: alpha-beta, depth 9 to 18, iterative deepening, history heuristic, selective move generation (36 situations, 324 expert-set Boolean values), "equivalence" transposition table.
- All four enhancements: 2,470 nodes and 0.068 s per search, versus 1,598,924 nodes and 21.3 s for plain alpha-beta.
- Play uses 60 samples. Response time: "a couple of seconds".
- Strength, score difference per game over 40 games: −2.8 vs a good player, +6.1 vs an average player, +3.5 vs the TAROK.net program.
- Stated limit: "inadequate for truly high-level play". No long-term strategy early in the game.
- Training cost: none. No learning involved.

**Gams, M., Luštrek, M. (2003). Intelligent System for Playing Tarok. *Journal of Computing and Information Technology* 11(3), 209–215.** DOI 10.2498/cit.2003.03.09.
- Same system. Abstract reports the 184-times tree reduction at 9 plies and "reasonably good" play per human judges.

**Marinčič, D., Gams, M., Luštrek, M. (2006). Knowledge vs. Simulation for Bidding in Tarok. *Informatica* 30, 467–476.**
- Game: four-player tarok. Program: Tarok7 (from Marinčič's 2004 UL FRI master's thesis). Card play uses minimax.
- Method A: hand-built Bayesian network. Inputs: bidder's card attributes, others' bids. Output: expected score per bid.
- Method B: the network picks 3 candidate bids, then 10 simulated games per bid.
- Result: the Bayesian network alone was best. Gap to a perfect-information reference player: 2.0 points per game with no simulation, 3.4–3.7 with heavy simulation weight. 30,000 games for the no-simulation run, 1,000 for the others.
- Versus three human experts: same bid in 92%, 82% and 80% of cases. Bids differing by more than one step: 1–2%.
- Complexity, four-player: ~4.7×10^25 average, 2.6×10^47 worst case.
- Stated advantage of the knowledge model: fast and "clearly explicable".
- "The source code ... can be obtained from the authors."

### UL FRI theses

**Lapajne, M. (2011). *Program za igranje taroka z uporabo drevesnega preiskovanja Monte-Carlo*. Diploma thesis, UL FRI. Mentor: Ivan Bratko.** RUL id 157187.
- Game: three-player. Results are for the declarer against two opponents.
- Method: UCT. Hidden cards handled by dealing random hands, building one tree per deal, summing root visit counts.
- Bidding and discarding use a separate heuristic. Announcements are not included.
- Perfect-information tests (100 games per setting): random player ~28 points. 250 simulations per move ~36–38. 10,000 simulations ~38–42. Best C was 0.5 against stronger opponents.
- Three simple playout heuristics helped at 250 simulations (+1.45 points vs the 1000-simulation opponent) and not at 10,000.
- Imperfect information: 1000 simulations × 50 deals scored 38.20 and 36.77 against perfect-information opponents with 250 and 2000 simulations.
- Versus human (author, 25 games, same deals): human 37.46 / 37.12. MCTS 1000×50: 37.91 / 36.01. MCTS 250×50: 36.88 / 35.81.
- Conclusion: level of an occasional player. Time per move and hardware: not found in the pages read.
- Training cost: none.

**Oblak, S. (2020). *Avtomatsko igranje igre Tarok s pomočjo botov*. Diploma thesis, UL FRI. Mentor: Tomaž Dobravec.** RUL id 121522.
- Game: four-player. Three rule-based bots: random, beginner, experienced. Keyword: Selenium.
- Result (abstract): the experienced bot scored almost twice the points of the random bot. Win rates differed little.
- Per Požrl: bots played on valat.si. In some cases the simple bot beat the advanced one. No bidding strategy, since the platform recommends a game.

**Požrl, D. (2021). *Reinforcement learning in Tarock*. Master's thesis, UL FRI. Mentor: Aleksander Sadikov.** RUL id 133070.
- Game: three-player. Klop and beggar are excluded.
- Method: tabular Q-learning, self-play. Six state models (Beginner, Elementary, Point of View, Massive POV, Hierarchical, Probabilistic). Three reward functions, two update rules, 36 agents in total.
- Training: 100 thousand to 2 million self-play games per model. Learning rate 0.1, discount 0.1, exploration 0.2. The author says these counts are too small for some models. Hardware and wall-clock time: not found in the pages read.
- Evaluation: 15,000 games per agent against "theoretical" players (random, and four one-trick look-ahead players that see all cards).
- Finding: about 10,000 games are needed before hand quality evens out and skill can be judged.
- Play complexity estimate: ~10^33, from 1 million random games (4.87 legal moves on average).
- Results: the Probabilistic model was best. It was the only one to reach 35 points against the strongest cheating player. It beat both Oblak bots. It scored worse on average than Silicon Tarokist (1,000 games per model, played through screen capture and simulated clicks).
- Humans: four amateurs, 20 chosen deals each. Against the Probabilistic model they averaged 27.67 to 36.00 points. One of four exceeded 35.
- Stated limits: no model learned long-term strategy. Bidding was too passive.
- No code repository link found in the pages read.

**"Mobilna aplikacija za tarok" (2025). Diploma thesis, UL FRI. Mentor: Aleš Smrdel.** RUL id 170528. Author not shown on the record.
- Expo cross-platform app. Online play against people. Rankings, leaderboards, achievements. No AI content in the abstract.

**Hoffman Jeglič, L. A. (2026). *Izdelava optimiziranega mobilnega odjemalca za igranje taroka s poudarkom na sodobni uporabniški izkušnji*. Master's thesis, UL FRI. Mentor: Slavko Žitnik.** RUL id 189590.
- Flutter client. Four players, each on their own phone. Server and client share one rules package. A computer player replaces anyone who leaves.
- Evaluated by heuristic analysis and a user study. Server handles several hundred concurrent players.

**Čampelj, M. (2022). *Sistem za razpoznavo igralnih kart na osnovi globokega učenja*. Final paper, UL Faculty of Mechanical Engineering. Mentor: Rok Vrabič.** RUL id 140070.
- YOLO model trained on synthetic images. Recognises tarok cards through a camera, sums points, checks bonuses. No accuracy figure in the abstract.
- Relevant to the real-life assistant: camera input instead of typing cards.

**Kregar, O. (2021). *Optimalno mešanje in deljenje kart pri taroku*. Diploma thesis.** RUL id 132738. Listing only. About shuffling and dealing, not play.

### Königrufen and other tarock games

- No academic paper or thesis on a Königrufen-playing program was found in the searches run.
- Related, different games: RL-tarot (French tarot, RLCard-based, GitHub). A Szeged diploma work on a Hungarian tarokk web client appeared in search results (https://diploma.bibl.u-szeged.hu/id/eprint/2312/, listing only).

### Summary of reported strength

| Work | Year | Variant | Method | Reported level |
|---|---|---|---|---|
| Silicon Tarokist | 2002–2003 | 3-player | Alpha-beta + sampled deals | Beats average player (+6.1/game), loses to good player (−2.8/game) |
| Tarok7 bidding | 2006 | 4-player | Bayesian network | Matches expert bids 80–92% |
| Lapajne | 2011 | 3-player | Determinized UCT | Occasional player |
| Oblak | 2020 | 4-player | Rules | ~2× random bot's points |
| Požrl | 2021 | 3-player, no klop/beggar | Tabular Q-learning, self-play | Around average amateur, below Silicon Tarokist |

No work found uses deep RL, neural networks, or search guided by a learned value function for Slovenian tarok.

---

## (e) Feature ideas worth borrowing

| # | Idea | Source |
|---|---|---|
| 1 | "Ask the bot why". The player asks why the bot proposes a move and gets its numeric reasoning. | KubiTarock learning mode (App Store listing) |
| 2 | Quiz mode. Questions about the current game state, scored, with progress tracked over time. | KubiTarock (App Store listing) |
| 3 | Move classification after the game. Grade each move by expected points lost: best, good, inaccuracy, mistake, blunder. Thresholds 0.02 / 0.05 / 0.10 / 0.20 of expected score. | Chess.com support article |
| 4 | Softer grading for beginners. Thresholds vary with player rating. | Chess.com support article |
| 5 | "Learn from your mistakes". Replay only your own errors and try again. | Lichess features page |
| 6 | Puzzles cut from real games. | Lichess features page |
| 7 | Tutor mode. The app stops you before a move worse than a set threshold. The threshold is adjustable. | GNU Backgammon manual (search excerpt) |
| 8 | Per-card outcome overlay. Each legal card shows the result if played, green or red with a number. | BBO "GIB" double-dummy button (forum excerpts) |
| 9 | Perfect-information solver as a separate library with a batch API. | dds-bridge/dds |
| 10 | "Helping hand". The card the bot would play is raised slightly in your hand. You may ignore it. | Trickster Cards |
| 11 | "See everything". Play with all hands face up to watch how others respond. | Trickster Cards |
| 12 | Slow mode. Longer pauses so tricks can be read. | Trickster Cards |
| 13 | Daily challenge deal to measure progress. | Trickster Cards |
| 14 | Duplicate play. Everyone plays the same deal against bots, then compares scores. Removes card luck. | TAROK.net "e-Turnir" |
| 15 | One evaluator for both the bot's move and the player's hint. | harbour-tarock README |
| 16 | Commented practice games written by a strong player. | Isar Interactive Skat "Strategie-Trainer" (search excerpt) |
| 17 | Bidding calculator that explains hand value step by step. | Skat Coach (search excerpt) |
| 18 | Score sheet with undo, statistics and live sharing. | Tarok Counter showcase |
| 19 | Explainable bidding model. Bayesian network from hand attributes to expected score per contract. | Marinčič et al. 2006 |
| 20 | Fixed reference opponents for testing bots: random, and one-trick look-ahead players that see all cards. | Požrl 2021 |
| 21 | Camera recognition of cards as input. | Čampelj 2022 |

---

## (f) Gaps against the three planned uses

**1. Play against bots — served, with limits.**
- TAROK.net, Palčka, Tarok Counter and Briskula.si all offer solo play against bots.
- None publishes bot strength, method details, or a graded difficulty ladder. Tarok Counter states two levels.
- No shipped Slovenian tarok app claims a self-play-trained bot.
- Open ground: a documented range from rule-based to search to learned bots, with measured strength.

**2. Learn by playing — not served for Slovenian tarok.**
- No Slovenian tarok app found states hints, move explanations, tutor mode, quizzes, or graded post-game review.
- Tarok Counter has replays without grading. TAROK.net has a rules page and a demo video.
- The nearest product is KubiTarock, for Königrufen. It explains bot moves and has a quiz mode.
- Open ground: explanations, mistake review, and lessons for Slovenian rules, in Slovenian.

**3. Real-life assistant — not served.**
- Existing real-life tools only keep score (Tarok Counter, KlemenKl/tarok, small GitHub projects).
- No tool found takes "my hand plus cards played" and returns a recommended card with a reason.
- Partial building blocks exist: Silicon Tarokist and Lapajne show sampled-deal search works for one hidden-information decision. Čampelj shows card recognition by camera.
- Open ground: the whole feature. A score sheet could share the same screen.

**Other gaps seen**
- Bot explanations: only the Tarok7 bidding paper names explicability as a goal. No play-phase explanation work was found for tarok.
- Four-player play with partner inference is thin in the literature. Most results are three-player.
- Klop and beggar are weak or excluded in every study read.

---

## (g) Verified vs inferred

### Verified (read on the page or in the PDF)

- TAROK.net: platforms, three variants, computer players, competitions, "free" claim, copyright 1996–2026.
- Valat.si: lobby options, credits, paid membership, Top 300, Google/Facebook login.
- Palčka: licences (AGPL-3.0 app, GPL-3.0 StockŠkis), F-Droid version and date, GitHub push date 2025-01-28, StockŠkis "early alpha" self-description.
- Tarok Counter: every feature listed comes from its own showcase README. The code is private, so none of it was checked.
- Briskula.si: games, bot, price, from the App Store listing.
- KubiTarock: learning mode, quiz mode, prices, from the App Store listing.
- OpenSpiel tarok: Apache-2.0, phases, 12 contracts, single round, radelci left to caller, "lightly tested", 3–4 players.
- semanticweights/tarok: MIT, last push 2020-09-28, merged into OpenSpiel.
- All numbers in section (d) for Luštrek 2002, ICGA 2003, Marinčič 2006, Lapajne 2011 and Požrl 2021 were read from the PDFs.
- Oblak 2020, the 2025 Expo thesis, Hoffman Jeglič 2026 and Čampelj 2022: abstracts only.
- Chess.com thresholds, Lichess feature list, Trickster features, dds licence.

### Inferred or weakly sourced

- **OpenSpiel tarok lacks announcements.** A search summary of the OpenSpiel docs says so. The header file read does not confirm or deny it. Check `tarok.cc` before relying on it.
- **Valat.si has bots.** Source is Požrl's description of Oblak's thesis, not the site.
- **Valat.si Android app, rating 2.9.** From a low-confidence aggregator. The Play Store page could not be read.
- **Silicon Tarokist is still downloadable.** The site answers with a frameset. The download was not checked.
- **TAROK.net bot strength.** Only data point is the 2003 ICGA result (Silicon Tarokist +3.5 points per game). The current version may differ.
- **StockŠkis method.** Not documented. File names suggest rules plus heuristic or combinatorial bots.
- **Palčka card art is public domain.** The README says the base deck is a c.1910 public-domain deck. Not independently checked.
- **KubiTarock on Android, Nussbaumer app on Android.** From search listings.
- **Licence consequences.** Reusing AGPL or GPL code obliges the derived app to use the same licence. Repositories without a licence file grant no reuse rights. This is general licence knowledge, not from a fetched page.
- **Lichess is open source.** From memory. The features page read does not say.
- **GNU Backgammon tutor mode and BBO GIB button.** From search-result excerpts. The pages themselves were rate-limited or not opened.
- **No deep-RL tarok work exists.** This is absence of evidence across the searches run. Not a proof.
- **No Königrufen academic work exists.** Same caveat. German-language repositories were searched only once.
- **Several small tarok repositories were pushed in October–November 2026.** This may indicate other student projects on the same topic. Not confirmed.
- **Tarok Counter overlaps strongly with the planned app** (score sheet, rule engine, PIMC bots, online play). If it belongs to the student, it is the starting point. If not, it is the closest competitor. Ownership was not determined.

### Not checked

- eprints.fri.uni-lj.si directly.
- Marinčič's 2004 master's thesis text.
- Full text of the Oblak, Expo-app and Hoffman Jeglič theses.
- Google Play listings (not readable by the fetch tool).
- The contents of `tarok.cc` in OpenSpiel and of StockŠkis source files.
- Hungarian tarokk and French tarot software beyond the listings named above.
