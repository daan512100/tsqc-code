//! Diversification operators for TSQC.

use crate::{solution::Solution, tabu::DualTabu};
use rand::seq::SliceRandom;
use rand::Rng;

/// Heavy perturbation: remove ⌈γ·|S|⌉ random vertices, then greedily add
/// the same number of highest-degree outsiders. Resets tabu.
pub fn heavy_perturbation<'g, R>(
    sol: &mut Solution<'g>,
    tabu: &mut DualTabu,
    rng: &mut R,
    gamma: f64,
) where
    R: Rng + ?Sized,
{
    let k = sol.size();
    if k == 0 { return; }
    let remove_cnt = ((gamma.clamp(0.1, 0.9) * k as f64).ceil() as usize).min(k);

    /* randomly pick vertices to remove */
    let mut inside: Vec<usize> = sol.bitset().iter_ones().collect();
    inside.shuffle(rng);
    for &v in &inside[..remove_cnt] {
        sol.remove(v);
    }

    /* add highest-degree outsiders (could include previously removed ones) */
    let mut outsiders: Vec<usize> =
        (0..sol.graph().n()).filter(|&v| !sol.bitset()[v]).collect();
    outsiders.sort_unstable_by_key(|&v| std::cmp::Reverse(sol.graph().degree(v)));
    for &v in outsiders.iter().take(remove_cnt) {
        sol.add(v);
    }

    tabu.reset();
}

/// Mild perturbation: drop worst critical vertex, add best outsider.
pub fn mild_perturbation<'g>(sol: &mut Solution<'g>, tabu: &mut DualTabu) {
    let curr_d   = sol.density();
    let crit_thr = (curr_d * (sol.size() as f64 - 1.0)).floor() as usize;

    /* worst critical vertex = lowest internal degree < threshold */
    let mut worst: Option<(usize /*deg*/, usize /*v*/)> = None;
    for v in sol.bitset().iter_ones() {
        let deg_in = sol.graph().neigh_row(v)
            .iter_ones()
            .filter(|&j| sol.bitset()[j])
            .count();
        if deg_in < crit_thr && worst.map_or(true, |(d, _)| deg_in < d) {
            worst = Some((deg_in, v));
        }
    }
    let (_, u) = match worst { Some(p) => p, None => return };

    sol.remove(u);                    // end immutable borrows

    /* best outsider by #edges into current S */
    let mut best: Option<(usize, usize)> = None;
    for w in 0..sol.graph().n() {
        if sol.bitset()[w] { continue; }
        let edges = sol.graph().neigh_row(w)
            .iter_ones()
            .filter(|&j| sol.bitset()[j])
            .count();
        if best.map_or(true, |(e, _)| edges > e) {
            best = Some((edges, w));
        }
    }
    if let Some((_, w)) = best { sol.add(w); }

    tabu.reset();
}

/*────────────────── tests ──────────────────*/
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{construct::greedy_k, graph::Graph, tabu::DualTabu};
    use rand_chacha::ChaCha8Rng;
    use rand::SeedableRng;
    use std::io::Cursor;

    fn square() -> Graph {
        let dimacs = b"p edge 4 4\ne 1 2\ne 2 3\ne 3 4\ne 4 1\n";
        Graph::parse_dimacs(Cursor::new(dimacs)).unwrap()
    }

    #[test]
    fn heavy_keeps_size() {
        let g = square();
        let mut sol  = greedy_k(&g, 3);
        let mut tabu = DualTabu::new(g.n(), 2, 2);
        let before_k = sol.size();

        let mut rng = ChaCha8Rng::seed_from_u64(7);
        heavy_perturbation(&mut sol, &mut tabu, &mut rng, 0.5);

        // Heavy perturbation must keep |S| constant.
        assert_eq!(sol.size(), before_k);
    }

    #[test]
    fn mild_keeps_size() {
        let g = square();
        let mut sol  = greedy_k(&g, 3);
        let mut tabu = DualTabu::new(g.n(), 2, 2);
        let before_k = sol.size();

        mild_perturbation(&mut sol, &mut tabu);

        assert_eq!(sol.size(), before_k);
    }
}