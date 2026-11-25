//! `tarok-engine`: the rules of four-player Slovenian tarok.
//!
//! Built so far:
//!
//! - Spec 002: cards, card sets, card points, the seeded deal, the trick
//!   winner and legal cards for normal contracts, the card play of one deal
//!   ([`PlayState`]).
//! - Spec 005 (flow A): a whole deal of the six normal contracts ([`Game`]):
//!   bidding, choice of the contract, king call, talon exchange, twelve
//!   tricks and the score with its parts.
//! - Spec 006: the encoding ([`encode`]), the vectorised environment and
//!   the game record ([`record`]).
//! - Spec 009 (flow B): the round of announcements and kontra between the
//!   talon exchange and the first lead ([`announce`]).
//!
//! Klop, berač and the higher contracts come with spec 010.
//!
//! Card names are Slovenian without diacritics: `Kralj`, `Dama`, `Kaval`,
//! `Fant`; `PAGAT` (tarok I), `MOND` (tarok XXI), `SKIS`; `Srce`, `Karo`,
//! `Pik`, `Kriz`.
//!
//! A whole deal with the lowest legal action at every step. In the round
//! of announcements every seat passes:
//!
//! ```
//! use tarok_engine::{Action, Game, Phase, Seat};
//!
//! // Seat 3 deals, so seat 0 is forehand.
//! let mut game = Game::from_seed(7, Seat::ALL[3]);
//! while game.phase() != Phase::Over {
//!     let seat = game.to_act().unwrap();
//!     let mut action = game.legal_actions().iter().next().unwrap();
//!     if game.phase() == Phase::Announcements {
//!         action = Action::Done;
//!     }
//!     // What the seat may look at: the public state and its own view.
//!     assert!(game.private(seat).hand().len() <= 15);
//!     game.apply(action).unwrap();
//! }
//! let score = game.score().unwrap();
//! assert_eq!(score.declarer_points() + score.opponents_points(), 70);
//! // The entries: declarer and partner the same, opponents 0, and -20 for
//! // a lost mond.
//! let entries = score.entries();
//! assert!(entries.iter().all(|entry| entry % 5 == 0));
//! ```
//!
//! The card play alone, without bidding and scoring:
//!
//! ```
//! use tarok_engine::{Deal, PlayState, Seat, points};
//!
//! let deal = Deal::from_seed(7);
//! let mut state = PlayState::from_deal(&deal, Seat::ALL[0]);
//! while !state.is_over() {
//!     let card = state.legal_cards().iter().next().unwrap();
//!     state.play(card).unwrap();
//! }
//! let piles = Seat::ALL.map(|seat| state.pile(seat));
//! let all = piles[0] | piles[1] | piles[2] | piles[3] | deal.talon_set();
//! assert_eq!(points::count(all), 70);
//! ```

#![forbid(unsafe_code)]

pub mod action;
pub mod announce;
pub mod bidding;
pub mod card;
pub mod cardset;
pub mod contract;
pub mod deal;
pub mod encode;
pub mod game;
pub mod play;
pub mod points;
pub mod record;
pub mod score;
pub mod talon;
pub mod trick;
#[cfg(feature = "vecenv")]
pub mod vecenv;

/// The version of this crate. Game records carry it.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub use action::{Action, ActionSet, Packet};
pub use announce::{Bonus, Target};
pub use bidding::{BidEvent, Bidding};
pub use card::{Card, Face, Rank, Suit};
pub use cardset::CardSet;
pub use contract::Contract;
pub use deal::{Deal, DealError, Seat, shuffled_deck};
pub use game::{
    Event, EventKind, FullView, Game, GameError, Phase, PrivateView, PublicPartner, PublicView,
};
pub use play::{PlayError, PlayState, Trick};
pub use score::{Score, ScoreParts};
