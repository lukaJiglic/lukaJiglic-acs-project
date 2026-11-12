//! Spec 006, criterion 6: no leak.
//!
//! Take a random moment of a random deal and one observing seat. Change two
//! cards the observer cannot see between two hidden places. If the same
//! actions are still a legal deal, the observer's tokens, state and mask
//! must be the same in both worlds.
//!
//! Hidden places: the hands of the three other seats, the declarer's hidden
//! discards, and the talon before it is turned up.
//!
//! A fourth kind changes no unseen card. The packet the declarer took lies
//! open, so every seat has seen its cards. What stays hidden is which cards
//! the declarer put down: a card of the packet, or a card it was dealt. The
//! two worlds are the same deal and the same actions but for that one
//! discard, and no other seat may tell them apart.

mod common;

use std::cell::Cell;

use common::flow::play_random;
use common::{run_cases, seat};
use proptest::prelude::*;
use tarok_engine::encode::{Observation, observe};
use tarok_engine::{Action, Card, CardSet, Deal, Game, Seat};

/// Replays the actions on another deal. `None` if one of them is not legal
/// there: the changed cards do not fit what was played.
fn replay(deal: &Deal, dealer: Seat, actions: &[Action]) -> Option<Game> {
    let mut game = Game::new(deal, dealer);
    for &action in actions {
        game.apply(action).ok()?;
    }
    Some(game)
}

/// The deal with two cards changed over, wherever they lie.
fn changed_over(deal: &Deal, one: Card, two: Card) -> Deal {
    let swap = |card: Card| {
        if card == one {
            two
        } else if card == two {
            one
        } else {
            card
        }
    };
    let hands = deal
        .hands()
        .map(|hand| hand.iter().map(swap).collect::<CardSet>());
    let talon = deal.talon().map(swap);
    Deal::new(hands, talon).expect("still a whole deal")
}

/// The cards the observer has not seen, from the truth: not in its hand,
/// not played, not in the shown talon, not a shown discard, not an own
/// discard.
fn truly_unseen(game: &Game, observer: Seat) -> CardSet {
    let public = game.public();
    let mut seen = game.private(observer).hand() | game.private(observer).discards();
    seen = seen | public.shown_discards();
    for trick in public.tricks() {
        seen = seen | trick.cards.into_iter().collect();
    }
    seen = seen | public.current_trick().iter().copied().collect();
    if let Some(talon) = public.talon() {
        seen = seen | talon.into_iter().collect();
    }
    seen.complement()
}

fn same_for_the_observer(one: &Observation, two: &Observation) {
    assert_eq!(one.tokens, two.tokens, "the tokens differ");
    for (index, (a, b)) in one.state.iter().zip(&two.state).enumerate() {
        assert_eq!(a, b, "state feature {index} differs");
    }
    assert_eq!(one.mask, two.mask, "the mask differs");
}

fn case() -> impl Strategy<Value = (u64, u8, Vec<u8>, usize, u8, u64)> {
    (
        any::<u64>(),
        0..4u8,
        proptest::collection::vec(any::<u8>(), 96),
        0..80usize,
        0..4u8,
        any::<u64>(),
    )
}

/// Picks element `pick mod len` of a set, or `None` if it is empty.
fn pick_from(cards: CardSet, pick: u64) -> Option<Card> {
    if cards.is_empty() {
        None
    } else {
        cards.nth((pick % u64::from(cards.len())) as u32)
    }
}

