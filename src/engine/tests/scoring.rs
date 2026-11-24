//! Spec 005, criteria 29-44: scoring of the six normal contracts, with the
//! rulings of spec 013: a valat keeps the mondfang (D125, the `d125_`
//! tests), pagat ultimo follows the side (D126, `c39`), and in a valat the
//! declarer wins the packets not taken (D133, the `d133_` test).
//!
//! Seats: A is forehand (seat 0), then B, C, D; D (seat 3) deals.
//!
//! Most tests build a deal in which the wanted event cannot be avoided: one
//! seat holds a hand that wins by force, and its cards are played in a
//! written order. The other seats play their lowest legal card. The built
//! deals are in `common::built` (moved there with spec 009).

mod common;

use common::built::*;
use common::flow::{
    A, B, C, D, NORMAL, act, deal_with, declare, exchange, exchange_back, finish, game,
    king_taken_and_led, play_random, seeded_game, whole_deal,
};
use common::second_scorer::{assert_scores_agree, second_score};
use common::written::{deal_1_partner, deal_2_solo, deal_3_king_in_talon};
use common::{SUITS, kralj, run_cases, seat, tarok};
use proptest::prelude::*;
use tarok_engine::Contract::{Dve, Ena, SoloDve, SoloEna, SoloTri, Tri};
use tarok_engine::{
    Action, Card, CardSet, Contract, Deal, Game, PublicPartner, Score, Seat, Suit, points, score,
};

fn case() -> impl Strategy<Value = (u64, u8, Vec<u8>)> {
    (
        any::<u64>(),
        0..4u8,
        proptest::collection::vec(any::<u8>(), 96),
    )
}

// ---------------------------------------------------------------------
// Criteria
// ---------------------------------------------------------------------

#[test]
fn c29_each_side_counts_its_cards_and_the_counts_sum_to_70() {
    run_cases(3_000, case(), |(seed, dealer, choices)| {
        let game = play_random(seed, seat(dealer), &choices, |_| {});
        // The second scorer of the tests collects the two piles one by one
        // from the record of the deal: for the declarer's side the tricks
        // of the declarer and the partner, the discards, and the rest of
        // the talon in the case of criterion 23; for the opponents their
        // tricks and the talon cards that stayed with them. It compares
        // both piles, both counts and every part of every entry with the
        // engine.
        let second = assert_scores_agree(&game);
        // Every card is in one pile, and the two counts sum to 70.
        prop_assert!(second.declarer_pile.is_disjoint(second.opponents_pile));
        prop_assert_eq!(second.declarer_pile | second.opponents_pile, CardSet::FULL);
        prop_assert_eq!(
            points::count(second.declarer_pile) + points::count(second.opponents_pile),
            70
        );
        Ok(())
    });
}

#[test]
fn c30_declarer_wins_with_36_or_more_and_35_is_a_loss() {
    for contract in NORMAL {
        for points in 0..=70u8 {
            let (game, _) = score::game_and_difference(contract, points).unwrap();
            if points >= 36 {
                assert!(game > 0, "{contract:?} with {points} is won");
            } else {
                assert!(game < 0, "{contract:?} with {points} is lost");
            }
        }
    }
    // In whole deals: the written deals with 52, 31 and 54 points.
    let (game, _) = deal_1_partner();
    assert!(game.score().unwrap().declarer_won());
    let (game, _) = deal_2_solo();
    assert!(!game.score().unwrap().declarer_won());
    let (game, _) = deal_3_king_in_talon();
    assert!(game.score().unwrap().declarer_won());

    // The edge in whole deals, by the entries written: 36 points win and
    // 35 points lose.
    whole_deal_case(548, Tri, 36, [0, 10, 10, 0]);
    whole_deal_case(69, Tri, 35, [-10, 0, -10, 0]);
    whole_deal_case(1049, SoloTri, 36, [0, 0, 40, 0]);
    whole_deal_case(476, Dve, 35, [-20, 0, -20, 0]);
}

/// A whole deal that ends with a wanted contract and count and with no
/// bonus and no mondfang, so the entries show the game and the difference
/// alone. The deal is `seeded_game(seed)`: fixed cards, bids and plays. It
/// was found by trying seeds; what it must give is written here from the
/// criteria. The count is taken from the second scorer of the tests, which
/// collects the piles from the tricks on its own.
fn whole_deal_case(seed: u64, contract: Contract, points: u8, entries: [i32; 4]) {
    let game = seeded_game(seed);
    assert_eq!(game.public().contract(), Some(contract), "seed {seed}");
    let second = second_score(&game);
    assert_eq!(second.declarer_points, points, "seed {seed}");
    assert_eq!(second.opponents_points, 70 - points, "seed {seed}");
    // Nothing but game and difference in any entry.
    for parts in second.parts {
        assert_eq!(parts[2..], [0; 6], "seed {seed}: a bonus or a mondfang");
    }
    let score = game.score().unwrap();
    assert_eq!(score.declarer_points(), points, "seed {seed}");
    assert_eq!(score.declarer_won(), points >= 36, "seed {seed}");
    assert_eq!(score.entries(), entries, "seed {seed}");
}

#[test]
fn c31_difference_is_points_minus_35_rounded_to_the_nearest_5() {
    // The source examples.
    assert_eq!(score::rounded_difference(43), 10);
    assert_eq!(score::rounded_difference(37), 0);
    assert_eq!(score::rounded_difference(21), -15);

    // Written out for every count around the middle.
    let around: [(u8, i32); 11] = [
        (30, -5),
        (31, -5),
        (32, -5),
        (33, 0),
        (34, 0),
        (35, 0),
        (36, 0),
        (37, 0),
        (38, 5),
        (39, 5),
        (40, 5),
    ];
    for (points, expected) in around {
        assert_eq!(score::rounded_difference(points), expected, "{points}");
    }
    assert_eq!(score::rounded_difference(0), -35);
    assert_eq!(score::rounded_difference(70), 35);

    // For every count: a multiple of 5, and no other multiple is nearer.
    for points in 0..=70u8 {
        let exact = i32::from(points) - 35;
        let rounded = score::rounded_difference(points);
        assert_eq!(rounded % 5, 0, "{points}");
        assert!((rounded - exact).abs() <= 2, "{points}: {rounded}");
    }
}

#[test]
fn c32_game_values() {
    let values = [
        (Tri, 10),
        (Dve, 20),
        (Ena, 30),
        (SoloTri, 40),
        (SoloDve, 50),
        (SoloEna, 60),
    ];
    for (contract, value) in values {
        assert_eq!(contract.game_value(), Some(value));
        // A bare win and a bare loss (difference 0) are plus and minus it.
        assert_eq!(score::game_and_difference(contract, 36), Some((value, 0)));
        assert_eq!(score::game_and_difference(contract, 35), Some((-value, 0)));
    }
    // The other contracts have no value in this slice.
    for contract in Contract::ALL {
        if !NORMAL.contains(&contract) {
            assert_eq!(contract.game_value(), None);
            assert_eq!(score::game_and_difference(contract, 40), None);
        }
    }
}

#[test]
fn c33_a_win_scores_game_value_plus_rounded_difference() {
    // The source example: ena won with 43 points scores 40.
    let (game, difference) = score::game_and_difference(Ena, 43).unwrap();
    assert_eq!((game, difference), (30, 10));
    assert_eq!(game + difference, 40);

    assert_eq!(score::game_and_difference(Tri, 36), Some((10, 0)));
    assert_eq!(score::game_and_difference(Tri, 70), Some((10, 35)));
    assert_eq!(score::game_and_difference(SoloEna, 48), Some((60, 15)));

    // Whole deals. Deal 1: tri won with 52 points, 10 + 15, plus a trula.
    let (game, _) = deal_1_partner();
    let score = game.score().unwrap();
    assert_eq!(score.declarer_points(), 52);
    assert_eq!((score.parts(A).game, score.parts(A).difference), (10, 15));
    assert_eq!(score.entry(A), 10 + 15 + 10);
    // Deal 3: dve won with 54 points, 20 + 20, plus the kings.
    let (game, _) = deal_3_king_in_talon();
    let score = game.score().unwrap();
    assert_eq!(score.declarer_points(), 54);
    assert_eq!((score.parts(D).game, score.parts(D).difference), (20, 20));

    // The source example as a whole deal: ena won with 43 points. The
    // declarer (seat 0) and the partner (seat 2) write 40.
    whole_deal_case(115, Ena, 43, [40, 0, 40, 0]);
}

