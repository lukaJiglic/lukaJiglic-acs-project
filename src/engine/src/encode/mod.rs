//! The encoding of `docs/encoding.md`, version 1: what the network sees and
//! what it can do.
//!
//! For one seat (the observer) at one moment of a deal:
//!
//! - `tokens`: the public events so far, five numbers each ([`tokens`]).
//! - `state`: 1,382 features in 0 to 1 ([`state`], with the exact inference
//!   bits of block F in [`inference`]).
//! - `mask`: which of the 160 action slots are legal ([`write_mask`]).
//! - `critic`: the state and 274 hidden features, for training only
//!   ([`critic`]).
//! - the two auxiliary targets, for training only ([`critic::targets`]).
//!
//! And at the end of a deal the reward of each seat and its ten parts
//! ([`reward`]).
//!
//! Tokens, state and mask read the public view and the observer's private
//! view of the game only. The critic features and the targets read the full
//! view. Rules that are not built yet (announcements, kontra, klop, berač,
//! radelci) leave their features zero and their slots masked.
//!
//! The sizes below are the contract. `tests/encoding_contract.rs` reads the
//! document and compares.

pub mod critic;
pub mod inference;
pub mod reward;
pub mod state;
pub mod tokens;

use crate::deal::Seat;
use crate::game::Game;

pub use critic::{targets, write_critic_extra};
pub use reward::{Rewards, rewards};
pub use state::write_state;
pub use tokens::{event_id, write_tokens};

/// The version of `docs/encoding.md` this module implements.
pub const ENCODING_VERSION: u32 = 1;

/// Event ids of the token vocabulary.
pub const N_EVENTS: usize = 288;
/// Features of the state token.
pub const STATE_FEATURES: usize = 1382;
/// Action slots.
pub const N_ACTIONS: usize = 160;
/// The sequence limit.
pub const MAX_TOKENS: usize = 128;
/// Numbers per token: event, seat, trick, position in trick, index.
pub const TOKEN_FIELDS: usize = 5;
/// How many values each token field can take.
pub const TOKEN_FIELD_SIZES: [usize; TOKEN_FIELDS] = [N_EVENTS, 5, 13, 5, MAX_TOKENS];
/// The seat field of an event without a seat.
pub const NO_SEAT: i16 = 4;
/// Hidden features that follow the state in the critic's input.
pub const CRITIC_EXTRA: usize = 274;
/// Length of the critic's input.
pub const CRITIC_FEATURES: usize = STATE_FEATURES + CRITIC_EXTRA;
/// Parts of an entry (D121).
pub const N_SCORE_PARTS: usize = 10;
/// A reward is a difference of entries divided by this.
pub const REWARD_SCALE: f32 = 50.0;

/// The event vocabulary: name, first id, count.
pub const EVENT_GROUPS: [(&str, usize, usize); 15] = [
    ("PAD", 0, 1),
    ("BOS", 1, 1),
    ("BID", 2, 13),
    ("CONTRACT", 15, 13),
    ("KING", 28, 4),
    ("TALON_SHOWN", 32, 54),
    ("TALON_TAKEN", 86, 6),
    ("DISCARD_SHOWN", 92, 54),
    ("ANNOUNCE", 146, 5),
    ("KONTRA", 151, 6),
    ("ANNOUNCE_DONE", 157, 1),
    ("ANNOUNCE_BARVNI_VALAT", 158, 1),
    ("PLAY", 159, 54),
    ("KLOP_TALON", 213, 54),
    ("RESERVED", 267, 21),
];

/// The blocks of the state token: name, offset, width.
pub const STATE_BLOCKS: [(&str, usize, usize); 37] = [
    ("A1", 0, 54),
    ("A2", 54, 216),
    ("A3", 270, 162),
    ("A4", 432, 54),
    ("A5", 486, 54),
    ("A6", 540, 54),
    ("A7", 594, 54),
    ("A8", 648, 54),
    ("B1", 702, 8),
    ("B2", 710, 13),
    ("B3", 723, 4),
    ("B4", 727, 4),
    ("B5", 731, 4),
    ("B6", 735, 4),
    ("B7", 739, 4),
    ("B8", 743, 8),
    ("B9", 751, 4),
    ("C1", 755, 13),
    ("C2", 768, 4),
    ("C3", 772, 4),
    ("C4", 776, 52),
    ("D1", 828, 10),
    ("D2", 838, 30),
    ("D3", 868, 1),
    ("E1", 869, 4),
    ("E2", 873, 4),
    ("E3", 877, 4),
    ("E4", 881, 2),
    ("F1", 883, 15),
    ("F2", 898, 162),
    ("F3", 1060, 162),
    ("F4", 1222, 54),
    ("F5", 1276, 54),
    ("F6", 1330, 16),
    ("G1", 1346, 19),
    ("G2", 1365, 13),
    ("G3", 1378, 4),
];

