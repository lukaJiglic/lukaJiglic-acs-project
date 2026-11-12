//! Spec 006, criteria 5, 9 and 14, and the state blocks A to E and G on a
//! position written out by hand.

mod common;

use common::encoding::{block, card_bits, features, one_hot, relative, scripted_position};
use common::flow::{A, B, C, D, play_random};
use common::{court, kralj, pip, run_cases, seat, tarok};
use proptest::prelude::*;
use tarok_engine::encode::{self, observe};
use tarok_engine::{Action, Card, Game, Phase, Rank, Seat, Suit};

fn case() -> impl Strategy<Value = (u64, u8, Vec<u8>, usize)> {
    (
        any::<u64>(),
        0..4u8,
        proptest::collection::vec(any::<u8>(), 96),
        0..80usize,
    )
}

/// A random deal stopped before action number `stop` (or at its last
/// decision if it is shorter).
fn random_moment(seed: u64, dealer: u8, choices: &[u8], stop: usize) -> Game {
    let mut step = 0;
    let mut snapshot = None;
    play_random(seed, seat(dealer), choices, |game| {
        if step <= stop {
            snapshot = Some(game.clone());
        }
        step += 1;
    });
    snapshot.expect("a deal has a first decision")
}

/// The blocks with one entry (or one group of entries) per seat, in
/// relative seat order: the name and the width of one seat's part.
const PER_SEAT_BLOCKS: [(&str, usize); 10] = [
    ("A2", 54),
    ("B3", 1),
    ("B6", 1),
    ("B9", 1),
    ("C2", 1),
    ("C3", 1),
    ("C4", 13),
    ("E1", 1),
    ("E2", 1),
    ("E3", 1),
];

/// The public blocks that do not depend on who looks.
const SAME_FOR_ALL: [&str; 15] = [
    "A3", "A4", "A5", "A7", "A8", "B1", "B2", "B5", "C1", "D1", "D2", "D3", "E4", "G1", "G2",
];

#[test]
fn c05_seat_rotation() {
    run_cases(3_000, case(), |(seed, dealer, choices, stop)| {
        let game = random_moment(seed, dealer, &choices, stop);
        for observer in Seat::ALL {
            let here = observe(&game, observer);
            let there = observe(&game, observer.next());

            // Per-seat blocks: seat r of the next observer is seat r + 1 of
            // this one.
            for (name, width) in PER_SEAT_BLOCKS {
                let mine = features(&here, name);
                let theirs = features(&there, name);
                prop_assert_eq!(mine.len(), 4 * width);
                for r in 0..4 {
                    let shifted = (r + 1) % 4;
                    prop_assert_eq!(
                        &theirs[r * width..(r + 1) * width],
                        &mine[shifted * width..(shifted + 1) * width],
                        "block {} seat {}",
                        name,
                        r
                    );
                }
            }
            // B8 holds two per-seat parts: the dealer and forehand.
            let (mine, theirs) = (features(&here, "B8"), features(&there, "B8"));
            for part in 0..2 {
                for r in 0..4 {
                    prop_assert_eq!(theirs[part * 4 + r], mine[part * 4 + (r + 1) % 4]);
                }
            }
            // "Other seat" blocks have three parts: next, across, previous.
            // The next observer's first and second other seat are this
            // observer's second and third. F1 (shown voids) is public, so
            // those parts are equal.
            let (mine, theirs) = (features(&here, "F1"), features(&there, "F1"));
            prop_assert_eq!(mine.len(), 15);
            prop_assert_eq!(&theirs[..10], &mine[5..], "block F1");
            // The critic's true hands of the other seats turn the same way.
            let hands = encode::STATE_FEATURES..encode::STATE_FEATURES + 3 * 54;
            let (mine, theirs) = (&here.critic[hands.clone()], &there.critic[hands]);
            prop_assert_eq!(&theirs[..108], &mine[54..], "the critic's other hands");
            // So does the card-location target: seat 2 or 3 of this observer
            // is seat 1 or 2 of the next; seat 1 is the next observer itself.
            for (mine, theirs) in here.card_target.iter().zip(there.card_target) {
                match mine {
                    0 => prop_assert!(theirs == 0 || theirs == 3),
                    1 => prop_assert_eq!(theirs, 0),
                    _ => prop_assert_eq!(theirs, mine - 1),
                }
            }
            // Public blocks without seats are the same for both.
            for name in SAME_FOR_ALL {
                prop_assert_eq!(
                    features(&here, name),
                    features(&there, name),
                    "block {}",
                    name
                );
            }

            // Tokens: the same events; the seat field turns by one.
            prop_assert_eq!(here.tokens.len(), there.tokens.len());
            for (mine, theirs) in here.tokens.iter().zip(&there.tokens) {
                prop_assert_eq!(
                    [mine[0], mine[2], mine[3], mine[4]],
                    [theirs[0], theirs[2], theirs[3], theirs[4]]
                );
                let turned = if mine[1] == encode::NO_SEAT {
                    encode::NO_SEAT
                } else {
                    (mine[1] + 3) % 4
                };
                prop_assert_eq!(theirs[1], turned);
            }
        }
        Ok(())
    });
}

