# Encoding contract

Status: **approved by the user on 2026-11-10 (D117).** Encoding version: **1**. Changes follow section 9.

This document fixes what the network sees and what it can do. The Rust engine produces these arrays; the Python learner consumes them. Both tracks build against this file (D109). It covers **every phase and every Default-preset contract from the start** (D107). Slots for things not built yet are reserved and masked, so a checkpoint stays loadable when they arrive.

Decisions behind it: D15, D18, D19, D21, D30, D47, D107, D110. Sources for the starting point: `docs/planning/research/06` section (g) and `research/05` sections C2–C4. Everything beyond those is Claude's proposal for the user to correct. Items marked * are pinned off in the first release (D44).

The sequence layout is L2 (D116): play and bid tokens attend causally to each other; each decision's state token attends to the tokens before it and to itself. The arrays below do not depend on the layout.

## 1. Cards and seats

- **Card index 0–53** (frozen by spec 002): taroks I–XXI are 0–20, škis is 21. Then srce 22–29, karo 30–37, pik 38–45, križ 46–53, each low to high (srce and karo: 4, 3, 2, 1, fant, kaval, dama, kralj; pik and križ: 7, 8, 9, 10, fant, kaval, dama, kralj).
- **Seats are relative to the acting seat:** 0 = self, 1 = next to play, 2 = across, 3 = previous. Every "per seat" block and every token's seat field uses this order. The same deal seen from two seats differs only by this rotation.
- Card sets are 54 bits in card-index order.

## 2. One decision

At every decision the engine gives, for the acting seat:

| Array | Shape | Type | Content |
|-------|-------|------|---------|
| `tokens` | up to 128 × 5 | small integers | the public events so far (section 3) |
| `state` | 1382 | numbers in 0–1 | the state-token features (section 4) |
| `mask` | 160 | bits | which actions are legal now (section 5) |
| `critic` | 1382 + 274 | numbers in 0–1 | training only: the state plus the hidden information (section 6) |
| `targets` | section 7 | | training only: labels for the auxiliary heads |

At the end of the deal it gives one reward per seat and the same reward split into ten score parts, 4 seats × 10 numbers (section 8).

The MLP control (D18) reads `state` only. The transformer reads `tokens` and `state`.

## 3. Tokens

One token per public event, in the order it happened. Private events (the declarer's hidden discards, anyone's hand) are never tokens; they live in `state`.

Each token has five fields. The network adds up one learned embedding per field (D30).

| Field | Values | Meaning |
|-------|--------|---------|
| event | 0–287 | what happened (table below) |
| seat | 0–3, or 4 for "no seat" | who did it, relative to the acting seat |
| trick | 0–12 | 0 before card play, then the trick number |
| position in trick | 0–4 | 0 outside card play; 1–4 for the first to fourth card |
| index | 0–127 | position in the sequence |

Event vocabulary (288 ids):

| Id | Count | Name | Meaning |
|----|-------|------|---------|
| 0 | 1 | `PAD` | padding |
| 1 | 1 | `BOS` | start of the deal |
| 2–14 | 13 | `BID_x` | a bid: naprej (pass), klop, tri, dve, ena, solo tri, solo dve, solo ena, berač, solo brez, odprti berač*, barvni valat*, valat |
| 15–27 | 13 | `CONTRACT_x` | the contract the declarer finally plays (same 13 values; may be higher than the winning bid, S3 a) |
| 28–31 | 4 | `KING_suit` | called king: srce, karo, pik, križ |
| 32–85 | 54 | `TALON_SHOWN_card` | one token per talon card when the talon is turned up |
| 86–91 | 6 | `TALON_TAKEN_group` | which packet the declarer took (index 0–5) |
| 92–145 | 54 | `DISCARD_SHOWN_card` | a discarded tarok, shown to all (S14) |
| 146–150 | 5 | `ANNOUNCE_x` | trula, kralji, kralj ultimo, pagat ultimo, valat |
| 151–156 | 6 | `KONTRA_target` | raises the kontra level by one on: game, trula, kralji, kralj ultimo, pagat ultimo, valat |
| 157 | 1 | `ANNOUNCE_DONE` | never written: "done" is an action, not an event (section 4a, spec 009) |
| 158 | 1 | `ANNOUNCE_BARVNI_VALAT`* | reserved (S7 a) |
| 159–212 | 54 | `PLAY_card` | a card played to a trick |
| 213–266 | 54 | `KLOP_TALON_card` | a talon card added to a trick in klop |
| 267–287 | 21 | reserved | for later toggles |

