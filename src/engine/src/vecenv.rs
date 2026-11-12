//! Many tables stepped at once: the environment the learner trains on.
//!
//! Every table plays deal after deal. One step is one decision per table:
//! the seat to act at each table takes one action. When a deal ends, its
//! rewards are delivered with that step and the table starts its next deal
//! at once, so there is a decision at every table at every step.
//!
//! **Runs.** A table plays runs of 24 deals (D117). Inside a run the dealer
//! moves one seat per deal. The first dealer of run `r` at table `t` is
//! seat `(t + r) mod 4`.
//!
//! **Seeds.** The cards of a deal depend on four numbers only: the base
//! seed of the environment, the table, the run and the deal's number in
//! the run (D120). [`deal_seed`] mixes them into the seed of
//! [`Deal::from_seed`](crate::Deal::from_seed), so any deal can be dealt
//! again from those four numbers.
//!
//! **Threads.** The tables are independent and each writes its own rows of
//! the arrays, so the result is the same for any number of threads.
//!
//! The arrays are those of `docs/encoding.md`, one row per table, for the
//! seat to act (see [`Observations`]).

use std::fmt;

use rayon::prelude::*;

use crate::action::Action;
use crate::deal::Seat;
use crate::encode::{
    self, CRITIC_FEATURES, MAX_TOKENS, N_ACTIONS, N_SCORE_PARTS, STATE_FEATURES, TOKEN_FIELDS,
};
use crate::game::{Game, MAX_ACTIONS, Phase};

/// Deals in a run of a table.
pub const DEALS_PER_RUN: usize = 24;

const TOKEN_ROW: usize = MAX_TOKENS * TOKEN_FIELDS;
const PARTS_ROW: usize = 4 * N_SCORE_PARTS;
/// Tables handed to a thread at a time: enough work to be worth the hand-over.
const TABLES_PER_TASK: usize = 16;

/// SplitMix64's finishing step: every input bit changes about half of the
/// output bits.
const fn mix(mut z: u64) -> u64 {
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// The seed of deal number `deal` of run `run` at table `table`, for an
/// environment with this base seed. Frozen by a golden test: changing it
/// changes every stored position.
///
/// The four numbers are mixed in one after the other; each step adds a
/// constant so that zeros do not stay zeros.
///
/// Reading B12-8 (Batch 12, open): this mixing function.
pub const fn deal_seed(base_seed: u64, table: u64, run: u64, deal: u64) -> u64 {
    const STEP: u64 = 0x9E37_79B9_7F4A_7C15;
    let mut seed = mix(base_seed.wrapping_add(STEP));
    seed = mix(seed ^ table.wrapping_add(STEP.wrapping_mul(2)));
    seed = mix(seed ^ run.wrapping_add(STEP.wrapping_mul(3)));
    mix(seed ^ deal.wrapping_add(STEP.wrapping_mul(4)))
}

/// The dealer of the first deal of a run: seat `(table + run) mod 4`.
///
/// Reading B12-8 (Batch 12, open): this rule.
pub const fn first_dealer(table: u64, run: u64) -> Seat {
    Seat::ALL[((table % 4 + run % 4) % 4) as usize]
}

/// Why the environment refused a call.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VecEnvError {
    /// An environment needs at least one table.
    NoTables,
    /// An environment needs at least one thread.
    NoThreads,
    /// `step` got another number of actions than there are tables.
    WrongActionCount { expected: usize, got: usize },
    /// The action of a table is not legal there. No table has moved.
    IllegalAction { table: usize, slot: i64 },
    /// A state given to `set_state` does not fit this environment.
    BadState(String),
    /// An array handed in does not hold one row per table.
    BufferSize {
        name: &'static str,
        expected: usize,
        got: usize,
    },
    /// The thread pool could not be started.
    Threads(String),
}