#[test]
fn c09_mask_equals_the_legal_actions() {
    run_cases(3_000, case(), |(seed, dealer, choices, _)| {
        let mut decisions = 0;
        play_random(seed, seat(dealer), &choices, |game| {
            decisions += 1;
            let acting = game.to_act().expect("a seat acts before the end");
            let observation = observe(game, acting);
            assert_eq!(observation.mask.len(), encode::N_ACTIONS);
            let legal = game.legal_actions();
            for (slot, &bit) in observation.mask.iter().enumerate() {
                // The mask bit of a slot is "this slot is a legal action".
                let expected = Action::from_slot(slot).is_some_and(|action| legal.contains(action));
                assert_eq!(bit, expected, "slot {slot} in {:?}", game.phase());
                assert_eq!(bit, legal.contains_slot(slot));
            }
            // At least one slot is legal.
            assert!(observation.mask.iter().any(|&bit| bit));
            assert_eq!(
                observation.mask.iter().filter(|&&bit| bit).count(),
                legal.len() as usize
            );
            // The flat writer gives the same.
            let mut flat = vec![true; encode::N_ACTIONS];
            encode::write_mask(game, acting, &mut flat);
            assert_eq!(flat, observation.mask);
            // A seat that is not to act has no legal action.
            let waiting = observe(game, acting.next());
            assert!(waiting.mask.iter().all(|&bit| !bit));
        });
        prop_assert!(decisions >= 53);
        Ok(())
    });
}

#[test]
fn c14_every_feature_lies_in_0_to_1() {
    run_cases(2_000, case(), |(seed, dealer, choices, _)| {
        let game = play_random(seed, seat(dealer), &choices, |game| {
            for observer in Seat::ALL {
                let observation = observe(game, observer);
                assert_eq!(observation.state.len(), encode::STATE_FEATURES);
                assert_eq!(observation.critic.len(), encode::CRITIC_FEATURES);
                for (index, &value) in observation.critic.iter().enumerate() {
                    assert!(
                        (0.0..=1.0).contains(&value),
                        "feature {index} is {value} in {:?}",
                        game.phase()
                    );
                }
                // The critic starts with the state.
                assert_eq!(
                    &observation.critic[..encode::STATE_FEATURES],
                    &observation.state[..]
                );
            }
        });
        // Also at the end of the deal, when nobody acts.
        for observer in Seat::ALL {
            let observation = observe(&game, observer);
            prop_assert!(
                observation
                    .critic
                    .iter()
                    .all(|value| (0.0..=1.0).contains(value))
            );
        }
        Ok(())
    });
}

// ---------------------------------------------------------------------
// The blocks on the position written out in `common::encoding`
// ---------------------------------------------------------------------

