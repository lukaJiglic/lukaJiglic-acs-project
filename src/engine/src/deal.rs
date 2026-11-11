//! Seats and the seeded deal.
//!
//! The shuffle is written here and not taken from a library, so the deal for
//! a seed cannot change with a library upgrade. Only the ChaCha8 block
//! function comes from `rand_chacha` (pinned to an exact version). All
//! arithmetic is on `u32`, so native, WASM and Python builds give the same
//! deal.
//!
//! Steps for a seed:
//! 1. ChaCha8 key: the seed as 8 little-endian bytes, then 24 zero bytes.
//!    Stream 0, from the start.
//! 2. The deck starts in index order, 0 to 53.
//! 3. Fisher-Yates from the top: for `i` = 53 down to 1, draw `j` uniformly
//!    from 0 to `i` and swap positions `i` and `j`.
//! 4. A draw below `n` takes 32-bit words from the stream, skips words under
//!    `2^32 mod n` and returns the first kept word modulo `n`. No bias.
//! 5. Positions 0-5 are the talon. Then packets of six go to seats 0, 1, 2,
//!    3 and again 0, 1, 2, 3 (talon first, packets of six: research/01 B3).

use std::fmt;

use rand_chacha::ChaCha8Rng;
use rand_chacha::rand_core::{Rng, SeedableRng};

use crate::card::Card;
use crate::cardset::CardSet;

/// A seat at the table, 0 to 3. The next seat in turn is +1 modulo 4.
/// Which way that runs on screen is a display matter (D91).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Seat(u8);

impl Seat {
    pub const ALL: [Seat; 4] = [Seat(0), Seat(1), Seat(2), Seat(3)];

    /// The seat with this number, or `None` above 3.
    pub const fn new(index: u8) -> Option<Seat> {
        if index < 4 { Some(Seat(index)) } else { None }
    }

    /// The seat number, usable as an array index.
    pub const fn index(self) -> usize {
        self.0 as usize
    }

    /// The seat that plays after this one.
    pub const fn next(self) -> Seat {
        self.plus(1)
    }

    /// The seat `steps` turns after this one. Any number of steps is fine.
    pub const fn plus(self, steps: usize) -> Seat {
        // `steps` is reduced first, so the sum cannot overflow.
        Seat(((self.0 as usize + steps % 4) % 4) as u8)
    }
}

/// Twelve cards for each seat and six for the talon: all 54 cards, each
/// once.
///
/// The fields are private, so a `Deal` can only come from
/// [`Deal::from_seed`] or from [`Deal::new`], which checks the cards. A
/// `Deal` is always whole.
///
/// ```compile_fail
/// use tarok_engine::{Card, CardSet, Deal};
/// // Does not compile: a deal cannot be put together by hand.
/// let deal = Deal { hands: [CardSet::EMPTY; 4], talon: [Card::PAGAT; 6] };
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Deal {
    hands: [CardSet; 4],
    talon: [Card; 6],
}

/// The cards given to [`Deal::new`] are not a whole deal.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DealError {
    /// Not four hands of twelve and six talon cards with every card of the
    /// deck exactly once.
    NotWhole,
}

impl fmt::Display for DealError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("not four hands of twelve and a talon of six with every card once")
    }
}

impl std::error::Error for DealError {}

impl Deal {
    /// A deal of chosen cards, for tests and written examples. Each hand
    /// must hold twelve cards, and the hands and the talon together every
    /// card of the deck exactly once.
    pub fn new(hands: [CardSet; 4], talon: [Card; 6]) -> Result<Deal, DealError> {
        let mut seen = CardSet::EMPTY;
        for hand in hands {
            if hand.len() != 12 || !seen.is_disjoint(hand) {
                return Err(DealError::NotWhole);
            }
            seen = seen | hand;
        }
        for card in talon {
            if !seen.insert(card) {
                return Err(DealError::NotWhole);
            }
        }
        // 48 + 6 different cards are the whole deck.
        Ok(Deal { hands, talon })
    }

    /// The deal for a seed. The same seed always gives the same deal.
    pub fn from_seed(seed: u64) -> Deal {
        let deck = shuffled_deck(seed);
        let talon = [deck[0], deck[1], deck[2], deck[3], deck[4], deck[5]];
        let mut hands = [CardSet::EMPTY; 4];
        for (packet, cards) in deck[6..].chunks_exact(6).enumerate() {
            for &card in cards {
                hands[packet % 4].insert(card);
            }
        }
        Deal { hands, talon }
    }

