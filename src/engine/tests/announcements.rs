//! Spec 009, criteria 1-10 and 17: the round of announcements.
//!
//! Seats: A is forehand (seat 0), then B, C, D; D deals. Most tests use
//! `common::flow::round_deal`: A declares, B holds the pagat, C holds the
//! srce kralj unless the test moves it.

mod common;

use common::flow::{
    A, B, C, D, NORMAL, act, deal_with, declare, exchange_back_open, exchange_open, game,
    king_in_talon_deal, levels, may_by_seat, packet, pass_round, round_deal, round_game, say,
    seeded_game,
};
use common::second_scorer::assert_scores_agree;
use common::written::{deal_1_partner, deal_2_solo, deal_3_king_in_talon};
use common::{kralj, pip, tarok};
use tarok_engine::Contract::{Dve, Ena, SoloDve, SoloEna, SoloTri, Tri};
use tarok_engine::{
    Action, ActionSet, Bonus, Card, Game, GameError, Phase, PublicPartner, Seat, Suit, Target,
    trick,
};

fn may(game: &Game, action: Action) -> bool {
    game.legal_actions().contains(action)
}

fn announce(bonus: Bonus) -> Action {
    Action::Announce(bonus)
}

/// The five announcements that are legal now, as flags in the order trula,
/// kralji, kralj ultimo, pagat ultimo, valat.
fn open_bonuses(game: &Game) -> [bool; 5] {
    Bonus::ALL.map(|bonus| may(game, announce(bonus)))
}

#[test]
fn c01_after_the_last_discard_the_declarer_opens_the_round() {
    // Forehand declares each of the six normal contracts.
    for contract in NORMAL {
        let mut game = game(&round_deal(C));
        declare(&mut game, A, contract);
        if game.phase() == Phase::KingCall {
            act(&mut game, A, Action::CallKing(Suit::Srce));
        }
        let cards = game.public().packet(packet(0)).unwrap().to_vec();
        act(&mut game, A, Action::TakePacket(packet(0)));
        for card in cards {
            // Not before the last discard.
            assert_eq!(game.phase(), Phase::Discard, "{contract}");
            act(&mut game, A, Action::Discard(card));
        }
        assert_eq!(game.phase(), Phase::Announcements, "{contract}");
        // No card is led yet.
        assert_eq!(game.public().leader(), None);
        assert!(game.public().current_trick().is_empty());
        // The declarer has the first turn, then the seats in order of play.
        for seat in [A, B, C, D] {
            assert_eq!(game.phase(), Phase::Announcements, "{contract}");
            assert_eq!(game.to_act(), Some(seat), "{contract}");
            act(&mut game, seat, Action::Done);
        }
        assert_eq!(game.phase(), Phase::Play, "{contract}");
    }

    // Another declarer: C wins the auction with dve and names the contract.
    // The first turn is the declarer's, not forehand's.
    for contract in [Dve, Ena, SoloTri, SoloDve, SoloEna] {
        let mut game = game(&round_deal(C));
        declare(&mut game, C, contract);
        if game.phase() == Phase::KingCall {
            act(&mut game, C, Action::CallKing(Suit::Pik));
        }
        exchange_back_open(&mut game, 0);
        for seat in [C, D, A, B] {
            assert_eq!(game.phase(), Phase::Announcements, "{contract}");
            assert_eq!(game.to_act(), Some(seat), "{contract}");
            act(&mut game, seat, Action::Done);
        }
        assert_eq!(game.phase(), Phase::Play, "{contract}");
    }
}

