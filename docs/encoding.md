# Encoding contract

Status: **draft 1, 2026-11-10. Waiting for the user's approval** (spec `docs/specs/004-encoding-contract.md`). Encoding version: **1**.

This document fixes what the network sees and what it can do. The Rust engine produces these arrays; the Python learner consumes them. Both tracks build against this file (D109). It covers **every phase and every Default-preset contract from the start** (D107). Slots for things not built yet are reserved and masked, so a checkpoint stays loadable when they arrive.

Decisions behind it: D15, D18, D19, D21, D30, D47, D107, D110. Sources for the starting point: `docs/planning/research/06` section (g) and `research/05` sections C2–C4. Everything beyond those is Claude's proposal for the user to correct. Items marked * are pinned off in the first release (D44).

The sequence layout (how the state token attends to the other tokens) is chosen after spec 003 measures both options (D110). This contract is the same for both.

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

At the end of the deal it gives one reward per seat (section 8).

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
| 157 | 1 | `ANNOUNCE_DONE` | a seat ends its announcements |
| 158 | 1 | `ANNOUNCE_BARVNI_VALAT`* | reserved (S7 a) |
| 159–212 | 54 | `PLAY_card` | a card played to a trick |
| 213–266 | 54 | `KLOP_TALON_card` | a talon card added to a trick in klop |
| 267–287 | 21 | reserved | for later toggles |

Longest sequence, counted by hand: 1 start + up to about 24 bids + 1 contract + 1 king + 6 talon cards + 1 packet + up to 3 shown discards + up to about 16 announcement events + 48 plays + 6 klop talon cards. The limit is 128. An engine test must confirm that no legal deal exceeds it.

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
- In the announcement phase a seat may take several actions (announce, kontra) and then `ANNOUNCE_DONE`. The exact turn order is fixed in the announcements spec.
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

Proposal (open point A): a zero-sum form of the paper score.

- Let `e` be the four entries the deal would write on paper under the rules in force: declarer and partner the same score, opponents 0, individual entries for klop and mondfang, radelc doubling applied.
- Reward of seat i = ( e_i − mean of the other three e ) / 50.
- It is given once, at the end of the deal. No per-trick shaping (research/05 C4).

Why not the raw paper score: opponents always write 0, so they would get no signal for defending. The relative form pays them the negative of what the declarer's side gains.

The decomposed value head (D15) predicts the parts of `e` separately: game and difference, each bonus, mondfang.

## 9. Versioning

- `ENCODING_VERSION = 1` is stored in every checkpoint, weight file and game record.
- Adding a meaning to a reserved slot keeps the version. Moving or resizing anything raises it.
- The Rust engine exports the sizes (288, 1382, 160, 128) as constants; a test on each side checks them against this file.

## 10. Open points for the user

- **A. Reward form.** (a) the zero-sum relative score above, recommended; (b) "pay" model: each opponent gets minus the declarer's score; (c) raw paper entries.
- **B. Value of cancelling a radelc.** Under your table's rule a leftover radelc costs 70 at the end (D44, D88), so winning as declarer with a radelc is worth 70 more than the paper shows. (a) add +70 to the declarer's entry when a win cancels a radelc, recommended, since it changes how boldly one should bid; (b) ignore it in training.
- **C. Radelci during self-play.** (a) tables play runs of deals and radelci carry over as in a real session; a table is reset after 24 deals (Claude's number), recommended; (b) radelci drawn at random per deal; (c) radelci off in training.
- **D. State features.** The list in section 4 is your design (D18, D30) written out. Add or remove anything?
- **E. Reward scale 50 and sequence limit 128** are Claude's numbers. Both are easy to change before the first run and expensive after.
