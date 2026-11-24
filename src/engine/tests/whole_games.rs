//! Spec 005, criteria 46-50: whole deals with random legal actions. Spec
//! 009, criteria 41-44: the same with announcements and kontra.
//!
//! One case is a deal seed, a dealer and a list of choices. Choice `i`
//! picks action `i` among the legal actions; see `common::flow::play_random`.

mod common;

use common::flow::{play_random, play_random_b, seeded_game, seeded_game_b};
use common::second_scorer::{assert_scores_agree, second_score};
use common::{run_cases, seat};
use proptest::prelude::*;
use tarok_engine::{
    Action, Bonus, Card, CardSet, Contract, Game, Phase, Seat, Target, action::slots, points,
};

const CASES: u32 = 10_000;

fn case() -> impl Strategy<Value = (u64, u8, Vec<u8>)> {
    (
        any::<u64>(),
        0..4u8,
        proptest::collection::vec(any::<u8>(), 96),
    )
}

/// Phases in the order a deal goes through them.
fn order(phase: Phase) -> u8 {
    match phase {
        Phase::Bidding => 0,
        Phase::ContractChoice => 1,
        Phase::KingCall => 2,
        Phase::TalonPick => 3,
        Phase::Discard => 4,
        Phase::Announcements => 5,
        Phase::Play => 6,
        Phase::Over => 7,
    }
}

#[test]
fn c46_every_deal_reaches_over_and_the_legal_set_is_never_empty_before() {
    run_cases(CASES, case(), |(seed, dealer, choices)| {
        let mut actions = 0u32;
        let mut last_phase = Phase::Bidding;
        let game = play_random(seed, seat(dealer), &choices, |game| {
            actions += 1;
            assert_ne!(game.phase(), Phase::Over);
            assert!(!game.legal_actions().is_empty());
            assert!(game.to_act().is_some());
            assert!(game.score().is_none(), "no score before the end");
            // The phases only move forward.
            assert!(order(game.phase()) >= order(last_phase));
            last_phase = game.phase();
        });
        prop_assert_eq!(game.phase(), Phase::Over);
        prop_assert!(game.legal_actions().is_empty());
        prop_assert_eq!(game.to_act(), None);
        prop_assert!(game.score().is_some());
        // At most 13 bids and passes, one contract, one king, one packet,
        // three discards, the four passes of the round of announcements
        // (spec 009) and 48 cards.
        prop_assert!(
            actions <= 13 + 1 + 1 + 1 + 3 + 4 + 48,
            "{} actions",
            actions
        );
        prop_assert!(game.public().bids().len() <= 13);
        Ok(())
    });
}

#[test]
fn c47_all_54_cards_end_in_the_two_piles_and_the_counts_sum_to_70() {
    run_cases(CASES, case(), |(seed, dealer, choices)| {
        let game = play_random(seed, seat(dealer), &choices, |_| {});
        // The second scorer of the tests collects each side's pile on its
        // own from the tricks, the discards and the talon. Neither pile is
        // taken as "the rest of the deck".
        let second = second_score(&game);
        let ours = second.declarer_pile;
        let theirs = second.opponents_pile;
        // Each card once: 54 in all, none in both piles.
        prop_assert_eq!(ours.len() + theirs.len(), 54);
        prop_assert!(ours.is_disjoint(theirs));
        prop_assert_eq!(ours | theirs, CardSet::FULL);
        // The engine has the same two piles.
        let full = game.full();
        prop_assert_eq!(full.declarer_pile(), ours);
        prop_assert_eq!(full.opponents_pile(), theirs);
        // Nothing is left in a hand.
        for seat in Seat::ALL {
            prop_assert!(full.hand(seat).is_empty());
        }
        // Each pile counted by itself; the two counts sum to 70.
        let our_points = points::count(ours);
        let their_points = points::count(theirs);
        prop_assert_eq!(our_points + their_points, 70);
        let score = game.score().unwrap();
        prop_assert_eq!(score.declarer_points(), our_points);
        prop_assert_eq!(score.opponents_points(), their_points);
        Ok(())
    });
}

