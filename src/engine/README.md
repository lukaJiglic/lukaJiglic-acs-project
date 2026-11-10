# src/engine

The Rust crate `tarok-engine`: cards, deals, rules of four-player Slovenian tarok, scoring, seeded deals, game records and the encoding the network sees. It is built natively for training and tests and as WASM for the browser.

Built so far (spec 002): cards, card sets, card points, the seeded deal, the trick winner and legal cards for normal contracts, and the card play of one deal. Not yet: bidding, king call, talon, contracts, scoring, klop and berač rules, bindings.

## Layout

| File | Holds |
|---|---|
| `src/card.rs` | `Card`, `Suit`, `Rank`, `Face`. The card index order. |
| `src/cardset.rs` | `CardSet`, a 64-bit mask. |
| `src/points.rs` | Card values and the counting of a pile. |
| `src/deal.rs` | `Seat`, `Deal`, the seeded shuffle. |
| `src/trick.rs` | Trick winner and legal cards. |
| `src/play.rs` | `PlayState`: twelve tricks of one deal. |
| `src/bin/bench_play.rs` | Benchmark: random legal play, card plays per second. |
| `examples/print_deal.rs` | Prints the deal for a seed. CI compares it with `tests/golden_deal_reference.py`. |
| `tests/` | Tests named by acceptance criterion; the name starts with the criterion number. |

## Commands

Run from `src\engine`.

```
cargo test
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo run --release --bin bench_play
```

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
