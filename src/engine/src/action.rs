//! Actions and their slots.
//!
//! One `Action` type covers all phases of a deal. Every action has a fixed
//! slot in the 160-slot action vector of `docs/encoding.md` section 5:
//!
//! | Slots | Action |
//! |---|---|
//! | 0 | `Pass` (naprej) |
//! | 1-12 | `Bid(contract)`: klop, tri, dve, ena, solo tri, solo dve, solo ena, berač, solo brez, odprti berač, barvni valat, valat. Used in the bidding and for the choice of the final contract |
//! | 13-16 | `CallKing(suit)`: srce, karo, pik, križ |
//! | 17-22 | `TakePacket(packet)`: packet index 0-5 |
//! | 23-76 | `Discard(card)`: 23 + card index |
//! | 77-89 | announcements and kontra (spec 009): no action yet |
//! | 90-143 | `Play(card)`: 90 + card index |
//! | 144-159 | reserved |

use std::fmt;

use crate::card::{Card, Suit};
use crate::cardset::CardSet;
use crate::contract::Contract;

/// The slot ranges of the action vector.
pub mod slots {
    use std::ops::Range;

    /// Pass and the twelve contracts.
    pub const BIDS: Range<usize> = 0..13;
    /// The four suits of a king call.
    pub const KINGS: Range<usize> = 13..17;
    /// The six talon packets.
    pub const PACKETS: Range<usize> = 17..23;
    /// A discard of each card.
    pub const DISCARDS: Range<usize> = 23..77;
    /// A play of each card.
    pub const PLAYS: Range<usize> = 90..144;
}

/// The index of a talon packet, 0 to 5. How many packets there are depends
/// on the contract: two in tri, three in dve, six in ena.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Packet(u8);

impl Packet {
    pub const ALL: [Packet; 6] = [
        Packet(0),
        Packet(1),
        Packet(2),
        Packet(3),
        Packet(4),
        Packet(5),
    ];

    /// The packet with this index, or `None` above 5.
    pub const fn new(index: u8) -> Option<Packet> {
        if index < 6 { Some(Packet(index)) } else { None }
    }

    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

/// Something a player does at a decision.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Action {
    /// "Naprej": no bid.
    Pass,
    /// A bid in the auction, or the declarer naming the contract to play.
    Bid(Contract),
    /// The declarer calls the king of a suit.
    CallKing(Suit),
    /// The declarer takes a talon packet.
    TakePacket(Packet),
    /// The declarer puts one card down.
    Discard(Card),
    /// A card played to a trick.
    Play(Card),
}

impl Action {
    /// Length of the action vector.
    pub const SLOTS: usize = 160;

    /// The slot of this action in the action vector.
    pub const fn slot(self) -> usize {
        match self {
            Action::Pass => 0,
            Action::Bid(contract) => 1 + contract.index(),
            Action::CallKing(suit) => slots::KINGS.start + suit.index() as usize,
            Action::TakePacket(packet) => slots::PACKETS.start + packet.index(),
            Action::Discard(card) => slots::DISCARDS.start + card.index() as usize,
            Action::Play(card) => slots::PLAYS.start + card.index() as usize,
        }
    }

    /// The action of a slot. `None` for the slots that have no action yet
    /// (77-89 and 144-159) and for numbers above 159.
    pub fn from_slot(slot: usize) -> Option<Action> {
        // The ranges are checked first, so the narrowing casts cannot lose
        // anything.
        if slot == 0 {
            Some(Action::Pass)
        } else if slots::BIDS.contains(&slot) {
            Contract::from_index(slot - 1).map(Action::Bid)
        } else if slots::KINGS.contains(&slot) {
            Some(Action::CallKing(Suit::ALL[slot - slots::KINGS.start]))
        } else if slots::PACKETS.contains(&slot) {
            Packet::new((slot - slots::PACKETS.start) as u8).map(Action::TakePacket)
        } else if slots::DISCARDS.contains(&slot) {
            Card::from_index((slot - slots::DISCARDS.start) as u8).map(Action::Discard)
        } else if slots::PLAYS.contains(&slot) {
            Card::from_index((slot - slots::PLAYS.start) as u8).map(Action::Play)
        } else {
            None
        }
    }
}

/// A set of actions: one bit per slot. This is the legality mask of the
/// encoding.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ActionSet([u64; 3]);

impl ActionSet {
    /// No action.
    pub const EMPTY: ActionSet = ActionSet([0; 3]);

    /// Adds the action. Returns true if it was not in the set.
    pub const fn insert(&mut self, action: Action) -> bool {
        let slot = action.slot();
        let added = !self.contains_slot(slot);
        self.0[slot / 64] |= 1 << (slot % 64);
        added
    }