/// Not a numbered criterion: the engine's score against the second scorer
/// of the tests (`tests/common/second_scorer.rs`, written from criteria
/// 29-44 of spec 005 with the answers of D118, and from spec 013: a valat
/// keeps the mondfang, pagat ultimo follows the side) on 10,000 random
/// deals: both piles, both counts and all eight parts of all four entries.
#[test]
fn score_matches_the_second_scorer_on_random_deals() {
    run_cases(CASES, case(), |(seed, dealer, choices)| {
        let game = play_random(seed, seat(dealer), &choices, |_| {});
        assert_scores_agree(&game);
        Ok(())
    });
}

#[test]
fn c48_declarer_and_partner_write_the_same_and_opponents_0_or_minus_20() {
    run_cases(CASES, case(), |(seed, dealer, choices)| {
        let game = play_random(seed, seat(dealer), &choices, |_| {});
        let score = game.score().unwrap();
        let declarer = game.public().declarer().unwrap();
        let partner = game.full().partner();
        for seat in Seat::ALL {
            let parts = score.parts(seat);
            prop_assert!(parts.mondfang == 0 || parts.mondfang == -20);
            if seat == declarer {
                continue;
            }
            if Some(seat) == partner {
                // Equal apart from a mondfang.
                prop_assert_eq!(
                    score.entry(seat) - parts.mondfang,
                    score.entry(declarer) - score.parts(declarer).mondfang
                );
            } else {
                let entry = score.entry(seat);
                prop_assert!(entry == 0 || entry == -20, "opponent wrote {}", entry);
            }
        }
        // At most one player loses the mond.
        let mondfangs = Seat::ALL
            .iter()
            .filter(|&&s| score.parts(s).mondfang != 0)
            .count();
        prop_assert!(mondfangs <= 1);
        Ok(())
    });
}

#[test]
fn c49_same_seed_dealer_and_choices_give_the_same_result() {
    // This can only fail if the engine is not deterministic; that is what
    // the criterion asks.
    run_cases(CASES, case(), |(seed, dealer, choices)| {
        let first = play_random(seed, seat(dealer), &choices, |_| {});
        let second = play_random(seed, seat(dealer), &choices, |_| {});
        prop_assert_eq!(first.score(), second.score());
        prop_assert_eq!(first.public().bids(), second.public().bids());
        prop_assert_eq!(first.public().tricks(), second.public().tricks());
        prop_assert_eq!(&first, &second);
        // Another dealer gives another deal flow: forehand differs.
        let other = Game::from_seed(seed, seat(dealer).next());
        prop_assert_ne!(other.public().forehand(), first.public().forehand());
        Ok(())
    });
}

#[test]
fn c50_every_legal_action_maps_to_a_slot_of_the_current_phase() {
    let normal = [
        Contract::Tri,
        Contract::Dve,
        Contract::Ena,
        Contract::SoloTri,
        Contract::SoloDve,
        Contract::SoloEna,
    ];
    run_cases(CASES, case(), |(seed, dealer, choices)| {
        play_random(seed, seat(dealer), &choices, |game| {
            // The ranges are written out from `docs/encoding.md` section 5,
            // not taken from the engine's constants.
            let range = match game.phase() {
                Phase::Bidding | Phase::ContractChoice => 0..13,
                Phase::KingCall => 13..17,
                Phase::TalonPick => 17..23,
                Phase::Discard => 23..77,
                // Announcements 77-81, kontras 82-87, "done" 88 (spec 009).
                Phase::Announcements => 77..89,
                Phase::Play => 90..144,
                Phase::Over => unreachable!("no action after the end"),
            };
            assert!(
                [
                    slots::BIDS,
                    slots::KINGS,
                    slots::PACKETS,
                    slots::DISCARDS,
                    slots::ANNOUNCEMENTS.start..slots::DONE + 1,
                    slots::PLAYS
                ]
                .contains(&range)
            );
            let legal = game.legal_actions();
            for action in legal.iter() {
                let slot = action.slot();
                assert!(slot < Action::SLOTS);
                assert!(range.contains(&slot), "{action:?} in {:?}", game.phase());
                // The slot names the action and nothing else.
                assert_eq!(Action::from_slot(slot), Some(action));
                assert!(legal.contains_slot(slot));
                // Things not built yet are never legal.
                if let Action::Bid(contract) = action {
                    assert!(normal.contains(&contract), "{contract:?} offered");
                }
            }
            // As a mask of 160 bits: exactly the legal slots are set.
            let set_slots = (0..Action::SLOTS)
                .filter(|&slot| legal.contains_slot(slot))
                .count();
            assert_eq!(set_slots, legal.len() as usize);
        });
        Ok(())
    });
}

