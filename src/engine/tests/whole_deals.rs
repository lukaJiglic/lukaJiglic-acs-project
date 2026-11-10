//! Spec 002, criteria 27-30: whole deals with random legal play.
//!
//! One case is a deal seed, the seat that leads the first trick and 48
//! choices. Choice `i` picks the card for play `i`: the legal cards are
//! taken in index order and the choice is reduced modulo their number.

mod common;

use common::{run_cases, seat};
use proptest::prelude::*;
use tarok_engine::{CardSet, Deal, PlayState, Trick, points};

const CASES: u32 = 10_000;

fn case() -> impl Strategy<Value = (u64, u8, Vec<u8>)> {
    (
        any::<u64>(),
        0..4u8,
        proptest::collection::vec(any::<u8>(), 48),
    )
}

/// Plays the deal to the end. `before_play` sees the state before each of
/// the 48 plays.
fn play_out(
    seed: u64,
    leader: u8,
    choices: &[u8],
    mut before_play: impl FnMut(&PlayState) -> Result<(), TestCaseError>,
) -> Result<(Deal, PlayState), TestCaseError> {
    let deal = Deal::from_seed(seed);
    let mut state = PlayState::from_deal(&deal, seat(leader));
    for &choice in choices {
        before_play(&state)?;
        let legal = state.legal_cards();
        prop_assert!(!legal.is_empty());
        let card = legal.nth(u32::from(choice) % legal.len()).unwrap();
        prop_assert_eq!(state.play(card), Ok(()));
    }
    Ok((deal, state))
}

#[test]
fn c27_legal_set_is_never_empty_before_the_deal_is_over() {
    run_cases(CASES, case(), |(seed, leader, choices)| {
        let mut plays = 0;
        let (_, state) = play_out(seed, leader, &choices, |state| {
            plays += 1;
            prop_assert!(!state.is_over());
            let legal = state.legal_cards();
            prop_assert!(!legal.is_empty());
            // Legal cards come from the hand of the player to move.
            prop_assert!(legal.is_subset(state.hand(state.to_move())));
            Ok(())
        })?;
        prop_assert_eq!(plays, 48);
        prop_assert!(state.is_over());
        prop_assert!(state.legal_cards().is_empty());
        Ok(())
    });
}

#[test]
fn c28_after_12_tricks_hands_are_empty_and_piles_hold_48_distinct_cards() {
    run_cases(CASES, case(), |(seed, leader, choices)| {
        let (deal, state) = play_out(seed, leader, &choices, |_| Ok(()))?;
        prop_assert!(state.is_over());
        prop_assert_eq!(state.trick_history().len(), 12);

        let mut in_piles = CardSet::EMPTY;
        let mut pile_sizes = 0;
        for index in 0..4 {
            prop_assert!(state.hand(seat(index)).is_empty());
            let pile = state.pile(seat(index));
            // A pile is whole tricks.
            prop_assert_eq!(pile.len() % 4, 0);
            pile_sizes += pile.len();
            in_piles = in_piles | pile;
        }
        // 48 cards counted pile by pile and 48 in the union: no card twice.
        prop_assert_eq!(pile_sizes, 48);
        prop_assert_eq!(in_piles.len(), 48);
        // They are the dealt cards: everything but the talon.
        prop_assert_eq!(in_piles, deal.talon_set().complement());

        // Each pile is the tricks its seat won.
        for index in 0..4 {
            let won: CardSet = state
                .trick_history()
                .iter()
                .filter(|t| t.winner == seat(index))
                .flat_map(|t| t.cards)
                .collect();
            prop_assert_eq!(state.pile(seat(index)), won);
        }
        Ok(())
    });
}

#[test]
fn c29_any_two_way_grouping_of_piles_and_talon_sums_to_70() {
    run_cases(CASES, case(), |(seed, leader, choices)| {
        let (deal, state) = play_out(seed, leader, &choices, |_| Ok(()))?;
        let piles = [
            state.pile(seat(0)),
            state.pile(seat(1)),
            state.pile(seat(2)),
            state.pile(seat(3)),
            deal.talon_set(),
        ];
        // Bit i of `grouping` puts pile i on the first side. All 32 ways.
        for grouping in 0..32u32 {
            let mut first = CardSet::EMPTY;
            let mut second = CardSet::EMPTY;
            for (i, &pile) in piles.iter().enumerate() {
                if grouping >> i & 1 == 1 {
                    first = first | pile;
                } else {
                    second = second | pile;
                }
            }
            prop_assert_eq!(
                points::count(first) + points::count(second),
                70,
                "grouping {:05b}",
                grouping
            );
        }
        // `points(seat)` is the count of that seat's pile.
        for index in 0..4 {
            prop_assert_eq!(
                state.points(seat(index)),
                points::count(piles[index as usize])
            );
        }
        Ok(())
    });
}

#[test]
fn c30_same_seed_and_choices_give_the_same_trick_history() {
    run_cases(CASES, case(), |(seed, leader, choices)| {
        let (first_deal, first) = play_out(seed, leader, &choices, |_| Ok(()))?;
        let (second_deal, second) = play_out(seed, leader, &choices, |_| Ok(()))?;
        let first_history: &[Trick] = first.trick_history();
        prop_assert_eq!(first_history.len(), 12);
        prop_assert_eq!(first_history, second.trick_history());
        prop_assert_eq!(first_deal, second_deal);
        prop_assert_eq!(first, second);
        Ok(())
    });
}