impl fmt::Display for VecEnvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VecEnvError::NoTables => write!(f, "an environment needs at least one table"),
            VecEnvError::NoThreads => write!(f, "an environment needs at least one thread"),
            VecEnvError::WrongActionCount { expected, got } => {
                write!(f, "{got} actions for {expected} tables")
            }
            VecEnvError::IllegalAction { table, slot } => {
                write!(f, "slot {slot} is not a legal action at table {table}")
            }
            VecEnvError::BadState(why) => write!(f, "the state does not fit: {why}"),
            VecEnvError::BufferSize {
                name,
                expected,
                got,
            } => write!(f, "the array {name} has {got} entries, not {expected}"),
            VecEnvError::Threads(why) => write!(f, "the thread pool did not start: {why}"),
        }
    }
}

impl std::error::Error for VecEnvError {}

/// The arrays of one step, one row per table, for the seat to act there.
/// Flat vectors; the row of table `t` starts at `t * width`.
#[derive(Clone, Debug, PartialEq)]
pub struct Observations {
    /// The seat to act, 0 to 3.
    pub seat: Vec<i64>,
    /// How many deals the table has finished: it changes when a new deal
    /// starts.
    pub deal: Vec<i64>,
    /// 128 x 5 per table: event, relative seat, trick, position, index.
    /// Zero after the last token.
    pub tokens: Vec<i16>,
    /// The number of tokens.
    pub n_tokens: Vec<i64>,
    /// 1,382 per table.
    pub state: Vec<f32>,
    /// 160 per table.
    pub mask: Vec<bool>,
    /// 1,656 per table: the state, then the 274 hidden features.
    pub critic: Vec<f32>,
    /// The relative seat of the one team-mate, 1 to 3, or 0 for none.
    pub partner_target: Vec<i64>,
    /// 54 per table: 1 to 3 if that other seat holds the card, else 0.
    pub card_target: Vec<i64>,
}

/// What a step gives back, one row per table.
#[derive(Clone, Debug, PartialEq)]
pub struct StepResult {
    /// True where the deal ended with this step.
    pub done: Vec<bool>,
    /// 4 per table, by seat number: the rewards of a deal that ended, zero
    /// elsewhere.
    pub reward: Vec<f32>,
    /// 4 x 10 per table: the rewards by score part; zero where the deal
    /// goes on. The ten parts of a seat add up to its reward.
    pub parts: Vec<f32>,
    /// 4 per table: the entries the deal wrote on paper; zero where the
    /// deal goes on.
    pub entries: Vec<i32>,
}

/// Everything needed to go on from here in another environment with the
/// same base seed: where each table stands and the actions of its current
/// deal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnvState {
    pub base_seed: u64,
    /// The run each table is in.
    pub run: Vec<u64>,
    /// The number of the current deal in its run, 0 to 23.
    pub deal_in_run: Vec<u32>,
    /// How many deals each table has finished.
    pub deals_done: Vec<u64>,
    /// How many actions the current deal of each table has had.
    pub n_actions: Vec<u32>,
    /// The slots of those actions: `MAX_ACTIONS` per table, zero after the
    /// last one.
    pub actions: Vec<u8>,
}

/// One table: where it stands in its runs, and the deal it is playing.
#[derive(Clone, Debug)]
struct Table {
    index: u64,
    run: u64,
    deal_in_run: u32,
    deals_done: u64,
    game: Game,
}

impl Table {
    fn dealer(&self) -> Seat {
        first_dealer(self.index, self.run).plus(self.deal_in_run as usize)
    }

    fn deal_out(&mut self, base_seed: u64) {
        let seed = deal_seed(base_seed, self.index, self.run, u64::from(self.deal_in_run));
        self.game = Game::from_seed(seed, self.dealer());
    }

    /// Moves on to the next deal, or to the first deal of the next run.
    fn next_deal(&mut self, base_seed: u64) {
        self.deals_done += 1;
        self.deal_in_run += 1;
        if self.deal_in_run as usize == DEALS_PER_RUN {
            self.deal_in_run = 0;
            self.run += 1;
        }
        self.deal_out(base_seed);
    }
}

/// The observation arrays of one step as slices to write into: the same
/// layout as [`Observations`], in memory owned by the caller (the Python
/// binding hands in NumPy arrays).
pub struct ObservationBuffers<'a> {
    pub seat: &'a mut [i64],
    pub deal: &'a mut [i64],
    pub tokens: &'a mut [i16],
    pub n_tokens: &'a mut [i64],
    pub state: &'a mut [f32],
    pub mask: &'a mut [bool],
    pub critic: &'a mut [f32],
    pub partner_target: &'a mut [i64],
    pub card_target: &'a mut [i64],
}

