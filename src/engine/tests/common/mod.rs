//! Helpers shared by the integration tests of spec 002.
//!
//! Every expected value in the tests is written from the spec text
//! (`docs/specs/002-engine-core.md`), not derived from engine internals.

#![allow(dead_code)]

use proptest::strategy::Strategy;
use proptest::test_runner::{Config, RngAlgorithm, TestCaseError, TestRng, TestRunner};
use tarok_engine::{Card, CardSet, Rank, Seat, Suit};

/// Runs a property test: `cases` random values of `strategy` go through
/// `test`. A failing value is shrunk and printed by proptest.
///
/// proptest is built without its "std" feature (see `Cargo.toml`), so it
/// cannot seed itself from the system and does not catch panics. This
/// helper does both:
///
/// - The seed comes from the clock, so every run tries new cases. It is
///   printed before the first case. `cargo test` shows that line for every
///   failing test, whatever the failure is. To replay a run, set the
///   environment variable `TAROK_TEST_SEED` to the printed seed.
/// - A panic inside a case (an `unwrap` in the test, an `expect` or an index
///   out of bounds in the engine) is caught and handed to proptest as a
///   failed case, so the failing value is shrunk and printed too.
pub fn run_cases<S: Strategy>(
    cases: u32,
    strategy: S,
    test: impl Fn(S::Value) -> Result<(), TestCaseError>,
) {
    let seed = run_seed();
    eprintln!("property test: {cases} cases, seed {seed} (replay: TAROK_TEST_SEED={seed})");
    let mut bytes = [0u8; 32];
    bytes[..8].copy_from_slice(&seed.to_le_bytes());
    let rng = TestRng::from_seed(RngAlgorithm::ChaCha, &bytes);
    let mut runner = TestRunner::new_with_rng(Config::with_cases(cases), rng);

    let catching = |value: S::Value| {
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| test(value)));
        outcome.unwrap_or_else(|panic| {
            let message = panic
                .downcast_ref::<String>()
                .map(String::as_str)
                .or_else(|| panic.downcast_ref::<&str>().copied())
                .unwrap_or("a panic without a text message");
            Err(TestCaseError::fail(format!("panic in the case: {message}")))
        })
    };
    if let Err(failure) = runner.run(&strategy, catching) {
        panic!("{failure}\nseed of this run: {seed} (replay: TAROK_TEST_SEED={seed})");
    }
}

fn run_seed() -> u64 {
    if let Ok(text) = std::env::var("TAROK_TEST_SEED") {
        return text.parse().expect("TAROK_TEST_SEED is a whole number");
    }
    let since_epoch = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("the clock is after 1970");
    since_epoch.as_nanos() as u64
}

/// Tarok by its number, 1 (pagat) to 21 (mond).
pub fn tarok(number: u8) -> Card {
    Card::tarok(number).expect("tarok number 1-21")
}

/// Pip card by the number printed on it (1-4 red, 7-10 black).
pub fn pip(suit: Suit, number: u8) -> Card {
    Card::pip(suit, number).expect("pip number valid for the suit")
}

pub fn court(suit: Suit, rank: Rank) -> Card {
    Card::court(suit, rank)
}

pub fn kralj(suit: Suit) -> Card {
    Card::court(suit, Rank::Kralj)
}

pub fn seat(index: u8) -> Seat {
    Seat::new(index).expect("seat 0-3")
}

pub const SUITS: [Suit; 4] = [Suit::Srce, Suit::Karo, Suit::Pik, Suit::Kriz];

/// The eight cards of a suit from the strongest to the weakest, as the rules
/// source states it (research/01 B1): red suits kralj, dama, kaval, fant,
/// 1, 2, 3, 4; black suits kralj, dama, kaval, fant, 10, 9, 8, 7.
pub fn suit_high_to_low(suit: Suit) -> [Card; 8] {
    let pips: [u8; 4] = match suit {
        Suit::Srce | Suit::Karo => [1, 2, 3, 4],
        Suit::Pik | Suit::Kriz => [10, 9, 8, 7],
    };
    [
        court(suit, Rank::Kralj),
        court(suit, Rank::Dama),
        court(suit, Rank::Kaval),
        court(suit, Rank::Fant),
        pip(suit, pips[0]),
        pip(suit, pips[1]),
        pip(suit, pips[2]),
        pip(suit, pips[3]),
    ]
}