/// C's eleven cards in the scripted position.
fn hand_of_c() -> Vec<Card> {
    vec![
        court(Suit::Srce, Rank::Kaval),
        court(Suit::Srce, Rank::Dama),
        court(Suit::Karo, Rank::Fant),
        court(Suit::Karo, Rank::Kaval),
        court(Suit::Karo, Rank::Dama),
        kralj(Suit::Karo),
        pip(Suit::Pik, 7),
        pip(Suit::Pik, 8),
        pip(Suit::Pik, 9),
        pip(Suit::Pik, 10),
        court(Suit::Pik, Rank::Fant),
    ]
}

fn talon() -> [Card; 6] {
    [
        kralj(Suit::Srce),
        pip(Suit::Karo, 4),
        pip(Suit::Karo, 3),
        Card::MOND,
        pip(Suit::Karo, 2),
        pip(Suit::Karo, 1),
    ]
}

#[test]
fn block_a_cards_of_the_scripted_position() {
    let game = scripted_position();
    let from_c = observe(&game, C);
    // Seen from C: C is 0, D is 1, A is 2, B is 3.
    assert_eq!(
        [
            relative(C, C),
            relative(C, D),
            relative(C, A),
            relative(C, B)
        ],
        [0, 1, 2, 3]
    );

    // A1: the own hand.
    assert_eq!(features(&from_c, "A1"), card_bits(hand_of_c()));
    // A2: the cards each seat has played, in relative order.
    let played = [
        vec![pip(Suit::Srce, 3)],            // C
        vec![pip(Suit::Srce, 2)],            // D
        vec![kralj(Suit::Srce), tarok(12)],  // A
        vec![pip(Suit::Srce, 4), tarok(13)], // B
    ];
    let a2: Vec<f32> = played
        .iter()
        .flat_map(|cards| card_bits(cards.clone()))
        .collect();
    assert_eq!(features(&from_c, "A2"), a2);
    // A3: the current trick by play order: XII, XIII, nothing.
    let a3: Vec<f32> = [vec![tarok(12)], vec![tarok(13)], vec![]]
        .into_iter()
        .flat_map(card_bits)
        .collect();
    assert_eq!(features(&from_c, "A3"), a3);
    // Review item A4. One card later C has played the pik 7 and D is to
    // act with three cards on the table: XII, XIII, pik 7.
    let mut three = game.clone();
    three.apply(Action::Play(pip(Suit::Pik, 7))).unwrap();
    assert_eq!(three.to_act(), Some(D));
    let a3: Vec<f32> = [vec![tarok(12)], vec![tarok(13)], vec![pip(Suit::Pik, 7)]]
        .into_iter()
        .flat_map(card_bits)
        .collect();
    assert_eq!(features(&observe(&three, D), "A3"), a3);
    // After D's card the trick is over and the table is empty again.
    three.apply(Action::Play(pip(Suit::Kriz, 7))).unwrap();
    assert!(
        features(&observe(&three, B), "A3")
            .iter()
            .all(|&v| v == 0.0)
    );
    // A4: the talon, shown. A5: the packet A took.
    assert_eq!(features(&from_c, "A4"), card_bits(talon()));
    assert_eq!(
        features(&from_c, "A5"),
        card_bits([kralj(Suit::Srce), pip(Suit::Karo, 4)])
    );
    // A6: own discards. C has none; the declarer A sees both of its own.
    assert_eq!(features(&from_c, "A6"), card_bits([]));
    let from_a = observe(&game, A);
    assert_eq!(
        features(&from_a, "A6"),
        card_bits([pip(Suit::Karo, 4), tarok(2)])
    );
    // A7: the discards shown to all: the tarok II, for every seat.
    for observer in Seat::ALL {
        assert_eq!(
            features(&observe(&game, observer), "A7"),
            card_bits([tarok(2)])
        );
    }
    // A8: klop talon cards. Not built: zero.
    assert_eq!(features(&from_c, "A8"), card_bits([]));

    // The declarer's hand: I, III-XI (the XII and the srce kralj are played).
    let hand_of_a: Vec<Card> = [1u8, 3, 4, 5, 6, 7, 8, 9, 10, 11].map(tarok).to_vec();
    assert_eq!(features(&from_a, "A1"), card_bits(hand_of_a));
}

