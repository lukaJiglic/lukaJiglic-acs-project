//! Spec 005, criteria 26-28: the card play inside a whole deal.
//!
//! Seats: A is forehand (seat 0), then B, C, D; D (seat 3) deals.

mod common;

use common::flow::{A, B, C, D, act, deal_with, declare, game, play_random, run_until};
use common::{kralj, run_cases, seat, tarok};
use proptest::prelude::*;
use tarok_engine::Contract::{Dve, SoloTri, Tri};
use tarok_engine::{
    Action, Card, CardSet, Deal, Game, Phase, PublicPartner, Seat, Suit, action, trick,
};

#[test]
fn c26_forehand_leads_the_first_trick_whoever_declares() {
    for dealer in Seat::ALL {
        let forehand = dealer.next();
        for declarer in Seat::ALL {
            let mut game = Game::new(&Deal::from_seed(5), dealer);
            assert_eq!(game.public().forehand(), forehand);
            let contract = if declarer == forehand { Tri } else { SoloTri };
            declare(&mut game, declarer, contract);
            run_until(&mut game, |g| g.phase() == Phase::Play);

            // Forehand is on the lead and may play any card of the hand.
            assert_eq!(game.phase(), Phase::Play);
            assert_eq!(game.public().leader(), Some(forehand));
            assert_eq!(game.to_act(), Some(forehand));
            assert!(game.public().current_trick().is_empty());
            let hand = game.private(forehand).hand();
            assert_eq!(hand.len(), 12);
            let plays: Vec<Action> = hand.iter().map(Action::Play).collect();
            assert_eq!(game.legal_actions().iter().collect::<Vec<_>>(), plays);

            // The first card on the table is forehand's.
            let card = hand.iter().next().unwrap();
            act(&mut game, forehand, Action::Play(card));
            assert_eq!(game.public().current_trick(), &[card]);
            assert_eq!(game.to_act(), Some(forehand.next()));
        }
    }
}

#[test]
fn c27_legal_cards_and_trick_winners_are_those_of_spec_002() {
    let case = (
        any::<u64>(),
        0..4u8,
        proptest::collection::vec(any::<u8>(), 96),
    );
    run_cases(2_000, case, |(seed, dealer, choices)| {
        let mut plays_checked = 0;
        let game = play_random(seed, seat(dealer), &choices, |game| {
            if game.phase() != Phase::Play {
                return;
            }
            // The legal plays are `trick::legal_cards` for the hand of the
            // player to move and the led card.
            let to_act = game.to_act().unwrap();
            let hand = game.private(to_act).hand();
            let led = game.public().current_trick().first().copied();
            let expected: Vec<Action> = trick::legal_cards(hand, led)
                .iter()
                .map(Action::Play)
                .collect();
            assert_eq!(game.legal_actions().iter().collect::<Vec<_>>(), expected);
            plays_checked += 1;
        });
        prop_assert_eq!(plays_checked, 48);

        // Every trick went to the seat `trick::winner` names, and the
        // winner led the next one. Forehand led the first.
        let public = game.public();
        prop_assert_eq!(public.tricks().len(), 12);
        let mut leader = public.forehand();
        for done in public.tricks() {
            prop_assert_eq!(done.leader, leader);
            let winner = done.leader.plus(trick::winner(&done.cards));
            prop_assert_eq!(done.winner, winner);
            leader = winner;
        }
        // The cards a seat won are its tricks.
        for seat in Seat::ALL {
            let won: CardSet = public
                .tricks()
                .iter()
                .filter(|t| t.winner == seat)
                .flat_map(|t| t.cards)
                .collect();
            prop_assert_eq!(public.tricks_won(seat), won);
        }
        Ok(())
    });
}

#[test]
fn c28_called_king_played_makes_the_partner_public() {
    // A plays tri and calls karo. B holds the karo kralj and the karo 4;
    // C and D hold a karo each.
    let karo = |n| common::pip(Suit::Karo, n);
    let deal = deal_with(
        [
            &[karo(1)],
            &[kralj(Suit::Karo), karo(4)],
            &[karo(3)],
            &[karo(2)],
        ],
        &[tarok(2), tarok(3), tarok(4), tarok(5), tarok(6), tarok(7)],
    );
    let mut game = game(&deal);
    declare(&mut game, A, Tri);
    act(&mut game, A, Action::CallKing(Suit::Karo));
    run_until(&mut game, |g| g.phase() == Phase::Play);
    assert_eq!(game.full().partner(), Some(B));

    // A leads the karo 1. Nothing is public yet.
    act(&mut game, A, Action::Play(karo(1)));
    assert_eq!(game.public().partner(), PublicPartner::NotKnown);
    // B plays the called king: public from that moment, in mid-trick.
    act(&mut game, B, Action::Play(kralj(Suit::Karo)));
    assert_eq!(game.public().partner(), PublicPartner::With(B));
    assert_eq!(game.public().current_trick().len(), 2);
    act(&mut game, C, Action::Play(karo(3)));
    act(&mut game, D, Action::Play(karo(2)));
    assert_eq!(game.public().partner(), PublicPartner::With(B));

    // The same deal with B as the declarer in dve calling pik: the partner
    // stays unknown as long as the pik kralj is not played.
    let mut other = common::flow::game(&deal);
    declare(&mut other, B, Dve);
    act(&mut other, B, Action::CallKing(Suit::Pik));
    let partner = other
        .full()
        .partner()
        .expect("a partner holds the pik kralj");
    let king = kralj(Suit::Pik);
    let mut actions = 0;
    while other.phase() != Phase::Over {
        actions += 1;
        assert!(
            actions <= common::flow::MAX_ACTIONS,
            "the deal does not end"
        );
        let played: CardSet = other
            .public()
            .tricks()
            .iter()
            .flat_map(|t| t.cards)
            .chain(other.public().current_trick().iter().copied())
            .collect();
        let expected = if played.contains(king) {
            PublicPartner::With(partner)
        } else {
            PublicPartner::NotKnown
        };
        assert_eq!(other.public().partner(), expected);
        let next = other.legal_actions().iter().next().unwrap();
        other.apply(next).unwrap();
    }
    assert_eq!(other.public().partner(), PublicPartner::With(partner));
    // The play slots are 90 + card index.
    assert_eq!(Action::Play(Card::PAGAT).slot(), action::slots::PLAYS.start);
}
