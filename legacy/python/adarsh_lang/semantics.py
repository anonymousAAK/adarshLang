"""Semantic analysis for AdarshLang."""

import os

from .ast import (
    AdarshAssignNode,
    AdarshAttributeAccessNode,
    AdarshAttributeAssignNode,
    AdarshAttributeCompoundAssignNode,
    AdarshBinOpNode,
    AdarshBlockNode,
    AdarshBoolLiteralNode,
    AdarshBreakNode,
    AdarshCallNode,
    AdarshCompoundAssignNode,
    AdarshContinueNode,
    AdarshDhachaConstructNode,
    AdarshDhachaDefNode,
    AdarshDictLiteralNode,
    AdarshDikhaoNode,
    AdarshForEachNode,
    AdarshForNode,
    AdarshFunctionExprNode,
    AdarshIfNode,
    AdarshImportNode,
    AdarshIndexAccessNode,
    AdarshIndexAssignNode,
    AdarshIndexCompoundAssignNode,
    AdarshKaamDefNode,
    AdarshListLiteralNode,
    AdarshNullLiteralNode,
    AdarshNumLiteralNode,
    AdarshParam,
    AdarshProgramNode,
    AdarshStringLiteralNode,
    AdarshSwitchNode,
    AdarshThrowNode,
    AdarshTryCatchNode,
    AdarshUnaryOpNode,
    AdarshVarReferenceNode,
    AdarshWhileNode,
    AdarshWapasNode,
    BadloNode,
)
from .lexer import AdarshLexer
from .parser import AdarshParser


class AdarshSemanticError(Exception):
    pass


class AdarshFunctionSignature:
    def __init__(self, params):
        self.params = params
        self.has_varargs = any(p.is_varargs for p in params)
        self.min_args = 0
        self.max_args = 0
        defaults_started = False
        for param in params:
            if param.is_varargs:
                self.max_args = None
                break
            self.max_args += 1
            if param.default_expr is None and not defaults_started:
                self.min_args += 1
            else:
                defaults_started = True

    def allows(self, arg_count):
        if arg_count < self.min_args:
            return False
        if self.max_args is None:
            return True
        return arg_count <= self.max_args


class AdarshSymbolTable:
    def __init__(self, parent=None):
        self.parent = parent
        self.variables = {}
        self.functions = {}
        self.types = {}

    def declare_variable(self, name):
        if name not in self.variables:
            self.variables[name] = None

    def set_variable(self, name, value):
        if name in self.variables:
            self.variables[name] = value
        elif self.parent:
            self.parent.set_variable(name, value)
        else:
            raise AdarshSemanticError(
                f"Variable '{name}' not declared in AdarshLang scope."
            )

    def get_variable(self, name):
        if name in self.variables:
            return self.variables[name]
        if self.parent:
            return self.parent.get_variable(name)
        raise AdarshSemanticError(
            f"Variable '{name}' not declared in AdarshLang scope."
        )

    def declare_function(self, name, signature):
        self.functions[name] = signature

    def has_function(self, name):
        if name in self.functions:
            return True
        if self.parent:
            return self.parent.has_function(name)
        return False

    def get_function_signature(self, name):
        if name in self.functions:
            return self.functions[name]
        if self.parent:
            return self.parent.get_function_signature(name)
        raise AdarshSemanticError(
            f"Function '{name}' not declared in AdarshLang scope."
        )

    def declare_type(self, name, fields):
        self.types[name] = fields

    def has_type(self, name):
        if name in self.types:
            return True
        if self.parent:
            return self.parent.has_type(name)
        return False

    def get_type_fields(self, name):
        if name in self.types:
            return self.types[name]
        if self.parent:
            return self.parent.get_type_fields(name)
        raise AdarshSemanticError(
            f"Dhacha '{name}' not declared in AdarshLang."
        )