Longest sequence, counted by hand: 1 start + up to about 24 bids + 1 contract + 1 king + 6 talon cards + 1 packet + up to 3 shown discards + up to about 16 announcement events + 48 plays + 6 klop talon cards. The limit is 128. An engine test must confirm that no legal deal exceeds it.

Counted again with the rules as built (spec 009): at most 13 calls, so at most 26 tokens before the round of announcements; at most 29 in the round (5 announcements, 24 kontras); 48 plays. 103 in all. The engine's tests walk the longest auction, build the longest round and play 10,000 random deals.

## 4. State token (1382 features)

All features are 0 or 1 unless a divisor is given. "Per seat" means four entries in relative seat order; "other seat" means the three seats that are not self.

| Block | Width | Offset | Notes |
|-------|-------|--------|-------|
| **A. Cards** | **702** | | |
| A1 own hand | 54 | 0–53 |  |
| A2 cards played, per seat (4 × 54) | 216 | 54–269 | completed tricks and the current one |
| A3 current trick by play order (3 × 54) | 162 | 270–431 | first, second, third card; zero if not played |
| A4 talon cards shown | 54 | 432–485 | once the talon is turned up |
| A5 talon cards taken by the declarer | 54 | 486–539 |  |
| A6 own discards | 54 | 540–593 | private; only the declarer has them |
| A7 discards shown to all | 54 | 594–647 | taroks (S14) |
| A8 klop talon cards given to tricks so far | 54 | 648–701 |  |
| **B. Contract and roles** | **53** | | |
| B1 phase | 8 | 702–709 | bidding, contract choice, king call, talon pick, discard, announcements, play, over |
| B2 contract | 13 | 710–722 | zero until fixed |
| B3 declarer seat | 4 | 723–726 | zero in klop |
| B4 own role | 4 | 727–730 | declarer, called partner, opponent, no teams (klop) |
| B5 called suit | 4 | 731–734 |  |
| B6 revealed partner seat | 4 | 735–738 | zero until the called king is played or the partner is otherwise public |
| B7 flags | 4 | 739–742 | called king played; called king lies in the shown talon; I hold the called king; declarer is publicly alone |
| B8 dealer seat (4), forehand seat (4) | 8 | 743–750 |  |
| B9 leader of the current trick | 4 | 751–754 |  |
| **C. Bidding** | **73** | | |
| C1 highest bid so far | 13 | 755–767 |  |
| C2 seat of the highest bidder | 4 | 768–771 |  |
| C3 seat has passed | 4 | 772–775 |  |
| C4 last bid per seat (4 × 13) | 52 | 776–827 |  |
| **D. Announcements and kontra** | **41** | | |
| D1 announcement made, per side (2 × 5) | 10 | 828–837 | declarer side, opponents |
| D2 kontra level per target (6 × 5) | 30 | 838–867 | none, kontra, re, sub, mort |
| D3 barvni valat announced* | 1 | 868–868 | reserved |
| **E. Counts (numbers in 0–1)** | **14** | | |
| E1 cards in hand per seat / 15 | 4 | 869–872 |  |
| E2 tricks won per seat / 12 | 4 | 873–876 |  |
| E3 card points in each seat's pile / 70 | 4 | 877–880 | public: every trick is seen |
| E4 trick number / 12; cards in the current trick / 3 | 2 | 881–882 |  |
| **F. Exact inference bits (D30)** | **463** | | |
| F1 other seat is void (3 seats × 5) | 15 | 883–897 | srce, karo, pik, križ, taroks; shown by not following |
| F2 other seat cannot hold the card (3 × 54) | 162 | 898–1059 | seen elsewhere, or the seat is void in its suit |
| F3 other seat certainly holds the card (3 × 54) | 162 | 1060–1221 | by elimination; includes the revealed called king |
| F4 card is unseen | 54 | 1222–1275 | not in own hand, not played, not shown |
| F5 own card is the highest unseen of its suit | 54 | 1276–1329 | taroks count as a suit |
| F6 derived counts | 16 | 1330–1345 | taroks unseen / 22 (1); unseen per suit / 8 (4); king unseen (4); škis, mond, pagat unseen (3); card points unseen / 70 (1); most taroks each other seat can still hold / 12 (3) |
| **G. Rules and session** | **36** | | |
| G1 scoring toggles (D47) | 19 | 1346–1364 | S15 (2), S22 (2), S25 (2), S26 (2), S28 (2), S29 (2), S31 (2), S32 (3), S33 (2), one-hot each |
| G2 reserved rule slots | 13 | 1365–1377 | for the later rule vector (D20) |
| G3 radelci per seat / 3, capped | 4 | 1378–1381 | see open point C |
| **Total** | **1382** | | flat features 919, exact inference bits 463 |

