"""
Run the Rust TSQC solver on one DIMACS instance.

Modes
-----
1. Any-k search        (default)
2. Fixed k             (--k N)
"""

from __future__ import annotations
from pathlib import Path
from time import perf_counter
from typing import Any, Dict, Optional

# PyO3 exports from the compiled extension -----------------
from tsqc import solve_k_py, solve_max_py
from tsqc.data_loader import load_dimacs


def run_instance(
    path: Path,
    gamma: float,
    runs: int = 1,
    k_target: Optional[int] = None,
    seed_base: int = 42,
) -> Dict[str, Any]:
    """
    Execute TSQC *runs* times on a DIMACS file and keep the best result.

    Returns a JSON-ready dict with:
        {instance, gamma, n_vertices, n_edges,
         best_size, density, time_s}
    """
    n_vertices, edge_list = load_dimacs(path)

    best_size: int = -1
    best_density: float = 0.0
    best_time: float = float("inf")

    for i in range(runs):
        seed = seed_base + i
        start = perf_counter()

        if k_target is None:
            # ---------- any-k search ----------
            size, density = solve_max_py(edge_list, n_vertices, gamma, seed)
        else:
            # ---------- fixed-k search ----------
            density = solve_k_py(edge_list, n_vertices, k_target, gamma, seed)
            size = k_target

        elapsed = perf_counter() - start

        if density > best_density or (density == best_density and elapsed < best_time):
            best_density = density
            best_size = size
            best_time = elapsed

    return {
        "instance":   path.name,
        "gamma":      gamma,
        "n_vertices": n_vertices,
        "n_edges":    len(edge_list),
        "best_size":  best_size,
        "density":    best_density,
        "time_s":     best_time,
    }


# ─────────────────────────── CLI ────────────────────────────
if __name__ == "__main__":  # pragma: no cover
    import argparse, json

    parser = argparse.ArgumentParser("Run TSQC on one DIMACS instance")
    parser.add_argument("-i", "--instance", type=Path, required=True)
    parser.add_argument("-g", "--gamma",   type=float, required=True,
                        help="Target density threshold γ (0<γ≤1)")
    parser.add_argument("-r", "--runs",    type=int,   default=1,
                        help="Independent runs for the same parameters")
    parser.add_argument("-k", "--k",       type=int,
                        help="Fix subset size instead of any-k search")
    parser.add_argument("-s", "--seed",    type=int,   default=42,
                        help="Base RNG seed (incremented per run)")
    parser.add_argument("-o", "--out",     type=Path,
                        help="Write JSON result to file")
    args = parser.parse_args()

    res = run_instance(args.instance, args.gamma, args.runs, args.k, args.seed)

    if args.out:
        args.out.parent.mkdir(parents=True, exist_ok=True)
        args.out.write_text(json.dumps(res, indent=2))
        print(f"✓ saved → {args.out}")
    else:
        print(json.dumps(res, indent=2))