/// The result arrays of one step as slices to write into: the same layout
/// as [`StepResult`].
pub struct ResultBuffers<'a> {
    pub done: &'a mut [bool],
    pub reward: &'a mut [f32],
    pub parts: &'a mut [f32],
    pub entries: &'a mut [i32],
}

impl ObservationBuffers<'_> {
    /// Every slice must hold exactly one row per table.
    fn check(&self, tables: usize) -> Result<(), VecEnvError> {
        let sizes = [
            ("seat", self.seat.len(), 1),
            ("deal", self.deal.len(), 1),
            ("tokens", self.tokens.len(), TOKEN_ROW),
            ("n_tokens", self.n_tokens.len(), 1),
            ("state", self.state.len(), STATE_FEATURES),
            ("mask", self.mask.len(), N_ACTIONS),
            ("critic", self.critic.len(), CRITIC_FEATURES),
            ("partner_target", self.partner_target.len(), 1),
            ("card_target", self.card_target.len(), 54),
        ];
        check_sizes(&sizes, tables)
    }
}

impl ResultBuffers<'_> {
    fn check(&self, tables: usize) -> Result<(), VecEnvError> {
        let sizes = [
            ("done", self.done.len(), 1),
            ("reward", self.reward.len(), 4),
            ("parts", self.parts.len(), PARTS_ROW),
            ("entries", self.entries.len(), 4),
        ];
        check_sizes(&sizes, tables)
    }
}

fn check_sizes(sizes: &[(&'static str, usize, usize)], tables: usize) -> Result<(), VecEnvError> {
    for &(name, got, width) in sizes {
        let expected = tables * width;
        if got != expected {
            return Err(VecEnvError::BufferSize {
                name,
                expected,
                got,
            });
        }
    }
    Ok(())
}

/// The rows of one table in the observation arrays.
struct ObservationRow<'a> {
    seat: &'a mut i64,
    deal: &'a mut i64,
    tokens: &'a mut [i16],
    n_tokens: &'a mut i64,
    state: &'a mut [f32],
    mask: &'a mut [bool],
    critic: &'a mut [f32],
    partner_target: &'a mut i64,
    card_target: &'a mut [i64],
}

/// Writes the observation of the seat to act at a table.
fn encode_table(table: &Table, row: ObservationRow<'_>) {
    let game = &table.game;
    // A table is never left in a finished deal, so a seat is to act. The
    // dealer stands in only to keep this free of a panic path.
    let seat = game.to_act().unwrap_or(table.dealer());
    *row.seat = seat.index() as i64;
    *row.deal = table.deals_done as i64;
    *row.n_tokens = encode::write_tokens(game, seat, row.tokens) as i64;
    encode::write_state(game, seat, row.state);
    encode::write_mask(game, seat, row.mask);
    let (state_part, extra) = row.critic.split_at_mut(STATE_FEATURES);
    state_part.copy_from_slice(row.state);
    encode::write_critic_extra(game, seat, extra);
    let (partner, cards) = encode::targets(game, seat);
    *row.partner_target = i64::from(partner);
    for (target, place) in row.card_target.iter_mut().zip(cards) {
        *target = i64::from(place);
    }
}

/// What a run of random steps did. See [`Tables::run_random`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RandomRun {
    /// Decisions taken: steps times tables.
    pub decisions: u64,
    /// Deals finished.
    pub deals: u64,
}

/// N tables without arrays of their own: every call writes into slices the
/// caller hands in. [`VecEnv`] wraps this with arrays it owns.
pub struct Tables {
    base_seed: u64,
    threads: usize,
    pool: rayon::ThreadPool,
    tables: Vec<Table>,
}

impl fmt::Debug for Tables {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Tables")
            .field("tables", &self.tables.len())
            .field("base_seed", &self.base_seed)
            .field("threads", &self.threads)
            .finish()
    }
}

