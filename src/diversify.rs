//! Diversification operators for TSQC (applied when the search is stuck in a local optimum).
//!
//! Heavy perturbation introduces a large disruption: it swaps out one vertex from the solution
//! for a very low-degree vertex not in the solution, producing a worse (lower-density) interim
//! solution to escape a local optimum. Mild perturbation is a smaller change: it swaps out a
//! “critical” vertex (one of the least connected in S) for a well-connected outsider, often
//! yielding only a slight decrease in density.  Both moves reset the tabu lists, and the search
//! then continues from the perturbed solution.

use crate::{solution::Solution, tabu::DualTabu, params::Params};
use rand::seq::SliceRandom;
use rand::Rng;

/*───────────────────────────────────────────────────────────────────*/
/*  Heavy perturbation                                               */
/*───────────────────────────────────────────────────────────────────*/

/// Heavy perturbation: remove one random vertex from S, then add an outsider with very few
/// connections to S.
///
/// A “low-degree” outside vertex is chosen (degree < *h* in the current S) such that the new
/// solution is worse (density decreases), helping the search jump to a new region.  The tabu
/// lists are **reset** after this move, clearing any short-term memory.  The parameter `p` is
/// used for `gamma_target` (quasi-clique density) in adaptive tenure updates.
pub fn heavy_perturbation<'g, R>(
    sol: &mut Solution<'g>,
    tabu: &mut DualTabu,
    rng: &mut R,
    p: &Params,
    freq: &mut [usize],
) where
    R: Rng + ?Sized,
{
    /* Guard */
    let k = sol.size();
    if k == 0 {
        return;
    }

    /* 1 ─ randomly remove one vertex from S */
    let mut inside: Vec<usize> = sol.bitset().iter_ones().collect();
    inside.shuffle(rng);
    let u = inside[0];
    sol.remove(u);

    /* 2 ─ determine threshold h for “low-degree” outsider.
     *     Heuristic: if the graph is very sparse, sqrt(k) may be too strict – use k^0.85. */
    let n = sol.graph().n();
    let graph_density = if n < 2 {
        0.0
    } else {
        2.0 * (sol.graph().m() as f64) / ((n * (n - 1)) as f64)
    };
    let mut h: f64 = if graph_density * (k as f64) <= 1.0 {
        // extremely sparse –- relax threshold
        (k as f64).powf(0.85)
    } else {
        (k as f64).sqrt()
    };
    h = h.clamp(1.0, k as f64 - 1.0).ceil();          // ensure 1 ≤ h ≤ k-1
    let h_thresh = h as usize;

    /* 3 ─ pick outsider with < h edges into current S */
    let mut outsiders: Vec<usize> =
        (0..sol.graph().n()).filter(|&v| !sol.bitset()[v]).collect();
    outsiders.shuffle(rng);

    let mut v_opt = None;
    for &w in &outsiders {
        let deg_in = sol
            .graph()
            .neigh_row(w)
            .iter_ones()
            .filter(|&j| sol.bitset()[j])
            .count();
        if deg_in < h_thresh {
            v_opt = Some(w);
            break;
        }
    }
    let v = v_opt.unwrap_or_else(|| {
        // no outsider below threshold – take one with minimal degree into S
        outsiders
            .iter()
            .copied()
            .min_by_key(|&w| {
                sol.graph()
                    .neigh_row(w)
                    .iter_ones()
                    .filter(|&j| sol.bitset()[j])
                    .count()
            })
            .unwrap()
    });
    sol.add(v);

    /* 4 ─ update long-term frequencies */
    for &vx in &[u, v] {
        freq[vx] += 1;
        if freq[vx] > k {
            freq.fill(0);
        }
    }

    /* 5 ─ adapt tabu tenures to new (worse) solution & reset lists */
    tabu.update_tenures(sol.size(), sol.edges(), p.gamma_target, rng);
    tabu.reset();
}

/*───────────────────────────────────────────────────────────────────*/
/*  Mild perturbation                                                */
/*───────────────────────────────────────────────────────────────────*/

/// Mild perturbation: swap worst vertex in S for best outsider (smallest drop in density).
///
/// Removes one critical vertex (lowest internal degree) and adds one outsider with the most
/// edges into S.  Often only slightly degrades density and provides gentle diversification.
pub fn mild_perturbation<'g, R>(
    sol: &mut Solution<'g>,
    tabu: &mut DualTabu,
    rng: &mut R,
    p: &Params,
    freq: &mut [usize],
) where
    R: Rng + ?Sized,
{
    /* Guard */
    if sol.size() == 0 {
        return;
    }

    /* 1 ─ identify worst vertex in S */
    let curr_d = sol.density();
    let crit_thr = (curr_d * ((sol.size() as f64) - 1.0)).floor() as usize;

    let mut worst_v = None;
    let mut worst_deg = usize::MAX;

    for u in sol.bitset().iter_ones() {
        let deg_in = sol
            .graph()
            .neigh_row(u)
            .iter_ones()
            .filter(|&j| sol.bitset()[j])
            .count();
        if deg_in < crit_thr && deg_in < worst_deg {
            worst_deg = deg_in;
            worst_v = Some(u);
        }
    }
    // if no vertex is strictly critical, take one with minimal internal degree anyway
    if worst_v.is_none() {
        for u in sol.bitset().iter_ones() {
            let deg_in = sol
                .graph()
                .neigh_row(u)
                .iter_ones()
                .filter(|&j| sol.bitset()[j])
                .count();
            if deg_in < worst_deg {
                worst_deg = deg_in;
                worst_v = Some(u);
            }
        }
    }
    let u = worst_v.expect("S non-empty, so a vertex must exist");
    sol.remove(u);

    /* 2 ─ outsider with max connections into new S */
    let mut max_edges = 0usize;
    for w in 0..sol.graph().n() {
        if sol.bitset()[w] {
            continue;
        }
        let edges_in = sol
            .graph()
            .neigh_row(w)
            .iter_ones()
            .filter(|&j| sol.bitset()[j])
            .count();
        max_edges = max_edges.max(edges_in);
    }
    let mut best_outsiders = Vec::new();
    for w in 0..sol.graph().n() {
        if sol.bitset()[w] {
            continue;
        }
        let edges_in = sol
            .graph()
            .neigh_row(w)
            .iter_ones()
            .filter(|&j| sol.bitset()[j])
            .count();
        if edges_in == max_edges {
            best_outsiders.push(w);
        }
    }
    best_outsiders.shuffle(rng);
    let v = best_outsiders[0];
    sol.add(v);

    /* 3 ─ update frequencies */
    for &vx in &[u, v] {
        freq[vx] += 1;
        if freq[vx] > sol.size() {
            freq.fill(0);
        }
    }

    /* 4 ─ update tabu & reset */
    tabu.update_tenures(sol.size(), sol.edges(), p.gamma_target, rng);
    tabu.reset();
}
