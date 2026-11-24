//! Spec 005, "In scope": what each seat may know. No numbered criterion.
//!
//! The debug text of a view is a handy stand-in for "everything the view
//! shows". These tests change hidden cards and check that the text of the
//! public view, and of the private view of a seat that is not involved,
//! stays the same.

mod common;

use common::flow::{A, B, C, D, DEALER, MAX_ACTIONS, act, deal_with, declare, packet};
use common::{kralj, pip, tarok};
use tarok_engine::{Action, Card, CardSet, Contract, Deal, Game, Phase, Seat, Suit};

fn public_text(game: &Game) -> String {
    format!("{:?}", game.public())
}

fn private_text(game: &Game, seat: Seat) -> String {
    format!("{:?}", game.private(seat))
}

/// The deal with one card of B and one card of C changed over: the highest
/// card each of them holds in a suit of which both hold two or more.
fn swapped_between_b_and_c(deal: &Deal) -> Option<(Deal, Card, Card)> {
    let of_suit = |seat: Seat, suit: Suit| deal.hand(seat) & CardSet::suit(suit);
    let suit = common::SUITS
        .into_iter()
        .find(|&suit| of_suit(B, suit).len() >= 2 && of_suit(C, suit).len() >= 2)?;
    let from_b = of_suit(B, suit).iter().last()?;
    let from_c = of_suit(C, suit).iter().last()?;
    Some((changed_over(deal, from_b, from_c), from_b, from_c))
}

/// The deal with B's card `from_b` and C's card `from_c` changed over.
fn changed_over(deal: &Deal, from_b: Card, from_c: Card) -> Deal {
    let mut hands = deal.hands();
    assert!(hands[B.index()].remove(from_b), "B holds {from_b}");
    assert!(hands[C.index()].remove(from_c), "C holds {from_c}");
    hands[B.index()].insert(from_c);
    hands[C.index()].insert(from_b);
    Deal::new(hands, deal.talon()).expect("still a whole deal")
}

#[test]
fn public_view_does_not_change_when_hidden_cards_change_hands() {
    let mut deals_compared = 0;
    let mut steps_compared = 0;
    for seed in 0..40u64 {
        let deal = Deal::from_seed(seed);
        let Some((other_deal, from_b, from_c)) = swapped_between_b_and_c(&deal) else {
            continue;
        };
        deals_compared += 1;
        let mut first = Game::new(&deal, DEALER);
        let mut second = Game::new(&other_deal, DEALER);
        assert_ne!(first, second, "the two deals differ in hidden cards");

        for _ in 0..MAX_ACTIONS {
            // Nothing public differs, and nothing A or D can see.
            assert_eq!(public_text(&first), public_text(&second), "seed {seed}");
            for seat in [A, D] {
                assert_eq!(private_text(&first, seat), private_text(&second, seat));
            }
            // B and C do see their own cards.
            assert_ne!(private_text(&first, B), private_text(&second, B));
            assert_ne!(private_text(&first, C), private_text(&second, C));
            steps_compared += 1;

            // The same action in both games: the lowest one that is legal
            // in both and does not play one of the two changed cards. When
            // there is none, the comparison of this deal ends.
            let common_action = first.legal_actions().iter().find(|&action| {
                second.legal_actions().contains(action)
                    && action != Action::Play(from_b)
                    && action != Action::Play(from_c)
            });
            let Some(action) = common_action else {
                break;
            };
            first.apply(action).unwrap();
            second.apply(action).unwrap();
            if first.phase() == Phase::Over {
                break;
            }
        }
    }
    assert!(deals_compared >= 20, "{deals_compared} deals");
    // The comparison goes deep into the play, not only through the bidding.
    assert!(
        steps_compared >= deals_compared * 30,
        "{steps_compared} steps"
    );
}

