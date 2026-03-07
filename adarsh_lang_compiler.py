"""Backward-compatible re-export of the AdarshLang compiler package."""

from adarsh_lang import *  # noqa: F401,F403
from adarsh_lang import __all__ as _package_all

__all__ = list(_package_all)