/// All 22 taroks from the weakest to the strongest: I to XXI, then the škis.
pub fn taroks_low_to_high() -> Vec<Card> {
    let mut taroks: Vec<Card> = (1..=21).map(tarok).collect();
    taroks.push(Card::SKIS);
    taroks
}

/// All 32 suit cards.
pub fn suit_cards() -> Vec<Card> {
    SUITS.iter().flat_map(|&s| suit_high_to_low(s)).collect()
}

/// The winner of a trick, written a second time for the tests and apart
/// from the engine: it ranks cards by the two order lists above
/// (`suit_high_to_low`, `taroks_low_to_high`), not by card index. Returns the
/// position (0-3) in the order of play.
pub fn reference_winner(cards: [Card; 4]) -> usize {
    let position_of = |wanted: Card| cards.iter().position(|&c| c == wanted);

    // Škis, mond and pagat in one trick: the pagat wins.
    if let (Some(_), Some(_), Some(pagat)) = (
        position_of(Card::SKIS),
        position_of(Card::MOND),
        position_of(Card::PAGAT),
    ) {
        return pagat;
    }

    // Otherwise the highest tarok.
    let taroks = taroks_low_to_high();
    if let Some(&highest) = taroks.iter().rev().find(|&&t| cards.contains(&t)) {
        return position_of(highest).unwrap();
    }

    // No tarok: the highest card of the led suit. The led card is of that
    // suit, so one is always found.
    let led_suit = SUITS
        .into_iter()
        .find(|&s| suit_high_to_low(s).contains(&cards[0]))
        .expect("without a tarok the led card is a suit card");
    let highest = suit_high_to_low(led_suit)
        .into_iter()
        .find(|c| cards.contains(c))
        .unwrap();
    position_of(highest).unwrap()
}

/// The 24 orders of four things.
pub fn perms4() -> Vec<[usize; 4]> {
    let mut perms = Vec::with_capacity(24);
    for a in 0..4 {
        for b in 0..4 {
            for c in 0..4 {
                for d in 0..4 {
                    let p = [a, b, c, d];
                    let mut seen = [false; 4];
                    for &i in &p {
                        seen[i] = true;
                    }
                    if seen.iter().all(|&s| s) {
                        perms.push(p);
                    }
                }
            }
        }
    }
    assert_eq!(perms.len(), 24);
    perms
}

/// The four cards in every order of play.
pub fn orders(cards: [Card; 4]) -> Vec<[Card; 4]> {
    perms4()
        .into_iter()
        .map(|p| [cards[p[0]], cards[p[1]], cards[p[2]], cards[p[3]]])
        .collect()
}

pub fn set(cards: &[Card]) -> CardSet {
    let set: CardSet = cards.iter().copied().collect();
    assert_eq!(
        set.len() as usize,
        cards.len(),
        "duplicate card in a test set"
    );
    set
}

/// A hand-built deal used by the play tests (criteria 19 and 26).
///
/// - seat 0: srce 4, 3, 2, 1 and all eight karo
/// - seat 1: srce fant, kaval, dama, kralj and all eight pik
/// - seat 2: all eight križ and taroks I, II, XX, XXI
/// - seat 3: taroks V to XVI
///
/// Not dealt: taroks III, IV, XVII, XVIII, XIX and the škis.
pub fn built_hands() -> [CardSet; 4] {
    let mut seat0 = vec![
        pip(Suit::Srce, 4),
        pip(Suit::Srce, 3),
        pip(Suit::Srce, 2),
        pip(Suit::Srce, 1),
    ];
    seat0.extend(suit_high_to_low(Suit::Karo));

    let mut seat1 = vec![
        court(Suit::Srce, Rank::Fant),
        court(Suit::Srce, Rank::Kaval),
        court(Suit::Srce, Rank::Dama),
        court(Suit::Srce, Rank::Kralj),
    ];
    seat1.extend(suit_high_to_low(Suit::Pik));

    let mut seat2 = vec![tarok(1), tarok(2), tarok(20), tarok(21)];
    seat2.extend(suit_high_to_low(Suit::Kriz));

    let seat3: Vec<Card> = (5..=16).map(tarok).collect();

    [set(&seat0), set(&seat1), set(&seat2), set(&seat3)]
}
