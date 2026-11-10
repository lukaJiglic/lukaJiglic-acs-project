//! Benchmark for spec 002, criterion 33: whole deals of random legal cards on
//! one thread. Prints card plays per second.
//!
//! Run it in release mode from `src/engine`:
//!
//! ```text
//! cargo run --release --bin bench_play
//! cargo run --release --bin bench_play -- 2000000
//! ```
//!
//! The argument is the number of deals (default 1,000,000). Each deal is
//! dealt from its seed (0, 1, 2, ...) and played for 48 cards. The time
//! covers the deal, the legal-card sets, the random choice and the plays.

#![forbid(unsafe_code)]

use std::hint::black_box;
use std::time::Instant;

use tarok_engine::{Deal, PlayState, Seat};

const DEFAULT_DEALS: u64 = 1_000_000;

/// SplitMix64: a small generator for the card choices. The deal itself uses
/// the engine's own ChaCha8 shuffle.
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
                eprintln!("usage: bench_play [deals]   (a whole number above 0)");
                std::process::exit(2);
            }
        },
    };

    let mut choices = SplitMix64(0x7A20_C0DE);
    let mut plays: u64 = 0;
    // Summed so the compiler cannot drop the work.
    let mut checksum: u64 = 0;

    let start = Instant::now();
    for seed in 0..deals {
        let deal = Deal::from_seed(black_box(seed));
        let mut state = PlayState::from_deal(&deal, Seat::ALL[(seed % 4) as usize]);
        while !state.is_over() {
            let legal = state.legal_cards();
            let card = legal
                .nth(choices.below(legal.len()))
                .expect("a legal card before the deal is over");
            state.play(card).expect("a legal card is accepted");
            plays += 1;
        }
        checksum += u64::from(state.points(Seat::ALL[0]));
    }
    let seconds = start.elapsed().as_secs_f64();

    println!("deals:                 {deals}");
    println!("card plays:            {plays}");
    println!("seconds:               {seconds:.3}");
    println!("card plays per second: {:.0}", plays as f64 / seconds);
    println!("deals per second:      {:.0}", deals as f64 / seconds);
    println!(
        "checksum (points of seat 0, summed): {}",
        black_box(checksum)
    );
}
