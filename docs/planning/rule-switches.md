# Rule switches - proposed toggles

Derived from `research/01-rules-and-variants.md` (research date 2026-11-06). Planning only. No code.

Source keys (PAG, TN, TZS10, TZS-P, TZS18, SLW, ST, HRW, LUS, VAL) are the ones defined in research/01 section (a). Variation point numbers (#1 to #55) refer to research/01 section (c).

Conventions used below:

- Default = the first option in each row. Where PAG has a 4-player rule, the default follows PAG. Where it does not, the default is the option most sources support.
- "Agent" column: **yes** = changes legal actions, contract set or information the agent sees. **scoring** = changes the payoff only; legal play is unchanged. **no** = no effect on play.
- Tier: **T0** = every option is in the first trained model and the first release. **T1** = planned. **T2** = stretch. Until a toggle ships, the game uses its default. One exception: S7 (barvni valat) is pinned off in the first release. See open question 9.
- Options with a weak source are marked "(unverified)". Nothing below is added from memory unless marked "(from memory, unverified)".
- A toggle's value is set per table and fixed for the whole session.

---

## 1. Baseline (fixed, not toggles)

Four players, called king. 54 cards, card values and the 70-point total as in research/01 B1 (all sources agree). 12 cards each, 6 in the talon, dealt in packets of six (PAG). Bidding by priority: forehand highest, dealer lowest; a lower-priority player must bid higher, a higher-priority player may hold; three passes end the auction; klop and tri only for forehand when the other three pass (PAG; TZS10 gives the same priority rule for 3 players). Contract ladder and values: klop, tri 10, dve 20, ena 30, solo tri 40, solo dve 50, solo ena 60, berač 70, solo brez 80, odprti berač 90, barvni valat 125, valat 500 (PAG, TN, SLW agree on every value present in each; TZS10 for 3 players differs: no solo tri/dve/ena, berač 125 above solo brez, valat only to outbid berač). Difference = card points minus 35 (all). From berač upward flat values, no difference, no bonuses (PAG, TN). Declarer names a suit before the talon is opened; the holder of that king is the secret partner; declarer may call his own king (PAG, TN). Talon is shown and the declarer takes a packet of the bid size (PAG and ST wording, "the part with the king"). Kings and trula are never discarded (all). Talon cards not taken go to the opponents; a mond left there counts as captured (PAG, TN, LUS). Announcements after the talon exchange, by both sides; only the pagat holder announces pagat ultimo, only the called-king holder announces kralj ultimo (PAG). Bonus values: trula 10/20, kralji 10/20, kralj ultimo 10/20, pagat ultimo 25/50, valat 250/500 (PAG, TZS10 for the ones it has). Valat cancels all other scores of the hand (PAG). Play: follow suit, else tarok; no duty to overtake in normal contracts (PAG, TN, LUS); trula in one trick is won by the pagat (all); forehand leads up to solo ena, declarer leads from berač upward and in barvni valat (PAG, TZS10, TN); in klop, berač and odprti berač a player must beat the highest card on the table if able, and the pagat may be played only when it is the only legal card (PAG; TN's "last tarok, never led" has the same effect); in klop each of the first six tricks gets one talon card (PAG, TZS10); in barvni valat taroks do not beat the led suit and the trula rule does not apply when a suit is led (PAG). Scoring: declarer and partner write the same score, opponents nothing (PAG; partner share is toggle S26); individual entries for klop and mond; klop with no tricks is +70 (PAG, TN); mondfang is not multiplied by kontra or radelc and applies only in normal contracts and solo brez (PAG, TN); klop results are never doubled by a radelc (TZS10, TN, LUS). Kontra and its levels are said against the game and against each announcement separately; no kontra against your own partner (PAG, TZS10). Misdeals, renons penalties and time limits are not modelled: the software cannot misdeal and does not allow illegal cards.

---

## 2. Proposed toggles

Columns: ID, name, options (default first), affects, agent, tier, sources, player-facing description.

### Dealing

| ID | Name | Options (default first) | Affects | Agent | Tier | Sources | Description |
|---|---|---|---|---|---|---|---|
| S1 | Compulsory klop triggers (#4, #11) | (a) after a no-tarok redeal and when a cumulative score is exactly 0; (b) none; (c) also at every multiple of 500 | session, bidding | no | T1 | a: PAG main. b: TZS10, TN. c: PAG variation | When a klop is forced instead of bidding. |
| S2 | Seats at the table (#1) | (a) 4; (b) 5, dealer sits out, scores nothing, gets a radelc when all do | session | no | T2 | a: PAG, TN. b: PAG | Five people can sit at one table; the dealer rests. |

### Bidding and contracts

| ID | Name | Options (default first) | Affects | Agent | Tier | Sources | Description |
|---|---|---|---|---|---|---|---|
| S3 | Raising the won bid (#9) | (a) declarer may play the bid or any higher contract; (b) must play exactly the bid | bidding | yes (mask) | T1 | a: PAG, TZS10. b: PAG variation | Whether the winner of the auction may go higher. |
| S4 | Klop (#10) | (a) on; (b) off | bidding, play | yes (mask) | T1 | a: all rule texts. b: PAG money variation | Play klop when everybody passes, or not. |
| S5 | Pikolo (#13) | (a) off; (b) on, 70, between solo ena and berač; (c) on, 35, between ena and solo tri | bidding, play, scoring | yes (new contract) | T2 | a: PAG, TN, TZS10. b: PAG variation. c: SLW, HRW ("unofficial") | Contract: win exactly one trick. |
| S6 | Odprti berač (#14) | (a) on, 90; (b) off | bidding, play | yes (new information) | T0 | a: PAG, SLW. b: TN, TZS10 | Berač with the hand exposed after the first trick. |
| S7 | Barvni valat: how it is reached (#18) | (a) only by raising solo tri/dve/ena after the talon exchange; (b) a bid of its own above odprti berač; (c) off. First release: off | bidding, play | yes (new action, new trick rule) | T1 | a: PAG. b: SLW, HRW. c: TZS10 (3p) | Valat in which suit cards beat taroks. |
| S8 | Barvni valat value (#18) | (a) 125 flat; (b) by base contract 125 / 150 / 175 / 250 for solo tri / dve / ena / brez | scoring | scoring | T1 | a: PAG, SLW list. b: TN | What a barvni valat is worth. |
| S9 | Barvni valat with a partner (#19) | (a) not allowed; (b) allowed from tri/dve/ena, TN values 100 / 110 / 120, partner writes half | bidding, scoring | yes | T2 | a: PAG. b: PAG variation, TN | Whether a called-king game can become a barvni valat. |
| S10 | Barvni valat: suits before taroks (#20) | (a) any order; (b) declarer must play all suit cards before leading a tarok | play | yes (mask) | T2 | a: PAG. b: PAG variation | Order of play forced on the declarer in barvni valat. |
| S11 | Valat values (#21, #22) | (a) contract 500, announced 500, silent 250; (b) graded by base contract: solo 250 / 300 / 350 / 500, with partner 200 / 220 / 240, silent adds 50 | scoring | scoring | T1 | a: PAG, TZS10, SLW, ST. b: TN | What a valat is worth. |

### Called king and talon

| ID | Name | Options (default first) | Affects | Agent | Tier | Sources | Description |
|---|---|---|---|---|---|---|---|
| S12 | Called king in the talon (#23) | (a) declarer plays alone; if he takes the packet with the king and wins a trick with it he also gets the rest of the talon; (b) as a, plus 5 points if he takes the king but it wins no trick; (c) declarer must take the packet with the king (unverified) | talon, scoring | a, b: scoring. c: yes (mask) | T1 | a: PAG, TN. b: TN. c: ST (digest only) | What happens when the called king lies in the talon. |
| S13 | Calling with all four kings (#24) | (a) must call own king; (b) may call a trula card instead; (c) may call "the fourth king" without naming it | king call | yes (new actions) | T2 | a: PAG. b, c: PAG variations | Who to call when you hold every king. |
| S14 | Discarding taroks (#25) | (a) allowed, shown face up; (b) only when the hand forces it, shown | talon | yes (mask) | T0 | a: PAG, TZS10. b: TN, ST, HRW | Whether taroks may go into the discard freely. |

### Announcements and kontra

| ID | Name | Options (default first) | Affects | Agent | Tier | Sources | Description |
|---|---|---|---|---|---|---|---|
| S15 | Kralj ultimo value (#34) | (a) 10 silent / 20 announced; (b) 10 / 20 if the partner wins the last trick with the king, 25 / 50 if the declarer does | scoring | scoring | T0 | a: PAG, SLW. b: TN | Bonus for taking the last trick with the called king. |
| S16 | Lost silent pagat ultimo (#36) | (a) the whole side pays 25; (b) only the pagat holder pays; (c) silent pagat is scored only as a loss, never as a bonus | scoring | scoring | T1 | a: PAG, TZS10, SLW. b, c: PAG variations | Who pays when the pagat falls in the last trick unannounced. |
| S17 | Bonuses in solo brez (#37) | (a) none; (b) silent and announced bonuses count | announcements, scoring | yes (announcements become legal) | T2 | a: PAG, TN. b: TZS-P (3p) | Whether trula, kings and ultimos score in solo brez. |
| S18 | Kontra levels (#38) | (a) 4: kontra, re, sub, mort, x16; (b) 2: kontra, rekontra, x4; (c) 5: extra "hirškontra" between sub and mort, multiplier not in source (T2) | announcements, scoring | yes (b: mask; c: new action) | T0 | a: PAG, TZS10, SLW, LUS. b: TN. c: PAG variation | How far doubling can go. |
| S19 | Kontra in klop (#39) | (a) none; (b) one level, by one player; doubles the klop results; (c) loser pays 140; if nobody is full the kontra player pays 140; (d) kontra and rekontra; the kontra player may take at most 17 points | announcements, scoring | yes (new actions) | T2 | a: PAG, SLW. b: TZS10. c: PAG variation. d: TN | Whether anyone may double a klop. |
| S20 | Failed kontra by an opponent (#40) | (a) the whole side pays; (b) only the player who said kontra pays the extra; (c) the opponents who did not say kontra get the difference as a bonus | scoring | scoring | T1 | a: PAG. b: TN (2015). c: TZS10 B1.22, LUS (3p) | Who carries a kontra that fails. |
| S21 | Silent bonuses under game kontra (#41) | (a) silent results are never multiplied by kontra; (b) a silent valat is multiplied by kontra | scoring | scoring | T2 | a: TN; PAG implied (kontra is said per announced item). b: TZS-P | Whether a kontra on the game also hits silent bonuses. |

### Play

| ID | Name | Options (default first) | Affects | Agent | Tier | Sources | Description |
|---|---|---|---|---|---|---|---|
| S22 | Mond lost in a trula trick (#30) | (a) mondfang penalty applies; (b) no penalty | scoring | scoring | T0 | a: PAG. b: TZS10 | Whether the mond pays when the pagat wins a trula trick. |
| S23 | Leading the mond in klop (#29) | (a) free; (b) not before the škis has been played | play | yes (mask) | T2 | a: PAG, TZS10. b: PAG variation | Restriction on leading the mond in klop. |

### Scoring

| ID | Name | Options (default first) | Affects | Agent | Tier | Sources | Description |
|---|---|---|---|---|---|---|---|
| S24 | Winning threshold (#42) | (a) 36 card points; (b) 35 points plus one extra card | scoring | scoring | T1 | a: TZS10, PAG, TN ("more than 35"), HRW. b: PAG variation | How many points the declarer's side needs. |
| S25 | Rounding of the difference (#43, #46) | (a) nearest 5, also for klop entries; (b) exact | scoring | scoring | T0 | a: PAG 4p. b: TZS10, TN default, VAL (snippet) | Whether scores are rounded to fives. |
| S26 | Partner's share (#44) | (a) same as declarer; (b) half of declarer's score | scoring | scoring | T0 | a: PAG, SLW. b: TN | What the partner writes. |
| S27 | Scoring model (#45) | (a) paper: declarer's side writes, opponents nothing; (b) money: each loser pays a winner, a soloist collects from all three, mondfang paid to each player; (c) settle against the table average | scoring, session | scoring | T2 | a: PAG, TZS10. b, c: PAG money game | How results are written up. |
| S28 | Klop "full" threshold (#46) | (a) more than 35 points: -70; (b) 35 or more: -70 | scoring | scoring | T0 | a: PAG. b: TZS10, TN, LUS | When a klop player is "full". |
| S29 | Mondfang size (#52) | (a) -20; (b) -21 | scoring | scoring | T0 | a: PAG 4p. b: TZS10, TN, ST, LUS, PAG 3p | Penalty for losing the mond to the škis. |
| S30 | Mond captured by own partner (#53) | (a) the holder pays; (b) both partners pay | scoring | scoring | T1 | a: PAG (holder pays, no exception). b: TN | Who pays when the partner's škis takes the mond. |
| S31 | Solo brez difference (#16) | (a) 80 flat; (b) 80 plus difference | scoring | scoring | T0 | a: PAG. b: TN; TZS10 does not exclude it | Whether solo brez scores card points above 35. |

### Session and radelci

| ID | Name | Options (default first) | Affects | Agent | Tier | Sources | Description |
|---|---|---|---|---|---|---|---|
| S32 | Radelc triggers (#47) | (a) klop and every contract from berač upward; (b) klop only (unverified); (c) off | scoring, session | scoring | T0 | a: PAG, TN; TZS18 for the contracts 3p has. b: ST (digest only). c: PAG money variation | When every player gets a radelc. |
| S33 | Radelc cancellation (#48) | (a) only when the declarer wins; one per win; (b) after the player's next game as declarer, won or lost | scoring | scoring | T0 | a: PAG, TN, ST. b: TZS10, LUS | How a radelc is used up. |
| S34 | Unused radelci at session end (#51) | (a) -100 each; (b) no penalty; (c) -40 each (unverified) | session, scoring | scoring | T1 | a: PAG. b: TZS10, TN (none stated). c: VAL (snippet only) | What a leftover radelc costs. |
| S35 | Session end (#6) | (a) škis round: the holder of the škis in that deal deals the last hand; (b) fixed number of deals: 24 (TN), 18 or 21 (TZS); (c) open-ended (app convenience, no source) | session | no | T1 | a: PAG, TN. b: TN, TZS10, TZS-ORG | How a session is closed. |

### 3-player only (all deferred to the 3-player mode)

| ID | Name | Options (default first) | Affects | Agent | Tier | Sources | Description |
|---|---|---|---|---|---|---|---|
| S36 | Compulsory tri for forehand (#8) | (a) forehand must open with tri and may fall back to klop; (b) no compulsory bid; a forehand who bids tri loses the right to klop | bidding | yes | T2 | a: TZS10, LUS. b: PAG 3p, TN | Whether forehand is forced to open. |
| S37 | 3-player contract ladder (#14, #15, #18, #21) | (a) TZS10: tri, dve, ena, solo brez 80, berač 125, valat 500 only to outbid berač; (b) PAG 3p: berač 70 below solo brez 80, optional odprti berač and barvni valat | bidding, scoring | yes | T2 | a: TZS10. b: PAG, TN, LUS (2002, with solo 50) | Order and value of the 3-player contracts. |
| S38 | 3-player tri / dve / ena values (#17) | (a) 10 / 20 / 30; (b) 10 / 30 / 50 | scoring | scoring | T2 | a: TZS10, PAG, LUS. b: TN | Base values of the talon games. |
| S39 | Klop: pagat on mond and škis (#28) | (a) off; (b) if mond and škis are in the trick, the third player must play the pagat if he holds it | play | yes (mask) | T2 | a: PAG, TN. b: TZS10 | Forced pagat in klop when both big taroks are down. |

Totals: 39 toggles. T0: 12 (S6, S14, S15, S18, S22, S25, S26, S28, S29, S31, S32, S33). T1: 13 (S1, S3, S4, S7, S8, S11, S12, S16, S20, S24, S30, S34, S35). T2: 14 (S2, S5, S9, S10, S13, S17, S19, S21, S23, S27, S36, S37, S38, S39).

---

## 3. Incompatibilities and dependencies

### Cannot both be on, or forced values

| Pair | Rule | Reason |
|---|---|---|
| S4 = off and S32 != off | S4 off forces S32 off | PAG describes "no klop and no radelci" as one money variation. Radelci after berač/valat without klop have no source. |
| S4 = off and S1, S19, S23, S28 | S4 off forces S1 = none; S19, S23, S28 inactive | No klop, nothing to trigger, kontra or lead in. What happens when all pass with klop off is not in any source. Redeal is the obvious choice (from memory, unverified). |
| S3 = b (no raise) and S7 = a (raise after talon) | Not allowed together | Barvni valat by raising needs the raise. With S3 = b, S7 must be b or c. |
| S7 = c (off) | S8, S9, S10 inactive | Nothing to value or restrict. |
| S9 = b with S8 = a | Partner barvni valat value is undefined under PAG flat 125 | PAG allows it as a variation but gives no value. Only TN gives values (100 / 110 / 120). Use TN values or fix one. |
| S27 = b or c (money) and S26 | S26 inactive | Money scoring is per player; "partner writes half" has no meaning. |
| S34 != b and S32 = c | S34 inactive | No radelci, nothing left over. |
| S34 and S35 = c (open-ended) | S34 applies only when a session has an end | Open-ended sessions never settle the leftovers. |
| S11 = b (TN graded valat) and S9 = a | Tension | TN's with-partner valat values imply valat with a partner exists. Whether TN also has a stand-alone valat bid is unclear from the source. |
| S20 = c (TZS10 non-kontra bonus) in 4-player | Underspecified | TZS10 B1.22 is written for two opponents. With three opponents against a soloist the split is not defined. |
| S17 = b and S31 = a | No source for this mix | TZS-P (bonuses count) goes with a solo brez whose difference is not excluded. PAG (no bonuses) goes with flat 80. Allowed, but no table plays it. |
| S2 = b (5 seats) | Needs S32; dealer gets a radelc when all do (PAG) | Radelc bookkeeping must cover a seat that does not play. |
| 3-player mode | S6, S7 to S10, S12 to S15, S26, S30 inactive; S36 to S39 active | No king call, no partner, no solo tri/dve/ena in 3 players. |

### Cost for the agent

Expensive (new actions, new contracts, or new information; the network and the self-play curriculum change):

- S5 pikolo: new contract, new objective (exactly one trick).
- S7 barvni valat: new declaration action and a different trick-resolution rule. S9 and S10 add legal-move rules on top.
- S13: new call actions (trula card, unnamed fourth king).
- S18 = c: a fifth kontra action.
- S19: kontra actions inside klop, which the PAG default never has.
- S17: announcement actions in a contract where they are normally masked.
- S6: an "exposed hand" observation slot. Moderate, since the "off" option is a mask.
- S36 to S39: 3-player mode is a separate state and action layout anyway.

Cheap masks (the alternative is a subset of the default action set): S3, S4, S10, S12 = c, S14, S18 = b, S23, S39.

Scoring only (payoff changes, legal play does not): S8, S11, S12 = a/b, S15, S16, S20, S21, S22, S24 to S34, S38. These still move the optimal policy a little (for example S25 rounding makes the 38-to-40 boundary matter). Proposal: feed the T0 scoring toggles into the network as input features and randomise them during self-play, so one model covers them. T1/T2 scoring toggles can be accepted with slight suboptimality or added to the feature vector later.

No effect on play: S1, S2, S35.

---

## 4. Presets

"?" = the source does not state it; the preset falls back to the default. "n/a" = not applicable to that mode. "Majority (derived)" takes, per toggle, the option with the most Slovenian sources behind it. It is not a documented ruleset.

| Toggle | pagat.com (default) | Tarok.net | TARZS tournament (3-player) | Majority (derived) |
|---|---|---|---|---|
| S1 compulsory klop | a (no-tarok, score 0) | b none | b none | b none (TN, TZS10) |
| S2 seats | a 4 | a 4 | n/a (3 players) | a 4 |
| S3 raise bid | a yes | a yes ? | a yes | a yes |
| S4 klop | a on | a on | a on | a on |
| S5 pikolo | a off | a off | a off | a off |
| S6 odprti berač | a on 90 | b off | n/a (optional in PAG 3p) | a on (PAG, SLW vs TN) |
| S7 barvni valat reached | a raise | ? (TN has it; mechanism not stated) | n/a | ? (PAG vs SLW, one each) |
| S8 barvni valat value | a 125 | b graded | n/a | a 125 |
| S9 barvni valat with partner | a no | b yes 100/110/120 | n/a | a no |
| S10 suits before taroks | a free | a free ? | n/a | a free |
| S11 valat values | a 500/500/250 | b graded, silent +50 | a 500/250 | a |
| S12 king in talon | a | b (+5) | n/a | a |
| S13 four kings | a own king | a ? | n/a | a |
| S14 tarok discards | a allowed | b forced only | a allowed | b forced only (TN, ST, HRW vs PAG, TZS10) |
| S15 kralj ultimo | a 10/20 | b split | n/a | a (PAG, SLW vs TN) |
| S16 lost silent pagat | a side pays | a ? | a side pays | a |
| S17 bonuses in solo brez | a none | a none | b scored | a none |
| S18 kontra levels | a 4 | b 2 | a 4 | a 4 |
| S19 kontra in klop | a none | d kontra+re, 17 | b one level | ? (three different rules) |
| S20 failed kontra | a side | b kontra player | c non-kontra bonus | ? |
| S21 silent under kontra | a no | a no | b silent valat yes | a no |
| S22 mond in trula trick | a penalty | ? | b no penalty | ? (PAG vs TZS10) |
| S23 mond lead in klop | a free | a free | a free | a free |
| S24 threshold | a 36 | a 36 | a 36 | a 36 |
| S25 rounding | a nearest 5 | b exact | b exact | b exact |
| S26 partner share | a equal | b half | n/a | a equal (PAG, SLW) |
| S27 scoring model | a paper | a paper | a paper | a paper |
| S28 klop full | a >35 | b >=35 | b >=35 | b >=35 |
| S29 mondfang | a -20 | b -21 | b -21 | b -21 |
| S30 partner mondfang | a holder | b both | n/a | a ? (only TN speaks) |
| S31 solo brez difference | a flat | b +difference | b ? (not excluded) | ? |
| S32 radelc triggers | a | a | a (klop, berač, solo brez, valat) | a |
| S33 radelc cancellation | a win only | a win only | b after next game | a win only |
| S34 unused radelci | a -100 | b none | b none | b none |
| S35 session end | a škis round | a or b 24 deals | b 18 or 21 deals, 4 min 30 s per deal (timer not modelled) | a |
| S36 compulsory tri | n/a | n/a | a compulsory | n/a |
| S37 3p ladder | n/a | n/a | a TZS10 | n/a |
| S38 3p values | n/a | n/a | a 10/20/30 | n/a |
| S39 klop pagat rule | n/a | n/a | b on | n/a |

Uncertain cells in the Tarok.net preset: S3, S7, S10, S13, S16, S22. In the TARZS preset: S31. The Majority preset is undecided on S7, S19, S20, S22, S31.

---

## 5. Left out

Variation points from research/01 that did not become a toggle.

| # | Variation point | Reason |
|---|---|---|
| 1 | Number of players (part) | 3 vs 4 is a game mode, not a toggle. 5 seats became S2. "3+1" (dealer rests but plays in klop, TN competitive) deferred with 3-player mode; mechanics unclear. 2-player "Napoleon": changes the game too much. |
| 2 | Direction of deal and play | Cosmetics. In software the seat order is a display choice; forehand is always the next seat in play direction. |
| 3 | Deal order and cut, knock | Cosmetics. A digital deal is random; packet order and cutting change nothing. |
| 4 | Hand without taroks (part) | Redeal is automatic in software. The "compulsory klop after it" part became S1. "Annul with one tarok or two below X": PAG 3-player variation only, rare. |
| 5 | Misdeal penalty | Not applicable. The software cannot misdeal. |
| 7 | Time limit per game | Tournament logistics, not a rule of play. May return as an app timer setting. |
| 11 | Compulsory klop (part) | Folded into S1. The "after a misdeal" trigger is not applicable. |
| 12 | Solo tri / dve / ena values | Fixed in the baseline (40 / 50 / 60). ST's other values came from a garbled table; unverified. |
| 13, 14, 16 | ST values (pikolo 60, odprti berač 80, solo brez 50) and LUS 2002 solo brez 50 | Garbled extraction or obsolete (TARZS changed values between 2002 and 2010). Not offered as options. |
| 18 (part) | Barvni valat 250 (SLW bonus section) | SLW contradicts its own list. No second source. |
| 22 (part) | Valat 250 only (HRW) | Croatian Wikipedia only; no Slovenian source. |
| 26 | Mond left in the talon | Every source that mentions it agrees it counts as captured. Baseline. |
| 27 | First lead | Main sources agree. LUS "declarer always" is a program simplification. ST "left of declarer in berač" is a single digest. Baseline. |
| 28 (part) | Pagat in klop and berač, 4-player wording | PAG and TN differ in words, not in effect. Baseline. The TZS10 mond-and-škis rule became S39 (3-player). |
| 31 | Berač with škis forcing mond and pagat | Too rare. One PAG variation; interaction with forced overtaking not described. |
| 32 | "Vogel frei" | 3-player PAG variation only, rare. |
| 33 | Looking at earlier tricks | UI, not a rule. The agent remembers every played card regardless; the human client can show the last trick. |
| 35 | Who may announce | No real alternative with a 4-player source. Baseline: holder announces ultimos, both sides may announce. |
| 38 (part) | Hirškontra multiplier | Kept as S18 option c but the multiplier is not in the source. |
| 46 (part) | Klop +10 / -35 (ST) | Garbled table, one source. Klop rounding folded into S25. |
| 49 | Shared radelc pool ("Štajerci") | One garbled source. Too rare to carry. |
| 50 | Radelc and klop | All sources agree klop is not doubled. Baseline. |
| 53 (part) | Partner captures mond without need | TZS25 handles it by ethics code and referee. Not a rule the software can enforce. Contract scope (normal contracts and solo brez) is baseline. |
| 54 | Renons penalty | Not applicable. The software does not allow an illegal card, and talk cannot be judged. |
| 55 | Session result as table points 3 / 2 / 1 | Tournament format for 3-player tables. No 4-player version in any source. May return with a tournament mode. |

Fully dropped points: 15 (2, 3, 5, 7, 12, 26, 27, 31, 32, 33, 35, 49, 50, 54, 55). Partly dropped: 1, 4, 11, 13, 14, 16, 18, 22, 28, 38, 46, 53. Every other point maps to a toggle in section 2.

---

## 6. Open questions for the user

Defaults that are not settled by the sources. Each follows PAG for now.

1. **S14 tarok discards.** PAG and TZS10 allow discarding taroks (shown). TN, ST and HRW allow it only when forced. Default a (allowed) or b (forced only)?
2. **S25 rounding.** PAG rounds to 5 for 4 players. TN and TZS10 score exactly. Default nearest 5 or exact?
3. **S29 mondfang.** PAG 4-player says -20. Every other source says -21. Default -20 or -21?
4. **S28 klop full.** PAG says more than 35. TZS10, TN and LUS say 35 or more. Which?
5. **S26 partner share.** PAG and SLW: equal. TN: half. Which?
6. **S33 radelc cancellation.** PAG, TN, ST: only on a win. TZS10, LUS: after the next game, won or lost. Which?
7. **S34 and S35 session rules.** Unused radelci -100 (PAG) or none (TN, TZS10)? Session closed by škis round, fixed deal count, or open-ended?
8. **S6 odprti berač in the first model.** Proposed yes (T0, default on). It costs one observation slot for the exposed hand. Keep it in the first model, or pin it off like barvni valat?
9. **S7 barvni valat in the first release.** Proposed: implement the rule in the engine but pin it off until T1, so the first agent does not have to learn the reversed trick rule. Acceptable, or must the first release match PAG fully?
10. **S1 compulsory klop at a cumulative score of exactly 0.** PAG only. Keep as default, or default to none (TN, TZS10)?
11. **S18 kontra depth.** PAG, TZS10 and SLW say four levels (x16). TN says two (x4). Four is proposed. Confirm.
