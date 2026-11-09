# 01 - Tarok rules and variation points

Research date: 2026-11-06. Planning only. No code.

Main finding: the only official ruleset is for 3 players. Tarok zveza Slovenije (TARZS) publishes "Pravila tekmovalnega taroka" (16.01.2010). It does not cover the 4-player game. For 4 players there is no official text. The best written 4-player description is pagat.com. The second is the Tarok.net rulebook 1.08.

---

## (a) Sources checked

All accessed 2026-11-06.

| Key | Source | URL | Date / version | Read how |
|---|---|---|---|---|
| TZS10 | TARZS, Pravila tekmovalnega taroka | https://tarokzveza.si/wp-content/uploads/2020/11/PRAVILA-TEKMOVALNEGA-TAROKA-16.01.2010.doc | 16.01.2010, adopted by Škis odbor | Full text extracted from the .doc |
| TZS-IDX | TARZS, Akti in pravilniki (index) | https://tarokzveza.si/zveza/akti-in-pravilniki-tarok-zveze-slovenije/ | page lists docs 2008-2026 | Fetched |
| TZS-P | TARZS, Pojasnila v zvezi s pravili, 1.0 | https://tarokzveza.si/zveza/akti-in-pravilniki-tarok-zveze-slovenije/pojasnila-v-zvezi-s-pravili-tekmovalnega-taroka-1-0/ | 3.2.2010 | Fetched |
| TZS17 | TARZS, Sprememba točkovanja | https://tarokzveza.si/novice/sprememba-tockovanja-na-tarok-turnirjih/ | 9.1.2017 | Fetched |
| TZS18 | TARZS, Sprememba pravil (solo brez talona) | https://tarokzveza.si/novice/sprememba-tekmovalnih-pravil-v-letu-2018/ | adopted 28.1.2017, valid from 1.1.2018 | Fetched |
| TZS22 | TARZS, Dopolnitev pravil (mešanje, karte) | https://tarokzveza.si/novice/dopolnitev-pravil-tekmovalnega-taroka-19-11-2022/ | 19.11.2022 | Fetched |
| TZS25 | TARZS, Zapisnik 2. seje Sodniške komisije | https://tarokzveza.si/wp-content/uploads/2025/12/2-seja-SK-9.12.2025-1.pdf | 9.12.2025 | Full text extracted from PDF |
| TZS-ORG | TARZS, Pravilnik o organizaciji in izvedbi tekmovanj | https://tarokzveza.si/zveza/akti-in-pravilniki-tarok-zveze-slovenije/pravilnik-o-organizaciji-in-izvedbi-tekmovanj/ | 12.5.2018 | Fetched (summary) |
| PAG | pagat.com, Slovenian Tarok (John McLeod, from Zoran Obradovic) | https://www.pagat.com/tarot/sltarok.html | last updated 31.10.2020 | Fetched three times with different questions |
| TN | Tarok.net, Pravila taroka (Miroslav Cigan) | https://www.tarok.net/pravila108.pdf (index: https://www.tarok.net/pravila.php) | ver. 1.08 (01), 11.8.2016 | Full text extracted from PDF |
| SLW | Slovenian Wikipedia, Tarok | https://sl.wikipedia.org/wiki/Tarok | live page | Fetched twice |
| ST | slovenski-tarok.si, Pravila (Lada Zei) | https://www.slovenski-tarok.si/pravila | no date | Fetched twice. Table came through garbled |
| LUS | M. Luštrek, diploma thesis, Računalniško igranje iger s kartami, UL FRI | https://dis.ijs.si/mitjal/documents/Racunalnisko_igranje_iger_s_kartami-BSc_thesis-02.pdf | 5.7.2002 | Section 4.1 extracted from PDF. States it follows TARZS rules of that time |
| VAL | valat.si, Pravila slovenskega taroka | https://valat.si/pravila | - | Page returns 404. Only a search-engine snippet was seen |
| HRW | Croatian Wikipedia, Tarok | https://hr.wikipedia.org/wiki/Tarok | live page | Fetched |
| KOE | pagat.com, Königrufen | https://www.pagat.com/tarot/koenig.html | - | Fetched |
| ENW-K | English Wikipedia, Königrufen | https://en.wikipedia.org/wiki/K%C3%B6nigrufen | live page | Fetched (both halves) |
| HUN | pagat.com, Hungarian Tarokk | https://www.pagat.com/tarot/xx-hivas.html | - | Fetched |
| TAR | pagat.com, Taroky | https://www.pagat.com/tarot/taroky.html | - | Fetched |
| PAG-IDX | pagat.com, Tarot games index | https://www.pagat.com/tarot/ | - | Fetched |
| PAG-HR | pagat.com, Card games in Croatia | https://www.pagat.com/national/croatia.html | - | Fetched |

