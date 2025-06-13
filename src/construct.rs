//! Constructors for an initial vertex subset S₀.
//!
//! * `random_k` – choose k distinct vertices uniformly at random.
//! * `greedy_k` – take the k highest-degree vertices.
//! * `greedy_random_k` – greedy heuristic with a random start (for TSQC initial solution).
//! 
//! All these constructors return a ready-to-use [`Solution`] with edge counts pre-computed.

use crate::{graph::Graph, solution::Solution};
use rand::seq::SliceRandom;
use rand::Rng;

/// Pick `k` distinct vertices uniformly at random.
///
/// `rng` is any mutable RNG; `k` must not exceed `graph.n()`.
pub fn random_k<'g, R>(graph: &'g Graph, k: usize, rng: &mut R) -> Solution<'g>
where
    R: Rng + ?Sized,
{
    assert!(k <= graph.n(), "k larger than graph size");
    let mut idx: Vec<usize> = (0..graph.n()).collect();
    idx.shuffle(rng);

    let mut sol = Solution::new(graph);
    for &v in &idx[..k] {
        sol.add(v);
    }
    sol
}

/// Greedy initialization: take the `k` highest-degree vertices.
pub fn greedy_k<'g>(graph: &'g Graph, k: usize) -> Solution<'g> {
    assert!(k <= graph.n(), "k larger than graph size");
    let mut idx: Vec<usize> = (0..graph.n()).collect();
    idx.sort_unstable_by_key(|&v| std::cmp::Reverse(graph.degree(v)));

    let mut sol = Solution::new(graph);
    for &v in &idx[..k] {
        sol.add(v);
    }
    sol
}

/// Greedy-random initialization: start with one random vertex, then iteratively add the vertex 
/// with the most neighbors in the current set (tie-break randomly) until size `k`.
///
/// This heuristic injects randomness into the construction of the initial subset S₀, as described 
/// in TSQC §3.3:contentReference[oaicite:60]{index=60}:contentReference[oaicite:61]{index=61}. It helps generate diverse starting solutions rather 
/// than always beginning with the same highest-degree vertices.
pub fn greedy_random_k<'g, R>(graph: &'g Graph, k: usize, rng: &mut R) -> Solution<'g>
where
    R: Rng + ?Sized,
{
    assert!(k <= graph.n(), "k larger than graph size");
    let mut sol = Solution::new(graph);
    if k == 0 {
        return sol;
    }
    // Step 1: add one random vertex as the starting seed
    let v0 = rng.gen_range(0..graph.n());
    sol.add(v0);
    // Step 2: Iteratively add the vertex with the most neighbors in the current set
    // Continue until the solution reaches size k.
    while sol.size() < k {
        let mut best_neighbor_count = 0;
        let mut best_candidates: Vec<usize> = Vec::new();
        // Evaluate each outsider vertex by how many connections it has into the current solution
        for w in 0..graph.n() {
            if sol.bitset()[w] {
                continue;
            }
            let neighbors_in_sol = graph.neigh_row(w)
                .iter_ones()
                .filter(|&u| sol.bitset()[u])
                .count();
            if neighbors_in_sol > best_neighbor_count {
                // Found a new outsider with a higher connection count
                best_neighbor_count = neighbors_in_sol;
                best_candidates.clear();
                best_candidates.push(w);
            } else if neighbors_in_sol == best_neighbor_count {
                // Tied for highest connections - include in candidates for random tie-break
                best_candidates.push(w);
            }
        }
        if best_candidates.is_empty() {
            break;  // No outsiders (should not happen unless k equals graph.n())
        }
        // Choose one of the top candidates at random to introduce some randomness
        best_candidates.shuffle(rng);
        let w = best_candidates[0];
        sol.add(w);
    }
    return sol;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use rand_chacha::ChaCha8Rng;
    use rand::SeedableRng;

    fn triangle() -> Graph {
        // Simple 3-vertex triangle graph
        let dimacs = b"p edge 3 3\n\
                       e 1 2\n\
                       e 1 3\n\
                       e 2 3\n";
        Graph::parse_dimacs(Cursor::new(dimacs)).unwrap()
    }

    #[test]
    fn random_vs_greedy() {
        let g = triangle();
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let r = random_k(&g, 2, &mut rng);
        assert_eq!(r.size(), 2);
        let g2 = greedy_k(&g, 2);
        assert_eq!(g2.size(), 2);
    }

    // (No direct test for greedy_random_k here, but its behavior can be inferred from its construction.)
}
