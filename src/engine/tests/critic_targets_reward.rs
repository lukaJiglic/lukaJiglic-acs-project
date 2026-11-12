//! Spec 006, criteria 10, 11 and 12: the critic's hidden features, the
//! auxiliary targets and the reward.

mod common;

use common::encoding::{card_bits, one_hot, relative, scripted_position};
use common::flow::{A, B, C, D, act, deal_with, declare, play_random, run_until, seeded_game};
use common::written::{deal_1_partner, deal_3_king_in_talon};
use common::{kralj, pip, run_cases, seat, tarok};
use proptest::prelude::*;
use tarok_engine::encode::{self, observe, rewards};
use tarok_engine::{Action, Card, CardSet, Contract, Game, Phase, Seat, Suit};

/// The 274 hidden features: 3 x 54 other hands, 54 talon not yet public,
/// 54 hidden discards, 4 true partner seat.
fn extras(game: &Game, observer: Seat) -> Vec<f32> {
    let observation = observe(game, observer);
    assert_eq!(observation.critic.len(), encode::CRITIC_FEATURES);
    // The critic array starts with the state, unchanged.
    assert_eq!(
        &observation.critic[..encode::STATE_FEATURES],
        &observation.state[..]
    );
    observation.critic[encode::STATE_FEATURES..].to_vec()
}

/// A plays tri and calls karo; B holds the karo kralj. The exchange is made
/// with the lowest legal actions; forehand A is on the lead.
fn partner_game() -> Game {
    let karo = |n| pip(Suit::Karo, n);
    let deal = deal_with(
        [
            &[karo(1)],
            &[kralj(Suit::Karo), karo(4)],
            &[karo(3)],
            &[karo(2)],
        ],
        &[2, 3, 4, 5, 6, 7].map(tarok),
    );
    let mut game = common::flow::game(&deal);
    declare(&mut game, A, Contract::Tri);
    act(&mut game, A, Action::CallKing(Suit::Karo));
    run_until(&mut game, |g| g.phase() == Phase::Play);
    assert_eq!(game.full().partner(), Some(B));
    game
}

#[test]
fn c10_critic_features_are_the_state_plus_the_hidden_information() {
    // The scripted position, seen by C. In relative order the other seats
    // are D, A, B.
    let game = scripted_position();
    let extra = extras(&game, C);
    assert_eq!(extra.len(), 274);
    let hand = |seat: Seat| card_bits(game.full().hand(seat).iter());
    assert_eq!(extra[0..54], hand(D));
    assert_eq!(extra[54..108], hand(A));
    assert_eq!(extra[108..162], hand(B));
    // Written out for A: I and III-XI.
    assert_eq!(
        extra[54..108],
        card_bits([1u8, 3, 4, 5, 6, 7, 8, 9, 10, 11].map(tarok))
    );
    // The talon is public: nothing of it is hidden any more.
    assert_eq!(extra[162..216], card_bits([]));
    // The hidden discard: the karo 4. The shown one (II) is not hidden.
    assert_eq!(extra[216..270], card_bits([pip(Suit::Karo, 4)]));
    // The declarer has no partner: zero.
    assert_eq!(extra[270..274], [0.0; 4]);
    // For the declarer too the hidden discards are in the critic (and its
    // own in block A6 of the state).
    assert_eq!(extras(&game, A)[216..270], card_bits([pip(Suit::Karo, 4)]));

    // Before the talon is turned up: all six cards are "not yet public".
    let fresh = common::flow::game(&common::flow::king_in_talon_deal(false));
    let extra = extras(&fresh, B);
    assert_eq!(extra[162..216], card_bits(fresh.full().talon()));
    assert_eq!(extra[216..274], [0.0; 58]);

    // The true partner seat: the declarer's partner, in the observer's
    // numbering, from the call on (long before it is public).
    let game = partner_game();
    for observer in Seat::ALL {
        let extra = extras(&game, observer);
        assert_eq!(
            extra[270..274],
            one_hot(4, relative(observer, B)),
            "observer {}",
            observer.index()
        );
        // The public state does not show it yet.
        let b6 = &observe(&game, observer).state[common::encoding::block("B6")];
        assert_eq!(b6, [0.0; 4]);
    }

    // Random deals: at every decision, for all four observers, the 274
    // features equal the truth.
    let case = (
        any::<u64>(),
        0..4u8,
        proptest::collection::vec(any::<u8>(), 96),
    );
    run_cases(1_000, case, |(seed, dealer, choices)| {
        play_random(seed, seat(dealer), &choices, |game| {
            for observer in Seat::ALL {
                let extra = extras(game, observer);
                let full = game.full();
                let mut expected = Vec::new();
                for steps in 1..4 {
                    expected.extend(card_bits(full.hand(observer.plus(steps)).iter()));
                }
                let talon_hidden = game.public().talon().is_none();
                let talon: Vec<Card> = if talon_hidden {
                    full.talon().to_vec()
                } else {
                    Vec::new()
                };
                expected.extend(card_bits(talon));
                expected.extend(card_bits(
                    (full.discards() - game.public().shown_discards()).iter(),
                ));
                let mut partner = [0.0; 4];
                if let Some(seat) = full.partner() {
                    partner[relative(observer, seat)] = 1.0;
                }
                expected.extend(partner);
                assert_eq!(extra, expected);
            }
        });
        Ok(())
    });
}

