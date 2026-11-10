//! Prints the deal for a seed: the four hands in index order, one per line,
//! then the talon in the order it lies.
//!
//! The output has the same form as `tests/golden_deal_reference.py`, so the
//! two can be compared line by line. The CI workflow does that.
//!
//! ```text
//! cargo run --example print_deal -- 0
//! ```

#![forbid(unsafe_code)]

use tarok_engine::{Card, Deal};

fn main() {
    let seed = match std::env::args().nth(1).map(|text| text.parse::<u64>()) {
        Some(Ok(seed)) => seed,
        _ => {
            eprintln!("usage: print_deal <seed>   (a whole number, 0 to 2^64 - 1)");
            std::process::exit(2);
        }
    };
    let deal = Deal::from_seed(seed);
    for hand in deal.hands() {
        let indices: Vec<u8> = hand.iter().map(Card::index).collect();
        println!("{indices:?}");
    }
    println!("{:?}", deal.talon().map(Card::index));
}
