//! Spec 006, criteria 2, 3, 4 and 13: the tokens.
//!
//! A token is five numbers: event id, seat relative to the observer (4 for
//! "no seat"), trick number, position in the trick, index in the sequence.

mod common;

use common::encoding::scripted_position;
use common::flow::{A, B, C, D, MAX_ACTIONS, act, deal_with, declare, exchange, game, seeded_game};
use common::{kralj, pip, tarok};
use tarok_engine::encode::{self, observe};
use tarok_engine::{Action, Card, Contract, Game, Phase, Seat, Suit};

// Event ids of docs/encoding.md section 3, written out here.
const BOS: i16 = 1;
const BID: i16 = 2; // + 0 pass, 1 klop, 2 tri, 3 dve, 4 ena, 5 solo tri, 6 solo dve, 7 solo ena
const CONTRACT: i16 = 15; // + the same numbers
const KING: i16 = 28; // + 0 srce, 1 karo, 2 pik, 3 križ
const TALON_SHOWN: i16 = 32; // + card index
const TALON_TAKEN: i16 = 86; // + packet index
const DISCARD_SHOWN: i16 = 92; // + card index
const PLAY: i16 = 159; // + card index
const NO_SEAT: i16 = 4;

#[test]
fn c02_tokens_of_a_scripted_deal_equal_a_hand_written_list() {
    let game = scripted_position();
    // C observes. Seen from C: C is 0, D is 1, A is 2, B is 3.
    let (c, d, a, b) = (0, 1, 2, 3);
    let _ = d;
    // Card indices: tarok n is n - 1; srce 4, 3, 2 are 22, 23, 24; srce
    // kralj is 29; karo 4, 3, 2, 1 are 30, 31, 32, 33; XXI is 20.
    let expected: Vec<[i16; 5]> = vec![
        // event, seat, trick, position, index
        [BOS, NO_SEAT, 0, 0, 0],
        [BID + 3, b, 0, 0, 1],                 // B bids dve
        [BID, c, 0, 0, 2],                     // C passes
        [BID, 1, 0, 0, 3],                     // D passes
        [BID + 3, a, 0, 0, 4],                 // A holds dve
        [BID, b, 0, 0, 5],                     // B passes
        [CONTRACT + 3, a, 0, 0, 6],            // A plays dve
        [KING, a, 0, 0, 7],                    // A calls srce
        [TALON_SHOWN + 29, NO_SEAT, 0, 0, 8],  // srce kralj
        [TALON_SHOWN + 30, NO_SEAT, 0, 0, 9],  // karo 4
        [TALON_SHOWN + 31, NO_SEAT, 0, 0, 10], // karo 3
        [TALON_SHOWN + 20, NO_SEAT, 0, 0, 11], // XXI
        [TALON_SHOWN + 32, NO_SEAT, 0, 0, 12], // karo 2
        [TALON_SHOWN + 33, NO_SEAT, 0, 0, 13], // karo 1
        [TALON_TAKEN, a, 0, 0, 14],            // the first packet
        // The karo 4 was put down hidden: no token. The tarok II is shown.
        [DISCARD_SHOWN + 1, a, 0, 0, 15],
        [PLAY + 29, a, 1, 1, 16], // A srce kralj
        [PLAY + 22, b, 1, 2, 17], // B srce 4
        [PLAY + 23, c, 1, 3, 18], // C srce 3
        [PLAY + 24, 1, 1, 4, 19], // D srce 2
        [PLAY + 11, a, 2, 1, 20], // A XII
        [PLAY + 12, b, 2, 2, 21], // B XIII
    ];
    let observation = observe(&game, C);
    assert_eq!(observation.tokens, expected);

    // The same moment seen by A: only the seat fields differ.
    // Seen from A: A is 0, B is 1, C is 2, D is 3.
    let from_a = observe(&game, A);
    assert_eq!(from_a.tokens.len(), expected.len());
    for (seen, written) in from_a.tokens.iter().zip(&expected) {
        let seat = match written[1] {
            NO_SEAT => NO_SEAT,
            relative_to_c => (relative_to_c + 2) % 4,
        };
        assert_eq!(
            *seen,
            [written[0], seat, written[2], written[3], written[4]]
        );
    }
    assert_eq!(from_a.tokens[1], [BID + 3, 1, 0, 0, 1]);
    assert_eq!(from_a.tokens[19], [PLAY + 24, 3, 1, 4, 19]);

    // The flat form for the arrays: 128 rows of five, zero after the end.
    let mut flat = vec![-1i16; encode::MAX_TOKENS * encode::TOKEN_FIELDS];
    let written = encode::write_tokens(&game, C, &mut flat);
    assert_eq!(written, expected.len());
    for (index, row) in flat.chunks(encode::TOKEN_FIELDS).enumerate() {
        match expected.get(index) {
            Some(token) => assert_eq!(row, token),
            None => assert_eq!(row, [0; 5], "padding at {index}"),
        }
    }

    // The event log behind the tokens is public: one event per token.
    assert_eq!(game.public().events().len(), expected.len());
    let _ = (B, D);
}

