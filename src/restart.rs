// src/restart.rs
//! Multi-start Tabu Search for fixed-k γ-quasi-clique (TSQC Alg. 1 & 2).
//! Implements:
//!  1) Greedy-random initialisation (§ 3.3).
//!  2) Intensification via `improve_once` (§ 3.4.1).
//!  3) Tight one-swap UB stopping (“U1-tight”, § 3.4.3).
//!  4) Adaptive heavy/mild diversification (§ 3.4.2).
//!  5) Restart strategy with long-term frequency memory (§ 3.5).
//!  6) Global cap on total moves (`p.max_iter`).

use crate::{
    construct::greedy_random_k,
    diversify::{heavy_perturbation, mild_perturbation},
    neighbour::improve_once,
    params::Params,
    solution::Solution,
    tabu::DualTabu,
    Graph,
};
use rand::seq::SliceRandom;
use rand::Rng;
use std::f64;

/// Solve the fixed-k γ-quasi-clique problem on `graph`, returning the best
/// γ-quasi-clique of size `k` found (or an empty solution if none feasible).
pub fn solve_fixed_k<'g, R>(
    graph: &'g Graph,
    k: usize,
    rng: &mut R,
    p: &Params,
) -> Solution<'g>
where
    R: Rng + ?Sized,
{
    // 0) Precompute required edges for feasibility: ceil(γ·C(k,2))
    let needed_edges = ((p.gamma_target * ((k * (k - 1) / 2) as f64)).ceil()) as usize;
    // Quick impossibility check
    if (k * (k - 1) / 2) < needed_edges {
        return Solution::new(graph);
    }

    // Long-term frequency memory for restarts
    let mut freq = vec![0usize; graph.n()];
    // Global best solution over all restarts (for aspiration & final return)
    let mut best_global = Solution::new(graph);
    let mut best_global_rho = 0.0;
    // Total moves across all restarts
    let mut total_moves = 0usize;

    // Outer restart loop
    while total_moves < p.max_iter {
        // 1) INITIAL SOLUTION
        let mut cur = if best_global.size() == 0 {
            // First run: pure greedy-random
            greedy_random_k(graph, k, rng)
        } else {
            // Restart: seed from least-used vertex + greedy fill (§ 3.5)
            let min_f = *freq.iter().min().unwrap();
            let mut pool: Vec<usize> =
                (0..graph.n()).filter(|&v| freq[v] == min_f).collect();
            pool.shuffle(rng);
            let seed = pool[0];

            let mut s = Solution::new(graph);
            s.add(seed);
            while s.size() < k {
                // pick outsider with max internal edges
                let mut best_deg = 0;
                let mut cand = Vec::new();
                for v in 0..graph.n() {
                    if s.bitset()[v] { continue; }
                    let deg = graph
                        .neigh_row(v)
                        .iter_ones()
                        .filter(|&u| s.bitset()[u])
                        .count();
                    match deg.cmp(&best_deg) {
                        std::cmp::Ordering::Greater => {
                            best_deg = deg;
                            cand.clear();
                            cand.push(v);
                        }
                        std::cmp::Ordering::Equal => cand.push(v),
                        _ => {}
                    }
                }
                s.add(*cand.choose(rng).unwrap());
            }
            s
        };

        // 2) INITIALISE TABU STRUCTURE and one initial tenure adaptation
        let mut tabu = DualTabu::new(graph.n(), p.tenure_u, p.tenure_v);
        tabu.update_tenures(cur.size(), cur.edges(), p.gamma_target, rng);

        // Track best in this run
        let mut best_run = cur.clone();
        let mut rho_run = cur.density();
        let mut stagnation = 0usize;

        // 3) LOCAL SEARCH LOOP until stagnation or global cap
        while stagnation < p.stagnation_iter && total_moves < p.max_iter {
            // Intensification step (§ 3.4.1)
            let _moved = improve_once(
                &mut cur,
                &mut tabu,
                best_global_rho,
                &mut freq,
                p,
                rng,
            );
            total_moves += 1;

            // Update run-best
            let rho = cur.density();
            if rho > rho_run {
                rho_run = rho;
                best_run = cur.clone();
                stagnation = 0;
            } else {
                stagnation += 1;
            }

            // If feasible, return immediately
            if rho_run + f64::EPSILON >= p.gamma_target {
                return best_run;
            }

            // 3a) U1-tight stopping (§ 3.4.3)
            let mut min_in = usize::MAX;
            for u in best_run.bitset().iter_ones() {
                let d = graph
                    .neigh_row(u)
                    .iter_ones()
                    .filter(|&j| best_run.bitset()[j])
                    .count();
                min_in = min_in.min(d);
            }
            let mut max_out = 0;
            for v in 0..graph.n() {
                if best_run.bitset()[v] { continue; }
                let d = graph
                    .neigh_row(v)
                    .iter_ones()
                    .filter(|&j| best_run.bitset()[j])
                    .count();
                max_out = max_out.max(d);
            }
            let ub = best_run.edges() + max_out.saturating_sub(min_in);
            if ub < needed_edges {
                break;
            }

            // 3b) Diversification if stagnated (§ 3.4.2)
            if stagnation >= p.stagnation_iter {
                // Compute heavy-shake probability
                let max_edges = k * (k - 1) / 2;
                let deficit = if cur.edges() < needed_edges {
                    (needed_edges - cur.edges()) as f64
                        / (max_edges - cur.edges()) as f64
                } else {
                    0.0
                };
                let p_heavy = (deficit + 2.0 / (k as f64)).min(1.0);

                if rng.gen_bool(p_heavy) {
                    heavy_perturbation(&mut cur, &mut tabu, rng, p, &mut freq);
                } else {
                    mild_perturbation(&mut cur, &mut tabu, rng, p, &mut freq);
                }

                // reset stagnation
                stagnation = 0;
            }
        }

        // 4) Update global best if run-best improved
        if rho_run > best_global_rho {
            best_global_rho = rho_run;
            best_global = best_run;
        }
    }

    // Return overall best found
    best_global
}
