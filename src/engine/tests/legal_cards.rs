//! Spec 002, criteria 20-26: legal cards in normal contracts.
//!
//! `trick::legal_cards(hand, led)` takes the hand of the player to move and
//! the led card (`None` on the lead).

mod common;

use common::{SUITS, built_hands, court, kralj, pip, seat, set, tarok};
use tarok_engine::{Card, CardSet, PlayError, PlayState, Rank, Suit, shuffled_deck, trick};

/// Hands of 1 to 12 cards cut from seeded shuffles. Small hands are often
/// void in a suit or without taroks, so every case below occurs many times.
fn sample_hands() -> Vec<CardSet> {
    let mut hands = Vec::new();
    for seed in 0..300u64 {
        let deck = shuffled_deck(seed);
        let size = 1 + (seed % 12) as usize;
        hands.push(deck[..size].iter().copied().collect());
    }
    hands
}

/// Runs `check` on every sampled hand and every led card that is not in the
/// hand, when `applies` holds. Returns how many cases were checked.
fn for_each_case(
    applies: impl Fn(CardSet, Card) -> bool,
    check: impl Fn(CardSet, Card, CardSet),
) -> u32 {
    let mut cases = 0;
    for hand in sample_hands() {
        for led in Card::all().filter(|&c| !hand.contains(c)) {
            if applies(hand, led) {
                check(hand, led, trick::legal_cards(hand, Some(led)));
                cases += 1;
            }
        }
    }
    cases
}

fn holds_suit(hand: CardSet, suit: Suit) -> bool {
    hand.iter().any(|c| c.suit() == Some(suit))
}

fn holds_tarok(hand: CardSet) -> bool {
    hand.iter().any(Card::is_tarok)
}

fn taroks_of(hand: CardSet) -> CardSet {
    hand.iter().filter(|c| c.is_tarok()).collect()
}

#[test]
fn c20_suit_led_and_held_only_that_suit_is_legal() {
    let hand = set(&[
        pip(Suit::Srce, 4),
        kralj(Suit::Srce),
        pip(Suit::Pik, 7),
        tarok(10),
        Card::SKIS,
    ]);
    let legal = trick::legal_cards(hand, Some(pip(Suit::Srce, 2)));
    assert_eq!(legal, set(&[pip(Suit::Srce, 4), kralj(Suit::Srce)]));
    // No duty to overtake: the low srce is legal under a led srce dama.
    let legal = trick::legal_cards(hand, Some(court(Suit::Srce, Rank::Dama)));
    assert_eq!(legal, set(&[pip(Suit::Srce, 4), kralj(Suit::Srce)]));

    let cases = for_each_case(
        |hand, led| led.suit().is_some_and(|s| holds_suit(hand, s)),
        |hand, led, legal| {
            let expected: CardSet = hand.iter().filter(|c| c.suit() == led.suit()).collect();
            assert_eq!(legal, expected, "hand {hand:?}, led {led}");
        },
    );
    assert!(cases > 1000, "{cases} cases");
}

#[test]
fn c21_suit_led_void_with_taroks_only_taroks_are_legal() {
    let hand = set(&[
        pip(Suit::Pik, 7),
        kralj(Suit::Kriz),
        tarok(1),
        tarok(10),
        Card::SKIS,
    ]);
    let legal = trick::legal_cards(hand, Some(kralj(Suit::Srce)));
    assert_eq!(legal, set(&[tarok(1), tarok(10), Card::SKIS]));

    let cases = for_each_case(
        |hand, led| led.suit().is_some_and(|s| !holds_suit(hand, s)) && holds_tarok(hand),
        |hand, led, legal| {
            assert_eq!(legal, taroks_of(hand), "hand {hand:?}, led {led}");
        },
    );
    assert!(cases > 1000, "{cases} cases");
}

#[test]
fn c22_suit_led_void_without_taroks_every_card_is_legal() {
    let hand = set(&[
        pip(Suit::Pik, 7),
        kralj(Suit::Kriz),
        court(Suit::Karo, Rank::Dama),
    ]);
    assert_eq!(trick::legal_cards(hand, Some(pip(Suit::Srce, 1))), hand);

    let cases = for_each_case(
        |hand, led| led.suit().is_some_and(|s| !holds_suit(hand, s)) && !holds_tarok(hand),
        |hand, led, legal| {
            assert_eq!(legal, hand, "hand {hand:?}, led {led}");
        },
    );
    assert!(cases > 100, "{cases} cases");
}

