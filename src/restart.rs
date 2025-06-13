//! Multi-start controller: initial construction → local search → diversification → possible restart.
//!
//! The `solve_fixed_k` function runs the TSQC search for a fixed subset size `k`. If no γ-feasible
//! solution is found in one run of tabu search, it **restarts** with a new initial solution generated
//! from long-term frequency memory. This continues until a quasi-clique is found or the global
//! iteration limit is reached, following Algorithm 2 in the thesis.

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

/// Tabu search for a fixed subset size `k`.  
/// Returns the best solution found (γ-feasible or densest illegal subset).
pub fn solve_fixed_k<'g, R>(
    graph: &'g Graph,
    k: usize,
    rng: &mut R,
    p: &Params,
) -> Solution<'g>
where
    R: Rng + ?Sized,
{
    /*────────────────── long-term frequency memory ──────────────────*/
    let mut freq: Vec<usize> = vec![0; graph.n()];

    /*────────────────── initial solution (greedy-random) ────────────*/
    let mut best_solution = greedy_random_k(graph, k, rng);
    let mut best_density = best_solution.density();

    /*──────── global best across restarts ────────*/
    let mut global_best = best_solution.clone();
    let mut global_best_d = best_density;

    if best_density + f64::EPSILON >= p.gamma_target {
        return best_solution;
    }

    /*──────── counters ────────*/
    let mut total_it = 0usize;

    /*================================================================
     * Restart loop
     *================================================================*/
    loop {
        /*── tabu list for this run ─*/
        let mut tabu = DualTabu::new(graph.n(), p.tenure_u, p.tenure_v);
        tabu.update_tenures(
            best_solution.size(),
            best_solution.edges(),
            p.gamma_target,
            rng,
        );

        /*── current solution state ─*/
        let mut cur = best_solution.clone();
        let mut run_best = cur.clone();
        let mut run_best_d = cur.density();
        let mut stagn = 0usize;

        /*============================================================
         * Inner loop – intensification & diversification
         *===========================================================*/
        loop {
            /* Intensification: one swap */
            if improve_once(
                &mut cur,
                &mut tabu,
                run_best_d,
                &mut freq,
                p,
                rng,
            ) {
                total_it += 1;
                let d = cur.density();
                if d > run_best_d {
                    run_best_d = d;
                    run_best = cur.clone();
                    stagn = 0;
                } else {
                    stagn += 1;
                }

                /* feasible? */
                if d + f64::EPSILON >= p.gamma_target {
                    return cur;
                }
                if total_it >= p.max_iter {
                    return global_best;
                }
                if stagn < p.stagnation_iter {
                    continue;
                }
            } else {
                stagn += 1; // no allowable swap (local optimum)
            }

            /* Diversification on stagnation */
            if stagn >= p.stagnation_iter {
                if rng.gen_bool(p.heavy_prob) {
                    heavy_perturbation(&mut cur, &mut tabu, rng, p, &mut freq);
                } else {
                    mild_perturbation(&mut cur, &mut tabu, rng, p, &mut freq); // <-- p toegevoegd
                }
                stagn = 0;
                total_it += 1;
                if total_it >= p.max_iter {
                    return global_best;
                }
                continue; // back to intensification
            }
            break; // inner loop finished without diversification
        }

        /*── update global best ─*/
        if run_best_d > global_best_d {
            global_best_d = run_best_d;
            global_best = run_best.clone();
        }
        if total_it >= p.max_iter {
            break;
        }

        /*================================================================
         * Build new initial solution using long-term frequency memory
         *================================================================*/
        // a) start vertex = least-frequent
        let min_f = *freq.iter().min().unwrap();
        let mut cand: Vec<usize> = (0..graph.n()).filter(|&v| freq[v] == min_f).collect();
        cand.shuffle(rng);
        let first = cand[0];

        let mut new_sol = Solution::new(graph);
        new_sol.add(first);

        // b) greedily add vertices (highest degree, low freq)
        while new_sol.size() < k {
            let mut max_deg = 0usize;
            for w in 0..graph.n() {
                if new_sol.bitset()[w] {
                    continue;
                }
                max_deg = max_deg.max(graph.degree(w));
            }
            let mut best_deg_verts = Vec::new();
            for w in 0..graph.n() {
                if new_sol.bitset()[w] || graph.degree(w) < max_deg {
                    continue;
                }
                best_deg_verts.push(w);
            }
            if best_deg_verts.is_empty() {
                break;
            }
            // prefer vertices with minimal frequency among the top-degree set
            let min_f2 = best_deg_verts.iter().map(|&v| freq[v]).min().unwrap();
            let mut best_choices: Vec<usize> =
                best_deg_verts.into_iter().filter(|&v| freq[v] == min_f2).collect();
            best_choices.shuffle(rng);
            new_sol.add(best_choices[0]);
        }

        // c) update frequencies
        for v in new_sol.bitset().iter_ones() {
            freq[v] += 1;
            if freq[v] > k {
                freq.fill(0);
            }
        }

        /*── prepare next run ─*/
        best_solution = new_sol;
        best_density = best_solution.density();
        if best_density + f64::EPSILON >= p.gamma_target {
            return best_solution;
        }
    }

    /* max_iter reached */
    global_best
}
