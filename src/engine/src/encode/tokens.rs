//! Tokens: one per public event, five numbers each (`docs/encoding.md`
//! section 3).
//!
//! | Field | Values |
//! |---|---|
//! | event | 0-287, see [`event_id`] |
//! | seat | 0-3 relative to the observer, 4 for an event without a seat |
//! | trick | 0 before card play, then 1-12 |
//! | position in trick | 0 outside card play, then 1-4 |
//! | index | the position in the sequence, from 0 |

use crate::action::Action;
use crate::deal::Seat;
use crate::game::{EventKind, Game};

use super::{MAX_TOKENS, NO_SEAT, TOKEN_FIELDS, relative};

const BOS: usize = 1;
const BID: usize = 2;
const CONTRACT: usize = 15;
const KING: usize = 28;
const TALON_SHOWN: usize = 32;
const TALON_TAKEN: usize = 86;
const DISCARD_SHOWN: usize = 92;
const PLAY: usize = 159;

/// The event id of a public event.
///
/// A bid and a contract use the thirteen values of the bid slots: 0 is the
/// pass, then the twelve contracts in bidding order.
pub const fn event_id(kind: EventKind) -> usize {
    match kind {
        EventKind::Start => BOS,
        EventKind::Bid(None) => BID,
        EventKind::Bid(Some(contract)) => BID + Action::Bid(contract).slot(),
        EventKind::Contract(contract) => CONTRACT + Action::Bid(contract).slot(),
        EventKind::King(suit) => KING + suit.index() as usize,
        EventKind::TalonShown(card) => TALON_SHOWN + card.index() as usize,
        EventKind::TalonTaken(packet) => TALON_TAKEN + packet.index(),
        EventKind::DiscardShown(card) => DISCARD_SHOWN + card.index() as usize,
        EventKind::Play(card) => PLAY + card.index() as usize,
    }
}

/// Writes the tokens of the deal so far as `observer` sees them and returns
/// their number. Token `i` takes the five entries from `5 * i`; the rest of
/// the first [`MAX_TOKENS`] rows is set to zero (padding).
///
/// Every seat sees the same events; only the seat field differs. `out`
/// should hold `MAX_TOKENS * TOKEN_FIELDS` entries; rows that do not fit
/// are left out.
pub fn write_tokens(game: &Game, observer: Seat, out: &mut [i16]) -> usize {
    let events = game.public().events();
    let rows = out.chunks_exact_mut(TOKEN_FIELDS).take(MAX_TOKENS);
    let mut written = 0;
    for (index, row) in rows.enumerate() {
        match events.get(index) {
            Some(event) => {
                // Every number is small: an id below 288, a seat, a trick
                // up to 12, a position up to 4, an index below 128.
                row[0] = event_id(event.kind) as i16;
                row[1] = match event.seat {
                    Some(seat) => relative(observer, seat) as i16,
                    None => NO_SEAT,
                };
                row[2] = i16::from(event.trick);
                row[3] = i16::from(event.position);
                row[4] = index as i16;
                written += 1;
            }
            None => row.fill(0),
        }
    }
    written
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::Packet;
    use crate::card::{Card, Suit};
    use crate::contract::Contract;
    use crate::encode::{EVENT_GROUPS, N_EVENTS};

    fn group(name: &str) -> (usize, usize) {
        let (_, first, count) = EVENT_GROUPS
            .into_iter()
            .find(|(group, _, _)| *group == name)
            .unwrap();
        (first, count)
    }

    #[test]
    fn event_ids_lie_in_their_groups() {
        assert_eq!(event_id(EventKind::Start), group("BOS").0);
        assert_eq!(event_id(EventKind::Bid(None)), group("BID").0);
        for (index, contract) in Contract::ALL.into_iter().enumerate() {
            assert_eq!(
                event_id(EventKind::Bid(Some(contract))),
                group("BID").0 + 1 + index
            );
            assert_eq!(
                event_id(EventKind::Contract(contract)),
                group("CONTRACT").0 + 1 + index
            );
        }
        for (index, suit) in Suit::ALL.into_iter().enumerate() {
            assert_eq!(event_id(EventKind::King(suit)), group("KING").0 + index);
        }
        for (index, packet) in Packet::ALL.into_iter().enumerate() {
            assert_eq!(
                event_id(EventKind::TalonTaken(packet)),
                group("TALON_TAKEN").0 + index
            );
        }
        for card in Card::all() {
            let index = usize::from(card.index());
            assert_eq!(
                event_id(EventKind::TalonShown(card)),
                group("TALON_SHOWN").0 + index
            );
            assert_eq!(
                event_id(EventKind::DiscardShown(card)),
                group("DISCARD_SHOWN").0 + index
            );
            assert_eq!(event_id(EventKind::Play(card)), group("PLAY").0 + index);
            assert!(event_id(EventKind::Play(card)) < N_EVENTS);
        }
        // The last id of each group used is inside the group.
        assert_eq!(group("BID").1, 13);
        assert_eq!(group("CONTRACT").1, 13);
    }
}
