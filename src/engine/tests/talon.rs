//! Spec 005, criteria 17-25: the talon exchange and the called king in the
//! talon.
//!
//! Seats: A is forehand (seat 0), then B, C, D; D (seat 3) deals.

mod common;

use common::flow::{
    A, B, C, D, NORMAL, REST_WITHOUT_KING, act, deal_with, declare, exchange, finish, game,
    king_in_talon_deal, king_taken_and_led, legal, packet, run_until,
};
use common::second_scorer::second_score;
use common::{SUITS, kralj, pip, set, tarok};
use tarok_engine::Contract::{Dve, SoloDve, SoloTri, Tri};
use tarok_engine::{
    Action, Card, CardSet, Contract, Deal, Game, GameError, Phase, PublicPartner, Rank, Suit,
};

/// No king and no trula card in the talon; A holds the škis, the mond, the
/// pagat, the srce kralj and the srce 4 (and taroks II-VIII).
fn plain_deal() -> Deal {
    deal_with(
        [
            &[
                Card::SKIS,
                Card::MOND,
                Card::PAGAT,
                kralj(Suit::Srce),
                pip(Suit::Srce, 4),
            ],
            &[kralj(Suit::Karo)],
            &[kralj(Suit::Pik)],
            &[kralj(Suit::Kriz)],
        ],
        &TALON,
    )
}

const fn t(number: u8) -> Card {
    Card::tarok(number).unwrap()
}

/// The talon of `plain_deal`, in the order it lies.
const TALON: [Card; 6] = [
    t(10),
    Card::pip(Suit::Karo, 4).unwrap(),
    t(11),
    Card::pip(Suit::Pik, 7).unwrap(),
    Card::court(Suit::Kriz, Rank::Fant),
    t(12),
];

fn packet_size(contract: Contract) -> usize {
    match contract {
        Tri | SoloTri => 3,
        Dve | SoloDve => 2,
        _ => 1,
    }
}

/// A is the declarer; a king is called where the contract asks for one.
fn to_talon(deal: &Deal, contract: Contract, call: Suit) -> Game {
    let mut game = game(deal);
    declare(&mut game, A, contract);
    if game.phase() == Phase::KingCall {
        act(&mut game, A, Action::CallKing(call));
    }
    assert_eq!(game.phase(), Phase::TalonPick);
    game
}

#[test]
fn c17_talon_is_shown_in_packets_by_contract() {
    for contract in NORMAL {
        let mut game = game(&plain_deal());
        // Hidden during the bidding and the contract choice.
        assert_eq!(game.public().talon(), None);
        declare(&mut game, A, contract);
        if game.phase() == Phase::KingCall {
            assert_eq!(game.public().talon(), None, "hidden until the call");
            act(&mut game, A, Action::CallKing(Suit::Karo));
        }

        // Public now, in the order it lies.
        assert_eq!(game.phase(), Phase::TalonPick);
        assert_eq!(game.to_act(), Some(A));
        assert_eq!(game.public().talon(), Some(TALON));

        // Two packets of three, three of two, six of one.
        let size = packet_size(contract);
        assert_eq!(game.public().packet_size(), Some(size));
        let count = 6 / size;
        let takes: Vec<Action> = (0..count as u8)
            .map(|i| Action::TakePacket(packet(i)))
            .collect();
        assert_eq!(legal(&game), takes);
        // Packets follow the talon's order.
        for index in 0..6u8 {
            let expected = (usize::from(index) < count)
                .then(|| &TALON[usize::from(index) * size..(usize::from(index) + 1) * size]);
            assert_eq!(game.public().packet(packet(index)), expected);
        }
        // A packet that does not exist is refused.
        if count < 6 {
            let beyond = Action::TakePacket(packet(count as u8));
            let before = game.clone();
            assert_eq!(game.apply(beyond), Err(GameError::NotLegal(beyond)));
            assert_eq!(game, before);
        }
    }
}