impl Tables {
    /// `tables` tables, each at the first deal of run 0, stepped on
    /// `threads` threads. The same base seed gives the same deals.
    pub fn new(tables: usize, base_seed: u64, threads: usize) -> Result<Tables, VecEnvError> {
        if tables == 0 {
            return Err(VecEnvError::NoTables);
        }
        if threads == 0 {
            return Err(VecEnvError::NoThreads);
        }
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .map_err(|error| VecEnvError::Threads(error.to_string()))?;
        let all = (0..tables as u64)
            .map(|index| Table {
                index,
                run: 0,
                deal_in_run: 0,
                deals_done: 0,
                game: Game::from_seed(deal_seed(base_seed, index, 0, 0), first_dealer(index, 0)),
            })
            .collect();
        Ok(Tables {
            base_seed,
            threads,
            pool,
            tables: all,
        })
    }

    pub fn len(&self) -> usize {
        self.tables.len()
    }

    /// Never true: an environment has at least one table.
    pub fn is_empty(&self) -> bool {
        self.tables.is_empty()
    }

    pub fn threads(&self) -> usize {
        self.threads
    }

    pub fn base_seed(&self) -> u64 {
        self.base_seed
    }

    /// Where a table stands: its run, the number of its deal in the run and
    /// the number of deals it has finished. `None` for a table that does
    /// not exist.
    pub fn position(&self, table: usize) -> Option<(u64, u32, u64)> {
        let table = self.tables.get(table)?;
        Some((table.run, table.deal_in_run, table.deals_done))
    }

    /// The deal a table is playing. For tests and tools: it shows hidden
    /// cards.
    pub fn game(&self, table: usize) -> Option<&Game> {
        self.tables.get(table).map(|table| &table.game)
    }

    /// Writes the observation of the seat to act at every table.
    pub fn observe_into(&self, out: ObservationBuffers<'_>) -> Result<(), VecEnvError> {
        out.check(self.tables.len())?;
        let tables = &self.tables;
        self.pool.install(|| {
            (
                tables.par_iter(),
                out.seat.par_iter_mut(),
                out.deal.par_iter_mut(),
                out.tokens.par_chunks_mut(TOKEN_ROW),
                out.n_tokens.par_iter_mut(),
                out.state.par_chunks_mut(STATE_FEATURES),
                out.mask.par_chunks_mut(N_ACTIONS),
                out.critic.par_chunks_mut(CRITIC_FEATURES),
                out.partner_target.par_iter_mut(),
                out.card_target.par_chunks_mut(54),
            )
                .into_par_iter()
                .with_min_len(TABLES_PER_TASK)
                .for_each(
                    |(
                        table,
                        seat,
                        deal,
                        tokens,
                        n_tokens,
                        state,
                        mask,
                        critic,
                        partner_target,
                        card_target,
                    )| {
                        encode_table(
                            table,
                            ObservationRow {
                                seat,
                                deal,
                                tokens,
                                n_tokens,
                                state,
                                mask,
                                critic,
                                partner_target,
                                card_target,
                            },
                        );
                    },
                );
        });
        Ok(())
    }

    /// One action per table, as a slot number. Every table moves, or none:
    /// if one action is not legal, the call fails and nothing changes.
    ///
    /// Where a deal ends, `result` gets its rewards and the table starts
    /// its next deal at once. `observations` gets the next decision of
    /// every table.
    pub fn step_into(
        &mut self,
        actions: &[i64],
        observations: ObservationBuffers<'_>,
        result: ResultBuffers<'_>,
    ) -> Result<(), VecEnvError> {
        if actions.len() != self.tables.len() {
            return Err(VecEnvError::WrongActionCount {
                expected: self.tables.len(),
                got: actions.len(),
            });
        }
        observations.check(self.tables.len())?;
        result.check(self.tables.len())?;
        // Check every action first.
        for (index, (table, &slot)) in self.tables.iter().zip(actions).enumerate() {
            let legal = usize::try_from(slot)
                .is_ok_and(|slot| table.game.legal_actions().contains_slot(slot));
            if !legal {
                return Err(VecEnvError::IllegalAction { table: index, slot });
            }
        }
        self.advance(
            |table| {
                usize::try_from(actions[table.index as usize])
                    .ok()
                    .and_then(Action::from_slot)
            },
            observations,
            result,
        );
        Ok(())
    }

