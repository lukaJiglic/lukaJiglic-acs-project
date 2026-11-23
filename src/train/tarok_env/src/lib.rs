//! `tarok_env`: the Python bindings of the tarok engine (spec 006).
//!
//! A thin layer. The rules, the encoding and the stepping of many tables are
//! in the `tarok-engine` crate (`src/engine`); this crate hands NumPy arrays
//! to it.
//!
//! The arrays belong to Python. [`Tables`] writes straight into them, so a
//! step copies nothing. `tarok_train/env.py` owns the arrays and gives the
//! learner the interface of the mock environment.
//!
//! **Shapes.** Every array must have exactly the documented shape and be
//! C-contiguous (the rows of one table next to each other). Anything else
//! is refused with a `ValueError` that names the array and both shapes.
//!
//! **Threads.** `observe_into`, `step_into` and `run_random` release the GIL
//! while the engine works; the other calls are short and keep it. While one
//! of the three runs, the engine writes into the arrays without the GIL.
//! No other thread may read, write or resize those arrays, or call into the
//! same `Tables` object, until the call returns: a reader would see rows of
//! two different steps. `tarok_train.env.EngineEnv` holds a lock around its
//! methods for this reason.

use numpy::ndarray::Dimension;
use numpy::{
    Element, IntoPyArray, PyArrayMethods, PyReadonlyArray, PyReadonlyArray1, PyReadonlyArray2,
    PyReadwriteArray, PyReadwriteArray1, PyReadwriteArray2, PyReadwriteArray3,
    PyUntypedArrayMethods,
};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use tarok_engine::game::MAX_ACTIONS;
use tarok_engine::record::Record;
use tarok_engine::vecenv::{self, EnvState, ObservationBuffers, ResultBuffers, VecEnvError};
use tarok_engine::{Action, Game, Phase, Seat, encode};

/// An engine error as a Python `ValueError`.
fn value_error(error: impl std::fmt::Display) -> PyErr {
    PyValueError::new_err(error.to_string())
}

fn env_error(error: VecEnvError) -> PyErr {
    value_error(error)
}

/// A shape the way NumPy prints it: `(4, 1382)`, `(4,)`.
fn shape_text(shape: &[usize]) -> String {
    let numbers: Vec<String> = shape.iter().map(usize::to_string).collect();
    match numbers.as_slice() {
        [one] => format!("({one},)"),
        _ => format!("({})", numbers.join(", ")),
    }
}

/// Refuses an array that does not have exactly the expected shape in C
/// order. The slice the engine fills is the array's memory as it lies, so a
/// Fortran-ordered array or one with its axes swapped would be filled with
/// the right numbers in the wrong places.
fn check_layout(
    name: &str,
    shape: &[usize],
    c_contiguous: bool,
    expected: &[usize],
) -> PyResult<()> {
    if shape != expected {
        return Err(value_error(format!(
            "the array {name} has shape {}; the engine needs {}",
            shape_text(shape),
            shape_text(expected)
        )));
    }
    if !c_contiguous {
        return Err(value_error(format!(
            "the array {name} of shape {} is not C-contiguous; the engine needs {} in C order",
            shape_text(shape),
            shape_text(expected)
        )));
    }
    Ok(())
}

/// The memory of an array the engine writes into.
fn writable<'a, T: Element, D: Dimension>(
    name: &str,
    array: &'a mut PyReadwriteArray<'_, T, D>,
    expected: &[usize],
) -> PyResult<&'a mut [T]> {
    check_layout(name, array.shape(), array.is_c_contiguous(), expected)?;
    Ok(array.as_slice_mut()?)
}

/// The memory of an array the engine reads.
fn readable<'a, T: Element, D: Dimension>(
    name: &str,
    array: &'a PyReadonlyArray<'_, T, D>,
    expected: &[usize],
) -> PyResult<&'a [T]> {
    check_layout(name, array.shape(), array.is_c_contiguous(), expected)?;
    Ok(array.as_slice()?)
}

