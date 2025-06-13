//! Multi-start controller: constructive start → local search → diversification.

use crate::{
    construct::{greedy_k, random_k},
    diversify::{heavy_perturbation, mild_perturbation},
    neighbour::improve_until_local_optimum,
    params::Params,
    solution::Solution,
    tabu::DualTabu,
    Graph,
};
use rand::Rng;

/// Run one TSQC search for a **fixed** `k`.
///
/// Stops early once a solution reaches the desired γ-density (`p.gamma_target`).
pub fn solve_fixed_k<'g, R>(
    graph: &'g Graph,
    k: usize,
    rng: &mut R,
    p: &Params,
) -> Solution<'g>
where
    R: Rng + ?Sized,
{
    /* 1 ── deterministic greedy start */
    let mut best_sol = greedy_k(graph, k);
    let mut best_d   = best_sol.density();

    /* 2 ── local-search state */
    let mut tabu  = DualTabu::new(graph.n(), p.tenure_u, p.tenure_v);
    let mut cur   = best_sol.clone();
    let mut stagn = 0usize;

    /* 3 ── main TSQC loop */
    for iter in 0..p.max_iter {
        improve_until_local_optimum(&mut cur, &mut tabu, best_d);

        let d = cur.density();
        if d > best_d {
            best_d   = d;
            best_sol = cur.clone();
            stagn    = 0;
        } else {
            stagn += 1;
        }

        /* diversification when stuck */
        if stagn >= p.stagnation_iter {
            if rng.gen_bool(p.heavy_prob) {
                heavy_perturbation(&mut cur, &mut tabu, rng, p.gamma);
            } else {
                mild_perturbation(&mut cur, &mut tabu);
            }
            stagn = 0;
        }

        /* early exit once a γ-quasi-clique has been found */
        if best_d + f64::EPSILON >= p.gamma_target {
            break;
        }

        /* periodic random reset to escape deep local minima */
        if iter % 2_000 == 1_999 {
            cur = random_k(graph, k, rng);
            tabu.reset();
        }
    }

    best_sol
}