/// Not a numbered criterion: the random deals of this file reach every
/// contract, every declarer seat, partner games, lone games and the called
/// king in the talon. Fixed seeds, so the counts do not change.
#[test]
fn random_deals_cover_contracts_seats_and_partner_cases() {
    let mut by_contract = [0u32; 12];
    let mut by_declarer_rank = [0u32; 4];
    let (mut partner, mut own_king, mut king_in_talon, mut rest_won) = (0u32, 0u32, 0u32, 0u32);
    let (mut mondfang, mut won, mut lost) = (0u32, 0u32, 0u32);
    for seed in 0..6_000u64 {
        let game = seeded_game(seed);
        let public = game.public();
        let declarer = public.declarer().unwrap();
        let contract = public.contract().unwrap();
        by_contract[contract.index()] += 1;
        by_declarer_rank[(declarer.index() + 4 - public.forehand().index()) % 4] += 1;
        if game.full().partner().is_some() {
            partner += 1;
        } else if public.king_in_talon() {
            king_in_talon += 1;
            rest_won += u32::from(game.full().talon_rest_won());
        } else if public.called_suit().is_some() {
            own_king += 1;
        }
        let score = game.score().unwrap();
        mondfang += u32::from(Seat::ALL.iter().any(|&s| score.parts(s).mondfang != 0));
        if score.declarer_won() {
            won += 1;
        } else {
            lost += 1;
        }
    }
    // Only the six normal contracts, each of them often.
    for contract in Contract::ALL {
        let count = by_contract[contract.index()];
        if contract.is_normal() {
            assert!(count >= 300, "{contract}: {count} deals");
        } else {
            assert_eq!(count, 0, "{contract} is not played in this slice");
        }
    }
    // Forehand, second, third and the dealer all declare.
    for (rank, &count) in by_declarer_rank.iter().enumerate() {
        assert!(
            count >= 500,
            "declarer {rank} seats after forehand: {count} deals"
        );
    }
    assert!(partner >= 1_000, "{partner} partner deals");
    assert!(
        own_king >= 100,
        "{own_king} deals with the declarer's own king called"
    );
    assert!(
        king_in_talon >= 100,
        "{king_in_talon} deals with the king in the talon"
    );
    assert!(
        rest_won >= 5,
        "{rest_won} deals with the rest of the talon won"
    );
    assert!(mondfang >= 100, "{mondfang} deals with a mondfang");
    assert!(won >= 500 && lost >= 500, "{won} won, {lost} lost");
}

// ---------------------------------------------------------------------
// Spec 009, criteria 41-44: whole deals with announcements and kontra
// ---------------------------------------------------------------------

/// A case of flow B: more choices, because a deal has more decisions.
fn case_b() -> impl Strategy<Value = (u64, u8, Vec<u8>)> {
    (
        any::<u64>(),
        0..4u8,
        proptest::collection::vec(any::<u8>(), 191),
    )
}

