//! Spec 006, criteria 15 to 19: the vectorised environment. Spec 009,
//! criteria 50 and 51: the hold flag per table.
//!
//! N tables step at once. Every step is one decision per table. A table
//! plays runs of 24 deals; the dealer moves one seat per deal.

mod common;

use common::encoding::block;
use tarok_engine::encode::{self, rewards};
use tarok_engine::vecenv::{
    DEALS_PER_RUN, EnvState, Observations, VecEnv, VecEnvError, deal_seed, first_dealer,
};
use tarok_engine::{Action, Deal, Game, Phase, Seat};

/// A small generator for the choices of the tests (SplitMix64).
struct Choices(u64);

impl Choices {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
}

/// The mask row of a table.
fn mask_of(observations: &Observations, table: usize) -> &[bool] {
    &observations.mask[table * encode::N_ACTIONS..(table + 1) * encode::N_ACTIONS]
}

/// One legal slot per table, drawn from the masks.
fn random_actions(observations: &Observations, choices: &mut Choices) -> Vec<i64> {
    let tables = observations.seat.len();
    (0..tables)
        .map(|table| {
            let legal: Vec<usize> = mask_of(observations, table)
                .iter()
                .enumerate()
                .filter(|(_, bit)| **bit)
                .map(|(slot, _)| slot)
                .collect();
            assert!(!legal.is_empty(), "table {table} has no legal action");
            legal[(choices.next() % legal.len() as u64) as usize] as i64
        })
        .collect()
}

/// The absolute dealer of a table, read from its observation: block B8
/// holds the dealer relative to the acting seat.
fn dealer_of(observations: &Observations, table: usize) -> usize {
    let state =
        &observations.state[table * encode::STATE_FEATURES..(table + 1) * encode::STATE_FEATURES];
    let b8 = &state[block("B8")];
    let relative = b8[..4]
        .iter()
        .position(|&bit| bit == 1.0)
        .expect("a dealer");
    (observations.seat[table] as usize + relative) % 4
}

fn phase_bit(observations: &Observations, table: usize, index: usize) -> bool {
    let state =
        &observations.state[table * encode::STATE_FEATURES..(table + 1) * encode::STATE_FEATURES];
    state[block("B1")][index] == 1.0
}

#[test]
fn shapes_of_the_arrays() {
    let mut env = VecEnv::new(5, 7, 1).unwrap();
    assert_eq!((env.tables(), env.threads(), env.base_seed()), (5, 1, 7));
    let observations = env.observe().clone();
    assert_eq!(observations.seat.len(), 5);
    assert_eq!(observations.deal, vec![0; 5]);
    assert_eq!(observations.tokens.len(), 5 * 128 * 5);
    assert_eq!(observations.n_tokens, vec![1; 5]);
    assert_eq!(observations.state.len(), 5 * 1382);
    assert_eq!(observations.mask.len(), 5 * 160);
    assert_eq!(observations.critic.len(), 5 * 1656);
    assert_eq!(observations.partner_target, vec![0; 5]);
    assert_eq!(observations.card_target.len(), 5 * 54);
    for table in 0..5 {
        // The critic row starts with the state row.
        assert_eq!(
            &observations.critic[table * 1656..table * 1656 + 1382],
            &observations.state[table * 1382..(table + 1) * 1382]
        );
        // The first decision of a deal: bidding, by the seat after forehand.
        assert!(phase_bit(&observations, table, 0));
        let dealer = dealer_of(&observations, table);
        assert_eq!(dealer, first_dealer(table as u64, 0).index());
        assert_eq!(observations.seat[table] as usize, (dealer + 2) % 4);
    }

    let mut choices = Choices(1);
    let actions = random_actions(&observations, &mut choices);
    let result = env.step(&actions).unwrap().clone();
    assert_eq!(result.done, vec![false; 5]);
    assert_eq!(result.reward, vec![0.0; 5 * 4]);
    assert_eq!(result.parts, vec![0.0; 5 * 4 * 10]);
    assert_eq!(result.entries, vec![0; 5 * 4]);

    // No tables, no threads: refused.
    assert_eq!(VecEnv::new(0, 7, 1).err(), Some(VecEnvError::NoTables));
    assert_eq!(VecEnv::new(4, 7, 0).err(), Some(VecEnvError::NoThreads));
}

#[test]
fn c15_same_seed_and_actions_give_identical_arrays_with_1_and_12_threads() {
    let mut one = VecEnv::new(96, 2026, 1).unwrap();
    let mut twelve = VecEnv::new(96, 2026, 12).unwrap();
    let mut again = VecEnv::new(96, 2026, 12).unwrap();
    let mut choices = Choices(15);
    let mut deals = 0;
    for step in 0..600 {
        let observations = one.observe().clone();
        assert_eq!(
            &observations,
            twelve.observe(),
            "step {step}: 1 thread against 12"
        );
        assert_eq!(
            &observations,
            again.observe(),
            "step {step}: 12 threads twice"
        );
        let actions = random_actions(&observations, &mut choices);
        let result = one.step(&actions).unwrap().clone();
        assert_eq!(&result, twelve.step(&actions).unwrap(), "step {step}");
        assert_eq!(&result, again.step(&actions).unwrap(), "step {step}");
        deals += result.done.iter().filter(|&&done| done).count();
    }
    // The comparison ran across the ends of deals. With the round of
    // announcements (spec 009) a random deal has about 88 decisions, not
    // about 60: 600 steps at 96 tables finish about 600 deals.
    assert!(deals >= 500, "{deals} deals finished");

    // Another base seed gives other deals.
    let other = VecEnv::new(96, 2027, 1).unwrap();
    let first = VecEnv::new(96, 2026, 1).unwrap();
    assert_ne!(other.observe().state, first.observe().state);
}

