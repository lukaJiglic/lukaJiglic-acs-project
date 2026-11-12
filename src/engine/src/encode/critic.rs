//! What only training may see: the critic's hidden features and the two
//! auxiliary targets (`docs/encoding.md` sections 6 and 7).
//!
//! These read the full view of the game. The actor never gets them.

use crate::cardset::CardSet;
use crate::deal::Seat;
use crate::game::Game;

use super::{CRITIC_EXTRA, PARTNER_NONE, relative};

const OTHER_HANDS: usize = 0;
const TALON_NOT_PUBLIC: usize = 162;
const HIDDEN_DISCARDS: usize = 216;
const TRUE_PARTNER: usize = 270;

fn set_cards(out: &mut [f32], offset: usize, cards: CardSet) {
    for card in cards {
        out[offset + usize::from(card.index())] = 1.0;
    }
}

/// Writes the 274 hidden features that follow the state in the critic's
/// input:
///
/// - 162: the hands of the three other seats (next, across, previous),
/// - 54: the talon cards that are not public yet (all six until the talon
///   is turned up, none after),
/// - 54: the declarer's hidden discards,
/// - 4: the seat of the declarer's true partner in the observer's
///   numbering (index 0 if the observer is the partner); zero if there is
///   no partner.
///
/// `out` must hold at least [`CRITIC_EXTRA`] entries; a shorter slice is
/// left untouched.
pub fn write_critic_extra(game: &Game, observer: Seat, out: &mut [f32]) {
    let Some(out) = out.get_mut(..CRITIC_EXTRA) else {
        return;
    };
    out.fill(0.0);
    let full = game.full();
    let public = game.public();
    for steps in 1..4 {
        set_cards(
            out,
            OTHER_HANDS + 54 * (steps - 1),
            full.hand(observer.plus(steps)),
        );
    }
    if public.talon().is_none() {
        set_cards(out, TALON_NOT_PUBLIC, full.talon().into_iter().collect());
    }
    set_cards(
        out,
        HIDDEN_DISCARDS,
        full.discards() - public.shown_discards(),
    );
    // Reading B12-2 (Batch 12, open): the "true partner seat" is the
    // declarer's partner, relative to the observer, like block B6. Empty
    // when the declarer plays alone.
    if let Some(partner) = full.partner() {
        out[TRUE_PARTNER + relative(observer, partner)] = 1.0;
    }
}

/// The two auxiliary targets of `observer`.
///
/// **Partner seat**: the relative seat, 1 to 3, of the one other seat on
/// the observer's side. [`PARTNER_NONE`] (0) when there is no such single
/// seat: before the teams are fixed, for a declarer who plays alone, and
/// for each of three opponents of a lone declarer.
///
/// **Card location**, one number per card: 1 to 3 if that other seat holds
/// the card, 0 for every other place (the observer's hand, played cards,
/// discards, the talon). The loss counts unseen cards only (block F4).
pub fn targets(game: &Game, observer: Seat) -> (u8, [u8; 54]) {
    let full = game.full();
    let public = game.public();

    let teams_fixed = public
        .contract()
        .is_some_and(|contract| !contract.calls_king() || public.called_suit().is_some());
    // Reading B12-3 (Batch 12, open): "none" unless exactly one other seat
    // is on the observer's side. An opponent in a two-against-two game has
    // the other opponent as its class.
    let mut partner_target = PARTNER_NONE;
    if teams_fixed {
        let my_side = full.on_declarer_side(observer);
        let mut mates = (1..4u8)
            .filter(|&steps| full.on_declarer_side(observer.plus(usize::from(steps))) == my_side);
        if let (Some(only), None) = (mates.next(), mates.next()) {
            partner_target = only;
        }
    }

    let mut card_target = [0u8; 54];
    for steps in 1..4u8 {
        for card in full.hand(observer.plus(usize::from(steps))) {
            card_target[usize::from(card.index())] = steps;
        }
    }
    (partner_target, card_target)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encode::CRITIC_EXTRA_BLOCKS;

    #[test]
    fn offsets_equal_the_exported_blocks() {
        let mut offset = 0;
        let mut offsets = Vec::new();
        for (_, width) in CRITIC_EXTRA_BLOCKS {
            offsets.push(offset);
            offset += width;
        }
        assert_eq!(
            offsets,
            [OTHER_HANDS, TALON_NOT_PUBLIC, HIDDEN_DISCARDS, TRUE_PARTNER]
        );
        assert_eq!(offset, CRITIC_EXTRA);
    }
}
