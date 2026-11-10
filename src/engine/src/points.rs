//! Card values and the counting of a pile.
//!
//! Rules source: research/01 B1. Cards are counted in groups of three: add
//! the values and subtract 2 per group. A leftover of one or two cards
//! loses 1. That is the same as each card being worth its value minus 2/3,
//! rounded once for the pile. The code counts in thirds of a point and
//! rounds once.

use crate::card::{Card, Face, Rank};
use crate::cardset::CardSet;

/// Points of the whole deck.
pub const TOTAL: u8 = 70;

impl Card {
    /// The value printed in the rules: kralj 5, škis 5, mond 5, pagat 5,
    /// dama 4, kaval 3, fant 2, every other card 1.
    pub const fn value(self) -> u8 {
        match self.face() {
            Face::Skis | Face::Tarok(1 | 21) | Face::Court(_, Rank::Kralj) => 5,
            Face::Court(_, Rank::Dama) => 4,
            Face::Court(_, Rank::Kaval) => 3,
            Face::Court(_, Rank::Fant) => 2,
            Face::Tarok(_) | Face::Pip(..) => 1,
        }
    }
}

/// The pile in thirds of a point, before rounding: each card is worth
/// `3 * value - 2` thirds. The whole deck is 210 thirds.
pub fn thirds(pile: CardSet) -> u16 {
    pile.iter()
        .map(|card| 3 * u16::from(card.value()) - 2)
        .sum()
}

/// Points of a pile, rounded once.
///
/// A pile of 3k cards is a whole number of points. With one card left over
/// the pile is 1/3 above a whole number and rounds down. With two left over
/// it is 2/3 above and rounds up. Both match "a leftover of one or two cards
/// loses 1".
pub fn count(pile: CardSet) -> u8 {
    // At most 210 thirds, so the result fits a `u8`.
    ((thirds(pile) + 1) / 3) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deck_is_210_thirds() {
        assert_eq!(thirds(CardSet::FULL), 210);
        assert_eq!(thirds(CardSet::EMPTY), 0);
    }
}
