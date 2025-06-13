//! Candidate solution: a vertex subset S with cached |S| and m(S).
//!
//! • O(1) access to size and edge count.  
//! • O(n / 64) per add/remove operation.  
//! • Works together with [`Graph`] and [`DualTabu`].

use bitvec::prelude::*;
use crate::graph::Graph;

/// Mutable quasi-clique candidate bound to a single [`Graph`].
#[derive(Clone, Debug)]
pub struct Solution<'g> {
    graph:      &'g Graph,
    vertices:   BitVec,
    edge_count: usize,
    size:       usize,
}

/*───────────────────────── impl ─────────────────────────*/

impl<'g> Solution<'g> {
    /* constructors */

    /// Empty solution.
    pub fn new(graph: &'g Graph) -> Self {
        Self {
            graph,
            vertices: bitvec![0; graph.n()],
            edge_count: 0,
            size: 0,
        }
    }

    /// Build from an initial bitset; computes edge count.
    pub fn from_bitset(graph: &'g Graph, subset: &BitSlice) -> Self {
        assert_eq!(subset.len(), graph.n());

        let size = subset.count_ones();
        let mut e = 0usize;
        for i in 0..graph.n() {
            if subset[i] {
                for j in graph.neigh_row(i).iter_ones().filter(|&j| j > i) {
                    if subset[j] { e += 1; }
                }
            }
        }

        let mut vertices = BitVec::repeat(false, graph.n());
        vertices |= subset;

        Self { graph, vertices, edge_count: e, size }
    }

    /* queries */

    #[inline] pub fn size(&self) -> usize          { self.size }
    #[inline] pub fn edges(&self) -> usize         { self.edge_count }
    #[inline] pub fn bitset(&self) -> &BitVec      { &self.vertices }
    #[inline] pub fn graph(&self) -> &Graph        { self.graph }

    /// Density 2 m(S) / (|S|·(|S|−1)); returns 0 for |S| < 2.
    pub fn density(&self) -> f64 {
        if self.size < 2 { 0.0 }
        else { 2.0 * self.edge_count as f64 / (self.size * (self.size - 1)) as f64 }
    }

    pub fn is_gamma_feasible(&self, gamma: f64) -> bool {
       self.density() + f64::EPSILON >= gamma
    }
    /* mutators */

    /// Add vertex *v* (no-op if already present).
    pub fn add(&mut self, v: usize) {
        if self.vertices[v] { return; }
        let added = self.graph.neigh_row(v)
            .iter_ones()
            .filter(|&j| self.vertices[j])
            .count();
        self.vertices.set(v, true);
        self.size       += 1;
        self.edge_count += added;
    }

    /// Remove vertex *v* (no-op if absent).
    pub fn remove(&mut self, v: usize) {
        if !self.vertices[v] { return; }
        let removed = self.graph.neigh_row(v)
            .iter_ones()
            .filter(|&j| self.vertices[j])
            .count();
        self.vertices.set(v, false);
        self.size       -= 1;
        self.edge_count -= removed;
    }

    /// Toggle membership; returns `true` if *v* is in the set afterwards.
    pub fn toggle(&mut self, v: usize) -> bool {
        if self.vertices[v] { self.remove(v); false } else { self.add(v); true }
    }

    /// Clear S completely.
    pub fn clear(&mut self) {
        self.vertices.fill(false);
        self.size = 0;
        self.edge_count = 0;
    }
}

/*───────────────────────── tests ─────────────────────────*/

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn triangle_graph() -> Graph {
        let dimacs = b"p edge 3 3\ne 1 2\ne 1 3\ne 2 3\n";
        Graph::parse_dimacs(Cursor::new(dimacs)).unwrap()
    }

    #[test]
    fn add_remove_consistency() {
        let g = triangle_graph();
        let mut sol = Solution::new(&g);

        sol.add(0);
        sol.add(1);
        sol.add(2);
        approx::assert_relative_eq!(sol.density(), 1.0);

        sol.remove(1);
        assert_eq!(sol.size(), 2);
        assert_eq!(sol.edges(), 1);
    }
}
