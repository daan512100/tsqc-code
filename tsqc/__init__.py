"""Thin Python façade for the native `tsqc` extension."""
from importlib import metadata as _md, import_module as _im

_im("tsqc.tsqc")           # loads the compiled *.pyd/.so into this package

__version__ = _md.version("tsqc")
del _md, _im