/// The nine observation arrays of `n` tables as slices, each checked.
#[allow(clippy::too_many_arguments)]
fn observation_buffers<'a>(
    n: usize,
    seat: &'a mut PyReadwriteArray1<'_, i64>,
    deal: &'a mut PyReadwriteArray1<'_, i64>,
    tokens: &'a mut PyReadwriteArray3<'_, i16>,
    n_tokens: &'a mut PyReadwriteArray1<'_, i64>,
    state: &'a mut PyReadwriteArray2<'_, f32>,
    mask: &'a mut PyReadwriteArray2<'_, bool>,
    critic: &'a mut PyReadwriteArray2<'_, f32>,
    partner_target: &'a mut PyReadwriteArray1<'_, i64>,
    card_target: &'a mut PyReadwriteArray2<'_, i64>,
) -> PyResult<ObservationBuffers<'a>> {
    Ok(ObservationBuffers {
        seat: writable("seat", seat, &[n])?,
        deal: writable("deal", deal, &[n])?,
        tokens: writable(
            "tokens",
            tokens,
            &[n, encode::MAX_TOKENS, encode::TOKEN_FIELDS],
        )?,
        n_tokens: writable("n_tokens", n_tokens, &[n])?,
        state: writable("state", state, &[n, encode::STATE_FEATURES])?,
        mask: writable("mask", mask, &[n, encode::N_ACTIONS])?,
        critic: writable("critic", critic, &[n, encode::CRITIC_FEATURES])?,
        partner_target: writable("partner_target", partner_target, &[n])?,
        card_target: writable("card_target", card_target, &[n, 54])?,
    })
}

/// The four result arrays of `n` tables as slices, each checked.
fn result_buffers<'a>(
    n: usize,
    done: &'a mut PyReadwriteArray1<'_, bool>,
    reward: &'a mut PyReadwriteArray2<'_, f32>,
    parts: &'a mut PyReadwriteArray3<'_, f32>,
    entries: &'a mut PyReadwriteArray2<'_, i32>,
) -> PyResult<ResultBuffers<'a>> {
    Ok(ResultBuffers {
        done: writable("done", done, &[n])?,
        reward: writable("reward", reward, &[n, 4])?,
        parts: writable("parts", parts, &[n, 4, encode::N_SCORE_PARTS])?,
        entries: writable("entries", entries, &[n, 4])?,
    })
}

/// N tables of tarok. Every call writes into NumPy arrays handed in by the
/// caller; see `tarok_train/env.py` for their shapes. See the module text
/// for the rule about threads.
#[pyclass(module = "tarok_env")]
struct Tables {
    inner: vecenv::Tables,
}

#[pymethods]
impl Tables {
    /// `Tables(tables, seed, threads)`: that many tables, each at the first
    /// deal of its first run. The same seed gives the same deals.
    #[new]
    fn new(tables: usize, seed: u64, threads: usize) -> PyResult<Self> {
        let inner = vecenv::Tables::new(tables, seed, threads).map_err(env_error)?;
        Ok(Tables { inner })
    }

    #[getter]
    fn n_tables(&self) -> usize {
        self.inner.len()
    }

    #[getter]
    fn threads(&self) -> usize {
        self.inner.threads()
    }

    #[getter]
    fn base_seed(&self) -> u64 {
        self.inner.base_seed()
    }

    /// `(run, deal in the run, deals finished)` of a table.
    fn position(&self, table: usize) -> PyResult<(u64, u32, u64)> {
        self.inner
            .position(table)
            .ok_or_else(|| value_error(format!("there is no table {table}")))
    }

    /// Writes the observation of the seat to act at every table. The GIL
    /// is released while the engine writes.
    #[allow(clippy::too_many_arguments)]
    fn observe_into(
        &self,
        py: Python<'_>,
        mut seat: PyReadwriteArray1<'_, i64>,
        mut deal: PyReadwriteArray1<'_, i64>,
        mut tokens: PyReadwriteArray3<'_, i16>,
        mut n_tokens: PyReadwriteArray1<'_, i64>,
        mut state: PyReadwriteArray2<'_, f32>,
        mut mask: PyReadwriteArray2<'_, bool>,
        mut critic: PyReadwriteArray2<'_, f32>,
        mut partner_target: PyReadwriteArray1<'_, i64>,
        mut card_target: PyReadwriteArray2<'_, i64>,
    ) -> PyResult<()> {
        let buffers = observation_buffers(
            self.inner.len(),
            &mut seat,
            &mut deal,
            &mut tokens,
            &mut n_tokens,
            &mut state,
            &mut mask,
            &mut critic,
            &mut partner_target,
            &mut card_target,
        )?;
        let inner = &self.inner;
        py.detach(|| inner.observe_into(buffers)).map_err(env_error)
    }

