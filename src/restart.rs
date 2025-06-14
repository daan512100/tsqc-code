//! Multi-start tabu search for one fixed k (TSQC Algorithm 2)
//!
//! A run is aborted as soon as the *tight* upper bound
//!   UB = cur_edges + best_gain_one_swap
//! falls below γ·C(k,2)   (rule U1-tight).

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

/*──────────────────────────────────────────────────────────*/

pub fn solve_fixed_k<'g, R>(
    graph: &'g Graph,
    k: usize,
    rng: &mut R,
    p: &Params,
) -> Solution<'g>
where
    R: Rng + ?Sized,
{
    /* γ-required edge count */
    let needed = ((p.gamma_target * (k * (k - 1) / 2) as f64).ceil()) as usize;

    /* quick impossibility via degree bound */
    if (k * (k - 1) / 2) < needed {
        return Solution::new(graph);
    }

    /* long-term vertex frequency (restart diversification) */
    let mut freq = vec![0usize; graph.n()];

    /* best over all runs (for return if infeasible) */
    let mut best_global = Solution::new(graph);
    let mut best_rho    = 0.0;

    /* total moves across restarts */
    let mut total_moves = 0usize;

    /* per-run move cap  (heuristic, a few thousand) */
    let run_iter_cap = 4 * k * k;

    /*──────────────── outer restart loop ────────────────*/
    loop {
        /* 1. initial subset */
        let mut cur = if best_global.size() == 0 {
            greedy_random_k(graph, k, rng)                 // first run
        } else {
            /* seed = least-used vertex, then greedy fill */
            let min_f = *freq.iter().min().unwrap();
            let mut pool: Vec<_> =
                (0..graph.n()).filter(|&v| freq[v] == min_f).collect();
            pool.shuffle(rng);
            let seed = pool[0];

            let mut s = Solution::new(graph);
            s.add(seed);
            while s.size() < k {
                let mut best_deg = 0usize;
                let mut cand     = Vec::new();
                for v in 0..graph.n() {
                    if s.bitset()[v] { continue; }
                    let deg = graph.neigh_row(v)
                                   .iter_ones()
                                   .filter(|&u| s.bitset()[u])
                                   .count();
                    match deg.cmp(&best_deg) {
                        std::cmp::Ordering::Greater => { best_deg = deg; cand.clear(); cand.push(v); }
                        std::cmp::Ordering::Equal   => cand.push(v),
                        _ => {}
                    }
                }
                s.add(*cand.choose(rng).unwrap());
            }
            s
        };

        /* tabu structures */
        let mut tabu = DualTabu::new(graph.n(), p.tenure_u, p.tenure_v);
        tabu.update_tenures(k, cur.edges(), p.gamma_target, rng);

        /* best inside this run */
        let mut best_run = cur.clone();
        let mut rho_run  = cur.density();

        let mut stagn   = 0usize;
        let mut moves   = 0usize;              // moves in this run

        /* helper: tight UB using *one* best swap */
        let mut impossible = |sol: &Solution<'_>| -> bool {
            /* collect internal degrees for in-set vertices */
            let mut min_in = usize::MAX;
            for v in sol.bitset().iter_ones() {
                let d = sol.graph().neigh_row(v)
                           .iter_ones()
                           .filter(|&u| sol.bitset()[u])
                           .count();
                min_in = min_in.min(d);
            }
            /* best outsider degree into current set */
            let mut max_out = 0usize;
            for v in 0..sol.graph().n() {
                if sol.bitset()[v] { continue; }
                let d = sol.graph().neigh_row(v)
                           .iter_ones()
                           .filter(|&u| sol.bitset()[u])
                           .count();
                max_out = max_out.max(d);
            }
            let gain = max_out.saturating_sub(min_in);
            let ub   = sol.edges() + gain;
            ub < needed
        };

        /*──────── inner tabu loop ────────*/
        loop {
            let moved = improve_once(
                &mut cur, &mut tabu, rho_run,
                &mut freq, p, rng);
            moves      += 1;
            total_moves += 1;

            if moved {
                let rho = cur.density();
                if rho > rho_run {
                    rho_run  = rho;
                    best_run = cur.clone();
                    stagn    = 0;
                } else {
                    stagn += 1;
                }
            } else {
                stagn += 1;
            }

            /* γ-feasible found */
            if rho_run + f64::EPSILON >= p.gamma_target {
                return best_run;
            }

            /* diverge after L moves with no improvement */
            if stagn >= p.stagnation_iter {
                if rng.gen_bool(p.heavy_prob) {
                    heavy_perturbation(&mut cur, &mut tabu, rng, p, &mut freq);
                } else {
                    mild_perturbation (&mut cur, &mut tabu, rng, p, &mut freq);
                }
                stagn = 0;

                if impossible(&cur) { break; }        // U1-tight
            }

            /* if no admissible swap *and* UB says impossible */
            if !moved && impossible(&cur) { break; }

            /* per-run or global caps */
            if moves >= run_iter_cap || total_moves >= p.max_iter {
                break;
            }
        }

        /* update global best (still infeasible) */
        if rho_run > best_rho {
            best_rho    = rho_run;
            best_global = best_run.clone();
        }

        /* update frequencies */
        for v in best_run.bitset().iter_ones() { freq[v] += 1; }
    }
}
