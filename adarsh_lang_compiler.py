"""Compatibility facade that mirrors the historic ``adarsh_lang_compiler`` module.

The original project exposed nearly every compiler building block from a single
file.  When the codebase was modularised into the :mod:`adarsh_lang` package we
want existing imports (``from adarsh_lang_compiler import AdarshLexer`` etc.) to
keep working.  Rather than duplicate the implementation we re-export the public
symbols from the new modules and stitch their ``__all__`` metadata together so
star-imports continue to behave like the monolith.
"""

from adarsh_lang.ast import *  # noqa: F401,F403
from adarsh_lang.ast import __all__ as _ast_all
from adarsh_lang.lexer import *  # noqa: F401,F403
from adarsh_lang.lexer import __all__ as _lexer_all
from adarsh_lang.parser import *  # noqa: F401,F403
from adarsh_lang.parser import __all__ as _parser_all
from adarsh_lang.pipeline import *  # noqa: F401,F403
from adarsh_lang.pipeline import __all__ as _pipeline_all
from adarsh_lang.runtime import *  # noqa: F401,F403
from adarsh_lang.runtime import __all__ as _runtime_all
from adarsh_lang.semantics import *  # noqa: F401,F403
from adarsh_lang.semantics import __all__ as _semantics_all
from adarsh_lang.tokens import *  # noqa: F401,F403
from adarsh_lang.tokens import __all__ as _tokens_all


def _dedupe(sequence):
    seen = set()
    result = []
    for item in sequence:
        if item not in seen:
            seen.add(item)
            result.append(item)
    return result


__all__ = _dedupe(
    _tokens_all
    + _lexer_all
    + _ast_all
    + _parser_all
    + _semantics_all
    + _runtime_all
    + _pipeline_all
)


if __name__ == '__main__':
    main()