#[test]
fn c23_tarok_led_with_taroks_only_taroks_are_legal() {
    let hand = set(&[
        pip(Suit::Pik, 7),
        kralj(Suit::Kriz),
        tarok(1),
        tarok(10),
        Card::MOND,
    ]);
    let expected = set(&[tarok(1), tarok(10), Card::MOND]);
    assert_eq!(trick::legal_cards(hand, Some(tarok(15))), expected);
    // The škis is a tarok: led škis asks for taroks too.
    assert_eq!(trick::legal_cards(hand, Some(Card::SKIS)), expected);
    // No duty to overtake: the pagat is legal under a led tarok.
    assert!(trick::legal_cards(hand, Some(tarok(2))).contains(tarok(1)));

    let cases = for_each_case(
        |hand, led| led.is_tarok() && holds_tarok(hand),
        |hand, led, legal| {
            assert_eq!(legal, taroks_of(hand), "hand {hand:?}, led {led}");
        },
    );
    assert!(cases > 1000, "{cases} cases");
}

#[test]
fn c24_tarok_led_without_taroks_every_card_is_legal() {
    let hand = set(&[
        pip(Suit::Pik, 7),
        kralj(Suit::Kriz),
        court(Suit::Karo, Rank::Dama),
        pip(Suit::Srce, 1),
    ]);
    assert_eq!(trick::legal_cards(hand, Some(tarok(15))), hand);
    assert_eq!(trick::legal_cards(hand, Some(Card::SKIS)), hand);

    let cases = for_each_case(
        |hand, led| led.is_tarok() && !holds_tarok(hand),
        |hand, led, legal| {
            assert_eq!(legal, hand, "hand {hand:?}, led {led}");
        },
    );
    assert!(cases > 100, "{cases} cases");
}

#[test]
fn c25_on_the_lead_every_card_is_legal() {
    let hand = set(&[pip(Suit::Pik, 7), kralj(Suit::Kriz), tarok(1), Card::SKIS]);
    assert_eq!(trick::legal_cards(hand, None), hand);
    for hand in sample_hands() {
        assert_eq!(trick::legal_cards(hand, None), hand);
    }

    // In a deal: the first leader may play any of the twelve cards.
    for leader in 0..4 {
        let state = PlayState::new(built_hands(), seat(leader)).expect("valid hands");
        assert_eq!(state.legal_cards(), built_hands()[leader as usize]);
        assert_eq!(state.legal_cards().len(), 12);
    }
}

#[test]
fn c26_illegal_card_returns_an_error_and_changes_nothing() {
    /// `play(card)` must fail with `expected` and leave the state as it was.
    fn refuse(state: &mut PlayState, card: Card, expected: PlayError) {
        let before = state.clone();
        assert_eq!(state.play(card), Err(expected));
        assert_eq!(*state, before, "state changed after refused card {card}");
    }

    // A hand-built deal: see `common::built_hands`. Seat 0 leads.
    let mut state = PlayState::new(built_hands(), seat(0)).expect("valid hands");

    // On the lead: a card of another seat, a card that was not dealt.
    refuse(
        &mut state,
        kralj(Suit::Pik),
        PlayError::NotInHand(kralj(Suit::Pik)),
    );
    refuse(&mut state, Card::SKIS, PlayError::NotInHand(Card::SKIS));
    state.play(pip(Suit::Srce, 4)).unwrap();

    // Seat 1 holds srce and must follow.
    refuse(
        &mut state,
        kralj(Suit::Pik),
        PlayError::NotLegal(kralj(Suit::Pik)),
    );
    // The card already on the table is in no hand.
    refuse(
        &mut state,
        pip(Suit::Srce, 4),
        PlayError::NotInHand(pip(Suit::Srce, 4)),
    );
    state.play(court(Suit::Srce, Rank::Fant)).unwrap();

    // Seat 2 is void in srce and holds taroks: a križ is not legal.
    refuse(
        &mut state,
        kralj(Suit::Kriz),
        PlayError::NotLegal(kralj(Suit::Kriz)),
    );
    state.play(tarok(1)).unwrap();
    state.play(tarok(5)).unwrap();

    // Seat 3 leads a tarok. Seat 0 has none and may play anything; seat 2
    // has taroks and may not play a križ.
    state.play(tarok(6)).unwrap();
    state.play(pip(Suit::Karo, 4)).unwrap();
    state.play(pip(Suit::Pik, 7)).unwrap();
    for suit in SUITS {
        let card = kralj(suit);
        let expected = if suit == Suit::Kriz {
            PlayError::NotLegal(card)
        } else {
            PlayError::NotInHand(card)
        };
        refuse(&mut state, card, expected);
    }
    // A card that seat 2 played in the first trick.
    refuse(&mut state, tarok(1), PlayError::NotInHand(tarok(1)));
    state.play(tarok(21)).unwrap();

    // Play the deal out, then no card is accepted.
    while !state.is_over() {
        let card = state.legal_cards().iter().next().expect("a legal card");
        state.play(card).unwrap();
    }
    for card in Card::all() {
        refuse(&mut state, card, PlayError::DealOver);
    }
}