#[test]
fn c18_declarer_takes_one_packet_into_the_hand() {
    for contract in NORMAL {
        let size = packet_size(contract);
        for index in 0..(6 / size) as u8 {
            let mut game = to_talon(&plain_deal(), contract, Suit::Karo);
            let hand_before = game.private(A).hand();
            assert_eq!(hand_before.len(), 12);
            let taken = set(&TALON[usize::from(index) * size..(usize::from(index) + 1) * size]);

            act(&mut game, A, Action::TakePacket(packet(index)));
            assert_eq!(game.private(A).hand(), hand_before | taken);
            assert_eq!(game.private(A).hand().len() as usize, 12 + size);
            assert_eq!(game.public().hand_size(A) as usize, 12 + size);
            // Which packet was taken is public; the other hands are as dealt.
            assert_eq!(game.public().taken_packet(), Some(packet(index)));
            assert_eq!(game.public().taken_cards(), taken);
            for seat in [B, C, D] {
                assert_eq!(game.private(seat).hand().len(), 12);
            }
            // The declarer now discards.
            assert_eq!(game.phase(), Phase::Discard);
            assert_eq!(game.to_act(), Some(A));
            // A second packet is not on offer.
            assert!(legal(&game).iter().all(|a| matches!(a, Action::Discard(_))));
        }
    }
}

#[test]
fn c19_declarer_discards_as_many_cards_one_by_one_never_kings_or_trula() {
    let never: CardSet = SUITS
        .into_iter()
        .map(kralj)
        .chain([Card::SKIS, Card::MOND, Card::PAGAT])
        .collect();

    for contract in NORMAL {
        let size = packet_size(contract);
        let mut game = to_talon(&plain_deal(), contract, Suit::Karo);
        act(&mut game, A, Action::TakePacket(packet(0)));

        for made in 0..size {
            assert_eq!(game.phase(), Phase::Discard);
            assert_eq!(game.to_act(), Some(A));
            assert_eq!(game.public().discards_made(), made);
            let hand = game.private(A).hand();
            assert_eq!(hand.len() as usize, 12 + size - made);

            // Legal: every card of the hand except kings and trula. A holds
            // the trula and the srce kralj.
            let expected: Vec<Action> = hand
                .iter()
                .filter(|&card| !never.contains(card))
                .map(Action::Discard)
                .collect();
            assert_eq!(legal(&game), expected);
            assert_eq!(expected.len(), hand.len() as usize - 4);
            for card in [Card::SKIS, Card::MOND, Card::PAGAT, kralj(Suit::Srce)] {
                assert!(hand.contains(card));
                let refused = Action::Discard(card);
                let before = game.clone();
                assert_eq!(game.apply(refused), Err(GameError::NotLegal(refused)));
                assert_eq!(game, before);
            }
            // A card of another seat is refused too.
            let foreign = Action::Discard(kralj(Suit::Karo));
            assert_eq!(game.apply(foreign), Err(GameError::NotLegal(foreign)));

            // One card per action: the highest legal one here.
            let Action::Discard(card) = *expected.last().unwrap() else {
                unreachable!()
            };
            act(&mut game, A, Action::Discard(card));
            assert!(!game.private(A).hand().contains(card));
            assert!(game.private(A).discards().contains(card));
        }

        // As many discards as cards taken: back to twelve, then the play.
        assert_eq!(game.public().discards_made(), size);
        assert_eq!(game.private(A).hand().len(), 12);
        assert_eq!(game.private(A).discards().len() as usize, size);
        assert_eq!(game.phase(), Phase::Play);
    }
}

#[test]
fn c20_discarded_taroks_are_public_other_discards_hidden() {
    // A plays tri, takes the first packet (X, karo 4, XI) and discards the
    // tarok X, the karo 4 and a srce from the hand.
    let mut game = to_talon(&plain_deal(), Tri, Suit::Karo);
    let hidden_card = game
        .private(A)
        .hand()
        .iter()
        .find(|c| c.suit() == Some(Suit::Srce) && *c != kralj(Suit::Srce))
        .expect("A holds a low srce");
    // Taroks other than the trula are legal discards (S14 a).
    act(&mut game, A, Action::TakePacket(packet(0)));
    for number in [2, 10, 11] {
        let discard = Action::Discard(tarok(number));
        assert_eq!(
            game.legal_actions().contains(discard),
            game.private(A).hand().contains(tarok(number))
        );
    }
    assert!(game.legal_actions().contains(Action::Discard(tarok(10))));

    act(&mut game, A, Action::Discard(tarok(10)));
    assert_eq!(game.public().shown_discards(), set(&[tarok(10)]));
    act(&mut game, A, Action::Discard(pip(Suit::Karo, 4)));
    act(&mut game, A, Action::Discard(hidden_card));
    assert_eq!(game.phase(), Phase::Play);

    // Everyone sees the tarok and the number of discards, nothing else.
    assert_eq!(game.public().shown_discards(), set(&[tarok(10)]));
    assert_eq!(game.public().discards_made(), 3);
    // The declarer sees all three. The other seats see none of their own.
    let all = set(&[tarok(10), pip(Suit::Karo, 4), hidden_card]);
    assert_eq!(game.private(A).discards(), all);
    for seat in [B, C, D] {
        assert_eq!(game.private(seat).discards(), CardSet::EMPTY);
    }
    // The full view (tests, critic) has them.
    assert_eq!(game.full().discards(), all);

    // It stays that way to the end of the deal.
    finish(&mut game);
    assert_eq!(game.public().shown_discards(), set(&[tarok(10)]));
    assert_eq!(game.full().discards(), all);
}