#[test]
fn c16_tables_are_independent() {
    const TABLES: usize = 24;
    const ODD_ONE: usize = 7;
    let mut plain = VecEnv::new(TABLES, 16, 4).unwrap();
    let mut changed = VecEnv::new(TABLES, 16, 4).unwrap();
    let mut choices = Choices(16);
    let mut other_choices = Choices(1600);
    let mut odd_table_differed = false;

    let row = |values: &[f32], table: usize, width: usize| {
        values[table * width..(table + 1) * width].to_vec()
    };
    for step in 0..500 {
        let here = plain.observe().clone();
        let there = changed.observe().clone();
        // Every table but one sees exactly the same in both environments.
        for table in (0..TABLES).filter(|&table| table != ODD_ONE) {
            assert_eq!(
                here.seat[table], there.seat[table],
                "step {step} table {table}"
            );
            assert_eq!(here.deal[table], there.deal[table]);
            assert_eq!(here.n_tokens[table], there.n_tokens[table]);
            assert_eq!(
                here.tokens[table * 640..(table + 1) * 640],
                there.tokens[table * 640..(table + 1) * 640]
            );
            assert_eq!(
                row(&here.critic, table, 1656),
                row(&there.critic, table, 1656)
            );
            assert_eq!(mask_of(&here, table), mask_of(&there, table));
            assert_eq!(here.partner_target[table], there.partner_target[table]);
            assert_eq!(
                here.card_target[table * 54..(table + 1) * 54],
                there.card_target[table * 54..(table + 1) * 54]
            );
        }
        odd_table_differed |= row(&here.state, ODD_ONE, 1382) != row(&there.state, ODD_ONE, 1382);

        // The same actions everywhere, except at the odd table, which
        // chooses on its own in the second environment.
        let actions = random_actions(&here, &mut choices);
        let mut other_actions = actions.clone();
        other_actions[ODD_ONE] = random_actions(&there, &mut other_choices)[ODD_ONE];
        let result = plain.step(&actions).unwrap().clone();
        let other_result = changed.step(&other_actions).unwrap().clone();
        for table in (0..TABLES).filter(|&table| table != ODD_ONE) {
            assert_eq!(result.done[table], other_result.done[table]);
            assert_eq!(
                result.reward[table * 4..(table + 1) * 4],
                other_result.reward[table * 4..(table + 1) * 4]
            );
            assert_eq!(
                result.parts[table * 40..(table + 1) * 40],
                other_result.parts[table * 40..(table + 1) * 40]
            );
        }
    }
    assert!(odd_table_differed, "the changed table did play differently");
}

#[test]
fn c17_a_finished_deal_is_followed_by_the_next_with_the_dealer_one_seat_on() {
    const TABLES: usize = 6;
    const BASE: u64 = 17;
    let mut env = VecEnv::new(TABLES, BASE, 2).unwrap();
    let mut choices = Choices(17);
    let mut deals_done = [0u64; TABLES];
    let mut runs_started = [0u64; TABLES];

    // Enough steps for more than two runs of 24 deals at every table. A
    // random deal with the round of announcements (spec 009) has about 88
    // decisions.
    for _ in 0..(2 * DEALS_PER_RUN + 4) * 130 {
        let before = env.observe().clone();
        let dealers: Vec<usize> = (0..TABLES).map(|table| dealer_of(&before, table)).collect();
        let actions = random_actions(&before, &mut choices);
        let done = env.step(&actions).unwrap().done.clone();
        let after = env.observe().clone();

        for table in 0..TABLES {
            let (run, deal_in_run, counter) = env.position(table).unwrap();
            if !done[table] {
                // The same deal goes on.
                assert_eq!(after.deal[table], before.deal[table]);
                assert_eq!(dealer_of(&after, table), dealers[table]);
                // A hidden discard adds no token; nothing ever takes one away.
                assert!(after.n_tokens[table] >= before.n_tokens[table]);
                continue;
            }
            deals_done[table] += 1;
            // A new deal at once: its first decision, in the bidding, with
            // only the start token, and the deal counter one up.
            assert_eq!(after.deal[table], before.deal[table] + 1);
            assert_eq!(counter, deals_done[table]);
            assert_eq!(after.deal[table] as u64, deals_done[table]);
            assert!(phase_bit(&after, table, 0), "the bidding phase");
            assert_eq!(after.n_tokens[table], 1);
            assert_eq!(after.tokens[table * 640..table * 640 + 5], [1, 4, 0, 0, 0]);
            assert_eq!(after.tokens[table * 640 + 5..(table + 1) * 640], [0; 635]);

            // 24 deals make a run; then the next run starts at deal 0.
            assert_eq!(run, deals_done[table] / DEALS_PER_RUN as u64);
            assert_eq!(
                u64::from(deal_in_run),
                deals_done[table] % DEALS_PER_RUN as u64
            );
            let new_dealer = dealer_of(&after, table);
            if deal_in_run == 0 {
                runs_started[table] += 1;
                assert_eq!(new_dealer, first_dealer(table as u64, run).index());
            } else {
                // Inside a run the dealer moves one seat.
                assert_eq!(new_dealer, (dealers[table] + 1) % 4);
            }
            // The cards are those of the deal's own seed, and the first to
            // speak is the seat after forehand.
            let seed = deal_seed(BASE, table as u64, run, u64::from(deal_in_run));
            let deal = Deal::from_seed(seed);
            let acting = Seat::new(after.seat[table] as u8).unwrap();
            assert_eq!(acting.index(), (new_dealer + 2) % 4);
            let state = &after.state[table * 1382..(table + 1) * 1382];
            let hand: Vec<f32> = tarok_engine::Card::all()
                .map(|card| {
                    if deal.hand(acting).contains(card) {
                        1.0
                    } else {
                        0.0
                    }
                })
                .collect();
            assert_eq!(&state[block("A1")], &hand[..]);
        }
    }
    for table in 0..TABLES {
        assert!(
            deals_done[table] >= 2 * DEALS_PER_RUN as u64,
            "{} deals",
            deals_done[table]
        );
        assert!(runs_started[table] >= 2);
    }
}