    /// `steps` steps with a random legal action at every table, chosen in
    /// Rust: the speed of the environment alone, and a way to move many
    /// tables on. The choices depend on `seed`, the table and the step, not
    /// on the number of threads. After the call the buffers hold the last
    /// step.
    pub fn run_random(
        &mut self,
        steps: u64,
        seed: u64,
        observations: ObservationBuffers<'_>,
        result: ResultBuffers<'_>,
    ) -> Result<RandomRun, VecEnvError> {
        observations.check(self.tables.len())?;
        result.check(self.tables.len())?;
        let mut deals = 0;
        for step in 0..steps {
            let salt = mix(seed ^ mix(step.wrapping_add(1)));
            self.advance(
                |table| {
                    let legal = table.game.legal_actions();
                    let pick = mix(salt ^ table.index.wrapping_mul(0x9E37_79B9_7F4A_7C15));
                    legal.nth((pick % u64::from(legal.len().max(1))) as u32)
                },
                ObservationBuffers {
                    seat: &mut *observations.seat,
                    deal: &mut *observations.deal,
                    tokens: &mut *observations.tokens,
                    n_tokens: &mut *observations.n_tokens,
                    state: &mut *observations.state,
                    mask: &mut *observations.mask,
                    critic: &mut *observations.critic,
                    partner_target: &mut *observations.partner_target,
                    card_target: &mut *observations.card_target,
                },
                ResultBuffers {
                    done: &mut *result.done,
                    reward: &mut *result.reward,
                    parts: &mut *result.parts,
                    entries: &mut *result.entries,
                },
            );
            deals += result.done.iter().filter(|&&done| done).count() as u64;
        }
        Ok(RandomRun {
            decisions: steps * self.tables.len() as u64,
            deals,
        })
    }

    /// Moves every table by the action `choose` gives for it, delivers the
    /// rewards of the deals that end, starts their next deals and writes
    /// the next observations. The buffers were checked by the caller.
    fn advance(
        &mut self,
        choose: impl Fn(&Table) -> Option<Action> + Sync,
        observations: ObservationBuffers<'_>,
        result: ResultBuffers<'_>,
    ) {
        let base_seed = self.base_seed;
        let tables = &mut self.tables;
        self.pool.install(|| {
            let rows = (
                tables.par_iter_mut(),
                observations.seat.par_iter_mut(),
                observations.deal.par_iter_mut(),
                observations.tokens.par_chunks_mut(TOKEN_ROW),
                observations.n_tokens.par_iter_mut(),
                observations.state.par_chunks_mut(STATE_FEATURES),
                observations.mask.par_chunks_mut(N_ACTIONS),
                observations.critic.par_chunks_mut(CRITIC_FEATURES),
                observations.partner_target.par_iter_mut(),
                observations.card_target.par_chunks_mut(54),
            );
            let results = (
                result.done.par_iter_mut(),
                result.reward.par_chunks_mut(4),
                result.parts.par_chunks_mut(PARTS_ROW),
                result.entries.par_chunks_mut(4),
            );
            rows.into_par_iter()
                .zip(results)
                .with_min_len(TABLES_PER_TASK)
                .for_each(|(row, (done, reward, parts, entries))| {
                    let (
                        table,
                        seat,
                        deal,
                        tokens,
                        n_tokens,
                        state,
                        mask,
                        critic,
                        partner_target,
                        card_target,
                    ) = row;
                    // The action is a legal one: checked by `step_into`, or
                    // drawn from the legal set.
                    if let Some(action) = choose(table) {
                        let applied = table.game.apply(action);
                        debug_assert!(applied.is_ok());
                    }
                    *done = table.game.phase() == Phase::Over;
                    reward.fill(0.0);
                    parts.fill(0.0);
                    entries.fill(0);
                    if *done {
                        if let Some(score) = table.game.score() {
                            let rewards = encode::rewards(&score);
                            reward.copy_from_slice(&rewards.reward);
                            entries.copy_from_slice(&rewards.entries);
                            for (out, row) in
                                parts.chunks_exact_mut(N_SCORE_PARTS).zip(rewards.parts)
                            {
                                out.copy_from_slice(&row);
                            }
                        }
                        table.next_deal(base_seed);
                    }
                    encode_table(
                        table,
                        ObservationRow {
                            seat,
                            deal,
                            tokens,
                            n_tokens,
                            state,
                            mask,
                            critic,
                            partner_target,
                            card_target,
                        },
                    );
                });
        });
    }

