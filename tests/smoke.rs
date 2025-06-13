use tsqc::{Graph, Params, solve_fixed_k};
use rand_chacha::ChaCha8Rng;
use rand::SeedableRng;

#[test]
fn smoke_fixed_k() {
    // 5-vertex complete graph minus one edge (2-3 missing)
    let edges = vec![
        (0,1),(0,2),(0,3),(0,4),
        (1,2),(1,3),(1,4),
        (2,4),
        (3,4),
    ];
    let g = Graph::from_edge_list(5, &edges);

    let mut rng = ChaCha8Rng::seed_from_u64(1);
    let sol = solve_fixed_k(&g, 5, &mut rng, &Params::default());

    // Solver should reach at least the original 0.9 density,
    // and may reach 1.0 after improving the edge set.
    assert!(sol.density() >= 0.9);
}
