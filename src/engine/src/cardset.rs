//! A set of cards as a 64-bit mask. Bit `i` is the card with index `i`.

use std::fmt;
use std::ops::{BitAnd, BitOr, Sub};

use crate::card::{Card, Suit};

/// A set of cards. Copying it is copying one `u64`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct CardSet(u64);

impl CardSet {
    /// No cards.
    pub const EMPTY: CardSet = CardSet(0);
    /// All 54 cards.
    pub const FULL: CardSet = CardSet((1 << Card::COUNT) - 1);
    /// The 22 taroks: I to XXI and the škis.
    pub const TAROKS: CardSet = CardSet((1 << 22) - 1);

    /// The eight cards of a suit.
    pub const fn suit(suit: Suit) -> CardSet {
        CardSet(0xFF << (22 + 8 * suit.index()))
    }

    /// The set holding one card.
    pub const fn single(card: Card) -> CardSet {
        CardSet(1 << card.index())
    }

    /// The mask. Bit `i` is the card with index `i`.
    pub const fn bits(self) -> u64 {
        self.0
    }

    /// The set for a mask, or `None` if a bit above 53 is set.
    pub const fn from_bits(bits: u64) -> Option<CardSet> {
        if bits & !CardSet::FULL.0 == 0 {
            Some(CardSet(bits))
        } else {
            None
        }
    }

    /// Adds the card. Returns true if it was not in the set.
    pub const fn insert(&mut self, card: Card) -> bool {
        let added = !self.contains(card);
        self.0 |= 1 << card.index();
        added
    }

    /// Removes the card. Returns true if it was in the set.
    pub const fn remove(&mut self, card: Card) -> bool {
        let removed = self.contains(card);
        self.0 &= !(1 << card.index());
        removed
    }

    pub const fn contains(self, card: Card) -> bool {
        self.0 >> card.index() & 1 == 1
    }

    /// Number of cards in the set.
    pub const fn len(self) -> u32 {
        self.0.count_ones()
    }

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Cards in either set.
    pub const fn union(self, other: CardSet) -> CardSet {
        CardSet(self.0 | other.0)
    }

    /// Cards in both sets.
    pub const fn intersection(self, other: CardSet) -> CardSet {
        CardSet(self.0 & other.0)
    }

    /// Cards in this set and not in the other.
    pub const fn difference(self, other: CardSet) -> CardSet {
        CardSet(self.0 & !other.0)
    }

    /// The cards of the deck that are not in this set.
    pub const fn complement(self) -> CardSet {
        CardSet(!self.0 & CardSet::FULL.0)
    }

    pub const fn is_subset(self, other: CardSet) -> bool {
        self.0 & !other.0 == 0
    }

    pub const fn is_disjoint(self, other: CardSet) -> bool {
        self.0 & other.0 == 0
    }

    /// The cards in index order.
    pub const fn iter(self) -> Iter {
        Iter(self.0)
    }

    /// The card at position `n` (from 0) of the index order, or `None` if
    /// the set has `n` cards or fewer.
    pub fn nth(self, n: u32) -> Option<Card> {
        if n >= self.len() {
            return None;
        }
        let mut bits = self.0;
        for _ in 0..n {
            // Clears the lowest set bit.
            bits &= bits.wrapping_sub(1);
        }
        lowest(bits)
    }
}

/// The card of the lowest set bit.
fn lowest(bits: u64) -> Option<Card> {
    // A `CardSet` never has a bit above 53, so the index is a card.
    Card::from_index(bits.trailing_zeros() as u8)
}

/// Iterator over a [`CardSet`] in index order.
#[derive(Clone, Debug)]
pub struct Iter(u64);

impl Iterator for Iter {
    type Item = Card;

    fn next(&mut self) -> Option<Card> {
        let card = lowest(self.0)?;
        self.0 &= self.0 - 1;
        Some(card)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let left = self.0.count_ones() as usize;
        (left, Some(left))
    }
}

impl ExactSizeIterator for Iter {}

impl IntoIterator for CardSet {
    type Item = Card;
    type IntoIter = Iter;

    fn into_iter(self) -> Iter {
        self.iter()
    }
}

impl FromIterator<Card> for CardSet {
    fn from_iter<I: IntoIterator<Item = Card>>(cards: I) -> CardSet {
        let mut set = CardSet::EMPTY;
        for card in cards {
            set.insert(card);
        }
        set
    }
}

/// Union.
impl BitOr for CardSet {
    type Output = CardSet;

    fn bitor(self, other: CardSet) -> CardSet {
        self.union(other)
    }
}

/// Intersection.
impl BitAnd for CardSet {
    type Output = CardSet;

    fn bitand(self, other: CardSet) -> CardSet {
        self.intersection(other)
    }
}

/// Difference.
impl Sub for CardSet {
    type Output = CardSet;

    fn sub(self, other: CardSet) -> CardSet {
        self.difference(other)
    }
}

impl fmt::Debug for CardSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(self.iter()).finish()
    }
}