/// Review items A2 and A3. The scripted deal names its winning bid and
/// takes the first packet. Here the declarer names a higher contract than
/// it bid and takes another packet: the tokens carry the named contract and
/// the taken packet.
#[test]
fn c02_tokens_carry_the_named_contract_and_the_taken_packet() {
    // B bids dve, A holds it, the others pass: the winning bid is dve.
    let auction = |game: &mut Game| {
        let dve = Action::Bid(Contract::Dve);
        act(game, B, dve);
        act(game, C, Action::Pass);
        act(game, D, Action::Pass);
        act(game, A, dve);
        act(game, B, Action::Pass);
        assert_eq!(game.public().highest_bid(), Some((A, Contract::Dve)));
    };
    let deal = common::flow::king_in_talon_deal(false);
    // Seen from C, the declarer A is seat 2.
    let a = 2;

    // A names ena (bid slot 4), calls pik (suit 2) and takes the fourth of
    // six packets.
    let mut ena = game(&deal);
    auction(&mut ena);
    act(&mut ena, A, Action::Bid(Contract::Ena));
    act(&mut ena, A, Action::CallKing(Suit::Pik));
    act(&mut ena, A, Action::TakePacket(common::flow::packet(3)));
    let tokens = observe(&ena, C).tokens;
    assert_eq!(tokens[6], [CONTRACT + 4, a, 0, 0, 6]);
    assert_eq!(tokens[7], [KING + 2, a, 0, 0, 7]);
    assert_eq!(tokens[14], [TALON_TAKEN + 3, a, 0, 0, 14]);
    assert_eq!(tokens.len(), 15);

    // A names dve (slot 3) and takes the third of three packets.
    let mut dve = game(&deal);
    auction(&mut dve);
    act(&mut dve, A, Action::Bid(Contract::Dve));
    act(&mut dve, A, Action::CallKing(Suit::Kriz));
    act(&mut dve, A, Action::TakePacket(common::flow::packet(2)));
    let tokens = observe(&dve, C).tokens;
    assert_eq!(tokens[6], [CONTRACT + 3, a, 0, 0, 6]);
    assert_eq!(tokens[7], [KING + 3, a, 0, 0, 7]);
    assert_eq!(tokens[14], [TALON_TAKEN + 2, a, 0, 0, 14]);

    // A names solo tri (slot 5): no king is called, the talon follows the
    // contract at once. A takes the second of two packets.
    let mut solo = game(&deal);
    auction(&mut solo);
    act(&mut solo, A, Action::Bid(Contract::SoloTri));
    act(&mut solo, A, Action::TakePacket(common::flow::packet(1)));
    let tokens = observe(&solo, C).tokens;
    assert_eq!(tokens[6], [CONTRACT + 5, a, 0, 0, 6]);
    assert_eq!(
        tokens[7][0],
        TALON_SHOWN + 29,
        "the srce kralj, first talon card"
    );
    assert_eq!(tokens[13], [TALON_TAKEN + 1, a, 0, 0, 13]);
    assert_eq!(tokens.len(), 14);
}