/// The action slots: name, first slot, count.
pub const ACTION_GROUPS: [(&str, usize, usize); 10] = [
    ("BID", 0, 13),
    ("KING", 13, 4),
    ("TALON", 17, 6),
    ("DISCARD", 23, 54),
    ("ANNOUNCE", 77, 5),
    ("KONTRA", 82, 6),
    ("ANNOUNCE_DONE", 88, 1),
    ("ANNOUNCE_BARVNI_VALAT", 89, 1),
    ("PLAY", 90, 54),
    ("RESERVED", 144, 16),
];

/// The critic's hidden features: name, width.
pub const CRITIC_EXTRA_BLOCKS: [(&str, usize); 4] = [
    ("other hands", 162),
    ("talon not public", 54),
    ("hidden discards", 54),
    ("true partner seat", 4),
];

/// The parts of an entry, in the contract's order.
pub const SCORE_PARTS: [&str; N_SCORE_PARTS] = [
    "game",
    "difference",
    "trula",
    "kralji",
    "kralj ultimo",
    "pagat ultimo",
    "valat",
    "mondfang",
    "klop",
    "radelc",
];

/// The partner target of a seat with no single team-mate.
pub const PARTNER_NONE: u8 = 0;

/// The preset the engine plays. Toggles arrive with a later spec.
pub const PRESET: &str = "default";

/// The scoring toggles of block G1 (D47): name, the option of the Default
/// preset (0 is option a), the number of options. One-hot each, in this
/// order.
///
/// Reading B12-1 (Batch 12, open): before the toggles exist G1 holds the
/// Default preset's values, not zeros.
pub const SCORING_TOGGLES: [(&str, usize, usize); 9] = [
    ("S15", 0, 2),
    ("S22", 0, 2),
    ("S25", 0, 2),
    ("S26", 0, 2),
    ("S28", 1, 2),
    ("S29", 0, 2),
    ("S31", 0, 2),
    ("S32", 0, 3),
    ("S33", 0, 2),
];

/// The seat number of `seat` as `observer` sees it: 0 self, 1 the next seat
/// to play, 2 the seat across, 3 the previous seat.
pub const fn relative(observer: Seat, seat: Seat) -> usize {
    (seat.index() + 4 - observer.index()) % 4
}

/// Everything the encoding gives for one seat at one moment, in vectors of
/// its own. Handy for tests and for single games; the vectorised
/// environment writes into its big arrays with the `write_` functions.
#[derive(Clone, Debug, PartialEq)]
pub struct Observation {
    /// The seat that looks.
    pub seat: Seat,
    /// One row per public event: event, relative seat, trick, position,
    /// index.
    pub tokens: Vec<[i16; TOKEN_FIELDS]>,
    /// The 1,382 state features.
    pub state: Vec<f32>,
    /// The 160 mask bits. All false if the seat is not the one to act.
    pub mask: Vec<bool>,
    /// The state followed by the 274 hidden features.
    pub critic: Vec<f32>,
    /// The relative seat (1 to 3) of the one seat on the observer's side,
    /// or [`PARTNER_NONE`].
    pub partner_target: u8,
    /// Per card: 1 to 3 if that other seat holds it, else 0.
    pub card_target: [u8; 54],
}

/// The whole observation of a seat.
pub fn observe(game: &Game, observer: Seat) -> Observation {
    let mut flat = [0i16; MAX_TOKENS * TOKEN_FIELDS];
    let count = write_tokens(game, observer, &mut flat);
    let tokens = flat
        .chunks_exact(TOKEN_FIELDS)
        .take(count)
        .map(|row| [row[0], row[1], row[2], row[3], row[4]])
        .collect();
    let mut critic = vec![0.0; CRITIC_FEATURES];
    let (state, extra) = critic.split_at_mut(STATE_FEATURES);
    write_state(game, observer, state);
    write_critic_extra(game, observer, extra);
    let mut mask = vec![false; N_ACTIONS];
    write_mask(game, observer, &mut mask);
    let (partner_target, card_target) = targets(game, observer);
    Observation {
        seat: observer,
        tokens,
        state: critic[..STATE_FEATURES].to_vec(),
        mask,
        critic,
        partner_target,
        card_target,
    }
}

/// Writes the 160 mask bits: true for every slot that is a legal action of
/// the observer now. All false if the observer is not the seat to act.
///
/// `out` must hold [`N_ACTIONS`] entries; a longer slice gets its first 160
/// written, a shorter one is filled as far as it goes.
pub fn write_mask(game: &Game, observer: Seat, out: &mut [bool]) {
    let legal = if game.to_act() == Some(observer) {
        game.legal_actions()
    } else {
        crate::action::ActionSet::EMPTY
    };
    for (slot, bit) in out.iter_mut().take(N_ACTIONS).enumerate() {
        *bit = legal.contains_slot(slot);
    }
}
