//! Spec 009, criteria 18-32: the score with announcements and kontra.
//!
//! Seats: A is forehand (seat 0), then B, C, D; D deals. Most tests take a
//! built deal of `common::built`, in which the outcome is forced, and play
//! it again with something said in the round of announcements
//! (`common::flow::with_round`). What is said changes no card: the hands
//! and the play are those of the silent deal.
//!
//! Every amount is written by the declarer and the partner; the opponents
//! write 0 for it (S20 a, S26 a). "For the owner" is plus when the
//! declarer's side announced the bonus and minus when an opponent did.

mod common;

use common::built::*;
use common::flow::{A, B, C, D, deal_with, seeded_game, with_round};
use common::second_scorer::{assert_scores_agree, second_score};
use common::written::deal_1_partner;
use common::{kralj, tarok};
use tarok_engine::Contract::{Dve, Ena, SoloEna, SoloTri, Tri};
use tarok_engine::{Action, Bonus, Card, Game, PublicPartner, Score, Seat, Suit, Target, encode};

// The parts of an entry, in the order of `docs/encoding.md` section 8.
const GAME: usize = 0;
const DIFFERENCE: usize = 1;
const TRULA: usize = 2;
const KRALJI: usize = 3;
const KRALJ_ULTIMO: usize = 4;
const PAGAT_ULTIMO: usize = 5;
const VALAT: usize = 6;
const MONDFANG: usize = 7;

fn ann(bonus: Bonus) -> Action {
    Action::Announce(bonus)
}

fn k(target: Target) -> Action {
    Action::Kontra(target)
}

/// The eight parts of a seat: game, difference, trula, kralji, kralj
/// ultimo, pagat ultimo, valat, mondfang.
fn parts(score: &Score, seat: Seat) -> [i32; 8] {
    let p = score.parts(seat);
    [
        p.game,
        p.difference,
        p.trula,
        p.kralji,
        p.kralj_ultimo,
        p.pagat_ultimo,
        p.valat,
        p.mondfang,
    ]
}

/// `base` with some parts replaced.
fn but(base: [i32; 8], changes: &[(usize, i32)]) -> [i32; 8] {
    let mut parts = base;
    for &(index, value) in changes {
        parts[index] = value;
    }
    parts
}

/// A built deal played again with `turns` said in the round. The engine's
/// score is compared with the tests' second scorer on the way.
fn said(turns: &[(Seat, &[Action])], build: impl FnOnce() -> (Game, Score)) -> (Game, Score) {
    let (game, score) = with_round(turns, build);
    assert_scores_agree(&game);
    assert_eq!(game.score(), Some(score));
    (game, score)
}

/// The declarer's parts of a built deal with `turns` said.
fn declarer_parts(turns: &[(Seat, &[Action])], build: impl FnOnce() -> (Game, Score)) -> [i32; 8] {
    let (game, score) = said(turns, build);
    let declarer = game.public().declarer().expect("a declarer");
    // The opponents write nothing but a mondfang of their own.
    for seat in Seat::ALL {
        if !game.full().on_declarer_side(seat) {
            assert_eq!(parts(&score, seat)[..MONDFANG], [0; 7], "{seat:?}");
        } else {
            // The partner writes what the declarer writes (S26 a).
            assert_eq!(
                parts(&score, seat)[..MONDFANG],
                parts(&score, declarer)[..MONDFANG]
            );
        }
    }
    parts(&score, declarer)
}

// ---------------------------------------------------------------------
// More built deals
// ---------------------------------------------------------------------

/// A plays tri and calls srce, holding the srce kralj itself. The king is
/// played to the last trick. A takes that trick, or B does
/// (`last_trick_to_b`).
fn own_king_in_the_last_trick(last_trick_to_b: bool) -> (Game, Score) {
    let game = a_declares(&ultimo_deal(true), Tri, Some(Suit::Srce));
    finished(
        game,
        [
            &ultimo_script_a(),
            &ultimo_script_b(!last_trick_to_b),
            &[],
            &[],
        ],
    )
}

/// A plays tri and calls srce; the partner C holds the srce kralj. C keeps
/// it for the last trick and wins that trick with it, or (`king_early`)
/// throws it on A's fifth tarok lead. The play is the one of `c38` in
/// `tests/scoring.rs`.
fn partners_king(king_early: bool) -> (Game, Score) {
    let game = a_declares(&ultimo_deal(false), Tri, Some(Suit::Srce));
    assert_eq!(game.full().partner(), Some(C));
    let king = kralj(Suit::Srce);
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
    if king_early {
        c_script.push(king);
    }
    c_script.extend([
        srce(4),
        srce(3),
        srce(2),
        kriz(7),
        kriz(8),
        kriz(9),
        kriz(10),
    ]);
    if !king_early {
        c_script.push(king);
    }
    finished(game, [&a_script, &b_script, &c_script, &[]])
}

/// The pagat (C's) is played to the last trick, in which the seat `xix`
/// (A or D) plays the XIX and takes the trick. A plays `contract`.
fn pagat_under_the_xix(
    xix: Seat,
    contract: tarok_engine::Contract,
    call: Option<Suit>,
) -> (Game, Score) {
    let game = a_declares(&pagat_and_a_higher_tarok(xix), contract, call);
    finished(game, pagat_and_a_higher_tarok_scripts(xix))
}

/// Škis, mond, XX-XII and the pagat: twelve taroks that take twelve tricks
/// from the lead.
fn twelve_taroks() -> Vec<Card> {
    let mut cards = top_taroks(12);
    cards.push(PAGAT);
    cards
}

/// A plays solo ena with the twelve taroks and takes all twelve tricks.
fn all_twelve_for_a() -> (Game, Score) {
    let deal = deal_with([&twelve_taroks(), &[], &[], &[]], &low_talon());
    let game = a_declares(&deal, SoloEna, None);
    let (game, score) = finished(game, [&twelve_taroks(), &[], &[], &[]]);
    assert_eq!(tricks_won(&game), [12, 0, 0, 0]);
    (game, score)
}

/// A plays solo ena; B holds the twelve taroks and takes all twelve
/// tricks: the XII on A's lead, then from the top, the pagat last.
fn all_twelve_for_b() -> (Game, Score) {
    let deal = deal_with([&[], &twelve_taroks(), &[], &[]], &low_talon());
    let game = a_declares(&deal, SoloEna, None);
    let mut b_script = vec![tarok(12), S];
    b_script.extend((13..=21).rev().map(tarok));
    b_script.push(PAGAT);
    let (game, score) = finished(game, [&[], &b_script, &[], &[]]);
    assert_eq!(tricks_won(&game), [0, 12, 0, 0]);
    (game, score)
}

/// A plays solo ena, leads the mond, and B takes it with the škis.
fn declarers_mond_is_caught() -> (Game, Score) {
    let deal = deal_with([&[MOND], &[S], &[], &[]], &low_talon());
    let game = a_declares(&deal, SoloEna, None);
    let (game, score) = finished(game, [&[MOND], &[S], &[], &[]]);
    assert_eq!(skis_mond_trick(&game), Some(0));
    // Neither side takes all twelve tricks.
    let by_a = tricks_won(&game)[0];
    assert!(by_a > 0 && by_a < 12, "{by_a} tricks for A");
    (game, score)
}

