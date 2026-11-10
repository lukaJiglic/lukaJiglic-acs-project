//! Spec 002, "In scope": `CardSet` operations. No numbered criterion.

mod common;

use common::{SUITS, kralj, pip, set, tarok};
use tarok_engine::{Card, CardSet, Suit};

#[test]
fn cardset_insert_remove_contains_len() {
    let mut cards = CardSet::EMPTY;
    assert!(cards.is_empty());
    assert_eq!(cards.len(), 0);

    assert!(cards.insert(Card::SKIS));
    assert!(!cards.insert(Card::SKIS), "second insert reports no change");
    assert!(cards.insert(pip(Suit::Kriz, 7)));
    assert!(cards.contains(Card::SKIS));
    assert!(cards.contains(pip(Suit::Kriz, 7)));
    assert!(!cards.contains(Card::MOND));
    assert_eq!(cards.len(), 2);

    assert!(cards.remove(Card::SKIS));
    assert!(!cards.remove(Card::SKIS), "second remove reports no change");
    assert!(!cards.contains(Card::SKIS));
    assert_eq!(cards.len(), 1);
    assert!(!cards.is_empty());
}

#[test]
fn cardset_iterates_in_index_order() {
    let cards = set(&[kralj(Suit::Kriz), tarok(3), Card::SKIS, pip(Suit::Srce, 4)]);
    let listed: Vec<u8> = cards.iter().map(Card::index).collect();
    assert_eq!(listed, vec![2, 21, 22, 53]);
    assert_eq!(cards.iter().len(), 4);
    assert_eq!(cards.nth(0), Some(tarok(3)));
    assert_eq!(cards.nth(3), Some(kralj(Suit::Kriz)));
    assert_eq!(cards.nth(4), None);
    // Past the end, however far: `None` at once, no long loop.
    assert_eq!(cards.nth(u32::MAX), None);
    assert_eq!(CardSet::FULL.nth(53), Some(kralj(Suit::Kriz)));
    assert_eq!(CardSet::FULL.nth(54), None);
    assert_eq!(CardSet::FULL.nth(u32::MAX), None);
    assert_eq!(CardSet::EMPTY.nth(0), None);
    assert_eq!(CardSet::EMPTY.nth(u32::MAX), None);

    let all: Vec<Card> = CardSet::FULL.iter().collect();
    let expected: Vec<Card> = Card::all().collect();
    assert_eq!(all, expected);
    assert_eq!(CardSet::EMPTY.iter().next(), None);
}

#[test]
fn cardset_set_operations() {
    let a = set(&[tarok(1), tarok(2), kralj(Suit::Srce)]);
    let b = set(&[tarok(2), kralj(Suit::Srce), kralj(Suit::Pik)]);

    assert_eq!(
        a.union(b),
        set(&[tarok(1), tarok(2), kralj(Suit::Srce), kralj(Suit::Pik)])
    );
    assert_eq!(a.intersection(b), set(&[tarok(2), kralj(Suit::Srce)]));
    assert_eq!(a.difference(b), set(&[tarok(1)]));
    assert_eq!(a | b, a.union(b));
    assert_eq!(a & b, a.intersection(b));
    assert_eq!(a - b, a.difference(b));

    assert!(a.intersection(b).is_subset(a));
    assert!(!a.is_subset(b));
    assert!(a.is_disjoint(set(&[kralj(Suit::Pik)])));
    assert!(!a.is_disjoint(b));

    let rest = a.complement();
    assert_eq!(rest.len(), 51);
    assert!(rest.is_disjoint(a));
    assert_eq!(rest | a, CardSet::FULL);
    assert_eq!(CardSet::FULL.complement(), CardSet::EMPTY);
}

#[test]
fn cardset_bits_round_trip() {
    let cards = set(&[tarok(1), Card::SKIS, kralj(Suit::Kriz)]);
    assert_eq!(cards.bits(), 1 | 1 << 21 | 1 << 53);
    assert_eq!(CardSet::from_bits(cards.bits()), Some(cards));
    assert_eq!(CardSet::FULL.bits(), (1u64 << 54) - 1);
    assert_eq!(CardSet::from_bits(1 << 54), None, "bit 54 is not a card");
    assert_eq!(CardSet::from_bits(u64::MAX), None);
}

#[test]
fn cardset_taroks_and_suits_partition_the_deck() {
    assert_eq!(CardSet::TAROKS.len(), 22);
    assert!(CardSet::TAROKS.iter().all(Card::is_tarok));
    let mut seen = CardSet::TAROKS;
    for suit in SUITS {
        let of_suit = CardSet::suit(suit);
        assert_eq!(of_suit.len(), 8);
        assert!(of_suit.iter().all(|c| c.suit() == Some(suit)));
        assert!(seen.is_disjoint(of_suit));
        seen = seen | of_suit;
    }
    assert_eq!(seen, CardSet::FULL);
}
