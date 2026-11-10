//! The 54 cards of the tarok deck.
//!
//! A card is an index 0-53. The order is fixed (spec 002, frozen by spec 004):
//!
//! | Index | Cards |
//! |---|---|
//! | 0-20 | taroks I to XXI (0 is the pagat, 20 is the mond) |
//! | 21 | škis |
//! | 22-29 | srce: 4, 3, 2, 1, fant, kaval, dama, kralj |
//! | 30-37 | karo: 4, 3, 2, 1, fant, kaval, dama, kralj |
//! | 38-45 | pik: 7, 8, 9, 10, fant, kaval, dama, kralj |
//! | 46-53 | križ: 7, 8, 9, 10, fant, kaval, dama, kralj |
//!
//! Within the taroks and within each suit a higher index is a stronger card.

use std::fmt;

/// The four suits, in index order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Suit {
    Srce,
    Karo,
    Pik,
    Kriz,
}

impl Suit {
    pub const ALL: [Suit; 4] = [Suit::Srce, Suit::Karo, Suit::Pik, Suit::Kriz];

    /// 0 for srce, 1 for karo, 2 for pik, 3 for križ.
    pub const fn index(self) -> u8 {
        self as u8
    }

    /// Srce and karo are red. Their pips run 1 (high) to 4 (low).
    pub const fn is_red(self) -> bool {
        matches!(self, Suit::Srce | Suit::Karo)
    }

    /// Index of the lowest card of the suit.
    const fn first_index(self) -> u8 {
        FIRST_SUIT_INDEX + 8 * self.index()
    }

    const fn name(self) -> &'static str {
        match self {
            Suit::Srce => "srce",
            Suit::Karo => "karo",
            Suit::Pik => "pik",
            Suit::Kriz => "kriz",
        }
    }
}

/// The four court cards of a suit, weakest first.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Rank {
    Fant,
    Kaval,
    Dama,
    Kralj,
}

impl Rank {
    pub const ALL: [Rank; 4] = [Rank::Fant, Rank::Kaval, Rank::Dama, Rank::Kralj];

    const fn name(self) -> &'static str {
        match self {
            Rank::Fant => "fant",
            Rank::Kaval => "kaval",
            Rank::Dama => "dama",
            Rank::Kralj => "kralj",
        }
    }
}

/// What is printed on a card.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Face {
    /// A tarok by its number, 1 (pagat) to 21 (mond).
    Tarok(u8),
    /// The škis, the highest tarok.
    Skis,
    /// A pip card by its number: 1-4 in srce and karo, 7-10 in pik and križ.
    Pip(Suit, u8),
    /// A court card.
    Court(Suit, Rank),
}

const SKIS_INDEX: u8 = 21;
const FIRST_SUIT_INDEX: u8 = 22;

/// One of the 54 cards. Holds the card index.
///
/// `Ord` is the index order. It is not the strength in a trick; see
/// [`crate::trick::winner`] for that.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Card(u8);

impl Card {
    /// Number of cards in the deck.
    pub const COUNT: usize = 54;

    /// Tarok I.
    pub const PAGAT: Card = Card(0);
    /// Tarok XXI.
    pub const MOND: Card = Card(20);
    /// The škis.
    pub const SKIS: Card = Card(SKIS_INDEX);

    /// The card with this index, or `None` above 53.
    pub const fn from_index(index: u8) -> Option<Card> {
        if (index as usize) < Card::COUNT {
            Some(Card(index))
        } else {
            None
        }
    }

    /// The index 0-53.
    pub const fn index(self) -> u8 {
        self.0
    }

    /// Tarok by its number, 1 (pagat) to 21 (mond). The škis is [`Card::SKIS`].
    pub const fn tarok(number: u8) -> Option<Card> {
        if 1 <= number && number <= 21 {
            Some(Card(number - 1))
        } else {
            None
        }
    }

    /// Court card of a suit.
    pub const fn court(suit: Suit, rank: Rank) -> Card {
        Card(suit.first_index() + 4 + rank as u8)
    }

    /// Pip card by the number printed on it: 1-4 in srce and karo, 7-10 in
    /// pik and križ. `None` for any other number.
    pub const fn pip(suit: Suit, number: u8) -> Option<Card> {
        // Red pips rise from 4 to 1, black pips from 7 to 10.
        let strength = if suit.is_red() {
            if 1 <= number && number <= 4 {
                4 - number
            } else {
                return None;
            }
        } else if 7 <= number && number <= 10 {
            number - 7
        } else {
            return None;
        };
        Some(Card(suit.first_index() + strength))
    }

    /// All 54 cards in index order.
    pub fn all() -> impl ExactSizeIterator<Item = Card> + Clone {
        (0..Card::COUNT as u8).map(Card)
    }

    /// True for taroks I-XXI and the škis.
    pub const fn is_tarok(self) -> bool {
        self.0 <= SKIS_INDEX
    }

    /// The suit, or `None` for a tarok.
    pub const fn suit(self) -> Option<Suit> {
        if self.is_tarok() {
            return None;
        }
        Some(match (self.0 - FIRST_SUIT_INDEX) / 8 {
            0 => Suit::Srce,
            1 => Suit::Karo,
            2 => Suit::Pik,
            _ => Suit::Kriz,
        })
    }

    /// What is printed on the card.
    pub const fn face(self) -> Face {
        let Some(suit) = self.suit() else {
            return if self.0 == SKIS_INDEX {
                Face::Skis
            } else {
                Face::Tarok(self.0 + 1)
            };
        };
        let strength = (self.0 - FIRST_SUIT_INDEX) % 8;
        match strength {
            0..=3 => Face::Pip(
                suit,
                if suit.is_red() {
                    4 - strength
                } else {
                    7 + strength
                },
            ),
            4 => Face::Court(suit, Rank::Fant),
            5 => Face::Court(suit, Rank::Kaval),
            6 => Face::Court(suit, Rank::Dama),
            _ => Face::Court(suit, Rank::Kralj),
        }
    }
}

const ROMAN: [&str; 21] = [
    "I", "II", "III", "IV", "V", "VI", "VII", "VIII", "IX", "X", "XI", "XII", "XIII", "XIV", "XV",
    "XVI", "XVII", "XVIII", "XIX", "XX", "XXI",
];

/// Short ASCII name: `XXI`, `skis`, `srce kralj`, `pik 7`.
impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.face() {
            Face::Tarok(number) => f.write_str(ROMAN[usize::from(number) - 1]),
            Face::Skis => f.write_str("skis"),
            Face::Pip(suit, number) => write!(f, "{} {}", suit.name(), number),
            Face::Court(suit, rank) => write!(f, "{} {}", suit.name(), rank.name()),
        }
    }
}

impl fmt::Debug for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_names() {
        assert_eq!(Card::PAGAT.to_string(), "I");
        assert_eq!(Card::MOND.to_string(), "XXI");
        assert_eq!(Card::SKIS.to_string(), "skis");
        assert_eq!(
            Card::court(Suit::Srce, Rank::Kralj).to_string(),
            "srce kralj"
        );
        assert_eq!(Card::pip(Suit::Kriz, 10).unwrap().to_string(), "kriz 10");
        assert_eq!(Card::pip(Suit::Karo, 1).unwrap().to_string(), "karo 1");
        assert_eq!(
            format!("{:?}", Card::court(Suit::Pik, Rank::Fant)),
            "pik fant"
        );
    }
}