    /// One action per table. Writes the next observations and, where a deal
    /// ended, its rewards. If an action is not legal, nothing changes and a
    /// `ValueError` is raised. The GIL is released while the tables step.
    #[allow(clippy::too_many_arguments)]
    fn step_into(
        &mut self,
        py: Python<'_>,
        actions: PyReadonlyArray1<'_, i64>,
        mut seat: PyReadwriteArray1<'_, i64>,
        mut deal: PyReadwriteArray1<'_, i64>,
        mut tokens: PyReadwriteArray3<'_, i16>,
        mut n_tokens: PyReadwriteArray1<'_, i64>,
        mut state: PyReadwriteArray2<'_, f32>,
        mut mask: PyReadwriteArray2<'_, bool>,
        mut critic: PyReadwriteArray2<'_, f32>,
        mut partner_target: PyReadwriteArray1<'_, i64>,
        mut card_target: PyReadwriteArray2<'_, i64>,
        mut done: PyReadwriteArray1<'_, bool>,
        mut reward: PyReadwriteArray2<'_, f32>,
        mut parts: PyReadwriteArray3<'_, f32>,
        mut entries: PyReadwriteArray2<'_, i32>,
    ) -> PyResult<()> {
        let n = self.inner.len();
        let actions = readable("actions", &actions, &[n])?;
        let observations = observation_buffers(
            n,
            &mut seat,
            &mut deal,
            &mut tokens,
            &mut n_tokens,
            &mut state,
            &mut mask,
            &mut critic,
            &mut partner_target,
            &mut card_target,
        )?;
        let result = result_buffers(n, &mut done, &mut reward, &mut parts, &mut entries)?;
        let inner = &mut self.inner;
        py.detach(|| inner.step_into(actions, observations, result))
            .map_err(env_error)
    }

    /// `steps` steps with a random legal action at every table, chosen in
    /// Rust, with the GIL released for the whole call. Returns
    /// `(decisions, deals finished)`. The arrays hold the last step. The
    /// call cannot be interrupted from Python before it returns.
    #[allow(clippy::too_many_arguments)]
    fn run_random(
        &mut self,
        py: Python<'_>,
        steps: u64,
        seed: u64,
        mut seat: PyReadwriteArray1<'_, i64>,
        mut deal: PyReadwriteArray1<'_, i64>,
        mut tokens: PyReadwriteArray3<'_, i16>,
        mut n_tokens: PyReadwriteArray1<'_, i64>,
        mut state: PyReadwriteArray2<'_, f32>,
        mut mask: PyReadwriteArray2<'_, bool>,
        mut critic: PyReadwriteArray2<'_, f32>,
        mut partner_target: PyReadwriteArray1<'_, i64>,
        mut card_target: PyReadwriteArray2<'_, i64>,
        mut done: PyReadwriteArray1<'_, bool>,
        mut reward: PyReadwriteArray2<'_, f32>,
        mut parts: PyReadwriteArray3<'_, f32>,
        mut entries: PyReadwriteArray2<'_, i32>,
    ) -> PyResult<(u64, u64)> {
        let n = self.inner.len();
        let observations = observation_buffers(
            n,
            &mut seat,
            &mut deal,
            &mut tokens,
            &mut n_tokens,
            &mut state,
            &mut mask,
            &mut critic,
            &mut partner_target,
            &mut card_target,
        )?;
        let result = result_buffers(n, &mut done, &mut reward, &mut parts, &mut entries)?;
        let inner = &mut self.inner;
        let run = py
            .detach(|| inner.run_random(steps, seed, observations, result))
            .map_err(env_error)?;
        Ok((run.decisions, run.deals))
    }

