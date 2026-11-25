//! Spec 005, criterion 45: three whole deals written out by hand. Spec
//! 009, criterion 33: the same deals with announcements and kontra.
//!
//! The deals, the plays and the paper arithmetic are in
//! `tests/common/written.rs`.

mod common;

use common::flow::{A, B, C, D, with_round};
use common::second_scorer::assert_scores_agree;
use common::written::{Expected, deal_1_partner, deal_2_solo, deal_3_king_in_talon};
use tarok_engine::Contract::{Dve, Ena, SoloTri, Tri};
use tarok_engine::{Action, Bonus, Card, Game, Phase, PublicPartner, Seat, Suit, Target};

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

/// Spec 009, criterion 33: the three written deals played again with
/// something said in the round of announcements. The cards and the plays
/// are those of `tests/common/written.rs`; the entries are worked out here
/// by hand from the counts written there.
#[test]
fn c33_three_written_deals_with_announcements_and_kontra() {
    let trula = Action::Announce(Bonus::Trula);
    let kralji = Action::Announce(Bonus::Kralji);
    let valat = Action::Announce(Bonus::Valat);

    // Deal 1, an announced bonus under kontra. A plays tri with C and wins
    // with 52: 10 + 15. A announces trula and makes it; B, an opponent,
    // says kontra on it: 20 x 2 = 40. A and C write 10 + 15 + 40 = 65.
    let (game, expected) = with_round(
        &[(A, &[trula]), (B, &[Action::Kontra(Target::Trula)])],
        deal_1_partner,
    );
    // The same tricks and the same counts as in the silent deal.
    let winners: Vec<Seat> = game.public().tricks().iter().map(|t| t.winner).collect();
    assert_eq!(winners, expected.winners.to_vec());
    assert_eq!(game.public().announced(Bonus::Trula), Some(A));
    assert_eq!(game.public().kontra_level(Target::Trula), 1);
    assert_scores_agree(&game);
    let score = game.score().unwrap();
    assert_eq!(score.declarer_points(), 52);
    let a = score.parts(A);
    assert_eq!((a.game, a.difference, a.trula), (10, 15, 40));
    assert_eq!((a.kralji, a.kralj_ultimo, a.pagat_ultimo), (0, 0, 0));
    assert_eq!((a.valat, a.mondfang), (0, 0));
    assert_eq!(score.parts(C), a);
    assert_eq!(score.entries(), [65, 0, 65, 0]);

    // Deal 2, an announced valat that fails. B plays solo tri alone and
    // announces valat. B loses with 31 points and has the silent trula;
    // none of that is scored: -500.
    let (game, expected) = with_round(&[(B, &[valat])], deal_2_solo);
    let winners: Vec<Seat> = game.public().tricks().iter().map(|t| t.winner).collect();
    assert_eq!(winners, expected.winners.to_vec());
    assert_eq!(game.public().announced(Bonus::Valat), Some(B));
    assert_scores_agree(&game);
    let score = game.score().unwrap();
    assert_eq!(score.declarer_points(), 31);
    let b = score.parts(B);
    assert_eq!((b.game, b.difference, b.trula, b.kralji), (0, 0, 0, 0));
    assert_eq!((b.kralj_ultimo, b.pagat_ultimo, b.mondfang), (0, 0, 0));
    assert_eq!(b.valat, -500);
    assert_eq!(score.entries(), [0, -500, 0, 0]);

    // Deal 3, an announcement by the opponents. D plays dve alone (the
    // called king lay in the talon) and wins with 54: 20 + 20. A, an
    // opponent, announces kralji; D has the four kings, so the opponents
    // lose 20 and D writes +20 in place of the silent 10. C says kontra on
    // the game: (20 + 20) x 2 = 80. D writes 80 + 20 = 100. B's mond was
    // captured: -20, not multiplied.
    let (game, expected) = with_round(
        &[(A, &[kralji]), (C, &[Action::Kontra(Target::Game)])],
        deal_3_king_in_talon,
    );
    let winners: Vec<Seat> = game.public().tricks().iter().map(|t| t.winner).collect();
    assert_eq!(winners, expected.winners.to_vec());
    assert_eq!(game.public().announced(Bonus::Kralji), Some(A));
    assert_eq!(game.public().kontra_level(Target::Game), 1);
    assert_scores_agree(&game);
    let score = game.score().unwrap();
    assert_eq!(score.declarer_points(), 54);
    let d = score.parts(D);
    assert_eq!((d.game, d.difference, d.trula, d.kralji), (40, 40, 0, 20));
    assert_eq!(
        (d.kralj_ultimo, d.pagat_ultimo, d.valat, d.mondfang),
        (0, 0, 0, 0)
    );
    assert_eq!(score.parts(B).mondfang, -20);
    assert_eq!(score.entries(), [0, -20, 0, 100]);
}