/// Nothing of a deal reaches the next deal of the table (the second review
/// of spec 009, point 2). At every step, for every table, the observation
/// of the environment is the one of a fresh game of that deal's seed and
/// dealer that takes the deal's actions so far: tokens, state, mask and
/// the critic's features. So a table that kept anything of the deal
/// before, such as the seats shown as opponents in its round of
/// announcements, would differ in the deal after.
#[test]
fn c17_every_decision_of_a_table_is_that_of_a_fresh_game_of_its_deal() {
    const TABLES: usize = 4;
    const BASE: u64 = 1709;
    let mut env = VecEnv::new(TABLES, BASE, 2).unwrap();
    let mut choices = Choices(1709);
    // The actions of each table's deal so far.
    let mut played: Vec<Vec<Action>> = vec![Vec::new(); TABLES];
    let mut deals = 0u32;
    // Decisions with the partner hidden, in a deal that follows a deal in
    // which a seat was shown as an opponent: there a kept flag would show.
    let mut after_a_shown_seat = 0u32;
    let mut shown_now = [false; TABLES];
    let mut shown_before = [false; TABLES];
    for _ in 0..12_000 {
        let before = env.observe().clone();
        let mut actions = random_actions(&before, &mut choices);
        for table in 0..TABLES {
            let (run, deal_in_run, _) = env.position(table).unwrap();
            let seed = deal_seed(BASE, table as u64, run, u64::from(deal_in_run));
            let dealer = first_dealer(table as u64, run).plus(deal_in_run as usize);
            let mut fresh = Game::from_seed(seed, dealer);
            for &action in &played[table] {
                fresh.apply(action).expect("legal in a fresh game");
            }
            let seat = fresh.to_act().expect("a decision");
            assert_eq!(before.seat[table], seat.index() as i64);
            // In the auction the lowest legal action three times out of
            // four, as `common::flow::play_random` does: drawn uniformly,
            // nearly every auction ends in a solo and no partner is ever
            // hidden.
            if matches!(fresh.phase(), Phase::Bidding | Phase::ContractChoice)
                && !choices.next().is_multiple_of(4)
            {
                let lowest = mask_of(&before, table).iter().position(|&bit| bit);
                actions[table] = lowest.expect("a legal action") as i64;
            }

            let own = encode::observe(&fresh, seat);
            assert_eq!(mask_of(&before, table), &own.mask[..], "table {table}");
            let state =
                &before.state[table * encode::STATE_FEATURES..(table + 1) * encode::STATE_FEATURES];
            assert_eq!(state, &own.state[..], "table {table}");
            let critic = &before.critic
                [table * encode::CRITIC_FEATURES..(table + 1) * encode::CRITIC_FEATURES];
            assert_eq!(critic, &own.critic[..], "table {table}");
            assert_eq!(before.n_tokens[table] as usize, own.tokens.len());
            let tokens = &before.tokens[table * 640..(table + 1) * 640];
            for (row, token) in own.tokens.iter().enumerate() {
                assert_eq!(&tokens[row * 5..row * 5 + 5], &token[..], "table {table}");
            }

            let public = fresh.public();
            let hidden = public.called_king().is_some()
                && public.partner() == tarok_engine::PublicPartner::NotKnown;
            if hidden && shown_before[table] {
                after_a_shown_seat += 1;
            }
            shown_now[table] |= hidden
                && Seat::ALL
                    .into_iter()
                    .any(|seat| public.side_of(seat) == Some(false));
        }
        let done = env.step(&actions).unwrap().done.clone();
        for table in 0..TABLES {
            if done[table] {
                played[table].clear();
                deals += 1;
                shown_before[table] = shown_now[table];
                shown_now[table] = false;
            } else {
                let action = Action::from_slot(actions[table] as usize).expect("a slot");
                played[table].push(action);
            }
        }
    }
    println!(
        "{deals} deals at {TABLES} tables, every decision compared with a fresh game; \
         {after_a_shown_seat} decisions with the partner hidden in a deal that follows a deal \
         with a seat shown as an opponent"
    );
    assert!(deals >= 400, "{deals} deals");
    assert!(after_a_shown_seat >= 500, "{after_a_shown_seat}");
}

