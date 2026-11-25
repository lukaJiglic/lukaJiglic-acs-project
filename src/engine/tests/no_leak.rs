//! Spec 006, criterion 6: no leak. Spec 009, criterion 38: the same on
//! deals with announcements and kontra.
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
//!
//! With announcements (spec 009) a change of hidden cards can change who
//! the partner is, and who holds the pagat or the called king. If what was
//! said is still legal with the cards changed, the observer must not see a
//! difference either: not in the tokens, not in block D of the state, not
//! in the legal actions of its own turn in the round.
//!
//! A fifth kind, on deals of flow B only, aims at that (the review of spec
//! 009, point 1): the called king changes hands between two seats the
//! observer does not see. The partner is another seat in the other world,
//! or there is none, or there is one now. Whatever the round showed about
//! the sides (reading 3 of spec 009, D147, D148) must have been said by
//! the holder of the king: else the same actions are legal in the other
//! world, the engine shows another partner there, and the two
//! observations differ.

mod common;

use std::cell::Cell;

use common::flow::{play_random, play_random_b};
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

/// A case: the seed of the cards, the dealer, the choices of the play, the
/// moment (the number of actions taken), the observer and a number that
/// picks the change. A deal of flow B is longer, so its moment goes
/// further.
fn case(flow_b: bool) -> impl Strategy<Value = (u64, u8, Vec<u8>, usize, u8, u64)> {
    let (choices, moments): (usize, usize) = if flow_b { (191, 190) } else { (96, 80) };
    (
        any::<u64>(),
        0..4u8,
        proptest::collection::vec(any::<u8>(), choices),
        0..moments,
        0..4u8,
        any::<u64>(),
    )
}

/// What the cases of a run were.
struct Counts {
    two_hands: u32,
    hand_and_discards: u32,
    hand_and_talon: u32,
    which_card_hidden: u32,
    /// Compared cases of the fifth kind: the called king in another hand.
    king_changes_hands: u32,
    /// Of those: something was said before the moment.
    king_changes_hands_after_something_said: u32,
    /// Cases of the fifth kind in which the same actions were not legal
    /// with the king in the other hand.
    king_change_not_legal: u32,
    /// Compared cases in which a seat is shown as an opponent while the
    /// partner is hidden (D150).
    opponent_shown: u32,
    /// Of those: cases of the fifth kind.
    opponent_shown_and_king_moved: u32,
    not_a_legal_deal: u32,
    /// Compared cases with an announcement or a kontra before the moment.
    after_something_said: u32,
    /// Of those: the partner was still hidden at the moment.
    said_and_partner_hidden: u32,
    /// Compared cases in which the partner is another seat in the other
    /// world.
    another_partner: u32,
}

impl Counts {
    fn compared(&self) -> u32 {
        self.two_hands
            + self.hand_and_discards
            + self.hand_and_talon
            + self.which_card_hidden
            + self.king_changes_hands
    }
}

/// Picks element `pick mod len` of a set, or `None` if it is empty.
fn pick_from(cards: CardSet, pick: u64) -> Option<Card> {
    if cards.is_empty() {
        None
    } else {
        cards.nth((pick % u64::from(cards.len())) as u32)
    }
}