/// A plays solo ena, leads the škis, and B must give the mond.
fn opponents_mond_is_caught() -> (Game, Score) {
    let deal = deal_with([&[S], &[MOND], &[], &[]], &low_talon());
    let game = a_declares(&deal, SoloEna, None);
    let (game, score) = finished(game, [&[S], &[MOND], &[], &[]]);
    assert_eq!(skis_mond_trick(&game), Some(0));
    // Neither side takes all twelve tricks.
    let by_a = tricks_won(&game)[0];
    assert!(by_a > 0 && by_a < 12, "{by_a} tricks for A");
    (game, score)
}

/// A (solo ena) holds škis, XX-XI and the pagat and takes all twelve
/// tricks. B's only tarok is the mond: it falls under the škis.
fn all_twelve_and_a_caught_mond() -> (Game, Score) {
    let mut a = vec![S];
    a.extend((11..=20).rev().map(tarok));
    a.push(PAGAT);
    let deal = deal_with([&a, &[MOND], &taroks(2..=10), &[]], &low_talon());
    let game = a_declares(&deal, SoloEna, None);
    let (game, score) = finished(game, [&a, &[MOND], &[], &[]]);
    assert_eq!(tricks_won(&game), [12, 0, 0, 0]);
    assert_eq!(skis_mond_trick(&game), Some(0));
    (game, score)
}

// ---------------------------------------------------------------------
// Criteria
// ---------------------------------------------------------------------

#[test]
fn c18_announced_trula_scores_20_for_or_against_its_owner() {
    // The declarer announces and its side has škis, mond and pagat: +20.
    let base = declarer_parts(&[], trula_for_a);
    assert_eq!(base[TRULA], 10, "silent: +10");
    let (game, score) = said(&[(A, &[ann(Bonus::Trula)])], trula_for_a);
    assert_eq!(game.public().announced(Bonus::Trula), Some(A));
    assert_eq!(parts(&score, A), but(base, &[(TRULA, 20)]));
    for seat in [B, C, D] {
        assert_eq!(parts(&score, seat), [0; 8], "the opponents write 0");
    }
    assert_eq!(score.entry(A), base.iter().sum::<i32>() + 10);

    // The declarer announces and the three cards are split: -20. (A holds
    // all three, but D takes the pagat in the last trick.)
    let base = declarer_parts(&[], pagat_ultimo_own_loses);
    assert_eq!(base[TRULA], 0, "silent: nothing");
    assert_eq!(
        declarer_parts(&[(A, &[ann(Bonus::Trula)])], pagat_ultimo_own_loses),
        but(base, &[(TRULA, -20)])
    );
    // The declarer announces and the opponents have all three: -20.
    let base = declarer_parts(&[], trula_for_the_opponents);
    assert_eq!(base[TRULA], -10);
    assert_eq!(
        declarer_parts(&[(A, &[ann(Bonus::Trula)])], trula_for_the_opponents),
        but(base, &[(TRULA, -20)])
    );

    // An opponent announces and its side has all three: 20 for the owner,
    // so the declarer writes -20.
    assert_eq!(
        declarer_parts(&[(B, &[ann(Bonus::Trula)])], trula_for_the_opponents),
        but(base, &[(TRULA, -20)])
    );
    // An opponent announces and fails: 20 against the owner, +20 for the
    // declarer. Once with the trula split, once with all three with A.
    let base = declarer_parts(&[], pagat_ultimo_own_loses);
    assert_eq!(
        declarer_parts(&[(D, &[ann(Bonus::Trula)])], pagat_ultimo_own_loses),
        but(base, &[(TRULA, 20)])
    );
    let base = declarer_parts(&[], trula_for_a);
    for opponent in [B, C, D] {
        assert_eq!(
            declarer_parts(&[(opponent, &[ann(Bonus::Trula)])], trula_for_a),
            but(base, &[(TRULA, 20)]),
            "{opponent:?} announces"
        );
    }
}

#[test]
fn c19_announced_kralji_scores_20_for_or_against_its_owner() {
    // The declarer announces and has the four kings: +20.
    let base = declarer_parts(&[], kings_for_a);
    assert_eq!(base[KRALJI], 10, "silent: +10");
    assert_eq!(
        declarer_parts(&[(A, &[ann(Bonus::Kralji)])], kings_for_a),
        but(base, &[(KRALJI, 20)])
    );
    // An opponent announces and fails: +20 for the declarer.
    assert_eq!(
        declarer_parts(&[(C, &[ann(Bonus::Kralji)])], kings_for_a),
        but(base, &[(KRALJI, 20)])
    );

    // The opponents have the four kings.
    let for_the_opponents = || kings_for_the_opponents(false);
    let base = declarer_parts(&[], for_the_opponents);
    assert_eq!(base[KRALJI], -10);
    // The declarer announces: -20.
    assert_eq!(
        declarer_parts(&[(A, &[ann(Bonus::Kralji)])], for_the_opponents),
        but(base, &[(KRALJI, -20)])
    );
    // An opponent announces and wins: the declarer writes -20.
    assert_eq!(
        declarer_parts(&[(B, &[ann(Bonus::Kralji)])], for_the_opponents),
        but(base, &[(KRALJI, -20)])
    );

    // The kings are split (written deal 1: the karo kralj is in a trick of
    // the opponents): the announcer's side loses 20. A and its partner C
    // write the same.
    let silent = deal_1_partner().0.score().unwrap();
    assert_eq!(silent.parts(A).kralji, 0);
    let game = with_round(&[(A, &[ann(Bonus::Kralji)])], deal_1_partner).0;
    assert_scores_agree(&game);
    let score = game.score().unwrap();
    assert_eq!(game.full().partner(), Some(C));
    assert_eq!(parts(&score, A), but(parts(&silent, A), &[(KRALJI, -20)]));
    assert_eq!(parts(&score, C), parts(&score, A));
    assert_eq!([score.entry(B), score.entry(D)], [0, 0]);
    // An opponent announces them there: +20 for A and C.
    let game = with_round(&[(B, &[ann(Bonus::Kralji)])], deal_1_partner).0;
    assert_scores_agree(&game);
    let score = game.score().unwrap();
    assert_eq!(parts(&score, A), but(parts(&silent, A), &[(KRALJI, 20)]));
    assert_eq!(parts(&score, C), parts(&score, A));
}