    /// Where every table stands, to go on from here later.
    pub fn state(&self) -> EnvState {
        let n = self.tables.len();
        let mut state = EnvState {
            base_seed: self.base_seed,
            run: Vec::with_capacity(n),
            deal_in_run: Vec::with_capacity(n),
            deals_done: Vec::with_capacity(n),
            n_actions: Vec::with_capacity(n),
            actions: vec![0; n * MAX_ACTIONS],
        };
        for (table, row) in self
            .tables
            .iter()
            .zip(state.actions.chunks_exact_mut(MAX_ACTIONS))
        {
            let full = table.game.full();
            let slots = full.action_slots();
            state.run.push(table.run);
            state.deal_in_run.push(table.deal_in_run);
            state.deals_done.push(table.deals_done);
            state.n_actions.push(slots.len() as u32);
            row[..slots.len()].copy_from_slice(slots);
        }
        state
    }

    /// Puts every table where the state says. Each deal is dealt again from
    /// its seed and its actions are replayed. If the state does not fit
    /// (another base seed, another number of tables, an action that is not
    /// legal, a count of finished deals that contradicts the run and the
    /// deal), the call fails and the tables stay as they were.
    pub fn set_state(&mut self, state: &EnvState) -> Result<(), VecEnvError> {
        let n = self.tables.len();
        let bad = |why: String| Err(VecEnvError::BadState(why));
        if state.base_seed != self.base_seed {
            return bad(format!(
                "it was made with base seed {}, this environment has {}",
                state.base_seed, self.base_seed
            ));
        }
        let lengths = [
            state.run.len(),
            state.deal_in_run.len(),
            state.deals_done.len(),
            state.n_actions.len(),
        ];
        if lengths != [n; 4] || state.actions.len() != n * MAX_ACTIONS {
            return bad(format!("it is not for {n} tables"));
        }

        let mut tables = Vec::with_capacity(n);
        for index in 0..n {
            let deal_in_run = state.deal_in_run[index];
            let count = state.n_actions[index] as usize;
            if deal_in_run as usize >= DEALS_PER_RUN || count > MAX_ACTIONS {
                return bad(format!("table {index}: a number is out of range"));
            }
            // The deals a table has finished follow from where it stands:
            // 24 per run and the deals of the current run. The count goes
            // into the `deal` array as a signed 64-bit number.
            let run = state.run[index];
            let finished = run
                .checked_mul(DEALS_PER_RUN as u64)
                .and_then(|deals| deals.checked_add(u64::from(deal_in_run)))
                .filter(|&deals| i64::try_from(deals).is_ok());
            let Some(finished) = finished else {
                return bad(format!("table {index}: run {run} is out of range"));
            };
            if state.deals_done[index] != finished {
                return bad(format!(
                    "table {index}: {} deals done does not fit run {run}, deal {deal_in_run} \
                     (that is {finished})",
                    state.deals_done[index]
                ));
            }
            let mut table = Table {
                index: index as u64,
                run: state.run[index],
                deal_in_run,
                deals_done: state.deals_done[index],
                game: Game::from_seed(0, Seat::ALL[0]),
            };
            table.deal_out(self.base_seed);
            let slots = &state.actions[index * MAX_ACTIONS..index * MAX_ACTIONS + count];
            for (number, &slot) in slots.iter().enumerate() {
                let applied = Action::from_slot(usize::from(slot))
                    .ok_or(())
                    .and_then(|action| table.game.apply(action).map_err(|_| ()));
                if applied.is_err() {
                    return bad(format!(
                        "table {index}: action {number} (slot {slot}) is not legal in its deal"
                    ));
                }
            }
            if table.game.phase() == Phase::Over {
                return bad(format!("table {index}: its deal is already over"));
            }
            tables.push(table);
        }
        self.tables = tables;
        Ok(())
    }
}

