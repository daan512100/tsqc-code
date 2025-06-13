//! Dual-tabu list with *adaptive* tenures (Section 5.3 of the TSQC PDF).
//!
//! Tenures are recalculated every time the solution size |S| changes:
//!     Tu = ⌈0.6 · |S|⌉ and Tv = ⌈0.4 · |S|⌉.

#[derive(Clone, Debug)]
pub struct DualTabu {
    expiry_u: Vec<usize>,
    expiry_v: Vec<usize>,
    iter:     usize,
    tu:       usize,
    tv:       usize,
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

    /*────────── adaptive tenure ──────────*/

    /// Recompute Tu and Tv for the current |S|.
    pub fn update_tenures(&mut self, size_s: usize) {
        self.tu = ((size_s as f64) * 0.6).ceil().max(1.0) as usize;
        self.tv = ((size_s as f64) * 0.4).ceil().max(1.0) as usize;
    }

    /*────────── iteration control ──────────*/

    #[inline] pub fn step(&mut self) { self.iter += 1; }

    /*────────── tabu queries ──────────*/

    #[inline] pub fn is_tabu_u(&self, v: usize) -> bool {
        self.expiry_u[v] > self.iter
    }
    #[inline] pub fn is_tabu_v(&self, v: usize) -> bool {
        self.expiry_v[v] > self.iter
    }

    /*────────── tabu setters ──────────*/

    #[inline] pub fn forbid_u(&mut self, v: usize) {
        self.expiry_u[v] = self.iter + self.tu;
    }
    #[inline] pub fn forbid_v(&mut self, v: usize) {
        self.expiry_v[v] = self.iter + self.tv;
    }

    /*────────── reset after diversification ──────────*/

    pub fn reset(&mut self) {
        self.expiry_u.fill(0);
        self.expiry_v.fill(0);
    }
}


/*──────────── unit tests ────────────*/
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_tabu_logic() {
        let mut t = DualTabu::new(3, 2, 3); // Tu=2, Tv=3

        // iteration 0
        assert!(!t.is_tabu_u(1));
        t.forbid_u(1);
        t.forbid_v(2);

        // iteration 0: forbids take effect immediately
        assert!( t.is_tabu_u(1));
        assert!( t.is_tabu_v(2));

        t.step(); // iter 1
        assert!( t.is_tabu_u(1)); // still tabu (expiry 2)
        assert!( t.is_tabu_v(2)); // still tabu (expiry 3)

        t.step(); // iter 2
        assert!(!t.is_tabu_u(1)); // U-tabu expired
        assert!( t.is_tabu_v(2)); // V-tabu still active

        t.step(); // iter 3
        assert!(!t.is_tabu_v(2)); // V-tabu expired
    }
}
