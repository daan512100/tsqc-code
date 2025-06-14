// src/diversify.rs
//! Adaptive diversification (heavy vs. mild perturbations) for TSQC (§ 3.4.2).
//!
//! ▲ heavy_perturbation: “large shake”
//! ▲ mild_perturbation: “small shake”
//!
//! After each perturbation we:
//!  1. Increment long‐term frequency memory for swapped vertices.
//!  2. Reset the tabu lists.
//!  3. Recompute tabu tenures based on the new solution.

use crate::{params::Params, solution::Solution, tabu::DualTabu, Graph};
use rand::seq::SliceRandom;
use rand::Rng;
use std::f64;

/// Heavy perturbation (“large shake”):
/// 1. Remove a random vertex `u` ∈ S.
/// 2. Compute threshold `h = ⌈k^0.85⌉ if graph density ≥ 0.5 else ⌈k^0.5⌉`.
/// 3. Collect outsiders `v ∉ S` with `deg_in(v) < h`; if none, take those with minimal `deg_in`.
/// 4. Add one randomly chosen `v`.
/// 5. Increment `freq[u]` and `freq[v]`; if any `> k`, reset all to 0.
/// 6. Clear tabu lists and then update tenures.
pub fn heavy_perturbation<'g, R>(
    sol: &mut Solution<'g>,
    tabu: &mut DualTabu,
    rng: &mut R,
    p: &Params,
    freq: &mut Vec<usize>,
) where
    R: Rng + ?Sized,
{
    let k = sol.size();
    if k < 1 {
        return;
    }

    // 1) pick and remove random u ∈ S
    let u = *sol
        .bitset()
        .iter_ones()
        .collect::<Vec<_>>()
        .choose(rng)
        .expect("Solution must be non-empty");
    sol.remove(u);

    // 2) threshold h, density‐based
    let n = sol.graph().n();
    let m = sol.graph().m();
    let dn = 2.0 * (m as f64) / ((n * (n - 1) / 2) as f64);
    let h = if dn >= 0.5 {
        (k as f64).powf(0.85).ceil() as usize
    } else {
        (k as f64).sqrt().ceil() as usize
    }
    .clamp(1, k.saturating_sub(1));

    // 3) collect outsiders
    let outsiders: Vec<usize> = (0..n).filter(|&v| !sol.bitset()[v]).collect();

    let mut candidates: Vec<usize> = outsiders
        .iter()
        .copied()
        .filter(|&v| {
            sol.graph()
                .neigh_row(v)
                .iter_ones()
                .filter(|&j| sol.bitset()[j])
                .count()
                < h
        })
        .collect();

    // fallback to minimal deg_in if none < h
    if candidates.is_empty() {
        let min_deg = outsiders
            .iter()
            .map(|&v| {
                sol.graph()
                    .neigh_row(v)
                    .iter_ones()
                    .filter(|&j| sol.bitset()[j])
                    .count()
            })
            .min()
            .unwrap_or(0);
        candidates = outsiders
            .into_iter()
            .filter(|&v| {
                sol.graph()
                    .neigh_row(v)
                    .iter_ones()
                    .filter(|&j| sol.bitset()[j])
                    .count()
                    == min_deg
            })
            .collect();
    }

    // 4) add random v
    let &v = candidates
        .choose(rng)
        .expect("At least one outsider must exist");
    sol.add(v);

    // 5) update frequency memory
    freq[u] = freq[u].saturating_add(1);
    freq[v] = freq[v].saturating_add(1);
    if freq[u] > k || freq[v] > k {
        freq.fill(0);
    }

    // 6) reset tabu and update tenures
    tabu.reset();
    tabu.update_tenures(sol.size(), sol.edges(), p.gamma_target, rng);
}

/// Mild perturbation (“small shake”):
/// 1. Build critical sets A (u ∈ S with minimal deg_in) and B (v ∉ S with maximal deg_in).
/// 2. Pick random `u ∈ A`, `v ∈ B` and swap them.
/// 3. Increment `freq[u]` and `freq[v]`; if any `> k`, reset all to 0.
/// 4. Clear tabu lists and then update tenures.
pub fn mild_perturbation<'g, R>(
    sol: &mut Solution<'g>,
    tabu: &mut DualTabu,
    rng: &mut R,
    p: &Params,
    freq: &mut Vec<usize>,
) where
    R: Rng + ?Sized,
{
    let k = sol.size();
    if k < 1 {
        return;
    }
    let graph = sol.graph();
    let n = graph.n();

    // 1) critical set A: u ∈ S of minimal internal degree
    let mut min_in = usize::MAX;
    for u in sol.bitset().iter_ones() {
        let d = graph
            .neigh_row(u)
            .iter_ones()
            .filter(|&j| sol.bitset()[j])
            .count();
        min_in = min_in.min(d);
    }
    let A: Vec<usize> = sol
        .bitset()
        .iter_ones()
        .filter(|&u| {
            graph
                .neigh_row(u)
                .iter_ones()
                .filter(|&j| sol.bitset()[j])
                .count()
                == min_in
        })
        .collect();

    // 2) critical set B: v ∉ S of maximal internal degree into S
    let mut max_out = 0;
    for v in 0..n {
        if sol.bitset()[v] {
            continue;
        }
        let d = graph
            .neigh_row(v)
            .iter_ones()
            .filter(|&j| sol.bitset()[j])
            .count();
        max_out = max_out.max(d);
    }
    let B: Vec<usize> = (0..n)
        .filter(|&v| {
            !sol.bitset()[v]
                && graph
                    .neigh_row(v)
                    .iter_ones()
                    .filter(|&j| sol.bitset()[j])
                    .count()
                    == max_out
        })
        .collect();

    // swap random u∈A, v∈B
    let &u = A.choose(rng).expect("A must be non-empty");
    let &v = B.choose(rng).expect("B must be non-empty");
    sol.remove(u);
    sol.add(v);

    // 3) update frequency memory
    freq[u] = freq[u].saturating_add(1);
    freq[v] = freq[v].saturating_add(1);
    if freq[u] > k || freq[v] > k {
        freq.fill(0);
    }

    // 4) reset tabu and update tenures
    tabu.reset();
    tabu.update_tenures(sol.size(), sol.edges(), p.gamma_target, rng);
}