#[test]
fn block_b_contract_and_roles_of_the_scripted_position() {
    let game = scripted_position();
    let from_c = observe(&game, C);
    let from_a = observe(&game, A);

    // B1: the phase. Play is the seventh of the eight.
    assert_eq!(features(&from_c, "B1"), one_hot(8, 6));
    // B2: the contract, dve: the fourth of the thirteen bid values.
    assert_eq!(features(&from_c, "B2"), one_hot(13, 3));
    // B3: the declarer A, across from C.
    assert_eq!(features(&from_c, "B3"), one_hot(4, 2));
    assert_eq!(features(&from_a, "B3"), one_hot(4, 0));
    // B4: own role: declarer, called partner, opponent, no teams.
    assert_eq!(features(&from_a, "B4"), one_hot(4, 0));
    for observer in [B, C, D] {
        assert_eq!(features(&observe(&game, observer), "B4"), one_hot(4, 2));
    }
    // B5: the called suit, srce.
    assert_eq!(features(&from_c, "B5"), one_hot(4, 0));
    // B6: no partner is revealed (the declarer is alone).
    assert_eq!(features(&from_c, "B6"), [0.0; 4]);
    // B7: called king played; called king in the shown talon; I hold the
    // called king; the declarer is publicly alone.
    assert_eq!(features(&from_c, "B7"), [1.0, 1.0, 0.0, 1.0]);
    assert_eq!(features(&from_a, "B7"), [1.0, 1.0, 0.0, 1.0]);
    // B8: the dealer D (next after C) and forehand A (across).
    let b8: Vec<f32> = [one_hot(4, 1), one_hot(4, 2)].concat();
    assert_eq!(features(&from_c, "B8"), b8);
    // B9: A leads the current trick.
    assert_eq!(features(&from_c, "B9"), one_hot(4, 2));
}

#[test]
fn block_b_through_the_phases() {
    // The phase bit and the role at each kind of decision of one deal.
    let mut game = Game::from_seed(3, D);
    let phase_index = |phase: Phase| match phase {
        Phase::Bidding => 0,
        Phase::ContractChoice => 1,
        Phase::KingCall => 2,
        Phase::TalonPick => 3,
        Phase::Discard => 4,
        // 5 is the announcements, not built.
        Phase::Play => 6,
        Phase::Over => 7,
    };
    let mut seen = [false; 8];
    loop {
        let phase = game.phase();
        for observer in Seat::ALL {
            let observation = observe(&game, observer);
            assert_eq!(features(&observation, "B1"), one_hot(8, phase_index(phase)));
            let role = features(&observation, "B4");
            let declarer = game.public().declarer();
            match phase {
                // No declarer yet: no role.
                Phase::Bidding => assert_eq!(role, [0.0; 4]),
                _ if declarer == Some(observer) => assert_eq!(role, one_hot(4, 0)),
                // The others learn their role with the contract (a solo) or
                // with the call.
                Phase::ContractChoice | Phase::KingCall => assert_eq!(role, [0.0; 4]),
                _ => {
                    let partner = game.full().partner() == Some(observer);
                    assert_eq!(role, one_hot(4, if partner { 1 } else { 2 }));
                }
            }
            // B2, B3, B5 are zero until they are fixed.
            if game.public().contract().is_none() {
                assert_eq!(features(&observation, "B2"), [0.0; 13]);
            }
            if declarer.is_none() {
                assert_eq!(features(&observation, "B3"), [0.0; 4]);
            }
            if game.public().called_suit().is_none() {
                assert_eq!(features(&observation, "B5"), [0.0; 4]);
            }
            if phase != Phase::Play && phase != Phase::Over {
                assert_eq!(features(&observation, "B9"), [0.0; 4]);
            }
        }
        seen[phase_index(phase)] = true;
        if phase == Phase::Over {
            break;
        }
        // The lowest legal action; in the contract choice the lowest is the
        // winning bid.
        let action = game.legal_actions().iter().next().unwrap();
        game.apply(action).unwrap();
    }
    // Seed 3 with the lowest actions: three passes, tri, a king call.
    assert_eq!(seen, [true, true, true, true, true, false, true, true]);
}