Rules for the inference block F:
- It holds only facts that follow with certainty from what this seat has seen. No probabilities. Beliefs are the network's job (the auxiliary heads).
- F2 and F3 use simple elimination: seen cards, shown voids, the shown talon, the called king once revealed. The declarer's hidden discards keep most suit cards uncertain for the others; the bits are then 0.
- Every F feature has a unit test with a hand-built position.

This is larger than the roughly 620 features sketched in research/06, because D30 adds the inference block and D107 adds the bidding and announcement state from the start.

## 4a. Readings fixed on 2026-11-12 (D130)

Spec 006 found places where sections 3 to 7 left a detail open. The user fixed each one (Batch 12). No size and no id changes. The code marks each place with "Reading B12-n".

- **G1.** The engine has no toggles yet. The 19 bits hold the one-hot of the Default preset's values.
- **Critic, true partner seat.** The declarer's true partner, relative to the observing seat, like B6. Zero if there is none.
- **Partner target.** "None" when the acting seat has no single team-mate: the declarer alone, an opponent in a one-against-three game, every seat during the bidding. The loss skips these decisions. An opponent in a two-against-two game has the other opponent as target.
- **F5.** Every own card that no unseen card of its suit beats. King and queen in hand: both.
- **F3.** Sources: a king or a trula card in the packet the declarer took is in the declarer's hand until played; so is a taken tarok that was not shown as a discard; every taken card when the declarer has no hidden discard; plain elimination.
- **F6.** "Card points unseen / 70" counts the unseen cards as one pile. "Most taroks a seat can still hold / 12" is the smaller of its hand size and the taroks it can still hold, capped at 12.
- **F4.** The taken packet and the declarer's own discards count as seen. The card-location head is not trained on a taken card.
- **Token seats.** The start token and the six shown talon cards carry "no seat". Contract, called king, taken packet and shown discard carry the declarer.
- **C4.** A seat's last real bid, kept after it passes. All thirteen entries are zero for a seat that never bid; C3 holds the pass.
- **B7, E3, E4.** "Called king lies in the shown talon" stays set after the king is taken and played. E3 rounds each seat's pile on its own. E4's trick number is 0 before play and 12/12 at the end.
- **Hidden discards.** A hidden discard is no token. Between the declarer's discard decisions the token sequence is the same; only the state token differs.
- **Seeds.** The record stores the seed as a decimal string. The seed of a deal is a SplitMix64 mixing of base seed, table, run and deal. The first dealer of run r at table t is (t + r) mod 4.