/// Where a card truly is, as the observer's card-location class: 1 to 3 for
/// the hand of that other seat, 0 for anything else.
fn true_place(game: &Game, observer: Seat, card: Card) -> u8 {
    for steps in 1..4 {
        if game.full().hand(observer.plus(steps)).contains(card) {
            return steps as u8;
        }
    }
    0
}

#[test]
fn c11_partner_target_and_card_location_target() {
    // A partner game: A and B against C and D. Each seat's target is the
    // relative seat of its team-mate.
    let game = partner_game();
    assert_eq!(observe(&game, A).partner_target, 1, "A: B is the next seat");
    assert_eq!(
        observe(&game, B).partner_target,
        3,
        "B: A is the previous seat"
    );
    assert_eq!(observe(&game, C).partner_target, 1, "C: D is the next seat");
    assert_eq!(
        observe(&game, D).partner_target,
        3,
        "D: C is the previous seat"
    );

    // One against three: nobody has a single team-mate. "None" is 0.
    let alone = scripted_position();
    for observer in Seat::ALL {
        assert_eq!(observe(&alone, observer).partner_target, 0);
    }
    // In the bidding there are no teams yet.
    let fresh = common::flow::game(&common::flow::king_in_talon_deal(false));
    for observer in Seat::ALL {
        assert_eq!(observe(&fresh, observer).partner_target, 0);
    }

    // Card location in the scripted position, seen by C (others: D is 1,
    // A is 2, B is 3).
    let from_c = observe(&alone, C);
    let place = |card: Card| from_c.card_target[usize::from(card.index())];
    assert_eq!(place(kralj(Suit::Pik)), 1, "the pik kralj is with D");
    assert_eq!(place(tarok(1)), 2, "the pagat is with A");
    assert_eq!(place(Card::SKIS), 3, "the škis is with B");
    assert_eq!(place(pip(Suit::Karo, 4)), 0, "the karo 4 is a discard");
    assert_eq!(place(Card::MOND), 0, "the mond lies in the talon");
    assert_eq!(place(kralj(Suit::Srce)), 0, "the srce kralj is played");
    assert_eq!(
        place(kralj(Suit::Karo)),
        0,
        "the karo kralj is in C's own hand"
    );

    // Random deals: at every decision, for every observer, the target of
    // every unseen card is its true place; the partner target is the one
    // team-mate or none.
    let unseen_checked = std::cell::Cell::new(0u64);
    let with_mate = std::cell::Cell::new(0u64);
    let case = (
        any::<u64>(),
        0..4u8,
        proptest::collection::vec(any::<u8>(), 96),
    );
    run_cases(2_000, case, |(seed, dealer, choices)| {
        play_random(seed, seat(dealer), &choices, |game| {
            for observer in Seat::ALL {
                let observation = observe(game, observer);
                let unseen = &observation.state[common::encoding::block("F4")];
                for card in Card::all() {
                    let target = observation.card_target[usize::from(card.index())];
                    assert!(target <= 3);
                    if unseen[usize::from(card.index())] == 1.0 {
                        assert_eq!(target, true_place(game, observer, card), "{card}");
                        unseen_checked.set(unseen_checked.get() + 1);
                    }
                }
                // The talon while it is face down, and the discards, are
                // "out of play".
                if game.public().talon().is_none() {
                    for card in game.full().talon() {
                        assert_eq!(observation.card_target[usize::from(card.index())], 0);
                    }
                }
                for card in game.full().discards() {
                    assert_eq!(observation.card_target[usize::from(card.index())], 0);
                }

                // The partner target, from the truth: the seats on the
                // observer's side, once the teams are fixed.
                let full = game.full();
                let teams_fixed = game.public().contract().is_some_and(|contract| {
                    !contract.calls_king() || game.public().called_suit().is_some()
                });
                let mates: Vec<usize> = (1..4)
                    .filter(|&steps| {
                        teams_fixed
                            && full.on_declarer_side(observer.plus(steps))
                                == full.on_declarer_side(observer)
                    })
                    .collect();
                let expected = if mates.len() == 1 { mates[0] as u8 } else { 0 };
                assert_eq!(observation.partner_target, expected);
                with_mate.set(with_mate.get() + u64::from(expected != 0));
            }
        });
        Ok(())
    });
    assert!(unseen_checked.get() > 1_000_000);
    assert!(
        with_mate.get() > 10_000,
        "{} decisions with a team-mate",
        with_mate.get()
    );
}

