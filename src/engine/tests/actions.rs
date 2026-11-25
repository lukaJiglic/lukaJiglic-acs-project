//! Spec 005, "In scope": the fixed mapping of `Action` to the 160 action
//! slots of `docs/encoding.md` section 5. No numbered criterion.

mod common;

use common::flow::{A, B, C, D, deal_with, game, packet};
use common::{SUITS, set};
use tarok_engine::{
    Action, ActionSet, Card, CardSet, Contract, Deal, DealError, Game, GameError, Packet, Phase,
    Seat, action::slots,
};

#[test]
fn action_slots_match_the_encoding_contract() {
    assert_eq!(Action::SLOTS, 160);

    // 0-12: naprej (pass), klop, tri, dve, ena, solo tri, solo dve, solo
    // ena, berač, solo brez, odprti berač, barvni valat, valat.
    let ladder = [
        Contract::Klop,
        Contract::Tri,
        Contract::Dve,
        Contract::Ena,
        Contract::SoloTri,
        Contract::SoloDve,
        Contract::SoloEna,
        Contract::Berac,
        Contract::SoloBrez,
        Contract::OdprtiBerac,
        Contract::BarvniValat,
        Contract::Valat,
    ];
    assert_eq!(Contract::ALL, ladder);
    assert_eq!(Action::Pass.slot(), 0);
    for (index, contract) in ladder.into_iter().enumerate() {
        assert_eq!(Action::Bid(contract).slot(), 1 + index);
    }
    // 13-16: srce, karo, pik, križ.
    for (index, suit) in SUITS.into_iter().enumerate() {
        assert_eq!(Action::CallKing(suit).slot(), 13 + index);
    }
    // 17-22: the packet index.
    for index in 0..6u8 {
        assert_eq!(
            Action::TakePacket(packet(index)).slot(),
            17 + usize::from(index)
        );
    }
    assert_eq!(Packet::new(6), None);
    // 23-76: a discard by card index. 90-143: a play by card index.
    for card in Card::all() {
        assert_eq!(Action::Discard(card).slot(), 23 + usize::from(card.index()));
        assert_eq!(Action::Play(card).slot(), 90 + usize::from(card.index()));
    }

    assert_eq!(slots::BIDS, 0..13);
    assert_eq!(slots::KINGS, 13..17);
    assert_eq!(slots::PACKETS, 17..23);
    assert_eq!(slots::DISCARDS, 23..77);
    assert_eq!(slots::PLAYS, 90..144);
}

#[test]
fn every_slot_is_one_action_or_reserved() {
    let mut actions = 0;
    for slot in 0..160usize {
        match Action::from_slot(slot) {
            Some(action) => {
                assert_eq!(action.slot(), slot);
                actions += 1;
            }
            // 89 (barvni valat) and 144-159 are reserved.
            None => assert!(slot == 89 || (144..160).contains(&slot)),
        }
    }
    // With spec 009: five announcements, six kontra targets and "done".
    assert_eq!(actions, 13 + 4 + 6 + 54 + 5 + 6 + 1 + 54);
    assert_eq!(Action::from_slot(160), None);
    assert_eq!(Action::from_slot(usize::MAX), None);
}

#[test]
fn action_set_holds_actions_in_slot_order() {
    let mut actions = ActionSet::EMPTY;
    assert!(actions.is_empty());
    assert_eq!(actions.len(), 0);
    assert_eq!(actions.iter().next(), None);
    assert_eq!(actions.nth(0), None);

    let listed = [
        Action::Play(Card::SKIS),
        Action::Pass,
        Action::Discard(Card::PAGAT),
        Action::Bid(Contract::Valat),
        Action::Play(Card::from_index(53).unwrap()),
        Action::TakePacket(packet(5)),
    ];
    for action in listed {
        assert!(actions.insert(action));
        assert!(!actions.insert(action), "second insert reports no change");
    }
    assert_eq!(actions.len(), 6);
    assert!(!actions.is_empty());
    let in_order: Vec<usize> = actions.iter().map(Action::slot).collect();
    assert_eq!(in_order, vec![0, 12, 22, 23, 111, 143]);
    for (position, slot) in in_order.iter().enumerate() {
        assert_eq!(actions.nth(position as u32).map(Action::slot), Some(*slot));
        assert!(actions.contains_slot(*slot));
    }
    assert_eq!(actions.nth(6), None);
    assert_eq!(actions.nth(u32::MAX), None);
    assert!(actions.contains(Action::Pass));
    assert!(!actions.contains(Action::Play(Card::PAGAT)));
    assert!(!actions.contains_slot(1));
    assert!(!actions.contains_slot(159));
    assert!(!actions.contains_slot(160));
    assert!(!actions.contains_slot(usize::MAX));

    let collected: ActionSet = listed.into_iter().collect();
    assert_eq!(collected, actions);
}

