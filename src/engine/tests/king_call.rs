//! Spec 005, criteria 12-16: the king call.
//!
//! Seats: A is forehand (seat 0), then B, C, D; D (seat 3) deals.

mod common;

use common::flow::{A, B, C, D, act, deal_with, declare, finish, game, legal, run_until};
use common::{SUITS, kralj, tarok};
use tarok_engine::Contract::{Dve, Ena, SoloDve, SoloEna, SoloTri, Tri};
use tarok_engine::{Action, Card, Deal, Game, Phase, PublicPartner, Seat, Suit};

fn calls() -> Vec<Action> {
    SUITS.into_iter().map(Action::CallKing).collect()
}

/// A holds the srce kralj, B the karo kralj, C the pik kralj, D the križ
/// kralj. No king lies in the talon.
fn one_king_each() -> Deal {
    deal_with(
        [
            &[kralj(Suit::Srce)],
            &[kralj(Suit::Karo)],
            &[kralj(Suit::Pik)],
            &[kralj(Suit::Kriz)],
        ],
        &[tarok(2), tarok(3), tarok(4), tarok(5), tarok(6), tarok(7)],
    )
}

fn holder_of(game: &Game, card: Card) -> Option<Seat> {
    Seat::ALL
        .into_iter()
        .find(|&seat| game.full().hand(seat).contains(card))
}

#[test]
fn c12_declarer_names_one_of_four_suits_also_an_own_king() {
    for contract in [Tri, Dve, Ena] {
        for declarer in [A, B, C, D] {
            // Only forehand can play tri.
            if contract == Tri && declarer != A {
                continue;
            }
            let mut game = game(&one_king_each());
            declare(&mut game, declarer, contract);
            assert_eq!(game.phase(), Phase::KingCall);
            assert_eq!(game.to_act(), Some(declarer));
            assert_eq!(legal(&game), calls());
            assert_eq!(game.public().called_suit(), None);

            // Every suit is accepted, the declarer's own too.
            for suit in SUITS {
                let mut game = game.clone();
                act(&mut game, declarer, Action::CallKing(suit));
                assert_eq!(game.public().called_suit(), Some(suit));
                assert_eq!(game.public().called_king(), Some(kralj(suit)));
                assert_eq!(game.phase(), Phase::TalonPick);
            }
        }
    }
}

#[test]
fn c13_solo_contracts_have_no_king_call() {
    for contract in [SoloTri, SoloDve, SoloEna] {
        for declarer in [A, B, C, D] {
            let mut game = game(&one_king_each());
            declare(&mut game, declarer, contract);
            // Straight to the talon.
            assert_eq!(game.phase(), Phase::TalonPick);
            for call in calls() {
                assert!(!game.legal_actions().contains(call));
            }
            assert_eq!(game.public().called_suit(), None);
            assert_eq!(game.public().called_king(), None);
            // Alone, and everyone knows it.
            assert_eq!(game.full().partner(), None);
            assert_eq!(game.public().partner(), PublicPartner::Alone);

            finish(&mut game);
            assert_eq!(game.public().called_suit(), None);
            assert_eq!(game.full().partner(), None);
        }
    }
}