#[test]
fn c34_a_loss_scores_minus_game_value_and_difference() {
    // The source examples: dve lost with 21 points scores -35; tri with
    // exactly 35 points scores -10.
    let (game, difference) = score::game_and_difference(Dve, 21).unwrap();
    assert_eq!((game, difference), (-20, -15));
    assert_eq!(game + difference, -35);
    let (game, difference) = score::game_and_difference(Tri, 35).unwrap();
    assert_eq!(game + difference, -10);

    assert_eq!(score::game_and_difference(SoloEna, 0), Some((-60, -35)));
    assert_eq!(score::game_and_difference(Ena, 33), Some((-30, 0)));

    // A whole deal. Deal 2: solo tri lost with 31 points: -(40 + 5).
    let (game, _) = deal_2_solo();
    let score = game.score().unwrap();
    assert_eq!(score.declarer_points(), 31);
    assert_eq!((score.parts(B).game, score.parts(B).difference), (-40, -5));

    // The source examples as whole deals: dve lost with 21 points, the
    // declarer (seat 2) and the partner (seat 3) write -35; tri with
    // exactly 35 points, the declarer (seat 2) and the partner (seat 0)
    // write -10.
    whole_deal_case(5083, Dve, 21, [0, 0, -35, -35]);
    whole_deal_case(69, Tri, 35, [-10, 0, -10, 0]);
}

#[test]
fn c35_partner_writes_the_same_entry_and_opponents_write_no_game() {
    // The written partner deal: A and C write 35, B and D write 0.
    let (game, _) = deal_1_partner();
    let score = game.score().unwrap();
    assert_eq!(game.full().partner(), Some(C));
    assert_eq!(score.parts(A), score.parts(C));
    assert_eq!(score.entries(), [35, 0, 35, 0]);

    // Random deals with a partner.
    let with_partner = std::cell::Cell::new(0u32);
    run_cases(3_000, case(), |(seed, dealer, choices)| {
        let game = play_random(seed, seat(dealer), &choices, |_| {});
        let score = game.score().unwrap();
        let declarer = game.public().declarer().unwrap();
        let partner = game.full().partner();
        if partner.is_some() {
            with_partner.set(with_partner.get() + 1);
        }
        for seat in Seat::ALL {
            let parts = score.parts(seat);
            if Some(seat) == partner {
                // The same entry as the declarer, part by part. Only the
                // mondfang is each player's own.
                let mut same = *score.parts(declarer);
                same.mondfang = parts.mondfang;
                prop_assert_eq!(*parts, same);
            } else if seat != declarer {
                // An opponent: nothing for the game or the bonuses.
                prop_assert_eq!(
                    [
                        parts.game,
                        parts.difference,
                        parts.trula,
                        parts.kralji,
                        parts.kralj_ultimo,
                        parts.pagat_ultimo,
                        parts.valat
                    ],
                    [0; 7]
                );
            }
        }
        Ok(())
    });
    assert!(with_partner.get() > 300, "{} deals", with_partner.get());
}

#[test]
fn c36_silent_trula() {
    // +10: škis, mond and pagat all in the declarer's tricks.
    let (game, score) = trula_for_a();
    assert_eq!(tricks_won(&game), [11, 1, 0, 0]);
    assert_eq!(score.parts(A).valat, 0);
    for card in [S, MOND, PAGAT] {
        assert!(game.public().tricks_won(A).contains(card));
    }
    assert_eq!(score.parts(A).trula, 10);
    for seat in [B, C, D] {
        assert_eq!(score.parts(seat).trula, 0);
    }

    // -10: all three in the opponents' tricks.
    let (game, score) = trula_for_the_opponents();
    assert_eq!(tricks_won(&game), [1, 11, 0, 0]);
    for card in [S, MOND, PAGAT] {
        assert!(game.public().tricks_won(B).contains(card));
    }
    assert_eq!(score.parts(A).trula, -10);

    // -10 also when the third card lies in the talon that was not taken
    // (answer B). The pagat is in the first packet; A takes the second.
    // B holds škis and mond; each wins the trick it is played to.
    let deal = deal_with(
        [&taroks([20, 19, 18, 17]), &[S, MOND], &[], &[]],
        &[PAGAT, karo(4), karo(3), srce(4), srce(3), srce(2)],
    );
    let mut game = common::flow::game(&deal);
    declare(&mut game, A, SoloTri);
    exchange_back(&mut game, 1);
    finish(&mut game);
    let score = game.score().unwrap();
    assert_eq!(score.parts(A).valat, 0);
    assert!(game.full().opponents_pile().contains(PAGAT));
    assert!(game.public().tricks_won(B).contains(S));
    assert!(game.public().tricks_won(B).contains(MOND));
    assert_eq!(score.parts(A).trula, -10);

    // 0 when the three are split: A wins škis and mond, D wins the pagat.
    let (game, score) = pagat_ultimo_own_loses();
    assert!(game.public().tricks_won(A).contains(S));
    assert!(game.public().tricks_won(A).contains(MOND));
    assert!(game.public().tricks_won(D).contains(PAGAT));
    assert_eq!(score.parts(A).trula, 0);
}

#[test]
fn c37_silent_kralji() {
    let kings: CardSet = SUITS.into_iter().map(kralj).collect();

    // +10: the four kings in the declarer's tricks.
    let (game, score) = kings_for_a();
    assert_eq!(tricks_won(&game), [11, 1, 0, 0]);
    assert!(kings.is_subset(game.public().tricks_won(A)));
    assert_eq!(score.parts(A).kralji, 10);
    for seat in [B, C, D] {
        assert_eq!(score.parts(seat).kralji, 0);
    }

    // -10: the four kings in the opponents' tricks.
    let (game, score) = kings_for_the_opponents(false);
    assert_eq!(tricks_won(&game), [1, 11, 0, 0]);
    assert!(kings.is_subset(game.public().tricks_won(B)));
    assert_eq!(score.parts(A).kralji, -10);

    // -10 also when one king lies in the talon that was not taken
    // (answer B).
    let (game, score) = kings_for_the_opponents(true);
    assert_eq!(tricks_won(&game), [1, 11, 0, 0]);
    assert_eq!(game.public().tricks_won(B).intersection(kings).len(), 3);
    assert!(game.full().opponents_pile().contains(kralj(Suit::Srce)));
    assert!(!game.public().taken_cards().contains(kralj(Suit::Srce)));
    assert_eq!(score.parts(A).kralji, -10);

    // 0 when the kings are split: A wins the srce kralj, B the others.
    let (game, score) = trula_for_the_opponents();
    assert!(game.public().tricks_won(A).contains(kralj(Suit::Srce)));
    assert_eq!(score.parts(A).kralji, 0);
}

#[test]
fn c38_silent_kralj_ultimo() {
    let king = kralj(Suit::Srce);

    // +10: A calls srce, holds the srce kralj (answer C) and wins the last
    // trick with it.
    let game = a_declares(&ultimo_deal(true), Tri, Some(Suit::Srce));
    let (game, score) = finished(game, [&ultimo_script_a(), &ultimo_script_b(true), &[], &[]]);
    assert_eq!(tricks_won(&game), [11, 1, 0, 0]);
    assert_eq!(trick_of(&game, king), 12);
    assert_eq!(game.public().tricks()[11].winner, A);
    assert_eq!(score.parts(A).kralj_ultimo, 10);
    for seat in [B, C, D] {
        assert_eq!(score.parts(seat).kralj_ultimo, 0);
    }

    // -10: the same, but B leads a pik to the last trick and wins it.
    let game = a_declares(&ultimo_deal(true), Tri, Some(Suit::Srce));
    let (game, score) = finished(
        game,
        [&ultimo_script_a(), &ultimo_script_b(false), &[], &[]],
    );
    assert_eq!(trick_of(&game, king), 12);
    assert_eq!(game.public().tricks()[11].winner, B);
    assert_eq!(score.parts(A).kralj_ultimo, -10);

    // +10 with a partner: C holds the called king and wins the last trick
    // with it. Both write the bonus.
    let game = a_declares(&ultimo_deal(false), Tri, Some(Suit::Srce));
    assert_eq!(game.full().partner(), Some(C));
    let mut a_script = top_taroks(13);
    a_script.extend([karo(4), pik(8)]);
    let mut b_script = taroks(2..=5);
    b_script.extend([
        pik(7),
        pik(9),
        pik(10),
        fant(Suit::Pik),
        kaval(Suit::Pik),
        dama(Suit::Pik),
        karo(3),
        dama(Suit::Srce),
    ]);
    let mut c_script = taroks(6..=9);
    c_script.extend([
        srce(4),
        srce(3),
        srce(2),
        kriz(7),
        kriz(8),
        kriz(9),
        kriz(10),
        king,
    ]);
    let (game, score) = finished(game, [&a_script, &b_script, &c_script, &[]]);
    assert_eq!(trick_of(&game, king), 12);
    assert_eq!(game.public().tricks()[11].winner, C);
    assert_eq!(game.public().partner(), PublicPartner::With(C));
    assert_eq!(score.parts(A).kralj_ultimo, 10);
    assert_eq!(score.parts(C).kralj_ultimo, 10);
    assert_eq!(score.entry(A), score.entry(C));

    // 0 when the called king is played earlier: deal 1, first trick.
    let (game, _) = deal_1_partner();
    assert_eq!(trick_of(&game, kralj(Suit::Kriz)), 1);
    assert_eq!(game.score().unwrap().parts(A).kralj_ultimo, 0);

    // Never in a solo contract: the same last trick, no bonus.
    let game = a_declares(&ultimo_deal(true), SoloTri, None);
    let (game, score) = finished(game, [&ultimo_script_a(), &ultimo_script_b(true), &[], &[]]);
    assert_eq!(trick_of(&game, king), 12);
    assert_eq!(game.public().tricks()[11].winner, A);
    assert_eq!(score.parts(A).kralj_ultimo, 0);
}