#[test]
fn deal_new_takes_a_whole_deal_only() {
    let dealt = Deal::from_seed(3);
    let rebuilt = Deal::new(dealt.hands(), dealt.talon()).expect("a whole deal");
    assert_eq!(rebuilt, dealt);

    // A hand of eleven.
    let mut hands = dealt.hands();
    let card = hands[1].iter().next().unwrap();
    hands[1].remove(card);
    assert_eq!(Deal::new(hands, dealt.talon()), Err(DealError::NotWhole));
    // The same card in two hands (and so one card missing).
    hands[1].insert(hands[2].iter().next().unwrap());
    assert_eq!(Deal::new(hands, dealt.talon()), Err(DealError::NotWhole));
    // A talon card that is also in a hand.
    let mut talon = dealt.talon();
    talon[0] = dealt.hands()[0].iter().next().unwrap();
    assert_eq!(Deal::new(dealt.hands(), talon), Err(DealError::NotWhole));
    // The same talon card twice.
    let mut talon = dealt.talon();
    talon[1] = talon[0];
    assert_eq!(Deal::new(dealt.hands(), talon), Err(DealError::NotWhole));
}

#[test]
fn game_starts_from_a_deal_a_seed_or_chosen_cards() {
    for dealer in Seat::ALL {
        let from_seed = Game::from_seed(9, dealer);
        let from_deal = Game::new(&Deal::from_seed(9), dealer);
        assert_eq!(from_seed, from_deal);
        assert_eq!(from_seed.phase(), Phase::Bidding);
        assert_eq!(from_seed.public().dealer(), dealer);
        assert_eq!(from_seed.public().forehand(), dealer.next());
        // Forehand speaks last: the seat after forehand speaks first.
        assert_eq!(from_seed.to_act(), Some(dealer.next().next()));
        // The hands are those of the deal by seat index, whoever deals.
        let deal = Deal::from_seed(9);
        for seat in Seat::ALL {
            assert_eq!(from_seed.private(seat).hand(), deal.hand(seat));
            assert_eq!(from_seed.full().hand(seat), deal.hand(seat));
        }
        assert_eq!(from_seed.full().talon(), deal.talon());
    }

    // Chosen hands, talon and dealer.
    let deal = deal_with([&[Card::SKIS], &[Card::MOND], &[Card::PAGAT], &[]], &[]);
    let game = game(&deal);
    assert!(game.private(A).hand().contains(Card::SKIS));
    assert!(game.private(B).hand().contains(Card::MOND));
    assert!(game.private(C).hand().contains(Card::PAGAT));
    assert_eq!(game.private(D).hand().len(), 12);
    let all: CardSet = Seat::ALL
        .into_iter()
        .map(|s| game.full().hand(s))
        .fold(set(&game.full().talon()), |a, b| a | b);
    assert_eq!(all, CardSet::FULL);
}

#[test]
fn an_illegal_action_returns_an_error_and_changes_nothing() {
    // In every phase of one deal, try every one of the 160 slots.
    let mut game = Game::from_seed(11, D);
    let mut refused = 0;
    let mut actions = 0;
    while game.phase() != Phase::Over {
        actions += 1;
        assert!(
            actions <= common::flow::MAX_ACTIONS,
            "the deal does not end"
        );
        let legal = game.legal_actions();
        for slot in 0..Action::SLOTS {
            let Some(action) = Action::from_slot(slot) else {
                continue;
            };
            if legal.contains(action) {
                continue;
            }
            let before = game.clone();
            assert_eq!(game.apply(action), Err(GameError::NotLegal(action)));
            assert_eq!(game, before, "{action:?} changed the state");
            refused += 1;
        }
        let next = legal.iter().next().unwrap();
        game.apply(next).unwrap();
    }
    assert!(refused > 5_000);

    // After the end every action is refused.
    let before = game.clone();
    for slot in 0..Action::SLOTS {
        if let Some(action) = Action::from_slot(slot) {
            assert_eq!(game.apply(action), Err(GameError::Over));
        }
    }
    assert_eq!(game, before);
}