    /// The hand of each seat.
    pub const fn hands(&self) -> [CardSet; 4] {
        self.hands
    }

    /// The hand of one seat.
    pub const fn hand(&self, seat: Seat) -> CardSet {
        self.hands[seat.index()]
    }

    /// The talon in the order it lies. The order matters when the talon is
    /// split into packets (a later spec).
    pub const fn talon(&self) -> [Card; 6] {
        self.talon
    }

    /// The talon as a set.
    pub fn talon_set(&self) -> CardSet {
        self.talon.iter().copied().collect()
    }
}

/// The 54 cards in the shuffled order for a seed. [`Deal::from_seed`] deals
/// from this order.
pub fn shuffled_deck(seed: u64) -> [Card; 54] {
    let mut key = [0u8; 32];
    key[..8].copy_from_slice(&seed.to_le_bytes());
    let mut stream = ChaCha8Rng::from_seed(key);

    let mut deck = [Card::PAGAT; Card::COUNT];
    for (slot, card) in deck.iter_mut().zip(Card::all()) {
        *slot = card;
    }
    for i in (1..Card::COUNT).rev() {
        let j = below(|| stream.next_u32(), i as u32 + 1) as usize;
        deck.swap(i, j);
    }
    deck
}

/// A uniform draw from `0..n` (`n` at least 1). `next_word` gives the next
/// 32-bit word of the stream. It is a parameter so a test can feed chosen
/// words.
fn below(mut next_word: impl FnMut() -> u32, n: u32) -> u32 {
    // `2^32 mod n`. Words under it are skipped; the 2^32 - skip words that
    // are left are a whole multiple of `n`, so the remainder is uniform.
    let skip = n.wrapping_neg() % n;
    loop {
        let word = next_word();
        if word >= skip {
            return word % n;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seats_go_round() {
        let seat = |i| Seat::new(i).unwrap();
        assert_eq!(seat(0).next(), seat(1));
        assert_eq!(seat(3).next(), seat(0));
        assert_eq!(seat(2).plus(3), seat(1));
        assert_eq!(seat(1).plus(0), seat(1));
        assert_eq!(Seat::new(4), None);
        assert_eq!(Seat::ALL.map(Seat::index), [0, 1, 2, 3]);
    }

    #[test]
    fn seat_plus_takes_any_number_of_steps() {
        let seat = |i| Seat::new(i).unwrap();
        // usize::MAX is 3 modulo 4 on 32-bit and 64-bit targets.
        assert_eq!(usize::MAX % 4, 3);
        for start in 0..4u8 {
            assert_eq!(seat(start).plus(usize::MAX), seat((start + 3) % 4));
            assert_eq!(seat(start).plus(usize::MAX - 3), seat(start));
            assert_eq!(seat(start).plus(4_000_000), seat(start));
        }
    }

    #[test]
    fn draws_stay_below_the_bound() {
        let mut stream = ChaCha8Rng::from_seed([7; 32]);
        for n in 1..=54 {
            for _ in 0..200 {
                assert!(below(|| stream.next_u32(), n) < n);
            }
        }
    }

    /// Feeds `words` to one draw below `n`. Returns the draw and how many
    /// words it used.
    fn draw_from(words: &[u32], n: u32) -> (u32, usize) {
        let mut used = 0;
        let drawn = below(
            || {
                let word = words[used];
                used += 1;
                word
            },
            n,
        );
        (drawn, used)
    }

    #[test]
    fn draw_skips_words_under_the_threshold() {
        // The threshold 2^32 mod n is computed here in 64 bits, apart from
        // the code under test.
        for n in 2..=54u32 {
            let skip = ((1u64 << 32) % u64::from(n)) as u32;
            if skip == 0 {
                // Only for a power of two. Nothing is skipped: word 0 is kept.
                assert!(n.is_power_of_two());
                assert_eq!(draw_from(&[0], n), (0, 1));
                continue;
            }
            // The word `skip - 1` is rejected and the next word is used.
            assert_eq!(draw_from(&[skip - 1, skip], n), (skip % n, 2), "n = {n}");
            assert_eq!(draw_from(&[0, skip - 1, u32::MAX], n), (u32::MAX % n, 3));
            // The word `skip` is kept.
            assert_eq!(draw_from(&[skip], n), (skip % n, 1), "n = {n}");
        }
        // The first draw of the shuffle is below 54, and 2^32 mod 54 is 22.
        assert_eq!(draw_from(&[21, 22], 54), (22, 2));
        assert_eq!(draw_from(&[22], 54), (22, 1));
    }
}