/// Criterion 39 as changed by spec 013 (criteria 4 to 7, D126): pagat
/// ultimo follows the side that takes the last trick, like kralj ultimo.
#[test]
fn c39_silent_pagat_ultimo() {
    let last_trick = |game: &Game| game.public().tricks()[11];

    // 1. +25: the declarer's pagat wins the last trick.
    let (game, score) = pagat_ultimo_own_wins();
    assert_eq!(trick_of(&game, PAGAT), 12);
    assert_eq!(last_trick(&game).winner, A);
    assert_eq!(last_trick(&game).cards[3], PAGAT);
    assert_eq!(score.parts(A).valat, 0);
    assert_eq!(score.parts(A).pagat_ultimo, 25);
    for seat in [B, C, D] {
        assert_eq!(score.parts(seat).pagat_ultimo, 0);
    }

    // 2. +25: the partner C plays the pagat to the last trick and the
    // declarer A takes the trick with the XIX. The side takes the trick
    // with another card and wins the bonus. (Spec 005 scored this -25.)
    let game = a_declares(&pagat_and_a_higher_tarok(A), Tri, Some(Suit::Kriz));
    assert_eq!(game.full().partner(), Some(C));
    let (game, score) = finished(game, pagat_and_a_higher_tarok_scripts(A));
    assert_eq!(tricks_won(&game), [10, 1, 1, 0]);
    assert_eq!(trick_of(&game, PAGAT), 12);
    assert_eq!(last_trick(&game).cards[2], PAGAT);
    assert_eq!(last_trick(&game).winner, A);
    assert_eq!(score.parts(A).valat, 0);
    assert_eq!(score.parts(A).pagat_ultimo, 25);
    assert_eq!(score.parts(C).pagat_ultimo, 25);
    assert_eq!(score.entry(A), score.entry(C));
    assert_eq!(
        [score.parts(B).pagat_ultimo, score.parts(D).pagat_ultimo],
        [0, 0]
    );

    // 3. +25: the same cards and the same play, but C is the declarer (dve,
    // srce called) and A, who holds the srce kralj, is the partner. The
    // declarer's pagat is taken by the partner's XIX.
    let mut game = common::flow::game(&pagat_and_a_higher_tarok(A));
    declare(&mut game, C, Dve);
    act(&mut game, C, Action::CallKing(Suit::Srce));
    exchange_back(&mut game, 0);
    assert_eq!(game.full().partner(), Some(A));
    let (game, score) = finished(game, pagat_and_a_higher_tarok_scripts(A));
    assert_eq!(tricks_won(&game), [10, 1, 1, 0]);
    assert_eq!(game.public().declarer(), Some(C));
    assert_eq!(last_trick(&game).cards[2], PAGAT);
    assert_eq!(last_trick(&game).winner, A);
    assert_eq!(score.parts(C).valat, 0);
    assert_eq!(score.parts(C).pagat_ultimo, 25);
    assert_eq!(score.parts(A).pagat_ultimo, 25);
    assert_eq!(
        [score.parts(B).pagat_ultimo, score.parts(D).pagat_ultimo],
        [0, 0]
    );

    // 4. -25: an opponent's pagat is taken by another opponent. A plays
    // solo, so B, C and D are one side: C plays the pagat, D takes the
    // trick with the XIX, and the opponents win the bonus. (Spec 005
    // scored this +25.)
    let game = a_declares(&pagat_and_a_higher_tarok(D), SoloTri, None);
    let (game, score) = finished(game, pagat_and_a_higher_tarok_scripts(D));
    assert_eq!(tricks_won(&game), [9, 1, 1, 1]);
    assert_eq!(trick_of(&game, PAGAT), 12);
    assert_eq!(last_trick(&game).cards[2], PAGAT);
    assert_eq!(last_trick(&game).winner, D);
    assert_eq!(score.parts(A).pagat_ultimo, -25);
    for seat in [B, C, D] {
        assert_eq!(score.parts(seat).pagat_ultimo, 0);
    }

    // The same in tri when A calls its own srce kralj: A is alone, C and D
    // are opponents again.
    let game = a_declares(&pagat_and_a_higher_tarok(D), Tri, Some(Suit::Srce));
    assert_eq!(game.full().partner(), None);
    let (game, score) = finished(game, pagat_and_a_higher_tarok_scripts(D));
    assert_eq!(last_trick(&game).winner, D);
    assert_eq!(score.parts(A).pagat_ultimo, -25);

    // -25: an opponent's pagat wins the last trick itself.
    let (game, score) = trula_for_the_opponents();
    assert_eq!(trick_of(&game, PAGAT), 12);
    assert_eq!(last_trick(&game).winner, B);
    assert_eq!(last_trick(&game).cards[0], PAGAT);
    assert_eq!(score.parts(A).valat, 0);
    assert_eq!(score.parts(A).pagat_ultimo, -25);

    // 5. -25: the declarer's pagat is played to the last trick and an
    // opponent takes it.
    let (game, score) = pagat_ultimo_own_loses();
    assert_eq!(trick_of(&game, PAGAT), 12);
    assert_eq!(last_trick(&game).winner, D);
    assert!(game.private(A).hand().is_empty());
    assert_eq!(score.parts(A).pagat_ultimo, -25);

    // -25: the partner's pagat, taken by an opponent. Tri with križ
    // called: C holds the križ kralj and the pagat, D takes the last trick.
    let game = a_declares(&pagat_and_a_higher_tarok(D), Tri, Some(Suit::Kriz));
    assert_eq!(game.full().partner(), Some(C));
    let (game, score) = finished(game, pagat_and_a_higher_tarok_scripts(D));
    assert_eq!(last_trick(&game).winner, D);
    assert_eq!(score.parts(A).pagat_ultimo, -25);
    assert_eq!(score.parts(C).pagat_ultimo, -25);

    // 6. +25: an opponent's pagat is taken by the declarer's side.
    let (game, score) = pagat_ultimo_opponent_loses();
    assert_eq!(trick_of(&game, PAGAT), 12);
    assert_eq!(last_trick(&game).cards[0], PAGAT);
    assert_eq!(last_trick(&game).winner, A);
    assert_eq!(score.parts(A).valat, 0);
    assert_eq!(score.parts(A).pagat_ultimo, 25);

    // 7. 0 when the pagat falls earlier: trick 11 here, trick 6 in deal 1.
    let (game, score) = trula_for_a();
    assert_eq!(trick_of(&game, PAGAT), 11);
    assert_eq!(score.parts(A).pagat_ultimo, 0);
    let (game, _) = deal_1_partner();
    assert_eq!(trick_of(&game, PAGAT), 6);
    assert_eq!(game.score().unwrap().parts(A).pagat_ultimo, 0);
}

#[test]
fn c40_game_and_bonuses_are_independent() {
    // Deal 2: B loses solo tri with 31 points and has the whole trula.
    let (game, _) = deal_2_solo();
    let score = game.score().unwrap();
    let parts = score.parts(B);
    assert!(!score.declarer_won());
    assert_eq!((parts.game, parts.difference), (-40, -5));
    assert_eq!(parts.trula, 10);
    // The loss plus 10.
    assert_eq!(score.entry(B), -45 + 10);

    // A won game with a bonus on top: A wins solo ena and has the four
    // kings.
    let (_, score) = kings_for_a();
    let parts = score.parts(A);
    assert!(parts.game > 0);
    assert_eq!(parts.kralji, 10);

    // The entry is always the sum of the parts.
    for (_, score) in [
        trula_for_a(),
        trula_for_the_opponents(),
        kings_for_a(),
        kings_for_the_opponents(false),
        pagat_ultimo_own_wins(),
        pagat_ultimo_own_loses(),
    ] {
        for seat in Seat::ALL {
            let p = score.parts(seat);
            let sum = p.game
                + p.difference
                + p.trula
                + p.kralji
                + p.kralj_ultimo
                + p.pagat_ultimo
                + p.valat
                + p.mondfang;
            assert_eq!(score.entry(seat), sum);
            assert_eq!(p.entry(), sum);
        }
    }
    // A lost game with two bonuses against: every part is there.
    let (_, score) = trula_for_the_opponents();
    let parts = score.parts(A);
    assert!(parts.game < 0);
    assert_eq!(parts.trula, -10);
    assert_eq!(parts.pagat_ultimo, -25);
    assert_eq!(score.entry(A), parts.game + parts.difference - 10 - 25);
}

