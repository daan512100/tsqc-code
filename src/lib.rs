//! tsqc – Rust implementation of the TSQC algorithm with optional Python bindings.

use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

pub mod graph;
pub mod solution;
pub mod tabu;
pub mod construct;
pub mod neighbour;
pub mod diversify;
pub mod params;
pub mod restart;
pub mod maxk;

pub use graph::Graph;
pub use solution::Solution;
pub use params::Params;
pub use restart::solve_fixed_k;
pub use maxk::solve_maxk;

/* helper re-exports for advanced Rust callers */
pub use construct::{random_k, greedy_k};
pub use neighbour::{improve_once, improve_until_local_optimum};
pub use diversify::{heavy_perturbation, mild_perturbation};

use rand_chacha::ChaCha8Rng;
use rand::SeedableRng;
use std::fs::File;
use std::io::BufReader;

//////////////////////////////////////////////////////////////
/// --------------- PyO3 wrappers ---------------------------
//////////////////////////////////////////////////////////////

/// Fixed-k solver.  
/// Returns the **density** of the best k-subset found
/// (γ-feasible or not – caller decides how to interpret it).
#[pyfunction]
#[pyo3(signature = (edges, n, k, gamma, seed))]
fn solve_k_py(
    edges: Vec<(usize, usize)>,
    n:     usize,
    k:     usize,
    gamma: f64,
    seed:  u64,
) -> PyResult<f64> {
    let g = Graph::from_edge_list(n, &edges);

    let mut params = Params::default();
    params.gamma_target = gamma;

    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let sol      = solve_fixed_k(&g, k, &mut rng, &params);

    Ok(sol.density())
}

/// Any-k solver.  
/// Returns **(best_size, density)** of the largest γ-feasible quasi-clique found.
#[pyfunction]
#[pyo3(signature = (edges, n, gamma, seed))]
fn solve_max_py(
    edges: Vec<(usize, usize)>,
    n:     usize,
    gamma: f64,
    seed:  u64,
) -> PyResult<(usize, f64)> {
    let g = Graph::from_edge_list(n, &edges);

    let mut params = Params::default();
    params.gamma_target = gamma;

    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let sol      = solve_maxk(&g, &mut rng, &params);

    Ok((sol.size(), sol.density()))
}

/// Fast DIMACS *.clq parser exposed to Python.
/// Returns *(n_vertices, edge_list)* with 0-based indices.
#[pyfunction]
fn parse_dimacs_py(path: &str) -> PyResult<(usize, Vec<(usize, usize)>)> {
    let file  = File::open(path)?;
    let graph = Graph::parse_dimacs(BufReader::new(file))?;
    Ok((graph.n(), graph.edge_list()))
}

//////////////////////////////////////////////////////////////
/// --------------- module bootstrap ------------------------
//////////////////////////////////////////////////////////////

/// Compiled as `tsqc/_native.*`
#[pymodule]
fn _native(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(solve_k_py,   m)?)?;
    m.add_function(wrap_pyfunction!(solve_max_py, m)?)?;
    m.add_function(wrap_pyfunction!(parse_dimacs_py, m)?)?;
    Ok(())
}
