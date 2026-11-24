//! What every seat knows about the sides, worked out a second time from the
//! public events alone (spec 009, criteria 10, 14 and 15 with D147, D148
//! and D150). The engine's own answer is `PublicView::partner` and
//! `PublicView::side_of`; the tests compare the two and hold both against
//! the truth of the deal.
//!
//! Only what was said is read here: who announced what and who raised
//! what, in order. No hand is looked at.

use tarok_engine::{Bonus, EventKind, Game, PublicPartner, Seat};

/// What the round of announcements has shown so far.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Known {
    /// The seat that was shown to hold the called king: by kralj ultimo,
    /// or by a kontra only the holder (or only the declarer's side) could
    /// say.
    pub holder: Option<Seat>,
    /// The seats shown as opponents by their own kontra or sub on the game
    /// or on a bonus of the declarer (D150).
    pub opponents: [bool; 4],
}

impl Known {
    /// All three other seats are shown as opponents.
    fn declarer_is_alone(&self, declarer: Option<Seat>) -> bool {
        Seat::ALL
            .into_iter()
            .all(|seat| Some(seat) == declarer || self.opponents[seat.index()])
    }

    /// What all know about the partner, as long as the called king has not
    /// been played.
    pub fn partner(&self, game: &Game) -> PublicPartner {
        let public = game.public();
        let Some(contract) = public.contract() else {
            return PublicPartner::NotKnown;
        };
        if !contract.calls_king() || public.king_in_talon() {
            return PublicPartner::Alone;
        }
        let declarer = public.declarer();
        match self.holder {
            Some(seat) if Some(seat) == declarer => PublicPartner::Alone,
            Some(seat) => PublicPartner::With(seat),
            None if self.declarer_is_alone(declarer) => PublicPartner::Alone,
            None => PublicPartner::NotKnown,
        }
    }

    /// What all know about the side of `seat`: `Some(true)` the
    /// declarer's, `Some(false)` the opponents', `None` not known.
    pub fn side_of(&self, game: &Game, seat: Seat) -> Option<bool> {
        if game.public().declarer() == Some(seat) {
            return Some(true);
        }
        match self.partner(game) {
            PublicPartner::With(partner) => Some(seat == partner),
            PublicPartner::Alone => Some(false),
            PublicPartner::NotKnown => self.opponents[seat.index()].then_some(false),
        }
    }
}

/// Reads the announcements and kontras of the deal in order.
pub fn known_from_events(game: &Game) -> Known {
    let public = game.public();
    let declarer = public.declarer();
    let mut known = Known {
        holder: None,
        opponents: [false; 4],
    };
    let mut levels = [0u8; 6];
    let mut announcers: [Option<Seat>; 5] = [None; 5];
    for event in public.events() {
        let Some(seat) = event.seat else { continue };
        match event.kind {
            EventKind::Announce(bonus) => {
                announcers[bonus.index()] = Some(seat);
                // Only the holder of the called king may announce it.
                if bonus == Bonus::KraljUltimo {
                    known.holder = Some(seat);
                }
            }
            EventKind::Kontra(target) => {
                let level = levels[target.index()];
                levels[target.index()] += 1;
                // Once the sides are public nothing more can be learnt.
                if known.holder.is_some() || known.declarer_is_alone(declarer) {
                    continue;
                }
                // Kontra (the first level) and sub (the third) are said
                // against the owner, re and mort for it.
                let for_the_owner = level % 2 == 1;
                // Whose the target is, as every seat knows it.
                let announcer = target
                    .bonus()
                    .map(|bonus| announcers[bonus.index()].expect("an announced bonus"));
                let owner = match announcer {
                    // The game is the declarer's.
                    None => Some(true),
                    Some(announcer) if Some(announcer) == declarer => Some(true),
                    Some(announcer) if known.opponents[announcer.index()] => Some(false),
                    Some(_) => None,
                };
                let by_declarer = Some(seat) == declarer;
                match owner {
                    // A bonus of a seat whose side is not public: only a
                    // holder of the called king knows whose it is.
                    None => known.holder = Some(seat),
                    // The declarer's side is known; its raise shows nothing.
                    Some(_) if by_declarer => {}
                    // For the declarer's side, by another seat: the partner.
                    Some(declarers) if declarers == for_the_owner => known.holder = Some(seat),
                    // Against the declarer's side: an opponent (D150).
                    Some(true) => known.opponents[seat.index()] = true,
                    // For the opponents, on a bonus of a seat shown as an
                    // opponent: the seat is an opponent too, but the
                    // engine does not record that (not built, D150).
                    Some(false) => {}
                }
            }
            _ => {}
        }
    }
    known
}
