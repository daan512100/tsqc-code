#!/usr/bin/env python3
"""
runner.py – core driver for one or more fixed-k TSQC runs.

Usage:
    tsqc.runner -i INST.clq -k K -g GAMMA -r REPEAT -s SEED
"""

import argparse
import sys
import time

from pathlib import Path

from tsqc._native import solve_k_py, parse_dimacs_py  # your pyo3 bindings

def main():
    p = argparse.ArgumentParser(
        description="Run TSQC fixed-k solver (prints one table)"
    )
    p.add_argument("-i", "--input",   required=True, help="DIMACS .clq file")
    p.add_argument("-k", "--k",       type=int, required=True, help="target size k")
    p.add_argument("-g", "--gamma",   type=float, required=True, help="density threshold γ")
    p.add_argument("-r", "--repeat",  type=int, default=1, help="how many independent runs")
    p.add_argument("-s", "--seed",    type=int, default=0, help="base RNG seed")
    args = p.parse_args()

    inst_path = Path(args.input)
    if not inst_path.exists():
        print(f"ERROR: file not found: {inst_path}", file=sys.stderr)
        sys.exit(1)

    # Pre–parse n, m
    try:
        n, m = parse_dimacs_py(str(inst_path))
    except Exception as e:
        print(f"ERROR parsing DIMACS header: {e}", file=sys.stderr)
        sys.exit(1)

    mode = "fixed"
    header = (
        f"{inst_path.name}: n={n} m={m} "
        f"mode={mode} k={args.k} γ={args.gamma}"
    )
    print(header)

    # single table header
    print(" run  seed   size  edges  density     sec")

    for run_idx in range(1, args.repeat + 1):
        seed = args.seed + run_idx - 1
        start = time.perf_counter()
        try:
            rho = solve_k_py(str(inst_path), args.k, args.gamma, seed)
            # solve_k_py returns density only; for fixed-k runs size==k
            size = args.k
            # we need edges = density * (k*(k-1)/2) rounded
            edges = int(round(rho * (size * (size - 1) / 2)))
            elapsed = time.perf_counter() - start

            # print one row
            print(f"{run_idx:4d}  {seed:4d}  {size:4d}  {edges:5d}   "
                  f"{rho:7.4f}   {elapsed:6.2f}")

        except Exception as e:
            elapsed = time.perf_counter() - start
            print(f"{run_idx:4d}  {seed:4d}  FAILED      —     —   {elapsed:6.2f}",
                  file=sys.stderr)
            sys.exit(1)

if __name__ == "__main__":
    main()
