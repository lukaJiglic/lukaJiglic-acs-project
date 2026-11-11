//! The talon exchange: packets and discards.
//!
//! Rules source: spec 005 criteria 17-21 and research/01 B3.
//!
//! - The six talon cards are shown in packets that follow the talon's
//!   order: two packets of three in tri and solo tri, three of two in dve
//!   and solo dve, six of one in ena and solo ena.
//! - The declarer takes one packet and then puts down the same number of
//!   cards. Kings and the trula (škis, mond, pagat) may never be put down.
//!   Other taroks may (S14 a); a discarded tarok is shown to all.

use crate::action::Packet;
use crate::card::{Card, Rank, Suit};
use crate::cardset::CardSet;

/// Cards in the talon.
pub const TALON_SIZE: usize = 6;

/// The cards that may never be discarded: the four kings and the trula.
pub const NEVER_DISCARDED: CardSet = CardSet::EMPTY
    .union(CardSet::single(Card::court(Suit::Srce, Rank::Kralj)))
    .union(CardSet::single(Card::court(Suit::Karo, Rank::Kralj)))
    .union(CardSet::single(Card::court(Suit::Pik, Rank::Kralj)))
    .union(CardSet::single(Card::court(Suit::Kriz, Rank::Kralj)))
    .union(CardSet::single(Card::SKIS))
    .union(CardSet::single(Card::MOND))
    .union(CardSet::single(Card::PAGAT));

/// The trula: škis, mond and pagat.
pub const TRULA: CardSet = CardSet::single(Card::SKIS)
    .union(CardSet::single(Card::MOND))
    .union(CardSet::single(Card::PAGAT));

/// The four kings.
pub const KINGS: CardSet = NEVER_DISCARDED.difference(TRULA);

/// How many packets a talon has for a packet size of 3, 2 or 1. Zero for
/// a size of 0.
pub const fn packet_count(size: usize) -> usize {
    if size == 0 { 0 } else { TALON_SIZE / size }
}

/// The cards of a packet, in the talon's order. `None` if the talon has no
/// packet with this index at this packet size.
pub fn packet_cards(talon: &[Card; TALON_SIZE], size: usize, packet: Packet) -> Option<&[Card]> {
    let first = packet.index().checked_mul(size)?;
    talon.get(first..first.checked_add(size)?)
}

/// The cards of a hand that may be discarded.
pub const fn discardable(hand: CardSet) -> CardSet {
    hand.difference(NEVER_DISCARDED)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seven_cards_are_never_discarded() {
        assert_eq!(NEVER_DISCARDED.len(), 7);
        assert_eq!(TRULA.len(), 3);
        assert_eq!(KINGS.len(), 4);
        assert!(KINGS.iter().all(|c| c.value() == 5 && !c.is_tarok()));
        assert_eq!(discardable(CardSet::FULL).len(), 47);
    }

    #[test]
    fn packets_cut_the_talon_in_order() {
        let talon: [Card; 6] = [10, 20, 30, 40, 50, 5].map(|i| Card::from_index(i).unwrap());
        let packet = |i| Packet::new(i).unwrap();
        assert_eq!(packet_cards(&talon, 3, packet(1)), Some(&talon[3..6]));
        assert_eq!(packet_cards(&talon, 3, packet(2)), None);
        assert_eq!(packet_cards(&talon, 2, packet(2)), Some(&talon[4..6]));
        assert_eq!(packet_cards(&talon, 2, packet(3)), None);
        assert_eq!(packet_cards(&talon, 1, packet(5)), Some(&talon[5..6]));
        assert_eq!(
            (packet_count(3), packet_count(2), packet_count(1)),
            (2, 3, 6)
        );
        // Sizes no contract has: no panic.
        assert_eq!(packet_count(0), 0);
        assert_eq!(packet_cards(&talon, 0, packet(0)), Some(&talon[0..0]));
        assert_eq!(packet_cards(&talon, 7, packet(0)), None);
        assert_eq!(packet_cards(&talon, usize::MAX, packet(5)), None);
    }
}
