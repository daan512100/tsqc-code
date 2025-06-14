//! Multi-start controller for one fixed k  (Algorithm 2).
//!
//! • Runs tabu-search until it finds a γ-feasible subset *or*
//!   the clique upper-bound (rule U1) proves infeasibility.
//! • If infeasible → build a new initial subset biased by the
//!   long-term frequencies and restart, until `max_iter`.

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

/* ────────────────────────────────────────────────────────── */

/// return the best solution found for this `k`
pub fn solve_fixed_k<'g, R>(
    graph: &'g Graph,
    k: usize,
    rng: &mut R,
    p: &Params,
) -> Solution<'g>
where
    R: Rng + ?Sized,
{
    /* quick impossibility check */
    let req = ((p.gamma_target * (k * (k - 1) / 2) as f64).ceil()) as usize;
    if (k * (k - 1) / 2) < req {
        return Solution::new(graph);                     // impossible size
    }

    /* long-term frequency memory */
    let mut freq: Vec<usize> = vec![0; graph.n()];

    /* global best (may be infeasible) */
    let mut best_overall = Solution::new(graph);
    let mut best_rho     = 0.0;

    let mut total_it = 0usize;
    let     mut run  = 0usize;

    /* helper: clique upper-bound rule U1 for *current* solution */
    let mut infeasible = |edges_cur: usize| -> bool {
        let ub = k * (k - 1) / 2;      // clique
        ub < req || edges_cur >= ub    // second term can only happen if
    };                                 // we already have a clique

    /* ========================================================= */
    'restart: loop {
        run += 1;

        /* ---- 1  initial subset -------------------------------- */
        let mut cur = if run == 1 {
            greedy_random_k(graph, k, rng)
        } else {
            /* biased by long-term frequency (least-used vertex seed) */
            let min_f = *freq.iter().min().unwrap();
            let mut cand: Vec<_> = (0..graph.n()).filter(|&v| freq[v] == min_f).collect();
            cand.shuffle(rng);
            let seed = cand[0];

            let mut s = Solution::new(graph);
            s.add(seed);
            while s.size() < k {
                let mut best_deg = 0usize;
                let mut pool     = Vec::new();
                for v in 0..graph.n() {
                    if s.bitset()[v] { continue; }
                    let d = graph.neigh_row(v)
                                  .iter_ones()
                                  .filter(|&u| s.bitset()[u])
                                  .count();
                    match d.cmp(&best_deg) {
                        std::cmp::Ordering::Greater => { best_deg = d; pool.clear(); pool.push(v); }
                        std::cmp::Ordering::Equal   => pool.push(v),
                        _ => {}
                    }
                }
                s.add(*pool.choose(rng).unwrap());
            }
            s
        };

        /* tabu structures */
        let mut tabu = DualTabu::new(graph.n(), p.tenure_u, p.tenure_v);
        tabu.update_tenures(k, cur.edges(), p.gamma_target, rng);

        let mut best_run = cur.clone();
        let mut best_rho_run = cur.density();

        let mut stagn = 0usize;

        /* ---- 2  inner loop ----------------------------------- */
        loop {
            let moved = improve_once(&mut cur, &mut tabu,
                                     best_rho_run, &mut freq, p, rng);
            total_it += 1;

            if moved {
                let rho = cur.density();
                if rho > best_rho_run {
                    best_rho_run = rho;
                    best_run     = cur.clone();
                    stagn        = 0;
                } else {
                    stagn += 1;
                }
            } else {
                stagn += 1;
            }

            /* γ-feasible → done */
            if best_rho_run + f64::EPSILON >= p.gamma_target {
                return best_run;
            }

            /* diversification after L iterations with no gain */
            if stagn >= p.stagnation_iter {
                if rng.gen_bool(p.heavy_prob) {
                    heavy_perturbation(&mut cur, &mut tabu, rng, p, &mut freq);
                } else {
                    mild_perturbation (&mut cur, &mut tabu, rng, p, &mut freq);
                }
                stagn = 0;

                /* immediately test rule U1 on the *current* solution */
                if infeasible(cur.edges()) {
                    break;              // abort this run, restart
                }
            }

            /* also abort if *no* admissible swap was possible */
            if !moved && infeasible(cur.edges()) {
                break;                  // restart
            }

            if total_it >= p.max_iter {
                return best_overall;    // hard cap
            }
        }

        /* ---- 3  update long-term memory & restart ------------- */
        if best_rho_run > best_rho {
            best_rho     = best_rho_run;
            best_overall = best_run.clone();
        }
        for v in best_run.bitset().iter_ones() {
            freq[v] += 1;
        }
    }
}