#[test]
fn c12_reward_is_zero_sum_and_relative_to_the_other_three() {
    // By hand. Written deal 3: the entries are A 0, B -20, C 0, D 50.
    //   A: (0 - (-20 + 0 + 50) / 3) / 50 = (0 - 10) / 50 = -0.2
    //   B: (-20 - (0 + 0 + 50) / 3) / 50 = -36.6667 / 50 = -0.733333
    //   C: -0.2
    //   D: (50 - (0 - 20 + 0) / 3) / 50 = 56.6667 / 50 = 1.133333
    let (game, expected) = deal_3_king_in_talon();
    assert_eq!(expected.entries, [0, -20, 0, 50]);
    let reward = rewards(&game.score().unwrap());
    assert_eq!(reward.entries, [0, -20, 0, 50]);
    let by_hand = [-0.2, -0.733_333_3, -0.2, 1.133_333_3];
    for (got, want) in reward.reward.iter().zip(by_hand) {
        assert!((got - want).abs() < 1e-6, "{got} against {want}");
    }
    // The parts of the entries: D has game 20, difference 20, kralji 10;
    // B has the mondfang -20. In the contract's order: game, difference,
    // trula, kralji, kralj ultimo, pagat ultimo, valat, mondfang, klop,
    // radelc.
    assert_eq!(
        reward.entry_parts[D.index()],
        [20, 20, 0, 10, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(
        reward.entry_parts[B.index()],
        [0, 0, 0, 0, 0, 0, 0, -20, 0, 0]
    );
    // The reward by part: (own part - mean of the others' part) / 50.
    //   game: D (20 - 0) / 50 = 0.4; the others (0 - 20 / 3) / 50 = -0.133333
    //   mondfang: B (-20 - 0) / 50 = -0.4; the others (0 + 20 / 3) / 50 = 0.133333
    let close = |got: f32, want: f32| assert!((got - want).abs() < 1e-6, "{got} against {want}");
    close(reward.parts[D.index()][0], 0.4);
    close(reward.parts[A.index()][0], -0.133_333_3);
    close(reward.parts[D.index()][1], 0.4);
    close(reward.parts[D.index()][3], 0.2);
    close(reward.parts[B.index()][7], -0.4);
    close(reward.parts[A.index()][7], 0.133_333_3);
    close(reward.parts[D.index()][7], 0.133_333_3);

    // Written deal 1: A 35, B 0, C 35, D 0.
    //   A and C: (35 - 35 / 3) / 50 = 0.466667; B and D: (0 - 70 / 3) / 50 = -0.466667
    let (game, _) = deal_1_partner();
    let reward = rewards(&game.score().unwrap());
    for (got, want) in
        reward
            .reward
            .iter()
            .zip([0.466_666_7, -0.466_666_7, 0.466_666_7, -0.466_666_7])
    {
        close(*got, want);
    }

    // Every deal: the four rewards sum to zero, each is (own entry - mean
    // of the other three) / 50, and the ten parts of a seat add up to its
    // reward. Klop and radelc are zero until flow C.
    let mut nonzero = 0;
    for seed in 0..10_000u64 {
        let game = seeded_game(seed);
        let score = game.score().unwrap();
        let reward = rewards(&score);
        let entries = score.entries();
        assert_eq!(reward.entries, entries);
        let sum: f32 = reward.reward.iter().sum();
        assert!(sum.abs() < 1e-6, "seed {seed}: the rewards sum to {sum}");
        for seat in Seat::ALL {
            let i = seat.index();
            let others: i32 = entries.iter().sum::<i32>() - entries[i];
            let expected = (f64::from(entries[i]) - f64::from(others) / 3.0) / 50.0;
            assert!(
                (f64::from(reward.reward[i]) - expected).abs() < 1e-6,
                "seed {seed}"
            );
            let of_parts: f32 = reward.parts[i].iter().sum();
            assert!(
                (of_parts - reward.reward[i]).abs() < 1e-6,
                "seed {seed}: {of_parts}"
            );
            // The parts of the entry add up to the entry.
            assert_eq!(reward.entry_parts[i].iter().sum::<i32>(), entries[i]);
            assert_eq!(reward.entry_parts[i][8..], [0, 0]);
            assert_eq!(reward.parts[i][8..], [0.0, 0.0]);
        }
        // Each part on its own is zero-sum too.
        for part in 0..encode::N_SCORE_PARTS {
            let sum: f32 = Seat::ALL
                .iter()
                .map(|seat| reward.parts[seat.index()][part])
                .sum();
            assert!(sum.abs() < 1e-6);
        }
        nonzero += usize::from(reward.reward.iter().any(|&r| r != 0.0));
    }
    // Nearly every deal has an entry that is not zero (a lost tri with 35
    // points and a silent trula for the declarer writes 0).
    assert!(nonzero > 9_500, "{nonzero} deals with a reward");
    let _ = (CardSet::EMPTY, C);
}