#[test]
fn c02_a_turn_is_any_number_of_actions_and_then_done() {
    let mut game = round_game(C, Tri);
    // "Done" is legal before anything is said, and it is slot 88.
    assert!(may(&game, Action::Done));
    assert_eq!(Action::Done.slot(), 88);

    // Two announcements in one turn, one per action. The turn stays with A.
    act(&mut game, A, announce(Bonus::Trula));
    assert_eq!(game.to_act(), Some(A));
    assert!(may(&game, Action::Done));
    assert_eq!(game.public().announced(Bonus::Trula), Some(A));
    assert_eq!(game.public().announced(Bonus::Valat), None);
    act(&mut game, A, announce(Bonus::Valat));
    assert_eq!(game.to_act(), Some(A));
    assert!(may(&game, Action::Done));
    assert_eq!(game.public().announced(Bonus::Valat), Some(A));
    // "Done" ends the turn.
    act(&mut game, A, Action::Done);
    assert_eq!(game.to_act(), Some(B));
    assert_eq!(game.phase(), Phase::Announcements);

    // B: a kontra and an announcement in one turn.
    assert!(may(&game, Action::Done));
    act(&mut game, B, Action::Kontra(Target::Game));
    assert_eq!(game.to_act(), Some(B));
    act(&mut game, B, announce(Bonus::Kralji));
    assert_eq!(game.to_act(), Some(B));
    assert!(may(&game, Action::Done));
    act(&mut game, B, Action::Done);
    assert_eq!(game.to_act(), Some(C));

    // A turn of "done" alone is a pass: nothing is said and the turn moves.
    let before = (
        Bonus::ALL.map(|bonus| game.public().announced(bonus)),
        levels(&game),
    );
    act(&mut game, C, Action::Done);
    let after = (
        Bonus::ALL.map(|bonus| game.public().announced(bonus)),
        levels(&game),
    );
    assert_eq!(before, after);
    assert_eq!(game.to_act(), Some(D));
    // "Done" is legal at every decision of the round, to its end.
    while game.phase() == Phase::Announcements {
        assert!(may(&game, Action::Done));
        game.apply(Action::Done).unwrap();
    }
    // And not after it.
    assert!(!may(&game, Action::Done));
}

#[test]
fn c03_the_round_ends_after_three_passes_once_every_seat_had_a_turn() {
    // Four passes at the start end it: not three.
    let mut game = round_game(C, Tri);
    for seat in [A, B, C] {
        say(&mut game, seat, &[]);
        assert_eq!(
            game.phase(),
            Phase::Announcements,
            "after the pass of {seat:?}"
        );
    }
    say(&mut game, D, &[]);
    assert_eq!(game.phase(), Phase::Play);

    // The declarer speaks, three passes follow: over after four turns.
    let mut game = round_game(C, Tri);
    say(&mut game, A, &[announce(Bonus::Trula)]);
    say(&mut game, B, &[]);
    say(&mut game, C, &[]);
    assert_eq!(game.phase(), Phase::Announcements);
    say(&mut game, D, &[]);
    assert_eq!(game.phase(), Phase::Play);

    // The second seat speaks: two passes are not enough, a third one (by
    // the declarer, in its second turn) ends the round.
    let mut game = round_game(C, Tri);
    say(&mut game, A, &[]);
    say(&mut game, B, &[announce(Bonus::Trula)]);
    say(&mut game, C, &[]);
    say(&mut game, D, &[]);
    assert_eq!(game.phase(), Phase::Announcements);
    assert_eq!(game.to_act(), Some(A));
    say(&mut game, A, &[]);
    assert_eq!(game.phase(), Phase::Play);

    // Three passes, then the last seat speaks: three more passes.
    let mut game = round_game(C, Tri);
    say(&mut game, A, &[]);
    say(&mut game, B, &[]);
    say(&mut game, C, &[]);
    say(&mut game, D, &[announce(Bonus::Kralji)]);
    say(&mut game, A, &[]);
    say(&mut game, B, &[]);
    assert_eq!(game.phase(), Phase::Announcements);
    say(&mut game, C, &[]);
    assert_eq!(game.phase(), Phase::Play);

    // A turn with an action starts the count again, every time.
    let mut game = round_game(C, Tri);
    say(&mut game, A, &[]);
    say(&mut game, B, &[]);
    say(&mut game, C, &[announce(Bonus::Trula)]);
    say(&mut game, D, &[]);
    say(&mut game, A, &[]);
    say(&mut game, B, &[announce(Bonus::Kralji)]);
    say(&mut game, C, &[]);
    say(&mut game, D, &[]);
    assert_eq!(game.phase(), Phase::Announcements);
    // A kontra counts as an action too.
    say(&mut game, A, &[]);
    assert_eq!(game.phase(), Phase::Play);

    let mut game = round_game(C, SoloTri);
    say(&mut game, A, &[]);
    say(&mut game, B, &[Action::Kontra(Target::Game)]);
    say(&mut game, C, &[]);
    say(&mut game, D, &[]);
    assert_eq!(game.phase(), Phase::Announcements);
    say(&mut game, A, &[Action::Kontra(Target::Game)]);
    say(&mut game, B, &[]);
    say(&mut game, C, &[]);
    assert_eq!(game.phase(), Phase::Announcements);
    say(&mut game, D, &[]);
    assert_eq!(game.phase(), Phase::Play);
}

