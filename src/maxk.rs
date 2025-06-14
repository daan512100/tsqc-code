// src/maxk.rs
//! Outer **max-k** search for TSQC (Alg. 1, § 3.1).
//!
//! 1. Compute an initial γ-feasible subset S₀ via `greedy_until_gamma` → k_lb = |S₀|
//! 2. Build a degree‐prefix table for quick upper bounds.
//! 3. For k = k_lb..n:
//!     a) If k == best_sol.size(), skip (already feasible).
//!     b) Compute `required = ceil(γ * C(k,2))`.
//!     c) If `ub_edges(prefix, k) < required`:
//!           • If k > best_sol.size(), **break** (first impossibility above best).  
//!           • Otherwise `continue`.
//!     d) Run `solve_fixed_k(graph, k, ...)`.
//!     e) If solution is feasible (density ≥ γ): update best_sol.
//!        Else if k > best_sol.size(): **break** (first failure above best).
//! 4. Return `best_sol`.

use crate::{
    construct::greedy_until_gamma,
    params::Params,
    restart::solve_fixed_k,
    solution::Solution,
    graph::Graph,
};
use rand::Rng;

/// Build prefix sums of degrees in descending order:
/// `pref[i] = sum_{j< i} deg_j`, where `deg_0 ≥ deg_1 ≥ …`.
fn degree_prefix(graph: &Graph) -> Vec<usize> {
    let mut degs: Vec<usize> = (0..graph.n()).map(|v| graph.degree(v)).collect();
    degs.sort_unstable_by(|a, b| b.cmp(a)); // descending
    let mut pref = Vec::with_capacity(degs.len() + 1);
    pref.push(0);
    let mut sum = 0;
    for d in degs {
        sum += d;
        pref.push(sum);
    }
    pref
}

/// Upper‐bound on the number of edges in any k‐subset:
/// UB(k) = ½ Σ_{i=0..k-1} min(deg_i, k-1)
#[inline]
fn ub_edges(prefix: &[usize], k: usize) -> usize {
    let mut total = 0;
    for i in 0..k {
        // deg_i = prefix[i+1] - prefix[i]
        let di = prefix[i + 1] - prefix[i];
        total += di.min(k - 1);
    }
    total / 2
}

/// Solve the maximum γ-quasi-clique problem via incremental fixed-k tabu searches.
///
/// Returns the best γ‐quasi‐clique found.
pub fn solve_maxk<'g, R>(
    graph: &'g Graph,
    rng: &mut R,
    p: &Params,
) -> Solution<'g>
where
    R: Rng + ?Sized,
{
    // 1) initial greedy γ-feasible solution
    let mut best_sol = greedy_until_gamma(graph, p.gamma_target, rng);
    let k_lb = best_sol.size();

    // 2) degree-prefix for quick UB checks
    let pref = degree_prefix(graph);

    let n = graph.n();
    for k in k_lb..=n {
        // already feasible at this size?
        if k == best_sol.size() {
            continue;
        }

        // compute how many edges we need to satisfy γ at size k:
        let clique_edges = k.saturating_mul(k.saturating_sub(1)) / 2;
        let required = (p.gamma_target * (clique_edges as f64)).ceil() as usize;

        // quick impossibility test
        if ub_edges(&pref, k) < required {
            // first impossible above current best → stop
            if k > best_sol.size() {
                break;
            }
            continue;
        }

        // 3) expensive tabu search for fixed k
        let sol_k = solve_fixed_k(graph, k, rng, p);

        // if feasible, update best; otherwise, first failure above best → stop
        if sol_k.density() + f64::EPSILON >= p.gamma_target {
            best_sol = sol_k;
        } else if k > best_sol.size() {
            break;
        }
    }

    best_sol
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand_chacha::ChaCha8Rng;
    use rand::SeedableRng;

    #[test]
    fn simple_maxk_on_triangle() {
        // 3‐clique plus an extra leaf: best γ=1 is size 3
        let edges = &[(0,1),(1,2),(0,2),(2,3)];
        let g = Graph::from_edge_list(4, edges);
        let mut rng = ChaCha8Rng::seed_from_u64(0);
        let mut p = Params::default();
        p.gamma_target = 1.0;
        let sol = solve_maxk(&g, &mut rng, &p);
        assert_eq!(sol.size(), 3);
        assert!((sol.density() - 1.0).abs() < 1e-12);
    }
}