#[test]
fn c20_an_announced_bonus_replaces_the_silent_one_of_its_kind() {
    // An announced trula fails while the other side has all three: it
    // costs 20, not 20 and the silent 10.
    let base = declarer_parts(&[], trula_for_the_opponents);
    assert_eq!(base[TRULA], -10);
    let announced = declarer_parts(&[(A, &[ann(Bonus::Trula)])], trula_for_the_opponents);
    assert_eq!(announced[TRULA], -20);
    assert_eq!(
        announced.iter().sum::<i32>(),
        base.iter().sum::<i32>() - 10,
        "20 in place of 10, not on top of it"
    );
    // The same from the other side: B announces, A has all three.
    let base = declarer_parts(&[], trula_for_a);
    let announced = declarer_parts(&[(B, &[ann(Bonus::Trula)])], trula_for_a);
    assert_eq!((base[TRULA], announced[TRULA]), (10, 20));
    assert_eq!(announced.iter().sum::<i32>(), base.iter().sum::<i32>() + 10);

    // Kralji.
    let for_the_opponents = || kings_for_the_opponents(false);
    let base = declarer_parts(&[], for_the_opponents);
    let announced = declarer_parts(&[(A, &[ann(Bonus::Kralji)])], for_the_opponents);
    assert_eq!((base[KRALJI], announced[KRALJI]), (-10, -20));
    assert_eq!(announced.iter().sum::<i32>(), base.iter().sum::<i32>() - 10);

    // Kralj ultimo: 20 in place of the silent 10, won and lost.
    for (to_b, silent, loud) in [(false, 10, 20), (true, -10, -20)] {
        let deal = || own_king_in_the_last_trick(to_b);
        let base = declarer_parts(&[], deal);
        let announced = declarer_parts(&[(A, &[ann(Bonus::KraljUltimo)])], deal);
        assert_eq!(
            (base[KRALJ_ULTIMO], announced[KRALJ_ULTIMO]),
            (silent, loud)
        );
        assert_eq!(announced, but(base, &[(KRALJ_ULTIMO, loud)]));
    }

    // Pagat ultimo: 50 in place of the silent 25.
    let base = declarer_parts(&[], pagat_ultimo_own_wins);
    let announced = declarer_parts(&[(A, &[ann(Bonus::PagatUltimo)])], pagat_ultimo_own_wins);
    assert_eq!((base[PAGAT_ULTIMO], announced[PAGAT_ULTIMO]), (25, 50));
    assert_eq!(announced, but(base, &[(PAGAT_ULTIMO, 50)]));
    let base = declarer_parts(&[], trula_for_the_opponents);
    let announced = declarer_parts(&[(B, &[ann(Bonus::PagatUltimo)])], trula_for_the_opponents);
    assert_eq!((base[PAGAT_ULTIMO], announced[PAGAT_ULTIMO]), (-25, -50));
    assert_eq!(announced, but(base, &[(PAGAT_ULTIMO, -50)]));
}

#[test]
fn c21_announced_kralj_ultimo() {
    let king = kralj(Suit::Srce);
    let announce: &[(Seat, &[Action])] = &[(A, &[ann(Bonus::KraljUltimo)])];

    // The declarer holds the called king, announces, plays the king to the
    // last trick and takes that trick: +20.
    let deal = || own_king_in_the_last_trick(false);
    let base = declarer_parts(&[], deal);
    let (game, score) = said(announce, deal);
    assert_eq!(trick_of(&game, king), 12);
    assert_eq!(game.public().tricks()[11].winner, A);
    assert_eq!(parts(&score, A), but(base, &[(KRALJ_ULTIMO, 20)]));
    // The king is played to the last trick and an opponent takes it: -20.
    let deal = || own_king_in_the_last_trick(true);
    let base = declarer_parts(&[], deal);
    let (game, score) = said(announce, deal);
    assert_eq!(trick_of(&game, king), 12);
    assert_eq!(game.public().tricks()[11].winner, B);
    assert_eq!(parts(&score, A), but(base, &[(KRALJ_ULTIMO, -20)]));

    // The partner C holds the king, announces and wins the last trick with
    // it: +20 for both. C is public from its announcement on.
    let announce: &[(Seat, &[Action])] = &[(C, &[ann(Bonus::KraljUltimo)])];
    let deal = || partners_king(false);
    let base = declarer_parts(&[], deal);
    assert_eq!(base[KRALJ_ULTIMO], 10);
    let (game, score) = said(announce, deal);
    assert_eq!(game.public().announced(Bonus::KraljUltimo), Some(C));
    assert_eq!(game.public().partner(), PublicPartner::With(C));
    assert_eq!(trick_of(&game, king), 12);
    assert_eq!(game.public().tricks()[11].winner, C);
    assert_eq!(parts(&score, A), but(base, &[(KRALJ_ULTIMO, 20)]));
    assert_eq!(parts(&score, C), parts(&score, A));
    assert_eq!([score.entry(B), score.entry(D)], [0, 0]);

    // The king is played before the last trick (free play, D141): -20. The
    // silent bonus would be 0.
    let deal = || partners_king(true);
    let base = declarer_parts(&[], deal);
    assert_eq!(base[KRALJ_ULTIMO], 0);
    let (game, score) = said(announce, deal);
    assert_eq!(trick_of(&game, king), 5);
    assert_eq!(parts(&score, A), but(base, &[(KRALJ_ULTIMO, -20)]));
    assert_eq!(parts(&score, C), parts(&score, A));
}