#[test]
fn c18_rewards_and_parts_are_delivered_once_per_deal() {
    const TABLES: usize = 16;
    const BASE: u64 = 18;
    let mut env = VecEnv::new(TABLES, BASE, 3).unwrap();
    let mut choices = Choices(18);
    // The actions of the deal each table is in, to play it again by hand.
    let mut taken: Vec<Vec<i64>> = vec![Vec::new(); TABLES];
    let mut finished = 0;

    for _ in 0..2_000 {
        let observations = env.observe().clone();
        let positions: Vec<(u64, u32, u64)> = (0..TABLES)
            .map(|table| env.position(table).unwrap())
            .collect();
        let actions = random_actions(&observations, &mut choices);
        let result = env.step(&actions).unwrap().clone();
        for table in 0..TABLES {
            taken[table].push(actions[table]);
            let reward = &result.reward[table * 4..(table + 1) * 4];
            let parts = &result.parts[table * 40..(table + 1) * 40];
            let entries = &result.entries[table * 4..(table + 1) * 4];
            if !result.done[table] {
                // Nothing is delivered while the deal goes on.
                assert_eq!(reward, [0.0; 4]);
                assert_eq!(parts, [0.0; 40]);
                assert_eq!(entries, [0; 4]);
                continue;
            }
            finished += 1;
            // The deal again, outside the environment: the same seed, the
            // same dealer, the same actions.
            let (run, deal_in_run, _) = positions[table];
            let seed = deal_seed(BASE, table as u64, run, u64::from(deal_in_run));
            let dealer = first_dealer(table as u64, run).plus(deal_in_run as usize);
            let mut game = Game::from_seed(seed, dealer);
            for &slot in &taken[table] {
                game.apply(Action::from_slot(slot as usize).unwrap())
                    .unwrap();
            }
            assert_eq!(game.phase(), Phase::Over);
            let expected = rewards(&game.score().unwrap());
            assert_eq!(reward, expected.reward);
            assert_eq!(entries, expected.entries);
            // A reward is a 32-bit number. Since spec 009 an entry can be
            // thousands of points (a valat under kontra), so the room for
            // rounding grows with the size of the rewards: 1e-6 for
            // rewards up to 1, and in proportion above that.
            let largest = reward.iter().fold(0f32, |most, r| most.max(r.abs()));
            let room = 1e-6 * largest.max(1.0);
            for seat in 0..4 {
                let of_seat = &parts[seat * 10..(seat + 1) * 10];
                assert_eq!(of_seat, expected.parts[seat]);
                // The ten parts add up to the seat's reward.
                let sum: f32 = of_seat.iter().sum();
                assert!(
                    (sum - reward[seat]).abs() < room,
                    "{sum} against {}",
                    reward[seat]
                );
                // Klop and radelc: zero until flow C.
                assert_eq!(of_seat[8..], [0.0, 0.0]);
            }
            let total: f32 = reward.iter().sum();
            assert!(total.abs() < room, "the rewards sum to {total}");
            taken[table].clear();
        }
    }
    // About 88 decisions per random deal since spec 009: 2,000 steps at
    // 16 tables finish about 350 deals.
    assert!(finished >= 300, "{finished} deals finished");
}

#[test]
fn c19_a_random_policy_run_of_100_000_deals_raises_no_error() {
    const TABLES: usize = 512;
    let mut env = VecEnv::new(TABLES, 19, 12).unwrap();
    let mut choices = Choices(19);
    let mut deals: u64 = 0;
    let mut steps: u64 = 0;
    let mut longest = 0i64;
    while deals < 100_000 {
        let actions = {
            let observations = env.observe();
            longest = longest.max(*observations.n_tokens.iter().max().unwrap());
            random_actions(observations, &mut choices)
        };
        let result = env.step(&actions).expect("a legal action is never refused");
        deals += result.done.iter().filter(|&&done| done).count() as u64;
        steps += 1;
        assert!(steps < 20_000, "the deals do not end");
    }
    // Every table went through several runs of 24 deals.
    for table in 0..TABLES {
        let (run, _, counter) = env.position(table).unwrap();
        assert!(counter >= 100 && run >= 4, "table {table}: {counter} deals");
    }
    assert!(longest <= encode::MAX_TOKENS as i64);
    println!(
        "100,000 deals in {steps} steps of {TABLES} tables; longest sequence {longest} tokens"
    );
}

