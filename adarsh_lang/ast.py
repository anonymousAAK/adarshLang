"""Abstract syntax tree nodes for AdarshLang."""


class AdarshASTNode:
    pass


class AdarshProgramNode(AdarshASTNode):
    def __init__(self, statements):
        self.statements = statements


class BadloNode(AdarshASTNode):
    def __init__(self, var_name, init_expr=None):
        self.var_name = var_name
        self.init_expr = init_expr


class AdarshAssignNode(AdarshASTNode):
    def __init__(self, var_name, expr):
        self.var_name = var_name
        self.expr = expr


class AdarshBinOpNode(AdarshASTNode):
    def __init__(self, left, op, right):
        self.left = left
        self.op = op
        self.right = right


class AdarshUnaryOpNode(AdarshASTNode):
    def __init__(self, op, factor):
        self.op = op
        self.factor = factor


class AdarshNumLiteralNode(AdarshASTNode):
    def __init__(self, value):
        self.value = value


class AdarshBoolLiteralNode(AdarshASTNode):
    def __init__(self, value):
        self.value = value


class AdarshStringLiteralNode(AdarshASTNode):
    def __init__(self, value):
        self.value = value


class AdarshListLiteralNode(AdarshASTNode):
    def __init__(self, elements):
        self.elements = elements


class AdarshDictLiteralNode(AdarshASTNode):
    def __init__(self, entries):
        self.entries = entries


class AdarshDhachaConstructNode(AdarshASTNode):
    def __init__(self, type_name, field_exprs):
        self.type_name = type_name
        self.field_exprs = field_exprs


class AdarshIndexAssignNode(AdarshASTNode):
    def __init__(self, collection, index_expr, value_expr):
        self.collection = collection
        self.index_expr = index_expr
        self.value_expr = value_expr


class AdarshVarReferenceNode(AdarshASTNode):
    def __init__(self, var_name):
        self.var_name = var_name


class AdarshAttributeAccessNode(AdarshASTNode):
    def __init__(self, target, attribute):
        self.target = target
        self.attribute = attribute


class AdarshIndexAccessNode(AdarshASTNode):
    def __init__(self, collection, index_expr):
        self.collection = collection
        self.index_expr = index_expr


class AdarshAttributeAssignNode(AdarshASTNode):
    def __init__(self, target, attribute, value_expr):
        self.target = target
        self.attribute = attribute
        self.value_expr = value_expr


class AdarshDikhaoNode(AdarshASTNode):
    def __init__(self, expr):
        self.expr = expr


class AdarshBreakNode(AdarshASTNode):
    pass


class AdarshContinueNode(AdarshASTNode):
    pass


class AdarshThrowNode(AdarshASTNode):
    def __init__(self, expr):
        self.expr = expr


class AdarshImportNode(AdarshASTNode):
    def __init__(self, module_expr):
        self.module_expr = module_expr


class AdarshIfNode(AdarshASTNode):
    def __init__(self, condition, if_block, else_block=None):
        self.condition = condition
        self.if_block = if_block
        self.else_block = else_block


class AdarshWhileNode(AdarshASTNode):
    def __init__(self, condition, block):
        self.condition = condition
        self.block = block


class AdarshForNode(AdarshASTNode):
    def __init__(self, init_stmt, condition, update_expr, block):
        self.init_stmt = init_stmt
        self.condition = condition
        self.update_expr = update_expr
        self.block = block


class AdarshForEachNode(AdarshASTNode):
    def __init__(self, iter_var, iterable_expr, block):
        self.iter_var = iter_var
        self.iterable_expr = iterable_expr
        self.block = block


class AdarshSwitchCase:
    def __init__(self, match_exprs, block):
        self.match_exprs = match_exprs
        self.block = block


class AdarshSwitchNode(AdarshASTNode):
    def __init__(self, subject, cases, default_block=None):
        self.subject = subject
        self.cases = cases
        self.default_block = default_block


class AdarshTryCatchNode(AdarshASTNode):
    def __init__(self, try_block, err_name, catch_block):
        self.try_block = try_block
        self.err_name = err_name
        self.catch_block = catch_block


class AdarshBlockNode(AdarshASTNode):
    def __init__(self, statements):
        self.statements = statements


class AdarshParam:
    def __init__(self, name, default_expr=None, is_varargs=False):
        self.name = name
        self.default_expr = default_expr
        self.is_varargs = is_varargs


class AdarshKaamDefNode(AdarshASTNode):
    def __init__(self, func_name, params, body):
        self.func_name = func_name
        self.params = params
        self.body = body


class AdarshDhachaDefNode(AdarshASTNode):
    def __init__(self, name, fields):
        self.name = name
        self.fields = fields


class AdarshWapasNode(AdarshASTNode):
    def __init__(self, expr=None):
        self.expr = expr


class AdarshFunctionExprNode(AdarshASTNode):
    def __init__(self, params, body):
        self.params = params
        self.body = body


class AdarshCallNode(AdarshASTNode):
    def __init__(self, callee, args):
        self.callee = callee
        self.args = args


__all__ = [name for name in globals() if name.startswith('Adarsh') or name == 'BadloNode']
