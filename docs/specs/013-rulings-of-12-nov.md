# 013 — Three rulings of 12 November in the engine

Status: **approved** (2026-11-12). The criteria restate the user's rulings D125, D126 and D131. One reading is open (C12) and is built in one marked place.

Tool and model: Claude Code 2.1.306, `claude-fable-5-2`. Session `engine-core`.

## Goal

The engine plays and scores three points the way the user's table does:

1. A valat does not cancel a lost mond (D125).
2. Pagat ultimo follows the side that takes the last trick (D126).
3. Škis, mond and pagat in one trick: the pagat wins only if it falls last (D131).

## In scope

`src/engine`: the trick winner, `score.rs`, their tests, the README. The crate version. Notes in specs 002 and 005 that point here. The golden record if its content changes.

## Out of scope

Toggles: the engine still has none and plays the Default (S40 a, S22 a). Announced bonuses (flow B). Klop and berač (flow C). The Python side, apart from rebuilding the module for the parity test.

## Acceptance criteria

**Valat and the mond (D125)**
1. One side takes all twelve tricks and a player lost the mond. The mond's player writes −20 next to the ±250, in each of the four cases: (i) the declarer's side takes all, an opponent loses the mond to the škis; (ii) the declarer's side takes all, one of the two loses the mond to the other's škis; (iii) the opponents take all, the declarer loses the mond or leaves it in a packet not taken; (iv) the opponents take all, one of them loses the mond to another opponent's škis.
2. In those deals the score parts are: valat ±250, mondfang −20 for the mond's player only, game, difference and the other bonuses zero.
3. A valat without a lost mond scores as before (criterion 41 of spec 005).

**Pagat ultimo (D126)**
4. The pagat is played to the last trick. The side of the pagat's player takes the trick, with the pagat or with another card: that side wins the bonus. The declarer's side writes +25 when the pagat's player is on it and −25 when the player is an opponent.
5. The other side takes the trick: the pagat's side loses the bonus. The declarer's side writes −25 when the pagat's player is on it and +25 when the player is an opponent.
6. Tested cases: the declarer's pagat wins; the partner's pagat taken by the declarer's higher tarok (+25); the declarer's pagat taken by the partner's tarok (+25); an opponent's pagat taken by another opponent (−25); the declarer side's pagat taken by an opponent (−25); an opponent's pagat taken by the declarer's side (+25); a solo contract, where the three opponents are one side.
7. The pagat is not in the last trick: 0, as before.

**Trula in one trick (D131)**
8. Škis, mond and pagat are in one trick. The pagat wins the trick when it was played after both the škis and the mond (reading C12 a, in one marked place). Otherwise the škis wins it, as the highest tarok.
9. Tested with every order of the three cards and the fourth card first, between and last; the fourth card a suit card and another tarok.
10. The mond's player writes −20 in a trula trick whoever wins it (S22 a stays; when the škis wins, it is the plain mondfang).

**Tests of the tests**
11. Each old rule is planted once and fails at least one test: a valat cancels the mondfang; the pagat itself must take the last trick; the pagat wins the trula in any order; the pagat wins only as the fourth card (the other reading of C12).

**Consistency**
12. Rewards still add up to zero per deal and the ten parts of a seat add up to its reward (the property tests of spec 006 pass unchanged).
13. The crate version is raised, because the rules changed and a game record names the engine version. The golden record is written again only if needed; what changed in it is listed.
14. `src/engine/README.md` states the three rules. Spec 002 (criterion 17) and spec 005 (criteria 39 and 41, reading 2) get one line each that points here. Their old text stays.

**Quality gates**
15. `cargo fmt --check`, `cargo clippy --locked --all-targets -- -D warnings`, `cargo test --locked` and `cargo test --locked --no-default-features` pass for the engine; format, lints and tests pass for the binding crate; after the module is built again, `pytest tests/test_spec006.py` passes.

## Checks

Criteria 1–10, 12: `cargo test`. Criterion 11: planted in a scratch copy outside the repo, table in Results. Criteria 13, 14: by reading. Criterion 15: the commands' output in Results.

## Files expected to change

`src/engine/src/` (trick winner, `score.rs`), `src/engine/tests/` (`trick_winner.rs`, `scoring.rs`, written deals if their entries change), `src/engine/Cargo.toml`, `Cargo.lock`, `src/engine/README.md`, `src/engine/tests/golden/` if needed, one line each in `docs/specs/002-engine-core.md` and `docs/specs/005-deal-flow-a.md`, this spec. `src/train/tarok_env/Cargo.lock` if the engine version is named there.

## Commits

One commit: `fix(engine): valat keeps the mondfang, pagat ultimo by side, trula only when the pagat falls last (spec 013)`. Made by the main session after the review pass and the user's read of the diff (D72, D109).

## Open points for the user

- **C12.** "Falls last": (a) after both the škis and the mond, a fourth card may follow, recommended and built; (b) only as the fourth card of the trick.

## Results

(empty until executed)
