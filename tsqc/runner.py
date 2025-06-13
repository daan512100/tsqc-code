"""
Streaming TSQC runner – toont live voortgang.

Elke run:
    1) print een placeholder "… searching …"
    2) voert Rust-solver uit
    3) overschrijft de regel met definitieve gegevens
"""

from __future__ import annotations
from pathlib import Path
from time import perf_counter
import argparse, statistics, sys
from typing import Tuple

from tsqc import solve_max_py, solve_k_py, parse_dimacs_py


def edges_from_density(k: int, rho: float) -> int:
    return int(round(rho * (k * (k - 1) / 2))) if k > 1 else 0


def single_run(path: Path, gamma: float, k: int | None, seed: int) -> Tuple[int, int, float, float]:
    t0 = perf_counter()
    if k is None:
        size, rho = solve_max_py(str(path), gamma, seed)
    else:
        rho  = solve_k_py(str(path), k, gamma, seed)
        size = k
    sec   = perf_counter() - t0
    edges = edges_from_density(size, rho)
    return size, edges, rho, sec


def better(a, b):
    """a is better than b?  (size desc, edges desc, sec asc)."""
    return (a[0], a[1], -a[3]) > (b[0], b[1], -b[3])


def main(argv: list[str] | None = None) -> None:
    ap = argparse.ArgumentParser("TSQC runner (live)")
    ap.add_argument("-i", "--instance", type=Path, required=True)
    ap.add_argument("-g", "--gamma",    type=float, default=0.90)
    ap.add_argument("-r", "--runs",     type=int,   default=1)
    ap.add_argument("-k", "--k",        type=int)
    ap.add_argument("-s", "--seed",     type=int,   default=42)
    args = ap.parse_args(argv)

    n, m = parse_dimacs_py(str(args.instance))
    mode = f"fixed k={args.k}" if args.k else "max-k"
    header = f"{args.instance.name}: n={n} m={m} mode={mode} γ={args.gamma}"
    print(header)
    print("run  size  edges  density     sec")

    best = None
    all_rho, all_sec = [], []

    for r in range(1, args.runs + 1):
        # ── placeholder ──────────────────────────────────────────────
        placeholder = f"{r:>3}   …   …    ……….     …"
        print(placeholder, end="\r", flush=True)

        # ── run solver ───────────────────────────────────────────────
        size, edges, rho, sec = single_run(args.instance, args.gamma, args.k, args.seed + r - 1)
        all_rho.append(rho)
        all_sec.append(sec)

        # ── overwrite line with real data ───────────────────────────
        line = f"{r:>3} {size:>5} {edges:>6}  {rho:7.4f}  {sec:7.2f}"
        print(line + " " * max(0, len(placeholder) - len(line)))  # clean remainder

        cur = (size, edges, rho, sec, r)
        if best is None or better(cur, best):
            best = cur

    # ── summary ─────────────────────────────────────────────────────
    print(f"\nbest: run {best[4]}  size {best[0]}  edges {best[1]}  density {best[2]:.4f}")
    if args.runs > 1:
        avg_rho = statistics.mean(all_rho)
        std_rho = statistics.stdev(all_rho) if args.runs > 2 else 0
        print("avg density", f"{avg_rho:.4f} ± {std_rho:.4f}",
              "   avg sec", f"{statistics.mean(all_sec):.2f}")


if __name__ == "__main__":      # pragma: no cover
    main(sys.argv[1:])
