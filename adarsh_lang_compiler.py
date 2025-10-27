"""Compatibility facade for historical ``adarsh_lang_compiler`` imports."""

from adarsh_lang import *  # noqa: F401,F403
from adarsh_lang import __all__ as _package_all


if __name__ == '__main__':
    main()


__all__ = list(_package_all)
del _package_all
