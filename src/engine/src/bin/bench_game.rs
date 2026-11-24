//! Benchmark for spec 005, criterion 52: whole deals with random legal
//! actions on one thread. Prints deals per second.
//!
//! Run it in release mode from `src/engine`:
//!
//! ```text
//! cargo run --release --bin bench_game
//! cargo run --release --bin bench_game -- 2000000
//! ```
//!
//! The argument is the number of deals (default 500,000). Deal `i` is dealt
//! from seed `i` with seat `i mod 4` as the dealer and goes through the
//! bidding, the contract choice, the king call, the talon exchange, the
//! round of announcements and the twelve tricks. Every action is drawn
//! uniformly from the legal actions. The time covers the deal, the legal
//! sets, the choice, the actions and the score.
//!
//! After the timed run the same deals are played once more, not timed, to
//! count what spec 009 (criterion 46) asks for: decisions and tokens per
//! deal and per seat, and the share of the decisions that fall in the
//! round of announcements. At most 200,000 deals are counted.

#![forbid(unsafe_code)]

use std::hint::black_box;
use std::time::Instant;

use tarok_engine::{Game, Phase, Seat};

const DEFAULT_DEALS: u64 = 500_000;

/// SplitMix64: a small generator for the choices. The deal itself uses the
/// engine's own ChaCha8 shuffle.
struct SplitMix64(u64);

impl SplitMix64 {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A number in `0..n`. The small bias does not matter for a benchmark.
    fn below(&mut self, n: u32) -> u32 {
        (((self.next() >> 32) * u64::from(n)) >> 32) as u32
    }
}

fn main() {
    let deals = match std::env::args().nth(1) {
        None => DEFAULT_DEALS,
        Some(text) => match text.parse::<u64>() {
            Ok(deals) if deals > 0 => deals,
            _ => {
                eprintln!("usage: bench_game [deals]   (a whole number above 0)");
                std::process::exit(2);
            }
        },
    };

    let mut choices = SplitMix64(0x7A20_C0DE);
    let mut actions: u64 = 0;
    // Summed so the compiler cannot drop the work.
    let mut checksum: i64 = 0;

    let start = Instant::now();
    for seed in 0..deals {
        let dealer = Seat::ALL[(seed % 4) as usize];
        let mut game = Game::from_seed(black_box(seed), dealer);
        while game.phase() != Phase::Over {
            let legal = game.legal_actions();
            let action = legal
                .nth(choices.below(legal.len()))
                .expect("a legal action before the deal is over");
            game.apply(action).expect("a legal action is accepted");
            actions += 1;
        }
        let score = game.score().expect("a finished deal has a score");
        checksum += i64::from(score.entry(Seat::ALL[0]));
    }
    let seconds = start.elapsed().as_secs_f64();

    println!("deals:              {deals}");
    println!("actions:            {actions}");
    println!("seconds:            {seconds:.3}");
    println!("deals per second:   {:.0}", deals as f64 / seconds);
    println!("actions per second: {:.0}", actions as f64 / seconds);
    println!(
        "checksum (entries of seat 0, summed): {}",
        black_box(checksum)
    );

    count_decisions(deals.min(COUNTED_DEALS));
}

/// The most deals the second, counting run plays.
const COUNTED_DEALS: u64 = 200_000;

/// Plays the first `deals` deals again with the same choices and prints
/// how long they are.
fn count_decisions(deals: u64) {
    let mut choices = SplitMix64(0x7A20_C0DE);
    let (mut decisions, mut in_the_round, mut tokens) = (0u64, 0u64, 0u64);
    let (mut most_decisions, mut most_by_a_seat, mut most_tokens) = (0u64, 0u64, 0u64);
    for seed in 0..deals {
        let dealer = Seat::ALL[(seed % 4) as usize];
        let mut game = Game::from_seed(seed, dealer);
        let mut by_seat = [0u64; 4];
        while let Some(seat) = game.to_act() {
            let legal = game.legal_actions();
            let action = legal
                .nth(choices.below(legal.len()))
                .expect("a legal action before the deal is over");
            in_the_round += u64::from(game.phase() == Phase::Announcements);
            by_seat[seat.index()] += 1;
            game.apply(action).expect("a legal action is accepted");
        }
        let of_the_deal: u64 = by_seat.iter().sum();
        decisions += of_the_deal;
        most_decisions = most_decisions.max(of_the_deal);
        most_by_a_seat = most_by_a_seat.max(by_seat.into_iter().max().unwrap_or(0));
        // One token per public event; every seat sees the same number.
        let events = game.public().events().len() as u64;
        tokens += events;
        most_tokens = most_tokens.max(events);
    }
    let per_deal = |total: u64| total as f64 / deals as f64;
    println!();
    println!("counted on {deals} deals, not timed:");
    println!(
        "decisions per deal:      mean {:.1}, largest {most_decisions}",
        per_deal(decisions)
    );
    println!(
        "decisions per seat-deal: mean {:.1}, largest {most_by_a_seat}",
        per_deal(decisions) / 4.0
    );
    println!(
        "tokens per deal:         mean {:.1}, largest {most_tokens} (the limit is 128)",
        per_deal(tokens)
    );
    println!(
        "decisions in the round of announcements: {:.1} % of all",
        100.0 * in_the_round as f64 / decisions as f64
    );
}
