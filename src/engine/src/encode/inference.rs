//! What a seat knows for certain about the cards it cannot see: the basis
//! of block F of the state (`docs/encoding.md` section 4, D30).
//!
//! Only facts that follow with certainty from what the seat has seen. No
//! probabilities. The rules, all of them sound under legal play:
//!
//! **Voids** (public, the same for every observer). A seat that does not
//! follow a led suit is void in that suit. If it plays no tarok either, it
//! is void in taroks. A seat that plays no tarok on a led tarok is void in
//! taroks.
//!
//! **Seen cards**: the own hand, the own discards, every played card, the
//! talon once it is turned up, the discards shown to all. Everything else
//! is unseen.
//!
//! **Cannot hold.** Another seat cannot hold a card that is
//! - in the observer's hand or among its own discards,
//! - played, a shown discard, or a talon card nobody took,
//! - a card of the taken packet, unless that seat is the declarer,
//! - of a suit (or the taroks) in which the seat is void,
//! - certainly held by someone else (below).
//!
//! **Certainly holds.**
//! - The declarer holds a king or a trula card of the packet it took until
//!   it is played: those cards may not be put down. Once the discards are
//!   made it also holds a taken tarok that was not shown as a discard, and
//!   every taken card if no discard was hidden.
//! - By elimination: an unseen card that only one other seat can hold is in
//!   that seat's hand, provided the card cannot lie elsewhere. It could lie
//!   in the talon before the talon is turned up. It could be a hidden
//!   discard if the observer is not the declarer, a discard was hidden and
//!   the card is neither a tarok nor a king.

use crate::card::{Card, Suit};
use crate::cardset::CardSet;
use crate::deal::Seat;
use crate::game::{Game, Phase};
use crate::talon::NEVER_DISCARDED;

/// Index of the taroks in a list of five "suits": srce, karo, pik, križ,
/// taroks.
pub const TAROKS: usize = 4;

/// What `observer` knows for certain. Arrays are indexed by seat number;
/// the observer's own entries are empty.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Knowledge {
    /// The cards the observer has not seen.
    pub unseen: CardSet,
    /// Shown voids of each seat: srce, karo, pik, križ, taroks.
    pub voids: [[bool; 5]; 4],
    /// The cards each other seat cannot hold.
    pub cannot_hold: [CardSet; 4],
    /// The cards each other seat certainly holds.
    pub certainly_holds: [CardSet; 4],
}

/// The cards of "suit" number `index`: srce, karo, pik, križ, taroks.
pub const fn suit_cards(index: usize) -> CardSet {
    match index {
        0 => CardSet::suit(Suit::Srce),
        1 => CardSet::suit(Suit::Karo),
        2 => CardSet::suit(Suit::Pik),
        3 => CardSet::suit(Suit::Kriz),
        _ => CardSet::TAROKS,
    }
}

/// The voids every seat has shown so far by not following.
pub fn shown_voids(game: &Game) -> [[bool; 5]; 4] {
    let public = game.public();
    let mut voids = [[false; 5]; 4];
    let mut look = |leader: Seat, cards: &[Card]| {
        let Some(&led) = cards.first() else {
            return;
        };
        for (position, card) in cards.iter().enumerate().skip(1) {
            let seat = leader.plus(position).index();
            match led.suit() {
                Some(suit) => {
                    if card.suit() != Some(suit) {
                        voids[seat][suit.index() as usize] = true;
                        if !card.is_tarok() {
                            voids[seat][TAROKS] = true;
                        }
                    }
                }
                None => {
                    if !card.is_tarok() {
                        voids[seat][TAROKS] = true;
                    }
                }
            }
        }
    };
    for trick in public.tricks() {
        look(trick.leader, &trick.cards);
    }
    if let Some(leader) = public.leader() {
        look(leader, public.current_trick());
    }
    voids
}