#[test]
fn c21_discards_count_for_declarer_untaken_packets_for_opponents() {
    let mut valats = 0;
    for contract in NORMAL {
        let size = packet_size(contract);
        let last = (6 / size - 1) as u8;
        let mut game = to_talon(&plain_deal(), contract, Suit::Karo);
        // Take the last packet; discard the three lowest legal cards.
        act(&mut game, A, Action::TakePacket(packet(last)));
        while game.phase() == Phase::Discard {
            let lowest = legal(&game)[0];
            act(&mut game, A, lowest);
        }
        let discards = game.full().discards();
        assert_eq!(discards.len() as usize, size);
        let taken = game.public().taken_cards();
        let untaken = set(&TALON) - taken;
        assert_eq!(untaken.len() as usize, 6 - size);

        // Until the last trick the packets not taken lie in the opponents'
        // pile, in every contract.
        run_until(&mut game, |game| game.public().tricks().len() == 11);
        assert!(untaken.is_subset(game.full().opponents_pile()));

        finish(&mut game);
        let full = game.full();
        // One exception (spec 013, criterion 16, D133): the declarer's
        // side took all twelve tricks. Then the packets are the
        // declarer's. With the lowest legal cards that happens here in
        // some of the six contracts and not in others.
        let side = |seat| seat == A || Some(seat) == full.partner();
        let valat = game
            .public()
            .tricks()
            .iter()
            .all(|trick| side(trick.winner));
        valats += usize::from(valat);
        // The discards lie in the declarer's pile, the packets not taken in
        // the opponents' pile.
        assert!(discards.is_subset(full.declarer_pile()));
        if valat {
            assert!(untaken.is_subset(full.declarer_pile()));
        } else {
            assert!(untaken.is_subset(full.opponents_pile()));
        }
        assert!(full.declarer_pile().is_disjoint(full.opponents_pile()));
        assert_eq!(full.declarer_pile() | full.opponents_pile(), CardSet::FULL);

        // And they are counted there. The second scorer of the tests
        // collects each pile on its own and counts each by itself.
        let second = second_score(&game);
        assert!(discards.is_subset(second.declarer_pile));
        if valat {
            assert!(untaken.is_subset(second.declarer_pile));
        } else {
            assert!(untaken.is_subset(second.opponents_pile));
        }
        let score = game.score().unwrap();
        assert_eq!(score.declarer_points(), second.declarer_points);
        assert_eq!(score.opponents_points(), second.opponents_points);
        assert_eq!(second.declarer_points + second.opponents_points, 70);
    }
    println!("criterion 21: {valats} of the 6 contracts ended in a valat of the declarer's side");
    assert!(
        valats < 6,
        "the rule itself is tested in a contract or more"
    );
}

#[test]
fn c22_called_king_in_the_talon_declarer_alone_and_public() {
    let mut game = game(&king_in_talon_deal(false));
    declare(&mut game, A, Tri);
    assert_eq!(game.public().partner(), PublicPartner::NotKnown);
    assert!(!game.public().king_in_talon());

    act(&mut game, A, Action::CallKing(Suit::Srce));
    // The talon is shown and the king is in it: no partner, and public.
    assert!(game.public().talon().unwrap().contains(&kralj(Suit::Srce)));
    assert!(game.public().king_in_talon());
    assert_eq!(game.public().partner(), PublicPartner::Alone);
    assert_eq!(game.full().partner(), None);
    for seat in [B, C, D] {
        assert!(!game.full().on_declarer_side(seat));
    }

    // Another called suit in the same deal: a partner, and not public.
    let mut other = to_talon(&king_in_talon_deal(false), Tri, Suit::Pik);
    assert!(!other.public().king_in_talon());
    assert_eq!(other.public().partner(), PublicPartner::NotKnown);
    assert!(other.full().partner().is_some());
    finish(&mut other);

    finish(&mut game);
    assert_eq!(game.public().partner(), PublicPartner::Alone);
    assert_eq!(game.full().partner(), None);
}