#[test]
fn c22_announced_pagat_ultimo_follows_the_side_that_takes_the_last_trick() {
    let pagat_ultimo = ann(Bonus::PagatUltimo);
    let last = |game: &Game| game.public().tricks()[11];

    // The declarer's pagat takes the last trick: +50.
    let base = declarer_parts(&[], pagat_ultimo_own_wins);
    let (game, score) = said(&[(A, &[pagat_ultimo])], pagat_ultimo_own_wins);
    assert_eq!(trick_of(&game, PAGAT), 12);
    assert_eq!(last(&game).winner, A);
    assert_eq!(parts(&score, A), but(base, &[(PAGAT_ULTIMO, 50)]));
    // The declarer's pagat is played to the last trick and D takes it: -50.
    let base = declarer_parts(&[], pagat_ultimo_own_loses);
    let (game, score) = said(&[(A, &[pagat_ultimo])], pagat_ultimo_own_loses);
    assert_eq!(trick_of(&game, PAGAT), 12);
    assert_eq!(last(&game).winner, D);
    assert_eq!(parts(&score, A), but(base, &[(PAGAT_ULTIMO, -50)]));

    // The partner C announces. Its pagat is played to the last trick and
    // the declarer takes that trick with the XIX: the side takes it, with
    // another card. Won (D126, D141 B13-4 a): +50 for both.
    let deal = || pagat_under_the_xix(A, Tri, Some(Suit::Kriz));
    let base = declarer_parts(&[], deal);
    assert_eq!(base[PAGAT_ULTIMO], 25);
    let (game, score) = said(&[(C, &[pagat_ultimo])], deal);
    assert_eq!(game.full().partner(), Some(C));
    assert_eq!(last(&game).cards[2], PAGAT);
    assert_eq!(last(&game).winner, A);
    assert_eq!(parts(&score, A), but(base, &[(PAGAT_ULTIMO, 50)]));
    assert_eq!(parts(&score, C), parts(&score, A));
    // The same hands, the opponent D takes the last trick: -50 for both.
    let deal = || pagat_under_the_xix(D, Tri, Some(Suit::Kriz));
    let base = declarer_parts(&[], deal);
    let (game, score) = said(&[(C, &[pagat_ultimo])], deal);
    assert_eq!(last(&game).winner, D);
    assert_eq!(parts(&score, A), but(base, &[(PAGAT_ULTIMO, -50)]));
    assert_eq!(parts(&score, C), parts(&score, A));

    // An opponent announces. A plays solo: B, C and D are one side. C's
    // pagat is in the last trick and D takes it: the owner's side takes
    // it, so the declarer writes -50.
    let deal = || pagat_under_the_xix(D, SoloTri, None);
    let base = declarer_parts(&[], deal);
    let (game, score) = said(&[(C, &[pagat_ultimo])], deal);
    assert_eq!(last(&game).winner, D);
    assert_eq!(parts(&score, A), but(base, &[(PAGAT_ULTIMO, -50)]));
    // The declarer takes the opponent's announced pagat: +50.
    let deal = || pagat_under_the_xix(A, SoloTri, None);
    let base = declarer_parts(&[], deal);
    let (game, score) = said(&[(C, &[pagat_ultimo])], deal);
    assert_eq!(last(&game).winner, A);
    assert_eq!(parts(&score, A), but(base, &[(PAGAT_ULTIMO, 50)]));
    let base = declarer_parts(&[], pagat_ultimo_opponent_loses);
    assert_eq!(
        declarer_parts(&[(D, &[pagat_ultimo])], pagat_ultimo_opponent_loses),
        but(base, &[(PAGAT_ULTIMO, 50)])
    );
    // An opponent's pagat takes the last trick itself: -50.
    let base = declarer_parts(&[], trula_for_the_opponents);
    assert_eq!(
        declarer_parts(&[(B, &[pagat_ultimo])], trula_for_the_opponents),
        but(base, &[(PAGAT_ULTIMO, -50)])
    );

    // The pagat is played before the last trick (free play, D141): -50,
    // even though it takes its trick. The silent bonus would be 0.
    let base = declarer_parts(&[], trula_for_a);
    assert_eq!(base[PAGAT_ULTIMO], 0);
    let (game, score) = said(&[(A, &[pagat_ultimo])], trula_for_a);
    assert_eq!(trick_of(&game, PAGAT), 11);
    assert_eq!(game.public().tricks()[10].winner, A);
    assert_eq!(parts(&score, A), but(base, &[(PAGAT_ULTIMO, -50)]));
}

#[test]
fn c23_announced_valat_scores_500_and_nothing_else() {
    let only = |valat: i32| but([0; 8], &[(VALAT, valat)]);
    let valat: &[Action] = &[ann(Bonus::Valat)];

    // The declarer announces and takes all twelve tricks: +500. No game,
    // no difference, and the trula and the pagat ultimo of that deal are
    // not scored.
    let (_, score) = said(&[(A, valat)], all_twelve_for_a);
    assert_eq!(parts(&score, A), only(500));
    assert_eq!(score.entries(), [500, 0, 0, 0]);
    // Announced bonuses next to it are not scored either.
    let all: &[Action] = &[
        ann(Bonus::Trula),
        ann(Bonus::Kralji),
        ann(Bonus::PagatUltimo),
        ann(Bonus::Valat),
    ];
    let (_, score) = said(&[(A, all)], all_twelve_for_a);
    assert_eq!(parts(&score, A), only(500));
    assert_eq!(score.entries(), [500, 0, 0, 0]);

    // The declarer announces and takes eleven tricks: -500 and nothing
    // else, although the game is won and the trula is A's.
    let base = declarer_parts(&[], trula_for_a);
    assert!(base[GAME] > 0 && base[TRULA] == 10);
    let (game, score) = said(&[(A, valat)], trula_for_a);
    assert_eq!(tricks_won(&game), [11, 1, 0, 0]);
    assert_eq!(parts(&score, A), only(-500));
    assert_eq!(score.entries(), [-500, 0, 0, 0]);
    // With more announced: still -500 only.
    let (_, score) = said(&[(A, all)], trula_for_a);
    assert_eq!(parts(&score, A), only(-500));

    // An opponent announces and the opponents take all twelve: 500 for
    // the owner, so the declarer writes -500.
    let (_, score) = said(&[(B, valat)], all_twelve_for_b);
    assert_eq!(parts(&score, A), only(-500));
    assert_eq!(score.entries(), [-500, 0, 0, 0]);
    let (_, score) = said(&[(D, valat)], all_twelve_for_b);
    assert_eq!(parts(&score, A), only(-500));
    // An opponent announces and fails: 500 against the owner, +500 for
    // the declarer and nothing else. Eleven tricks for the opponents...
    let (game, score) = said(&[(B, valat)], trula_for_the_opponents);
    assert_eq!(tricks_won(&game), [1, 11, 0, 0]);
    assert_eq!(parts(&score, A), only(500));
    // ...and one trick for them.
    let (_, score) = said(&[(C, valat)], trula_for_a);
    assert_eq!(parts(&score, A), only(500));
    assert_eq!(score.entries(), [500, 0, 0, 0]);

    // The declarer announces and the opponents take all twelve: 500
    // against the owner. Not 750, and not the silent 250.
    let (_, score) = said(&[(A, valat)], all_twelve_for_b);
    assert_eq!(parts(&score, A), only(-500));
    // An opponent announces and the declarer takes all twelve: +500.
    let (_, score) = said(&[(B, valat)], all_twelve_for_a);
    assert_eq!(parts(&score, A), only(500));

    // A lost mond is still written (D125), next to a valat that is won...
    let (_, score) = said(&[(A, valat)], all_twelve_and_a_caught_mond);
    assert_eq!(parts(&score, A), only(500));
    assert_eq!(parts(&score, B), but([0; 8], &[(MONDFANG, -20)]));
    assert_eq!(score.entries(), [500, -20, 0, 0]);
    // ...and next to one that fails.
    let (_, score) = said(&[(A, valat)], declarers_mond_is_caught);
    assert_eq!(
        parts(&score, A),
        but([0; 8], &[(VALAT, -500), (MONDFANG, -20)])
    );
    assert_eq!(score.entries(), [-520, 0, 0, 0]);
}

