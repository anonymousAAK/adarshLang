"""AdarshLang compiler package with legacy-friendly exports."""

from . import ast as _ast
from . import lexer as _lexer
from . import parser as _parser
from . import pipeline as _pipeline
from . import runtime as _runtime
from . import semantics as _semantics
from . import tokens as _tokens

_export_order = (
    _tokens,
    _ast,
    _lexer,
    _parser,
    _semantics,
    _runtime,
    _pipeline,
)


def _export_module(module):
    names = getattr(module, '__all__', None)
    if names is None:
        names = [name for name in vars(module) if not name.startswith('_')]
    for name in names:
        globals()[name] = getattr(module, name)
    return list(names)


_aggregated_all = []
for _module in _export_order:
    _aggregated_all.extend(_export_module(_module))

__all__ = sorted(set(_aggregated_all))

# Housekeeping: avoid leaking helper names
del _aggregated_all, _export_module, _module, _export_order
del _ast, _lexer, _parser, _pipeline, _runtime, _semantics, _tokens