/// A holds škis, mond, XX-XII and the pagat: twelve taroks that win twelve
/// tricks from the lead.
fn all_twelve_for_a() -> Deal {
    let mut a = top_taroks(12);
    a.push(PAGAT);
    deal_with([&a, &[], &[], &[]], &low_talon())
}

/// B holds the same twelve taroks; A is the declarer.
fn all_twelve_for_b() -> Deal {
    let mut b = top_taroks(12);
    b.push(PAGAT);
    deal_with([&[], &b, &[], &[]], &low_talon())
}

/// Škis, mond, XX-XII from the top, then the pagat.
fn twelve_from_the_top() -> Vec<Card> {
    let mut script = top_taroks(12);
    script.push(PAGAT);
    script
}

/// A seat's parts in a deal in which one side took all twelve tricks: the
/// valat, the mondfang if this seat lost the mond, and nothing else (spec
/// 013, criterion 2).
fn valat_and_mondfang(score: &Score, seat: Seat, valat: i32, mondfang: i32) {
    let parts = score.parts(seat);
    assert_eq!(parts.valat, valat, "valat of seat {}", seat.index());
    assert_eq!(
        parts.mondfang,
        mondfang,
        "mondfang of seat {}",
        seat.index()
    );
    assert_eq!(
        [
            parts.game,
            parts.difference,
            parts.trula,
            parts.kralji,
            parts.kralj_ultimo,
            parts.pagat_ultimo,
        ],
        [0; 6],
        "no game, no difference and no other bonus for seat {}",
        seat.index()
    );
    assert_eq!(score.entry(seat), valat + mondfang);
}

/// A valat in which this seat lost no mond: the 250 and nothing else.
fn only_valat(score: &Score, seat: Seat, value: i32) {
    valat_and_mondfang(score, seat, value, 0);
}

#[test]
fn c41_silent_valat_scores_250_and_nothing_else() {
    // Alone, in solo ena.
    let game = a_declares(&all_twelve_for_a(), SoloEna, None);
    let (game, score) = finished(game, [&twelve_from_the_top(), &[], &[], &[]]);
    assert_eq!(tricks_won(&game), [12, 0, 0, 0]);
    // The trula is A's and the pagat wins the last trick: neither scores.
    assert_eq!(trick_of(&game, PAGAT), 12);
    only_valat(&score, A, 250);
    assert_eq!(score.entries(), [250, 0, 0, 0]);

    // With a partner, in tri: C holds the called srce kralj. The partner
    // writes the same.
    let game = a_declares(&all_twelve_for_a(), Tri, Some(Suit::Srce));
    assert_eq!(game.full().partner(), Some(C));
    let (game, score) = finished(game, [&twelve_from_the_top(), &[], &[], &[]]);
    assert_eq!(tricks_won(&game), [12, 0, 0, 0]);
    only_valat(&score, A, 250);
    only_valat(&score, C, 250);
    assert_eq!(score.entries(), [250, 0, 250, 0]);

    // Eleven tricks are not a valat: the game is scored as usual.
    let (game, score) = trula_for_a();
    assert_eq!(tricks_won(&game), [11, 1, 0, 0]);
    assert_eq!(score.parts(A).valat, 0);
    assert!(score.parts(A).game > 0);
}

#[test]
fn c41_opponents_take_all_twelve_tricks_scores_minus_250() {
    // Answer A (D118). A plays solo ena; B trumps the first trick and takes
    // every trick.
    let game = a_declares(&all_twelve_for_b(), SoloEna, None);
    let mut b_script = vec![tarok(12), S];
    b_script.extend((13..=21).rev().map(tarok));
    b_script.push(PAGAT);
    let (game, score) = finished(game, [&[], &b_script, &[], &[]]);
    assert_eq!(tricks_won(&game), [0, 12, 0, 0]);
    // An opponent's pagat wins the last trick and the trula is theirs:
    // neither is scored.
    assert_eq!(trick_of(&game, PAGAT), 12);
    only_valat(&score, A, -250);
    assert_eq!(score.entries(), [-250, 0, 0, 0]);

    // With a partner, in tri: A calls karo, C holds the karo kralj. Both
    // write -250.
    let game = a_declares(&all_twelve_for_b(), Tri, Some(Suit::Karo));
    assert_eq!(game.full().partner(), Some(C));
    let (game, score) = finished(game, [&[], &b_script, &[], &[]]);
    assert_eq!(tricks_won(&game), [0, 12, 0, 0]);
    only_valat(&score, A, -250);
    only_valat(&score, C, -250);
    assert_eq!(score.entries(), [-250, 0, -250, 0]);

    // Eleven tricks for the opponents are not a valat against.
    let (game, score) = trula_for_the_opponents();
    assert_eq!(tricks_won(&game), [1, 11, 0, 0]);
    assert_eq!(score.parts(A).valat, 0);
    assert!(score.parts(A).game < 0);
}

#[test]
fn c42_mondfang_the_mond_player_writes_minus_20() {
    // An opponent's škis takes the declarer's mond.
    let deal = deal_with([&[MOND], &[S], &[], &[]], &low_talon());
    let game = a_declares(&deal, SoloEna, None);
    let (game, score) = finished(game, [&[MOND], &[S], &[], &[]]);
    assert_eq!(skis_mond_trick(&game), Some(0));
    assert_eq!(game.public().tricks()[0].winner, B);
    assert_eq!(score.parts(A).valat, 0);
    assert_eq!(score.parts(A).mondfang, -20);
    // Alone: nobody else writes it.
    for seat in [B, C, D] {
        assert_eq!(score.parts(seat).mondfang, 0);
        assert_eq!(score.entry(seat), 0);
    }
    let parts = score.parts(A);
    assert_eq!(score.entry(A), parts.entry());
    assert_eq!(
        parts.entry(),
        parts.game + parts.difference + parts.trula + parts.kralji + parts.pagat_ultimo - 20
    );

    // The partner's škis takes the declarer's mond: the declarer writes
    // -20 all the same, the partner does not.
    let deal = deal_with([&[MOND], &[], &[S, kralj(Suit::Srce)], &[]], &low_talon());
    let game = a_declares(&deal, Tri, Some(Suit::Srce));
    assert_eq!(game.full().partner(), Some(C));
    let (game, score) = finished(game, [&[MOND], &[], &[S], &[]]);
    assert_eq!(skis_mond_trick(&game), Some(0));
    assert_eq!(game.public().tricks()[0].winner, C);
    assert_eq!(score.parts(A).valat, 0);
    assert_eq!(score.parts(A).mondfang, -20);
    assert_eq!(score.parts(C).mondfang, 0);
    assert_eq!(score.entry(A), score.entry(C) - 20);
    assert_eq!([score.entry(B), score.entry(D)], [0, 0]);

    // Škis, mond and pagat in one trick, the pagat last: the pagat wins
    // it and the mond's player still writes -20 (S22 a; spec 013,
    // criterion 10).
    let deal = deal_with([&[MOND], &[S], &[PAGAT], &[]], &low_talon());
    let game = a_declares(&deal, SoloEna, None);
    let (game, score) = finished(game, [&[MOND], &[S], &[PAGAT], &[]]);
    assert_eq!(skis_mond_trick(&game), Some(0));
    assert_eq!(game.public().tricks()[0].cards[..3], [MOND, S, PAGAT]);
    assert_eq!(game.public().tricks()[0].winner, C);
    assert_eq!(score.parts(A).valat, 0);
    assert_eq!(score.parts(A).mondfang, -20);
    for seat in [B, C, D] {
        assert_eq!(score.parts(seat).mondfang, 0);
    }

    // The same three cards with the pagat led: the škis wins the trick
    // (spec 013, criterion 8). It is the plain mondfang: C, who played the
    // mond, writes -20 and nobody else does.
    let deal = deal_with([&[PAGAT], &[S], &[MOND], &[]], &low_talon());
    let game = a_declares(&deal, SoloEna, None);
    let (game, score) = finished(game, [&[PAGAT], &[S], &[MOND], &[]]);
    assert_eq!(skis_mond_trick(&game), Some(0));
    assert_eq!(game.public().tricks()[0].cards[..3], [PAGAT, S, MOND]);
    assert_eq!(game.public().tricks()[0].winner, B);
    assert_eq!(score.parts(A).valat, 0);
    assert_eq!(score.parts(C).mondfang, -20);
    assert_eq!(score.entry(C), -20);
    for seat in [A, B, D] {
        assert_eq!(score.parts(seat).mondfang, 0);
    }

    // The pagat between the two (škis, pagat, mond): the škis wins again,
    // and A, who played the mond, writes -20. C wins the first trick with
    // the XX and leads the škis; D's only tarok left is the pagat.
    let deal = deal_with(
        [&[MOND], &[tarok(2)], &[S, tarok(20)], &[PAGAT, tarok(4)]],
        &low_talon(),
    );
    let game = a_declares(&deal, SoloEna, None);
    let (game, score) = finished(
        game,
        [&[tarok(3), MOND], &[], &[tarok(20), S], &[tarok(4), PAGAT]],
    );
    assert_eq!(skis_mond_trick(&game), Some(1));
    assert_eq!(game.public().tricks()[0].winner, C);
    assert_eq!(game.public().tricks()[1].cards[..3], [S, PAGAT, MOND]);
    assert_eq!(game.public().tricks()[1].winner, C);
    assert_eq!(score.parts(A).valat, 0);
    assert_eq!(score.parts(A).mondfang, -20);
    for seat in [B, C, D] {
        assert_eq!(score.parts(seat).mondfang, 0);
    }

    // No mondfang when the mond is not in a trick with the škis.
    let (game, score) = trula_for_a();
    assert_eq!(skis_mond_trick(&game), None);
    for seat in Seat::ALL {
        assert_eq!(score.parts(seat).mondfang, 0);
    }
}

