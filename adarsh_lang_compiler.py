"""Compatibility module that re-exports the AdarshLang compiler package."""

from adarsh_lang import (
    AdarshParserError,
    AdarshRuntimeError,
    AdarshSemanticAnalyzer,
    AdarshSemanticError,
    AdarshSymbolTable,
    AdarshUserException,
    adarshlang_compile_and_run,
    execute_with_state,
    main,
    run_repl,
)

__all__ = [
    'AdarshParserError',
    'AdarshRuntimeError',
    'AdarshSemanticAnalyzer',
    'AdarshSemanticError',
    'AdarshSymbolTable',
    'AdarshUserException',
    'adarshlang_compile_and_run',
    'execute_with_state',
    'run_repl',
    'main',
]


if __name__ == '__main__':
    main()