Sources that gave nothing:

- English Wikipedia has no article on Slovenian tarok. `en.wikipedia.org/wiki/Tarok` is a disambiguation page.
- `valat.si/pravila` returned 404 in both the fetch tool and the browser. `web.archive.org` is blocked for the fetch tool.
- `briskula.si/tarok` and `tarok.net/faq.php` returned empty pages.
- TZS-IDX has a heading for amendments adopted 21.10.2023 in Ptuj. No text of those amendments was found.

---

## (b) Baseline rules

Two baselines are given. 3 players: TZS10 (official). 4 players: PAG main text. Differences between sources are in section (c).

### B1. Deck and card values (all sources agree)

- 54 cards. 22 taroks: I (pagat) to XXI (mond), plus škis (highest). 32 suit cards, 8 per suit.
- Suit order, high to low. Black (pik, križ): kralj, dama, kaval, fant, 10, 9, 8, 7. Red (srce, karo): kralj, dama, kaval, fant, 1, 2, 3, 4.
- Trula = škis + mond + pagat.
- Values: kralj 5, škis 5, mond 5, pagat 5, dama 4, kaval 3, fant 2, every other card 1.
- Counting: take cards in threes, add values, subtract 2 per group. A leftover of one or two cards: subtract 1. Same as subtracting 2/3 per card (TN).
- Total is 70. Declarer needs more than half.

### B2. Three players (TZS10, official)

Deal

- Dealer shuffles. Player on dealer's right cuts, at least once, at most three times.
- Deal is clockwise. First 6 cards to the talon. Then 8 cards to each player, twice. 16 cards each.
- A player with no tarok shows the hand. Redeal.
- Wrong deal found before the first card is played: dealer writes -20, redeal.

Bidding

- Player on dealer's left starts. His opening bid "tri" is compulsory. Bidding goes clockwise.
- Each next player says "dalje" or bids a higher game.
- A bid that the others did not exceed can be raised only by the player with priority.
- If nobody exceeds "tri", the first player chooses: play tri, play any higher game, or klop.

Games and values

| Game | Value | Talon |
|---|---|---|
| klop | see below | talon cards go to the first six tricks |
| tri | 10 + difference | take 3 |
| dve | 20 + difference | take 2 |
| ena | 30 + difference | take 1 |
| solo brez talona | 80 | none |
| berač | 125 | none, talon stays closed |
| valat (as a game) | 500 | only allowed to outbid berač |

- Declarer always plays alone against two.
- Game is won with at least 36 points.
- Result is written without rounding.

Talon and discards

- Declarer exchanges with the talon, then makes announcements, then says "ležim".
- Kings and trula may not be discarded. Other taroks may be discarded but must be shown first.

Announcements

| Announcement | Announced | Silent |
|---|---|---|
| kralji | 20 | 10 |
| trula | 20 | 10 |
| pagat (ultimo) | 50 | 25 |
| valat | 500 | 250 |

- After "ležim" the two opponents speak in turn. They may announce too, or give special announcements, or say "dalje".
- Silent values do not apply in valat, klop and berač.
- A failed silent pagat counts as lost (25) for the other side.
- Silent valat: 250 replaces the normal scoring. It is increased only by special announcements and radelc.

Special announcements (kontra)

- kontra x2, re x4, sub x8, mort x16.
- Given separately for the game and for each announcement, in order of level.
- Opponents say kontra and sub. Declarer says re and mort.
- If a kontra fails, the opponent who did not say it gets a bonus equal to the difference caused by the kontra.

Play

- First card is led by the player who started the bidding. Exception: berač, where declarer leads.
- Follow suit. If void, play a tarok. (Stated in TN and LUS. TZS10 only defines the breach as renons.)
- Trula in one trick, any order: the player of the pagat wins the trick. The mond lost this way is not a mondfang.
- A player may look only at his own last trick.

