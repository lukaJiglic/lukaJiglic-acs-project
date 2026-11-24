//! The state token: 1,382 features in 0 to 1 (`docs/encoding.md`
//! section 4).
//!
//! "Per seat" blocks have four entries in relative seat order: the
//! observer, the next seat, the seat across, the previous seat. "Other
//! seat" blocks have three: next, across, previous.
//!
//! The features read the public view and the observer's private view only.
//! Blocks A8 (klop talon), D (announcements, kontra), G2 and G3 (reserved
//! rule slots, radelci) belong to rules that are not built: they stay zero.

use crate::action::Action;
use crate::card::Card;
use crate::cardset::CardSet;
use crate::contract::Contract;
use crate::deal::Seat;
use crate::game::{Game, Phase, PublicPartner};
use crate::points;
use crate::talon::{KINGS, TRULA};

use super::inference::{self, Knowledge, TAROKS, suit_cards};
use super::{SCORING_TOGGLES, STATE_FEATURES, relative};

// Offsets of the blocks. A unit test compares them with `STATE_BLOCKS`.
const A1: usize = 0;
const A2: usize = 54;
const A3: usize = 270;
const A4: usize = 432;
const A5: usize = 486;
const A6: usize = 540;
const A7: usize = 594;
const B1: usize = 702;
const B2: usize = 710;
const B3: usize = 723;
const B4: usize = 727;
const B5: usize = 731;
const B6: usize = 735;
const B7: usize = 739;
const B8: usize = 743;
const B9: usize = 751;
const C1: usize = 755;
const C2: usize = 768;
const C3: usize = 772;
const C4: usize = 776;
const E1: usize = 869;
const E2: usize = 873;
const E3: usize = 877;
const E4: usize = 881;
const F1: usize = 883;
const F2: usize = 898;
const F3: usize = 1060;
const F4: usize = 1222;
const F5: usize = 1276;
const F6: usize = 1330;
const G1: usize = 1346;

/// Sets the bit of each card, from `offset`.
fn set_cards(out: &mut [f32], offset: usize, cards: CardSet) {
    for card in cards {
        out[offset + usize::from(card.index())] = 1.0;
    }
}

/// The index of a phase in block B1: bidding, contract choice, king call,
/// talon pick, discard, announcements, play, over.
const fn phase_index(phase: Phase) -> usize {
    match phase {
        Phase::Bidding => 0,
        Phase::ContractChoice => 1,
        Phase::KingCall => 2,
        Phase::TalonPick => 3,
        Phase::Discard => 4,
        Phase::Announcements => 5,
        Phase::Play => 6,
        Phase::Over => 7,
    }
}

/// The index of a contract among the thirteen bid values: its bid slot
/// (0 is the pass).
const fn bid_index(contract: Contract) -> usize {
    Action::Bid(contract).slot()
}