#[test]
fn c14_partner_holds_the_called_king_and_stays_hidden_until_it_is_played() {
    // A plays tri and calls each of the three kings held by the others.
    for (suit, partner) in [(Suit::Karo, B), (Suit::Pik, C), (Suit::Kriz, D)] {
        let mut game = game(&one_king_each());
        declare(&mut game, A, Tri);
        // Before the call nobody is a partner.
        assert_eq!(game.full().partner(), None);
        assert_eq!(game.public().partner(), PublicPartner::NotKnown);

        act(&mut game, A, Action::CallKing(suit));
        let king = kralj(suit);
        assert_eq!(holder_of(&game, king), Some(partner));
        // The full view knows the partner; the public state does not.
        assert_eq!(game.full().partner(), Some(partner));
        assert_eq!(game.public().partner(), PublicPartner::NotKnown);
        // The partner sees the king in the own hand.
        assert!(game.private(partner).hand().contains(king));

        // Still hidden through the talon, the discards and the play, up to
        // the moment the king is played.
        let mut checks_in_play = 0;
        loop {
            run_until(&mut game, |g| {
                g.phase() == Phase::Play && g.legal_actions().contains(Action::Play(king))
            });
            assert_eq!(
                game.phase(),
                Phase::Play,
                "the king is played at some point"
            );
            assert_eq!(game.public().partner(), PublicPartner::NotKnown);
            assert_eq!(game.full().partner(), Some(partner));
            checks_in_play += 1;
            // The lowest legal card is played; when that is the king, stop.
            let next = game.legal_actions().iter().next().unwrap();
            if next == Action::Play(king) {
                break;
            }
            game.apply(next).unwrap();
        }
        assert!(checks_in_play >= 1);
        act(&mut game, partner, Action::Play(king));
        assert_eq!(game.public().partner(), PublicPartner::With(partner));
        finish(&mut game);
        assert_eq!(game.public().partner(), PublicPartner::With(partner));
        assert_eq!(game.full().partner(), Some(partner));
    }
}

#[test]
fn c15_declarer_with_the_called_king_plays_alone() {
    let mut game = game(&one_king_each());
    declare(&mut game, A, Tri);
    act(&mut game, A, Action::CallKing(Suit::Srce));
    assert_eq!(holder_of(&game, kralj(Suit::Srce)), Some(A));

    // No partner: the three others are all opponents.
    assert_eq!(game.full().partner(), None);
    assert!(game.full().on_declarer_side(A));
    for seat in [B, C, D] {
        assert!(!game.full().on_declarer_side(seat));
    }
    // The others cannot know it yet.
    assert_eq!(game.public().partner(), PublicPartner::NotKnown);

    // They know once the declarer plays the called king.
    run_until(&mut game, |g| {
        g.phase() == Phase::Play
            && g.to_act() == Some(A)
            && g.legal_actions().contains(Action::Play(kralj(Suit::Srce)))
    });
    assert_eq!(game.public().partner(), PublicPartner::NotKnown);
    act(&mut game, A, Action::Play(kralj(Suit::Srce)));
    assert_eq!(game.public().partner(), PublicPartner::Alone);

    finish(&mut game);
    let score = game.score().expect("the deal is over");
    // Alone against three: the three write no game entry.
    for seat in [B, C, D] {
        assert_eq!(score.parts(seat).game, 0);
        assert_eq!(score.parts(seat).difference, 0);
    }
    assert_ne!(score.parts(A).game, 0);
}

#[test]
fn c16_declarer_with_all_four_kings_calls_an_own_king_and_plays_alone() {
    let four_kings: Vec<Card> = SUITS.into_iter().map(kralj).collect();
    let deal = deal_with(
        [&four_kings, &[], &[], &[]],
        &[tarok(2), tarok(3), tarok(4), tarok(5), tarok(6), tarok(7)],
    );
    for contract in [Tri, Dve, Ena] {
        for suit in SUITS {
            let mut game = game(&deal);
            declare(&mut game, A, contract);
            // The four suits are the only choices, and each king is A's own.
            assert_eq!(legal(&game), calls());
            for king in &four_kings {
                assert!(game.private(A).hand().contains(*king));
            }
            act(&mut game, A, Action::CallKing(suit));
            assert_eq!(game.full().partner(), None);
            for seat in [B, C, D] {
                assert!(!game.full().on_declarer_side(seat));
            }
            finish(&mut game);
            assert_eq!(game.full().partner(), None);
            let score = game.score().unwrap();
            for seat in [B, C, D] {
                assert_eq!(score.parts(seat).game, 0);
            }
        }
    }
}