#[test]
fn c43_an_opponent_who_loses_the_mond_writes_minus_20() {
    // The declarer's škis takes B's mond.
    let deal = deal_with([&[S], &[MOND], &[], &[]], &low_talon());
    let game = a_declares(&deal, SoloEna, None);
    let (game, score) = finished(game, [&[S], &[MOND], &[], &[]]);
    assert_eq!(skis_mond_trick(&game), Some(0));
    assert_eq!(game.public().tricks()[0].winner, A);
    assert_eq!(score.parts(A).valat, 0);
    // B is an opponent: no game entry, and -20 on its own.
    let parts = score.parts(B);
    assert_eq!((parts.game, parts.difference), (0, 0));
    assert_eq!(parts.mondfang, -20);
    assert_eq!(score.entry(B), -20);
    assert_eq!([score.entry(C), score.entry(D)], [0, 0]);
    assert_eq!(score.parts(A).mondfang, 0);

    // Another opponent's škis takes B's mond: B writes -20 again, C nothing.
    let deal = deal_with([&[tarok(2)], &[MOND], &[S], &[]], &low_talon());
    let game = a_declares(&deal, SoloEna, None);
    let (game, score) = finished(game, [&[tarok(2)], &[MOND], &[S], &[]]);
    assert_eq!(skis_mond_trick(&game), Some(0));
    assert_eq!(game.public().tricks()[0].winner, C);
    assert_eq!(score.parts(A).valat, 0);
    assert_eq!(score.entry(B), -20);
    assert_eq!(score.entry(C), 0);
    assert_eq!(score.parts(A).mondfang, 0);

    // The written deal 3: B loses the mond to the declarer D.
    let (game, _) = deal_3_king_in_talon();
    assert_eq!(skis_mond_trick(&game), Some(1));
    assert_eq!(game.score().unwrap().entry(B), -20);
}

#[test]
fn c44_mond_in_a_packet_not_taken_costs_the_declarer_20() {
    // The mond lies in the first packet; A (solo tri, holding the škis)
    // takes the second.
    let talon = [MOND, karo(4), karo(3), srce(4), srce(3), srce(2)];
    let deal = deal_with([&[S], &[], &[], &[]], &talon);
    let mut game = game(&deal);
    declare(&mut game, A, SoloTri);
    exchange_back(&mut game, 1);
    finish(&mut game);
    let score = game.score().unwrap();
    assert_eq!(score.parts(A).valat, 0);
    assert!(game.full().opponents_pile().contains(MOND));
    assert_eq!(skis_mond_trick(&game), None);
    assert_eq!(score.parts(A).mondfang, -20);
    for seat in [B, C, D] {
        assert_eq!(score.entry(seat), 0);
    }

    // A takes the packet with the mond: no penalty.
    let mut game = common::flow::game(&deal);
    declare(&mut game, A, SoloTri);
    exchange(&mut game, 0, &[karo(4), karo(3), tarok(2)]);
    assert!(game.private(A).hand().contains(MOND));
    finish(&mut game);
    assert_eq!(game.score().unwrap().parts(A).mondfang, 0);

    // The called king lies in the talon too (criterion 23): A takes the
    // king's packet and wins a trick with the king. The rest of the talon,
    // with the mond, is A's: no penalty.
    let game = king_taken_and_led(false);
    assert!(game.full().talon_rest_won());
    assert!(game.full().declarer_pile().contains(MOND));
    let score = game.score().unwrap();
    assert_eq!(score.parts(A).valat, 0);
    assert_eq!(score.parts(A).mondfang, 0);

    // The king loses its trick (criterion 24): the rest stays with the
    // opponents and the mond costs 20.
    let game = king_taken_and_led(true);
    assert!(!game.full().talon_rest_won());
    assert!(game.full().opponents_pile().contains(MOND));
    let score = game.score().unwrap();
    assert_eq!(score.parts(A).valat, 0);
    assert_eq!(score.parts(A).mondfang, -20);
}

// ---------------------------------------------------------------------
// Partner games in which the partner's tricks decide (review of spec 005)
// ---------------------------------------------------------------------

/// A plays tri and calls srce. The partner C holds škis, mond, XX-XII and
/// the srce kralj and takes all twelve tricks; the declarer takes none.
fn partner_takes_all_twelve() -> (Game, Score) {
    let mut c = top_taroks(12);
    c.push(kralj(Suit::Srce));
    let deal = deal_with([&[], &[], &c, &[]], &low_talon());
    let game = a_declares(&deal, Tri, Some(Suit::Srce));
    assert_eq!(game.full().partner(), Some(C));
    // C overtakes A's first lead with the XII, leads the taroks from the
    // top and the srce kralj last.
    let mut c_script = vec![tarok(12), S];
    c_script.extend((13..=21).rev().map(tarok));
    c_script.push(kralj(Suit::Srce));
    finished(game, [&[], &[], &c_script, &[]])
}

#[test]
fn c41_valat_counts_the_tricks_of_the_side_not_of_the_declarer() {
    // The declarer takes no trick and the partner takes all twelve: a
    // valat for the side, +250 for both, not -250.
    let (game, score) = partner_takes_all_twelve();
    assert_eq!(tricks_won(&game), [0, 0, 12, 0]);
    only_valat(&score, A, 250);
    only_valat(&score, C, 250);
    assert_eq!(score.entries(), [250, 0, 250, 0]);

    // The declarer takes no trick, the partner takes one and the opponents
    // eleven: no valat at all. B, an opponent, holds škis, XX-XI and the
    // pagat. C, the partner, holds the mond and the called srce kralj and
    // wins one trick with the mond.
    let mut b = vec![S];
    b.extend((11..=20).rev().map(tarok));
    b.push(PAGAT);
    let deal = deal_with([&[], &b, &[MOND, kralj(Suit::Srce)], &[]], &low_talon());
    let game = a_declares(&deal, Tri, Some(Suit::Srce));
    assert_eq!(game.full().partner(), Some(C));
    // A leads a tarok; B plays the XI and C takes the trick with the mond.
    let (game, score) = finished(game, [&[], &[tarok(11)], &[MOND], &[]]);
    let won = tricks_won(&game);
    assert_eq!(won[A.index()], 0, "the declarer takes no trick");
    assert!(won[C.index()] >= 1, "the partner takes a trick");
    assert!(
        won[B.index()] + won[D.index()] >= 1,
        "the opponents take a trick"
    );
    for seat in Seat::ALL {
        assert_eq!(score.parts(seat).valat, 0);
    }
    assert_ne!(score.parts(A).game, 0);
    assert_eq!(score.parts(A).game, score.parts(C).game);
}