#[test]
fn c24_a_silent_valat_next_to_announced_bonuses_scores_250_and_nothing_else() {
    let only = |valat: i32| but([0; 8], &[(VALAT, valat)]);
    // A takes all twelve without announcing the valat. A announced trula
    // and pagat ultimo, and makes both: they are not scored.
    let said_by_a: &[Action] = &[
        ann(Bonus::Trula),
        ann(Bonus::Kralji),
        ann(Bonus::PagatUltimo),
    ];
    let (game, score) = said(&[(A, said_by_a)], all_twelve_for_a);
    assert_eq!(trick_of(&game, PAGAT), 12);
    assert_eq!(parts(&score, A), only(250));
    assert_eq!(score.entries(), [250, 0, 0, 0]);
    // An opponent's announcement is not scored either.
    let (_, score) = said(&[(B, &[ann(Bonus::Kralji)])], all_twelve_for_a);
    assert_eq!(parts(&score, A), only(250));

    // The opponents take all twelve. A announced trula and loses it: -250
    // only, not -270.
    let (_, score) = said(&[(A, &[ann(Bonus::Trula)])], all_twelve_for_b);
    assert_eq!(parts(&score, A), only(-250));
    assert_eq!(score.entries(), [-250, 0, 0, 0]);
    // B announced pagat ultimo and makes it: -250 only.
    let (game, score) = said(&[(B, &[ann(Bonus::PagatUltimo)])], all_twelve_for_b);
    assert_eq!(game.public().tricks()[11].cards[0], PAGAT);
    assert_eq!(parts(&score, A), only(-250));

    // A lost mond is still written.
    let (_, score) = said(
        &[(A, &[ann(Bonus::Trula)]), (B, &[k(Target::Trula)])],
        all_twelve_and_a_caught_mond,
    );
    assert_eq!(parts(&score, A), only(250));
    assert_eq!(score.entries(), [250, -20, 0, 0]);
}

#[test]
fn c25_a_bonus_nobody_announced_is_scored_as_in_flow_a() {
    // B has the trula and its pagat takes the last trick. A announces
    // kralji: trula and pagat ultimo stay the silent -10 and -25.
    let base = declarer_parts(&[], trula_for_the_opponents);
    assert_eq!((base[TRULA], base[PAGAT_ULTIMO]), (-10, -25));
    let announced = declarer_parts(&[(A, &[ann(Bonus::Kralji)])], trula_for_the_opponents);
    assert_eq!(announced[KRALJI].abs(), 20);
    assert_eq!(announced, but(base, &[(KRALJI, announced[KRALJI])]));

    // A has the four kings and announces trula: the kralji stay +10.
    let base = declarer_parts(&[], kings_for_a);
    assert_eq!(base[KRALJI], 10);
    let announced = declarer_parts(&[(A, &[ann(Bonus::Trula)])], kings_for_a);
    assert_eq!(announced[TRULA].abs(), 20);
    assert_eq!(announced, but(base, &[(TRULA, announced[TRULA])]));

    // The silent kralj ultimo next to an announced trula by an opponent.
    let deal = || partners_king(false);
    let base = declarer_parts(&[], deal);
    assert_eq!(base[KRALJ_ULTIMO], 10);
    let announced = declarer_parts(&[(B, &[ann(Bonus::Trula)])], deal);
    assert_eq!(announced, but(base, &[(TRULA, announced[TRULA])]));
    assert_eq!(announced[KRALJ_ULTIMO], 10);
}

/// The turns that take `target` to `level`, in a solo of A: B says kontra,
/// A re, B sub, A mort. `first` is said by A before that.
fn ladder(first: &[Action], target: Target, level: usize) -> Vec<(Seat, Vec<Action>)> {
    let mut turns = vec![(A, first.to_vec())];
    for step in 0..level {
        turns.push((if step % 2 == 0 { B } else { A }, vec![k(target)]));
    }
    turns
}

/// `ladder` in the shape `said` takes.
fn borrowed(turns: &[(Seat, Vec<Action>)]) -> Vec<(Seat, &[Action])> {
    turns
        .iter()
        .map(|(seat, actions)| (*seat, actions.as_slice()))
        .collect()
}

#[test]
fn c26_kontra_on_the_game_multiplies_game_and_difference_only() {
    // A wins solo ena with a silent trula.
    let base = declarer_parts(&[], trula_for_a);
    assert!(base[GAME] == 60 && base[DIFFERENCE] > 0 && base[TRULA] == 10);
    for (level, times) in [(1, 2), (2, 4), (3, 8), (4, 16)] {
        let turns = ladder(&[], Target::Game, level);
        let (game, score) = said(&borrowed(&turns), trula_for_a);
        assert_eq!(game.public().kontra_level(Target::Game), level as u8);
        assert_eq!(
            parts(&score, A),
            but(
                base,
                &[
                    (GAME, base[GAME] * times),
                    (DIFFERENCE, base[DIFFERENCE] * times)
                ]
            ),
            "level {level}"
        );
        assert_eq!(
            score.entry(A),
            (base[GAME] + base[DIFFERENCE]) * times + 10,
            "level {level}"
        );
    }

    // A loses solo ena; the opponents have the silent trula and pagat
    // ultimo. Kontra doubles the loss of game and difference and nothing
    // else.
    let base = declarer_parts(&[], trula_for_the_opponents);
    assert!(base[GAME] == -60 && base[DIFFERENCE] < 0);
    let turns = ladder(&[], Target::Game, 1);
    assert_eq!(
        declarer_parts(&borrowed(&turns), trula_for_the_opponents),
        but(base, &[(GAME, -120), (DIFFERENCE, 2 * base[DIFFERENCE])])
    );
    // An announced bonus next to the game is not touched by the game's
    // kontra.
    let base = declarer_parts(&[], kings_for_a);
    let turns = ladder(&[ann(Bonus::Kralji)], Target::Game, 2);
    assert_eq!(
        declarer_parts(&borrowed(&turns), kings_for_a),
        but(
            base,
            &[
                (GAME, 4 * base[GAME]),
                (DIFFERENCE, 4 * base[DIFFERENCE]),
                (KRALJI, 20)
            ]
        )
    );
}

