//! Constructors for an initial vertex subset S₀.
//!
//! * `random_k` – choose k distinct vertices uniformly at random.
//! * `greedy_k` – take the k highest-degree vertices.
//!
//! Both return a ready-to-use [`Solution`] with edge counts pre-computed.

use crate::{graph::Graph, solution::Solution};
use rand::seq::SliceRandom;
use rand::Rng;

/// Pick `k` distinct vertices uniformly-at-random.
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

/// Greedy initialisation: take the `k` highest-degree vertices.
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

/*──────────── tests ────────────*/
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use rand_chacha::ChaCha8Rng;
    use rand::SeedableRng;

    fn triangle() -> Graph {
        let dimacs = b"p edge 3 3\ne 1 2\ne 1 3\ne 2 3\n";
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
}