#[test]
fn c38_kralj_ultimo_follows_the_side_that_takes_the_last_trick() {
    // Criterion 38 as written: the called king is played to the last trick
    // and the declarer's side wins that trick, here with another card.
    //
    // A plays tri and calls srce: škis, mond, XX-XII and the karo 4. The
    // partner C holds the srce kralj and keeps it for the last trick. A
    // leads nine taroks, loses the tenth trick to B's karo 3, trumps B's
    // srce dama with the XII and leads the XIII to the last trick, on
    // which C plays the srce kralj.
    let mut a = top_taroks(12);
    a.push(karo(4));
    let mut b = taroks(2..=5);
    b.extend([
        karo(3),
        dama(Suit::Srce),
        pik(7),
        pik(8),
        pik(9),
        pik(10),
        fant(Suit::Pik),
        kaval(Suit::Pik),
    ]);
    let mut c = taroks(6..=9);
    c.extend([
        kralj(Suit::Srce),
        srce(4),
        srce(3),
        srce(2),
        kriz(7),
        kriz(8),
        kriz(9),
        kriz(10),
    ]);
    let mut d = taroks([10, 11, 1]);
    d.extend([
        srce(1),
        fant(Suit::Srce),
        kaval(Suit::Srce),
        dama(Suit::Pik),
        kralj(Suit::Pik),
        fant(Suit::Kriz),
        kaval(Suit::Kriz),
        dama(Suit::Kriz),
        kralj(Suit::Kriz),
    ]);
    let deal = whole_deal([&a, &b, &c, &d], karo_talon());
    let game = a_declares(&deal, Tri, Some(Suit::Srce));
    assert_eq!(game.full().partner(), Some(C));

    let mut a_script = top_taroks(14);
    a_script.extend([karo(4), tarok(12), tarok(13)]);
    let mut b_script = taroks(2..=5);
    b_script.extend([
        pik(7),
        pik(8),
        pik(9),
        pik(10),
        fant(Suit::Pik),
        karo(3),
        dama(Suit::Srce),
        kaval(Suit::Pik),
    ]);
    let mut c_script = taroks(6..=9);
    c_script.extend([
        kriz(7),
        kriz(8),
        kriz(9),
        kriz(10),
        srce(4),
        srce(3),
        srce(2),
        kralj(Suit::Srce),
    ]);
    let (game, score) = finished(game, [&a_script, &b_script, &c_script, &[]]);
    assert_eq!(tricks_won(&game), [11, 1, 0, 0]);
    let last = game.public().tricks()[11];
    assert_eq!(last.cards[0], tarok(13));
    assert_eq!(last.cards[2], kralj(Suit::Srce));
    assert_eq!(last.winner, A);
    // The king did not take the trick; its side did: +10 for both.
    assert_eq!(score.parts(A).kralj_ultimo, 10);
    assert_eq!(score.parts(C).kralj_ultimo, 10);
    assert_eq!(
        [score.parts(B).kralj_ultimo, score.parts(D).kralj_ultimo],
        [0, 0]
    );
}

#[test]
fn c42_a_partner_who_loses_the_mond_pays_alone() {
    // A plays tri and calls srce; the partner C holds the mond and the
    // srce kralj. A leads the tarok II, the opponent B plays the škis and
    // C plays the mond. C writes -20; the declarer does not.
    let deal = deal_with(
        [&[tarok(2)], &[S], &[MOND, kralj(Suit::Srce)], &[]],
        &low_talon(),
    );
    let game = a_declares(&deal, Tri, Some(Suit::Srce));
    assert_eq!(game.full().partner(), Some(C));
    let (game, score) = finished(game, [&[tarok(2)], &[S], &[MOND], &[]]);
    assert_eq!(skis_mond_trick(&game), Some(0));
    assert_eq!(game.public().tricks()[0].winner, B);
    assert_eq!(score.parts(A).valat, 0);
    assert_eq!(score.parts(C).mondfang, -20);
    assert_eq!(score.parts(A).mondfang, 0);
    assert_eq!([score.parts(B).mondfang, score.parts(D).mondfang], [0, 0]);
    // The same entry apart from the mondfang.
    assert_eq!(score.entry(C), score.entry(A) - 20);
    assert_eq!([score.entry(B), score.entry(D)], [0, 0]);
}

#[test]
fn c36_c37_trula_and_kralji_count_the_tricks_of_the_partner() {
    // A plays tri and calls križ; the partner C wins one trick, with the
    // križ kralj. The three other kings are in A's tricks. All four are
    // with the declarer's side: +10 for both.
    let game = a_declares(&pagat_and_a_higher_tarok(A), Tri, Some(Suit::Kriz));
    assert_eq!(game.full().partner(), Some(C));
    let (game, score) = finished(game, pagat_and_a_higher_tarok_scripts(A));
    assert_eq!(tricks_won(&game), [10, 1, 1, 0]);
    assert!(game.public().tricks_won(C).contains(kralj(Suit::Kriz)));
    for suit in [Suit::Srce, Suit::Karo, Suit::Pik] {
        assert!(game.public().tricks_won(A).contains(kralj(suit)));
    }
    assert_eq!(score.parts(A).kralji, 10);
    assert_eq!(score.parts(C).kralji, 10);

    // The trula the same way. A plays tri and calls srce. The partner C
    // holds the škis and the srce kralj; A holds the mond, the pagat, XI,
    // XII, XIV-XX and the srce 4; B holds II-X and D the XIII.
    // The škis is in a trick of C, the mond and the pagat in tricks of A.
    let mut a = vec![MOND, PAGAT, tarok(11), tarok(12), srce(4)];
    a.extend((14..=20).map(tarok));
    let talon = [
        karo(4),
        karo(3),
        karo(2),
        karo(1),
        fant(Suit::Karo),
        kaval(Suit::Karo),
    ];
    let deal = deal_with([&a, &[], &[S, kralj(Suit::Srce)], &[tarok(13)]], &talon);
    let game = a_declares(&deal, Tri, Some(Suit::Srce));
    assert_eq!(game.full().partner(), Some(C));
    // A leads the XI; C takes it with the škis and leads the pik 7, which
    // A trumps with the XII. Then A leads the mond and the taroks from the
    // top, the pagat in the eleventh trick and the srce 4 last, which B
    // wins with its last srce.
    let mut a_script = vec![tarok(11), tarok(12), MOND];
    a_script.extend((14..=20).rev().map(tarok));
    a_script.extend([PAGAT, srce(4)]);
    let (game, score) = finished(game, [&a_script, &[], &[S, pik(7)], &[]]);
    assert_eq!(tricks_won(&game), [10, 1, 1, 0]);
    assert!(game.public().tricks_won(C).contains(S));
    assert!(game.public().tricks_won(A).contains(MOND));
    assert!(game.public().tricks_won(A).contains(PAGAT));
    assert_eq!(trick_of(&game, PAGAT), 11);
    assert_eq!(score.parts(A).valat, 0, "the opponents take a trick");
    assert_eq!(skis_mond_trick(&game), None);
    assert_eq!(score.parts(A).trula, 10);
    assert_eq!(score.parts(C).trula, 10);
}

// ---------------------------------------------------------------------
// Spec 013, criteria 1 to 3 (D125): a valat does not cancel a lost mond.
// The mond's player writes -20 next to the 250. These tests replace the
// five `reading2_...` tests of spec 005, which pinned the other reading.
// ---------------------------------------------------------------------

#[test]
fn d125_valat_and_an_opponent_who_loses_the_mond() {
    // Case (i). A (solo ena) holds škis, XX-XI and the pagat and takes all
    // twelve tricks. B's only tarok is the mond: it falls under the škis
    // in the first trick. +250 for A, -20 for B.
    let mut a = vec![S];
    a.extend((11..=20).rev().map(tarok));
    a.push(PAGAT);
    let deal = deal_with([&a, &[MOND], &taroks(2..=10), &[]], &low_talon());
    let game = a_declares(&deal, SoloEna, None);
    let (game, score) = finished(game, [&a, &[MOND], &[], &[]]);
    assert_eq!(tricks_won(&game), [12, 0, 0, 0]);
    assert_eq!(skis_mond_trick(&game), Some(0));
    valat_and_mondfang(&score, A, 250, 0);
    valat_and_mondfang(&score, B, 0, -20);
    assert_eq!(score.entries(), [250, -20, 0, 0]);
}

#[test]
fn d125_valat_and_a_mond_lost_inside_the_declarers_side() {
    // Case (ii). A plays tri and calls srce: mond, XX-XI and the pagat.
    // The partner C holds the škis and the srce kralj. A leads the mond, C
    // must play the škis. The two take all twelve tricks. +250 for both,
    // and -20 for A alone.
    let mut a = vec![MOND];
    a.extend((11..=20).rev().map(tarok));
    a.push(PAGAT);
    let deal = deal_with([&a, &[], &[S, kralj(Suit::Srce)], &[]], &low_talon());
    let game = a_declares(&deal, Tri, Some(Suit::Srce));
    assert_eq!(game.full().partner(), Some(C));
    // C leads the srce kralj to the second trick; A trumps it with the XX
    // and leads the rest from the top, the pagat last.
    let (game, score) = finished(game, [&a, &[], &[S, kralj(Suit::Srce)], &[]]);
    assert_eq!(tricks_won(&game), [11, 0, 1, 0]);
    assert_eq!(skis_mond_trick(&game), Some(0));
    assert_eq!(game.public().tricks()[0].winner, C);
    valat_and_mondfang(&score, A, 250, -20);
    valat_and_mondfang(&score, C, 250, 0);
    assert_eq!(score.entries(), [230, 0, 250, 0]);

    // The other way round: A holds škis, XX-XI and the pagat; the partner
    // C holds the mond as its only tarok and the srce kralj. A leads the
    // škis and takes all twelve tricks. -20 for C alone.
    let mut a = vec![S];
    a.extend((11..=20).rev().map(tarok));
    a.push(PAGAT);
    let deal = deal_with(
        [&a, &taroks(2..=10), &[MOND, kralj(Suit::Srce)], &[]],
        &low_talon(),
    );
    let game = a_declares(&deal, Tri, Some(Suit::Srce));
    assert_eq!(game.full().partner(), Some(C));
    let (game, score) = finished(game, [&a, &[], &[MOND], &[]]);
    assert_eq!(tricks_won(&game), [12, 0, 0, 0]);
    assert_eq!(skis_mond_trick(&game), Some(0));
    valat_and_mondfang(&score, A, 250, 0);
    valat_and_mondfang(&score, C, 250, -20);
    assert_eq!(score.entries(), [250, 0, 230, 0]);
}