#[test]
fn an_illegal_action_is_an_error_and_changes_nothing() {
    let mut env = VecEnv::new(8, 20, 2).unwrap();
    let mut choices = Choices(20);
    for _ in 0..40 {
        let actions = random_actions(env.observe(), &mut choices);
        env.step(&actions).unwrap();
    }
    let before = env.observe().clone();
    let state_before = env.state();
    let mut actions = random_actions(&before, &mut choices);
    // A slot that is not legal at table 5.
    let illegal = mask_of(&before, 5).iter().position(|&bit| !bit).unwrap();
    actions[5] = illegal as i64;
    assert_eq!(
        env.step(&actions).err(),
        Some(VecEnvError::IllegalAction {
            table: 5,
            slot: illegal as i64
        })
    );
    // Slots outside 0-159 and the wrong number of actions too.
    for bad in [-1i64, 160, i64::MAX] {
        actions[5] = bad;
        assert_eq!(
            env.step(&actions).err(),
            Some(VecEnvError::IllegalAction {
                table: 5,
                slot: bad
            })
        );
    }
    assert_eq!(
        env.step(&actions[..7]).err(),
        Some(VecEnvError::WrongActionCount {
            expected: 8,
            got: 7
        })
    );
    // No table moved: not even the ones before table 5.
    assert_eq!(env.observe(), &before);
    assert_eq!(env.state(), state_before);
    let message = VecEnvError::IllegalAction {
        table: 5,
        slot: 160,
    }
    .to_string();
    assert!(
        message.contains("table 5") && message.contains("160"),
        "{message}"
    );
}

#[test]
fn the_state_restores_an_environment_exactly() {
    let mut env = VecEnv::new(20, 21, 3).unwrap();
    let mut choices = Choices(21);
    for _ in 0..777 {
        let actions = random_actions(env.observe(), &mut choices);
        env.step(&actions).unwrap();
    }
    let saved = env.state();
    // A fresh environment, other thread count, takes the state...
    let mut restored = VecEnv::new(20, 21, 1).unwrap();
    assert_ne!(restored.observe(), env.observe());
    restored.set_state(&saved).unwrap();
    assert_eq!(restored.state(), saved);
    // ...and goes on exactly as the first one.
    for step in 0..400 {
        let observations = env.observe().clone();
        assert_eq!(&observations, restored.observe(), "step {step}");
        let actions = random_actions(&observations, &mut choices);
        let result = env.step(&actions).unwrap().clone();
        assert_eq!(&result, restored.step(&actions).unwrap());
    }

    // A state for another number of tables or another base seed is refused.
    let mut small = VecEnv::new(4, 21, 1).unwrap();
    assert!(matches!(
        small.set_state(&saved),
        Err(VecEnvError::BadState(_))
    ));
    let mut other_seed = VecEnv::new(20, 22, 1).unwrap();
    assert!(matches!(
        other_seed.set_state(&saved),
        Err(VecEnvError::BadState(_))
    ));
    // A state with an action that is not legal in its deal is refused, and
    // the environment stays as it was.
    let mut broken = saved.clone();
    let table = (0..20).find(|&t| broken.n_actions[t] > 0).unwrap();
    let per_table = broken.actions.len() / 20;
    broken.actions[table * per_table] = 159;
    let before = restored.observe().clone();
    assert!(matches!(
        restored.set_state(&broken),
        Err(VecEnvError::BadState(_))
    ));
    assert_eq!(restored.observe(), &before);

    // Review item D13. The number of finished deals follows from the run
    // and the deal: 24 * run + deal. A state that says otherwise is
    // refused, and so is a run so large that the count leaves 63 bits.
    let message = |env: &mut VecEnv, state: &EnvState| -> String {
        match env.set_state(state) {
            Err(VecEnvError::BadState(why)) => why,
            other => panic!("not refused: {other:?}"),
        }
    };
    for table in 0..20 {
        assert_eq!(
            saved.deals_done[table],
            24 * saved.run[table] + u64::from(saved.deal_in_run[table])
        );
    }
    let mut contradicts = saved.clone();
    contradicts.deals_done[3] = 999_999;
    let why = message(&mut restored, &contradicts);
    assert!(why.contains("table 3") && why.contains("999999"), "{why}");
    let mut huge_run = saved.clone();
    huge_run.run[5] = 1 << 62;
    let why = message(&mut restored, &huge_run);
    assert!(why.contains("table 5") && why.contains("run"), "{why}");
    let mut huge_both = saved.clone();
    huge_both.run[5] = u64::MAX / 24;
    huge_both.deal_in_run[5] = 0;
    huge_both.deals_done[5] = 24 * (u64::MAX / 24);
    huge_both.n_actions[5] = 0;
    let why = message(&mut restored, &huge_both);
    assert!(why.contains("table 5") && why.contains("run"), "{why}");
    assert_eq!(restored.observe(), &before);
    // The largest run that is allowed still works.
    let mut far = saved.clone();
    far.run[5] = (i64::MAX as u64) / 24 - 1;
    far.deal_in_run[5] = 23;
    far.deals_done[5] = 24 * far.run[5] + 23;
    far.n_actions[5] = 0;
    far.actions[5 * per_table..6 * per_table].fill(0);
    restored.set_state(&far).unwrap();
    assert_eq!(restored.state(), far);
}

