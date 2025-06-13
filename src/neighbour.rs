//! Critical-set add / remove / swap neighbourhood (Algorithm 1)
//!
//! Move order per iteration (best-improving first):
//!   1. ADD       – insert outsider w that maximises Δρ
//!   2. REMOVE    – drop a “critical” vertex u (deg(u) < ⌊ρ(|S|-1)⌋)
//!   3. SWAP      – replace critical u with outsider w
//!
//! The first move that improves either the current density or the global
//! best (aspiration) is executed.  Dual-tabu lists prevent immediate
//! reversal unless the aspiration criterion fires.

use crate::{solution::Solution, tabu::DualTabu};

/// Apply **one** intensification step; returns `true` if `sol` improved.
pub fn improve_once<'g>(
    sol: &mut Solution<'g>,
    tabu: &mut DualTabu,
    global_best_density: f64,
) -> bool {
    tabu.update_tenures(sol.size());          // adaptive tenures

    let g  = sol.graph();
    let k  = sol.size();
    let m  = sol.edges();
    let d  = sol.density();

    /* helpers --------------------------------------------------------- */
    let dens_after = |m_new: usize, k_new: usize| -> f64 {
        if k_new < 2 { 0.0 } else { 2.0 * m_new as f64 / (k_new * (k_new - 1)) as f64 }
    };

    let dens_if_add = |w: usize| -> f64 {
        let gain = g.neigh_row(w)
            .iter_ones()
            .filter(|&j| sol.bitset()[j])
            .count();
        dens_after(m + gain, k + 1)
    };

    let dens_if_rem = |u: usize| -> f64 {
        let loss = g.neigh_row(u)
            .iter_ones()
            .filter(|&j| sol.bitset()[j])
            .count();
        dens_after(m - loss, k - 1)
    };

    /* identify critical vertices (internal degree below threshold) ---- */
    let crit_thr = (d * (k as f64 - 1.0)).floor() as usize;
    let critical: Vec<usize> = sol.bitset()
        .iter_ones()
        .filter(|&u| {
            let deg_in = g.neigh_row(u)
                .iter_ones()
                .filter(|&j| sol.bitset()[j])
                .count();
            deg_in < crit_thr
        })
        .collect();

    /* 1 ── ADD -------------------------------------------------------- */
    for w in 0..g.n() {
        if sol.bitset()[w] || tabu.is_tabu_u(w) { continue; }
        let d_new = dens_if_add(w);
        if d_new > d || d_new > global_best_density {
            sol.add(w);
            tabu.forbid_u(w);
            tabu.step();
            return true;
        }
    }

    /* 2 ── REMOVE (critical) ----------------------------------------- */
    for &u in &critical {
        if tabu.is_tabu_v(u) { continue; }
        let d_new = dens_if_rem(u);
        if d_new > d || d_new > global_best_density {
            sol.remove(u);
            tabu.forbid_v(u);
            tabu.step();
            return true;
        }
    }

    /* 3 ── SWAP critical u with outsider w --------------------------- */
    for &u in &critical {
        if tabu.is_tabu_v(u) { continue; }

        let loss_u = g.neigh_row(u)
            .iter_ones()
            .filter(|&j| sol.bitset()[j])
            .count();

        for w in 0..g.n() {
            if sol.bitset()[w] || tabu.is_tabu_u(w) { continue; }

            let gain_w = g.neigh_row(w)
                .iter_ones()
                .filter(|&j| j != u && sol.bitset()[j])
                .count();

            let d_new = dens_after(m - loss_u + gain_w, k);

            if d_new > d || d_new > global_best_density {
                sol.remove(u);
                sol.add(w);
                tabu.forbid_v(u);
                tabu.forbid_u(w);
                tabu.step();
                return true;
            }
        }
    }

    /* no improving move ------------------------------------------------ */
    tabu.step();
    false
}

/// Keep calling [`improve_once`] until no improving neighbour exists.
pub fn improve_until_local_optimum<'g>(
    sol: &mut Solution<'g>,
    tabu: &mut DualTabu,
    global_best_density: f64,
) {
    while improve_once(sol, tabu, global_best_density) {}
}
