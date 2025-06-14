//! Parameter bundle for TSQC (adaptive multistart tabu search).

#[derive(Clone, Debug)]
pub struct Params {
    /*── Tabu tenure base (adapted during search) ──*/
    pub tenure_u: usize,
    pub tenure_v: usize,

    /*── Diversification ──*/
    pub gamma:        f64,   // (unused by current heavy move)
    pub heavy_prob:   f64,   // probability of heavy perturbation

    /*── Quasi-clique feasibility goal ──*/
    pub gamma_target: f64,   // user-supplied γ

    /*── Stagnation & iteration limits ──*/
    pub stagnation_iter: usize, // iterations with no improvement before diversifying
    pub max_iter:        usize, // global hard cap
}

impl Default for Params {
    fn default() -> Self {
        Self {
            tenure_u: 7,
            tenure_v: 7,

            gamma:        0.50,      
            heavy_prob:   0.40,      

            gamma_target: 0.90,

            stagnation_iter: 1000,          // thesis uses 200
            max_iter:        100_000,    // thesis It_max
        }
    }
}