/// B, an opponent, holds škis, XX-XI and the pagat and takes all twelve
/// tricks against the declarer A: the XI on A's first lead, eight taroks
/// from the XX down, the škis, the XII and the pagat.
fn b_takes_all_twelve_script() -> Vec<Card> {
    let mut script = vec![tarok(11)];
    script.extend((13..=20).rev().map(tarok));
    script.extend([S, tarok(12), PAGAT]);
    script
}

fn b_strong_hand() -> Vec<Card> {
    let mut b = vec![S];
    b.extend((11..=20).rev().map(tarok));
    b.push(PAGAT);
    b
}

#[test]
fn d125_valat_against_and_the_declarers_lost_mond() {
    // Case (iii), the mond lost in a trick. A (solo ena) holds the mond
    // and II-X. B takes all twelve tricks and catches the mond with the
    // škis in the tenth. -250 and -20 for A.
    let deal = deal_with([&[MOND], &b_strong_hand(), &[], &[]], &low_talon());
    let game = a_declares(&deal, SoloEna, None);
    let (game, score) = finished(game, [&[tarok(2)], &b_takes_all_twelve_script(), &[], &[]]);
    assert_eq!(tricks_won(&game), [0, 12, 0, 0]);
    assert_eq!(skis_mond_trick(&game), Some(9));
    valat_and_mondfang(&score, A, -250, -20);
    assert_eq!(score.entries(), [-270, 0, 0, 0]);

    // The partner loses it. A plays tri and calls srce; C holds the mond
    // as its only tarok and the srce kralj. A leads the II, B plays the
    // škis and C must play the mond. B takes all twelve tricks. -250 for
    // both, and -20 for C alone.
    let deal = deal_with(
        [&[], &b_strong_hand(), &[MOND, kralj(Suit::Srce)], &[]],
        &low_talon(),
    );
    let game = a_declares(&deal, Tri, Some(Suit::Srce));
    assert_eq!(game.full().partner(), Some(C));
    let (game, score) = finished(game, [&[tarok(2)], &b_strong_hand(), &[MOND], &[]]);
    assert_eq!(tricks_won(&game), [0, 12, 0, 0]);
    assert_eq!(skis_mond_trick(&game), Some(0));
    valat_and_mondfang(&score, A, -250, 0);
    valat_and_mondfang(&score, C, -250, -20);
    assert_eq!(score.entries(), [-250, 0, -270, 0]);
}

#[test]
fn d125_valat_against_and_the_mond_left_in_the_talon() {
    // Case (iii), the mond in a packet not taken. The mond lies in the
    // first packet; A (solo tri) takes the second. B takes all twelve
    // tricks. -250 and -20 for A.
    let talon = [MOND, karo(4), karo(3), srce(4), srce(3), srce(2)];
    let deal = deal_with([&[], &b_strong_hand(), &[], &[]], &talon);
    let mut game = common::flow::game(&deal);
    declare(&mut game, A, SoloTri);
    exchange_back(&mut game, 1);
    // B: the XI on A's first lead, then škis, XX-XII, pagat.
    let mut b_script = vec![tarok(11), S];
    b_script.extend((12..=20).rev().map(tarok));
    b_script.push(PAGAT);
    let (game, score) = finished(game, [&[], &b_script, &[], &[]]);
    assert_eq!(tricks_won(&game), [0, 12, 0, 0]);
    // D133 is for the declarer's valat only: here the packet not taken
    // stays with the opponents.
    assert!(!game.full().talon_rest_won());
    assert!(game.full().opponents_pile().contains(MOND));
    assert!(!game.public().taken_cards().contains(MOND));
    assert_eq!(skis_mond_trick(&game), None);
    assert_scores_agree(&game);
    valat_and_mondfang(&score, A, -250, -20);
    assert_eq!(score.entries(), [-270, 0, 0, 0]);
}

#[test]
fn d125_valat_against_and_an_opponents_mond_taken_by_an_opponent() {
    // Case (iv). A plays solo ena. C holds škis, XX-XI and the pagat; B's
    // only tarok is the mond. A leads a tarok, B must play the mond and C
    // takes it with the škis. C takes all twelve tricks. -250 for A, -20
    // for B.
    let deal = deal_with([&[], &[MOND], &b_strong_hand(), &[]], &low_talon());
    let game = a_declares(&deal, SoloEna, None);
    let mut c_script = vec![S];
    c_script.extend((11..=20).rev().map(tarok));
    c_script.push(PAGAT);
    let (game, score) = finished(game, [&[], &[MOND], &c_script, &[]]);
    assert_eq!(tricks_won(&game), [0, 0, 12, 0]);
    assert_eq!(skis_mond_trick(&game), Some(0));
    valat_and_mondfang(&score, A, -250, 0);
    valat_and_mondfang(&score, B, 0, -20);
    assert_eq!(score.entries(), [-250, -20, 0, 0]);
}

// ---------------------------------------------------------------------
// Review of spec 013: two cases the suite did not have. The first one
// was then ruled on by the user (D133, criterion 16).
// ---------------------------------------------------------------------

/// Spec 013, criterion 16 (D133). The user's words: "In case of valat, the
/// declarer gets the packet he did not pick so he did not lose the mond".
/// The declarer's side takes all twelve tricks and the mond lies in a
/// packet not taken: the packets go to the declarer's side, +250, and no
/// mondfang for anyone.
#[test]
fn d133_in_a_valat_the_declarer_wins_the_packets_not_taken() {
    let talon = [MOND, karo(4), karo(3), srce(4), srce(3), srce(2)];

    // Alone. The mond lies in the first packet; A (solo tri) takes the
    // second and puts the same cards down. A holds škis, XX-XI and the
    // pagat and takes all twelve tricks.
    let mut a = vec![S];
    a.extend((11..=20).rev().map(tarok));
    a.push(PAGAT);
    let deal = deal_with([&a, &[], &[], &[]], &talon);
    let mut game = common::flow::game(&deal);
    declare(&mut game, A, SoloTri);
    exchange_back(&mut game, 1);
    // Eleven tricks first: until the last trick is taken the packet still
    // lies with the opponents.
    let mut led = 0;
    while game.public().tricks().len() < 11 {
        let action = if game.to_act() == Some(A) {
            led += 1;
            Action::Play(a[led - 1])
        } else {
            game.legal_actions().iter().next().expect("a legal card")
        };
        game.apply(action).expect("a legal card is accepted");
    }
    assert_eq!(tricks_won(&game), [11, 0, 0, 0]);
    assert!(!game.full().talon_rest_won());
    assert!(game.full().opponents_pile().contains(MOND));
    let (game, score) = finished(game, [&a[11..], &[], &[], &[]]);
    assert_eq!(tricks_won(&game), [12, 0, 0, 0]);
    assert!(!game.public().taken_cards().contains(MOND));
    assert_eq!(skis_mond_trick(&game), None);
    // The packet not taken is the declarer's: all 54 cards, all 70 points.
    assert!(game.full().talon_rest_won());
    assert_eq!(game.full().declarer_pile(), CardSet::FULL);
    assert!(game.full().opponents_pile().is_empty());
    assert_eq!((score.declarer_points(), score.opponents_points()), (70, 0));
    assert_scores_agree(&game);
    only_valat(&score, A, 250);
    for seat in [B, C, D] {
        valat_and_mondfang(&score, seat, 0, 0);
    }
    assert_eq!(score.entries(), [250, 0, 0, 0]);

    // With a partner, who takes one of the twelve tricks. A plays tri and
    // calls srce: X, XX-XI and the pagat. C holds the škis and the srce
    // kralj. A leads the X, C takes it with the škis and leads the srce
    // kralj; A trumps it with the XX and leads the rest from the top, the
    // pagat last. The mond lies in the packet A did not take.
    let mut a = vec![tarok(10)];
    a.extend((11..=20).rev().map(tarok));
    a.push(PAGAT);
    let deal = deal_with([&a, &[], &[S, kralj(Suit::Srce)], &[]], &talon);
    let mut game = common::flow::game(&deal);
    declare(&mut game, A, Tri);
    act(&mut game, A, Action::CallKing(Suit::Srce));
    exchange_back(&mut game, 1);
    assert_eq!(game.full().partner(), Some(C));
    let (game, score) = finished(game, [&a, &[], &[S, kralj(Suit::Srce)], &[]]);
    assert_eq!(tricks_won(&game), [11, 0, 1, 0]);
    assert!(!game.public().taken_cards().contains(MOND));
    assert_eq!(skis_mond_trick(&game), None);
    assert!(game.full().talon_rest_won());
    assert!(game.full().declarer_pile().contains(MOND));
    assert!(game.full().opponents_pile().is_empty());
    assert_scores_agree(&game);
    only_valat(&score, A, 250);
    only_valat(&score, C, 250);
    for seat in [B, D] {
        valat_and_mondfang(&score, seat, 0, 0);
    }
    assert_eq!(score.entries(), [250, 0, 250, 0]);
}

