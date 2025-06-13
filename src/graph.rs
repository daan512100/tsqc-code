//! Simple undirected graph stored as an adjacency BitVec per row.
//! Supports DIMACS *.clq parsing and edge iteration.

use bitvec::prelude::*;
use std::io::{BufRead, Read};

#[derive(Clone, Debug)]
pub struct Graph {
    /// Row‐major adjacency; `adj[i][j]` is 1 ⇔ edge (i,j) exists, j≠i.
    adj: Vec<BitVec>,
}

impl Graph {
    /*────────── constructors ──────────*/

    /// Empty graph with `n` isolated vertices.
    pub fn with_vertices(n: usize) -> Self {
        let mut rows = Vec::with_capacity(n);
        for _ in 0..n {
            rows.push(bitvec![0; n]);
        }
        Self { adj: rows }
    }

    /// Build from explicit edge list (0-based indices, undirected).
    pub fn from_edge_list(n: usize, edges: &[(usize, usize)]) -> Self {
        let mut g = Self::with_vertices(n);
        for &(u, v) in edges {
            g.add_edge(u, v);
        }
        g
    }

    /// Parse DIMACS *.clq format from any buffered reader.
    pub fn parse_dimacs<R: Read>(reader: R) -> std::io::Result<Self> {
        let mut n = 0usize;
        let mut edges: Vec<(usize, usize)> = Vec::new();

        for line in std::io::BufReader::new(reader).lines() {
            let line = line?;
            let line = line.trim();
            if line.is_empty() || line.starts_with('c') { continue; }
            if line.starts_with('p') {
                // p edge <n> <m>
                let parts: Vec<_> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    n = parts[2].parse().unwrap_or(0);
                }
            } else if line.starts_with('e') {
                // e u v   (1-based)
                let parts: Vec<_> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    let u: usize = parts[1].parse().unwrap();
                    let v: usize = parts[2].parse().unwrap();
                    edges.push((u - 1, v - 1));
                }
            }
        }
        Ok(Self::from_edge_list(n, &edges))
    }

    /*────────── getters ──────────*/

    #[inline] pub fn n(&self) -> usize { self.adj.len() }

    /// Number of edges (each counted once).
    pub fn m(&self) -> usize {
        let mut m = 0usize;
        for i in 0..self.n() {
            for j in self.neigh_row(i).iter_ones().filter(|&j| j > i) {
                if self.adj[i][j] { m += 1; }
            }
        }
        m
    }

    /// Degree of vertex v.
    #[inline]
    pub fn degree(&self, v: usize) -> usize {
        self.adj[v].count_ones()
    }

    /// Immutable row slice for adjacency of v.
    #[inline]
    pub fn neigh_row(&self, v: usize) -> &BitSlice {
        &self.adj[v]
    }

    /// Return all edges as Vec<(u,v)> with u < v.
    pub fn edge_list(&self) -> Vec<(usize, usize)> {
        let mut edges = Vec::with_capacity(self.m());
        for i in 0..self.n() {
            for j in self.neigh_row(i).iter_ones().filter(|&j| j > i) {
                edges.push((i, j));
            }
        }
        edges
    }

    /*────────── mutators ──────────*/

    #[inline]
    pub fn add_edge(&mut self, u: usize, v: usize) {
        assert!(u < self.n() && v < self.n() && u != v);
        self.adj[u].set(v, true);
        self.adj[v].set(u, true);
    }
}

/*────────────────── tiny unit check ──────────────────*/
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tiny_triangle() {
        let g = Graph::from_edge_list(3, &[(0, 1), (0, 2), (1, 2)]);
        assert_eq!(g.n(), 3);
        assert_eq!(g.m(), 3);
        assert_eq!(g.edge_list().len(), 3);
    }
}
