#!/usr/bin/env python3
"""
run_benchmarks.py – call tsqc.runner over all DIMACS instances
with per‐run timeout, reseeding across runs, seed logging, and a
‘Best:’ summary per dataset.

Usage:
  python run_benchmarks.py [-b BENCH_ROOT] [-r REPEAT]
                           [-a ATTEMPTS] [-s SEED] [-t TIMEOUT]
                           [-e EXTRA_ARGS]
"""
import argparse
import os
import re
import subprocess
import sys
from pathlib import Path
from typing import List, Tuple, Optional

# ────────────────────────────────────────────────────────────────────────────────
# 1) DIMACS benchmarks (filename, γ, target‐k)
# ────────────────────────────────────────────────────────────────────────────────
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

# ────────────────────────────────────────────────────────────────────────────────
# 2) CLI options
# ────────────────────────────────────────────────────────────────────────────────
def parse_args():
    p = argparse.ArgumentParser(
        description="Batch-runner for tsqc.runner (timeout, reseeding, best‐summary)"
    )
    p.add_argument(
        "-b", "--bench-root",
        default="benchmarks",
        help="directory where .clq/.dimacs files live"
    )
    p.add_argument(
        "-r", "--repeat",
        type=int, default=3,
        help="how many successful runs per instance"
    )
    p.add_argument(
        "-a", "--attempts",
        type=int, default=3,
        help="how many seeds to try per run before skipping"
    )
    p.add_argument(
        "-s", "--seed",
        type=int, default=42,
        help="base seed; for run R, attempt A: seed = base + (R-1)*A + (A-1)"
    )
    p.add_argument(
        "-t", "--timeout",
        type=float, default=30.0,
        help="seconds to wait before giving up on one seed‐attempt"
    )
    p.add_argument(
        "-e", "--extra-args",
        default="",
        help="extra flags to forward to tsqc.runner"
    )
    return p.parse_args()

# ────────────────────────────────────────────────────────────────────────────────
# 3) Parser voor runner-uitvoer:
#     lijn: " 1   <seed>  <size>  <edges>  <density>  <sec>"
# ────────────────────────────────────────────────────────────────────────────────
import re
_RUN_LINE = re.compile(r"^\s*1\s+(\d+)\s+(\d+)\s+(\d+)\s+([\d.]+)\s+([\d.]+)\s*$")

def parse_run_line(output: str) -> Tuple[int,int,float,float]:
    """Return (size, edges, density, sec)."""
    for L in output.splitlines():
        m = _RUN_LINE.match(L)
        if m:
            return (
                int(m.group(2)),
                int(m.group(3)),
                float(m.group(4)),
                float(m.group(5))
            )
    raise ValueError("Geen geldige run‐output gevonden")

# ────────────────────────────────────────────────────────────────────────────────
# 4) Main loop: per dataset, print tabel, en daarna "Best: …"
# ────────────────────────────────────────────────────────────────────────────────
def main():
    args = parse_args()
    bench_dir = Path(args.bench_root)
    extras    = args.extra_args.split() if args.extra_args else []

    for fname, gamma, k in BENCHMARKS:
        inst = bench_dir / fname
        if not inst.exists():
            print(f"⚠ {inst} niet gevonden; overslaan", file=sys.stderr)
            continue

        # Header dataset
        print("\n" + "="*60)
        print(f"▶ {fname}   γ={gamma}   k={k}   ×{args.repeat}")
        print("  run   seed   size   edges   density    sec")

        records = []  # sla alle succesvolle pogingen op

        run_num = 0
        while run_num < args.repeat:
            run_num += 1
            base_seed = args.seed + (run_num - 1) * args.attempts
            success: Optional[Tuple[str,int,int,float,float]] = None

            # probeer maximaal args.attempts seeds
            for att in range(1, args.attempts+1):
                seed = base_seed + (att - 1)
                cmd = [
                    sys.executable, "-m", "tsqc.runner",
                    "-i", str(inst),
                    "-g", str(gamma),
                    "-k", str(k),
                    "-r", "1",
                    "-s", str(seed),
                ] + extras

                label = f"{run_num}" if att == 1 else f"{run_num}.{att}"
                try:
                    res = subprocess.run(
                        cmd,
                        check=True,
                        stdout=subprocess.PIPE,
                        stderr=subprocess.DEVNULL,
                        text=True,
                        timeout=args.timeout,
                        env={**os.environ, "PYTHONIOENCODING":"utf-8"},
                    )
                    size, edges, density, sec = parse_run_line(res.stdout)
                    # Print regel en bewaar in records
                    print(f"  {label:<5}{seed:<7}{size:<7}{edges:<8}"
                          f"{density:<10.4f}{sec:<.2f}")
                    records.append((label, seed, size, edges, density, sec))
                    success = (label,seed,size,edges,density,sec)
                    break

                except subprocess.TimeoutExpired:
                    print(f"  {label:<5}TIMEOUT")
                except subprocess.CalledProcessError:
                    print(f"  {label:<5}ERROR")
                    break
                except Exception:
                    print(f"  {label:<5}PARSE_ERR")
                    break

            if not success:
                # volledig mislukte run: lege rij
                print(f"  {run_num:<5}{'-':<7}{'-':<7}{'-':<8}{'-':<10}{'-':<5}")

        # na alle runs: bepaal de beste
        if records:
            # sorteer op hoogste density, dan hoogste edges, dan laagste sec
            best = max(
                records,
                key=lambda r: (r[4], r[3], -r[5])
            )
            label, seed, size, edges, density, sec = best
            print(f"\n  Best: run={label} seed={seed} size={size} "
                  f"edges={edges} density={density:.4f} sec={sec:.2f}")
        else:
            print("\n  Best: — (geen succesvolle runs)")

    print("\n✅ All done.")

if __name__ == "__main__":
    main()
