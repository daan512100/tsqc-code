//! Parameter bundle for TSQC (Tabu Search for γ-quasi-cliques).

/// Tunables that steer tabu search, diversification and restart behaviour.
///
/// * `gamma` controls **how strong** a heavy perturbation is  
///   (fraction of the current solution to be replaced).  
/// * `gamma_target` is the **feasibility threshold** from the paper
///   (the required density γ of a quasi-clique).  
#[derive(Clone, Debug)]
pub struct Params {
    /* ─── Tabu list ─────────────────────────────────────────────── */
    pub tenure_u: usize,
    pub tenure_v: usize,

    /* ─── Diversification ───────────────────────────────────────── */
    pub gamma:        f64,   // heavy perturbation fraction
    pub heavy_prob:   f64,   // probability of choosing heavy vs. mild move

    /* ─── Quasi-clique feasibility goal ─────────────────────────── */
    pub gamma_target: f64,   // γ (target density), 0 < γ ≤ 1

    /* ─── Restart / search limits ───────────────────────────────── */
    pub stagnation_iter: usize, // diversify after this many non-improving steps
    pub max_iter:        usize, // hard cap on inner iterations
}

impl Default for Params {
    fn default() -> Self {
        Self {
            tenure_u: 7,
            tenure_v: 7,

            gamma:      0.50,
            heavy_prob: 0.40,

            /* 0.90 matches the “sparse benchmark” values in the thesis
               but can be raised (e.g. 0.999) by the caller when needed. */
            gamma_target: 0.90,

            stagnation_iter: 1_000,
            max_iter:        1_000_000,
        }
    }
}