#[test]
fn c41_every_deal_of_flow_b_reaches_over_and_the_round_is_bounded() {
    run_cases(CASES, case_b(), |(seed, dealer, choices)| {
        let mut actions = 0u32;
        let mut said = 0u32;
        let mut last_phase = Phase::Bidding;
        let mut announced = [false; 5];
        let mut levels = [0u8; 6];
        let game = play_random_b(seed, seat(dealer), &choices, |game| {
            actions += 1;
            assert_ne!(game.phase(), Phase::Over);
            // The legal set is never empty before the end.
            assert!(!game.legal_actions().is_empty());
            assert!(game.to_act().is_some());
            assert!(game.score().is_none(), "no score before the end");
            assert!(order(game.phase()) >= order(last_phase));
            last_phase = game.phase();
            // What was said only grows: a bonus stays announced, a level
            // goes up by one at a time and never above mort.
            let now_announced = Bonus::ALL.map(|bonus| game.public().announced(bonus).is_some());
            let now_levels = Target::ALL.map(|target| game.public().kontra_level(target));
            let mut steps = 0;
            for index in 0..5 {
                assert!(now_announced[index] || !announced[index]);
                steps += u32::from(now_announced[index] && !announced[index]);
            }
            for index in 0..6 {
                assert!(now_levels[index] >= levels[index] && now_levels[index] <= 4);
                steps += u32::from(now_levels[index] - levels[index]);
            }
            assert!(steps <= 1, "one thing per action");
            said += steps;
            (announced, levels) = (now_announced, now_levels);
            if game.phase() == Phase::Announcements {
                assert!(game.legal_actions().contains(Action::Done));
            }
        });
        prop_assert_eq!(game.phase(), Phase::Over);
        prop_assert!(game.legal_actions().is_empty());
        prop_assert_eq!(game.to_act(), None);
        prop_assert!(game.score().is_some());

        // The round is bounded: at most 29 announcements and kontras.
        let all: Vec<Action> = game.full().actions().collect();
        let in_the_round = all
            .iter()
            .filter(|action| matches!(action, Action::Announce(_) | Action::Kontra(_)))
            .count();
        prop_assert!(in_the_round <= 29, "{} things said", in_the_round);
        // The last thing said is not seen by the callback: it is counted
        // from the final state.
        let final_levels: u32 = Target::ALL
            .iter()
            .map(|&target| u32::from(game.public().kontra_level(target)))
            .sum();
        let final_announced = Bonus::ALL
            .iter()
            .filter(|&&bonus| game.public().announced(bonus).is_some())
            .count();
        prop_assert_eq!(in_the_round, final_announced + final_levels as usize);
        prop_assert!(said as usize <= in_the_round);
        // And its turns: one "done" per turn, at most 3 x 29 + 4.
        let turns = all.iter().filter(|&&action| action == Action::Done).count();
        prop_assert!((4..=91).contains(&turns), "{} turns", turns);
        prop_assert!(all.len() <= tarok_engine::game::MAX_ACTIONS);
        prop_assert_eq!(actions as usize, all.len());
        Ok(())
    });
}

#[test]
fn c42_the_second_scorer_agrees_on_random_deals_of_flow_b() {
    run_cases(CASES, case_b(), |(seed, dealer, choices)| {
        let game = play_random_b(seed, seat(dealer), &choices, |_| {});
        // Both piles, both counts and all eight parts of all four entries.
        assert_scores_agree(&game);
        Ok(())
    });
}

#[test]
fn c43_declarer_and_partner_write_the_same_and_an_opponent_0_or_minus_20() {
    run_cases(CASES, case_b(), |(seed, dealer, choices)| {
        let game = play_random_b(seed, seat(dealer), &choices, |_| {});
        let score = game.score().unwrap();
        let declarer = game.public().declarer().unwrap();
        let mut mondfangs = 0;
        for seat in Seat::ALL {
            let parts = score.parts(seat);
            prop_assert!(parts.mondfang == 0 || parts.mondfang == -20);
            mondfangs += u32::from(parts.mondfang != 0);
            if game.full().on_declarer_side(seat) {
                // Equal apart from a mondfang.
                prop_assert_eq!(
                    score.entry(seat) - parts.mondfang,
                    score.entry(declarer) - score.parts(declarer).mondfang
                );
            } else {
                // An opponent writes 0, or -20 for a lost mond: no kontra
                // and no announcement changes that (S20 a).
                prop_assert!(score.entry(seat) == 0 || score.entry(seat) == -20);
                prop_assert_eq!(score.entry(seat), parts.mondfang);
            }
            prop_assert_eq!(parts.entry(), score.entry(seat));
        }
        prop_assert!(mondfangs <= 1);
        Ok(())
    });
}