/// Writes the 1,382 state features of `observer` into `out`.
///
/// `out` must hold at least [`STATE_FEATURES`] entries; only the first
/// 1,382 are written. A shorter slice is left untouched.
pub fn write_state(game: &Game, observer: Seat, out: &mut [f32]) {
    let Some(out) = out.get_mut(..STATE_FEATURES) else {
        return;
    };
    out.fill(0.0);
    let public = game.public();
    let private = game.private(observer);
    let rel = |seat: Seat| relative(observer, seat);

    // --- A: cards -----------------------------------------------------
    let hand = private.hand();
    set_cards(out, A1, hand);
    for trick in public.tricks() {
        for (position, card) in trick.cards.into_iter().enumerate() {
            let player = rel(trick.leader.plus(position));
            out[A2 + 54 * player + usize::from(card.index())] = 1.0;
        }
    }
    if let Some(leader) = public.leader() {
        for (position, card) in public.current_trick().iter().enumerate() {
            let player = rel(leader.plus(position));
            out[A2 + 54 * player + usize::from(card.index())] = 1.0;
            // A3: the first, second and third card of the current trick.
            // A decision is never taken with four cards on the table.
            if position < 3 {
                out[A3 + 54 * position + usize::from(card.index())] = 1.0;
            }
        }
    }
    if let Some(talon) = public.talon() {
        set_cards(out, A4, talon.into_iter().collect());
    }
    set_cards(out, A5, public.taken_cards());
    set_cards(out, A6, private.discards());
    set_cards(out, A7, public.shown_discards());

    // --- B: contract and roles ------------------------------------------
    out[B1 + phase_index(public.phase())] = 1.0;
    let contract = public.contract();
    if let Some(contract) = contract {
        out[B2 + bid_index(contract)] = 1.0;
    }
    let declarer = public.declarer();
    if let Some(declarer) = declarer {
        out[B3 + rel(declarer)] = 1.0;
    }
    // B4, the own role: declarer, called partner, opponent, no teams. A
    // seat that is not the declarer knows its role once the teams are
    // fixed: with the contract in a solo, with the call otherwise.
    if declarer == Some(observer) {
        out[B4] = 1.0;
    } else if let Some(contract) = contract
        && (!contract.calls_king() || public.called_suit().is_some())
    {
        let role = if private.is_called_partner() { 1 } else { 2 };
        out[B4 + role] = 1.0;
    }
    if let Some(suit) = public.called_suit() {
        out[B5 + suit.index() as usize] = 1.0;
    }
    let partner = public.partner();
    if let PublicPartner::With(seat) = partner {
        out[B6 + rel(seat)] = 1.0;
    }
    // B7: called king played; called king in the shown talon; I hold the
    // called king; the declarer is publicly alone.
    if public.called_king_played() {
        out[B7] = 1.0;
    }
    // Reading B12-11 (Batch 12, open): "the called king lies in the shown
    // talon" is a fact about the deal. It stays 1 after the declarer took
    // the king and after the king was played.
    if public.king_in_talon() {
        out[B7 + 1] = 1.0;
    }
    if public.called_king().is_some_and(|king| hand.contains(king)) {
        out[B7 + 2] = 1.0;
    }
    if partner == PublicPartner::Alone {
        out[B7 + 3] = 1.0;
    }
    out[B8 + rel(public.dealer())] = 1.0;
    out[B8 + 4 + rel(public.forehand())] = 1.0;
    if let Some(leader) = public.leader() {
        out[B9 + rel(leader)] = 1.0;
    }

    // --- C: bidding -----------------------------------------------------
    if let Some((bidder, bid)) = public.highest_bid() {
        out[C1 + bid_index(bid)] = 1.0;
        out[C2 + rel(bidder)] = 1.0;
    }
    for seat in Seat::ALL {
        if public.has_passed(seat) {
            out[C3 + rel(seat)] = 1.0;
        }
    }
    // C4: the last bid of each seat, thirteen entries per seat: the pass
    // (entry 0) and the twelve contracts in ladder order.
    // Reading B12-9 (Batch 12, open): a seat's last real bid, kept after it
    // passes; nothing if it never bid. Entry 0 is therefore never set:
    // block C3 holds the passes.
    let mut last_bid: [Option<usize>; 4] = [None; 4];
    for call in public.bids() {
        if let Some(bid) = call.bid {
            last_bid[rel(call.seat)] = Some(bid_index(bid));
        }
    }
    for (seat, bid) in last_bid.into_iter().enumerate() {
        if let Some(index) = bid {
            out[C4 + 13 * seat + index] = 1.0;
        }
    }

    // --- E: counts --------------------------------------------------------
    let tricks = public.tricks();
    for seat in Seat::ALL {
        let r = rel(seat);
        out[E1 + r] = public.hand_size(seat) as f32 / 15.0;
        let won = tricks.iter().filter(|trick| trick.winner == seat).count();
        out[E2 + r] = won as f32 / 12.0;
        // Reading B12-11 (Batch 12, open): each seat's pile is counted and
        // rounded on its own, so the four need not add up to a side's
        // count.
        out[E3 + r] = f32::from(points::count(public.tricks_won(seat))) / 70.0;
    }
    // Reading B12-11 (Batch 12, open): the trick number is 0 before card
    // play and stays 12/12 once the last trick is over.
    if public.leader().is_some() {
        // The number of the current trick; after the last trick, 12.
        let number = (tricks.len() + 1).min(12);
        out[E4] = number as f32 / 12.0;
        out[E4 + 1] = public.current_trick().len() as f32 / 3.0;
    }

    // --- F: exact inference bits ----------------------------------------
    let known = inference::knowledge(game, observer);
    write_inference(game, observer, &known, out);

    // --- G: rules -----------------------------------------------------------
    let mut offset = G1;
    for (_, chosen, options) in SCORING_TOGGLES {
        out[offset + chosen] = 1.0;
        offset += options;
    }
}