#[test]
fn c23_king_packet_taken_king_wins_its_trick_rest_of_talon_to_declarer() {
    let game = king_taken_and_led(false);
    // Everyone followed srce: the king won the first trick.
    let first = game.public().tricks()[0];
    assert_eq!(first.cards[0], kralj(Suit::Srce));
    assert_eq!(first.winner, A);
    assert!(first.cards.iter().all(|c| c.suit() == Some(Suit::Srce)));

    // The rest of the talon is in the declarer's pile and counts there.
    let full = game.full();
    assert!(full.talon_rest_won());
    assert!(set(&REST_WITHOUT_KING).is_subset(full.declarer_pile()));
    assert!(set(&REST_WITHOUT_KING).is_disjoint(full.opponents_pile()));
    // Counted pile by pile by the second scorer of the tests: with the
    // rest of the talon on the declarer's side.
    let second = second_score(&game);
    assert!(set(&REST_WITHOUT_KING).is_subset(second.declarer_pile));
    let score = game.score().unwrap();
    assert_eq!(score.declarer_points(), second.declarer_points);
    assert_eq!(score.opponents_points(), second.opponents_points);
    assert_eq!(second.declarer_points + second.opponents_points, 70);
    // The three cards are worth counting: the mond is one of them.
    let without_rest = second.declarer_pile - set(&REST_WITHOUT_KING);
    assert!(tarok_engine::points::count(without_rest) < second.declarer_points);
}

#[test]
fn c24_king_packet_taken_king_loses_its_trick_rest_of_talon_to_opponents() {
    let game = king_taken_and_led(true);
    // B holds no srce and trumped the king.
    let first = game.public().tricks()[0];
    assert_eq!(first.cards[0], kralj(Suit::Srce));
    assert_eq!(first.winner, B);
    assert!(first.cards[1].is_tarok());

    let full = game.full();
    assert!(!full.talon_rest_won());
    assert!(set(&REST_WITHOUT_KING).is_subset(full.opponents_pile()));
    assert!(set(&REST_WITHOUT_KING).is_disjoint(full.declarer_pile()));
    // Counted pile by pile by the second scorer of the tests: with the
    // rest of the talon on the opponents' side.
    let second = second_score(&game);
    assert!(set(&REST_WITHOUT_KING).is_subset(second.opponents_pile));
    let score = game.score().unwrap();
    assert_eq!(score.declarer_points(), second.declarer_points);
    assert_eq!(score.opponents_points(), second.opponents_points);
    assert_eq!(second.declarer_points + second.opponents_points, 70);
}

#[test]
fn c25_king_packet_not_taken_rest_of_talon_stays_with_opponents() {
    // A takes the second packet (mond, karo 2, karo 1). The king's packet
    // stays down.
    let mut game = to_talon(&king_in_talon_deal(false), Tri, Suit::Srce);
    exchange(
        &mut game,
        1,
        &[pip(Suit::Karo, 2), pip(Suit::Karo, 1), tarok(2)],
    );
    // Alone all the same, and the king is never played.
    assert_eq!(game.public().partner(), PublicPartner::Alone);
    finish(&mut game);
    let played: CardSet = game
        .public()
        .tricks()
        .iter()
        .flat_map(|trick| trick.cards)
        .collect();
    assert!(!played.contains(kralj(Suit::Srce)));

    let untaken = set(&[kralj(Suit::Srce), pip(Suit::Karo, 4), pip(Suit::Karo, 3)]);
    let full = game.full();
    assert!(!full.talon_rest_won());
    assert!(untaken.is_subset(full.opponents_pile()));
    assert!(untaken.is_disjoint(full.declarer_pile()));
    // Even if the declarer wins every other card, a king is with the
    // opponents: no silent kralji for the declarer.
    assert!(game.score().unwrap().parts(A).kralji <= 0);
}