#[test]
fn c06_no_leak_when_hidden_cards_change_places() {
    let two_hands = Cell::new(0u32);
    let hand_and_discards = Cell::new(0u32);
    let hand_and_talon = Cell::new(0u32);
    let which_card_hidden = Cell::new(0u32);
    let not_a_legal_deal = Cell::new(0u32);

    run_cases(
        30_000,
        case(),
        |(seed, dealer, choices, stop, observer, pick)| {
            let dealer = seat(dealer);
            let observer = seat(observer);
            // The moment: before action number `stop` (or the end of the deal).
            let finished = play_random(seed, dealer, &choices, |_| {});
            let actions: Vec<Action> = finished.full().actions().collect();
            let stop = stop.min(actions.len());
            let history = &actions[..stop];
            let deal = Deal::from_seed(seed);
            let game = replay(&deal, dealer, history).expect("the deal as it was played");

            let unseen = truly_unseen(&game, observer);
            let others: Vec<Seat> = Seat::ALL.into_iter().filter(|&s| s != observer).collect();
            let declarer = game.public().declarer();
            let talon_hidden = game.public().talon().is_none();
            let hidden_discards = if declarer == Some(observer) {
                CardSet::EMPTY
            } else {
                game.full().discards() & unseen
            };

            // The declarer, if the observer is another seat.
            let hider = declarer.filter(|&seat| seat != observer);

            // Which kind of change, and which cards. The last number says
            // which kind it became: 0 two hands, 1 hand and discards, 2 hand
            // and talon, 3 which card the declarer hid.
            let kind = pick % 4;
            let (one, two, other_actions, used): (Card, Card, Vec<Action>, u8) = match kind {
                // A hand and the hidden discards: the declarer put down `two`
                // instead of `one`.
                1 if !hidden_discards.is_empty() => {
                    let from = others[(pick / 3 % 3) as usize];
                    let in_hand = game.full().hand(from) & unseen;
                    // A card that may be a hidden discard: no tarok (it would
                    // be shown), no king.
                    let may_be_hidden: CardSet = in_hand
                        .iter()
                        .filter(|card| !card.is_tarok() && card.value() != 5)
                        .collect();
                    let (Some(one), Some(two)) = (
                        pick_from(hidden_discards, pick / 9),
                        pick_from(may_be_hidden, pick / 81),
                    ) else {
                        return Ok(());
                    };
                    let renamed = history
                        .iter()
                        .map(|&action| {
                            if action == Action::Discard(one) {
                                Action::Discard(two)
                            } else {
                                action
                            }
                        })
                        .collect();
                    (one, two, renamed, 1)
                }
                // Which card the declarer hid: `one` in this world, `two`
                // in the other. One of the two comes from the packet it
                // took, the other from the hand it was dealt.
                3 if hider.is_some() => {
                    let Some(declarer) = hider else {
                        return Ok(());
                    };
                    let hidden = game.full().discards() - game.public().shown_discards();
                    let taken = game.public().taken_cards();
                    let Some(one) = pick_from(hidden, pick / 4) else {
                        return Ok(());
                    };
                    // A card still in the declarer's hand that may be put
                    // down hidden, from the other source than `one`.
                    let candidates: CardSet = game
                        .full()
                        .hand(declarer)
                        .iter()
                        .filter(|&card| {
                            !card.is_tarok()
                                && card.value() != 5
                                && taken.contains(card) != taken.contains(one)
                        })
                        .collect();
                    let Some(two) = pick_from(candidates, pick / 64) else {
                        return Ok(());
                    };
                    let renamed = history
                        .iter()
                        .map(|&action| {
                            if action == Action::Discard(one) {
                                Action::Discard(two)
                            } else {
                                action
                            }
                        })
                        .collect();
                    (one, two, renamed, 3)
                }
                // A hand and the talon that is still face down.
                2 if talon_hidden => {
                    let from = others[(pick / 3 % 3) as usize];
                    let talon: CardSet = game.full().talon().into_iter().collect();
                    let (Some(one), Some(two)) = (
                        pick_from(talon, pick / 9),
                        pick_from(game.full().hand(from), pick / 81),
                    ) else {
                        return Ok(());
                    };
                    (one, two, history.to_vec(), 2)
                }
                // Two other hands.
                _ => {
                    let first = others[(pick / 3 % 3) as usize];
                    let second = others[((pick / 3 + 1 + pick / 9 % 2) % 3) as usize];
                    let (Some(one), Some(two)) = (
                        pick_from(game.full().hand(first) & unseen, pick / 27),
                        pick_from(game.full().hand(second) & unseen, pick / 729),
                    ) else {
                        return Ok(());
                    };
                    (one, two, history.to_vec(), 0)
                }
            };
            if used == 3 {
                // Both cards may be seen ones: the packet lies open.
                let taken = game.public().taken_cards();
                assert!(taken.contains(one) != taken.contains(two));
            } else {
                assert!(unseen.contains(one) && unseen.contains(two));
            }

            // The other world. For "hand and discards" with the declarer's own
            // hand the deal is the same and only the discard differs.
            let declarer_holds_two = declarer.is_some_and(|d| game.full().hand(d).contains(two));
            let other_deal = if used == 3 || (used == 1 && declarer_holds_two) {
                deal
            } else {
                changed_over(&deal, one, two)
            };
            let Some(other) = replay(&other_deal, dealer, &other_actions) else {
                // The actions are not legal with the cards changed: this is not
                // a world the observer has to tell apart. Nothing to compare.
                not_a_legal_deal.set(not_a_legal_deal.get() + 1);
                return Ok(());
            };
            // The two worlds do differ in what is hidden...
            assert_ne!(game, other);
            let here = observe(&game, observer);
            let there = observe(&other, observer);
            assert_ne!(
                here.critic, there.critic,
                "the critic's hidden features tell the two worlds apart"
            );
            // ...and the observer sees no difference.
            same_for_the_observer(&here, &there);

            let counter = match used {
                1 => &hand_and_discards,
                2 => &hand_and_talon,
                3 => &which_card_hidden,
                _ => &two_hands,
            };
            counter.set(counter.get() + 1);
            Ok(())
        },
    );

    let compared =
        two_hands.get() + hand_and_discards.get() + hand_and_talon.get() + which_card_hidden.get();
    println!(
        "no-leak cases compared: {compared} (two hands {}, hand and hidden discards {}, hand and \
         unshown talon {}, which card the declarer hid {}); {} changes were not a legal deal",
        two_hands.get(),
        hand_and_discards.get(),
        hand_and_talon.get(),
        which_card_hidden.get(),
        not_a_legal_deal.get()
    );
    assert!(compared >= 10_000, "{compared} cases compared");
    assert!(two_hands.get() >= 5_000);
    assert!(hand_and_discards.get() >= 300);
    assert!(hand_and_talon.get() >= 300);
    assert!(which_card_hidden.get() >= 300);
}
