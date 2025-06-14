// src/params.rs
//! Parameter bundle for TSQC (Tabu Search for γ-quasi-cliques).
//!
//! We fix:
//! - `stagnation_iter` (L) = 1000  (max non-improving iterations before restart)  
//! - `max_iter`        (Itₘₐₓ) = 100_000_000  (hard cap on total iterations)
//!
//! The short‐term tabu tenures `tenure_u`/`tenure_v` are initialized to 1
//! but are always immediately **overwritten** by our adaptive formula
//! in `DualTabu::update_tenures(...)`.  
//!
//! `gamma_target` must be set by the caller to the desired density threshold.

/// All tunable controls for TSQC.
#[derive(Clone, Debug)]
pub struct Params {
    /// Base tenure for forbidding recently removed vertices (Tu).
    /// *Note:* this value is only a safety minimum—actual Tu is
    /// recomputed each iteration via §3.4.3.
    pub tenure_u:         usize,

    /// Base tenure for forbidding recently added vertices (Tv).
    /// *Note:* likewise, actual Tv is adaptive.
    pub tenure_v:         usize,

    /// Target density γ ∈ (0,1] defining a γ-quasi-clique.
    pub gamma_target:     f64,

    /// L: max consecutive non-improving swaps before a diversification
    /// restart (Section 3.1). Default = 1000.
    pub stagnation_iter:  usize,

    /// Itₘₐₓ: hard cap on total TSQ iterations across all restarts
    /// (Section 3.1). Default = 10⁸.
    pub max_iter:         usize,
}

impl Default for Params {
    fn default() -> Self {
        Params {
            tenure_u:        1,           // minimal safety base
            tenure_v:        1,           // minimal safety base
            gamma_target:    0.90,        // example default; override as needed
            stagnation_iter: 1_000,       // L = 1000
            max_iter:        100_000_000, // Itₘₐₓ = 1e8
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_params() {
        let p = Params::default();
        assert_eq!(p.tenure_u, 1);
        assert_eq!(p.tenure_v, 1);
        assert!((p.gamma_target - 0.90).abs() < 1e-12);
        assert_eq!(p.stagnation_iter, 1_000);
        assert_eq!(p.max_iter, 100_000_000);
    }
}
