//! Constructors for an initial subset *S*.
//!
//! • `random_k`
//! • `greedy_k`
//! • `greedy_random_k`
//! • `greedy_until_gamma` – grow until density ≥ γ and can’t be enlarged
//!
//! All functions return a ready-to-use [`Solution`].

use crate::{graph::Graph, solution::Solution};
use rand::seq::SliceRandom;
use rand::Rng;

/*───────────────────────────────────────────────────────────*/
/*  Random-k                                                 */
/*───────────────────────────────────────────────────────────*/

pub fn random_k<'g, R>(graph: &'g Graph, k: usize, rng: &mut R) -> Solution<'g>
where
    R: Rng + ?Sized,
{
    assert!(k <= graph.n());
    let mut idx: Vec<usize> = (0..graph.n()).collect();
    idx.shuffle(rng);

    let mut sol = Solution::new(graph);
    for &v in &idx[..k] {
        sol.add(v);
    }
    sol
}

/*───────────────────────────────────────────────────────────*/
/*  Greedy-k                                                 */
/*───────────────────────────────────────────────────────────*/

pub fn greedy_k<'g>(graph: &'g Graph, k: usize) -> Solution<'g> {
    assert!(k <= graph.n());
    let mut idx: Vec<usize> = (0..graph.n()).collect();
    idx.sort_unstable_by_key(|&v| std::cmp::Reverse(graph.degree(v)));

    let mut sol = Solution::new(graph);
    for &v in &idx[..k] {
        sol.add(v);
    }
    sol
}

/*───────────────────────────────────────────────────────────*/
/*  Greedy-random-k                                          */
/*───────────────────────────────────────────────────────────*/

pub fn greedy_random_k<'g, R>(graph: &'g Graph, k: usize, rng: &mut R) -> Solution<'g>
where
    R: Rng + ?Sized,
{
    assert!(k <= graph.n());

    let mut sol = Solution::new(graph);
    sol.add(rng.gen_range(0..graph.n())); // random seed

    while sol.size() < k {
        let mut best_edges = 0usize;
        let mut cand       = Vec::new();

        for v in 0..graph.n() {
            if sol.bitset()[v] { continue; }
            let edges = graph.neigh_row(v)
                .iter_ones()
                .filter(|&u| sol.bitset()[u])
                .count();
            if edges > best_edges {
                best_edges = edges;
                cand.clear();
                cand.push(v);
            } else if edges == best_edges {
                cand.push(v);
            }
        }
        sol.add(*cand.choose(rng).unwrap());
    }
    sol
}

/*───────────────────────────────────────────────────────────*/
/*  Greedy until γ-density cannot grow further               */
/*───────────────────────────────────────────────────────────*/

/// Grow *S* while density remains ≥ `gamma`.  
/// Uses the **internal-neighbour** count (as in the thesis) and random
/// tie-breaking every iteration.  Stops when inserting any outsider
/// would drop the density below `gamma`.
pub fn greedy_until_gamma<'g, R>(
    graph: &'g Graph,
    gamma: f64,
    rng: &mut R,
) -> Solution<'g>
where
    R: Rng + ?Sized,
{
    assert!((0.0..=1.0).contains(&gamma));

    /*── start with a random edge (or two random vertices) ──────────*/
    let mut sol = Solution::new(graph);
    let mut verts: Vec<usize> = (0..graph.n()).collect();
    verts.shuffle(rng);

    let mut edge_found = false;
    'edge: for &u in &verts {
        for &v in &verts {
            if u < v && graph.neigh_row(u)[v] {
                sol.add(u);
                sol.add(v);
                edge_found = true;
                break 'edge;
            }
        }
    }
    if !edge_found {
        sol.add(verts[0]);
        sol.add(verts[1]);
    }

    /*── greedy expansion with shuffle each round ───────────────────*/
    loop {
        let mut outsiders: Vec<usize> =
            (0..graph.n()).filter(|&v| !sol.bitset()[v]).collect();
        if outsiders.is_empty() { break; }

        outsiders.shuffle(rng);   // random tie-break every iteration

        // compute max neighbour count inside S
        let mut best_edges = 0usize;
        for &v in &outsiders {
            let e = graph.neigh_row(v)
                .iter_ones()
                .filter(|&u| sol.bitset()[u])
                .count();
            best_edges = best_edges.max(e);
        }

        // collect all outsiders achieving that max
        let mut cand: Vec<usize> = outsiders.into_iter()
            .filter(|&v| {
                graph.neigh_row(v)
                    .iter_ones()
                    .filter(|&u| sol.bitset()[u])
                    .count() == best_edges
            })
            .collect();
        if cand.is_empty() { break; }

        let v = *cand.choose(rng).unwrap();
        sol.add(v);
        if sol.density() + f64::EPSILON < gamma {
            sol.remove(v);
            break;          // no further vertex can be inserted
        }
    }

    /*── last-chance scan: try any remaining outsiders once ─────────*/
    let mut outsiders: Vec<usize> =
        (0..graph.n()).filter(|&v| !sol.bitset()[v]).collect();
    outsiders.shuffle(rng);
    for v in outsiders {
        sol.add(v);
        if sol.density() + f64::EPSILON < gamma {
            sol.remove(v);
        }
    }

    sol
}

/*──────────────────────── tests ───────────────────────────*/

#[cfg(test)]
mod tests {
    use super::*;
    use rand_chacha::ChaCha8Rng;
    use rand::SeedableRng;
    use std::io::Cursor;

    fn triangle() -> Graph {
        let dimacs = b"p edge 3 3\ne 1 2\ne 1 3\ne 2 3\n";
        Graph::parse_dimacs(Cursor::new(dimacs)).unwrap()
    }

    #[test]
    fn until_gamma_maximal() {
        let g = triangle();
        let mut rng = ChaCha8Rng::seed_from_u64(7);
        let sol = greedy_until_gamma(&g, 0.8, &mut rng);
        assert!(sol.density() >= 0.8);
        for v in 0..g.n() {
            if sol.bitset()[v] { continue; }
            let mut tmp = sol.clone();
            tmp.add(v);
            assert!(tmp.density() < 0.8);
        }
    }
}
