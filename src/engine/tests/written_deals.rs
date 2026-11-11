//! Spec 005, criterion 45: three whole deals written out by hand.
//!
//! The deals, the plays and the paper arithmetic are in
//! `tests/common/written.rs`.

mod common;

use common::flow::{A, B, C, D};
use common::written::{Expected, deal_1_partner, deal_2_solo, deal_3_king_in_talon};
use tarok_engine::Contract::{Dve, Ena, SoloTri, Tri};
use tarok_engine::{Card, Game, Phase, PublicPartner, Seat, Suit};

/// The checks every written deal shares.
fn check(game: &Game, expected: &Expected) {
    assert_eq!(game.phase(), Phase::Over);
    assert_eq!(game.to_act(), None);
    assert!(game.legal_actions().is_empty());

    let winners: Vec<Seat> = game.public().tricks().iter().map(|t| t.winner).collect();
    assert_eq!(winners, expected.winners.to_vec(), "trick winners");

    let score = game.score().expect("the deal is over");
    assert_eq!(score.declarer_points(), expected.declarer_points);
    assert_eq!(score.opponents_points(), expected.opponents_points);
    assert_eq!(score.entries(), expected.entries, "entries of A, B, C, D");
}

#[test]
fn c45_written_deal_1_partner_game_tri() {
    let (game, expected) = deal_1_partner();
    check(&game, &expected);

    let public = game.public();
    assert_eq!(public.declarer(), Some(A));
    assert_eq!(public.contract(), Some(Tri));
    assert_eq!(public.called_suit(), Some(Suit::Kriz));
    assert_eq!(game.full().partner(), Some(C));
    assert_eq!(public.partner(), PublicPartner::With(C));

    // Tri won with 52: 10 + 15, and the silent trula.
    let score = game.score().unwrap();
    let a = score.parts(A);
    assert_eq!(
        (a.game, a.difference, a.trula, a.kralji, a.kralj_ultimo),
        (10, 15, 10, 0, 0)
    );
    assert_eq!((a.pagat_ultimo, a.valat, a.mondfang), (0, 0, 0));
    assert_eq!(score.parts(C), a);
    assert_eq!(score.parts(B).entry(), 0);
    assert_eq!(score.parts(D).entry(), 0);
}

#[test]
fn c45_written_deal_2_solo_tri_lost() {
    let (game, expected) = deal_2_solo();
    check(&game, &expected);

    let public = game.public();
    assert_eq!(public.declarer(), Some(B));
    assert_eq!(public.winning_bid(), Some(Ena));
    assert_eq!(public.contract(), Some(SoloTri));
    assert_eq!(public.called_suit(), None);
    assert_eq!(game.full().partner(), None);
    assert_eq!(public.partner(), PublicPartner::Alone);
    // Six bids were made: B, C, D, A, B, D.
    assert_eq!(public.bids().len(), 6);

    // Solo tri lost with 31: -40 - 5, and the silent trula for B.
    let score = game.score().unwrap();
    let b = score.parts(B);
    assert_eq!(
        (b.game, b.difference, b.trula, b.kralji, b.kralj_ultimo),
        (-40, -5, 10, 0, 0)
    );
    assert_eq!((b.pagat_ultimo, b.valat, b.mondfang), (0, 0, 0));
    for seat in [A, C, D] {
        assert_eq!(score.parts(seat).entry(), 0);
    }
}

#[test]
fn c45_written_deal_3_called_king_in_the_talon() {
    let (game, expected) = deal_3_king_in_talon();
    check(&game, &expected);

    let public = game.public();
    assert_eq!(public.declarer(), Some(D));
    assert_eq!(public.contract(), Some(Dve));
    assert_eq!(public.called_suit(), Some(Suit::Pik));
    assert!(public.king_in_talon());
    assert_eq!(game.full().partner(), None);
    assert_eq!(public.partner(), PublicPartner::Alone);
    // D took the king's packet and won the fourth trick with the king: the
    // four other talon cards are D's.
    assert!(game.full().talon_rest_won());
    assert_eq!(game.full().declarer_pile().len(), 40 + 2 + 4);
    assert_eq!(game.full().opponents_pile().len(), 8);
    assert!(game.full().declarer_pile().contains(Card::MOND));

    // Dve won with 54: 20 + 20, the silent kralji, and B's mondfang.
    let score = game.score().unwrap();
    let d = score.parts(D);
    assert_eq!(
        (d.game, d.difference, d.trula, d.kralji, d.kralj_ultimo),
        (20, 20, 0, 10, 0)
    );
    assert_eq!((d.pagat_ultimo, d.valat, d.mondfang), (0, 0, 0));
    let b = score.parts(B);
    assert_eq!((b.game, b.difference, b.kralji, b.mondfang), (0, 0, 0, -20));
    assert_eq!(score.parts(A).entry(), 0);
    assert_eq!(score.parts(C).entry(), 0);
}