#[test]
fn c44_fixed_seeds_cover_the_rules_of_flow_b() {
    // Who announced: the declarer, the partner, an opponent.
    let mut announced = [[0u32; 3]; 5];
    // Levels reached on the game and on any bonus: 1 to 4.
    let mut game_levels = [0u32; 5];
    let mut bonus_levels = [0u32; 5];
    let (mut valat_won, mut valat_lost) = (0u32, 0u32);
    let (mut pagat_early, mut king_early) = (0u32, 0u32);
    let (mut pagat_made, mut king_made) = (0u32, 0u32);
    let mut partner_shown = 0u32;

    for seed in 0..10_000u64 {
        let game = seeded_game_b(seed);
        let public = game.public();
        let full = game.full();
        let declarer = public.declarer().unwrap();
        let score = game.score().unwrap();
        for bonus in Bonus::ALL {
            let Some(seat) = public.announced(bonus) else {
                continue;
            };
            let who = if seat == declarer {
                0
            } else if full.partner() == Some(seat) {
                1
            } else {
                2
            };
            announced[bonus.index()][who] += 1;
        }
        game_levels[usize::from(public.kontra_level(Target::Game))] += 1;
        for target in Target::ALL {
            if target != Target::Game && public.announced(target.bonus().unwrap()).is_some() {
                bonus_levels[usize::from(public.kontra_level(target))] += 1;
            }
        }
        if let Some(seat) = public.announced(Bonus::Valat) {
            // Won: the announcer's side wrote plus.
            let owner_sign = if full.on_declarer_side(seat) { 1 } else { -1 };
            if score.parts(declarer).valat * owner_sign > 0 {
                valat_won += 1;
            } else {
                valat_lost += 1;
            }
        }
        let trick_of = |card: Card| {
            public
                .tricks()
                .iter()
                .position(|trick| trick.cards.contains(&card))
        };
        // Without an announced valat the ultimo bonuses are scored.
        let no_valat = score.parts(declarer).valat == 0;
        if public.announced(Bonus::PagatUltimo).is_some() {
            pagat_early += u32::from(trick_of(Card::PAGAT) != Some(11));
            pagat_made += u32::from(no_valat && score.parts(declarer).pagat_ultimo.abs() >= 50);
        }
        if public.announced(Bonus::KraljUltimo).is_some() {
            let king = public.called_king().unwrap();
            king_early += u32::from(trick_of(king) != Some(11));
            king_made += u32::from(no_valat && score.parts(declarer).kralj_ultimo > 0);
        }
        // A partner shown by the round alone: public before its king fell.
        partner_shown += u32::from(
            full.partner().is_some()
                && public
                    .announced(Bonus::KraljUltimo)
                    .is_some_and(|seat| seat != declarer),
        );
    }
    println!(
        "10,000 seeded deals of flow B. Announced by declarer / partner / opponent: trula \
         {:?}, kralji {:?}, kralj ultimo {:?}, pagat ultimo {:?}, valat {:?}. Game at level 0 \
         to 4: {game_levels:?}. Announced bonuses at level 0 to 4: {bonus_levels:?}. Announced \
         valat won {valat_won}, lost {valat_lost}. Announced pagat played before the last \
         trick {pagat_early}, announced king {king_early}; announced pagat ultimo scored \
         {pagat_made}, kralj ultimo won {king_made}. Partner shown by kralj ultimo \
         {partner_shown}.",
        announced[0], announced[1], announced[2], announced[3], announced[4]
    );

    // Each of the five announcements, by the declarer, by the partner and
    // by an opponent where the rules allow it.
    for bonus in [
        Bonus::Trula,
        Bonus::Kralji,
        Bonus::PagatUltimo,
        Bonus::Valat,
    ] {
        for (who, count) in announced[bonus.index()].into_iter().enumerate() {
            assert!(count >= 20, "{bonus:?} by {who}: {count}");
        }
    }
    // Kralj ultimo: by the declarer or the partner, never by an opponent.
    let [by_declarer, by_partner, by_opponent] = announced[Bonus::KraljUltimo.index()];
    assert!(by_declarer >= 20 && by_partner >= 20);
    assert_eq!(by_opponent, 0);
    // Each kontra level on the game and on a bonus.
    for level in 1..=4 {
        assert!(game_levels[level] >= 20, "game at level {level}");
        assert!(bonus_levels[level] >= 20, "a bonus at level {level}");
    }
    // An announced valat won and lost.
    assert!(valat_won >= 3, "{valat_won} announced valats won");
    assert!(valat_lost >= 100);
    // An announced ultimo card played before the last trick.
    assert!(pagat_early >= 100 && king_early >= 100);
    assert!(pagat_made >= 20 && king_made >= 3);
    assert!(partner_shown >= 20);
}