## 4b. Readings fixed on 2026-11-24 (spec 009, D141, D143)

Deal flow B (announcements and kontra) fills slots, ids and features that were reserved. No size and no id changes. The first two are readings of Claude that the user did not object to.

- **"Done" writes no token.** An announcement and a kontra write one token each, with the seat that said it. `ANNOUNCE_DONE` (id 157) stays unused. With a token per "done" a legal deal could pass the limit of 128. What a seat did not say is not in the tokens.
- **D1, unknown side.** An announcement is shown on the side of the seat that made it, as far as the observing seat knows that side. It knows it when the announcer is the declarer or the observer itself, when the partner is public (or the declarer is publicly alone), or when the observer holds the called king. Otherwise both sides' bits are set. They become one bit when the partner becomes public.
- **D2 is a one-hot per target.** Six targets in the order of the kontra slots (game, trula, kralji, kralj ultimo, pagat ultimo, valat), five entries each: none, kontra, re, sub, mort. One entry is always set; before the round and for a bonus nobody announced it is "none".
- **B1.** The phase "announcements" is entry 5: after the last discard, before the first lead.
- **B6, B7.** The partner is public in four cases: the called king is played, the called king lies in the shown talon, a seat other than the declarer announces kralj ultimo, a seat other than the declarer says re or mort on the game. The engine does not work out sides from other kontras.
- **Actions.** The declarer speaks first, then the seats in order of play. A turn is any number of announcements and kontras, one per action, then `ANNOUNCE_DONE` (slot 88), which is always legal in the round. The round ends when every seat has had a turn and the last three turns were "done" alone. Slot 89 stays never legal.
- **Score parts.** An announced bonus is in the part of its kind, in place of the silent one. A kontra multiplies its own target: parts 0 and 1 together (game), or one of parts 2 to 6. A multiplied part holds the multiplied amount. Part 7 (mondfang) is never multiplied.
- **Reward size.** The scale of 50 is unchanged. A deal's reward can be large: mort on a won solo ena with full difference is 16 x 95 = 1,520 points, a reward of about 30; mort on an announced valat is 8,000 points. Rewards are 32-bit numbers: the four rewards of a deal sum to zero, and the ten parts of a seat to its reward, within 1e-6 times the largest reward of the deal (1e-6 for rewards up to 1).

## 5. Actions (160 slots)

One output vector for all phases (D19). The engine's `mask` marks the legal slots; the learner masks the logits before sampling and in the loss (research/05 C3).

| Slot | Count | Name | Meaning |
|------|-------|------|---------|
| 0–12 | 13 | `BID_x` | used in bidding and when the declarer chooses the final contract: naprej (pass), klop, tri, dve, ena, solo tri, solo dve, solo ena, berač, solo brez, odprti berač*, barvni valat*, valat |
| 13–16 | 4 | `KING_suit` | srce, karo, pik, križ |
| 17–22 | 6 | `TALON_group` | packet index: two packets of three (tri), three of two (dve), six of one (ena) |
| 23–76 | 54 | `DISCARD_card` | one card per decision, repeated until the hand is back to 12 |
| 77–81 | 5 | `ANNOUNCE_x` | trula, kralji, kralj ultimo, pagat ultimo, valat |
| 82–87 | 6 | `KONTRA_target` | game, trula, kralji, kralj ultimo, pagat ultimo, valat |
| 88 | 1 | `ANNOUNCE_DONE` | end own announcements |
| 89 | 1 | `ANNOUNCE_BARVNI_VALAT`* | reserved |
| 90–143 | 54 | `PLAY_card` | play a card |
| 144–159 | 16 | reserved | for later toggles (e.g. S13 calling a trula card) |

Notes:
- Discards are single-card decisions, repeated. Kings and trula are never legal discards.
- In the announcement phase a seat may take several actions (announce, kontra) and then `ANNOUNCE_DONE`. The turn order is in section 4b.
- "Klop" as a bid is legal only for forehand after three passes, like "tri" (baseline).