#[test]
fn deal_seeds_are_frozen_and_differ() {
    // Golden values: if this fails, stored (base seed, table, run, deal)
    // numbers point at other deals than before.
    assert_eq!(deal_seed(0, 0, 0, 0), GOLDEN_SEEDS[0]);
    assert_eq!(deal_seed(1, 0, 0, 0), GOLDEN_SEEDS[1]);
    assert_eq!(deal_seed(0, 1, 0, 0), GOLDEN_SEEDS[2]);
    assert_eq!(deal_seed(0, 0, 1, 0), GOLDEN_SEEDS[3]);
    assert_eq!(deal_seed(0, 0, 0, 1), GOLDEN_SEEDS[4]);
    assert_eq!(deal_seed(u64::MAX, 1023, 77, 23), GOLDEN_SEEDS[5]);

    // No two of 40,000 neighbouring positions share a seed.
    let mut seen = std::collections::HashSet::new();
    for base in 0..5u64 {
        for table in 0..20u64 {
            for run in 0..20u64 {
                for deal in 0..20u64 {
                    assert!(seen.insert(deal_seed(base, table, run, deal)));
                }
            }
        }
    }
    // The first dealer of a run: (table + run) mod 4.
    assert_eq!(first_dealer(0, 0).index(), 0);
    assert_eq!(first_dealer(5, 0).index(), 1);
    assert_eq!(first_dealer(5, 2).index(), 3);
    assert_eq!(first_dealer(u64::MAX, u64::MAX).index(), 2);
    assert_eq!(DEALS_PER_RUN, 24);
}

/// Generated once on 2026-11-11 and frozen.
const GOLDEN_SEEDS: [u64; 6] = [
    12_321_809_464_288_559_627,
    12_174_098_620_933_058_904,
    14_815_432_490_932_255_636,
    4_514_656_745_954_956_636,
    17_960_153_663_297_965_470,
    1_794_999_558_585_611_450,
];

#[test]
fn random_runs_do_not_depend_on_the_thread_count() {
    let mut one = VecEnv::new(40, 23, 1).unwrap();
    let mut many = VecEnv::new(40, 23, 12).unwrap();
    let first = one.run_random(700, 5).unwrap();
    assert_eq!(first, many.run_random(700, 5).unwrap());
    assert_eq!(first.decisions, 700 * 40);
    // About 88 decisions per random deal since spec 009.
    assert!(first.deals >= 250, "{} deals", first.deals);
    assert_eq!(one.observe(), many.observe());
    assert_eq!(one.state(), many.state());
    // Another seed plays other cards.
    let mut other = VecEnv::new(40, 23, 1).unwrap();
    other.run_random(700, 6).unwrap();
    assert_ne!(other.state(), one.state());
    // No steps: nothing moves.
    let before = one.state();
    assert_eq!(one.run_random(0, 5).unwrap().decisions, 0);
    assert_eq!(one.state(), before);
}

#[test]
fn tables_write_into_arrays_of_the_caller_and_check_their_sizes() {
    use tarok_engine::vecenv::{ObservationBuffers, ResultBuffers, Tables};
    let mut tables = Tables::new(3, 24, 2).unwrap();
    let mut reference = VecEnv::new(3, 24, 1).unwrap();
    let (mut seat, mut deal, mut n_tokens, mut partner) =
        ([0i64; 3], [0i64; 3], [0i64; 3], [0i64; 3]);
    let mut tokens = vec![0i16; 3 * 640];
    let mut state = vec![0f32; 3 * 1382];
    let mut mask = vec![false; 3 * 160];
    let mut critic = vec![0f32; 3 * 1656];
    let mut cards = vec![0i64; 3 * 54];
    tables
        .observe_into(ObservationBuffers {
            seat: &mut seat,
            deal: &mut deal,
            tokens: &mut tokens,
            n_tokens: &mut n_tokens,
            state: &mut state,
            mask: &mut mask,
            critic: &mut critic,
            partner_target: &mut partner,
            card_target: &mut cards,
        })
        .unwrap();
    assert_eq!(state, reference.observe().state);
    assert_eq!(tokens, reference.observe().tokens);

    let actions = random_actions(reference.observe(), &mut Choices(24));
    let (mut done, mut reward, mut parts, mut entries) =
        ([false; 3], [0f32; 12], [0f32; 120], [0i32; 12]);
    tables
        .step_into(
            &actions,
            ObservationBuffers {
                seat: &mut seat,
                deal: &mut deal,
                tokens: &mut tokens,
                n_tokens: &mut n_tokens,
                state: &mut state,
                mask: &mut mask,
                critic: &mut critic,
                partner_target: &mut partner,
                card_target: &mut cards,
            },
            ResultBuffers {
                done: &mut done,
                reward: &mut reward,
                parts: &mut parts,
                entries: &mut entries,
            },
        )
        .unwrap();
    reference.step(&actions).unwrap();
    assert_eq!(critic, reference.observe().critic);
    assert_eq!(mask, reference.observe().mask);

    // An array of the wrong size is refused by name, before anything moves.
    let before = tables.state();
    let mut short_state = vec![0f32; 3 * 1382 - 1];
    let error = tables
        .step_into(
            &actions,
            ObservationBuffers {
                seat: &mut seat,
                deal: &mut deal,
                tokens: &mut tokens,
                n_tokens: &mut n_tokens,
                state: &mut short_state,
                mask: &mut mask,
                critic: &mut critic,
                partner_target: &mut partner,
                card_target: &mut cards,
            },
            ResultBuffers {
                done: &mut done,
                reward: &mut reward,
                parts: &mut parts,
                entries: &mut entries,
            },
        )
        .unwrap_err();
    assert_eq!(
        error,
        VecEnvError::BufferSize {
            name: "state",
            expected: 3 * 1382,
            got: 3 * 1382 - 1
        }
    );
    assert_eq!(tables.state(), before);
    assert_eq!(
        (tables.len(), tables.threads(), tables.base_seed()),
        (3, 2, 24)
    );
}