/// Block F from what the observer knows for certain.
fn write_inference(game: &Game, observer: Seat, known: &Knowledge, out: &mut [f32]) {
    let public = game.public();
    let hand = game.private(observer).hand();
    for steps in 1..4 {
        let seat = observer.plus(steps).index();
        let other = steps - 1;
        // F1: shown voids.
        for (index, &void) in known.voids[seat].iter().enumerate() {
            if void {
                out[F1 + 5 * other + index] = 1.0;
            }
        }
        set_cards(out, F2 + 54 * other, known.cannot_hold[seat]);
        set_cards(out, F3 + 54 * other, known.certainly_holds[seat]);
        // F6, last three: the most taroks the seat can still hold, of 12.
        // Reading B12-6 (Batch 12, open): the smaller of its hand size and
        // the taroks it can still hold, capped at 12.
        let possible = (CardSet::TAROKS - known.cannot_hold[seat]).len();
        let most = possible.min(public.hand_size(observer.plus(steps))).min(12);
        out[F6 + 13 + other] = most as f32 / 12.0;
    }
    let unseen = known.unseen;
    set_cards(out, F4, unseen);
    set_cards(out, F5, inference::highest_unseen(hand, unseen));

    // F6: taroks unseen / 22; unseen per suit / 8; king unseen; škis, mond,
    // pagat unseen; card points unseen / 70.
    out[F6] = (unseen & suit_cards(TAROKS)).len() as f32 / 22.0;
    for suit in 0..4 {
        out[F6 + 1 + suit] = (unseen & suit_cards(suit)).len() as f32 / 8.0;
    }
    for (index, king) in KINGS.iter().enumerate() {
        // `KINGS` iterates in card order: srce, karo, pik, križ.
        if unseen.contains(king) {
            out[F6 + 5 + index] = 1.0;
        }
    }
    for (index, card) in [Card::SKIS, Card::MOND, Card::PAGAT]
        .into_iter()
        .enumerate()
    {
        debug_assert!(TRULA.contains(card));
        if unseen.contains(card) {
            out[F6 + 9 + index] = 1.0;
        }
    }
    // Reading B12-6 (Batch 12, open): the unseen cards counted as one pile.
    out[F6 + 12] = f32::from(points::count(unseen)) / 70.0;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encode::STATE_BLOCKS;

    #[test]
    fn offsets_equal_the_exported_blocks() {
        let named = [
            ("A1", A1),
            ("A2", A2),
            ("A3", A3),
            ("A4", A4),
            ("A5", A5),
            ("A6", A6),
            ("A7", A7),
            ("B1", B1),
            ("B2", B2),
            ("B3", B3),
            ("B4", B4),
            ("B5", B5),
            ("B6", B6),
            ("B7", B7),
            ("B8", B8),
            ("B9", B9),
            ("C1", C1),
            ("C2", C2),
            ("C3", C3),
            ("C4", C4),
            ("E1", E1),
            ("E2", E2),
            ("E3", E3),
            ("E4", E4),
            ("F1", F1),
            ("F2", F2),
            ("F3", F3),
            ("F4", F4),
            ("F5", F5),
            ("F6", F6),
            ("G1", G1),
        ];
        for (name, offset) in named {
            let (_, exported, _) = STATE_BLOCKS
                .into_iter()
                .find(|(block, _, _)| *block == name)
                .unwrap();
            assert_eq!(offset, exported, "{name}");
        }
        // The toggles fill G1 exactly.
        let width: usize = SCORING_TOGGLES.iter().map(|toggle| toggle.2).sum();
        assert_eq!(width, 19);
    }
}
