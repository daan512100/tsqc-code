"""
Batch-run TSQC on a folder of DIMACS graphs.

Example:
    python -m tsqc.benchmarks -d benchmarks -g 0.7,0.8,0.9 -r 3
"""

from __future__ import annotations
from pathlib import Path
from typing import List
import argparse, pandas as pd, itertools, sys

from tsqc.runner import run_instance


def parse_gamma_list(s: str) -> List[float]:
    return [float(x) for x in s.split(",") if x.strip()]


def benchmark_dimacs(
    dimacs_dir: Path,
    pattern: str,
    gammas: List[float],
    runs: int,
    k_target: int | None,
) -> pd.DataFrame:
    """
    Run TSQC over every file matching pattern × every γ.
    Keeps the best run per (file, γ).
    """
    rows = []
    files = sorted(dimacs_dir.glob(pattern))
    total = len(files) * len(gammas)
    print(f"Benchmarking {total} combos …\n")

    for file_path, gamma in itertools.product(files, gammas):
        if file_path.name.startswith("._"):  # macOS junk
            continue
        print(f"→ {file_path.name}  γ={gamma}")
        res = run_instance(file_path, gamma, runs=runs, k_target=k_target)
        rows.append(res)

    return pd.DataFrame(rows)


# ─────────────────────────── CLI ────────────────────────────
def main(argv: list[str] | None = None) -> None:
    parser = argparse.ArgumentParser("TSQC DIMACS benchmark")
    parser.add_argument("-d", "--dir",      type=Path, required=True)
    parser.add_argument("-p", "--pattern",  default="*.clq")
    parser.add_argument("-g", "--gammas",   required=True,
                        help="Comma-sep list, e.g. 0.7,0.8")
    parser.add_argument("-r", "--runs",     type=int, default=1,
                        help="Independent runs each combo")
    parser.add_argument("-k", "--k",        type=int,
                        help="Fixed size k instead of any-k search")
    parser.add_argument("-o", "--out",      type=Path,
                        default=Path("benchmark_results.csv"))
    args = parser.parse_args(argv)

    df = benchmark_dimacs(
        args.dir, args.pattern,
        parse_gamma_list(args.gammas),
        args.runs, args.k,
    )

    args.out.parent.mkdir(parents=True, exist_ok=True)
    df.to_csv(args.out, index=False)
    print(f"\n✓ Results saved -> {args.out}")


if __name__ == "__main__":  # pragma: no cover
    main(sys.argv[1:])
