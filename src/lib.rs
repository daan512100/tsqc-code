//! TSQC – Rust-kernel + PyO3 bindings.

use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
use pyo3::types::PyModule;
use pyo3::prelude::Bound;       // Bound<'py, T> API in PyO3 v0.25

/*───────── interne modules ─────────*/
pub mod graph;
pub mod solution;
pub mod tabu;
pub mod construct;
pub mod neighbour;
pub mod diversify;
pub mod params;
pub mod restart;
pub mod maxk;

/*───────── re-exports voor Rust-gebruikers ─────────*/
pub use graph::Graph;
pub use solution::Solution;
pub use params::Params;
pub use restart::solve_fixed_k;
pub use maxk::solve_maxk;

/*───────── extern util ─────────*/
use rand_chacha::ChaCha8Rng;
use rand::SeedableRng;
use std::fs::File;
use std::io::BufReader;

/*======================================================================
│  Python-functies
└=====================================================================*/

/// Fixed-k solver – returns density of best k-subset.
#[pyfunction]
#[pyo3(text_signature = "(graph_path, k, gamma, seed)")]
fn solve_k_py(graph_path: String, k: usize, gamma: f64, seed: u64) -> PyResult<f64> {
    let file = File::open(&graph_path)
        .map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))?;
    let graph = Graph::parse_dimacs(BufReader::new(file))
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

    let mut p = Params::default();
    p.gamma_target = gamma;

    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let sol = solve_fixed_k(&graph, k, &mut rng, &p);
    Ok(sol.density())
}

/// Max-k solver – returns (size, density) of best quasi-clique.
#[pyfunction]
#[pyo3(text_signature = "(graph_path, gamma, seed)")]
fn solve_max_py(graph_path: String, gamma: f64, seed: u64) -> PyResult<(usize, f64)> {
    let file = File::open(&graph_path)
        .map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))?;
    let graph = Graph::parse_dimacs(BufReader::new(file))
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

    let mut p = Params::default();
    p.gamma_target = gamma;

    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let sol = maxk::solve_maxk(&graph, &mut rng, &p);
    Ok((sol.size(), sol.density()))
}

/// Helper: parse DIMACS, return (n, m).
#[pyfunction]
#[pyo3(text_signature = "(graph_path)")]
fn parse_dimacs_py(graph_path: String) -> PyResult<(usize, usize)> {
    let file = File::open(&graph_path)
        .map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))?;
    let graph = Graph::parse_dimacs(BufReader::new(file))
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
    Ok((graph.n(), graph.m()))
}

/*======================================================================
│  PyO3 module-init
└=====================================================================*/

/// ***Important***: name `_native` must match `pyproject.toml -> module-name`.
#[pymodule]
fn _native(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(solve_k_py, m)?)?;
    m.add_function(wrap_pyfunction!(solve_max_py, m)?)?;
    m.add_function(wrap_pyfunction!(parse_dimacs_py, m)?)?;
    Ok(())
}
