from pathlib import Path
from tsqc.data_loader import load_dimacs

n, edges = load_dimacs(Path("benchmarks/hamming8-4.clq"))
print(f"n={n}, |E|={len(edges)}")
