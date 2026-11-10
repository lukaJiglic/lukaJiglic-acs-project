//! Spec 003 probe. One function: release the GIL, fill an array on `n_threads`
//! OS threads, return it to Python as a NumPy array.

use std::collections::HashSet;
use std::hint::black_box;

use numpy::{IntoPyArray, PyArray1};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

/// The value written at index `i`. The Python test recomputes it.
fn value_at(i: usize) -> f32 {
    ((i * 7 + 3) % 1000) as f32
}

/// A fixed amount of CPU work per element: `rounds` steps of a 64-bit generator.
/// `black_box` keeps the compiler from folding the loop away.
fn burn(seed: u64, rounds: u32) -> u64 {
    let mut x = seed;
    for _ in 0..rounds {
        x = black_box(
            x.wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407),
        );
    }
    x
}

/// Fill `n` values on `n_threads` threads with the GIL released.
///
/// Returns `(values, worker, os_threads)`:
/// - `values[i] = (7 i + 3) mod 1000` as f32,
/// - `worker[i]` = index of the thread that wrote element `i`,
/// - `os_threads` = number of distinct OS thread ids that did the work.
///
/// Every element also costs `rounds` steps of CPU work. The total work is
/// `n * rounds` whatever the thread count, so a test can compare the wall time
/// of 1 thread and of 12 threads.
#[pyfunction]
fn fill_parallel<'py>(
    py: Python<'py>,
    n: usize,
    n_threads: usize,
    rounds: u32,
) -> PyResult<(Bound<'py, PyArray1<f32>>, Bound<'py, PyArray1<u8>>, usize)> {
    if n_threads == 0 || n_threads > 255 {
        return Err(PyValueError::new_err("n_threads must be 1..=255"));
    }
    if n < n_threads {
        return Err(PyValueError::new_err("n must be at least n_threads"));
    }

    let (values, worker, os_threads) = py.detach(|| {
        let mut values = vec![0f32; n];
        let mut worker = vec![0u8; n];
        let base = n / n_threads;
        let extra = n % n_threads;

        let ids: HashSet<std::thread::ThreadId> = std::thread::scope(|scope| {
            let mut handles = Vec::with_capacity(n_threads);
            let mut rest_values = values.as_mut_slice();
            let mut rest_worker = worker.as_mut_slice();
            let mut start = 0usize;
            for w in 0..n_threads {
                let len = base + usize::from(w < extra);
                let (my_values, tail_values) = rest_values.split_at_mut(len);
                let (my_worker, tail_worker) = rest_worker.split_at_mut(len);
                rest_values = tail_values;
                rest_worker = tail_worker;
                let first = start;
                start += len;
                handles.push(scope.spawn(move || {
                    for (k, (v, who)) in my_values.iter_mut().zip(my_worker.iter_mut()).enumerate() {
                        let i = first + k;
                        black_box(burn(i as u64, rounds));
                        *v = value_at(i);
                        *who = w as u8;
                    }
                    std::thread::current().id()
                }));
            }
            handles
                .into_iter()
                .map(|h| h.join().expect("probe worker panicked"))
                .collect()
        });
        (values, worker, ids.len())
    });

    Ok((values.into_pyarray(py), worker.into_pyarray(py), os_threads))
}

#[pymodule]
fn tarok_pyo3_probe(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(fill_parallel, m)?)?;
    Ok(())
}
