//! Spec 002, criteria 1-6: cards and counting.

mod common;

use std::collections::HashSet;

use common::{SUITS, court, kralj, pip, run_cases, set, suit_high_to_low, tarok};
use proptest::prelude::*;
use tarok_engine::{Card, CardSet, Face, Rank, Suit, points};

#[test]
fn c01_54_distinct_cards_and_index_round_trip() {
    let mut cards = HashSet::new();
    let mut faces = HashSet::new();
    for index in 0..54u8 {
        let card = Card::from_index(index).expect("indices 0-53 are cards");
        assert_eq!(card.index(), index);
        assert!(cards.insert(card), "card {index} appears twice");
        assert!(faces.insert(card.face()), "face of card {index} repeats");
    }
    assert_eq!(cards.len(), 54);
    assert_eq!(faces.len(), 54);
    assert_eq!(Card::all().count(), 54);
    for index in 54..=255u8 {
        assert_eq!(Card::from_index(index), None);
    }
}

/// Not a numbered criterion: the index order of the "In scope" section.
#[test]
fn card_index_order_matches_the_spec() {
    // Taroks I-XXI are 0-20, the škis is 21.
    for number in 1..=21u8 {
        assert_eq!(tarok(number).index(), number - 1);
        assert_eq!(tarok(number).face(), Face::Tarok(number));
    }
    assert_eq!(Card::PAGAT, tarok(1));
    assert_eq!(Card::MOND, tarok(21));
    assert_eq!(Card::SKIS.index(), 21);
    assert_eq!(Card::SKIS.face(), Face::Skis);

    // Then srce, karo, pik, križ, eight cards each, low to high.
    let courts = [Rank::Fant, Rank::Kaval, Rank::Dama, Rank::Kralj];
    let expected: [(Suit, u8, [u8; 4]); 4] = [
        (Suit::Srce, 22, [4, 3, 2, 1]),
        (Suit::Karo, 30, [4, 3, 2, 1]),
        (Suit::Pik, 38, [7, 8, 9, 10]),
        (Suit::Kriz, 46, [7, 8, 9, 10]),
    ];
    for (suit, first, pips) in expected {
        for (offset, number) in pips.into_iter().enumerate() {
            let card = pip(suit, number);
            assert_eq!(card.index(), first + offset as u8);
            assert_eq!(card.face(), Face::Pip(suit, number));
            assert_eq!(card.suit(), Some(suit));
            assert!(!card.is_tarok());
        }
        for (offset, rank) in courts.into_iter().enumerate() {
            let card = court(suit, rank);
            assert_eq!(card.index(), first + 4 + offset as u8);
            assert_eq!(card.face(), Face::Court(suit, rank));
            assert_eq!(card.suit(), Some(suit));
        }
    }

    // Numbers that are not on a card.
    assert_eq!(Card::tarok(0), None);
    assert_eq!(Card::tarok(22), None);
    assert_eq!(Card::pip(Suit::Srce, 5), None);
    assert_eq!(Card::pip(Suit::Karo, 7), None);
    assert_eq!(Card::pip(Suit::Pik, 4), None);
    assert_eq!(Card::pip(Suit::Kriz, 11), None);
}

#[test]
fn c02_card_values() {
    for suit in SUITS {
        assert_eq!(court(suit, Rank::Kralj).value(), 5);
        assert_eq!(court(suit, Rank::Dama).value(), 4);
        assert_eq!(court(suit, Rank::Kaval).value(), 3);
        assert_eq!(court(suit, Rank::Fant).value(), 2);
        // The four pips are the last four of the high-to-low order.
        for card in &suit_high_to_low(suit)[4..] {
            assert_eq!(card.value(), 1);
        }
    }
    assert_eq!(Card::SKIS.value(), 5);
    assert_eq!(Card::MOND.value(), 5);
    assert_eq!(Card::PAGAT.value(), 5);
    for number in 2..=20u8 {
        assert_eq!(tarok(number).value(), 1);
    }

    // Every card is covered. Cards per value: 35 ones (16 pips, 19 taroks),
    // 4 fanti, 4 kavali, 4 dame, 7 fives (4 kralji and the trula).
    let mut by_value = [0u32; 6];
    for card in Card::all() {
        by_value[card.value() as usize] += 1;
    }
    assert_eq!(by_value, [0, 35, 4, 4, 4, 7]);
}

#[test]
fn c03_piles_of_three_cards() {
    let kings = set(&[kralj(Suit::Srce), kralj(Suit::Karo), kralj(Suit::Pik)]);
    assert_eq!(points::count(kings), 13);

    for suit in SUITS {
        let courts = set(&[
            court(suit, Rank::Kralj),
            court(suit, Rank::Dama),
            court(suit, Rank::Kaval),
        ]);
        assert_eq!(points::count(courts), 10);
    }

    let pips = set(&[pip(Suit::Srce, 4), pip(Suit::Pik, 7), pip(Suit::Kriz, 10)]);
    assert_eq!(points::count(pips), 1);
    let plain_taroks = set(&[tarok(2), tarok(11), tarok(20)]);
    assert_eq!(points::count(plain_taroks), 1);
}

#[test]
fn c04_piles_of_two_one_and_zero_cards() {
    for suit in SUITS {
        let two = set(&[court(suit, Rank::Kralj), court(suit, Rank::Dama)]);
        assert_eq!(points::count(two), 8);
        assert_eq!(points::count(set(&[kralj(suit)])), 4);
    }
    assert_eq!(points::count(CardSet::EMPTY), 0);
}

#[test]
fn c05_whole_deck_counts_70() {
    let deck: CardSet = Card::all().collect();
    assert_eq!(deck.len(), 54);
    assert_eq!(deck, CardSet::FULL);
    assert_eq!(points::count(deck), 70);
    assert_eq!(points::TOTAL, 70);
}

#[test]
fn c06_any_two_way_split_of_the_deck_counts_70() {
    run_cases(10_000, any::<u64>(), |mask| {
        // Bit i of the mask sends card i to the first pile.
        let first: CardSet = Card::all().filter(|c| mask >> c.index() & 1 == 1).collect();
        let second: CardSet = Card::all().filter(|c| mask >> c.index() & 1 == 0).collect();
        prop_assert_eq!(first.len() + second.len(), 54);
        prop_assert!(first.intersection(second).is_empty());
        prop_assert_eq!(points::count(first) + points::count(second), 70);
        Ok(())
    });
}
