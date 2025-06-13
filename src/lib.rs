//! tsqc – core Rust crate with optional Python bindings.
//
//  ── Build targets ──────────────────────────────────────────────────────────────
//  • `cargo check`            – bare Rust compile (no Python needed)
//  • `maturin develop --release`
//      ↳ builds a binary wheel and installs `tsqc` into the active venv
//
//  Public API will grow as soon as the algorithmic modules are implemented.
//

use pyo3::prelude::*;

/// Python entry-point.
///
/// After `maturin develop --release` you can simply:
/// ```python
/// >>> import tsqc                 # loads the native extension
/// >>> help(tsqc)                  # will list registered Rust functions later
/// ```
#[pymodule]
fn tsqc(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    // TODO: register functions once they exist, e.g.
    // m.add_function(wrap_pyfunction!(solve_mqcp, m)?)?;
    Ok(())
}
