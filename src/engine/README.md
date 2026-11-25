# src/engine

The Rust crate `tarok-engine`: cards, deals, rules of four-player Slovenian tarok, scoring, seeded deals, game records and the encoding the network sees. It is built natively for training and tests and as WASM for the browser.

Built so far:

- Spec 002: cards, card sets, card points, the seeded deal, the trick winner and legal cards for normal contracts, the card play of one deal.
- Spec 005 (flow A): a whole deal of the six normal contracts. Bidding, choice of the contract, king call, talon exchange, twelve tricks, score with its parts.
- Spec 013: four rulings of the user's table, see "Rules of the user's table" below. The crate version went to 0.2.0 with them.
- Spec 006: the encoding of `docs/encoding.md` version 1 (tokens, state features with the exact inference bits, action mask, critic features, targets, reward), a vectorised environment that steps many tables on a thread pool, and the JSON game record with replay. The Python bindings are a separate crate, `src/train/tarok_env`; this crate has no Python dependency.

- Spec 009 (flow B): the round of announcements and kontra between the talon exchange and the first lead, its scoring, its tokens and state features, and the replay of a record of another engine version. See "The round of announcements" below. The crate version went to 0.3.0 with it.

Not yet: klop, berač, solo brez, valat as a bid, radelci (spec 010). Their features stay zero and their action slots masked.

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
| `src/announce.rs` | `Bonus`, `Target`, `Round`: what was said in the round of announcements and whose turn it is. Who may say what is in `game.rs`. |
| `src/talon.rs` | Talon packets and the cards that may not be discarded. |
| `src/score.rs` | `Score`, `ScoreParts`: the entry of each seat and its parts. |
| `src/game.rs` | `Game`: the state machine of a whole deal, and its public, private and full views. |
| `src/bin/bench_play.rs` | Benchmark: random legal card play, card plays per second. |
| `src/encode/` | The encoding: `tokens.rs`, `state.rs` (blocks A to G), `inference.rs` (what a seat knows for certain, block F), `critic.rs` (hidden features, targets), `reward.rs`. The sizes are constants in `mod.rs`. |
| `src/vecenv.rs` | `Tables` and `VecEnv`: N tables, runs of 24 deals, deal seeds from (base seed, table, run, deal). A step can hold tables (`step_hold`): a held table does not move, so a trainer can make finished tables wait for the others (spec 009, D142). Needs the feature `vecenv` (on by default). |
| `src/record.rs` | `Record`: the JSON game record, export and replay. |
| `src/bin/bench_game.rs` | Benchmark: whole deals with random legal actions, deals per second. |
| `examples/print_deal.rs` | Prints the deal for a seed. CI compares it with `tests/golden_deal_reference.py`. |
| `examples/record_deal.rs` | Prints the JSON record of a deal played with the lowest legal actions. |
| `tests/` | Tests named by acceptance criterion; the name starts with the criterion number. |

Test files of spec 002: `cards_and_counting.rs`, `cardset.rs`, `deal.rs`, `trick_winner.rs`, `legal_cards.rs`, `whole_deals.rs`. Of spec 005: `bidding.rs`, `king_call.rs`, `talon.rs`, `game_play.rs`, `scoring.rs`, `written_deals.rs`, `whole_games.rs`, `actions.rs`. The tests of spec 013 are in `trick_winner.rs` (`c17`) and `scoring.rs` (`c39`, `c42`, the `d125_` tests, the `d133_` test). Of spec 006: `encoding_contract.rs` (reads `docs/encoding.md`), `tokens.rs`, `state_features.rs`, `inference.rs`, `no_leak.rs`, `critic_targets_reward.rs`, `vecenv.rs`, `record.rs`; the golden record is `tests/golden/record_seed_42.json`. Shared helpers are in `tests/common/`. Of spec 009: `announcements.rs` (criteria 1 to 10 and 17), `kontra.rs` (11 to 16), `scoring_b.rs` (18 to 32), `encode.rs` (34 to 38), and tests named `c33`, `c38` to `c44` and `c50` to `c51` in `written_deals.rs`, `no_leak.rs`, `record.rs`, `whole_games.rs` and `vecenv.rs`. The built deals both scoring files use are in `tests/common/built.rs`.

The tests play several hundred thousand deals. `Cargo.toml` turns optimisation on for test builds (`[profile.dev] opt-level = 2`); overflow checks and debug assertions stay on. The whole suite takes about 30 seconds.

## Rules of the user's table

Four points are played and scored as the user ruled on 2026-11-12 (spec 013). They differ from the rules as first built in specs 002 and 005.

