//! Spec 006, criteria 15 to 19: the vectorised environment.
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
    // announcements (spec 009) a random deal has about 94 decisions, not
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
    // random deal with the round of announcements (spec 009) has about 94
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
            for seat in 0..4 {
                let of_seat = &parts[seat * 10..(seat + 1) * 10];
                assert_eq!(of_seat, expected.parts[seat]);
                // The ten parts add up to the seat's reward.
                let sum: f32 = of_seat.iter().sum();
                assert!(
                    (sum - reward[seat]).abs() < 1e-6,
                    "{sum} against {}",
                    reward[seat]
                );
                // Klop and radelc: zero until flow C.
                assert_eq!(of_seat[8..], [0.0, 0.0]);
            }
            let total: f32 = reward.iter().sum();
            assert!(total.abs() < 1e-6);
            taken[table].clear();
        }
    }
    // About 94 decisions per random deal since spec 009: 2,000 steps at
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
    // About 94 decisions per random deal since spec 009.
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
