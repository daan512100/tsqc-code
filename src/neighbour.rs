//! Intensification: explore the swap neighborhood (Algorithm 1 of TSQC).
//!
//! In each iteration, TSQC tries to swap one vertex *u* ∈ S with one vertex *w* ∉ S. 
//! We define set **A** as the vertices in S with minimum internal degree, and set **B** as the 
//! vertices outside S with maximum degree (relative to S):contentReference[oaicite:62]{index=62}:contentReference[oaicite:63]{index=63}. 
//! The algorithm selects the swap that *does not decrease* the density (Δf ≥ 0) and yields the 
//! largest improvement if any. If no non-deteriorating swap is available (local optimum), 
//! intensification stops. A tabu move is allowed under the **aspiration** criterion if it produces 
//! a solution with higher density than any seen so far:contentReference[oaicite:64]{index=64}. Dual tabu lists prevent immediately 
//! undoing the last move unless such an aspirational move occurs.

use crate::{solution::Solution, tabu::DualTabu, params::Params};
use rand::Rng;

/// Apply one intensification step (one swap move) if possible. 
/// 
/// Examines potential swaps of one in-set vertex `u` (from the current solution) with one 
/// out-of-set vertex `w`. The swap that yields the highest density (with **non-negative** gain in edges) 
/// is chosen and executed. If a move is tabu, it will still be executed under the aspiration criterion 
/// if it achieves a greater density than `global_best_density` (the best density found so far). 
/// Returns `true` if a swap was performed (improving or equal-density move), or `false` if no allowable 
/// swap could improve or maintain the density (i.e., a local optimum is reached).
pub fn improve_once<'g, R>(
    sol: &mut Solution<'g>,
    tabu: &mut DualTabu,
    global_best_density: f64,
    freq: &mut [usize],
    p: &Params,
    rng: &mut R
) -> bool 
where
    R: Rng + ?Sized,
{
    let g = sol.graph();
    let k = sol.size();
    let m = sol.edges();
    let current_density = sol.density();

    // Identify critical vertices in S (those with degree < ⌊ρ*(|S|-1)⌋ in the current solution)
    let crit_thr = (current_density * ((k as f64) - 1.0)).floor() as usize;
    let mut critical_vertices: Vec<usize> = Vec::new();
    for u in sol.bitset().iter_ones() {
        let deg_in = g.neigh_row(u)
            .iter_ones()
            .filter(|&j| sol.bitset()[j])
            .count();
        if deg_in < crit_thr {
            critical_vertices.push(u);
        }
    }
    if critical_vertices.is_empty() {
        // No critical vertices (solution is very dense or |S| < 2); no beneficial swap exists
        // because any swap would likely remove a well-connected vertex.
        // Intensification cannot find a >=0 move in this case.
        tabu.step();  // count this as an iteration (no move made)
        return false;
    }

    // Try all swap combinations (u in A, w outside) to find the best allowed move
    let mut best_allowed: Option<(f64, usize, usize)> = None;
    let mut best_aspirant: Option<(f64, usize, usize)> = None;
    for &u in &critical_vertices {
        // Compute loss in edges if u is removed
        let loss_u = g.neigh_row(u)
            .iter_ones()
            .filter(|&j| sol.bitset()[j])
            .count();
        // Loop over all outside vertices w
        for w in 0..g.n() {
            if sol.bitset()[w] {
                continue; // skip vertices already in S
            }
            // Compute gain in edges if w is added (count neighbors of w in current S, excluding u)
            let gain_w = g.neigh_row(w)
                .iter_ones()
                .filter(|&j| j != u && sol.bitset()[j])
                .count();
            // Check the net effect on edge count
            if gain_w >= loss_u {
                // This swap does not decrease the number of edges (Δm >= 0, so Δρ >= 0 since |S| is constant)
                let new_m = m - loss_u + gain_w;
                let new_density = if k < 2 {
                    0.0 
                } else {
                    // Compute density of the would-be solution S' after the swap
                    2.0 * (new_m as f64) / ((k * (k - 1)) as f64)
                };
                let move_is_tabu = tabu.is_tabu_v(u) || tabu.is_tabu_u(w);
                if !move_is_tabu {
                    // Non-tabu swap candidate
                    if best_allowed.is_none() || new_density > best_allowed.as_ref().unwrap().0 {
                        best_allowed = Some((new_density, u, w));
                    }
                } else if new_density > global_best_density {
                    // Tabu move, but qualifies under aspiration (would exceed best global density seen)
                    if best_aspirant.is_none() || new_density > best_aspirant.as_ref().unwrap().0 {
                        best_aspirant = Some((new_density, u, w));
                    }
                }
            }
        }
    }

    // Decide which move to execute, if any
    let chosen_move = if let Some((_, u, w)) = best_allowed {
        // Execute the best allowed swap (non-negative Δρ)
        Some((u, w))
    } else if let Some((_, u, w)) = best_aspirant {
        // No allowed move improved density, but a tabu move can improve the global best – use aspiration
        Some((u, w))
    } else {
        None
    };

    if let Some((u, w)) = chosen_move {
        // Perform the swap: remove u from S and add w to S
        sol.remove(u);
        sol.add(w);
        // Update long-term frequencies for u and w
        freq[u] += 1;
        if freq[u] > p.stagnation_iter as usize {  // using stagnation_iter as a safe upper bound (k times might be too strict if k is small)
            freq.fill(0);
        }
        freq[w] += 1;
        if freq[w] > p.stagnation_iter as usize {
            freq.fill(0);
        }
        // Update tabu tenures based on the new solution state (|S| unchanged, but edges may change)
        tabu.update_tenures(sol.size(), sol.edges(), p.gamma_target, rng);
        // Mark this swap in the tabu lists: 
        // - u (just removed) is forbidden to re-enter S for Tu iterations 
        // - w (just added) is forbidden to leave S for Tv iterations
        tabu.forbid_v(u);
        tabu.forbid_u(w);
        tabu.step();  // advance the tabu list's iteration counter
        return true;
    } else {
        // No swap was found that improves or maintains density – local optimum reached for now.
        tabu.step();
        return false;
    }
}

// (The improve_until_local_optimum function from the original code has been removed, 
// as we now handle the intensification loop explicitly in solve_fixed_k for better control.)
