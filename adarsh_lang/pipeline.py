"""Compilation pipeline helpers for AdarshLang."""

import sys

from .lexer import AdarshLexer
from .parser import AdarshParser, AdarshParserError
from .runtime import (
    AdarshBreakSignal,
    AdarshContinueSignal,
    AdarshInterpreter,
    AdarshReturnSignal,
    AdarshRuntimeError,
    AdarshUserException,
)
from .semantics import AdarshSemanticAnalyzer, AdarshSemanticError, AdarshSymbolTable


def adarshlang_compile_and_run(source_code):
    lexer = AdarshLexer(source_code)
    tokens = lexer.tokenize()

    parser = AdarshParser(tokens)
    ast = parser.parse_program()

    sema = AdarshSemanticAnalyzer()
    sema.analyze(ast)

    interpreter = AdarshInterpreter()
    result = interpreter.interpret(ast)
    return result


def execute_with_state(source_code, interpreter, analyzer, semantic_scope):
    lexer = AdarshLexer(source_code)
    tokens = lexer.tokenize()
    parser = AdarshParser(tokens)
    ast = parser.parse_program()
    analyzer.analyze(ast, scope=semantic_scope)
    return interpreter.visit(ast, interpreter.global_scope)


def run_repl():
    print("AdarshLang REPL shuru! Blank line run kare, 'exit' se bahar niklo.")
    interpreter = AdarshInterpreter()
    analyzer = AdarshSemanticAnalyzer()
    semantic_scope = AdarshSymbolTable()
    for name, signature in analyzer.builtin_functions.items():
        semantic_scope.declare_function(name, signature)
        semantic_scope.declare_variable(name)

    buffer = []
    while True:
        try:
            prompt = '... ' if buffer else 'adarsh> '
            line = input(prompt)
        except EOFError:
            print()
            break
        if line.strip() == 'exit':
            break
        buffer.append(line)
        if line.strip() == '':
            source = '\n'.join(buffer).strip()
            buffer = []
            if not source:
                continue
            try:
                result = execute_with_state(source, interpreter, analyzer, semantic_scope)
                if isinstance(result, AdarshReturnSignal):
                    print(result.value)
                elif result is not None and not isinstance(result, (AdarshBreakSignal, AdarshContinueSignal)):
                    print(result)
            except AdarshSemanticError as exc:
                print(f"Soch: {exc}")
            except AdarshUserException as exc:
                print(f"Pakda: {exc.value}")
            except AdarshRuntimeError as exc:
                print(f"Galti: {exc}")
            except AdarshParserError as exc:
                print(f"Samjho: {exc}")


def main(argv=None):
    argv = argv if argv is not None else sys.argv
    if len(argv) < 2 or argv[1] == '--repl':
        run_repl()
        return

    filename = argv[1]
    with open(filename, 'r', encoding='utf-8') as handle:
        source_code = handle.read()

    adarshlang_compile_and_run(source_code)


__all__ = [
    'adarshlang_compile_and_run',
    'execute_with_state',
    'run_repl',
    'main',
]