/// Review items A2 and A3 on random deals: every token but the start and
/// the talon cards names the action that was taken, in order. A hidden
/// discard has no token.
#[test]
fn tokens_name_the_actions_that_were_taken() {
    let mut later_packets = 0;
    let mut named_above_the_bid = 0;
    for seed in 0..2_000u64 {
        let done = seeded_game(seed);
        let mut game = Game::from_seed(seed, done.public().dealer());
        let mut expected: Vec<i16> = Vec::new();
        for action in done.full().actions() {
            // Slots of docs/encoding.md section 5: bids 0 to 12, packets 17
            // to 22.
            let slot = action.slot() as i16;
            let id = match (game.phase(), action) {
                (Phase::Bidding, Action::Pass | Action::Bid(_)) => Some(BID + slot),
                (Phase::ContractChoice, Action::Bid(_)) => Some(CONTRACT + slot),
                (Phase::KingCall, Action::CallKing(suit)) => Some(KING + i16::from(suit.index())),
                (Phase::TalonPick, Action::TakePacket(_)) => {
                    later_packets += usize::from(slot > 17);
                    Some(TALON_TAKEN + slot - 17)
                }
                (Phase::Discard, Action::Discard(card)) => card
                    .is_tarok()
                    .then(|| DISCARD_SHOWN + i16::from(card.index())),
                // The four passes of the round of announcements (spec
                // 009): "done" writes no token.
                (Phase::Announcements, Action::Done) => None,
                (Phase::Play, Action::Play(card)) => Some(PLAY + i16::from(card.index())),
                (phase, action) => panic!("seed {seed}: {action:?} in {phase:?}"),
            };
            expected.extend(id);
            game.apply(action).unwrap();
        }
        assert_eq!(game, done);
        let seen: Vec<i16> = observe(&done, A)
            .tokens
            .iter()
            .map(|token| token[0])
            .filter(|id| *id != BOS && !(TALON_SHOWN..TALON_SHOWN + 54).contains(id))
            .collect();
        assert_eq!(seen, expected, "seed {seed}");
        let (_, winning) = done.public().highest_bid().unwrap();
        named_above_the_bid += usize::from(done.public().contract() != Some(winning));
    }
    // The deals do take other packets than the first and do name other
    // contracts than the winning bid.
    assert!(later_packets > 500, "{later_packets}");
    assert!(named_above_the_bid > 200, "{named_above_the_bid}");
}

/// The tokens of the acting seat before every action of a random deal, and
/// the game at the end.
fn tokens_through_a_deal(seed: u64) -> (Vec<Vec<[i16; 5]>>, Game) {
    let mut snapshots = Vec::new();
    let finished = {
        // `seeded_game` plays the deal; replay its actions to look at every
        // step.
        let done = seeded_game(seed);
        let dealer = done.public().dealer();
        let mut game = Game::from_seed(seed, dealer);
        for action in done.full().actions() {
            let seat = game.to_act().unwrap();
            snapshots.push(observe(&game, seat).tokens);
            game.apply(action).unwrap();
        }
        assert_eq!(game, done);
        game
    };
    (snapshots, finished)
}

#[test]
fn c03_trick_and_position_fields() {
    for seed in 0..300u64 {
        let (snapshots, game) = tokens_through_a_deal(seed);
        // The sequence of a seat only grows: an earlier one is a prefix of a
        // later one, whoever looks (the seat field aside).
        let last = observe(&game, A).tokens;
        for earlier in &snapshots {
            assert!(earlier.len() <= last.len());
            for (one, two) in earlier.iter().zip(&last) {
                assert_eq!(
                    [one[0], one[2], one[3], one[4]],
                    [two[0], two[2], two[3], two[4]]
                );
            }
        }

        let first_play = last
            .iter()
            .position(|token| (PLAY..PLAY + 54).contains(&token[0]))
            .expect("cards are played");
        // Before card play: trick 0 and position 0 on every token.
        for token in &last[..first_play] {
            assert_eq!((token[2], token[3]), (0, 0), "seed {seed}: {token:?}");
            assert!(token[0] < PLAY);
        }
        // In card play: 48 tokens, tricks 1 to 12, positions 1 to 4.
        let plays = &last[first_play..];
        assert_eq!(plays.len(), 48, "seed {seed}");
        for (number, token) in plays.iter().enumerate() {
            assert!((PLAY..PLAY + 54).contains(&token[0]));
            assert_eq!(token[2] as usize, number / 4 + 1, "trick of play {number}");
            assert_eq!(
                token[3] as usize,
                number % 4 + 1,
                "position of play {number}"
            );
        }
        // The index field counts from 0.
        for (index, token) in last.iter().enumerate() {
            assert_eq!(token[4] as usize, index);
            assert!((0..=4).contains(&token[1]));
        }
        assert_eq!(last[0], [BOS, NO_SEAT, 0, 0, 0]);
    }
}