    /// Where every table stands, as a dict of NumPy arrays and plain
    /// numbers: enough to go on from here in another object.
    fn get_state<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let state = self.inner.state();
        let tables = state.run.len();
        let as_i64 = |values: Vec<u64>| -> Vec<i64> {
            // Run and deal counters stay far below 2^63.
            values.into_iter().map(|value| value as i64).collect()
        };
        let dict = PyDict::new(py);
        dict.set_item("base_seed", state.base_seed)?;
        dict.set_item("run", as_i64(state.run).into_pyarray(py))?;
        dict.set_item(
            "deal_in_run",
            state
                .deal_in_run
                .into_iter()
                .map(i64::from)
                .collect::<Vec<i64>>()
                .into_pyarray(py),
        )?;
        dict.set_item("deals_done", as_i64(state.deals_done).into_pyarray(py))?;
        dict.set_item(
            "n_actions",
            state
                .n_actions
                .into_iter()
                .map(i64::from)
                .collect::<Vec<i64>>()
                .into_pyarray(py),
        )?;
        let per_table = state.actions.len() / tables.max(1);
        dict.set_item(
            "actions",
            state
                .actions
                .into_pyarray(py)
                .reshape([tables, per_table])?,
        )?;
        Ok(dict)
    }

    /// Puts every table where a state from `get_state` says. If the state
    /// does not fit, nothing changes and a `ValueError` is raised. The
    /// four counters have shape `(tables,)`, the actions
    /// `(tables, MAX_ACTIONS)`.
    fn set_state(
        &mut self,
        base_seed: u64,
        run: PyReadonlyArray1<'_, i64>,
        deal_in_run: PyReadonlyArray1<'_, i64>,
        deals_done: PyReadonlyArray1<'_, i64>,
        n_actions: PyReadonlyArray1<'_, i64>,
        actions: PyReadonlyArray2<'_, u8>,
    ) -> PyResult<()> {
        let unsigned = |name: &str, values: &[i64]| -> PyResult<Vec<u64>> {
            values
                .iter()
                .map(|&value| {
                    u64::try_from(value).map_err(|_| {
                        value_error(format!("the state does not fit: {name} is negative"))
                    })
                })
                .collect()
        };
        let narrow = |name: &str, values: &[i64]| -> PyResult<Vec<u32>> {
            values
                .iter()
                .map(|&value| {
                    u32::try_from(value).map_err(|_| {
                        value_error(format!("the state does not fit: {name} is out of range"))
                    })
                })
                .collect()
        };
        let n = self.inner.len();
        let state = EnvState {
            base_seed,
            run: unsigned("run", readable("run", &run, &[n])?)?,
            deal_in_run: narrow("deal_in_run", readable("deal_in_run", &deal_in_run, &[n])?)?,
            deals_done: unsigned("deals_done", readable("deals_done", &deals_done, &[n])?)?,
            n_actions: narrow("n_actions", readable("n_actions", &n_actions, &[n])?)?,
            actions: readable("actions", &actions, &[n, MAX_ACTIONS])?.to_vec(),
        };
        self.inner.set_state(&state).map_err(env_error)
    }
}

/// The seed of the cards of deal `deal` of run `run` at table `table`.
#[pyfunction]
fn deal_seed(base_seed: u64, table: u64, run: u64, deal: u64) -> u64 {
    vecenv::deal_seed(base_seed, table, run, deal)
}

/// The seat that deals the first deal of a run.
#[pyfunction]
fn first_dealer(table: u64, run: u64) -> usize {
    vecenv::first_dealer(table, run).index()
}