#[test]
fn c27_kontra_on_a_bonus_multiplies_that_announced_bonus_only() {
    // A announces trula and makes it: 20, 40, 80, 160, 320.
    let base = declarer_parts(&[], trula_for_a);
    for (level, trula) in [(0, 20), (1, 40), (2, 80), (3, 160), (4, 320)] {
        let turns = ladder(&[ann(Bonus::Trula)], Target::Trula, level);
        let (game, score) = said(&borrowed(&turns), trula_for_a);
        assert_eq!(game.public().kontra_level(Target::Trula), level as u8);
        assert_eq!(game.public().kontra_level(Target::Game), 0);
        // Game and difference are not touched.
        assert_eq!(
            parts(&score, A),
            but(base, &[(TRULA, trula)]),
            "level {level}"
        );
    }
    // Lost under kontra: -40. Under re: -80.
    let base = declarer_parts(&[], pagat_ultimo_own_loses);
    for (level, trula) in [(1, -40), (2, -80)] {
        let mut turns = vec![(A, vec![ann(Bonus::Trula)]), (D, vec![k(Target::Trula)])];
        if level == 2 {
            turns.push((A, vec![k(Target::Trula)]));
        }
        assert_eq!(
            declarer_parts(&borrowed(&turns), pagat_ultimo_own_loses),
            but(base, &[(TRULA, trula)])
        );
    }

    // The kontra on one bonus does not touch another announced bonus. A
    // announces trula and kralji and has both; B says kontra on the trula.
    let base = declarer_parts(&[], kings_for_a);
    let turns = ladder(&[ann(Bonus::Trula), ann(Bonus::Kralji)], Target::Trula, 1);
    let announced = declarer_parts(&borrowed(&turns), kings_for_a);
    assert_eq!(announced[KRALJI], 20);
    assert_eq!(announced[TRULA].abs(), 40);
    assert_eq!(
        announced,
        but(base, &[(TRULA, announced[TRULA]), (KRALJI, 20)])
    );

    // Each of the other bonuses under kontra. Pagat ultimo, won: 100.
    let base = declarer_parts(&[], pagat_ultimo_own_wins);
    let turns = ladder(&[ann(Bonus::PagatUltimo)], Target::PagatUltimo, 1);
    assert_eq!(
        declarer_parts(&borrowed(&turns), pagat_ultimo_own_wins),
        but(base, &[(PAGAT_ULTIMO, 100)])
    );
    // An opponent's pagat ultimo, won, under the declarer's kontra: -100.
    let base = declarer_parts(&[], trula_for_the_opponents);
    assert_eq!(
        declarer_parts(
            &[
                (B, &[ann(Bonus::PagatUltimo)]),
                (A, &[k(Target::PagatUltimo)])
            ],
            trula_for_the_opponents
        ),
        but(base, &[(PAGAT_ULTIMO, -100)])
    );
    // Kralj ultimo by the partner, won, under D's kontra and A's re: 40
    // and 80.
    let deal = || partners_king(false);
    let base = declarer_parts(&[], deal);
    let kontra: &[(Seat, &[Action])] = &[
        (C, &[ann(Bonus::KraljUltimo)]),
        (D, &[k(Target::KraljUltimo)]),
    ];
    assert_eq!(
        declarer_parts(kontra, deal),
        but(base, &[(KRALJ_ULTIMO, 40)])
    );
    let re: &[(Seat, &[Action])] = &[
        (C, &[ann(Bonus::KraljUltimo)]),
        (D, &[k(Target::KraljUltimo)]),
        (A, &[k(Target::KraljUltimo)]),
    ];
    assert_eq!(declarer_parts(re, deal), but(base, &[(KRALJ_ULTIMO, 80)]));
    // Kralji, lost, under kontra: -40.
    let for_the_opponents = || kings_for_the_opponents(false);
    let base = declarer_parts(&[], for_the_opponents);
    let turns = ladder(&[ann(Bonus::Kralji)], Target::Kralji, 1);
    assert_eq!(
        declarer_parts(&borrowed(&turns), for_the_opponents),
        but(base, &[(KRALJI, -40)])
    );
}

#[test]
fn c28_silent_bonuses_and_the_mondfang_are_never_multiplied() {
    // The game at mort. The silent trula and the silent pagat ultimo of
    // the opponents stay -10 and -25.
    let base = declarer_parts(&[], trula_for_the_opponents);
    assert_eq!((base[TRULA], base[PAGAT_ULTIMO]), (-10, -25));
    let turns = ladder(&[], Target::Game, 4);
    assert_eq!(
        declarer_parts(&borrowed(&turns), trula_for_the_opponents),
        but(
            base,
            &[(GAME, 16 * base[GAME]), (DIFFERENCE, 16 * base[DIFFERENCE])]
        )
    );
    // The silent kralji and the silent kralj ultimo.
    let base = declarer_parts(&[], kings_for_a);
    let turns = ladder(&[], Target::Game, 3);
    assert_eq!(
        declarer_parts(&borrowed(&turns), kings_for_a)[KRALJI],
        base[KRALJI]
    );
    let deal = || own_king_in_the_last_trick(false);
    let base = declarer_parts(&[], deal);
    let kontra: &[(Seat, &[Action])] = &[(B, &[k(Target::Game)])];
    assert_eq!(
        declarer_parts(kontra, deal),
        but(
            base,
            &[(GAME, 2 * base[GAME]), (DIFFERENCE, 2 * base[DIFFERENCE])]
        )
    );

    // The mondfang of the declarer: -20 under kontra and under mort.
    for level in [1, 4] {
        let turns = ladder(&[], Target::Game, level);
        let (_, score) = said(&borrowed(&turns), declarers_mond_is_caught);
        assert_eq!(score.parts(A).mondfang, -20, "level {level}");
        assert!(score.parts(A).game.abs() > 60);
    }
    // The mondfang of an opponent: -20, and it is its whole entry.
    let turns = ladder(&[], Target::Game, 2);
    let (_, score) = said(&borrowed(&turns), opponents_mond_is_caught);
    assert_eq!(parts(&score, B), but([0; 8], &[(MONDFANG, -20)]));
    assert_eq!(score.entry(B), -20);
    // Also when the mond's player announced something that is under
    // kontra: A announces trula, loses the mond and with it the trula.
    let turns = ladder(&[ann(Bonus::Trula)], Target::Trula, 1);
    let (_, score) = said(&borrowed(&turns), declarers_mond_is_caught);
    assert_eq!(score.parts(A).trula, -40);
    assert_eq!(score.parts(A).mondfang, -20);

    // The silent valat is never multiplied: see c29.
}

#[test]
fn c29_with_a_valat_only_the_kontra_on_the_announced_valat_counts() {
    let only = |valat: i32| but([0; 8], &[(VALAT, valat)]);
    // A silent valat: the kontra on the game has no effect.
    let turns = ladder(&[], Target::Game, 1);
    let (_, score) = said(&borrowed(&turns), all_twelve_for_a);
    assert_eq!(parts(&score, A), only(250));
    let turns = ladder(&[], Target::Game, 2);
    let (_, score) = said(&borrowed(&turns), all_twelve_for_b);
    assert_eq!(parts(&score, A), only(-250));
    // A silent valat and a kontra on an announced trula: no effect.
    let turns = ladder(&[ann(Bonus::Trula)], Target::Trula, 1);
    let (_, score) = said(&borrowed(&turns), all_twelve_for_a);
    assert_eq!(parts(&score, A), only(250));

    // An announced valat. Kontra on the game and on the trula: no effect.
    let said_by_a = [ann(Bonus::Trula), ann(Bonus::Valat)];
    let turns: &[(Seat, &[Action])] = &[(A, &said_by_a), (B, &[k(Target::Game), k(Target::Trula)])];
    let (_, score) = said(turns, all_twelve_for_a);
    assert_eq!(parts(&score, A), only(500));
    // Kontra on the valat itself: 1000, 2000, 4000, 8000.
    for (level, valat) in [(1, 1_000), (2, 2_000), (3, 4_000), (4, 8_000)] {
        let turns = ladder(&said_by_a, Target::Valat, level);
        let (_, score) = said(&borrowed(&turns), all_twelve_for_a);
        assert_eq!(parts(&score, A), only(valat), "level {level}");
        // Lost: A takes eleven tricks.
        let (_, score) = said(&borrowed(&turns), trula_for_a);
        assert_eq!(parts(&score, A), only(-valat), "level {level}");
    }
    // All three under kontra: only the valat's counts.
    let turns: &[(Seat, &[Action])] = &[
        (A, &said_by_a),
        (B, &[k(Target::Game), k(Target::Trula), k(Target::Valat)]),
    ];
    let (_, score) = said(turns, all_twelve_for_a);
    assert_eq!(parts(&score, A), only(1_000));
    assert_eq!(score.entries(), [1_000, 0, 0, 0]);
    // An opponent's valat under the declarer's kontra, won and lost.
    let turns: &[(Seat, &[Action])] = &[(B, &[ann(Bonus::Valat)]), (A, &[k(Target::Valat)])];
    let (_, score) = said(turns, all_twelve_for_b);
    assert_eq!(parts(&score, A), only(-1_000));
    let (_, score) = said(turns, trula_for_a);
    assert_eq!(parts(&score, A), only(1_000));
    // The mondfang next to it is not multiplied.
    let turns = ladder(&[ann(Bonus::Valat)], Target::Valat, 1);
    let (_, score) = said(&borrowed(&turns), all_twelve_and_a_caught_mond);
    assert_eq!(score.entries(), [1_000, -20, 0, 0]);
}