#[test]
fn c04_hidden_discards_are_no_tokens_and_a_discarded_tarok_is_one() {
    // A holds two low srce and plays solo tri. The talon: X, karo 4, XI |
    // XII, XIII, XIV.
    let talon = [10, 11, 12, 13, 14].map(tarok);
    let talon = [
        talon[0],
        pip(Suit::Karo, 4),
        talon[1],
        talon[2],
        talon[3],
        talon[4],
    ];
    let deal = deal_with(
        [&[pip(Suit::Srce, 4), pip(Suit::Srce, 3)], &[], &[], &[]],
        &talon,
    );
    let discard_tokens = |game: &Game| -> Vec<[i16; 5]> {
        observe(game, B)
            .tokens
            .into_iter()
            .filter(|token| (DISCARD_SHOWN..DISCARD_SHOWN + 54).contains(&token[0]))
            .collect()
    };

    // Three hidden discards: karo 4, srce 4, srce 3. No token for any.
    let mut game_hidden = game(&deal);
    declare(&mut game_hidden, A, Contract::SoloTri);
    let before = observe(&game_hidden, B).tokens.len();
    exchange(
        &mut game_hidden,
        0,
        &[pip(Suit::Karo, 4), pip(Suit::Srce, 4), pip(Suit::Srce, 3)],
    );
    let after = observe(&game_hidden, B).tokens;
    assert!(discard_tokens(&game_hidden).is_empty());
    // Only the "packet taken" token was added.
    assert_eq!(after.len(), before + 1);
    assert_eq!(after[before], [TALON_TAKEN, 3, 0, 0, before as i16]);
    // The declarer's own tokens show no more than the others'.
    assert_eq!(observe(&game_hidden, A).tokens.len(), after.len());
    // No token names a hidden discard in any way: none carries its card.
    for hidden in [pip(Suit::Karo, 4), pip(Suit::Srce, 4), pip(Suit::Srce, 3)] {
        let index = i16::from(hidden.index());
        for token in &after[before..] {
            assert_ne!(token[0], DISCARD_SHOWN + index);
            assert_ne!(token[0], PLAY + index);
        }
    }

    // One tarok among the discards: X, between two hidden cards. Exactly
    // one "discard shown" token, at its place in the order.
    let mut game_shown = game(&deal);
    declare(&mut game_shown, A, Contract::SoloTri);
    exchange(
        &mut game_shown,
        0,
        &[pip(Suit::Karo, 4), tarok(10), pip(Suit::Srce, 4)],
    );
    let shown = discard_tokens(&game_shown);
    // Seen from B, the declarer A is seat 3.
    assert_eq!(
        shown,
        vec![[DISCARD_SHOWN + 9, 3, 0, 0, (before + 1) as i16]]
    );
    assert_eq!(observe(&game_shown, B).tokens.len(), before + 2);

    // Random deals: as many "discard shown" tokens as taroks put down, and
    // the token count is 1 start + calls + contract + king + 6 talon cards
    // + packet + shown discards + 48 plays.
    let mut with_shown = 0;
    for seed in 0..400u64 {
        let game = seeded_game(seed);
        let tokens = observe(&game, A).tokens;
        let discards = game.full().discards();
        let taroks = discards.iter().filter(|card| card.is_tarok()).count();
        let shown = tokens
            .iter()
            .filter(|token| (DISCARD_SHOWN..DISCARD_SHOWN + 54).contains(&token[0]))
            .count();
        assert_eq!(shown, taroks, "seed {seed}");
        with_shown += usize::from(taroks > 0);
        let king = usize::from(game.public().called_suit().is_some());
        let calls = game.public().bids().len();
        assert_eq!(tokens.len(), 1 + calls + 1 + king + 6 + 1 + taroks + 48);
        for token in &tokens {
            if (DISCARD_SHOWN..DISCARD_SHOWN + 54).contains(&token[0]) {
                let card = Card::from_index((token[0] - DISCARD_SHOWN) as u8).unwrap();
                assert!(card.is_tarok() && discards.contains(card));
            }
        }
    }
    assert!(with_shown >= 50, "{with_shown} deals with a shown discard");
    let _ = (C, D, kralj(Suit::Srce));
}