/// The hands for the tests with the whole trula in the last trick. A holds
/// the škis, B the mond and C the pagat; every seat keeps its trula card
/// for the last trick. All 48 cards are played in the order of the lists.
///
///  1-6.  A leads srce kralj, pik kralj, pik dama, križ kralj, križ dama,
///        karo kralj and wins each; all follow.
///  7-10. A leads XX, XIX, XVIII, XVII and wins each.
///  11.   A leads its fifth tarok, B plays its lowest, C the VI, D the
///        karo 2. With `b_leads_the_last` A holds the XI and B the XVI: B
///        wins the trick. Else A holds the XVI and B the XI: A wins it.
///  12.   Led by A: škis, mond, pagat, karo 1. Led by B: mond, pagat,
///        karo 1, škis.
fn trula_in_the_last_trick_deal(b_leads_the_last: bool) -> (Deal, [Vec<Card>; 4]) {
    use Suit::{Karo, Kriz, Pik, Srce};
    let a_low = if b_leads_the_last { 11 } else { 16 };
    let a = vec![
        kralj(Srce),
        kralj(Pik),
        dama(Pik),
        kralj(Kriz),
        dama(Kriz),
        kralj(Karo),
        tarok(20),
        tarok(19),
        tarok(18),
        tarok(17),
        tarok(a_low),
        S,
    ];
    let mut b = vec![
        dama(Srce),
        pik(7),
        kaval(Pik),
        kriz(7),
        kaval(Kriz),
        dama(Karo),
    ];
    if b_leads_the_last {
        b.extend([15, 14, 13, 12, 16].map(tarok));
    } else {
        b.extend([15, 14, 13, 12, 11].map(tarok));
    }
    b.push(MOND);
    let c = vec![
        kaval(Srce),
        pik(8),
        fant(Pik),
        kriz(8),
        fant(Kriz),
        kaval(Karo),
        tarok(10),
        tarok(9),
        tarok(8),
        tarok(7),
        tarok(6),
        PAGAT,
    ];
    let d = vec![
        fant(Srce),
        pik(9),
        pik(10),
        kriz(9),
        kriz(10),
        fant(Karo),
        tarok(5),
        tarok(4),
        tarok(3),
        tarok(2),
        karo(2),
        karo(1),
    ];
    let deal = whole_deal([&a, &b, &c, &d], low_talon());
    (deal, [a, b, c, d])
}

/// Plays `trula_in_the_last_trick_deal` with `declarer` in `contract`. The
/// declarer puts the first packet down again, so the hands stay as dealt.
fn trula_in_the_last_trick(
    b_leads_the_last: bool,
    declarer: Seat,
    contract: Contract,
    call: Option<Suit>,
) -> (Game, Score) {
    let (deal, [a, b, c, d]) = trula_in_the_last_trick_deal(b_leads_the_last);
    let mut game = common::flow::game(&deal);
    declare(&mut game, declarer, contract);
    if let Some(suit) = call {
        act(&mut game, declarer, Action::CallKing(suit));
    }
    exchange_back(&mut game, 0);
    let (game, score) = finished(game, [&a, &b, &c, &d]);
    assert_eq!(game.public().declarer(), Some(declarer));
    assert_scores_agree(&game);
    (game, score)
}

/// Spec 013, criteria 4, 5, 8 and 10 together: škis, mond and pagat all
/// fall in the last trick.
#[test]
fn c39_pagat_ultimo_when_the_trula_falls_in_the_last_trick() {
    let last_trick = |game: &Game| game.public().tricks()[11];

    // (a) Škis, mond, pagat, karo 1: the pagat falls last and wins.
    //
    // A plays tri and called its own srce kralj, so A is alone. C, an
    // opponent, wins the trick with the pagat: -25 for A. The whole trula
    // is in C's trick: -10. B played the mond under the škis: -20 for B.
    let (game, score) = trula_in_the_last_trick(false, A, Tri, Some(Suit::Srce));
    assert_eq!(game.full().partner(), None);
    assert_eq!(last_trick(&game).cards, [S, MOND, PAGAT, karo(1)]);
    assert_eq!(last_trick(&game).leader, A);
    assert_eq!(last_trick(&game).winner, C);
    assert_eq!(tricks_won(&game), [11, 0, 1, 0]);
    assert_eq!(score.parts(A).valat, 0);
    assert_eq!(score.parts(A).pagat_ultimo, -25);
    assert_eq!(score.parts(A).kralj_ultimo, 0);
    assert_eq!(score.parts(A).trula, -10);
    assert_eq!(score.parts(A).mondfang, 0);
    assert_eq!(score.parts(B).mondfang, -20);
    assert_eq!(score.entry(B), -20);
    assert_eq!([score.entry(C), score.entry(D)], [0, 0]);

    // The same cards with C as the declarer, in solo tri: the declarer's
    // own pagat wins the last trick. +25 and the trula for C; -20 for B.
    let (game, score) = trula_in_the_last_trick(false, C, SoloTri, None);
    assert_eq!(last_trick(&game).cards, [S, MOND, PAGAT, karo(1)]);
    assert_eq!(last_trick(&game).winner, C);
    assert_eq!(tricks_won(&game), [11, 0, 1, 0]);
    assert_eq!(score.parts(C).valat, 0);
    assert_eq!(score.parts(C).pagat_ultimo, 25);
    assert_eq!(score.parts(C).trula, 10);
    assert_eq!(score.parts(C).mondfang, 0);
    assert_eq!(score.entry(B), -20);
    assert_eq!([score.entry(A), score.entry(D)], [0, 0]);

    // (b) Mond, pagat, karo 1, škis: the pagat is not last, the škis wins.
    //
    // A alone again. A takes the trick with the škis, and in it the pagat
    // of the opponent C: +25 for A, and the trula. -20 for B.
    let (game, score) = trula_in_the_last_trick(true, A, Tri, Some(Suit::Srce));
    assert_eq!(game.full().partner(), None);
    assert_eq!(last_trick(&game).cards, [MOND, PAGAT, karo(1), S]);
    assert_eq!(last_trick(&game).leader, B);
    assert_eq!(last_trick(&game).winner, A);
    assert_eq!(tricks_won(&game), [11, 1, 0, 0]);
    assert_eq!(score.parts(A).valat, 0);
    assert_eq!(score.parts(A).pagat_ultimo, 25);
    assert_eq!(score.parts(A).trula, 10);
    assert_eq!(score.parts(A).mondfang, 0);
    assert_eq!(score.parts(B).mondfang, -20);
    assert_eq!(score.entry(B), -20);

    // The pagat's own side takes the trick with the škis. C plays dve and
    // calls srce; A holds the srce kralj and is the partner. The pagat
    // loses the trick, but its side takes it: +25 for C and A, and the
    // trula. B is an opponent and writes -20.
    let (game, score) = trula_in_the_last_trick(true, C, Dve, Some(Suit::Srce));
    assert_eq!(game.full().partner(), Some(A));
    assert_eq!(last_trick(&game).cards, [MOND, PAGAT, karo(1), S]);
    assert_eq!(last_trick(&game).winner, A);
    assert_eq!(score.parts(C).valat, 0);
    assert_eq!(score.parts(C).pagat_ultimo, 25);
    assert_eq!(score.parts(A).pagat_ultimo, 25);
    assert_eq!(score.parts(C).trula, 10);
    assert_eq!(score.entry(A), score.entry(C));
    assert_eq!(score.entry(B), -20);
    assert_eq!(score.entry(D), 0);

    // The same on the other side. B plays solo tri; A, C and D are the
    // opponents. A takes the trick with the škis and in it the pagat of
    // its fellow opponent C: the opponents win the bonus, -25 for B. The
    // trula is theirs too, -10, and B lost its own mond: -20.
    let (game, score) = trula_in_the_last_trick(true, B, SoloTri, None);
    assert_eq!(last_trick(&game).cards, [MOND, PAGAT, karo(1), S]);
    assert_eq!(last_trick(&game).winner, A);
    assert_eq!(score.parts(B).valat, 0);
    assert_eq!(score.parts(B).pagat_ultimo, -25);
    assert_eq!(score.parts(B).trula, -10);
    assert_eq!(score.parts(B).mondfang, -20);
    assert_eq!([score.entry(A), score.entry(C), score.entry(D)], [0, 0, 0]);
}
