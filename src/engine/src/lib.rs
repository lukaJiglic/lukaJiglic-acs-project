//! `tarok-engine`: the rules of four-player Slovenian tarok.
//!
//! This is the first part (spec 002): cards, card sets, card points, the
//! seeded deal, the trick winner and legal cards for normal contracts, and
//! the card play of one deal. Bidding, contracts, the talon, scoring and the
//! bindings come with later specs.
//!
//! Card names are Slovenian without diacritics: `Kralj`, `Dama`, `Kaval`,
//! `Fant`; `PAGAT` (tarok I), `MOND` (tarok XXI), `SKIS`; `Srce`, `Karo`,
//! `Pik`, `Kriz`.
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

pub mod card;
pub mod cardset;
pub mod deal;
pub mod play;
pub mod points;
pub mod trick;

pub use card::{Card, Face, Rank, Suit};
pub use cardset::CardSet;
pub use deal::{Deal, Seat, shuffled_deck};
pub use play::{PlayError, PlayState, Trick};