#[test]
fn c04_forehand_leads_the_first_trick_also_after_an_announced_valat() {
    // C declares; A is forehand.
    for with_valat in [false, true] {
        let mut game = game(&round_deal(C));
        declare(&mut game, C, SoloTri);
        exchange_back_open(&mut game, 0);
        if with_valat {
            say(&mut game, C, &[announce(Bonus::Valat)]);
        }
        pass_round(&mut game);
        assert_eq!(game.phase(), Phase::Play);
        assert_eq!(game.public().forehand(), A);
        assert_eq!(game.to_act(), Some(A), "valat announced: {with_valat}");
        assert_eq!(game.public().leader(), Some(A));
        // Forehand is on the lead: any card of its hand is legal.
        assert_eq!(
            game.legal_actions(),
            ActionSet::plays(game.private(A).hand())
        );
    }
    // An opponent announces valat: forehand still leads.
    let mut game = game(&round_deal(C));
    declare(&mut game, C, SoloTri);
    exchange_back_open(&mut game, 0);
    say(&mut game, C, &[]);
    say(&mut game, D, &[announce(Bonus::Valat)]);
    pass_round(&mut game);
    assert_eq!(game.to_act(), Some(A));
    assert_eq!(game.public().leader(), Some(A));
}

#[test]
fn c05_a_deal_in_which_all_pass_plays_and_scores_as_in_flow_a() {
    // 300 seeded deals in which every seat passes in the round.
    for seed in 0..300u64 {
        let game = seeded_game(seed);
        let actions: Vec<Action> = game.full().actions().collect();
        // The round is four "done", right after the last discard and
        // before the first card.
        let dones: Vec<usize> = (0..actions.len())
            .filter(|&index| actions[index] == Action::Done)
            .collect();
        assert_eq!(dones.len(), 4, "seed {seed}");
        assert_eq!(dones[3], dones[0] + 3, "seed {seed}");
        assert!(
            matches!(actions[dones[0] - 1], Action::Discard(_)),
            "seed {seed}"
        );
        assert!(
            matches!(actions[dones[3] + 1], Action::Play(_)),
            "seed {seed}"
        );
        assert_eq!(actions.len() - dones[3] - 1, 48, "seed {seed}");
        assert!(
            actions
                .iter()
                .all(|action| !matches!(action, Action::Announce(_) | Action::Kontra(_))),
            "seed {seed}"
        );
        // Nothing was said.
        for bonus in Bonus::ALL {
            assert_eq!(game.public().announced(bonus), None, "seed {seed}");
        }
        assert_eq!(levels(&game), [0; 6], "seed {seed}");
        // The score is the one of flow A: the second scorer of the tests
        // agrees on every part of every seat.
        assert_scores_agree(&game);
    }
    // The three written deals of spec 005 give their entries with nothing
    // changed but the four passes.
    let (game, _) = deal_1_partner();
    assert_eq!(game.score().unwrap().entries(), [35, 0, 35, 0]);
    let (game, _) = deal_2_solo();
    assert_eq!(game.score().unwrap().entries(), [0, -35, 0, 0]);
    let (game, _) = deal_3_king_in_talon();
    assert_eq!(game.score().unwrap().entries(), [0, -20, 0, 50]);
    // Each of them has the four passes in it.
    for game in [
        deal_1_partner().0,
        deal_2_solo().0,
        deal_3_king_in_talon().0,
    ] {
        let dones = game
            .full()
            .actions()
            .filter(|&action| action == Action::Done)
            .count();
        assert_eq!(dones, 4);
    }
}