#[test]
fn c13_no_sequence_exceeds_128_tokens_in_100_000_random_deals() {
    let mut longest = 0;
    let mut longest_seed = 0;
    for seed in 0..100_000u64 {
        let game = seeded_game(seed);
        // One token per public event.
        let tokens = game.public().events().len();
        if tokens > longest {
            longest = tokens;
            longest_seed = seed;
        }
        assert!(tokens <= encode::MAX_TOKENS, "seed {seed}: {tokens} tokens");
    }
    // The encoder agrees with the event count on the longest deal.
    let game = seeded_game(longest_seed);
    assert_eq!(observe(&game, A).tokens.len(), longest);
    println!("longest token sequence in 100,000 random deals: {longest} (seed {longest_seed})");
    assert!(longest >= 60, "the random deals are not short ones only");
}

/// Not a numbered criterion: the longest sequence this slice of the rules
/// can produce at all, by walking every auction and contract choice.
#[test]
fn longest_possible_sequence_of_this_slice() {
    fn walk(game: &Game, longest: &mut (usize, usize, Game), depth: usize) {
        assert!(depth <= MAX_ACTIONS);
        match game.phase() {
            Phase::Bidding | Phase::ContractChoice => {
                for action in game.legal_actions().iter() {
                    let mut next = game.clone();
                    next.apply(action).unwrap();
                    walk(&next, longest, depth + 1);
                }
            }
            _ => {
                // The contract is named. From here: a king token if a king
                // is called, six talon cards, the packet, at most one shown
                // discard per card taken, 48 plays.
                let contract = game.public().contract().unwrap();
                let calls = game.public().bids().len();
                let king = usize::from(contract.calls_king());
                let taken = contract.packet_size().unwrap();
                let events = 1 + calls + 1 + king + 6 + 1 + taken + 48;
                // Actions: the calls, the contract, a king, the packet, the
                // discards, 48 cards.
                let actions = calls + 1 + king + 1 + taken + 48;
                if events > longest.0 {
                    longest.2 = game.clone();
                }
                longest.0 = longest.0.max(events);
                longest.1 = longest.1.max(actions);
            }
        }
    }
    let start = Game::from_seed(1, D);
    let mut longest = (0, 0, start.clone());
    walk(&start, &mut longest, 0);
    let (events, actions, mut game) = longest;
    // 1 start, 13 calls, the contract, 6 talon cards, the packet, 1 shown
    // discard and 48 plays in solo ena after the longest auction.
    assert_eq!(events, 71);
    assert!(events <= encode::MAX_TOKENS);
    // Review item D11: the logs of a game have room for the longest deal.
    // The same deal has the most actions: 13 calls, the contract, the
    // packet, 1 discard and 48 cards.
    assert_eq!(actions, 64);
    assert!(events <= tarok_engine::game::MAX_EVENTS);
    assert!(actions <= tarok_engine::game::MAX_ACTIONS);
    assert_eq!(tarok_engine::game::MAX_EVENTS, encode::MAX_TOKENS);
    // The deal with the most events, played to its end: nothing is cut off.
    common::flow::finish(&mut game);
    assert_eq!(game.phase(), Phase::Over);
    let calls = game.public().bids().len();
    assert_eq!(calls, 13);
    let shown = game
        .full()
        .discards()
        .iter()
        .filter(|c| c.is_tarok())
        .count();
    assert_eq!(
        game.public().events().len(),
        1 + calls + 1 + 6 + 1 + shown + 48
    );
    // The actions: with the four passes of the round of announcements
    // (spec 009).
    assert_eq!(game.full().action_slots().len(), calls + 1 + 1 + 1 + 4 + 48);
    assert_eq!(observe(&game, A).tokens.len(), game.public().events().len());
    let _ = (B, Seat::ALL, act as fn(&mut Game, Seat, Action));
}