#[test]
fn c30_kontra_examples_on_whole_deals() {
    // Tri won with 43 points writes 20. The deal is `seeded_game(167)`: A
    // declares, C is the partner, no bonus and no mondfang. It was found
    // by trying seeds; what it must give is written here by hand.
    let facts = |game: &Game, declarer: Seat, points: u8| {
        assert_eq!(game.public().contract(), Some(Tri));
        assert_eq!(game.public().declarer(), Some(declarer));
        let second = second_score(game);
        assert_eq!(second.declarer_points, points);
        // Nothing but game and difference in any entry.
        for seat_parts in second.parts {
            assert_eq!(seat_parts[2..], [0; 6]);
        }
    };
    let silent = seeded_game(167);
    facts(&silent, A, 43);
    assert_eq!(silent.full().partner(), Some(C));
    // 10 for tri, and 43 - 35 = 8, rounded 10.
    assert_eq!(silent.score().unwrap().entries(), [20, 0, 20, 0]);
    // Kontra by B: 40. Re by A: 80. Sub by D: 160. Mort by C: 320.
    let steps: [(Seat, i32); 4] = [(B, 40), (A, 80), (D, 160), (C, 320)];
    for level in 1..=4 {
        let turns: Vec<(Seat, &[Action])> = steps[..level]
            .iter()
            .map(|&(seat, _)| (seat, &[Action::Kontra(Target::Game)][..]))
            .collect();
        let game = with_round(&turns, || seeded_game(167));
        facts(&game, A, 43);
        assert_eq!(game.public().kontra_level(Target::Game), level as u8);
        assert_scores_agree(&game);
        let entry = steps[level - 1].1;
        assert_eq!(game.score().unwrap().entries(), [entry, 0, entry, 0]);
    }

    // Tri lost with 21 points and kontra writes -50. `seeded_game(367)`:
    // A declares and plays alone.
    let silent = seeded_game(367);
    facts(&silent, A, 21);
    assert_eq!(silent.full().partner(), None);
    // -10 for tri, and 21 - 35 = -14, rounded -15.
    assert_eq!(silent.score().unwrap().entries(), [-25, 0, 0, 0]);
    let game = with_round(&[(B, &[k(Target::Game)])], || seeded_game(367));
    facts(&game, A, 21);
    assert_scores_agree(&game);
    assert_eq!(game.score().unwrap().entries(), [-50, 0, 0, 0]);

    // An announced trula lost under kontra costs 40.
    let base = declarer_parts(&[], pagat_ultimo_own_loses);
    let turns: &[(Seat, &[Action])] = &[(A, &[ann(Bonus::Trula)]), (D, &[k(Target::Trula)])];
    let lost = declarer_parts(turns, pagat_ultimo_own_loses);
    assert_eq!(lost[TRULA], -40);
    assert_eq!(lost.iter().sum::<i32>(), base.iter().sum::<i32>() - 40);

    // The game under kontra and an announced trula won without kontra:
    // twice game and difference, plus 20.
    let base = declarer_parts(&[], trula_for_a);
    let turns: &[(Seat, &[Action])] = &[(A, &[ann(Bonus::Trula)]), (B, &[k(Target::Game)])];
    let (_, score) = said(turns, trula_for_a);
    assert_eq!(score.entry(A), 2 * (base[GAME] + base[DIFFERENCE]) + 20);
    assert_eq!(score.parts(A).trula, 20);
}

/// The third example of the source, built by hand. A plays ena and calls
/// srce; C holds the srce kralj.
///
/// - A: škis, XX-XIII; karo 4; pik 8, 7.
/// - B: mond, II-V; karo 3; pik 9, 10, fant, kaval, dama; srce dama.
/// - C: VI-IX; srce kralj, 4, 3, 2; križ 7, 8, 9, 10.
/// - D: X, XI, XII, pagat; srce 1, fant, kaval; karo kralj; križ fant,
///   kaval, dama, kralj.
/// - Talon: karo 2 | karo 1 | karo fant | karo kaval | karo dama | pik
///   kralj. A takes the karo 2 and puts it down again.
///
/// The play:
///
/// ```text
///  1.   A škis, B mond, C VI, D X. The mond is caught.
///  2-4. A XX, XIX, XVIII; B II, III, IV; C VII, VIII, IX; D XI, XII and
///       the pagat.
///  5-9. A XVII, XVI, XV, XIV, XIII; B V, karo 3, pik 9, 10, fant; C srce
///       4, 3, 2, križ 7, 8; D srce 1, fant, kaval, križ fant, kaval.
///  10.  A pik 8, B pik dama, C križ 9, D križ kralj: B wins.
///  11.  B pik kaval, C križ 10, D karo kralj, A pik 7: B wins.
///  12.  B srce dama, C srce kralj, D križ dama, A karo 4: C wins.
/// ```
///
/// The opponents hold tricks 10 and 11 and five talon cards: 13 cards
/// worth 1+4+1+5, 3+1+5+1 and 1+2+3+4+5, 36 in all; 13 cards lose 9, so 27
/// points. The declarer's side has 70 - 27 = 43.
fn source_example_ena() -> (Game, Score) {
    use Suit::{Karo, Kriz, Pik, Srce};
    let mut a = vec![S];
    a.extend((13..=20).rev().map(tarok));
    a.extend([pik(8), pik(7), karo(4)]);
    let b = [
        MOND,
        tarok(2),
        tarok(3),
        tarok(4),
        tarok(5),
        karo(3),
        pik(9),
        pik(10),
        fant(Pik),
        dama(Pik),
        kaval(Pik),
        dama(Srce),
    ];
    let c = [
        tarok(6),
        tarok(7),
        tarok(8),
        tarok(9),
        srce(4),
        srce(3),
        srce(2),
        kriz(7),
        kriz(8),
        kriz(9),
        kriz(10),
        kralj(Srce),
    ];
    let d = [
        tarok(10),
        tarok(11),
        tarok(12),
        PAGAT,
        srce(1),
        fant(Srce),
        kaval(Srce),
        fant(Kriz),
        kaval(Kriz),
        kralj(Kriz),
        kralj(Karo),
        dama(Kriz),
    ];
    let talon = [
        karo(2),
        karo(1),
        fant(Karo),
        kaval(Karo),
        dama(Karo),
        kralj(Pik),
    ];
    let deal = common::flow::whole_deal([&a, &b, &c, &d], talon);
    let game = a_declares(&deal, Ena, Some(Srce));
    assert_eq!(game.full().partner(), Some(C));
    // Every card is played in the order of its list.
    finished(game, [&a, &b, &c, &d])
}

