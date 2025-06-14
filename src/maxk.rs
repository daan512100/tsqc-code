//! Outer **max-k** search for TSQC (Algorithm 1).
//!
//! 1.  Build a greedy γ-feasible subset S₀  →  *k_lb* = |S₀|  
//! 2.  For k = k_lb, k_lb+1, …  
//!       • Skip k if a degree upper-bound proves impossibility.  
//!       • Otherwise call `solve_fixed_k` (tabu search).  
//!       • If k > best_size **and** infeasible  →  stop (first failure above best).  
//! 3.  Return the largest γ-quasi-clique found.

use crate::{
    construct::greedy_until_gamma,
    params::Params,
    restart::solve_fixed_k,
    solution::Solution,
    Graph,
};
use rand::Rng;

/*───────────────────────────────────────────────────────────*/
/*  Degree upper-bound UB(k) = ½ Σ₀^{k-1} min{dᵢ, k-1}        */
/*───────────────────────────────────────────────────────────*/

fn degree_prefix(graph: &Graph) -> Vec<usize> {
    let mut deg: Vec<usize> = (0..graph.n()).map(|v| graph.degree(v)).collect();
    deg.sort_unstable_by(|a, b| b.cmp(a)); // descending
    let mut pref = Vec::with_capacity(deg.len() + 1);
    pref.push(0);
    let mut s = 0usize;
    for d in deg {
        s += d;
        pref.push(s);
    }
    pref // pref[i] = Σ d₀..d_{i-1}
}

#[inline]
fn ub_edges(prefix: &[usize], k: usize) -> usize {
    let mut s = 0usize;
    for i in 0..k {
        let d_i = prefix[i + 1] - prefix[i];
        s += d_i.min(k - 1);
    }
    s / 2
}

/*───────────────────────────────────────────────────────────*/
/*  Public driver                                            */
/*───────────────────────────────────────────────────────────*/

pub fn solve_maxk<'g, R>(graph: &'g Graph, rng: &mut R, p: &Params) -> Solution<'g>
where
    R: Rng + ?Sized,
{
    /* 1 ─ greedy γ-feasible lower bound */
    let mut best_sol = greedy_until_gamma(graph, p.gamma_target, rng);
    let k_lb         = best_sol.size();

    /* 2 ─ degree upper-bound table */
    let pref = degree_prefix(graph);

    for k in k_lb..=graph.n() {
        eprintln!("k = {}", k);
        if k == best_sol.size() {
            continue; // already feasible at this size
        }

        // quick impossibility check
        let required = ((p.gamma_target * (k * (k - 1) / 2) as f64).ceil()) as usize;
        if ub_edges(&pref, k) < required {
            if k > best_sol.size() {
                break; // first failure above best size  → stop
            }
            continue;
        }

        // expensive tabu search
        let sol_k = solve_fixed_k(graph, k, rng, p);
        if sol_k.density() + f64::EPSILON >= p.gamma_target {
            best_sol = sol_k; // update best
        } else if k > best_sol.size() {
            break; // first infeasible k above best size
        }
    }

    best_sol
}
