//! Card play of one deal: twelve tricks of a normal contract.
//!
//! `PlayState` knows the four hands, who leads, the cards on the table and
//! the pile each seat has won. It does not know contracts or teams; those
//! come with later specs.

use std::fmt;

use crate::card::Card;
use crate::cardset::CardSet;
use crate::deal::{Deal, Seat};
use crate::{points, trick};

/// Tricks in a deal.
pub const TRICKS: usize = 12;

/// A finished trick.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Trick {
    /// The seat that led.
    pub leader: Seat,
    /// The four cards in the order of play. `cards[0]` is the led card;
    /// `cards[i]` was played by `leader.plus(i)`.
    pub cards: [Card; 4],
    /// The seat that won the trick.
    pub winner: Seat,
}

/// Why a card or a set of hands was refused.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PlayError {
    /// All twelve tricks are played.
    DealOver,
    /// The player to move does not hold the card.
    NotInHand(Card),
    /// The player holds the card but must follow the led card with another.
    NotLegal(Card),
    /// The hands given to [`PlayState::new`] are not four separate hands of
    /// twelve cards.
    InvalidHands,
}

impl fmt::Display for PlayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlayError::DealOver => write!(f, "the deal is over"),
            PlayError::NotInHand(card) => write!(f, "{card} is not in the hand of the player"),
            PlayError::NotLegal(card) => write!(f, "{card} does not follow the led card"),
            PlayError::InvalidHands => write!(f, "the hands are not four hands of twelve cards"),
        }
    }
}

impl std::error::Error for PlayError {}

/// The state of the card play. It holds no heap data, so a clone is cheap.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlayState {
    hands: [CardSet; 4],
    piles: [CardSet; 4],
    /// The seat that led the current trick.
    leader: Seat,
    /// Cards on the table. Only the first `on_table` entries count.
    table: [Card; 4],
    on_table: usize,
    /// Finished tricks. Only the first `tricks_done` entries count.
    history: [Trick; TRICKS],
    tricks_done: usize,
}

/// Fills the unused entries of `table` and `history`. Always the same value,
/// so two equal states compare equal.
const NO_CARD: Card = Card::PAGAT;

impl PlayState {
    /// Starts the play. Each hand must hold twelve cards and no card may be
    /// in two hands. `leader` leads the first trick.
    pub fn new(hands: [CardSet; 4], leader: Seat) -> Result<PlayState, PlayError> {
        let mut seen = CardSet::EMPTY;
        for hand in hands {
            if hand.len() as usize != TRICKS || !seen.is_disjoint(hand) {
                return Err(PlayError::InvalidHands);
            }
            seen = seen | hand;
        }
        let no_trick = Trick {
            leader,
            cards: [NO_CARD; 4],
            winner: leader,
        };
        Ok(PlayState {
            hands,
            piles: [CardSet::EMPTY; 4],
            leader,
            table: [NO_CARD; 4],
            on_table: 0,
            history: [no_trick; TRICKS],
            tricks_done: 0,
        })
    }

    /// Starts the play with the hands of a deal. The talon stays out.
    ///
    /// This cannot fail: a [`Deal`] only comes from [`Deal::from_seed`], so
    /// it always holds four separate hands of twelve.
    pub fn from_deal(deal: &Deal, leader: Seat) -> PlayState {
        PlayState::new(deal.hands(), leader).expect("a deal has four separate hands of twelve")
    }

    /// The seat that led the current trick. After a finished trick this is
    /// its winner.
    pub fn leader(&self) -> Seat {
        self.leader
    }

    /// The seat whose turn it is. After the last trick: its winner.
    pub fn to_move(&self) -> Seat {
        self.leader.plus(self.on_table)
    }

    /// The cards a seat still holds.
    pub fn hand(&self, seat: Seat) -> CardSet {
        self.hands[seat.index()]
    }

    /// The cards of the tricks a seat has won.
    pub fn pile(&self, seat: Seat) -> CardSet {
        self.piles[seat.index()]
    }

    /// The cards on the table, in the order of play. Empty before a lead.
    pub fn current_trick(&self) -> &[Card] {
        &self.table[..self.on_table]
    }

    /// The finished tricks, first to last.
    pub fn trick_history(&self) -> &[Trick] {
        &self.history[..self.tricks_done]
    }

    /// True after twelve tricks.
    pub fn is_over(&self) -> bool {
        self.tricks_done == TRICKS
    }

    /// The cards the player to move may play. Empty when the deal is over.
    pub fn legal_cards(&self) -> CardSet {
        let hand = self.hands[self.to_move().index()];
        trick::legal_cards(hand, self.current_trick().first().copied())
    }

    /// The player to move plays a card. On an error nothing changes.
    pub fn play(&mut self, card: Card) -> Result<(), PlayError> {
        if self.is_over() {
            return Err(PlayError::DealOver);
        }
        let seat = self.to_move();
        if !self.hands[seat.index()].contains(card) {
            return Err(PlayError::NotInHand(card));
        }
        if !self.legal_cards().contains(card) {
            return Err(PlayError::NotLegal(card));
        }

        self.hands[seat.index()].remove(card);
        self.table[self.on_table] = card;
        self.on_table += 1;
        if self.on_table == 4 {
            self.finish_trick();
        }
        Ok(())
    }

    /// Card points of a seat's pile, counted as one pile (see
    /// [`points::count`]). Rounding is per pile: to count a side of two
    /// seats, count the union of their piles, not the sum of two calls.
    pub fn points(&self, seat: Seat) -> u8 {
        points::count(self.piles[seat.index()])
    }

    /// Gives the four cards on the table to the winner, who leads next.
    fn finish_trick(&mut self) {
        let cards = self.table;
        let winner = self.leader.plus(trick::winner(&cards));
        for card in cards {
            self.piles[winner.index()].insert(card);
        }
        self.history[self.tricks_done] = Trick {
            leader: self.leader,
            cards,
            winner,
        };
        self.tricks_done += 1;
        self.leader = winner;
        self.table = [NO_CARD; 4];
        self.on_table = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_refuses_bad_hands() {
        let leader = Seat::ALL[0];
        let deal = Deal::from_seed(1);
        assert!(PlayState::new(deal.hands(), leader).is_ok());

        // A hand of eleven cards.
        let mut short = deal.hands();
        let card = short[2].iter().next().unwrap();
        short[2].remove(card);
        assert_eq!(PlayState::new(short, leader), Err(PlayError::InvalidHands));

        // The same card in two hands.
        let mut twice = short;
        let other = twice[1].iter().next().unwrap();
        twice[2].insert(other);
        assert_eq!(PlayState::new(twice, leader), Err(PlayError::InvalidHands));
    }
}