    pub const fn contains(self, action: Action) -> bool {
        self.contains_slot(action.slot())
    }

    /// True if the bit of this slot is set. False for a number above 159.
    pub const fn contains_slot(self, slot: usize) -> bool {
        slot < Action::SLOTS && self.0[slot / 64] >> (slot % 64) & 1 == 1
    }

    /// Number of actions in the set.
    pub const fn len(self) -> u32 {
        self.0[0].count_ones() + self.0[1].count_ones() + self.0[2].count_ones()
    }

    pub const fn is_empty(self) -> bool {
        self.0[0] == 0 && self.0[1] == 0 && self.0[2] == 0
    }

    /// The mask as three words: slot `s` is bit `s % 64` of word `s / 64`.
    pub const fn words(self) -> [u64; 3] {
        self.0
    }

    /// The actions in slot order.
    pub const fn iter(self) -> Iter {
        Iter {
            words: self.0,
            word: 0,
        }
    }

    /// The action at position `n` (from 0) of the slot order, or `None` if
    /// the set has `n` actions or fewer.
    pub fn nth(self, n: u32) -> Option<Action> {
        let mut left = n;
        for (index, &word) in self.0.iter().enumerate() {
            let here = word.count_ones();
            if left < here {
                let mut bits = word;
                for _ in 0..left {
                    // Clears the lowest set bit.
                    bits &= bits - 1;
                }
                return Action::from_slot(index * 64 + bits.trailing_zeros() as usize);
            }
            left -= here;
        }
        None
    }

    /// The set with `Discard(card)` (or `Play(card)`) for each card:
    /// the card bits moved to the slots that start at `first_slot`.
    const fn of_cards(cards: CardSet, first_slot: usize) -> ActionSet {
        // 54 card bits shifted by under 64 fit in 128 bits.
        let wide = (cards.bits() as u128) << (first_slot % 64);
        let mut words = [0u64; 3];
        words[first_slot / 64] = wide as u64;
        words[first_slot / 64 + 1] = (wide >> 64) as u64;
        ActionSet(words)
    }

    /// A discard of each of the cards.
    pub const fn discards(cards: CardSet) -> ActionSet {
        ActionSet::of_cards(cards, slots::DISCARDS.start)
    }

    /// A play of each of the cards.
    pub const fn plays(cards: CardSet) -> ActionSet {
        ActionSet::of_cards(cards, slots::PLAYS.start)
    }
}

/// Iterator over an [`ActionSet`] in slot order.
#[derive(Clone, Debug)]
pub struct Iter {
    words: [u64; 3],
    word: usize,
}

impl Iterator for Iter {
    type Item = Action;

    fn next(&mut self) -> Option<Action> {
        while self.word < 3 {
            let bits = self.words[self.word];
            if bits != 0 {
                self.words[self.word] = bits & (bits - 1);
                let slot = self.word * 64 + bits.trailing_zeros() as usize;
                // Only slots that have an action are ever set.
                return Action::from_slot(slot);
            }
            self.word += 1;
        }
        None
    }
}

impl IntoIterator for ActionSet {
    type Item = Action;
    type IntoIter = Iter;

    fn into_iter(self) -> Iter {
        self.iter()
    }
}

impl FromIterator<Action> for ActionSet {
    fn from_iter<I: IntoIterator<Item = Action>>(actions: I) -> ActionSet {
        let mut set = ActionSet::EMPTY;
        for action in actions {
            set.insert(action);
        }
        set
    }
}

impl fmt::Debug for ActionSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(self.iter()).finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn card_actions_land_on_their_slots() {
        let cards: CardSet = [Card::PAGAT, Card::SKIS, Card::from_index(53).unwrap()]
            .into_iter()
            .collect();
        let discards: Vec<Action> = ActionSet::discards(cards).iter().collect();
        let plays: Vec<Action> = ActionSet::plays(cards).iter().collect();
        let listed: Vec<Card> = cards.iter().collect();
        assert_eq!(
            discards,
            listed
                .iter()
                .map(|&c| Action::Discard(c))
                .collect::<Vec<_>>()
        );
        assert_eq!(
            plays,
            listed.iter().map(|&c| Action::Play(c)).collect::<Vec<_>>()
        );
        // The whole deck: 54 slots each, the right ones.
        let all = ActionSet::plays(CardSet::FULL);
        assert_eq!(all.len(), 54);
        assert!(all.iter().all(|a| slots::PLAYS.contains(&a.slot())));
        let all = ActionSet::discards(CardSet::FULL);
        assert_eq!(all.len(), 54);
        assert!(all.iter().all(|a| slots::DISCARDS.contains(&a.slot())));
    }
}