class AdarshSemanticAnalyzer:
    def __init__(self):
        self.imported_modules = set()
        self.builtin_functions = self._register_builtins()

    def _register_builtins(self):
        signatures = {}

        def register(name, signature):
            signatures[name] = signature

        register('length', AdarshFunctionSignature([AdarshParam('_value')]))
        register('push', AdarshFunctionSignature([
            AdarshParam('_list'),
            AdarshParam('_value'),
        ]))
        register('pop', AdarshFunctionSignature([
            AdarshParam('_list'),
            AdarshParam('_index', AdarshNumLiteralNode(-1)),
        ]))
        register('rakho', AdarshFunctionSignature([
            AdarshParam('_dict'),
            AdarshParam('_key'),
            AdarshParam('_value'),
        ]))
        register('nikalo', AdarshFunctionSignature([
            AdarshParam('_collection'),
            AdarshParam('_key_or_index'),
            AdarshParam('_default', AdarshBoolLiteralNode(False)),
        ]))
        register('map', AdarshFunctionSignature([
            AdarshParam('_fn'),
            AdarshParam('_iterable'),
        ]))
        register('filter', AdarshFunctionSignature([
            AdarshParam('_fn'),
            AdarshParam('_iterable'),
        ]))
        register('reduce', AdarshFunctionSignature([
            AdarshParam('_fn'),
            AdarshParam('_iterable'),
            AdarshParam('_initial', AdarshBoolLiteralNode(False)),
        ]))
        register('random_number', AdarshFunctionSignature([
            AdarshParam('_start', AdarshNumLiteralNode(0)),
            AdarshParam('_end', AdarshNumLiteralNode(1)),
        ]))
        register('current_time', AdarshFunctionSignature([]))
        register('abs', AdarshFunctionSignature([AdarshParam('_value')]))
        register('floor', AdarshFunctionSignature([AdarshParam('_value')]))
        register('ceil', AdarshFunctionSignature([AdarshParam('_value')]))
        register('upper', AdarshFunctionSignature([AdarshParam('_value')]))
        register('lower', AdarshFunctionSignature([AdarshParam('_value')]))
        register('join', AdarshFunctionSignature([
            AdarshParam('_iterable'),
            AdarshParam('_sep', AdarshStringLiteralNode('')),
        ]))
        register('split', AdarshFunctionSignature([
            AdarshParam('_value'),
            AdarshParam('_sep', AdarshStringLiteralNode(' ')),
        ]))
        register('prakar', AdarshFunctionSignature([AdarshParam('_value')]))
        register('shabdme', AdarshFunctionSignature([AdarshParam('_value')]))
        register('sankhya', AdarshFunctionSignature([AdarshParam('_value')]))
        register('range', AdarshFunctionSignature([
            AdarshParam('_start'),
            AdarshParam('_end', AdarshNumLiteralNode(0)),
            AdarshParam('_step', AdarshNumLiteralNode(1)),
        ]))
        register('keys', AdarshFunctionSignature([AdarshParam('_dict')]))
        register('values', AdarshFunctionSignature([AdarshParam('_dict')]))
        register('contains', AdarshFunctionSignature([
            AdarshParam('_collection'),
            AdarshParam('_value'),
        ]))
        register('sort', AdarshFunctionSignature([AdarshParam('_list')]))
        register('reverse', AdarshFunctionSignature([AdarshParam('_list')]))
        register('slice', AdarshFunctionSignature([
            AdarshParam('_collection'),
            AdarshParam('_start', AdarshNumLiteralNode(0)),
            AdarshParam('_end', AdarshNullLiteralNode()),
        ]))
        register('replace', AdarshFunctionSignature([
            AdarshParam('_string'),
            AdarshParam('_old'),
            AdarshParam('_new'),
        ]))
        register('trim', AdarshFunctionSignature([AdarshParam('_value')]))
        register('find', AdarshFunctionSignature([
            AdarshParam('_collection'),
            AdarshParam('_value'),
        ]))
        register('min_val', AdarshFunctionSignature([
            AdarshParam('_a'),
            AdarshParam('_b'),
        ]))
        register('max_val', AdarshFunctionSignature([
            AdarshParam('_a'),
            AdarshParam('_b'),
        ]))
        register('round_val', AdarshFunctionSignature([AdarshParam('_value')]))

        return signatures

    def analyze(self, node, scope=None, loop_depth=0):
        if scope is None:
            scope = AdarshSymbolTable()
            for name, signature in self.builtin_functions.items():
                scope.declare_function(name, signature)
                scope.declare_variable(name)

        if isinstance(node, AdarshProgramNode):
            for stmt in node.statements:
                self.analyze(stmt, scope, loop_depth)
        elif isinstance(node, BadloNode):
            scope.declare_variable(node.var_name)
            if node.init_expr is not None:
                self.analyze(node.init_expr, scope, loop_depth)
        elif isinstance(node, AdarshAssignNode):
            scope.get_variable(node.var_name)
            self.analyze(node.expr, scope, loop_depth)
        elif isinstance(node, AdarshCompoundAssignNode):
            scope.get_variable(node.var_name)
            self.analyze(node.expr, scope, loop_depth)
        elif isinstance(node, AdarshIndexAssignNode):
            self.analyze(node.collection, scope, loop_depth)
            self.analyze(node.index_expr, scope, loop_depth)
            self.analyze(node.value_expr, scope, loop_depth)
        elif isinstance(node, AdarshIndexCompoundAssignNode):
            self.analyze(node.collection, scope, loop_depth)
            self.analyze(node.index_expr, scope, loop_depth)
            self.analyze(node.value_expr, scope, loop_depth)
        elif isinstance(node, AdarshAttributeAssignNode):
            self.analyze(node.target, scope, loop_depth)
            self.analyze(node.value_expr, scope, loop_depth)
        elif isinstance(node, AdarshAttributeCompoundAssignNode):
            self.analyze(node.target, scope, loop_depth)
            self.analyze(node.value_expr, scope, loop_depth)
        elif isinstance(node, AdarshBinOpNode):
            self.analyze(node.left, scope, loop_depth)
            self.analyze(node.right, scope, loop_depth)
        elif isinstance(node, AdarshUnaryOpNode):
            self.analyze(node.factor, scope, loop_depth)
        elif isinstance(node, (AdarshNumLiteralNode, AdarshBoolLiteralNode, AdarshStringLiteralNode, AdarshNullLiteralNode)):
            pass
        elif isinstance(node, AdarshListLiteralNode):
            for element in node.elements:
                self.analyze(element, scope, loop_depth)
        elif isinstance(node, AdarshDictLiteralNode):
            for key, value in node.entries:
                self.analyze(key, scope, loop_depth)
                self.analyze(value, scope, loop_depth)
        elif isinstance(node, AdarshDhachaConstructNode):
            if not scope.has_type(node.type_name):
                raise AdarshSemanticError(
                    f"Dhacha '{node.type_name}' not declared in AdarshLang."
                )
            declared_fields = scope.get_type_fields(node.type_name)
            seen_fields = set()
            for field_name, expr in node.field_exprs:
                if field_name not in declared_fields:
                    raise AdarshSemanticError(
                        f"Field '{field_name}' not declared in dhacha '{node.type_name}' in AdarshLang."
                    )
                if field_name in seen_fields:
                    raise AdarshSemanticError(
                        f"Field '{field_name}' provided multiple times for dhacha '{node.type_name}' in AdarshLang."
                    )
                seen_fields.add(field_name)
                self.analyze(expr, scope, loop_depth)
        elif isinstance(node, AdarshVarReferenceNode):
            scope.get_variable(node.var_name)
        elif isinstance(node, AdarshAttributeAccessNode):
            self.analyze(node.target, scope, loop_depth)
        elif isinstance(node, AdarshIndexAccessNode):
            self.analyze(node.collection, scope, loop_depth)
            self.analyze(node.index_expr, scope, loop_depth)
        elif isinstance(node, AdarshDikhaoNode):
            self.analyze(node.expr, scope, loop_depth)
        elif isinstance(node, AdarshBreakNode):
            if loop_depth == 0:
                raise AdarshSemanticError(
                    "'bas' (break) can only be used inside loops in AdarshLang."
                )
        elif isinstance(node, AdarshContinueNode):
            if loop_depth == 0:
                raise AdarshSemanticError(
                    "'aage_badho' (continue) can only be used inside loops in AdarshLang."
                )
        elif isinstance(node, AdarshThrowNode):
            self.analyze(node.expr, scope, loop_depth)
        elif isinstance(node, AdarshImportNode):
            self.analyze(node.module_expr, scope, loop_depth)
            module_name = None
            if isinstance(node.module_expr, AdarshStringLiteralNode):
                module_name = node.module_expr.value
            elif isinstance(node.module_expr, AdarshVarReferenceNode):
                pass
            if module_name:
                module_path = module_name if module_name.endswith('.aak') else module_name + '.aak'
                if module_path not in self.imported_modules and os.path.exists(module_path):
                    self.imported_modules.add(module_path)
                    with open(module_path, 'r', encoding='utf-8') as handle:
                        module_source = handle.read()
                    lexer = AdarshLexer(module_source)
                    tokens = lexer.tokenize()
                    parser = AdarshParser(tokens)
                    module_ast = parser.parse_program()
                    self.analyze(module_ast, scope, loop_depth)
        elif isinstance(node, AdarshIfNode):
            self.analyze(node.condition, scope, loop_depth)
            self.analyze(node.if_block, AdarshSymbolTable(parent=scope), loop_depth)
            if node.else_block:
                self.analyze(node.else_block, AdarshSymbolTable(parent=scope), loop_depth)
        elif isinstance(node, AdarshWhileNode):
            self.analyze(node.condition, scope, loop_depth)
            self.analyze(node.block, AdarshSymbolTable(parent=scope), loop_depth + 1)
        elif isinstance(node, AdarshForNode):
            loop_scope = AdarshSymbolTable(parent=scope)
            if node.init_stmt is not None:
                self.analyze(node.init_stmt, loop_scope, loop_depth + 1)
            if node.condition is not None:
                self.analyze(node.condition, loop_scope, loop_depth + 1)
            if node.update_expr is not None:
                self.analyze(node.update_expr, loop_scope, loop_depth + 1)
            self.analyze(node.block, loop_scope, loop_depth + 1)
        elif isinstance(node, AdarshForEachNode):
            loop_scope = AdarshSymbolTable(parent=scope)
            iter_var, declare_new = node.iter_var
            if declare_new:
                loop_scope.declare_variable(iter_var)
            else:
                scope.get_variable(iter_var)
            self.analyze(node.iterable_expr, scope, loop_depth)
            self.analyze(node.block, loop_scope, loop_depth + 1)
        elif isinstance(node, AdarshSwitchNode):
            self.analyze(node.subject, scope, loop_depth)
            for case_clause in node.cases:
                case_scope = AdarshSymbolTable(parent=scope)
                for match_expr in case_clause.match_exprs:
                    self.analyze(match_expr, scope, loop_depth)
                self.analyze(case_clause.block, case_scope, loop_depth)
            if node.default_block:
                self.analyze(node.default_block, AdarshSymbolTable(parent=scope), loop_depth)
        elif isinstance(node, AdarshTryCatchNode):
            try_scope = AdarshSymbolTable(parent=scope)
            self.analyze(node.try_block, try_scope, loop_depth)
            catch_scope = AdarshSymbolTable(parent=scope)
            catch_scope.declare_variable(node.err_name)
            self.analyze(node.catch_block, catch_scope, loop_depth)
        elif isinstance(node, AdarshBlockNode):
            for statement in node.statements:
                self.analyze(statement, scope, loop_depth)
        elif isinstance(node, AdarshKaamDefNode):
            signature = AdarshFunctionSignature(node.params)
            scope.declare_function(node.func_name, signature)
            scope.declare_variable(node.func_name)
            func_scope = AdarshSymbolTable(parent=scope)
            for param in node.params:
                func_scope.declare_variable(param.name)
            self.analyze(node.body, func_scope, 0)
        elif isinstance(node, AdarshWapasNode):
            if node.expr is not None:
                self.analyze(node.expr, scope, loop_depth)
        elif isinstance(node, AdarshFunctionExprNode):
            func_scope = AdarshSymbolTable(parent=scope)
            for param in node.params:
                func_scope.declare_variable(param.name)
            self.analyze(node.body, func_scope, 0)
        elif isinstance(node, AdarshCallNode):
            self.analyze(node.callee, scope, loop_depth)
            for arg in node.args:
                self.analyze(arg, scope, loop_depth)
            if isinstance(node.callee, AdarshVarReferenceNode) and scope.has_function(node.callee.var_name):
                signature = scope.get_function_signature(node.callee.var_name)
                if not signature.allows(len(node.args)):
                    raise AdarshSemanticError(
                        f"Function '{node.callee.var_name}' called with invalid argument count {len(node.args)} in AdarshLang."
                    )
        elif isinstance(node, AdarshDhachaDefNode):
            scope.declare_type(node.name, node.fields)
        else:
            pass


__all__ = [
    'AdarshSemanticAnalyzer',
    'AdarshSemanticError',
    'AdarshSymbolTable',
    'AdarshFunctionSignature',
]
