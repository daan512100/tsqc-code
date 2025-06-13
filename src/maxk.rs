//! Outer “max-k” search with a tight degree-upper-bound check.
//!
//! We iterate k = 2 … n.  For each k we first test whether the graph
//! *could* in principe contain a γ-quasi-clique of size k by comparing
//! a fast degree upper bound with the required number of edges.  If the
//! answer is “no”, we skip the expensive tabu search for that k.

use crate::{params::Params, restart::solve_fixed_k, solution::Solution, Graph};
use rand::Rng;

/*───────────────────────────────────────────────────────────────────────*/
/*  Pre-compute degree prefix sums                                       */
/*───────────────────────────────────────────────────────────────────────*/

fn degree_prefix_sums(graph: &Graph) -> Vec<usize> {
    let mut degs: Vec<usize> = (0..graph.n()).map(|v| graph.degree(v)).collect();
    degs.sort_unstable_by(|a, b| b.cmp(a));          // dalend
    let mut pref = Vec::with_capacity(degs.len() + 1);
    pref.push(0);
    let mut acc = 0usize;
    for d in degs {
        acc += d;
        pref.push(acc);
    }
    pref                                                 // pref[k] = som top-k graden
}

/* Upper-bound op #randen in een willekeurige k-subset */
#[inline]
fn ub_edges(prefix: &[usize], k: usize) -> usize {
    // Σ min{deg_i, k-1}  i=0..k-1
    // = Σ deg_i  −  Σ max(0, deg_i − (k-1))
    // Maar eenvoudiger: iterate top-k: cap at k-1
    let mut sum = 0usize;
    for i in 0..k {
        let capped = prefix[i + 1] - prefix[i];
        sum += capped.min(k - 1);
    }
    sum / 2                                             // iedere rand wordt twee keer geteld
}

/*───────────────────────────────────────────────────────────────────────*/
/*  Publieke solver                                                     */
/*───────────────────────────────────────────────────────────────────────*/

pub fn solve_maxk<'g, R>(graph: &'g Graph, rng: &mut R, p: &Params) -> Solution<'g>
where
    R: Rng + ?Sized,
{
    let pref = degree_prefix_sums(graph);

    let mut best_sol = Solution::new(graph);
    let mut best_d   = 0.0;
    let mut consecutive_fail = 0usize;

    for k in 2..=graph.n() {
        /*──────── quick impossibility check ────────*/
        let need_edges = ((p.gamma_target * (k * (k - 1) / 2) as f64).ceil()) as usize;
        let ub = ub_edges(&pref, k);
        if ub < need_edges {
            consecutive_fail += 1;
            if consecutive_fail >= 2 {
                break;                     // paper stop-regel
            }
            continue;                      // ga naar volgend k
        }

        /*──────── dure tabu-search ────────*/
        let sol_k = solve_fixed_k(graph, k, rng, p);
        let d     = sol_k.density();

        if d + f64::EPSILON >= p.gamma_target {
            consecutive_fail = 0;          // success
            if k > best_sol.size() || (k == best_sol.size() && d > best_d) {
                best_d   = d;
                best_sol = sol_k;
            }
        } else {
            if k > best_sol.size() {
             break;             
          }
            consecutive_fail += 1;
            if consecutive_fail >= 2 {
                break;
            }
        }
    }

    best_sol
}