#[test]
fn c06_trula_kralji_and_valat_may_be_announced_by_every_seat() {
    for bonus in [Bonus::Trula, Bonus::Kralji, Bonus::Valat] {
        // A partner game with the partner hidden, the declarer alone with
        // its own king, and a solo.
        assert_eq!(may_by_seat(&round_game(C, Tri), announce(bonus)), [true; 4]);
        assert_eq!(may_by_seat(&round_game(A, Tri), announce(bonus)), [true; 4]);
        assert_eq!(
            may_by_seat(&round_game(C, SoloEna), announce(bonus)),
            [true; 4]
        );
    }
    // Each for its own side: an opponent announces, and the bonus is its.
    let mut game = round_game(C, Tri);
    say(&mut game, A, &[]);
    say(&mut game, B, &[announce(Bonus::Trula)]);
    say(&mut game, C, &[announce(Bonus::Kralji)]);
    say(&mut game, D, &[announce(Bonus::Valat)]);
    assert_eq!(game.public().announced(Bonus::Trula), Some(B));
    assert_eq!(game.public().announced(Bonus::Kralji), Some(C));
    assert_eq!(game.public().announced(Bonus::Valat), Some(D));
}

#[test]
fn c07_kralj_ultimo_only_by_the_holder_of_the_called_king() {
    let kralj_ultimo = announce(Bonus::KraljUltimo);
    // The partner holds the called king: the partner only.
    for holder in [B, C, D] {
        let mut expected = [false; 4];
        expected[holder.index()] = true;
        for contract in [Tri, Dve, Ena] {
            assert_eq!(
                may_by_seat(&round_game(holder, contract), kralj_ultimo),
                expected,
                "the king with {holder:?} in {contract}"
            );
        }
    }
    // The declarer called its own king: the declarer may.
    assert_eq!(
        may_by_seat(&round_game(A, Tri), kralj_ultimo),
        [true, false, false, false]
    );

    // The called king lies in the talon. Talon: srce kralj, karo 4, karo 3 |
    // mond, karo 2, karo 1. A takes the king's packet: A holds the king and
    // may announce.
    let mut game = game(&king_in_talon_deal(false));
    declare(&mut game, A, Tri);
    act(&mut game, A, Action::CallKing(Suit::Srce));
    exchange_open(
        &mut game,
        0,
        &[pip(Suit::Karo, 4), pip(Suit::Karo, 3), tarok(2)],
    );
    assert!(game.private(A).hand().contains(kralj(Suit::Srce)));
    assert_eq!(
        may_by_seat(&game, kralj_ultimo),
        [true, false, false, false]
    );
    // A takes the other packet: the king stays in the talon and nobody may.
    let mut game = common::flow::game(&king_in_talon_deal(false));
    declare(&mut game, A, Tri);
    act(&mut game, A, Action::CallKing(Suit::Srce));
    exchange_open(
        &mut game,
        1,
        &[pip(Suit::Karo, 2), pip(Suit::Karo, 1), tarok(2)],
    );
    assert_eq!(may_by_seat(&game, kralj_ultimo), [false; 4]);

    // Never in a solo contract, whoever holds a king.
    for contract in [SoloTri, SoloDve, SoloEna] {
        for holder in [A, C] {
            assert_eq!(
                may_by_seat(&round_game(holder, contract), kralj_ultimo),
                [false; 4],
                "{contract}"
            );
        }
    }
}

