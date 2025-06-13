//! Outer “max-k” search (Algorithm 2).
//!
//! For k = 2 … n it calls `solve_fixed_k`, keeping the **largest γ-feasible
//! set** found so far (ties broken by higher density).  The loop exits early
//! if it encounters a full clique (density ≈ 1).

use crate::{
    params::Params,
    restart::solve_fixed_k,
    solution::Solution,
    Graph,
};
use rand::Rng;

pub fn solve_maxk<'g, R>(
    graph: &'g Graph,
    rng: &mut R,
    p: &Params,
) -> Solution<'g>
where
    R: Rng + ?Sized,
{
    let mut best_sol = Solution::new(graph);  // empty ← not γ-feasible
    let mut best_k   = 0usize;                // track size for clarity

    for k in 2..=graph.n() {
        let sol_k = solve_fixed_k(graph, k, rng, p);

        /* consider only γ-feasible candidates */
        if !sol_k.is_gamma_feasible(p.gamma_target) {
            continue;
        }

        if sol_k.size() > best_k
            || (sol_k.size() == best_k && sol_k.density() > best_sol.density())
        {
            best_k   = sol_k.size();
            best_sol = sol_k;

            /* found a clique ⇒ nothing larger can beat density 1.0 */
            if best_sol.density() >= 1.0 - f64::EPSILON {
                break;
            }
        }
    }

    best_sol
}
