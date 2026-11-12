# src/engine

The Rust crate `tarok-engine`: cards, deals, rules of four-player Slovenian tarok, scoring, seeded deals, game records and the encoding the network sees. It is built natively for training and tests and as WASM for the browser.

Built so far:

- Spec 002: cards, card sets, card points, the seeded deal, the trick winner and legal cards for normal contracts, the card play of one deal.
- Spec 005 (flow A): a whole deal of the six normal contracts. Bidding, choice of the contract, king call, talon exchange, twelve tricks, score with its parts.
- Spec 006: the encoding of `docs/encoding.md` version 1 (tokens, state features with the exact inference bits, action mask, critic features, targets, reward), a vectorised environment that steps many tables on a thread pool, and the JSON game record with replay. The Python bindings are a separate crate, `src/train/tarok_env`; this crate has no Python dependency.

Not yet: announcements and kontra (spec 009); klop, berač, solo brez, valat as a bid, radelci (spec 010). Their features stay zero and their action slots masked.

## Layout

| File | Holds |
|---|---|
| `src/card.rs` | `Card`, `Suit`, `Rank`, `Face`. The card index order. |
| `src/cardset.rs` | `CardSet`, a 64-bit mask. |
| `src/points.rs` | Card values and the counting of a pile. |
| `src/deal.rs` | `Seat`, `Deal`, the seeded shuffle. |
| `src/trick.rs` | Trick winner and legal cards. |
| `src/play.rs` | `PlayState`: twelve tricks of one deal. |
| `src/contract.rs` | `Contract`: the ladder of twelve contracts, game values, packet sizes. |
| `src/action.rs` | `Action`, `ActionSet`, `Packet`. The 160 action slots of `docs/encoding.md`. |
| `src/bidding.rs` | `Bidding`: the auction. |
| `src/talon.rs` | Talon packets and the cards that may not be discarded. |
| `src/score.rs` | `Score`, `ScoreParts`: the entry of each seat and its parts. |
| `src/game.rs` | `Game`: the state machine of a whole deal, and its public, private and full views. |
| `src/bin/bench_play.rs` | Benchmark: random legal card play, card plays per second. |
| `src/encode/` | The encoding: `tokens.rs`, `state.rs` (blocks A to G), `inference.rs` (what a seat knows for certain, block F), `critic.rs` (hidden features, targets), `reward.rs`. The sizes are constants in `mod.rs`. |
| `src/vecenv.rs` | `Tables` and `VecEnv`: N tables, runs of 24 deals, deal seeds from (base seed, table, run, deal). Needs the feature `vecenv` (on by default). |
| `src/record.rs` | `Record`: the JSON game record, export and replay. |
| `src/bin/bench_game.rs` | Benchmark: whole deals with random legal actions, deals per second. |
| `examples/print_deal.rs` | Prints the deal for a seed. CI compares it with `tests/golden_deal_reference.py`. |
| `examples/record_deal.rs` | Prints the JSON record of a deal played with the lowest legal actions. |
| `tests/` | Tests named by acceptance criterion; the name starts with the criterion number. |

Test files of spec 002: `cards_and_counting.rs`, `cardset.rs`, `deal.rs`, `trick_winner.rs`, `legal_cards.rs`, `whole_deals.rs`. Of spec 005: `bidding.rs`, `king_call.rs`, `talon.rs`, `game_play.rs`, `scoring.rs`, `written_deals.rs`, `whole_games.rs`, `actions.rs`. Of spec 006: `encoding_contract.rs` (reads `docs/encoding.md`), `tokens.rs`, `state_features.rs`, `inference.rs`, `no_leak.rs`, `critic_targets_reward.rs`, `vecenv.rs`, `record.rs`; the golden record is `tests/golden/record_seed_42.json`. Shared helpers are in `tests/common/`.

The tests play several hundred thousand deals. `Cargo.toml` turns optimisation on for test builds (`[profile.dev] opt-level = 2`); overflow checks and debug assertions stay on. The whole suite takes about 30 seconds.

## One deal

```rust
use tarok_engine::{Game, Phase, Seat};

let mut game = Game::from_seed(7, Seat::ALL[3]); // seat 3 deals, seat 0 is forehand
while game.phase() != Phase::Over {
    let action = game.legal_actions().iter().next().unwrap();
    game.apply(action).unwrap();
}
let entries = game.score().unwrap().entries(); // what each seat writes
```

What a seat may know is split in three: `game.public()` is what all see, `game.private(seat)` adds that seat's hand and discards, `game.full()` has everything (the true partner, the hidden discards). A player or an actor network must not read `full()`.

## Commands

Run from `src\engine`.

```
cargo test
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo run --release --bin bench_play
cargo run --release --bin bench_game
cargo run --example record_deal -- 42 3
cargo test --no-default-features
```

The last line builds and tests the crate without the thread pool (`rayon`), as a WASM build will need: everything but `vecenv` and its test file.

The property tests take a new seed from the clock on every run. A failure prints the seed. To replay it:

```
set TAROK_TEST_SEED=<seed>
cargo test
```

## Compiler version

The crate needs Rust 1.94 or newer (`rust-version` in `Cargo.toml`). It was built and tested with 1.94.1 on `stable-x86_64-pc-windows-gnu`.

A root `rust-toolchain.toml` that pinned 1.94.1 is switched off (kept, untracked, as `rust-toolchain.toml.disabled`): it named no host, and on a PC whose rustup default host is MSVC it made rustup download a second toolchain. How the version is pinned is an open decision; see `docs/specs/002-engine-core.md`, "Note from the main session".

## Rules of the crate

- No code from other tarok engines (D9).
- No `unsafe` (`#![forbid(unsafe_code)]`).
- The shuffle is written in `src/deal.rs`. `rand_chacha` is pinned to an exact version. The deal for seed 0 is frozen in `tests/deal.rs`.