// ---------------------------------------------------------------------
// Spec 009, criteria 50 and 51: the hold flag (D142, B14-1 a)
// ---------------------------------------------------------------------

/// Everything the environment shows for one table, as text: the seat to
/// act, the deal number, the tokens, the state, the hidden features, the
/// mask and the two targets.
fn table_view(observations: &Observations, table: usize) -> String {
    format!(
        "{:?}",
        (
            observations.seat[table],
            observations.deal[table],
            observations.n_tokens[table],
            &observations.tokens[table * 640..(table + 1) * 640],
            &observations.state[table * 1382..(table + 1) * 1382],
            &observations.critic[table * 1656..(table + 1) * 1656],
            mask_of(observations, table),
            observations.partner_target[table],
            &observations.card_target[table * 54..(table + 1) * 54],
        )
    )
}

#[test]
fn c50_a_held_table_does_not_move() {
    const TABLES: usize = 12;
    let mut env = VecEnv::new(TABLES, 50, 3).unwrap();
    let mut choices = Choices(50);
    // Actions a held table may be handed: none of them is looked at.
    let junk: [i64; 6] = [-1, 89, 159, 1_000, i64::MAX, i64::MIN];
    let mut held_steps = 0;
    let mut held_with_a_legal_action = 0;
    let mut finished = 0;

    for step in 0..1_500 {
        let before = env.observe().clone();
        let state_before = env.state();
        let mut actions = random_actions(&before, &mut choices);
        // In the first steps no table is held, so the tables spread out.
        let hold: Vec<bool> = (0..TABLES)
            .map(|_| step >= 30 && choices.next().is_multiple_of(3))
            .collect();
        for table in 0..TABLES {
            if hold[table] {
                held_steps += 1;
                // Mostly junk; sometimes the legal action stays, and is
                // ignored all the same.
                if choices.next().is_multiple_of(4) {
                    held_with_a_legal_action += 1;
                } else {
                    actions[table] = junk[(choices.next() % 6) as usize];
                }
            }
        }
        let result = env.step_hold(&actions, &hold).unwrap().clone();
        let after = env.observe();
        let state_after = env.state();
        for (table, &is_held) in hold.iter().enumerate() {
            let reward = &result.reward[table * 4..(table + 1) * 4];
            let parts = &result.parts[table * 40..(table + 1) * 40];
            let entries = &result.entries[table * 4..(table + 1) * 4];
            let row = |state: &EnvState| {
                (
                    state.run[table],
                    state.deal_in_run[table],
                    state.deals_done[table],
                    state.n_actions[table],
                )
            };
            if is_held {
                // The same observation and the same seat to act.
                assert!(
                    table_view(&before, table) == table_view(after, table),
                    "step {step}, table {table}: a held table shows something else"
                );
                // No reward, not done.
                assert!(!result.done[table]);
                assert_eq!(reward, [0.0; 4]);
                assert_eq!(parts, [0.0; 40]);
                assert_eq!(entries, [0; 4]);
                // Nothing was played.
                assert_eq!(row(&state_before), row(&state_after));
            } else {
                // The table took its action: one action more in its deal,
                // or the deal ended and the next one started.
                let (run, deal, done, n) = row(&state_before);
                if result.done[table] {
                    finished += 1;
                    assert_eq!(state_after.deals_done[table], done + 1);
                    assert_eq!(state_after.n_actions[table], 0);
                } else {
                    assert_eq!(row(&state_after), (run, deal, done, n + 1));
                }
            }
        }
    }
    assert!(held_steps > 3_000, "{held_steps} held table-steps");
    assert!(held_with_a_legal_action > 500);
    assert!(
        finished > 50,
        "{finished} deals finished next to held tables"
    );

    // Every table held: nothing moves at all.
    let before = (env.observe().clone(), env.state());
    let result = env
        .step_hold(&[-7; TABLES], &[true; TABLES])
        .unwrap()
        .clone();
    assert_eq!(before, (env.observe().clone(), env.state()));
    assert!(result.done.iter().all(|&done| !done));
    assert!(result.reward.iter().all(|&reward| reward == 0.0));

    // No table held: the same as a plain step.
    let mut plain = VecEnv::new(TABLES, 50, 3).unwrap();
    plain.set_state(&env.state()).unwrap();
    let actions = random_actions(env.observe(), &mut choices);
    let with_flags = env.step_hold(&actions, &[false; TABLES]).unwrap().clone();
    assert_eq!(&with_flags, plain.step(&actions).unwrap());
    assert_eq!(env.observe(), plain.observe());
    assert_eq!(env.state(), plain.state());

    // An action that is not legal at a table that is not held is an error,
    // and no table moves: not the held ones, not the others.
    let before = (env.observe().clone(), env.state());
    let mut actions = random_actions(env.observe(), &mut choices);
    actions[3] = 89;
    actions[5] = 89;
    let mut hold = [false; TABLES];
    hold[5] = true;
    assert_eq!(
        env.step_hold(&actions, &hold).unwrap_err(),
        VecEnvError::IllegalAction { table: 3, slot: 89 }
    );
    assert_eq!(before, (env.observe().clone(), env.state()));
    // One flag per table.
    assert_eq!(
        env.step_hold(&actions, &hold[..TABLES - 1]).unwrap_err(),
        VecEnvError::WrongHoldCount {
            expected: TABLES,
            got: TABLES - 1
        }
    );
    assert_eq!(before, (env.observe().clone(), env.state()));
}

