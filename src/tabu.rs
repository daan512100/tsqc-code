//! Dual-tabu list with adaptive tenures (Section 3.4.3 of the TSQC paper).
//!
//! We maintain two tabu lists: 
//! **tabu_u** for vertices recently added to S (forbidding their removal for Tv iterations), 
//! and **tabu_v** for vertices recently removed from S (forbidding their re-addition for Tu iterations):contentReference[oaicite:83]{index=83}:contentReference[oaicite:84]{index=84}. 
//! The tabu tenures *Tu* and *Tv* are updated dynamically based on the current solution's status (density gap and random variation):contentReference[oaicite:85]{index=85}:contentReference[oaicite:86]{index=86}. 
//! This adaptation helps prevent the search from becoming stuck by allowing tenures to grow when far from a feasible solution and shrink when close:contentReference[oaicite:87]{index=87}.

use rand::Rng;

#[derive(Clone, Debug)]
pub struct DualTabu {
    expiry_u: Vec<usize>,  // iteration index until which vertex is forbidden to be removed (for each v in V)
    expiry_v: Vec<usize>,  // iteration index until which vertex is forbidden to be added
    iter:     usize,       // current global iteration count for tabu timing
    tu:       usize,       // current tenure for removed vertices (tabu_v duration)
    tv:       usize,       // current tenure for added vertices (tabu_u duration)
}

impl DualTabu {
    /*────────── constructor ──────────*/

    pub fn new(n: usize, initial_tu: usize, initial_tv: usize) -> Self {
        Self {
            expiry_u: vec![0; n],
            expiry_v: vec![0; n],
            iter:     0,
            tu:       initial_tu.max(1),
            tv:       initial_tv.max(1),
        }
    }

    /*────────── adaptive tenure update ──────────*/

    /// Recompute tabu tenures Tu and Tv based on the current solution size and edges.
    ///
    /// This implements the adaptive formula inspired by Wu & Hao (2013):contentReference[oaicite:88]{index=88}. 
    /// Let *L<sub>q</sub>* be the minimum number of edges required for a size-|S| quasi-clique (γ-target edges). 
    /// We compute `l = min{ L_q - m(S), 10 }` as the capped edge deficit. 
    /// Then `Tu = (l + 1) + Random(0..C)` and `Tv = 0.6*(l + 1) + Random(0..0.6*C)`, where `C = max{|S|/40, 6}`. 
    /// This means if the current solution is far from the density target (large deficit *l*), tenures increase (up to a cap), 
    /// and if it's close to feasible, tenures stay smaller. A random component prevents cycles where all vertices become tabu:contentReference[oaicite:89]{index=89}.
    pub fn update_tenures<R: Rng + ?Sized>(
        &mut self, 
        size_s: usize, 
        edges: usize, 
        gamma: f64, 
        rng: &mut R
    ) {
        // Compute the required number of edges for a quasi-clique of size_s (ceil of γ * (size_s choose 2))
        let clique_edges = if size_s < 2 {
            0 
        } else {
            (size_s * (size_s - 1)) / 2
        };
        let target_edges = (gamma * (clique_edges as f64)).ceil() as usize;
        // l = how many edges short of target (capped at 10)
        let deficit = if target_edges > edges {
            target_edges - edges
        } else {
            0
        };
        let l = deficit.min(10) as usize;
        // C = max{|S|/40, 6} as an integer
        let c = ((size_s / 40).max(6)) as usize;
        // Randomize tenures based on l and C
        let rand_u = rng.gen_range(0..=c);
        let rand_v = rng.gen_range(0..=((0.6 * (c as f64)).floor() as usize));
        // Tu = l + 1 + random(0..C)
        self.tu = (l + 1 + rand_u).max(1);
        // Tv = 0.6*(l + 1) + random(0..0.6*C)
        let base_v = ((l + 1) as f64 * 0.6).floor() as usize;
        self.tv = (base_v + rand_v).max(1);
    }

    /*────────── iteration control ──────────*/

    #[inline] 
    pub fn step(&mut self) { 
        // Advance the global iteration counter for tabu. This should be called at the end of each iteration (move or not).
        self.iter += 1; 
    }

    /*────────── tabu status queries ──────────*/

    #[inline] 
    pub fn is_tabu_u(&self, v: usize) -> bool {
        // Checks if vertex v is currently forbidden to be added to S (recently removed)
        self.expiry_u[v] > self.iter
    }
    #[inline] 
    pub fn is_tabu_v(&self, v: usize) -> bool {
        // Checks if vertex v is currently forbidden to be removed from S (recently added)
        self.expiry_v[v] > self.iter
    }

    /*────────── mark moves as tabu ──────────*/

    #[inline] 
    pub fn forbid_u(&mut self, v: usize) {
        // Forbid vertex v from being added back to S for the next Tu iterations
        self.expiry_u[v] = self.iter + self.tu;
    }
    #[inline] 
    pub fn forbid_v(&mut self, v: usize) {
        // Forbid vertex v from being removed from S for the next Tv iterations
        self.expiry_v[v] = self.iter + self.tv;
    }

    /*────────── reset after diversification ──────────*/

    pub fn reset(&mut self) {
        // Clear all tabu entries (long-term memory like frequencies remains untouched).
        self.expiry_u.fill(0);
        self.expiry_v.fill(0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_tabu_logic() {
        let mut t = DualTabu::new(3, 2, 3); // initial Tu=2, Tv=3
        // iteration 0
        assert!(!t.is_tabu_u(1));
        t.forbid_u(1);
        t.forbid_v(2);
        // iteration 0: forbids take effect immediately
        assert!( t.is_tabu_u(1));
        assert!( t.is_tabu_v(2));
        t.step(); // iter 1
        assert!( t.is_tabu_u(1)); // still tabu (expiry iter ~2)
        assert!( t.is_tabu_v(2)); // still tabu (expiry iter ~3)
        t.step(); // iter 2
        assert!(!t.is_tabu_u(1)); // U-tabu expired
        assert!( t.is_tabu_v(2));  // V-tabu still active
        t.step(); // iter 3
        assert!(!t.is_tabu_v(2)); // V-tabu expired
    }
}
