//! Parameter bundle for TSQC (Tabu Search for γ-quasi-cliques).
//!
//! These tunable parameters control the tabu search behavior, diversification intensity, and restart criteria.
//! 
//! - `tenure_u` and `tenure_v` are the initial tabu tenures for removed and added vertices (adaptive updates will override them during search).
//! - `gamma` controls the strength of heavy perturbation in the original design (fraction of the solution to replace). In our adapted implementation, heavy moves always remove 1 vertex (this parameter is unused by the current heavy perturbation logic).
//! - `gamma_target` is the density threshold (γ) that defines a quasi-clique (feasibility target).
//! - `stagnation_iter` is the number of consecutive non-improving iterations to tolerate before considering the search "stagnant". (In our implementation, we diversify immediately upon stagnation, so this effectively serves as an upper bound and as a safe value for frequency reset threshold).
//! - `max_iter` is the global cap on the total number of iterations (across all restarts and moves).

#[derive(Clone, Debug)]
pub struct Params {
    /* ─── Tabu tenure base (will be adapted dynamically) ─── */
    pub tenure_u: usize,
    pub tenure_v: usize,

    /* ─── Diversification ─── */
    pub gamma:        f64,   // (Unused in new heavy_perturbation) fraction of |S| to remove in original heavy perturbation
    pub heavy_prob:   f64,   // probability of choosing heavy vs. mild diversification

    /* ─── Quasi-clique feasibility goal ─── */
    pub gamma_target: f64,   // target density γ for a quasi-clique

    /* ─── Restart / search limits ─── */
    pub stagnation_iter: usize, // stagnation threshold (L in the paper – max consecutive iterations with no improvement)
    pub max_iter:        usize, // hard cap on total iterations (It_max)
}

impl Default for Params {
    fn default() -> Self {
        Self {
            tenure_u: 7,
            tenure_v: 7,
            // A small heavy perturbation probability by default, as TSQC applies heavy moves rarely
            gamma:      0.50,
            heavy_prob: 0.10,
            /* The default gamma_target of 0.90 matches the “sparse quasi-clique” benchmark in the thesis. 
               It can be adjusted by the caller if a different density threshold is needed. */
            gamma_target: 0.90,
            stagnation_iter: 1000,
            max_iter:        100_000,
        }
    }
}