Klop

- Played when nobody takes a game above the compulsory "tri". The player with priority announces and leads.
- Each player's card points are written as minus if they are 34 or fewer.
- A player with 35 or more writes -70.
- Kontra is allowed, first level only, by one player. Kontra doubles the points. If the kontra player himself takes the most points (not over 34), his result is doubled and the others write what they took.
- Radelc is not applied to klop results.
- If mond and škis are in the trick, the third player must play the pagat if he holds it. Otherwise the pagat must be played as the forced last tarok.

Berač

- Declarer wins if he takes no trick. Declarer leads. Talon stays closed to the end.

Radelci

- All players get a radelc after klop, berač and valat. From 1.1.2018 also after solo brez talona (TZS18).
- A radelc doubles the result of the game, announcements and special announcements.
- It is erased after the first game the player plays after getting it.

Mondfang

- Škis takes the mond: the mond's holder writes -21.

Renons

- Covers wrong discard, wrong card, giving the game away by talk or behaviour, unfair play.
- Penalty for the offender: 35 (full difference) + game value + announcements + special announcements. Declarer's radelc counts.
- Declarer may decline the renons if it would hurt his total.

Session

- Organiser announces rounds and games per round (TZS10: usually 4 rounds of 21 games. TZS-ORG 2018: 5 rounds of 18 games for the national championship).
- Max 4 minutes 30 seconds per game.
- Table ranking by score difference. Round points: 3 / 2 / 1 since 2017 (TZS17). Before that 2 / 1 / 0.5.

### B3. Four players (PAG main text, no official source)

Deal

- Anticlockwise. Player opposite the dealer cuts. Packets of six. The first packet is the talon. 12 cards each.
- A player with no taroks shows the hand. Same dealer redeals. A compulsory klop follows.

Bidding

