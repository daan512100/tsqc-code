"""
Fast DIMACS loader that delegates parsing to Rust when the compiled
`tsqc` wheel is available.  Falls back to pure-Python parsing if the Rust
symbol isn't present (e.g. during first `cargo test` before building the
wheel).

Usage:
    from tsqc.data_loader import load_dimacs
    n, edges = load_dimacs(Path("benchmarks/hamming8-4.clq"))
"""

from pathlib import Path
from typing import List, Tuple

try:
    # when the wheel is installed, this is the ultra-fast Rust parser
    from tsqc import parse_dimacs_py   # PyO3 export
except ImportError:
    parse_dimacs_py = None


# ---------------------------------------------------------------------------
def _python_dimacs(path: Path) -> Tuple[int, List[Tuple[int, int]]]:
    """Portable fallback DIMACS parser (line-by-line, ASCII)."""
    n_vertices = 0
    edges: List[Tuple[int, int]] = []

    with path.open("r", encoding="utf8") as fh:
        for line in fh:
            line = line.strip()
            if not line or line.startswith("c"):
                continue
            if line.startswith("p"):
                # line: 'p edge <n> <m>'
                parts = line.split()
                if len(parts) >= 3:
                    n_vertices = int(parts[2])
            elif line.startswith("e"):
                # line: 'e <u> <v>'  (1-based indices)
                parts = line.split()
                if len(parts) >= 3:
                    u = int(parts[1]) - 1
                    v = int(parts[2]) - 1
                    edges.append((u, v))

    return n_vertices, edges


# ---------------------------------------------------------------------------
def load_dimacs(path: Path) -> Tuple[int, List[Tuple[int, int]]]:
    """
    Returns (n_vertices, edge_list with 0-based indices).
    Tries the Rust backend first for speed, otherwise uses Python fallback.
    """
    if parse_dimacs_py is not None:
        return parse_dimacs_py(str(path))
    return _python_dimacs(path)
