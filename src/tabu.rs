// src/tabu.rs
//! Dual-tabu lists with adaptive tenures (see §3.4.3).
//!
//! We keep two short-term tabu memories:
//! - `tabu_u` forbids re-adding recently removed vertices for Tu iters.
//! - `tabu_v` forbids removing   recently added   vertices for Tv iters.
//!
//! After each move (successful or not), tenures Tu/Tv are recomputed based
//! on the current deficit from the γ-target (capped at 10) plus a random
//! component, preventing cycling and encouraging diversification.

use rand::Rng;

#[derive(Clone, Debug)]
pub struct DualTabu {
    expiry_u: Vec<usize>, // when each vertex may next be re-added
    expiry_v: Vec<usize>, // when each vertex may next be removed
    iter:     usize,      // global iteration counter
    tu:       usize,      // current tabu tenure for re-addition
    tv:       usize,      // current tabu tenure for removal
}

impl DualTabu {
    /// Create a fresh DualTabu for `n` vertices with minimum tenures.
    pub fn new(n: usize, initial_tu: usize, initial_tv: usize) -> Self {
        Self {
            expiry_u: vec![0; n],
            expiry_v: vec![0; n],
            iter:     0,
            tu:       initial_tu.max(1),
            tv:       initial_tv.max(1),
        }
    }

    /// Recompute *Tu* and *Tv* based on current size `size_s`, edge count `edges`,
    /// target density `gamma`, and randomness from `rng`.
    ///
    /// 1. `clique_edges = size_s*(size_s-1)/2`  
    /// 2. `target_edges = ceil(γ * clique_edges)`  
    /// 3. `deficit = max(target_edges - edges, 0)`  
    /// 4. `l = min(deficit, 10)`  
    /// 5. `C = max(size_s/40, 6)`  
    /// 6. `Tu = (l+1) + rand(0..C)`  
    /// 7. `Tv = floor(0.6*(l+1)) + rand(0..floor(0.6*C))`  
    pub fn update_tenures<R: Rng + ?Sized>(
        &mut self,
        size_s: usize,
        edges:   usize,
        gamma:   f64,
        rng:     &mut R,
    ) {
        // 1) Maximum edges in a full clique of size_s:
        let clique_edges = if size_s < 2 {
            0
        } else {
            size_s * (size_s - 1) / 2
        };

        // 2) Required edges to meet γ (rounded up):
        let target_edges = (gamma * (clique_edges as f64)).ceil() as usize;

        // 3) How many edges short, capped at 10:
        let deficit = target_edges.saturating_sub(edges);
        let l = deficit.min(10);

        // 4) Base C = max(size_s/40, 6):
        let c = (size_s / 40).max(6);

        // 5) Random components:
        let rand_u = if c > 0 {
            rng.gen_range(0..c)
        } else {
            0
        };
        let c6    = ((0.6 * (c as f64)).floor() as usize).max(1);
        let rand_v = rng.gen_range(0..c6);

        // 6) Update tenures (ensure ≥1):
        self.tu = (l + 1 + rand_u).max(1);
        let base_v = ((l + 1) as f64 * 0.6).floor() as usize;
        self.tv = (base_v + rand_v).max(1);
    }

    /// Advance the global iteration counter by one.  
    /// Call at the end of every move (whether it changes the solution or not).
    #[inline]
    pub fn step(&mut self) {
        self.iter += 1;
    }

    /// Is vertex `v` currently tabu from re-addition? (was just removed)
    #[inline]
    pub fn is_tabu_u(&self, v: usize) -> bool {
        self.expiry_u[v] > self.iter
    }

    /// Is vertex `v` currently tabu from removal? (was just added)
    #[inline]
    pub fn is_tabu_v(&self, v: usize) -> bool {
        self.expiry_v[v] > self.iter
    }

    /// Forbid re-adding `v` for the next `tu` iterations.
    #[inline]
    pub fn forbid_u(&mut self, v: usize) {
        self.expiry_u[v] = self.iter + self.tu;
    }

    /// Forbid removing `v` for the next `tv` iterations.
    #[inline]
    pub fn forbid_v(&mut self, v: usize) {
        self.expiry_v[v] = self.iter + self.tv;
    }

    /// Clear all tabu marks (used after a heavy/mild perturbation).
    pub fn reset(&mut self) {
        self.expiry_u.fill(0);
        self.expiry_v.fill(0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_tabu_logic() {
        let mut t = DualTabu::new(3, 2, 3);
        // Initially, nothing is tabu
        assert!(!t.is_tabu_u(1));
        assert!(!t.is_tabu_v(2));

        // Forbid and check at iter=0
        t.forbid_u(1);
        t.forbid_v(2);
        assert!(t.is_tabu_u(1));
        assert!(t.is_tabu_v(2));

        // Advance iter; u still tabu until iter ≥2, v until iter ≥3
        t.step(); // iter=1
        assert!(t.is_tabu_u(1));
        assert!(t.is_tabu_v(2));

        t.step(); // iter=2
        assert!(!t.is_tabu_u(1));
        assert!(t.is_tabu_v(2));

        t.step(); // iter=3
        assert!(!t.is_tabu_u(1));
        assert!(!t.is_tabu_v(2));
    }
}
