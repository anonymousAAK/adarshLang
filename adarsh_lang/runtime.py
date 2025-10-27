"""Runtime interpreter for AdarshLang."""

import math
import os
import random
import time

from .ast import (
    AdarshAttributeAccessNode,
    AdarshBlockNode,
    AdarshCallNode,
    AdarshDhachaConstructNode,
    AdarshDictLiteralNode,
    AdarshDikhaoNode,
    AdarshForEachNode,
    AdarshForNode,
    AdarshFunctionExprNode,
    AdarshIfNode,
    AdarshImportNode,
    AdarshIndexAccessNode,
    AdarshIndexAssignNode,
    AdarshListLiteralNode,
    AdarshNumLiteralNode,
    AdarshStringLiteralNode,
    AdarshSwitchNode,
    AdarshThrowNode,
    AdarshTryCatchNode,
    AdarshVarReferenceNode,
    AdarshWhileNode,
    AdarshWapasNode,
)
from .lexer import AdarshLexer
from .parser import AdarshParser
from .semantics import (
    AdarshFunctionSignature,
    AdarshSemanticAnalyzer,
    AdarshSemanticError,
    AdarshSymbolTable,
)


class AdarshRuntimeError(Exception):
    pass


class AdarshUserException(Exception):
    def __init__(self, value):
        super().__init__('User exception')
        self.value = value


class AdarshReturnSignal:
    def __init__(self, value):
        self.value = value


class AdarshBreakSignal:
    pass


class AdarshContinueSignal:
    pass


class AdarshCallable:
    def __init__(self, signature):
        self.signature = signature

    def call(self, interpreter, args):
        raise NotImplementedError


class AdarshFunctionValue(AdarshCallable):
    def __init__(self, name, params, body, closure_scope):
        super().__init__(AdarshFunctionSignature(params))
        self.name = name
        self.params = params
        self.body = body
        self.closure_scope = closure_scope

    def call(self, interpreter, args):
        if not self.signature.allows(len(args)):
            raise AdarshRuntimeError(
                f"Function '{self.name or '<anonymous>'}' expected between {self.signature.min_args} and "
                f"{self.signature.max_args if self.signature.max_args is not None else 'infinite'} arguments in AdarshLang."
            )
        call_scope = AdarshSymbolTable(parent=self.closure_scope)
        values, _ = interpreter._resolve_arguments(self.params, args, self.closure_scope)
        for param, value in zip(self.params, values):
            call_scope.declare_variable(param.name)
            call_scope.set_variable(param.name, value)
        result = interpreter.visit(self.body, call_scope)
        if isinstance(result, AdarshReturnSignal):
            return result.value
        if isinstance(result, AdarshBreakSignal):
            raise AdarshRuntimeError("'bas' cannot exit a function directly in AdarshLang.")
        if isinstance(result, AdarshContinueSignal):
            raise AdarshRuntimeError("'aage_badho' cannot escape to the caller in AdarshLang.")
        return result


class AdarshBuiltinFunction(AdarshCallable):
    def __init__(self, name, signature, impl):
        super().__init__(signature)
        self.name = name
        self.impl = impl

    def call(self, interpreter, args):
        if not self.signature.allows(len(args)):
            raise AdarshRuntimeError(
                f"Builtin '{self.name}' expected between {self.signature.min_args} and "
                f"{self.signature.max_args if self.signature.max_args is not None else 'infinite'} arguments in AdarshLang."
            )
        values, provided = interpreter._resolve_arguments(
            self.signature.params, args, interpreter.global_scope
        )
        return self.impl(interpreter, values, provided)