- Forehand (dealer's right) has top priority. Dealer has lowest.
- A lower-priority player must bid higher. A higher-priority player may hold (equal) the bid.
- A player who passed is out. Bidding ends after three passes in a row.
- Klop and tri are available only to forehand, and only if the other three pass.
- Declarer may play the last bid or any higher contract.

Contracts and values

| Contract | Value | Notes |
|---|---|---|
| klop | minus points taken, or 70 | all pass |
| tri | 10 + difference | call a king, take 3 |
| dve | 20 + difference | call a king, take 2 |
| ena | 30 + difference | call a king, take 1 |
| solo tri | 40 + difference | alone, take 3 |
| solo dve | 50 + difference | alone, take 2 |
| solo ena | 60 + difference | alone, take 1 |
| berač | 70 | alone, no tricks |
| solo brez | 80 | alone, no talon |
| odprti berač | 90 | hand exposed after the first trick |
| barvni valat | 125 | only by raising solo tri/dve/ena after the talon exchange |
| valat | 500 | alone, no talon, all tricks |

- Difference = card points minus 35, rounded to the nearest 5.
- From berač upward: flat value. No difference, no bonuses.

Calling a king and talon

- Declarer names a suit before the talon is opened. The holder of that king is the secret partner.
- Declarer may call his own king and play alone.
- King in the talon: declarer plays alone. If he takes the part with the king and wins a trick with it, he also gets the rest of the talon.
- Kings and trula may never be discarded. Other taroks may be discarded face up.
- Talon cards not taken go to the opponents.

Announcements and bonuses (silent / announced)

| Bonus | Silent | Announced |
|---|---|---|
| trula | 10 | 20 |
| kralji | 10 | 20 |
| kralj ultimo | 10 | 20 |
| pagat ultimo | 25 | 50 |
| valat | 250 | 500 |

- Only the pagat holder may announce pagat ultimo. Only the called-king holder may announce kralj ultimo.
- Valat cancels all other scores of the hand.
- Kontra, rekontra, subkontra, mordkontra. Up to x16. Game and each bonus are doubled separately. No kontra against your own partner.

Play

- Forehand leads up to solo ena. Declarer leads from berač upward, and in barvni valat.
- Follow suit. If void, play a tarok. No duty to overtake in normal contracts.
- Trula in one trick: pagat wins. Exception: barvni valat with a suit card led.
- Klop, berač, odprti berač: must beat the highest card on the table if able. Pagat may be played only when it is the only legal card.
- Klop: the first six tricks each get one talon card.
- Barvni valat: taroks do not beat the led suit.

Scoring

- Running total on paper. Declarer and partner write the same score. Opponents write nothing.
- Individual entries: klop, mond penalty, renons, misdeal.
- Klop: more than 35 points: -70. No tricks: +70. If anyone wins or loses this way, only they score. Otherwise each writes minus his points, rounded to 5.
- Captured mond: -20 for the holder. Applies in normal contracts and solo brez. Not multiplied by kontra or radelc. Also applies when the mond is left in the talon and goes to the opponents.
- Radelci: every player gets one after klop, after berač or higher, and after any valat. A declarer with a radelc has the score doubled. On a win one radelc is cancelled. On a loss none is cancelled. Each unused radelc at the end costs 100.
- Renons: offender pays the full game value with difference, bonuses, kontras and radelc.
- End of session: škis round. The holder of the škis in that deal deals the last hand.

Five players (PAG): dealer deals to the other four and sits out. He scores nothing. He gets a radelc when everyone does.

---

## (c) Variation points

"Official" means TZS10 with amendments. It is a 3-player ruleset. Where the point exists only in the 4-player game, the column says "no official rule" and names the PAG main-text option.

### Format and deal

| # | Variation point | Options seen | Official (TARZS, 3p) |
|---|---|---|---|
| 1 | Number of players | 3 (TZS10). 4 with called king (PAG, TN). 3+1: four at the table, dealer rests, but plays in klop (TN competitive). 5: dealer rests (PAG). 2: "Napoleon" (TN, ST) | 3 |
| 2 | Direction of deal and play | Clockwise (TZS10, TN competitive). Anticlockwise (PAG, ST, TN general part) | Clockwise |
| 3 | Deal order and cut | Talon first, then 2 x 8 or 2 x 6 (TZS10, TN). Talon as one of the packets (PAG, ST). Knock instead of cut allowed (PAG). Knock forbidden (TN) | Talon first. Cut 1 to 3 times |
| 4 | Hand without taroks | Redeal (TZS10, TN). Redeal plus compulsory klop (PAG). Also annul with one tarok or two taroks below X (PAG 3p variation) | Redeal |
| 5 | Misdeal penalty | -20 (TZS10). -20, then -40, -80 doubling (PAG). Warning, then -20, -40, -60 (TN) | -20 to the dealer |
| 6 | Length and end of session | Fixed number of games (TZS10: usually 21 per round. TZS-ORG: 18. TN: 24). Škis round (PAG, TN). Time limit per round (TZS10) | Fixed games plus time limit |
| 7 | Time limit per game | 4 min 30 s (TZS10). None (others) | 4 min 30 s |

### Bidding and contracts

| # | Variation point | Options seen | Official (TARZS, 3p) |
|---|---|---|---|
| 8 | Compulsory "tri" for forehand | Forehand must open with tri and may fall back to klop (TZS10, LUS). By agreement only. Without it, a forehand who bids tri loses the right to klop (TN). Forehand waits. Tri and klop only if all others pass (PAG 4p) | Compulsory tri |
| 9 | Raising after the auction | Winner may play the bid or any higher contract (PAG, TZS10 for forehand). Winner must play exactly the bid (PAG variation) | Raise allowed |
| 10 | Klop on or off | Klop when all pass (all rule texts). No klop and no radelci (PAG money variation) | On |
| 11 | Compulsory klop | None (TZS10, TN). When a cumulative score is exactly 0, or after a no-tarok deal (PAG). Also at 500, 1000, each multiple of 500, or after a misdeal (PAG variations) | None |
| 12 | Solo tri / dve / ena | In the 4p list at 40 / 50 / 60 (PAG, TN, SLW). Not in 3p (PAG, TZS10). ST gives other values. Its table did not extract cleanly | Not applicable in 3p |
| 13 | Pikolo (exactly one trick) | Not played (PAG default, TZS10, TN). 70, between solo ena and berač (PAG variation). 35, "unofficial", between ena and solo tri (SLW, HRW). 60 (ST) | Not in the list |
| 14 | Odprti berač | 90 (PAG 4p, SLW). 80 (ST). Optional in 3p (PAG). Absent (TZS10, TN) | Not in the list |
| 15 | Berač value and rank | 70, below solo brez (PAG, TN, SLW, ST, LUS). 125, above solo brez, only valat outbids it (TZS10) | 125, highest game except valat |
| 16 | Solo brez value and scoring | 80 flat, no difference, no bonuses (PAG). 80 + difference (TN). 80 (TZS10, difference not excluded). 50 (ST, LUS 2002) | 80 |
| 17 | Tri / dve / ena values in 3p | 10 / 20 / 30 (TZS10, PAG, LUS). 10 / 30 / 50 (TN) | 10 / 20 / 30 |
| 18 | Barvni valat: availability and value | Only by raising solo tri/dve/ena after the talon, 125 (PAG). A bid of its own, 125 (SLW, HRW). SLW also says 250 elsewhere. Value by base contract: 125 / 150 / 175 / 250 for solo tri / dva / ena / brez (TN). Optional in 3p (PAG). Absent (TZS10) | Not in the list |
| 19 | Barvni valat with a partner | Not allowed (PAG default). Allowed from tri/dve/ena (PAG variation). TN: 100 / 110 / 120, partner writes half | No official rule |
| 20 | Barvni valat: suits before taroks | Any order (PAG default). Declarer must play all suit cards before leading taroks (PAG variation) | No official rule |
| 21 | Valat as a contract | 500, alone, no talon (PAG). Only to outbid berač (TZS10). Graded by base contract: 250 / 300 / 350 / 500 solo, 200 / 220 / 240 with partner (TN) | Only above berač, 500 |
| 22 | Valat as a bonus | Announced 500, silent 250 (PAG, TZS10, SLW, ST). Silent valat +50 added to the game (TN). 250 only (HRW). Silent valat raised by kontra and radelc only (TZS-P) | 500 / 250 |

### Called king and talon

| # | Variation point | Options seen | Official (TARZS, 3p) |
|---|---|---|---|
| 23 | Called king in the talon | Declarer plays alone. Gets the rest of the talon if the king wins a trick (PAG, TN). If he takes the king but it wins no trick: +5 points instead (TN). Declarer must take the part with the king (ST) | No official rule (4p only). PAG: first option |
| 24 | Calling with four kings or three kings | Must call own king (PAG default). May call a trula card with four kings. May call "the fourth king" without naming the suit (PAG variations) | No official rule |
| 25 | Discarding taroks | Allowed if shown (TZS10, PAG). Only when the hand forces it, then shown (TN, ST, HRW). Kings and trula never (all) | Allowed if shown |
| 26 | Mond left in the talon | Counts as lost mond (PAG, TN, LUS). No penalty in solo brez (PAG). Not mentioned (TZS10) | Not stated |

### Play

| # | Variation point | Options seen | Official (TARZS, 3p) |
|---|---|---|---|
| 27 | First lead | Forehand in normal contracts, declarer in berač and above (PAG, TZS10, TN). Declarer always (LUS program). Player left of declarer in berač (ST) | Forehand. Declarer in berač |
| 28 | Pagat in klop and berač | Only when it is the only legal card (PAG). Must be thrown as the last tarok, never led (TN). Must be played on mond + škis, otherwise forced last tarok (TZS10, klop) | TZS10 wording |
| 29 | Leading the mond in klop | No limit (PAG default, TZS10). Mond may not be led before the škis is out (PAG variation) | No limit |
| 30 | Trula in one trick | Pagat wins (all). Mond lost this way is not a mondfang (TZS10). Mond penalty still applies (PAG). Not valid in barvni valat when a suit is led (PAG) | Pagat wins, no mond penalty |
| 31 | Berač with škis | Not special (default). Declarer leads škis to force mond and pagat into one trick. Fails if they do not both fall (PAG variation) | Not stated |
| 32 | "Vogel frei" | Not played (default). Pagat as the only tarok may be played to any trick and wins it (PAG 3p variation) | Not in the rules |
| 33 | Looking at earlier tricks | Own last trick only (TZS10). Not allowed (TN). Not stated (PAG) | Own last trick only |

### Announcements and kontra

| # | Variation point | Options seen | Official (TARZS, 3p) |
|---|---|---|---|
| 34 | Kralj ultimo | 10 / 20 (PAG, 4p). 10 silent (SLW). 10 / 20 if the partner brings it, 25 / 50 if declarer brings it himself (TN). Not in 3p (PAG, TZS10) | Not applicable in 3p |
| 35 | Who may announce | Only the holder for pagat and king ultimo (PAG). Both sides may announce (PAG, TZS10, TN) | Both sides, after "ležim" |
| 36 | Lost silent pagat ultimo | Whole side pays 25 (PAG default, TZS10 B1.21, SLW). Only the holder pays (PAG variation). Only when lost (PAG variation) | Counted for the side |
| 37 | Bonuses in high contracts | None from berač upward (PAG, TN). Silent bonuses excluded only in valat, klop, berač (TZS-P) | Excluded in valat, klop, berač |
| 38 | Kontra depth | x16: kontra, re, sub, mort (TZS10, PAG, SLW, LUS). x4: kontra and rekontra only (TN). Extra "hirškontra" level between sub and mord (PAG variation) | 4 levels, x16 |
| 39 | Kontra on klop | Not allowed (PAG default, SLW). One level, one player (TZS10). Loser pays 140 instead of 70. If nobody is over 35, the kontra player pays 140 (PAG variation). Kontra and rekontra, with a 17-point limit for the kontra player (TN) | One level |
| 40 | Kontra accounting between defenders | Non-kontra defender gets a bonus when the kontra fails (TZS10 B1.22, LUS). Only the kontra player carries the kontra loss (TN, 2015 change). Team shares (PAG 4p) | TZS10 B1.22 |
| 41 | Kontra on silent bonuses | Silent results are never under kontra (TN). Silent valat is raised by spoken special announcements (TZS-P) | Per TZS-P |

### Scoring

| # | Variation point | Options seen | Official (TARZS, 3p) |
|---|---|---|---|
| 42 | Winning threshold | At least 36 (TZS10, PAG, HRW). More than 35, i.e. 35 + two blanks (TN). 35 + one card (PAG variation). SLW: exactly 35 is a loss with no difference | 36 |
| 43 | Rounding of the difference | Nearest 5 (PAG 4p default). Exact (TZS10, PAG 3p and high-stakes, VAL snippet). Exact by default, 5 by agreement (TN) | Exact, no rounding |
| 44 | Partner's share | Same as declarer (PAG, SLW). Half of declarer's score (TN). Per-player payments (PAG money game) | Not applicable in 3p |
| 45 | Paper or money scoring | Only declarer's side writes (PAG, TZS10). Each loser pays a winner. A soloist collects from all three (PAG money game). Settle against the table average (PAG variation) | Declarer writes |
| 46 | Klop scoring | 35 or more: -70 (TZS10, TN, LUS). More than 35: -70 (PAG). No tricks: +70 (PAG, TN). Not stated (TZS10). Others write 0 when someone is full or empty (PAG, TN). ST: +10 and -35. Rounded to 5 (PAG) or exact (TZS10, TN) | 35+: -70. Exact |
| 47 | Radelci: on/off and triggers | Klop, berač, valat (TZS10, LUS). Plus solo brez from 2018 (TZS18). Klop, berač and higher, any valat (PAG). Berač, solo brez, barvni valat, valat, klop (TN). Klop only (ST). Off (PAG money variation) | Klop, berač, valat, solo brez |
| 48 | Radelc cancellation | Cancelled only when the declarer wins (PAG, TN, ST). Erased after the player's next game, won or lost (TZS10, LUS) | Erased after the next game |
| 49 | Radelc ownership | Personal (TZS10, PAG, TN). Shared pool. Any won game erases one (ST, "Štajerci" variant) | Personal |
| 50 | Radelc and klop | Klop result is not doubled (TZS10, TN, LUS) | Not doubled |
| 51 | Unused radelc at the end | -100 each (PAG). -40 each (VAL, snippet only). No penalty stated (TZS10, TN) | No penalty stated |
| 52 | Mondfang size | -21 (TZS10, TN, ST, LUS, PAG 3p and high-stakes). -20 (PAG 4p). Pays 20 to each player (PAG money game) | -21 |
| 53 | Mondfang details | Not under kontra or radelc (PAG, TN). Captured by own partner: both write -20 (TN). Captured by partner without need: unethical play, referee acts, -21 stays (TZS25). Applies only in normal contracts and solo brez (PAG) | -21. Partner capture handled by ethics code |
| 54 | Renons penalty | Full game value (PAG). 35 + game + announcements + special announcements, radelc counts (TZS10). Same plus kontras, +100 if intentional (TN). Game annulled (ST) | TZS10 formula |
| 55 | Session result | Raw totals (PAG, TN). Table points 3 / 2 / 1 per round, ties played off (TZS17). 2 / 1 / 0.5 (TZS10 original) | 3 / 2 / 1 |

Total: 55 variation points.

---

## (d) Neighbouring games

Austrian Königrufen (KOE, ENW-K). Same 54-card pack, same card values, 70 points, 36 to win. 4 or 5 players, anticlockwise. The partner is found by calling a king, as in Slovenian tarok. Scores are small game points, not card points. Card points only decide who won. The contract list is longer: Rufer 1, Trischaken 1, Piccolo 2, Solo 2, Bettel 3, Farbendreier 3, Dreier 4, Piccolo ouvert 5, Bettel ouvert 6, Farbensolo 6, Solodreier 8, plus Sechserdreier and Besserrufer. Extra bonuses exist for winning late tricks with tarok II, III, IIII (Uhu, Kakadu, Marabu/Quapil). Mondfang is a 1-point bonus. Valat multiplies the game by 4 or 8. Austria has several tournament rule sets (Raiffeisen, Hausruckviertler, Wiener Zeitung, Tiroler, Styrian cups). ENW-K says their contracts and values differ. ENW-K on Slovenia: Slovenian tarok lets declarer take two or one talon cards, has no Solo Rufer with hidden talon, and scores the points above 35.

Hungarian tarokk (HUN). 42 cards: the 54-card pack without the low pip cards. 4 players, or 5 with the dealer out. 9 cards each, 6 in the talon. 94 card points, 48 to win. Bids are three, two, one, solo, worth 1, 2, 3, 4 game points. A player needs an honour (škis, XXI or pagat) to bid. The partner is found by calling tarok XX, not a king. Bonuses: tuletroá, four kings, double game, volát, pagátultimó 5 / 10, XXI-catch 21 / 42. Kontra runs to mordkontra. Scoring is in game points.

Czech and Slovak taroky (TAR). Same 54-card pack and card values, 70 points. 4 players, or 5 with the dealer out, anticlockwise. One round of bidding: povinnost, druhá povinnost, preferanc (three levels), sólo. The negative game is varšava. The partner is found by calling tarok XIX. The talon is split 4-1-1 between declarer and the next two players. Players declare hand combinations (taroky, taročky, barvy, barvičky, trul, honéry). Doubling is flek, reflek, super. Payment is per card point above 35.

Croatian tarok. No primary source was found. PAG-HR lists no tarok game for Croatia. HRW has a Tarok article, but it does not name Croatia among the countries where the game is played. Its contract ladder is the Slovenian one with Croatian names: trojka 10, dvojka 20, jedinica 30, pikolo 35 (unofficial), solo tri 40, solo dva 50, solo jedan 60, prosjak 70, solo bez 80, otvoreni prosjak 90, valat u boji 125, valat 250. Kontra runs to mortkontra, x16. HRW says the deal is clockwise.

Assessment for the planner (my inference, not from a source). Königrufen, tarokk and taroky differ in deck size, partner mechanism or scoring unit. They are separate games, not settings of one game. The 55 points above all sit inside Slovenian tarok. "Croatian tarok" looks like Slovenian tarok with translated terms, but this is not verified.

---

## (e) Verified vs inferred

### Read directly in a source

- All of B2 (3-player baseline): read in the full text of TZS10. Amendments read in TZS-P, TZS17, TZS18, TZS22. Mondfang ethics note read in the full text of TZS25.
- All Tarok.net values and rules (TN): read in the extracted PDF text. The value tables were re-extracted in reading order to check column alignment.
- LUS section 4.1: read in the extracted PDF text.
- B3 (4-player baseline) and all "PAG" entries: taken from three fetches of the pagat page. The fetch tool returns a model-written digest, not the raw page. The three digests agree with each other. Numbers were not checked against raw HTML.
- SLW, ST, HRW, KOE, HUN, TAR, ENW-K, TZS-ORG: same limitation. Digest of the page, not raw text.

### Weak or indirect

- VAL (valat.si): the rules page was not readable (404). The two claims used (exact difference, unused radelc -40) come from a search-engine snippet only.
- ST contract values (solo 20 / 30 / 40, solo brez 50, pikolo 60, berač 70, odprti berač 80): the table was garbled in extraction. Treat as unverified. ST also reports a card total of 71, which conflicts with 70 everywhere else.
- ST "taroks must be answered with a higher tarok" in normal play: seen in one digest only. Not used in the table. PAG and TN say there is no duty to overtake in normal contracts.
- TZS10 klop with kontra (B2.4): the Slovenian wording is ambiguous. My reading is in B2. It needs a check with a tournament player.
- TZS10 pagat ultimo wording: "s pagatom, kot edinim tarokom osvoji zadnji vzetek". Literal meaning: the last trick is won by the pagat as the only tarok in it. I did not find a clarification.

### Inferred or from memory

- "No duty to overtake in normal contracts" for the official 3p game: TZS10 does not state the follow-suit rule in positive form. Taken from TN and LUS.
- Forced overtaking in official klop and berač: stated by PAG, TN and LUS. TZS10 does not state it in B2 or B3.
- That TARZS has no 4-player ruleset: inferred from the index of acts (TZS-IDX), which lists only "Pravila tekmovalnega taroka", and from TZS10, which defines three seats per table. PAG also says TARZS tournaments are 3-player.
- Croatian tarok equals Slovenian tarok: inference from HRW only.
- The assessment at the end of section (d) is mine.
- Nothing in sections (b) and (c) is taken from memory alone. Where a source was silent, the table says "not stated".

### Contradictions between sources

- Berač: 125 and top game (TZS10) vs 70 and below solo brez (PAG, TN, SLW, ST, LUS). LUS (2002) says it follows TARZS and gives 70, with solo 50. So TARZS values changed between 2002 and 2010.
- Direction: clockwise (TZS10, TN competitive part) vs anticlockwise (PAG, ST, TN general part). TN contradicts itself between its general and competitive parts.
- 3p tri / dve / ena: 10 / 20 / 30 (TZS10, PAG) vs 10 / 30 / 50 (TN).
- Mondfang: -21 (TZS10, TN, ST, LUS) vs -20 (PAG 4p).
- Mond in a trula trick: no penalty (TZS10) vs penalty applies (PAG).
- Radelc cancellation: after the next game regardless of result (TZS10, LUS) vs only after a win (PAG, TN, ST).
- Radelc triggers: four different lists (see point 47).
- Rounding: to 5 (PAG 4p) vs exact (TZS10, TN default).
- Klop "full" threshold: 35 or more (TZS10, TN, LUS) vs more than 35 (PAG).
- Kontra depth: x16 (TZS10, PAG) vs x4 (TN).
- Kontra on klop: none (PAG default, SLW) vs one level (TZS10) vs two levels (TN).
- Partner's score: equal (PAG, SLW) vs half (TN).
- Barvni valat: 125 (PAG, SLW list) vs 250 (SLW bonus section) vs 125 to 250 by base contract (TN).
- Valat: 500 / 250 (TZS10, PAG, SLW, ST) vs 250 (HRW) vs graded 200 to 500 with silent +50 (TN).
- Pikolo: 70 (PAG variation) vs 35 (SLW, HRW) vs 60 (ST).
- Solo brez: 80 (TZS10, PAG, TN) vs 50 (ST, LUS 2002).
- Kralj ultimo: 10 / 20 (PAG) vs 10 / 20 or 25 / 50 depending on who brings the king (TN).

### Not found

- Text of the TARZS amendments adopted 21.10.2023 in Ptuj.
- Any official or federation-level 4-player ruleset.
- A readable copy of the valat.si rules page.
- A primary source for tarok as played in Croatia.
- TZS25 mentions a proposal for new "Pravila tekmovalnega taroka". The referees' commission called it premature. The official rules may change after 2026.
