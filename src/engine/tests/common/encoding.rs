//! Helpers for the encoding tests of spec 006.

use std::ops::Range;

use tarok_engine::encode::{self, Observation};
use tarok_engine::{Action, Card, CardSet, Contract, Game, Phase, Seat, Suit};

use super::flow::{A, B, C, D, act, exchange, game, king_in_talon_deal};
use super::{kralj, pip, tarok};

/// The range of a state block by its name in `docs/encoding.md`, e.g. "A2".
pub fn block(name: &str) -> Range<usize> {
    let (_, offset, width) = encode::STATE_BLOCKS
        .into_iter()
        .find(|(block, _, _)| *block == name)
        .unwrap_or_else(|| panic!("no state block {name}"));
    offset..offset + width
}

/// The features of a block.
pub fn features<'a>(observation: &'a Observation, name: &str) -> &'a [f32] {
    &observation.state[block(name)]
}

/// 54 features with a 1 for each of the cards.
pub fn card_bits(cards: impl IntoIterator<Item = Card>) -> Vec<f32> {
    let set: CardSet = cards.into_iter().collect();
    Card::all()
        .map(|card| if set.contains(card) { 1.0 } else { 0.0 })
        .collect()
}

/// `width` features, all zero but a 1 at `index`.
pub fn one_hot(width: usize, index: usize) -> Vec<f32> {
    let mut bits = vec![0.0; width];
    bits[index] = 1.0;
    bits
}

/// The seat number of `seat` as `observer` sees it: 0 self, 1 next, 2
/// across, 3 previous.
pub fn relative(observer: Seat, seat: Seat) -> usize {
    (seat.index() + 4 - observer.index()) % 4
}

/// A position written out by hand, used by several tests.
///
/// The deal is `king_in_talon_deal(false)`; D deals, A is forehand.
///
/// - A: taroks I-XII.
/// - B: srce 4, XIII-XX, škis, srce 1, srce fant.
/// - C: srce 3, srce kaval, srce dama, karo fant, kaval, dama, kralj,
///   pik 7, 8, 9, 10, fant.
/// - D: srce 2, pik kaval, dama, kralj, the eight križ.
/// - Talon: srce kralj, karo 4 | karo 3, XXI | karo 2, karo 1.
///
/// What happened:
///
/// 1. Bidding: B dve, C pass, D pass, A dve (A holds the bid), B pass.
/// 2. A names dve and calls srce. The srce kralj lies in the talon.
/// 3. A takes the first packet (srce kralj, karo 4) and puts down the
///    karo 4 (hidden) and the tarok II (shown).
/// 4. Trick 1: A srce kralj, B srce 4, C srce 3, D srce 2. A wins.
/// 5. Trick 2: A leads the XII, B plays the XIII.
///
/// Now C is to act, with no tarok in hand: any card is legal.
pub fn scripted_position() -> Game {
    let mut game = game(&king_in_talon_deal(false));
    let dve = Action::Bid(Contract::Dve);
    act(&mut game, B, dve);
    act(&mut game, C, Action::Pass);
    act(&mut game, D, Action::Pass);
    act(&mut game, A, dve);
    act(&mut game, B, Action::Pass);
    act(&mut game, A, dve);
    act(&mut game, A, Action::CallKing(Suit::Srce));
    exchange(&mut game, 0, &[pip(Suit::Karo, 4), tarok(2)]);
    for (seat, card) in [
        (A, kralj(Suit::Srce)),
        (B, pip(Suit::Srce, 4)),
        (C, pip(Suit::Srce, 3)),
        (D, pip(Suit::Srce, 2)),
        (A, tarok(12)),
        (B, tarok(13)),
    ] {
        act(&mut game, seat, Action::Play(card));
    }
    assert_eq!(game.phase(), Phase::Play);
    assert_eq!(game.to_act(), Some(C));
    game
}

/// The tokens of an observation as rows of five numbers.
pub fn token_rows(observation: &Observation) -> Vec<[i16; 5]> {
    observation.tokens.clone()
}