## 6. Critic input (training only)

The actor's `state`, plus 274 features the actor must not see (D14, D15):

| Width | Content |
|-------|---------|
| 162 | the hands of the three other seats (3 × 54) |
| 54 | talon cards not yet public |
| 54 | the declarer's hidden discards |
| 4 | the true partner seat (zero if there is none) |

The critic never runs in the browser. The explanation panel uses the observation-only Q head instead (D15).

## 7. Auxiliary targets (training only)

| Head | Target | Loss |
|------|--------|------|
| Partner seat (D15) | which of the three other seats is on my side: 3 classes, plus "none" | cross-entropy; logged as accuracy before the called king falls (G8) |
| Card location (D15) | for each unseen card: other seat 1, 2, 3, or out of play (talon rest, discards) | cross-entropy per card, only over cards with F4 = 1 (G9) |

## 8. Reward

Decided (D117): a zero-sum form of the paper score.

- Let `e` be the four entries the deal would write on paper under the rules in force: declarer and partner the same score, opponents 0, individual entries for klop and mondfang, radelc doubling applied. In training, a win that cancels a radelc adds 70 to the declarer's entry, because a leftover radelc costs 70 at the end of a session (D44, D88).
- Reward of seat i = ( e_i − mean of the other three e ) / 50.
- It is given once, at the end of the deal. No per-trick shaping (research/05 C4).

Why not the raw paper score: opponents always write 0, so they would get no signal for defending. The relative form pays them the negative of what the declarer's side gains.

**Score parts (D121).** The entry `e` of each seat is a sum of ten parts, in this order:

| Index | Part | Content |
|-------|------|---------|
| 0 | game | the contract's value, with its sign |
| 1 | difference | the rounded difference, with its sign |
| 2 | trula | silent or announced |
| 3 | kralji | silent or announced |
| 4 | kralj ultimo | silent or announced |
| 5 | pagat ultimo | silent or announced |
| 6 | valat | ±250 silent, ±500 announced; when it applies, parts 0–5 are zero; the mondfang of part 7 is still written (D125) |
| 7 | mondfang | the individual −20 |
| 8 | klop | the entries of a klop deal (zero until flow C of the engine) |
| 9 | radelc | the +70 of a cancelled radelc in training (zero until flow C) |

Kontra and radelc doubling multiply the parts they apply to. The engine emits `reward_parts`: for each seat and part, ( own part − mean of the other three seats' part ) / 50. The ten numbers of a seat add up to its reward. The decomposed value head (D15) and the critic predict these ten numbers.

## 9. Versioning

- `ENCODING_VERSION = 1` is stored in every checkpoint, weight file and game record.
- Adding a meaning to a reserved slot keeps the version. Moving or resizing anything raises it.
- Amended on 2026-11-11, before any model was trained on it: ten score parts and the `reward_parts` array (D121). The version stays 1.
- Amended on 2026-11-12, before any model was trained on it: the readings of section 4a (D130) and the valat row of section 8 (D125). The version stays 1.
- Amended on 2026-11-24 with deal flow B (spec 009): the readings of section 4b. Tokens 146 to 156, slots 77 to 88, the phase entry 5 and block D1 and D2 are in use. The version stays 1.
- The Rust engine exports the sizes (288, 1382, 160, 128, and 10 score parts) as constants; a test on each side checks them against this file.

## 10. Answers to the open points (D117)

- A. Reward: the zero-sum relative score of section 8.
- B. A win that cancels a radelc adds 70 to the declarer's entry in training.
- C. Self-play tables play runs of 24 deals; radelci carry over within a run and reset with it. The radelci inputs (G3) and the doubling arrive with flow C of the engine; until then they are zero.
- D. The state features are as listed in section 4.
- E. Reward scale 50 and sequence limit 128.