/// N tables and their arrays: [`Tables`] with buffers of its own.
#[derive(Debug)]
pub struct VecEnv {
    tables: Tables,
    observations: Observations,
    result: StepResult,
}

impl Observations {
    fn zeros(tables: usize) -> Observations {
        Observations {
            seat: vec![0; tables],
            deal: vec![0; tables],
            tokens: vec![0; tables * TOKEN_ROW],
            n_tokens: vec![0; tables],
            state: vec![0.0; tables * STATE_FEATURES],
            mask: vec![false; tables * N_ACTIONS],
            critic: vec![0.0; tables * CRITIC_FEATURES],
            partner_target: vec![0; tables],
            card_target: vec![0; tables * 54],
        }
    }

    fn buffers(&mut self) -> ObservationBuffers<'_> {
        ObservationBuffers {
            seat: &mut self.seat,
            deal: &mut self.deal,
            tokens: &mut self.tokens,
            n_tokens: &mut self.n_tokens,
            state: &mut self.state,
            mask: &mut self.mask,
            critic: &mut self.critic,
            partner_target: &mut self.partner_target,
            card_target: &mut self.card_target,
        }
    }
}

impl StepResult {
    fn zeros(tables: usize) -> StepResult {
        StepResult {
            done: vec![false; tables],
            reward: vec![0.0; tables * 4],
            parts: vec![0.0; tables * PARTS_ROW],
            entries: vec![0; tables * 4],
        }
    }

    fn buffers(&mut self) -> ResultBuffers<'_> {
        ResultBuffers {
            done: &mut self.done,
            reward: &mut self.reward,
            parts: &mut self.parts,
            entries: &mut self.entries,
        }
    }
}

impl VecEnv {
    /// `tables` tables, each at the first deal of run 0, stepped on
    /// `threads` threads. The same base seed gives the same deals.
    pub fn new(tables: usize, base_seed: u64, threads: usize) -> Result<VecEnv, VecEnvError> {
        let all = Tables::new(tables, base_seed, threads)?;
        let mut observations = Observations::zeros(tables);
        all.observe_into(observations.buffers())?;
        Ok(VecEnv {
            tables: all,
            observations,
            result: StepResult::zeros(tables),
        })
    }

    pub fn tables(&self) -> usize {
        self.tables.len()
    }

    pub fn threads(&self) -> usize {
        self.tables.threads()
    }

    pub fn base_seed(&self) -> u64 {
        self.tables.base_seed()
    }

    /// See [`Tables::position`].
    pub fn position(&self, table: usize) -> Option<(u64, u32, u64)> {
        self.tables.position(table)
    }

    /// See [`Tables::game`].
    pub fn game(&self, table: usize) -> Option<&Game> {
        self.tables.game(table)
    }

    /// The arrays of the seat to act at every table. They stay as they are
    /// until the next step.
    pub fn observe(&self) -> &Observations {
        &self.observations
    }

    /// One action per table; see [`Tables::step_into`]. On an error nothing
    /// changes.
    pub fn step(&mut self, actions: &[i64]) -> Result<&StepResult, VecEnvError> {
        self.tables
            .step_into(actions, self.observations.buffers(), self.result.buffers())?;
        Ok(&self.result)
    }

    /// `steps` random steps; see [`Tables::run_random`].
    pub fn run_random(&mut self, steps: u64, seed: u64) -> Result<RandomRun, VecEnvError> {
        self.tables.run_random(
            steps,
            seed,
            self.observations.buffers(),
            self.result.buffers(),
        )
    }

    /// Where every table stands, to go on from here later.
    pub fn state(&self) -> EnvState {
        self.tables.state()
    }

    /// See [`Tables::set_state`]. On an error nothing changes.
    pub fn set_state(&mut self, state: &EnvState) -> Result<(), VecEnvError> {
        self.tables.set_state(state)?;
        self.result = StepResult::zeros(self.tables.len());
        self.tables.observe_into(self.observations.buffers())
    }
}