/// What a table played in one deal: its actions, the entries and rewards
/// the environment delivered, and the record of the deal as text.
type PlayedDeal = (Vec<i64>, Vec<i32>, Vec<f32>, String);

/// `TABLES` tables play `steps` steps. The action of a table depends on
/// the table and on how many decisions it has taken, on nothing else: so
/// a table that waits takes the same action later. With `holds` every
/// table is held at random, about one step in three.
fn play_with_holds(steps: usize, holds: bool) -> Vec<Vec<PlayedDeal>> {
    const TABLES: usize = 200;
    const BASE: u64 = 51;
    let mut env = VecEnv::new(TABLES, BASE, 4).unwrap();
    let mut held = Choices(5_100);
    let mut decisions = [0u64; TABLES];
    let mut current: Vec<Vec<i64>> = vec![Vec::new(); TABLES];
    let mut played: Vec<Vec<PlayedDeal>> = vec![Vec::new(); TABLES];

    for _ in 0..steps {
        let observations = env.observe().clone();
        let hold: Vec<bool> = (0..TABLES)
            .map(|_| holds && held.next().is_multiple_of(3))
            .collect();
        let actions: Vec<i64> = (0..TABLES)
            .map(|table| {
                if hold[table] {
                    // Not looked at.
                    return 89;
                }
                let legal: Vec<usize> = mask_of(&observations, table)
                    .iter()
                    .enumerate()
                    .filter(|(_, bit)| **bit)
                    .map(|(slot, _)| slot)
                    .collect();
                // The choice of this table at this decision of its own.
                let mut choice = Choices((table as u64) << 32 | decisions[table]);
                legal[(choice.next() % legal.len() as u64) as usize] as i64
            })
            .collect();
        let result = if holds {
            env.step_hold(&actions, &hold).unwrap().clone()
        } else {
            env.step(&actions).unwrap().clone()
        };
        for table in 0..TABLES {
            if hold[table] {
                assert!(!result.done[table]);
                continue;
            }
            decisions[table] += 1;
            current[table].push(actions[table]);
            if !result.done[table] {
                continue;
            }
            // The deal again, outside the environment, for its record.
            let number = played[table].len() as u64;
            let (run, deal_in_run) = (number / DEALS_PER_RUN as u64, number % DEALS_PER_RUN as u64);
            let seed = deal_seed(BASE, table as u64, run, deal_in_run);
            let dealer = first_dealer(table as u64, run).plus(deal_in_run as usize);
            let mut game = Game::from_seed(seed, dealer);
            for &slot in &current[table] {
                game.apply(Action::from_slot(slot as usize).unwrap())
                    .unwrap();
            }
            let record = tarok_engine::record::Record::of_game(seed, &game).unwrap();
            assert_eq!(
                record.entries[..],
                result.entries[table * 4..(table + 1) * 4]
            );
            played[table].push((
                std::mem::take(&mut current[table]),
                result.entries[table * 4..(table + 1) * 4].to_vec(),
                result.reward[table * 4..(table + 1) * 4].to_vec(),
                record.to_json(),
            ));
        }
    }
    played
}

#[test]
fn c51_holding_never_changes_what_a_table_plays() {
    let plain = play_with_holds(500, false);
    // With a third of the steps held, 750 steps move a table as far as 500
    // steps do without holds; 900 is some more.
    let held = play_with_holds(900, true);
    assert_eq!(plain.len(), 200);
    let mut compared = 0;
    for (table, (without, with)) in plain.iter().zip(&held).enumerate() {
        // The deals both runs finished at this table: the same actions,
        // the same entries, the same rewards, the same records, in the
        // same order.
        let both = without.len().min(with.len());
        assert!(both >= 2, "table {table}: {both} deals in both runs");
        assert_eq!(without[..both], with[..both], "table {table}");
        compared += both;
    }
    assert!(compared >= 800, "{compared} deals compared");
    // The run with holds did wait: it needed more steps for its deals.
    let deals = |played: &[Vec<PlayedDeal>]| played.iter().map(Vec::len).sum::<usize>();
    assert!(deals(&held) * 500 < deals(&plain) * 900);
}