#[test]
fn c31_the_three_examples_of_the_source() {
    // 1. Dve with 35 points and an announced pagat ultimo that fails
    // writes -70. `seeded_game(476)`: C declares dve, A is the partner, 35
    // points, no bonus. C holds the pagat and plays it to trick 7.
    let facts = |game: &Game| {
        assert_eq!(game.public().contract(), Some(Dve));
        assert_eq!(game.public().declarer(), Some(C));
        assert_eq!(game.full().partner(), Some(A));
        assert_eq!(second_score(game).declarer_points, 35);
        assert_eq!(trick_of(game, PAGAT), 7);
        let trick = game.public().tricks()[6];
        let position = trick.cards.iter().position(|&card| card == PAGAT).unwrap();
        assert_eq!(trick.leader.plus(position), C, "C plays the pagat");
    };
    let silent = seeded_game(476);
    facts(&silent);
    // Dve lost with 35: -20 and a difference of 0.
    assert_eq!(silent.score().unwrap().entries(), [-20, 0, -20, 0]);
    let game = with_round(&[(C, &[ann(Bonus::PagatUltimo)])], || seeded_game(476));
    facts(&game);
    assert_scores_agree(&game);
    let score = game.score().unwrap();
    assert_eq!(
        parts(&score, C),
        but([0; 8], &[(GAME, -20), (PAGAT_ULTIMO, -50)])
    );
    assert_eq!(score.entries(), [-70, 0, -70, 0]);

    // 2. Solo tri won by 15 with an announced trula writes 75.
    // `seeded_game(3185)`: C plays solo tri, 51 points, and has škis, mond
    // and pagat in its pile.
    let facts = |game: &Game| {
        assert_eq!(game.public().contract(), Some(SoloTri));
        assert_eq!(game.public().declarer(), Some(C));
        assert_eq!(second_score(game).declarer_points, 51);
    };
    let silent = seeded_game(3185);
    facts(&silent);
    // 40 for solo tri, 51 - 35 = 16 rounded 15, and the silent trula.
    assert_eq!(silent.score().unwrap().entries(), [0, 0, 65, 0]);
    let game = with_round(&[(C, &[ann(Bonus::Trula)])], || seeded_game(3185));
    facts(&game);
    assert_scores_agree(&game);
    let score = game.score().unwrap();
    assert_eq!(
        parts(&score, C),
        but([0; 8], &[(GAME, 40), (DIFFERENCE, 15), (TRULA, 20)])
    );
    assert_eq!(score.entries(), [0, 0, 75, 0]);

    // 3. Ena won by 10, kralj ultimo announced by the partner and won, a
    // silent trula: 70. An opponent whose mond is captured writes -20.
    let (game, score) = said(&[(C, &[ann(Bonus::KraljUltimo)])], source_example_ena);
    assert_eq!(game.public().contract(), Some(Ena));
    assert_eq!(tricks_won(&game), [9, 2, 1, 0]);
    assert_eq!(score.declarer_points(), 43);
    assert_eq!(score.opponents_points(), 27);
    assert_eq!(skis_mond_trick(&game), Some(0));
    assert_eq!(game.public().tricks()[0].cards[..2], [S, MOND]);
    assert_eq!(trick_of(&game, kralj(Suit::Srce)), 12);
    assert_eq!(game.public().tricks()[11].winner, C);
    assert_eq!(trick_of(&game, PAGAT), 4);
    // 30 for ena, 43 - 35 = 8 rounded 10, kralj ultimo 20, trula 10.
    let expected = but(
        [0; 8],
        &[
            (GAME, 30),
            (DIFFERENCE, 10),
            (TRULA, 10),
            (KRALJ_ULTIMO, 20),
        ],
    );
    assert_eq!(parts(&score, A), expected);
    assert_eq!(parts(&score, C), expected);
    assert_eq!(parts(&score, B), but([0; 8], &[(MONDFANG, -20)]));
    assert_eq!(score.entries(), [70, -20, 70, 0]);
}

#[test]
fn c32_the_ten_parts_of_a_seat_add_up_to_its_entry() {
    // A deal with multiplied parts: the game at re, an announced trula at
    // kontra, a silent pagat ultimo... and one with a valat at kontra and
    // a mondfang.
    let game_and_trula: &[(Seat, &[Action])] = &[
        (A, &[ann(Bonus::Trula)]),
        (B, &[k(Target::Game), k(Target::Trula)]),
        (A, &[k(Target::Game)]),
    ];
    let base = declarer_parts(&[], trula_for_a);
    let (_, score) = said(game_and_trula, trula_for_a);
    let rewards = encode::rewards(&score);
    // A multiplied part holds the multiplied amount.
    assert_eq!(
        rewards.entry_parts[A.index()],
        [
            4 * base[GAME],
            4 * base[DIFFERENCE],
            40,
            0,
            0,
            0,
            0,
            0,
            0,
            0
        ]
    );
    assert_eq!(
        score.parts(A).entry(),
        4 * (base[GAME] + base[DIFFERENCE]) + 40
    );

    let valat = ladder(&[ann(Bonus::Valat)], Target::Valat, 1);
    let deals: [(Game, Score); 4] = [
        said(game_and_trula, trula_for_a),
        said(&borrowed(&valat), all_twelve_and_a_caught_mond),
        said(
            &[(B, &[ann(Bonus::PagatUltimo), k(Target::Game)])],
            trula_for_the_opponents,
        ),
        said(&[(C, &[ann(Bonus::KraljUltimo)])], source_example_ena),
    ];
    for (game, score) in deals {
        let rewards = encode::rewards(&score);
        assert_eq!(rewards.entries, score.entries());
        let largest = rewards
            .reward
            .iter()
            .fold(0f32, |most, reward| most.max(reward.abs()));
        let room = 1e-6 * largest.max(1.0);
        for seat in Seat::ALL {
            let row = rewards.entry_parts[seat.index()];
            assert_eq!(row[..8], parts(&score, seat));
            assert_eq!(row.iter().sum::<i32>(), score.entry(seat), "{seat:?}");
            assert_eq!(score.parts(seat).entry(), score.entry(seat));
            // The reward parts add up to the reward. Rewards are 32-bit
            // numbers: the room for rounding is 1e-6 for rewards up to 1
            // and grows in proportion above that.
            let sum: f32 = rewards.parts[seat.index()].iter().sum();
            assert!((sum - rewards.reward[seat.index()]).abs() < room);
        }
        // Rewards add up to zero.
        assert!(rewards.reward.iter().sum::<f32>().abs() < room);
        assert_eq!(game.score(), Some(score));
    }
}
