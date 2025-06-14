// src/neighbour.rs
//!
//! Intensification (one-swap) for TSQC (§3.4.1).
//! Builds critical sets A (min internal deg) and B (max external deg),
//! scans all (u∈A, v∈B) for the best non-deteriorating or aspirational
//! swap, executes it, updates frequency memory, steps the tabu clocks,
//! and adapts tabu tenures.

use crate::{params::Params, solution::Solution, tabu::DualTabu, Graph};
use rand::Rng;

/// Attempt a single intensification move.  
/// - `best_global_rho`: best density seen so far (for aspiration).  
/// - `freq`: long‐term frequency memory (increment for any swapped u/v).  
/// Returns `true` if a swap was performed.
pub fn improve_once<'g, R>(
    sol: &mut Solution<'g>,
    tabu: &mut DualTabu,
    best_global_rho: f64,
    freq: &mut Vec<usize>,
    p: &Params,
    rng: &mut R,
) -> bool
where
    R: Rng + ?Sized,
{
    let graph = sol.graph();
    let k = sol.size();
    // trivial if nothing to swap
    if k < 1 || k > graph.n() {
        return false;
    }

    let m_cur = sol.edges();
    let max_edges = k.saturating_mul(k.saturating_sub(1)) / 2;

    // 1) compute MinInS and MaxOutS
    let mut min_in = usize::MAX;
    for u in sol.bitset().iter_ones() {
        let deg_in = graph
            .neigh_row(u)
            .iter_ones()
            .filter(|&j| sol.bitset()[j])
            .count();
        min_in = min_in.min(deg_in);
    }
    let mut max_out = 0;
    for v in 0..graph.n() {
        if sol.bitset()[v] { continue; }
        let deg_out = graph
            .neigh_row(v)
            .iter_ones()
            .filter(|&j| sol.bitset()[j])
            .count();
        max_out = max_out.max(deg_out);
    }

    // 2) build critical sets A and B
    let mut A = Vec::new();
    for u in sol.bitset().iter_ones() {
        let deg_in = graph
            .neigh_row(u)
            .iter_ones()
            .filter(|&j| sol.bitset()[j])
            .count();
        if deg_in == min_in && !tabu.is_tabu_u(u) {
            A.push(u);
        }
    }
    let mut B = Vec::new();
    for v in 0..graph.n() {
        if sol.bitset()[v] { continue; }
        let deg_out = graph
            .neigh_row(v)
            .iter_ones()
            .filter(|&j| sol.bitset()[j])
            .count();
        if deg_out == max_out && !tabu.is_tabu_v(v) {
            B.push(v);
        }
    }

    // 3) scan A×B for best allowed (non-deteriorating) or aspirational swap
    let mut best_allowed: Option<(f64, usize, usize)> = None;
    let mut best_aspire:  Option<(f64, usize, usize)> = None;

    for &u in &A {
        // loss = how many edges we lose by removing u
        let loss = graph
            .neigh_row(u)
            .iter_ones()
            .filter(|&j| sol.bitset()[j])
            .count();

        for &v in &B {
            // gain = how many edges we gain by adding v
            let gain = graph
                .neigh_row(v)
                .iter_ones()
                .filter(|&j| sol.bitset()[j])
                .count();

            // new total edges and density
            let m_new = m_cur + gain.saturating_sub(loss);
            let rho_new = (m_new as f64) / (max_edges as f64);

            let forbidden = tabu.is_tabu_u(u) || tabu.is_tabu_v(v);

            if !forbidden && gain >= loss {
                // non-deteriorating allowed swap
                if best_allowed
                    .as_ref()
                    .map_or(true, |&(r, _, _)| rho_new >= r)
                {
                    best_allowed = Some((rho_new, u, v));
                }
            } else if forbidden && rho_new > best_global_rho {
                // aspiration over global best density
                if best_aspire
                    .as_ref()
                    .map_or(true, |&(r, _, _)| rho_new > r)
                {
                    best_aspire = Some((rho_new, u, v));
                }
            }
        }
    }

    // choose aspirational if no allowed
    let chosen = best_allowed.or(best_aspire);

    // 4) execute swap if found
    let did_swap = if let Some((_, u, v)) = chosen {
        sol.remove(u);
        sol.add(v);

        // update long-term frequency memory
        freq[u] = freq[u].saturating_add(1);
        freq[v] = freq[v].saturating_add(1);

        // mark tabu for u,v
        tabu.forbid_u(u);
        tabu.forbid_v(v);
        true
    } else {
        false
    };

    // 5) advance tabu clock and adapt tenures
    tabu.step();
    tabu.update_tenures(sol.size(), sol.edges(), p.gamma_target, rng);

    did_swap
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand_chacha::ChaCha8Rng;
    use rand::SeedableRng;

    #[test]
    fn intensification_swaps() {
        // Build a tiny 4‐node graph where {0,1,2} is a 3‐quasi‐clique
        let edges = &[(0,1),(1,2),(0,2),(2,3)];
        let graph = Graph::from_edge_list(4, edges);
        let mut sol = Solution::new(&graph);
        sol.add(0); sol.add(1); sol.add(3); // initial
        let mut rng = ChaCha8Rng::seed_from_u64(0);
        let mut tabu = DualTabu::new(4, 1, 1);
        let mut freq = vec![0; 4];
        let mut p = Params::default();
        p.gamma_target = 0.5;

        let before = sol.density();
        let did = improve_once(&mut sol, &mut tabu, before, &mut freq, &p, &mut rng);
        assert!(did, "Should perform at least one swap");
        assert!(sol.density() >= before);
    }
}
