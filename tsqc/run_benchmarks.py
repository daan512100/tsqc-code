#!/usr/bin/env python3
"""
run_benchmarks.py – call tsqc.runner over all DIMACS instances from Djeddi et al. (2019).

Usage:
  python run_benchmarks.py
  python run_benchmarks.py --bench-root data\dimacs --repeat 5
  python run_benchmarks.py --extra-args "--seed 123"
"""

import argparse
import os
import subprocess
import sys
from pathlib import Path
from typing import List, Tuple

# ──────────────────────────────────────────────────────────────────────────────
# 1) List of all DIMACS benchmarks (filename, gamma, target-k) from Tables 1–3
# ──────────────────────────────────────────────────────────────────────────────
BENCHMARKS: List[Tuple[str, float, int]] = [
    # γ = 0.85
    ("p_hat300-1.clq",  0.85,  12),
    ("p_hat300-2.clq",  0.85,  85),
    ("brock200_2.clq",  0.85,  19),
    ("hamming8-4.clq",  0.85,  35),
    ("keller4.clq",     0.85,  31),
    ("brock200_4.clq",  0.85,  39),
    ("p_hat300-3.clq",  0.85, 180),
    ("brock400_2.clq",  0.85, 100),
    ("brock400_4.clq",  0.85, 102),
    ("p_hat700-1.clq",  0.85,  19),
    ("p_hat700-2.clq",  0.85, 223),
    ("p_hat700-3.clq",  0.85, 430),
    ("p_hat1500-2.clq", 0.85, 487),
    ("p_hat1500-3.clq", 0.85, 943),
    ("keller5.clq",     0.85, 286),

    # γ = 0.95
    ("p_hat300-1.clq",  0.95,   9),
    ("p_hat300-2.clq",  0.95,  41),
    ("brock200_2.clq",  0.95,  13),
    ("hamming8-4.clq",  0.95,  17),
    ("keller4.clq",     0.95,  15),
    ("brock200_4.clq",  0.95,  21),
    ("p_hat300-3.clq",  0.95,  71),
    ("brock400_2.clq",  0.95,  40),
    ("brock400_4.clq",  0.95,  39),
    ("p_hat700-1.clq",  0.95,  13),
    ("p_hat700-2.clq",  0.95,  96),
    ("p_hat700-3.clq",  0.95, 176),
    ("p_hat1500-2.clq", 0.95, 193),
    ("p_hat1500-3.clq", 0.95, 351),
    ("keller5.clq",     0.95,  47),

    # γ = 1.00
    ("p_hat300-1.clq",  1.00,   8),
    ("p_hat300-2.clq",  1.00,  25),
    ("brock200_2.clq",  1.00,  12),
    ("hamming8-4.clq",  1.00,  16),
    ("keller4.clq",     1.00,  11),
    ("brock200_4.clq",  1.00,  17),
    ("p_hat300-3.clq",  1.00,  36),
    ("brock400_2.clq",  1.00,  29),
    ("brock400_4.clq",  1.00,  33),
    ("p_hat700-1.clq",  1.00,  11),
    ("p_hat700-2.clq",  1.00,  44),
    ("p_hat700-3.clq",  1.00,  62),
    ("p_hat1500-2.clq", 1.00,  65),
    ("p_hat1500-3.clq", 1.00,  94),
    ("keller5.clq",     1.00,  27),
]

# ──────────────────────────────────────────────────────────────────────────────
# 2) CLI options
# ──────────────────────────────────────────────────────────────────────────────
def parse_args():
    p = argparse.ArgumentParser(
        description="Run tsqc.runner over all DIMACS benchmarks"
    )
    p.add_argument(
        "--bench-root", "-b",
        default="benchmarks",
        help="directory where your .clq/.dimacs files live"
    )
    p.add_argument(
        "--repeat", "-r",
        type=int, default=10,
        help="how many runs per instance (passed as `-r` to tsqc.runner)"
    )
    p.add_argument(
        "--extra-args", "-e",
        default="",
        help="any extra flags for tsqc.runner (e.g. \"--seed 42\")"
    )
    return p.parse_args()

# ──────────────────────────────────────────────────────────────────────────────
# 3) Drive runner.py in a subprocess
# ──────────────────────────────────────────────────────────────────────────────
def main():
    args = parse_args()
    bench_dir = Path(args.bench_root)
    extra = args.extra_args.strip().split() if args.extra_args else []

    for fname, gamma, k in BENCHMARKS:
        inst = bench_dir / fname
        if not inst.exists():
            print(f"⚠  {inst} not found; skipping", file=sys.stderr)
            continue

        # separator + summary header
        print("\n" + "="*60)
        print(f"▶ {fname}   γ={gamma}   k={k}   ×{args.repeat}")

        # build command
        cmd = [
            sys.executable, "-m", "tsqc.runner",
            "-i", str(inst),
            "-g", str(gamma),
            "-k", str(k),
            "-r", str(args.repeat),
        ] + extra

        # force UTF-8 so "γ" never crashes on Windows
        env = os.environ.copy()
        env["PYTHONIOENCODING"] = "utf-8"

        # shell out and let runner.py print directly
        subprocess.run(cmd, check=True, env=env)

    print("\n✅ All done.")

if __name__ == "__main__":
    main()