#[test]
fn c08_pagat_ultimo_only_by_the_holder_of_the_pagat() {
    let pagat_ultimo = announce(Bonus::PagatUltimo);
    // B holds the pagat, in a partner game and in a solo.
    for contract in [Tri, SoloTri] {
        let game = round_game(C, contract);
        assert!(game.private(B).hand().contains(Card::PAGAT));
        assert_eq!(
            may_by_seat(&game, pagat_ultimo),
            [false, true, false, false],
            "{contract}"
        );
    }

    // The pagat lies in the second packet of the talon.
    let talon = [
        pip(Suit::Srce, 4),
        pip(Suit::Srce, 3),
        pip(Suit::Srce, 2),
        Card::PAGAT,
        pip(Suit::Karo, 4),
        pip(Suit::Karo, 3),
    ];
    let deal = deal_with(
        [&[Card::SKIS, Card::MOND], &[], &[kralj(Suit::Srce)], &[]],
        &talon,
    );
    // A takes the first packet: the pagat is in no hand, nobody may.
    let mut game = common::flow::game(&deal);
    declare(&mut game, A, Tri);
    act(&mut game, A, Action::CallKing(Suit::Srce));
    exchange_back_open(&mut game, 0);
    assert_eq!(may_by_seat(&game, pagat_ultimo), [false; 4]);
    // A takes the pagat's packet: A holds it and may.
    let mut game = common::flow::game(&deal);
    declare(&mut game, A, Tri);
    act(&mut game, A, Action::CallKing(Suit::Srce));
    exchange_open(
        &mut game,
        1,
        &[pip(Suit::Karo, 4), pip(Suit::Karo, 3), tarok(2)],
    );
    assert!(game.private(A).hand().contains(Card::PAGAT));
    assert_eq!(
        may_by_seat(&game, pagat_ultimo),
        [true, false, false, false]
    );
}

#[test]
fn c09_each_bonus_can_be_announced_once_in_a_deal() {
    let mut game = round_game(C, Tri);
    assert_eq!(open_bonuses(&game), [true, true, false, false, true]);
    act(&mut game, A, announce(Bonus::Trula));
    // Not again by the same seat, in the same turn.
    assert_eq!(open_bonuses(&game), [false, true, false, false, true]);
    let before = game.clone();
    assert_eq!(
        game.apply(announce(Bonus::Trula)),
        Err(GameError::NotLegal(announce(Bonus::Trula)))
    );
    assert_eq!(game, before, "a refused action changes nothing");
    act(&mut game, A, Action::Done);

    // Not by the other side either (D141, B13-1 a). B holds the pagat.
    assert_eq!(open_bonuses(&game), [false, true, false, true, true]);
    say(
        &mut game,
        B,
        &[announce(Bonus::Kralji), announce(Bonus::PagatUltimo)],
    );
    // C holds the called king.
    assert_eq!(open_bonuses(&game), [false, false, true, false, true]);
    say(
        &mut game,
        C,
        &[announce(Bonus::KraljUltimo), announce(Bonus::Valat)],
    );
    // All five are said: none is legal for any seat, in any later turn.
    for seat in [D, A, B] {
        assert_eq!(game.to_act(), Some(seat));
        assert_eq!(open_bonuses(&game), [false; 5], "{seat:?}");
        for bonus in Bonus::ALL {
            assert!(game.apply(announce(bonus)).is_err());
        }
        // Each seat says something else, so the round goes on.
        let kontra = Action::Kontra(if seat == A {
            Target::Kralji
        } else {
            Target::Game
        });
        if may(&game, kontra) {
            say(&mut game, seat, &[kontra]);
        } else {
            say(&mut game, seat, &[]);
        }
    }
    let said = Bonus::ALL.map(|bonus| game.public().announced(bonus));
    assert_eq!(said, [Some(A), Some(B), Some(C), Some(B), Some(C)]);
}