/// Everything `observer` knows for certain at this moment. It reads the
/// public view and the observer's private view only.
pub fn knowledge(game: &Game, observer: Seat) -> Knowledge {
    let public = game.public();
    let private = game.private(observer);
    let hand = private.hand();
    let own_discards = private.discards();

    let mut played = CardSet::EMPTY;
    for trick in public.tricks() {
        for card in trick.cards {
            played.insert(card);
        }
    }
    for &card in public.current_trick() {
        played.insert(card);
    }
    let talon: CardSet = match public.talon() {
        Some(cards) => cards.into_iter().collect(),
        None => CardSet::EMPTY,
    };
    let talon_shown = public.talon().is_some();
    let taken = public.taken_cards();
    let shown_discards = public.shown_discards();
    // Reading B12-10 (Batch 12, open): the whole shown talon counts as
    // seen, the packet the declarer took included, and so do the
    // declarer's own discards. F4 is therefore 0 for a taken card, and the
    // card-location loss (unseen cards only) never covers one, although
    // the other seats do not know whether it is in the declarer's hand or
    // among the hidden discards.
    let seen = hand | own_discards | played | talon | shown_discards;
    let unseen = seen.complement();

    let voids = shown_voids(game);
    let declarer = public.declarer();
    let i_am_declarer = declarer == Some(observer);
    let discards_done = matches!(public.phase(), Phase::Play | Phase::Over);
    let hidden_discards = public.discards_made() - shown_discards.len() as usize;

    // Cards no other seat holds, whoever it is.
    let nobody = hand | own_discards | played | shown_discards | (talon - taken);
    let mut cannot_hold = [CardSet::EMPTY; 4];
    for seat in Seat::ALL {
        if seat == observer {
            continue;
        }
        let mut cannot = nobody;
        for (index, &void) in voids[seat.index()].iter().enumerate() {
            if void {
                cannot = cannot | suit_cards(index);
            }
        }
        if declarer != Some(seat) {
            cannot = cannot | taken;
        }
        cannot_hold[seat.index()] = cannot;
    }

    let mut certainly_holds = [CardSet::EMPTY; 4];
    // The taken packet in the declarer's hand.
    // Reading B12-5 (Batch 12, open): "certainly holds" has three sources.
    // A taken king or trula card, until it is played. Once the discards
    // are made, a taken tarok that was not shown as a discard, and every
    // taken card if no discard was hidden. And elimination, below.
    if let Some(declarer) = declarer
        && !i_am_declarer
    {
        for card in taken - played - shown_discards {
            let kept = NEVER_DISCARDED.contains(card)
                || (discards_done && (card.is_tarok() || hidden_discards == 0));
            if kept {
                certainly_holds[declarer.index()].insert(card);
            }
        }
    }
    // Elimination. Before the talon is turned up an unseen card may lie in
    // it, so nothing follows.
    if talon_shown {
        for card in unseen {
            let may_be_hidden_discard = !i_am_declarer
                && hidden_discards > 0
                && !card.is_tarok()
                && !NEVER_DISCARDED.contains(card);
            if may_be_hidden_discard {
                continue;
            }
            let mut holders = Seat::ALL
                .into_iter()
                .filter(|&seat| seat != observer && !cannot_hold[seat.index()].contains(card));
            if let (Some(only), None) = (holders.next(), holders.next()) {
                certainly_holds[only.index()].insert(card);
            }
        }
    }
    // A card one seat certainly holds, the others cannot hold.
    for holder in Seat::ALL {
        for seat in Seat::ALL {
            if seat != holder && seat != observer {
                cannot_hold[seat.index()] =
                    cannot_hold[seat.index()] | certainly_holds[holder.index()];
            }
        }
    }

    Knowledge {
        unseen,
        voids,
        cannot_hold,
        certainly_holds,
    }
}

/// The cards of `hand` that no unseen card of their suit beats. Taroks
/// count as one suit. Within a suit a higher index is a stronger card.
///
/// Reading B12-4 (Batch 12, open): every such card gets the bit, not only
/// the single highest of the hand.
pub fn highest_unseen(hand: CardSet, unseen: CardSet) -> CardSet {
    let mut highest = CardSet::EMPTY;
    for index in 0..5 {
        let suit = suit_cards(index);
        // The strongest unseen card of the suit, if any.
        let top_unseen = (unseen & suit).iter().last();
        for card in hand & suit {
            if top_unseen.is_none_or(|top| card.index() > top.index()) {
                highest.insert(card);
            }
        }
    }
    highest
}
