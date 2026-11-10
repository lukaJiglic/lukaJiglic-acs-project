//! Spec 002, criteria 7-10: the seeded deal.

mod common;

use std::cell::Cell;

use common::run_cases;
use proptest::prelude::*;
use tarok_engine::{Card, CardSet, Deal, shuffled_deck};

/// Hands and talon hold all 54 cards exactly once: 12, 12, 12, 12 and 6.
fn check_shape(deal: &Deal) -> Result<(), TestCaseError> {
    let mut seen = CardSet::EMPTY;
    let mut total = 0;
    for hand in deal.hands() {
        prop_assert_eq!(hand.len(), 12);
        prop_assert!(seen.is_disjoint(hand));
        seen = seen | hand;
        total += hand.len();
    }
    for card in deal.talon() {
        prop_assert!(seen.insert(card), "talon card repeats: {}", card);
        total += 1;
    }
    prop_assert_eq!(deal.talon_set().len(), 6);
    prop_assert_eq!(total, 54);
    prop_assert_eq!(seen, CardSet::FULL);
    Ok(())
}

#[test]
fn c07_any_seed_deals_four_hands_of_12_and_a_talon_of_6() {
    run_cases(10_000, any::<u64>(), |seed| {
        check_shape(&Deal::from_seed(seed))
    });
}

#[test]
fn c08_same_seed_gives_the_same_deal_twice() {
    for seed in [0, 1, 2, 42, 1 << 32, u64::MAX - 1, u64::MAX] {
        let first = Deal::from_seed(seed);
        let second = Deal::from_seed(seed);
        assert_eq!(first, second, "seed {seed}");
        assert_eq!(shuffled_deck(seed), shuffled_deck(seed), "seed {seed}");
    }
}

/// A frozen deal. Card indices as in the spec. Hands are listed in index
/// order; the talon is listed in the order it lies.
struct GoldenDeal {
    seed: u64,
    hands: [[u8; 12]; 4],
    talon: [u8; 6],
}

/// The deal for seed 0, generated once on 2026-11-10 and frozen.
///
/// The same listing comes out of `tests/golden_deal_reference.py`, a
/// separate pure-Python ChaCha8 and shuffle written from the steps in
/// `src/deal.rs`. The CI workflow compares the two programs.
///
/// If this test fails, the shuffle changed. Every stored seed, game record
/// and training log then points at a different deal. Do not edit the listing
/// to make the test pass.
const GOLDEN_SEED_0: GoldenDeal = GoldenDeal {
    seed: 0,
    hands: [
        [0, 13, 17, 24, 26, 28, 30, 35, 39, 41, 47, 50],
        [5, 8, 16, 20, 21, 29, 31, 37, 44, 45, 48, 51],
        [3, 11, 19, 23, 25, 32, 33, 34, 36, 38, 46, 52],
        [1, 2, 6, 7, 9, 10, 12, 18, 27, 40, 43, 49],
    ],
    talon: [53, 22, 4, 14, 15, 42],
};

/// A second frozen deal, generated on 2026-11-10 the same way.
///
/// Seed 0 gives an all-zero cipher key, so it cannot show how the seed is
/// written into the key. This seed has eight different bytes: a change of
/// byte order (or of which bytes are used) gives another deal and fails
/// this test.
const GOLDEN_SEED_0123456789ABCDEF: GoldenDeal = GoldenDeal {
    seed: 0x0123_4567_89AB_CDEF,
    hands: [
        [8, 12, 15, 17, 18, 28, 29, 37, 38, 42, 43, 44],
        [6, 7, 9, 11, 16, 20, 23, 24, 31, 36, 47, 48],
        [4, 5, 10, 19, 22, 33, 40, 41, 46, 49, 52, 53],
        [0, 1, 3, 21, 25, 26, 27, 30, 34, 35, 39, 51],
    ],
    talon: [32, 45, 2, 50, 14, 13],
};

fn assert_matches_golden(golden: &GoldenDeal) {
    let deal = Deal::from_seed(golden.seed);
    for (seat, expected) in golden.hands.iter().enumerate() {
        let hand: Vec<u8> = deal.hands()[seat].iter().map(Card::index).collect();
        assert_eq!(hand, expected.to_vec(), "hand of seat {seat}");
    }
    assert_eq!(deal.talon().map(Card::index), golden.talon, "talon");
}

#[test]
fn c09_seed_0_matches_the_golden_deal() {
    assert_matches_golden(&GOLDEN_SEED_0);
}

#[test]
fn c09_seed_0x0123456789abcdef_matches_the_golden_deal() {
    assert_eq!(GOLDEN_SEED_0123456789ABCDEF.seed, 81_985_529_216_486_895);
    assert_matches_golden(&GOLDEN_SEED_0123456789ABCDEF);
}

#[test]
fn c10_different_seeds_give_different_deals() {
    // 1,000 random pairs of different seeds.
    let tried = Cell::new(0u32);
    let different = Cell::new(0u32);
    let pairs = (any::<u64>(), any::<u64>()).prop_filter("the seeds differ", |(a, b)| a != b);
    run_cases(1_000, pairs, |(a, b)| {
        tried.set(tried.get() + 1);
        if Deal::from_seed(a) != Deal::from_seed(b) {
            different.set(different.get() + 1);
        }
        Ok(())
    });
    assert_eq!(tried.get(), 1_000);
    assert!(
        different.get() >= 999,
        "only {} of 1000 random seed pairs gave different deals",
        different.get()
    );

    // Neighbouring seeds, as a training run uses them: 0 and 1, 1 and 2, ...
    let neighbours = (0..1_000u64)
        .filter(|&seed| Deal::from_seed(seed) != Deal::from_seed(seed + 1))
        .count();
    assert!(
        neighbours >= 999,
        "only {neighbours} of 1000 neighbouring seed pairs gave different deals"
    );
}