class AdarshInterpreter:
    def __init__(self):
        self.global_scope = AdarshSymbolTable()
        self.loaded_modules = set()
        self._register_builtins()

    def _register_builtins(self):
        analyzer = AdarshSemanticAnalyzer()
        for name, signature in analyzer.builtin_functions.items():
            builtin = AdarshBuiltinFunction(name, signature, getattr(self, f'_builtin_{name}'))
            self.global_scope.declare_variable(name)
            self.global_scope.set_variable(name, builtin)

    def interpret(self, program):
        return self.visit(program, self.global_scope)

    def visit(self, node, scope):
        method_name = 'visit_' + node.__class__.__name__
        method = getattr(self, method_name, None)
        if method is None:
            raise AdarshRuntimeError(f"No visit_{node.__class__.__name__} method defined in AdarshInterpreter.")
        return method(node, scope)

    def visit_AdarshProgramNode(self, node, scope):
        result = None
        for stmt in node.statements:
            result = self.visit(stmt, scope)
            if isinstance(result, (AdarshReturnSignal, AdarshBreakSignal, AdarshContinueSignal)):
                return result
        return result

    def visit_BadloNode(self, node, scope):
        scope.declare_variable(node.var_name)
        value = None
        if node.init_expr is not None:
            value = self.visit(node.init_expr, scope)
            scope.set_variable(node.var_name, value)
        return value

    def visit_AdarshAssignNode(self, node, scope):
        value = self.visit(node.expr, scope)
        scope.set_variable(node.var_name, value)
        return value

    def visit_AdarshIndexAssignNode(self, node, scope):
        collection = self.visit(node.collection, scope)
        index_value = self.visit(node.index_expr, scope)
        value = self.visit(node.value_expr, scope)
        try:
            if isinstance(collection, dict):
                collection[index_value] = value
            else:
                index_value = self._normalize_index(index_value)
                collection[index_value] = value
        except TypeError:
            raise AdarshRuntimeError(
                "Only mutable collections like lists or dicts support indexed assignment in AdarshLang."
            )
        except IndexError:
            raise AdarshRuntimeError("Index out of range while assigning into collection in AdarshLang.")
        return value

    def visit_AdarshAttributeAssignNode(self, node, scope):
        target = self.visit(node.target, scope)
        value = self.visit(node.value_expr, scope)
        if isinstance(target, dict):
            target[node.attribute] = value
        else:
            raise AdarshRuntimeError("Attribute assignment only supported on dhacha objects or dictionaries in AdarshLang.")
        return value

    def visit_AdarshBinOpNode(self, node, scope):
        left = self.visit(node.left, scope)
        right = self.visit(node.right, scope)
        op = node.op
        if op == '+':
            return left + right
        elif op == '-':
            return left - right
        elif op == '*':
            return left * right
        elif op == '/':
            return left / right
        elif op == '==':
            return left == right
        elif op == '!=':
            return left != right
        elif op == '<':
            return left < right
        elif op == '>':
            return left > right
        elif op == '<=':
            return left <= right
        elif op == '>=':
            return left >= right
        elif op == '&&':
            return bool(left) and bool(right)
        elif op == '||':
            return bool(left) or bool(right)
        else:
            raise AdarshRuntimeError(f"Unknown binary operator '{op}' in AdarshLang.")

    def visit_AdarshUnaryOpNode(self, node, scope):
        factor = self.visit(node.factor, scope)
        if node.op == '-':
            return -factor
        elif node.op == '!':
            return not factor
        else:
            raise AdarshRuntimeError(f"Unknown unary operator '{node.op}' in AdarshLang.")

    def visit_AdarshNumLiteralNode(self, node, scope):
        return node.value

    def visit_AdarshBoolLiteralNode(self, node, scope):
        return node.value

    def visit_AdarshStringLiteralNode(self, node, scope):
        return node.value

    def visit_AdarshListLiteralNode(self, node, scope):
        return [self.visit(element, scope) for element in node.elements]

    def visit_AdarshDictLiteralNode(self, node, scope):
        result = {}
        for key_expr, value_expr in node.entries:
            key = self.visit(key_expr, scope)
            value = self.visit(value_expr, scope)
            result[key] = value
        return result

    def visit_AdarshDhachaConstructNode(self, node, scope):
        if not scope.has_type(node.type_name):
            raise AdarshRuntimeError(
                f"Dhacha '{node.type_name}' not declared before use in AdarshLang."
            )
        fields = {name: None for name in scope.get_type_fields(node.type_name)}
        for field_name, expr in node.field_exprs:
            fields[field_name] = self.visit(expr, scope)
        fields['__type__'] = node.type_name
        return fields

    def visit_AdarshVarReferenceNode(self, node, scope):
        return scope.get_variable(node.var_name)

    def visit_AdarshAttributeAccessNode(self, node, scope):
        target = self.visit(node.target, scope)
        if isinstance(target, dict):
            if node.attribute in target:
                return target[node.attribute]
            raise AdarshRuntimeError(
                f"Attribute '{node.attribute}' missing on object in AdarshLang."
            )
        raise AdarshRuntimeError(
            "Attribute access only supported on dhacha objects or dictionaries in AdarshLang."
        )

    def visit_AdarshIndexAccessNode(self, node, scope):
        collection = self.visit(node.collection, scope)
        index_value = self.visit(node.index_expr, scope)
        try:
            if isinstance(collection, dict):
                return collection[index_value]
            index_value = self._normalize_index(index_value)
            return collection[index_value]
        except KeyError:
            raise AdarshRuntimeError("Key not present during dictionary access in AdarshLang.")
        except TypeError:
            raise AdarshRuntimeError(
                "Only indexable values like lists, strings, or dicts support indexing in AdarshLang."
            )
        except IndexError:
            raise AdarshRuntimeError("Index out of range while accessing collection in AdarshLang.")

    def visit_AdarshDikhaoNode(self, node, scope):
        val = self.visit(node.expr, scope)
        print(val)
        return None

    def visit_AdarshThrowNode(self, node, scope):
        value = self.visit(node.expr, scope)
        raise AdarshUserException(value)

    def visit_AdarshImportNode(self, node, scope):
        module_value = self.visit(node.module_expr, scope)
        if not isinstance(module_value, str):
            raise AdarshRuntimeError("'lao' expects a string path in AdarshLang.")
        module_path = module_value if module_value.endswith('.aak') else module_value + '.aak'
        if module_path in self.loaded_modules:
            return None
        if not os.path.exists(module_path):
            raise AdarshRuntimeError(f"Module '{module_path}' not found for 'lao' in AdarshLang.")
        self.loaded_modules.add(module_path)
        with open(module_path, 'r', encoding='utf-8') as handle:
            source = handle.read()
        lexer = AdarshLexer(source)
        tokens = lexer.tokenize()
        parser = AdarshParser(tokens)
        ast = parser.parse_program()
        sema = AdarshSemanticAnalyzer()
        try:
            sema.analyze(ast)
        except AdarshSemanticError as exc:
            raise AdarshRuntimeError(str(exc))
        self.visit(ast, self.global_scope)
        return None

    def visit_AdarshIfNode(self, node, scope):
        cond_val = self.visit(node.condition, scope)
        if cond_val:
            return self.visit(node.if_block, AdarshSymbolTable(parent=scope))
        if node.else_block:
            return self.visit(node.else_block, AdarshSymbolTable(parent=scope))
        return None

    def visit_AdarshWhileNode(self, node, scope):
        result = None
        while True:
            cond_val = self.visit(node.condition, scope)
            if not cond_val:
                break
            body_scope = AdarshSymbolTable(parent=scope)
            result = self.visit(node.block, body_scope)
            if isinstance(result, AdarshReturnSignal):
                return result
            if isinstance(result, AdarshBreakSignal):
                return None
            if isinstance(result, AdarshContinueSignal):
                continue
        return result

    def visit_AdarshForNode(self, node, scope):
        loop_scope = AdarshSymbolTable(parent=scope)
        if node.init_stmt is not None:
            init_result = self.visit(node.init_stmt, loop_scope)
            if isinstance(init_result, AdarshReturnSignal):
                return init_result
        result = None
        while True:
            if node.condition is not None:
                condition_value = self.visit(node.condition, loop_scope)
                if not condition_value:
                    break
            body_result = self.visit(node.block, AdarshSymbolTable(parent=loop_scope))
            if isinstance(body_result, AdarshReturnSignal):
                return body_result
            if isinstance(body_result, AdarshBreakSignal):
                return None
            if isinstance(body_result, AdarshContinueSignal):
                pass
            if node.update_expr is not None:
                self.visit(node.update_expr, loop_scope)
        return result

    def visit_AdarshForEachNode(self, node, scope):
        iterable = self.visit(node.iterable_expr, scope)
        if not hasattr(iterable, '__iter__'):
            raise AdarshRuntimeError("'ke_liye' expects an iterable expression in AdarshLang.")
        loop_scope = AdarshSymbolTable(parent=scope)
        iter_var, declare_new = node.iter_var
        if declare_new:
            loop_scope.declare_variable(iter_var)
        result = None
        for item in iterable:
            if declare_new:
                loop_scope.set_variable(iter_var, item)
            else:
                scope.set_variable(iter_var, item)
            iteration_result = self.visit(node.block, AdarshSymbolTable(parent=loop_scope))
            if isinstance(iteration_result, AdarshReturnSignal):
                return iteration_result
            if isinstance(iteration_result, AdarshBreakSignal):
                return None
            if isinstance(iteration_result, AdarshContinueSignal):
                continue
            result = iteration_result
        return result

    def visit_AdarshSwitchNode(self, node, scope):
        subject_value = self.visit(node.subject, scope)
        for case_clause in node.cases:
            if any(self.visit(match_expr, scope) == subject_value for match_expr in case_clause.match_exprs):
                case_scope = AdarshSymbolTable(parent=scope)
                return self.visit(case_clause.block, case_scope)
        if node.default_block:
            return self.visit(node.default_block, AdarshSymbolTable(parent=scope))
        return None

    def visit_AdarshTryCatchNode(self, node, scope):
        try:
            try_scope = AdarshSymbolTable(parent=scope)
            result = self.visit(node.try_block, try_scope)
            if isinstance(result, (AdarshReturnSignal, AdarshBreakSignal, AdarshContinueSignal)):
                return result
            return result
        except AdarshUserException as exc:
            catch_scope = AdarshSymbolTable(parent=scope)
            catch_scope.declare_variable(node.err_name)
            catch_scope.set_variable(node.err_name, exc.value)
            return self.visit(node.catch_block, catch_scope)

    def visit_AdarshBlockNode(self, node, scope):
        result = None
        for statement in node.statements:
            result = self.visit(statement, scope)
            if isinstance(result, (AdarshReturnSignal, AdarshBreakSignal, AdarshContinueSignal)):
                return result
        return result

    def visit_AdarshKaamDefNode(self, node, scope):
        func_value = AdarshFunctionValue(node.func_name, node.params, node.body, scope)
        scope.declare_variable(node.func_name)
        scope.set_variable(node.func_name, func_value)
        return func_value

    def visit_AdarshDhachaDefNode(self, node, scope):
        scope.declare_type(node.name, node.fields)
        return None

    def visit_AdarshWapasNode(self, node, scope):
        value = None
        if node.expr is not None:
            value = self.visit(node.expr, scope)
        return AdarshReturnSignal(value)

    def visit_AdarshFunctionExprNode(self, node, scope):
        return AdarshFunctionValue(None, node.params, node.body, scope)

    def visit_AdarshCallNode(self, node, scope):
        callee_value = self.visit(node.callee, scope)
        if not isinstance(callee_value, AdarshCallable):
            raise AdarshRuntimeError("Attempted to call a non-callable value in AdarshLang.")
        arg_values = [self.visit(arg, scope) for arg in node.args]
        return callee_value.call(self, arg_values)

    def visit_AdarshBreakNode(self, node, scope):
        return AdarshBreakSignal()

    def visit_AdarshContinueNode(self, node, scope):
        return AdarshContinueSignal()

    def _resolve_arguments(self, params, args, scope):
        values = []
        provided_flags = []
        arg_index = 0
        for param in params:
            if param.is_varargs:
                values.append(args[arg_index:])
                provided_flags.append(bool(args[arg_index:]))
                arg_index = len(args)
                break
            if arg_index < len(args):
                values.append(args[arg_index])
                provided_flags.append(True)
                arg_index += 1
            elif param.default_expr is not None:
                default_value = self.visit(param.default_expr, scope)
                values.append(default_value)
                provided_flags.append(False)
            else:
                raise AdarshRuntimeError(
                    "Missing required argument in AdarshLang function call."
                )
        if arg_index < len(args) and not params[-1].is_varargs:
            raise AdarshRuntimeError(
                "Too many arguments provided in AdarshLang function call."
            )
        return values, provided_flags

    def _register_builtin(self, name, signature, impl):
        builtin = AdarshBuiltinFunction(name, signature, impl)
        self.global_scope.declare_variable(name)
        self.global_scope.set_variable(name, builtin)

    def _builtin_length(self, interpreter, values, provided):
        return len(values[0])

    def _builtin_push(self, interpreter, values, provided):
        target, value = values
        if not isinstance(target, list):
            raise AdarshRuntimeError("'push' expects a list target in AdarshLang.")
        target.append(value)
        return target

    def _builtin_pop(self, interpreter, values, provided):
        target = values[0]
        if not isinstance(target, list):
            raise AdarshRuntimeError("'pop' expects a list target in AdarshLang.")
        index = -1
        if len(values) > 1 and provided[1]:
            index = self._normalize_index(values[1])
        try:
            return target.pop(index)
        except IndexError:
            raise AdarshRuntimeError("Index out of range for 'pop' in AdarshLang.")

    def _builtin_rakho(self, interpreter, values, provided):
        target, key, value = values
        if not isinstance(target, dict):
            raise AdarshRuntimeError("'rakho' expects a dictionary target in AdarshLang.")
        target[key] = value
        return target

    def _builtin_nikalo(self, interpreter, values, provided):
        target, key_or_index = values[0], values[1]
        default_value = values[2] if len(values) > 2 else None
        has_default = len(values) > 2 and provided[2]
        if isinstance(target, dict):
            if key_or_index in target:
                return target[key_or_index]
            if has_default:
                return default_value
            raise AdarshRuntimeError("Key not present in 'nikalo' dictionary lookup in AdarshLang.")
        if isinstance(target, list):
            try:
                index = self._normalize_index(key_or_index)
                return target[index]
            except IndexError:
                if has_default:
                    return default_value
                raise AdarshRuntimeError("Index out of range for 'nikalo' in AdarshLang.")
        if isinstance(target, str):
            try:
                index = self._normalize_index(key_or_index)
                return target[index]
            except IndexError:
                if has_default:
                    return default_value
                raise AdarshRuntimeError("Index out of range for 'nikalo' in AdarshLang.")
        raise AdarshRuntimeError("'nikalo' expects a dictionary, list, or string target in AdarshLang.")

    def _call_callable(self, fn_value, args):
        if not isinstance(fn_value, AdarshCallable):
            raise AdarshRuntimeError("Expected a callable function value in AdarshLang.")
        return fn_value.call(self, args)

    def _builtin_map(self, interpreter, values, provided):
        fn_value, iterable = values
        if not hasattr(iterable, '__iter__'):
            raise AdarshRuntimeError("'map' expects an iterable second argument in AdarshLang.")
        return [self._call_callable(fn_value, [item]) for item in iterable]

    def _builtin_filter(self, interpreter, values, provided):
        fn_value, iterable = values
        if not hasattr(iterable, '__iter__'):
            raise AdarshRuntimeError("'filter' expects an iterable second argument in AdarshLang.")
        return [item for item in iterable if self._call_callable(fn_value, [item])]

    def _builtin_reduce(self, interpreter, values, provided):
        fn_value, iterable = values[0], values[1]
        if not hasattr(iterable, '__iter__'):
            raise AdarshRuntimeError("'reduce' expects an iterable second argument in AdarshLang.")
        iterator = iter(iterable)
        if len(provided) > 2 and provided[2]:
            accumulator = values[2]
        else:
            try:
                accumulator = next(iterator)
            except StopIteration:
                raise AdarshRuntimeError(
                    "'reduce' requires at least one item when no initial value is provided in AdarshLang."
                )
        for item in iterator:
            accumulator = self._call_callable(fn_value, [accumulator, item])
        return accumulator

    def _builtin_random_number(self, interpreter, values, provided):
        start, end = values
        if start > end:
            start, end = end, start
        return random.uniform(start, end)

    def _builtin_current_time(self, interpreter, values, provided):
        return time.time()

    def _builtin_abs(self, interpreter, values, provided):
        return abs(values[0])

    def _builtin_floor(self, interpreter, values, provided):
        return math.floor(values[0])

    def _builtin_ceil(self, interpreter, values, provided):
        return math.ceil(values[0])

    def _builtin_upper(self, interpreter, values, provided):
        return str(values[0]).upper()

    def _builtin_lower(self, interpreter, values, provided):
        return str(values[0]).lower()

    def _builtin_join(self, interpreter, values, provided):
        iterable, sep = values
        try:
            return str(sep).join(str(item) for item in iterable)
        except TypeError:
            raise AdarshRuntimeError("'join' expects an iterable of values in AdarshLang.")

    def _builtin_split(self, interpreter, values, provided):
        value, sep = values
        return str(value).split(str(sep))

    def _normalize_index(self, index_value):
        if isinstance(index_value, bool):
            raise AdarshRuntimeError("Index must be an integer value in AdarshLang.")
        if isinstance(index_value, float):
            if not index_value.is_integer():
                raise AdarshRuntimeError(
                    "Index expressions must evaluate to whole numbers in AdarshLang."
                )
            index_value = int(index_value)
        if not isinstance(index_value, int):
            raise AdarshRuntimeError("Index expressions must evaluate to integers in AdarshLang.")
        return index_value


__all__ = [
    'AdarshInterpreter',
    'AdarshRuntimeError',
    'AdarshUserException',
    'AdarshReturnSignal',
    'AdarshBreakSignal',
    'AdarshContinueSignal',
]