/// One deal played in Rust alone: at decision `i` the legal actions are
/// taken in slot order and number `choices[i] mod count` is played. Returns
/// `(entries, decisions, slots)`. Used by the parity test.
#[pyfunction]
fn play_deal(seed: u64, dealer: u8, choices: Vec<u64>) -> PyResult<(Vec<i32>, usize, Vec<u16>)> {
    let dealer = Seat::new(dealer).ok_or_else(|| value_error("the dealer is a seat, 0 to 3"))?;
    let mut game = Game::from_seed(seed, dealer);
    let mut slots = Vec::new();
    while game.phase() != Phase::Over {
        let legal = game.legal_actions();
        let Some(&choice) = choices.get(slots.len()) else {
            return Err(value_error(format!(
                "the deal needs more than {} choices",
                choices.len()
            )));
        };
        let action = legal
            .nth((choice % u64::from(legal.len().max(1))) as u32)
            .ok_or_else(|| value_error("no legal action before the end of the deal"))?;
        game.apply(action).map_err(value_error)?;
        slots.push(action.slot() as u16);
    }
    let score = game
        .score()
        .ok_or_else(|| value_error("the deal has no score"))?;
    Ok((score.entries().to_vec(), slots.len(), slots))
}

/// The JSON record of a deal from its seed, its dealer and the slots of
/// its actions. A slot that is not legal raises `ValueError` with the
/// index of the decision.
#[pyfunction]
fn record_json(seed: u64, dealer: u8, slots: Vec<u16>) -> PyResult<String> {
    let dealer = Seat::new(dealer).ok_or_else(|| value_error("the dealer is a seat, 0 to 3"))?;
    let mut game = Game::from_seed(seed, dealer);
    for (decision, &slot) in slots.iter().enumerate() {
        let applied = Action::from_slot(usize::from(slot))
            .ok_or(())
            .and_then(|action| game.apply(action).map_err(|_| ()));
        if applied.is_err() {
            return Err(value_error(format!(
                "decision {decision}: slot {slot} is not a legal action there"
            )));
        }
    }
    let record = Record::of_game(seed, &game).map_err(value_error)?;
    Ok(record.to_json())
}

/// Replays a JSON record and returns its four entries. A record that does
/// not replay raises `ValueError`.
#[pyfunction]
fn replay_record(text: &str) -> PyResult<Vec<i32>> {
    let game = Record::from_json(text)
        .and_then(|record| record.replay())
        .map_err(value_error)?;
    let score = game
        .score()
        .ok_or_else(|| value_error("the deal has no score"))?;
    Ok(score.entries().to_vec())
}

#[pymodule]
fn tarok_env(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Tables>()?;
    m.add_function(wrap_pyfunction!(deal_seed, m)?)?;
    m.add_function(wrap_pyfunction!(first_dealer, m)?)?;
    m.add_function(wrap_pyfunction!(play_deal, m)?)?;
    m.add_function(wrap_pyfunction!(record_json, m)?)?;
    m.add_function(wrap_pyfunction!(replay_record, m)?)?;
    m.add("ENCODING_VERSION", encode::ENCODING_VERSION)?;
    m.add("N_EVENTS", encode::N_EVENTS)?;
    m.add("STATE_FEATURES", encode::STATE_FEATURES)?;
    m.add("N_ACTIONS", encode::N_ACTIONS)?;
    m.add("MAX_TOKENS", encode::MAX_TOKENS)?;
    m.add("TOKEN_FIELDS", encode::TOKEN_FIELDS)?;
    m.add("CRITIC_EXTRA", encode::CRITIC_EXTRA)?;
    m.add("CRITIC_FEATURES", encode::CRITIC_FEATURES)?;
    m.add("N_SCORE_PARTS", encode::N_SCORE_PARTS)?;
    m.add("DEALS_PER_RUN", vecenv::DEALS_PER_RUN)?;
    m.add("MAX_ACTIONS", MAX_ACTIONS)?;
    m.add("ENGINE_VERSION", tarok_engine::VERSION)?;
    // The fingerprint of the sources this module was built from, written by
    // build.rs (D130). `tarok_train/env.py` compares it with the files on disk.
    m.add("SOURCE_HASH", env!("TAROK_SOURCE_HASH"))?;
    m.add(
        "SOURCE_MANIFEST",
        include_str!(concat!(env!("OUT_DIR"), "/source_manifest.txt")),
    )?;
    Ok(())
}