#[test]
fn block_b6_and_b7_when_the_partner_plays_the_called_king() {
    // A plays tri and calls karo; B holds the karo kralj and plays it on
    // A's karo 1.
    let karo = |n| pip(Suit::Karo, n);
    let deal = common::flow::deal_with(
        [
            &[karo(1)],
            &[kralj(Suit::Karo), karo(4)],
            &[karo(3)],
            &[karo(2)],
        ],
        &[2, 3, 4, 5, 6, 7].map(tarok),
    );
    let mut game = common::flow::game(&deal);
    common::flow::declare(&mut game, A, tarok_engine::Contract::Tri);
    common::flow::act(&mut game, A, Action::CallKing(Suit::Karo));
    common::flow::run_until(&mut game, |g| g.phase() == Phase::Play);
    common::flow::act(&mut game, A, Action::Play(karo(1)));

    // Before the king falls: B knows it holds the king, nobody else does.
    for observer in Seat::ALL {
        let observation = observe(&game, observer);
        assert_eq!(features(&observation, "B6"), [0.0; 4]);
        let holds = if observer == B { 1.0 } else { 0.0 };
        assert_eq!(features(&observation, "B7"), [0.0, 0.0, holds, 0.0]);
    }
    assert_eq!(features(&observe(&game, B), "B4"), one_hot(4, 1));

    common::flow::act(&mut game, B, Action::Play(kralj(Suit::Karo)));
    // Now the partner B is public, for every seat in its own numbering.
    for observer in Seat::ALL {
        let observation = observe(&game, observer);
        assert_eq!(
            features(&observation, "B6"),
            one_hot(4, relative(observer, B))
        );
        // The king is played; B no longer holds it.
        assert_eq!(features(&observation, "B7"), [1.0, 0.0, 0.0, 0.0]);
    }
    // B is still the called partner.
    assert_eq!(features(&observe(&game, B), "B4"), one_hot(4, 1));
}

#[test]
fn block_c_bidding_of_the_scripted_position() {
    let game = scripted_position();
    let from_c = observe(&game, C);
    // C1: the highest bid, dve. C2: its bidder A, across from C.
    assert_eq!(features(&from_c, "C1"), one_hot(13, 3));
    assert_eq!(features(&from_c, "C2"), one_hot(4, 2));
    // C3: C, D and B have passed; A has not.
    assert_eq!(features(&from_c, "C3"), [1.0, 1.0, 0.0, 1.0]);
    // C4: the last bid of each seat, kept after a pass (reading B12-9). C
    // and D never bid: nothing. A bid dve. B bid dve and passed later: its
    // dve stays. So C4 is not a copy of C3 once the auction is over.
    let c4: Vec<f32> = [vec![0.0; 13], vec![0.0; 13], one_hot(13, 3), one_hot(13, 3)].concat();
    assert_eq!(features(&from_c, "C4"), c4);

    // In the middle of the auction: after B's dve only.
    let mut game = common::flow::game(&common::flow::king_in_talon_deal(false));
    common::flow::act(&mut game, B, Action::Bid(tarok_engine::Contract::Dve));
    let from_c = observe(&game, C);
    assert_eq!(features(&from_c, "C1"), one_hot(13, 3));
    assert_eq!(features(&from_c, "C2"), one_hot(4, 3));
    assert_eq!(features(&from_c, "C3"), [0.0; 4]);
    // Only B has called.
    let c4: Vec<f32> = [vec![0.0; 39], one_hot(13, 3)].concat();
    assert_eq!(features(&from_c, "C4"), c4);
    // Before any call: all zero.
    let fresh = common::flow::game(&common::flow::king_in_talon_deal(false));
    let observation = observe(&fresh, B);
    for name in ["C1", "C2", "C3", "C4"] {
        assert!(features(&observation, name).iter().all(|&v| v == 0.0));
    }

    // Random deals, every decision, every observer: the thirteen entries of
    // a seat hold its last real bid or nothing. The first entry (the pass)
    // is never set; block C3 holds the passes.
    let mut kept_after_a_pass = 0;
    for seed in 0..200u64 {
        let done = common::flow::seeded_game(seed);
        let mut game = Game::from_seed(seed, done.public().dealer());
        for action in done.full().actions() {
            for observer in Seat::ALL {
                let observation = observe(&game, observer);
                let c4 = features(&observation, "C4");
                for seat in Seat::ALL {
                    let last = game
                        .public()
                        .bids()
                        .iter()
                        .rev()
                        .find(|call| call.seat == seat && call.bid.is_some())
                        .and_then(|call| call.bid);
                    let expected = match last {
                        Some(bid) => one_hot(13, Action::Bid(bid).slot()),
                        None => vec![0.0; 13],
                    };
                    let from = 13 * relative(observer, seat);
                    assert_eq!(&c4[from..from + 13], expected, "seed {seed}");
                    assert_eq!(c4[from], 0.0, "the pass entry");
                    kept_after_a_pass +=
                        usize::from(last.is_some() && game.public().has_passed(seat));
                }
            }
            game.apply(action).unwrap();
        }
    }
    assert!(kept_after_a_pass > 1_000, "{kept_after_a_pass}");
}