#[test]
fn hidden_discards_and_the_unshown_talon_do_not_show_in_the_public_view() {
    // A holds two low srce. The talon holds no king and no trula card.
    let talon = [
        tarok(10),
        pip(Suit::Karo, 4),
        tarok(11),
        tarok(12),
        tarok(13),
        tarok(14),
    ];
    let deal = deal_with(
        [&[pip(Suit::Srce, 4), pip(Suit::Srce, 3)], &[], &[], &[]],
        &talon,
    );

    // The talon before it is shown: the same deal with the first talon card
    // and a card of C changed over looks the same to all through the
    // bidding and the contract choice, and to A, B and D in private.
    let from_c = deal.hand(C).iter().last().unwrap();
    let mut hands = deal.hands();
    hands[C.index()].remove(from_c);
    hands[C.index()].insert(talon[0]);
    let mut other_talon = talon;
    other_talon[0] = from_c;
    let other_deal = Deal::new(hands, other_talon).unwrap();
    let mut first = Game::new(&deal, DEALER);
    let mut second = Game::new(&other_deal, DEALER);
    for (seat, action) in [
        (B, Action::Pass),
        (C, Action::Pass),
        (D, Action::Pass),
        (A, Action::Bid(Contract::SoloTri)),
    ] {
        assert_eq!(public_text(&first), public_text(&second));
        for uninvolved in [A, B, D] {
            assert_eq!(
                private_text(&first, uninvolved),
                private_text(&second, uninvolved)
            );
        }
        act(&mut first, seat, action);
        act(&mut second, seat, action);
    }
    assert_eq!(first.phase(), Phase::ContractChoice);
    assert_eq!(public_text(&first), public_text(&second));
    // Once the talon is turned up the difference is public.
    act(&mut first, A, Action::Bid(Contract::SoloTri));
    act(&mut second, A, Action::Bid(Contract::SoloTri));
    assert_eq!(first.phase(), Phase::TalonPick);
    assert_ne!(public_text(&first), public_text(&second));

    // Hidden discards: A takes the first packet (X, karo 4, XI) and puts
    // down the karo 4 and the X, then the srce 4 in one game and the
    // srce 3 in the other. The public view shows the X and "three
    // discards" in both, and nothing else.
    let exchanged = |third: Card| {
        let mut game = Game::new(&deal, DEALER);
        declare(&mut game, A, Contract::SoloTri);
        act(&mut game, A, Action::TakePacket(packet(0)));
        for card in [pip(Suit::Karo, 4), tarok(10), third] {
            act(&mut game, A, Action::Discard(card));
        }
        assert_eq!(game.phase(), Phase::Announcements);
        common::flow::pass_round(&mut game);
        assert_eq!(game.phase(), Phase::Play);
        game
    };
    let first = exchanged(pip(Suit::Srce, 4));
    let second = exchanged(pip(Suit::Srce, 3));
    assert_ne!(first, second);
    assert_eq!(public_text(&first), public_text(&second));
    for seat in [B, C, D] {
        assert_eq!(private_text(&first, seat), private_text(&second, seat));
    }
    // The declarer sees the own discards.
    assert_ne!(private_text(&first, A), private_text(&second, A));
    // A shown tarok discard does change the public view.
    let third = exchanged(tarok(11));
    assert_ne!(public_text(&first), public_text(&third));
}

#[test]
fn public_view_shows_the_partner_only_when_the_called_king_is_played() {
    // The pik kralj is with B in one deal and with C in the other; the two
    // deals are the same otherwise (B and C change the pik kralj and the
    // pik dama over). A plays tri and calls pik.
    let deal = deal_with(
        [
            &[],
            &[kralj(Suit::Pik), pip(Suit::Pik, 7)],
            &[
                common::court(Suit::Pik, tarok_engine::Rank::Dama),
                pip(Suit::Pik, 8),
            ],
            &[],
        ],
        &[tarok(2), tarok(3), tarok(4), tarok(5), tarok(6), tarok(7)],
    );
    let from_b = kralj(Suit::Pik);
    let from_c = common::court(Suit::Pik, tarok_engine::Rank::Dama);
    let other_deal = changed_over(&deal, from_b, from_c);

    let start = |deal: &Deal| {
        let mut game = Game::new(deal, DEALER);
        declare(&mut game, A, Contract::Tri);
        act(&mut game, A, Action::CallKing(Suit::Pik));
        game
    };
    let mut first = start(&deal);
    let mut second = start(&other_deal);
    // The true partner differs; the public view does not show it.
    assert_eq!(first.full().partner(), Some(B));
    assert_eq!(second.full().partner(), Some(C));
    let mut steps = 0;
    loop {
        assert_eq!(public_text(&first), public_text(&second));
        assert_eq!(private_text(&first, A), private_text(&second, A));
        steps += 1;
        let common_action = first.legal_actions().iter().find(|&action| {
            second.legal_actions().contains(action)
                && action != Action::Play(from_b)
                && action != Action::Play(from_c)
        });
        let Some(action) = common_action else {
            break;
        };
        first.apply(action).unwrap();
        second.apply(action).unwrap();
        assert!(steps < MAX_ACTIONS && first.phase() != Phase::Over);
    }
    // The comparison ran through the exchange into the play.
    assert_eq!(first.phase(), Phase::Play);
    assert!(steps > 5);
}