- **Trula in one trick (D131, D132).** Škis, mond and pagat are in one trick. The pagat wins the trick only if it falls last: it was played after both the škis and the mond. A fourth card may still follow it. Otherwise the škis wins, as the highest tarok. The place is `pagat_falls_last` in `src/trick.rs`. This is the Default of toggle S40; the engine has no toggles yet.
- **Pagat ultimo follows the side (D126).** The pagat is played to the last trick. The side of the pagat's player wins the bonus when one of its seats takes the trick, with the pagat or with another card. The declarer's side writes +25 when it takes that trick and -25 when the opponents take it. Kralj ultimo works the same way.
- **A valat does not cancel a lost mond (D125).** One side takes all twelve tricks: the declarer's side writes +250 or -250 and no game, no difference and no other bonus. A player who lost the mond in a trick still writes -20 next to it, to a škis of the other side or of the own side. So does the declarer who left the mond in a packet not taken, when the opponents take all twelve.
- **In a valat the declarer wins the packets not taken (D133).** The declarer's side takes all twelve tricks: the talon cards the declarer did not take count for the declarer's side, from the last trick on. A mond among them is not lost: +250 and no -20. The place is `talon_rest_won` in `src/game.rs`, the same that hands over the rest of the talon when the declarer wins a trick with the called king.

The mond's player writes the -20 in a trula trick too, whoever wins it (S22 a).

A game record names the engine version. A record written by 0.1.0 can replay to other entries under 0.2.0, or stop at a decision, when one of these points came up in its deal. A record of 0.2.0 has no round of announcements and stops at the first card under 0.3.0. In both cases the error names both versions.

## The round of announcements (spec 009)

After the last discard and before the first lead, in the six normal contracts. The rules are pagat.com's with the answers of the user's table (D141).

- **Turns.** The declarer speaks first, then the seats in order of play. A turn is any number of announcements and kontras, one per action, and then "done". "Done" alone is a pass. The round ends when every seat has had a turn and the last three turns were passes. Forehand then leads.
- **Announcements.** Trula, kralji and valat by every seat, for its side. Kralj ultimo only by the seat that holds the called king, the declarer included. Pagat ultimo only by the seat that holds the pagat. Each of the five once in a deal, whoever says it.
- **Kontra.** Six targets: the game and each announced bonus. Four levels: kontra, re, sub, mort. The game belongs to the declarer's side, a bonus to the side of its announcer. Kontra and sub are said by the other side, re and mort by the owning side. A seat may raise a bonus only if it knows the announcer's side: the announcer is the declarer or the seat itself, the partner is public, the announcer is shown as an opponent, or the seat holds the called king.
- **Sides.** An announcement does not show its seat's side. What does show the sides is an action only a holder of the called king, or only the declarer's side, could take (D147, D148): kralj ultimo; re or mort on the game or on a bonus of the declarer by another seat than the declarer; a raise of a bonus of another seat than the declarer whose side is not public, while the partner is hidden. A holder that is not the declarer is the partner; a declarer who holds the king plays alone.
- **A seat shown as an opponent (D150, D151).** Kontra or sub on the game or on a bonus of the declarer shows the seat that says it as an opponent; the partner stays hidden. Re or mort on a bonus of a seat shown as an opponent, by another seat, does the same. Every seat may then raise a bonus of that seat for its side, and block D1 shows that seat's announcements on the opponents' side. When all three other seats are shown so, the declarer is publicly alone. `PublicView::side_of` gives what all know about a seat's side. The one place that says what an action shows is `Game::shows` in `src/game.rs`; the one place that says what a seat knows is `Game::side_known_to`.
- **The rule behind the sides (D151).** The engine shows every fact about the sides that every seat can work out from what was said and from who may say what, and no other. `Game::shows` asks two questions: could the seat take this action if it held the called king, and could it if it did not? Only the first: it holds the king. Only the second: it is an opponent. Both: nothing. The legal set of the round is one function of "holds the called king" (`round_actions_of`), so the two questions use the same rules the legal set uses. Private conclusions of one seat are not used.
- **No duty to keep the card.** A seat that announced pagat ultimo or kralj ultimo may play the card whenever the rules of play allow. Played before the last trick, it loses the bonus.
- **Score.** An announced bonus replaces the silent one of its kind: trula and kralji 20, kralj ultimo 20, pagat ultimo 50, for the owner if made, against the owner if not. The announced pagat ultimo is made when the pagat is played to the last trick and the owner's side takes that trick. An announced valat is 500 for or against its owner and nothing else is scored but a lost mond; a silent valat removes announced bonuses too. A kontra multiplies its own target by 2, 4, 8 or 16: game and difference together, or one announced bonus. Silent bonuses, the silent valat and the mondfang are never multiplied.
- **Observation.** One token per announcement and per kontra, none for "done". State block D holds the announcements by side and the kontra levels (`docs/encoding.md`, section 4b). No deal has more than 103 tokens; the limit is 128.
- **Record.** The decisions of the round are in the record like every other decision. A record of another engine version is replayed as far as its actions are legal here; if it does not fit, the error names both versions (D135).

The helpers of the flow A tests pass the round: every seat says "done". So those deals play as before.

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

The property tests use fixed seeds: every run tries the same cases, so a test passes or fails the same way each time. Each call of `run_cases` has its own seed, made from a constant, the name of the test file and the line of the call (`tests/common/mod.rs`). A failure prints the seed. To try new cases on every run:

```
set TAROK_TEST_SEED=clock
cargo test
```

To run every property test with one seed, for example a printed one:

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