/// The cases of spec 006 on deals of flow A (all pass in the round) or of
/// flow B (announcements and kontras at random). Flow B runs four times
/// the cases: half of them are of the fifth kind, and few of those are
/// compared (a king must be called, unseen and still in a hand, and what
/// was said must be legal with the king in the other hand).
fn no_leak(flow_b: bool) -> Counts {
    let cases = if flow_b { 120_000 } else { 30_000 };
    let two_hands = Cell::new(0u32);
    let after_something_said = Cell::new(0u32);
    let said_and_partner_hidden = Cell::new(0u32);
    let another_partner = Cell::new(0u32);
    let hand_and_discards = Cell::new(0u32);
    let hand_and_talon = Cell::new(0u32);
    let which_card_hidden = Cell::new(0u32);
    let king_changes_hands = Cell::new(0u32);
    let king_changes_hands_after_something_said = Cell::new(0u32);
    let king_change_not_legal = Cell::new(0u32);
    let opponent_shown = Cell::new(0u32);
    let opponent_shown_and_king_moved = Cell::new(0u32);
    let not_a_legal_deal = Cell::new(0u32);

    run_cases(
        cases,
        case(flow_b),
        |(seed, dealer, choices, stop, observer, pick)| {
            let dealer = seat(dealer);
            let observer = seat(observer);
            // The moment: before action number `stop` (or the end of the deal).
            let finished = if flow_b {
                play_random_b(seed, dealer, &choices, |_| {})
            } else {
                play_random(seed, dealer, &choices, |_| {})
            };
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
            // and talon, 3 which card the declarer hid, 4 the called king
            // in another hand. The fifth kind is taken in one case of two
            // on deals of flow B and never on deals of flow A, so the cases
            // of `c06` are what they were.
            let kind = if flow_b && (pick >> 40).is_multiple_of(2) {
                4
            } else {
                pick % 4
            };
            let (one, two, other_actions, used): (Card, Card, Vec<Action>, u8) = match kind {
                // The called king changes hands. It lies in the hand of a
                // seat the observer does not see (the partner, or the
                // declarer who plays alone) and goes to another such seat.
                4 => {
                    let king = game
                        .public()
                        .called_king()
                        .filter(|&king| unseen.contains(king));
                    let holder = king.and_then(|king| {
                        others
                            .iter()
                            .copied()
                            .find(|&seat| game.full().hand(seat).contains(king))
                    });
                    let (Some(one), Some(holder)) = (king, holder) else {
                        return Ok(());
                    };
                    let rest: Vec<Seat> = others
                        .iter()
                        .copied()
                        .filter(|&seat| seat != holder)
                        .collect();
                    let to = rest[(pick / 3 % 2) as usize];
                    let hand = game.full().hand(to) & unseen;
                    // A card of the king's suit where there is one: every
                    // seat then follows suit as it did, so more of the
                    // changed deals stay legal.
                    let same_suit: CardSet = hand
                        .iter()
                        .filter(|card| card.suit() == one.suit())
                        .collect();
                    let Some(two) =
                        pick_from(same_suit, pick / 27).or_else(|| pick_from(hand, pick / 27))
                    else {
                        return Ok(());
                    };
                    (one, two, history.to_vec(), 4)
                }
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
                if used == 4 {
                    king_change_not_legal.set(king_change_not_legal.get() + 1);
                }
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
            // What all know about the sides is the same in both worlds: it
            // may depend only on what was said and on the cards all have
            // seen (criterion 16, D150).
            assert_eq!(game.public().partner(), other.public().partner());
            let shown = Seat::ALL.map(|seat| game.public().side_of(seat));
            assert_eq!(shown, Seat::ALL.map(|seat| other.public().side_of(seat)));
            if game.public().partner() == tarok_engine::PublicPartner::NotKnown
                && shown.contains(&Some(false))
            {
                opponent_shown.set(opponent_shown.get() + 1);
                if used == 4 {
                    opponent_shown_and_king_moved.set(opponent_shown_and_king_moved.get() + 1);
                }
            }

            let counter = match used {
                1 => &hand_and_discards,
                2 => &hand_and_talon,
                3 => &which_card_hidden,
                4 => &king_changes_hands,
                _ => &two_hands,
            };
            counter.set(counter.get() + 1);
            let said = history
                .iter()
                .any(|action| matches!(action, Action::Announce(_) | Action::Kontra(_)));
            if used == 4 {
                // The king in another hand: another partner, or none.
                assert_ne!(game.full().partner(), other.full().partner());
                if said {
                    king_changes_hands_after_something_said
                        .set(king_changes_hands_after_something_said.get() + 1);
                }
            }
            if said {
                after_something_said.set(after_something_said.get() + 1);
                if game.public().partner() == tarok_engine::PublicPartner::NotKnown {
                    said_and_partner_hidden.set(said_and_partner_hidden.get() + 1);
                }
            }
            if game.full().partner() != other.full().partner() {
                another_partner.set(another_partner.get() + 1);
            }
            Ok(())
        },
    );

    Counts {
        two_hands: two_hands.get(),
        hand_and_discards: hand_and_discards.get(),
        hand_and_talon: hand_and_talon.get(),
        which_card_hidden: which_card_hidden.get(),
        king_changes_hands: king_changes_hands.get(),
        king_changes_hands_after_something_said: king_changes_hands_after_something_said.get(),
        king_change_not_legal: king_change_not_legal.get(),
        opponent_shown: opponent_shown.get(),
        opponent_shown_and_king_moved: opponent_shown_and_king_moved.get(),
        not_a_legal_deal: not_a_legal_deal.get(),
        after_something_said: after_something_said.get(),
        said_and_partner_hidden: said_and_partner_hidden.get(),
        another_partner: another_partner.get(),
    }
}

#[test]
fn c06_no_leak_when_hidden_cards_change_places() {
    let counts = no_leak(false);
    let compared = counts.compared();
    println!(
        "no-leak cases compared: {compared} (two hands {}, hand and hidden discards {}, hand and \
         unshown talon {}, which card the declarer hid {}); {} changes were not a legal deal",
        counts.two_hands,
        counts.hand_and_discards,
        counts.hand_and_talon,
        counts.which_card_hidden,
        counts.not_a_legal_deal
    );
    assert!(compared >= 10_000, "{compared} cases compared");
    assert!(counts.two_hands >= 5_000);
    assert!(counts.hand_and_discards >= 300);
    assert!(counts.hand_and_talon >= 300);
    assert!(counts.which_card_hidden >= 300);
    // Flow A: nothing is said in these deals, and the fifth kind is not
    // taken.
    assert_eq!(counts.after_something_said, 0);
    assert_eq!(counts.king_changes_hands + counts.king_change_not_legal, 0);
}

/// Spec 009, criterion 38: hidden sides stay hidden. The cases of `c06` on
/// deals with announcements and kontras. A built pair of deals is in
/// `tests/encode.rs` (c38).
#[test]
fn c38_no_leak_with_announcements_and_kontra() {
    let counts = no_leak(true);
    let compared = counts.compared();
    println!(
        "no-leak cases of flow B compared: {compared} (two hands {}, hand and hidden discards \
         {}, hand and unshown talon {}, which card the declarer hid {}); {} changes were not a \
         legal deal; {} compared after something was said, {} of them with the partner still \
         hidden; {} with another partner in the other world. The called king in another \
         hand: {} compared, {} of them after something was said; {} were not a legal deal",
        counts.two_hands,
        counts.hand_and_discards,
        counts.hand_and_talon,
        counts.which_card_hidden,
        counts.not_a_legal_deal,
        counts.after_something_said,
        counts.said_and_partner_hidden,
        counts.another_partner,
        counts.king_changes_hands,
        counts.king_changes_hands_after_something_said,
        counts.king_change_not_legal
    );
    println!(
        "a seat shown as an opponent while the partner is hidden: {} compared, {} of them with \
         the called king in another hand",
        counts.opponent_shown, counts.opponent_shown_and_king_moved
    );
    assert!(counts.opponent_shown >= 1_000);
    assert!(counts.opponent_shown_and_king_moved >= 100);
    assert!(compared >= 15_000, "{compared} cases compared");
    assert!(counts.two_hands >= 10_000);
    assert!(counts.after_something_said >= 12_000);
    assert!(counts.said_and_partner_hidden >= 2_000);
    // The called king in another hand, with everything said still legal.
    // The built pairs of `tests/encode.rs` (c38) and `tests/kontra.rs`
    // (c14) are such cases written by hand.
    assert!(counts.king_changes_hands >= 1_200);
    assert!(counts.king_changes_hands_after_something_said >= 700);
    assert!(counts.another_partner >= 1_200);
}
