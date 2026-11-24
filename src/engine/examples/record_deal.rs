//! Plays one deal with the lowest legal action at every decision and prints
//! its JSON record, laid out for reading. In the round of announcements
//! every seat passes.
//!
//! ```text
//! cargo run --example record_deal -- 42 3
//! ```
//!
//! The arguments are the seed of the cards and the dealer's seat (0 to 3).
//! `tests/golden/record_seed_42.json` is the output for `42 3`.

#![forbid(unsafe_code)]

use tarok_engine::record::Record;
use tarok_engine::{Action, Game, Phase, Seat};

fn main() {
    let mut arguments = std::env::args().skip(1);
    let seed = arguments.next().and_then(|text| text.parse::<u64>().ok());
    let dealer = arguments
        .next()
        .and_then(|text| text.parse::<u8>().ok())
        .and_then(Seat::new);
    let (Some(seed), Some(dealer)) = (seed, dealer) else {
        eprintln!("usage: record_deal <seed> <dealer 0-3>");
        std::process::exit(2);
    };

    let mut game = Game::from_seed(seed, dealer);
    while game.phase() != Phase::Over {
        let Some(mut action) = game.legal_actions().iter().next() else {
            break;
        };
        if game.phase() == Phase::Announcements {
            action = Action::Done;
        }
        if game.apply(action).is_err() {
            break;
        }
    }
    match Record::of_game(seed, &game) {
        Ok(record) => print!("{}", record.to_json_pretty()),
        Err(error) => {
            eprintln!("no record: {error}");
            std::process::exit(1);
        }
    }
}
