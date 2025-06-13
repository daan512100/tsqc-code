"""
Python façade for the native Rust extension.

Import order:
1.   import tsqc         → this file
2.   this file imports tsqc._native (compiled pyd/so)
3.   re-exports the three PyO3 functions at top level
"""

from importlib import import_module, metadata as _md

# load the shared library (tsqc/_native.*)
_native = import_module("tsqc._native")

# re-export selected symbols so callers can do:  from tsqc import solve_k_py
solve_k_py      = _native.solve_k_py
solve_max_py    = _native.solve_max_py
parse_dimacs_py = _native.parse_dimacs_py

__all__ = [
    "solve_k_py",
    "solve_max_py",
    "parse_dimacs_py",
]

__version__ = _md.version("tsqc")

# clean up internal names
del _native, import_module, _md