#[test]
fn c10_an_announcement_does_not_show_the_side_except_kralj_ultimo() {
    // C is the hidden partner, B and D are opponents.
    let mut game = round_game(C, Tri);
    assert_eq!(game.full().partner(), Some(C));
    assert_eq!(game.public().partner(), PublicPartner::NotKnown);
    say(&mut game, A, &[announce(Bonus::Valat)]);
    // An opponent announces: nothing is shown.
    say(&mut game, B, &[announce(Bonus::PagatUltimo)]);
    assert_eq!(game.public().partner(), PublicPartner::NotKnown);
    // The partner announces trula: nothing is shown.
    act(&mut game, C, announce(Bonus::Trula));
    assert_eq!(game.public().partner(), PublicPartner::NotKnown);
    act(&mut game, C, announce(Bonus::Kralji));
    assert_eq!(game.public().partner(), PublicPartner::NotKnown);
    // Kralj ultimo by C: C is the partner, from this action on.
    act(&mut game, C, announce(Bonus::KraljUltimo));
    assert_eq!(game.public().partner(), PublicPartner::With(C));
    act(&mut game, C, Action::Done);
    assert_eq!(game.public().partner(), PublicPartner::With(C));
    pass_round(&mut game);
    assert_eq!(game.public().partner(), PublicPartner::With(C));
    // The king has not been played.
    assert!(!game.public().called_king_played());

    // The same with each other seat as the partner.
    for holder in [B, D] {
        let mut game = round_game(holder, Tri);
        while game.to_act() != Some(holder) {
            game.apply(Action::Done).unwrap();
        }
        assert_eq!(game.public().partner(), PublicPartner::NotKnown);
        act(&mut game, holder, announce(Bonus::KraljUltimo));
        assert_eq!(game.public().partner(), PublicPartner::With(holder));
    }

    // The declarer announces kralj ultimo with its own king (D147): only
    // the holder of the called king may announce it, so the declarer
    // plays alone and all know it, from that action on.
    let mut game = round_game(A, Tri);
    assert_eq!(game.full().partner(), None);
    assert_eq!(game.public().partner(), PublicPartner::NotKnown);
    // Another announcement of the declarer shows nothing.
    act(&mut game, A, announce(Bonus::Trula));
    assert_eq!(game.public().partner(), PublicPartner::NotKnown);
    act(&mut game, A, announce(Bonus::Valat));
    assert_eq!(game.public().partner(), PublicPartner::NotKnown);
    act(&mut game, A, announce(Bonus::KraljUltimo));
    assert_eq!(game.public().partner(), PublicPartner::Alone);
    act(&mut game, A, Action::Done);
    assert_eq!(game.public().partner(), PublicPartner::Alone);
    pass_round(&mut game);
    assert_eq!(game.public().partner(), PublicPartner::Alone);
    // The king has not been played, and it does not lie in the talon.
    assert!(!game.public().called_king_played());
    assert!(!game.public().king_in_talon());

    // Without the announcement the same declarer stays hidden: the other
    // seats cannot tell whether it has a partner.
    let mut game = round_game(A, Tri);
    say(&mut game, A, &[announce(Bonus::Trula)]);
    pass_round(&mut game);
    assert_eq!(game.public().partner(), PublicPartner::NotKnown);
}

/// Criterion 17 (D141, B13-3 b): no duty to keep an announced ultimo card.
#[test]
fn c17_an_announced_ultimo_card_may_be_played_when_the_rules_of_play_allow() {
    // Two games on the same cards: in the first B announces pagat ultimo
    // and C kralj ultimo, in the second all pass. Then the same cards are
    // played in both: A leads the tarok II, B plays the pagat to the first
    // trick and C, who has no tarok, the called king.
    let mut said = round_game(C, Tri);
    say(&mut said, A, &[]);
    say(&mut said, B, &[announce(Bonus::PagatUltimo)]);
    say(&mut said, C, &[announce(Bonus::KraljUltimo)]);
    pass_round(&mut said);
    let mut silent = round_game(C, Tri);
    pass_round(&mut silent);

    let script = [tarok(2), Card::PAGAT, kralj(Suit::Srce)];
    let mut played = 0;
    while said.phase() == Phase::Play {
        assert_eq!(silent.phase(), Phase::Play);
        let seat: Seat = said.to_act().unwrap();
        assert_eq!(silent.to_act(), Some(seat));
        // The legal cards are those of spec 002, in both games: follow the
        // suit, else a tarok, else any card.
        let led = said.public().current_trick().first().copied();
        let by_the_rules = ActionSet::plays(trick::legal_cards(said.private(seat).hand(), led));
        assert_eq!(said.legal_actions(), by_the_rules, "card {played}");
        assert_eq!(silent.legal_actions(), by_the_rules, "card {played}");
        let action = match script.get(played) {
            Some(&card) => Action::Play(card),
            None => said.legal_actions().iter().next().unwrap(),
        };
        // The announced cards are accepted in the first trick.
        assert_eq!(said.apply(action), Ok(()), "card {played}: {action:?}");
        assert_eq!(silent.apply(action), Ok(()));
        played += 1;
    }
    assert_eq!(played, 48);
    let first = said.public().tricks()[0];
    assert_eq!(first.cards[..3], script);
    assert_eq!(said.phase(), Phase::Over);
    assert_eq!(silent.phase(), Phase::Over);
}