#[test]
fn block_d_e_and_g_of_the_scripted_position() {
    let game = scripted_position();
    let from_c = observe(&game, C);
    // D: announcements and kontra. Not built: zero.
    for name in ["D1", "D2", "D3"] {
        assert!(features(&from_c, name).iter().all(|&v| v == 0.0));
    }
    // E1: cards in hand / 15: C 11, D 11, A 10, B 10.
    assert_eq!(
        features(&from_c, "E1"),
        [11.0 / 15.0, 11.0 / 15.0, 10.0 / 15.0, 10.0 / 15.0]
    );
    // E2: tricks won / 12: A has one.
    assert_eq!(features(&from_c, "E2"), [0.0, 0.0, 1.0 / 12.0, 0.0]);
    // E3: card points of each pile / 70. A's trick: srce kralj 5 and three
    // pips, 8 - 2 - 1 = 5.
    assert_eq!(features(&from_c, "E3"), [0.0, 0.0, 5.0 / 70.0, 0.0]);
    // E4: the second trick; two cards on the table.
    assert_eq!(features(&from_c, "E4"), [2.0 / 12.0, 2.0 / 3.0]);

    // G1: the nine scoring toggles of the Default preset, one-hot each:
    // S15 a, S22 a, S25 a, S26 a, S28 b, S29 a, S31 a, S32 a (of three),
    // S33 a.
    let g1: Vec<f32> = [
        vec![1.0, 0.0],      // S15
        vec![1.0, 0.0],      // S22
        vec![1.0, 0.0],      // S25
        vec![1.0, 0.0],      // S26
        vec![0.0, 1.0],      // S28
        vec![1.0, 0.0],      // S29
        vec![1.0, 0.0],      // S31
        vec![1.0, 0.0, 0.0], // S32
        vec![1.0, 0.0],      // S33
    ]
    .concat();
    assert_eq!(features(&from_c, "G1"), g1);
    // G2, G3: reserved rule slots and radelci. Zero.
    for name in ["G2", "G3"] {
        assert!(features(&from_c, name).iter().all(|&v| v == 0.0));
    }

    // During the exchange the declarer holds up to 15 cards: 15 / 15.
    let mut game = common::flow::game(&common::flow::king_in_talon_deal(false));
    common::flow::declare(&mut game, A, tarok_engine::Contract::SoloTri);
    common::flow::act(&mut game, A, Action::TakePacket(common::flow::packet(0)));
    let observation = observe(&game, A);
    assert_eq!(
        features(&observation, "E1"),
        [1.0, 12.0 / 15.0, 12.0 / 15.0, 12.0 / 15.0]
    );
    assert_eq!(features(&observation, "E4"), [0.0, 0.0]);
    assert_eq!(block("E1"), 869..873);
}
