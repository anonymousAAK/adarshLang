"""Parser for AdarshLang."""

from .ast import (
    AdarshAssignNode,
    AdarshAttributeAccessNode,
    AdarshAttributeAssignNode,
    AdarshBinOpNode,
    AdarshBlockNode,
    AdarshBoolLiteralNode,
    AdarshBreakNode,
    AdarshCallNode,
    AdarshContinueNode,
    AdarshDictLiteralNode,
    AdarshDhachaConstructNode,
    AdarshDhachaDefNode,
    AdarshDikhaoNode,
    AdarshForEachNode,
    AdarshForNode,
    AdarshFunctionExprNode,
    AdarshIfNode,
    AdarshImportNode,
    AdarshIndexAccessNode,
    AdarshIndexAssignNode,
    AdarshKaamDefNode,
    AdarshListLiteralNode,
    AdarshNumLiteralNode,
    AdarshParam,
    AdarshProgramNode,
    AdarshStringLiteralNode,
    AdarshSwitchCase,
    AdarshSwitchNode,
    AdarshThrowNode,
    AdarshTryCatchNode,
    AdarshUnaryOpNode,
    AdarshVarReferenceNode,
    AdarshWhileNode,
    AdarshWapasNode,
    BadloNode,
)
from .lexer import AdarshToken
from .tokens import AdarshTokenType


class AdarshParserError(Exception):
    pass


class AdarshParser:
    def __init__(self, tokens):
        self.tokens = tokens
        self.pos = 0
        self.current_token = self.tokens[self.pos]

    def advance(self):
        self.pos += 1
        if self.pos < len(self.tokens):
            self.current_token = self.tokens[self.pos]
        else:
            self.current_token = AdarshToken(AdarshTokenType.EOF, None, -1, -1)

    def peek(self):
        peek_pos = self.pos + 1
        if peek_pos < len(self.tokens):
            return self.tokens[peek_pos]
        return AdarshToken(AdarshTokenType.EOF, None, -1, -1)

    def eat(self, token_type):
        if self.current_token.type == token_type:
            self.advance()
        else:
            raise AdarshParserError(
                f"Expected token type '{token_type}' but got '{self.current_token.type}' "
                f"at line {self.current_token.line}, col {self.current_token.column}"
            )

    def parse_program(self):
        statements = []
        while self.current_token.type != AdarshTokenType.EOF:
            if self.current_token.type == AdarshTokenType.KAAM:
                func_def_node = self.parse_kaam_def()
                statements.append(func_def_node)
            elif self.current_token.type == AdarshTokenType.DHACHA:
                statements.append(self.parse_dhacha_def())
            else:
                stmt = self.parse_statement()
                statements.append(stmt)
        return AdarshProgramNode(statements)

    def parse_kaam_def(self):
        self.eat(AdarshTokenType.KAAM)
        func_name = self.current_token.value
        self.eat(AdarshTokenType.IDENT)
        self.eat(AdarshTokenType.LPAREN)
        params = []
        if self.current_token.type != AdarshTokenType.RPAREN:
            params = self.parse_param_list()
        self.eat(AdarshTokenType.RPAREN)
        body = self.parse_block()
        return AdarshKaamDefNode(func_name, params, body)

    def parse_param_list(self):
        params = []
        seen_varargs = False
        while True:
            is_varargs = False
            if self.current_token.type == AdarshTokenType.BAAKI:
                if seen_varargs:
                    raise AdarshParserError(
                        "Multiple 'baaki' varargs parameters are not allowed in AdarshLang."
                    )
                self.eat(AdarshTokenType.BAAKI)
                is_varargs = True
                seen_varargs = True
            name = self.current_token.value
            self.eat(AdarshTokenType.IDENT)
            default_expr = None
            if self.current_token.type == AdarshTokenType.ASSIGN:
                if is_varargs:
                    raise AdarshParserError(
                        "Varargs parameter cannot have a default value in AdarshLang."
                    )
                self.eat(AdarshTokenType.ASSIGN)
                default_expr = self.parse_expression()
            params.append(AdarshParam(name, default_expr, is_varargs))
            if self.current_token.type != AdarshTokenType.COMMA:
                break
            self.eat(AdarshTokenType.COMMA)
            if self.current_token.type == AdarshTokenType.RPAREN:
                break
        return params

    def parse_statement(self):
        if self.current_token.type == AdarshTokenType.BADLO:
            return self.parse_badlo_decl()
        elif self.current_token.type == AdarshTokenType.AGAR:
            return self.parse_agar()
        elif self.current_token.type == AdarshTokenType.JABTAK:
            return self.parse_jabtak()
        elif self.current_token.type == AdarshTokenType.GINNATI:
            return self.parse_ginnati()
        elif self.current_token.type == AdarshTokenType.KE_LIYE:
            return self.parse_ke_liye()
        elif self.current_token.type == AdarshTokenType.CHUNO:
            return self.parse_chuno()
        elif self.current_token.type == AdarshTokenType.DIKHAO:
            return self.parse_dikhao()
        elif self.current_token.type == AdarshTokenType.BAS:
            return self.parse_bas()
        elif self.current_token.type == AdarshTokenType.AAGE_BADHO:
            return self.parse_aage_badho()
        elif self.current_token.type == AdarshTokenType.PAKDO:
            return self.parse_pakdo()
        elif self.current_token.type == AdarshTokenType.CHHODDO:
            return self.parse_throw_statement()
        elif self.current_token.type == AdarshTokenType.LAO:
            return self.parse_import_statement()
        elif self.current_token.type == AdarshTokenType.WAPAS:
            return self.parse_wapas()
        elif self.current_token.type == AdarshTokenType.LBRACE:
            return self.parse_block()
        else:
            expr = self.parse_assignment_expression()
            self.eat(AdarshTokenType.SEMI)
            return expr

    def parse_badlo_decl(self):
        self.eat(AdarshTokenType.BADLO)
        var_name = self.current_token.value
        self.eat(AdarshTokenType.IDENT)
        init_expr = None
        if self.current_token.type == AdarshTokenType.ASSIGN:
            self.eat(AdarshTokenType.ASSIGN)
            init_expr = self.parse_expression()
        self.eat(AdarshTokenType.SEMI)
        return BadloNode(var_name, init_expr)

    def parse_dikhao(self):
        self.eat(AdarshTokenType.DIKHAO)
        expr = self.parse_expression()
        self.eat(AdarshTokenType.SEMI)
        return AdarshDikhaoNode(expr)

    def parse_bas(self):
        self.eat(AdarshTokenType.BAS)
        self.eat(AdarshTokenType.SEMI)
        return AdarshBreakNode()

    def parse_aage_badho(self):
        self.eat(AdarshTokenType.AAGE_BADHO)
        self.eat(AdarshTokenType.SEMI)
        return AdarshContinueNode()

    def parse_wapas(self):
        self.eat(AdarshTokenType.WAPAS)
        expr = None
        if self.current_token.type != AdarshTokenType.SEMI:
            expr = self.parse_expression()
        self.eat(AdarshTokenType.SEMI)
        return AdarshWapasNode(expr)

    def parse_block(self):
        self.eat(AdarshTokenType.LBRACE)
        statements = []
        while self.current_token.type != AdarshTokenType.RBRACE:
            statements.append(self.parse_statement())
        self.eat(AdarshTokenType.RBRACE)
        return AdarshBlockNode(statements)

    def parse_assignment_expression(self):
        expr = self.parse_expression()
        if self.current_token.type == AdarshTokenType.ASSIGN:
            self.eat(AdarshTokenType.ASSIGN)
            right_expr = self.parse_expression()
            if isinstance(expr, AdarshVarReferenceNode):
                return AdarshAssignNode(expr.var_name, right_expr)
            if isinstance(expr, AdarshIndexAccessNode):
                return AdarshIndexAssignNode(expr.collection, expr.index_expr, right_expr)
            if isinstance(expr, AdarshAttributeAccessNode):
                return AdarshAttributeAssignNode(expr.target, expr.attribute, right_expr)
            raise AdarshParserError("Invalid assignment target in AdarshLang.")
        return expr

    def parse_expression(self):
        return self.parse_logical_or()

    def parse_logical_or(self):
        node = self.parse_logical_and()
        while self.current_token.type == AdarshTokenType.YA:
            op = self.current_token.type
            self.eat(op)
            right = self.parse_logical_and()
            node = AdarshBinOpNode(node, op, right)
        return node

    def parse_logical_and(self):
        node = self.parse_equality()
        while self.current_token.type == AdarshTokenType.AUR:
            op = self.current_token.type
            self.eat(op)
            right = self.parse_equality()
            node = AdarshBinOpNode(node, op, right)
        return node

    def parse_equality(self):
        node = self.parse_comparison()
        while self.current_token.type in (AdarshTokenType.EQ, AdarshTokenType.NEQ):
            op = self.current_token.type
            self.eat(op)
            right = self.parse_comparison()
            node = AdarshBinOpNode(node, op, right)
        return node

    def parse_comparison(self):
        node = self.parse_term()
        while self.current_token.type in (
            AdarshTokenType.LT,
            AdarshTokenType.GT,
            AdarshTokenType.LTE,
            AdarshTokenType.GTE,
        ):
            op = self.current_token.type
            self.eat(op)
            right = self.parse_term()
            node = AdarshBinOpNode(node, op, right)
        return node

    def parse_term(self):
        node = self.parse_factor()
        while self.current_token.type in (AdarshTokenType.PLUS, AdarshTokenType.MINUS):
            op = self.current_token.type
            self.eat(op)
            right = self.parse_factor()
            node = AdarshBinOpNode(node, op, right)
        return node

    def parse_factor(self):
        node = self.parse_unary()
        while self.current_token.type in (AdarshTokenType.MUL, AdarshTokenType.DIV):
            op = self.current_token.type
            self.eat(op)
            right = self.parse_unary()
            node = AdarshBinOpNode(node, op, right)
        return node

    def parse_unary(self):
        if self.current_token.type in (AdarshTokenType.NAHIN, AdarshTokenType.MINUS):
            op = self.current_token.type
            self.eat(op)
            factor = self.parse_unary()
            return AdarshUnaryOpNode(op, factor)
        else:
            return self.parse_postfix()

    def parse_postfix(self):
        node = self.parse_primary()
        while True:
            if self.current_token.type == AdarshTokenType.LPAREN:
                self.eat(AdarshTokenType.LPAREN)
                args = []
                if self.current_token.type != AdarshTokenType.RPAREN:
                    args = self.parse_args()
                self.eat(AdarshTokenType.RPAREN)
                node = AdarshCallNode(node, args)
            elif self.current_token.type == AdarshTokenType.LBRACKET:
                self.eat(AdarshTokenType.LBRACKET)
                index_expr = self.parse_expression()
                self.eat(AdarshTokenType.RBRACKET)
                node = AdarshIndexAccessNode(node, index_expr)
            elif self.current_token.type == AdarshTokenType.DOT:
                self.eat(AdarshTokenType.DOT)
                attr_name = self.current_token.value
                self.eat(AdarshTokenType.IDENT)
                node = AdarshAttributeAccessNode(node, attr_name)
            elif (
                self.current_token.type == AdarshTokenType.LBRACE
                and isinstance(node, AdarshVarReferenceNode)
            ):
                node = self.parse_dhacha_construction(node.var_name)
            else:
                break
        return node

    def parse_primary(self):
        token = self.current_token
        if token.type == AdarshTokenType.IDENT:
            self.eat(AdarshTokenType.IDENT)
            node = AdarshVarReferenceNode(token.value)
        elif token.type == AdarshTokenType.NUMBER:
            self.eat(AdarshTokenType.NUMBER)
            node = AdarshNumLiteralNode(token.value)
        elif token.type == AdarshTokenType.SAHI_HAI_BE:
            self.eat(AdarshTokenType.SAHI_HAI_BE)
            node = AdarshBoolLiteralNode(True)
        elif token.type == AdarshTokenType.JHUTH:
            self.eat(AdarshTokenType.JHUTH)
            node = AdarshBoolLiteralNode(False)
        elif token.type == AdarshTokenType.STRING:
            self.eat(AdarshTokenType.STRING)
            node = AdarshStringLiteralNode(token.value)
        elif token.type == AdarshTokenType.LBRACKET:
            node = self.parse_list_literal()
        elif token.type == AdarshTokenType.LBRACE:
            node = self.parse_dict_literal()
        elif token.type == AdarshTokenType.LPAREN:
            self.eat(AdarshTokenType.LPAREN)
            node = self.parse_expression()
            self.eat(AdarshTokenType.RPAREN)
        elif token.type == AdarshTokenType.KAAM:
            self.eat(AdarshTokenType.KAAM)
            self.eat(AdarshTokenType.LPAREN)
            params = []
            if self.current_token.type != AdarshTokenType.RPAREN:
                params = self.parse_param_list()
            self.eat(AdarshTokenType.RPAREN)
            body = self.parse_block()
            node = AdarshFunctionExprNode(params, body)
        else:
            raise AdarshParserError(
                f"Unexpected token '{token.type}' at line {token.line}, col {token.column}"
            )
        return node

    def parse_list_literal(self):
        self.eat(AdarshTokenType.LBRACKET)
        elements = []
        if self.current_token.type != AdarshTokenType.RBRACKET:
            elements.append(self.parse_expression())
            while self.current_token.type == AdarshTokenType.COMMA:
                self.eat(AdarshTokenType.COMMA)
                elements.append(self.parse_expression())
        self.eat(AdarshTokenType.RBRACKET)
        return AdarshListLiteralNode(elements)

    def parse_dict_literal(self):
        self.eat(AdarshTokenType.LBRACE)
        entries = []
        if self.current_token.type != AdarshTokenType.RBRACE:
            key = self.parse_expression()
            self.eat(AdarshTokenType.COLON)
            value = self.parse_expression()
            entries.append((key, value))
            while self.current_token.type == AdarshTokenType.COMMA:
                self.eat(AdarshTokenType.COMMA)
                key = self.parse_expression()
                self.eat(AdarshTokenType.COLON)
                value = self.parse_expression()
                entries.append((key, value))
        self.eat(AdarshTokenType.RBRACE)
        return AdarshDictLiteralNode(entries)

    def parse_dhacha_construction(self, type_name):
        self.eat(AdarshTokenType.LBRACE)
        field_exprs = []
        if self.current_token.type != AdarshTokenType.RBRACE:
            field_name = self.current_token.value
            self.eat(AdarshTokenType.IDENT)
            self.eat(AdarshTokenType.COLON)
            expr = self.parse_expression()
            field_exprs.append((field_name, expr))
            while self.current_token.type == AdarshTokenType.COMMA:
                self.eat(AdarshTokenType.COMMA)
                field_name = self.current_token.value
                self.eat(AdarshTokenType.IDENT)
                self.eat(AdarshTokenType.COLON)
                expr = self.parse_expression()
                field_exprs.append((field_name, expr))
        self.eat(AdarshTokenType.RBRACE)
        return AdarshDhachaConstructNode(type_name, field_exprs)

    def parse_args(self):
        args = [self.parse_expression()]
        while self.current_token.type == AdarshTokenType.COMMA:
            self.eat(AdarshTokenType.COMMA)
            args.append(self.parse_expression())
        return args

    def parse_agar(self):
        self.eat(AdarshTokenType.AGAR)
        self.eat(AdarshTokenType.LPAREN)
        condition = self.parse_expression()
        self.eat(AdarshTokenType.RPAREN)
        if_block = self.parse_block()
        else_block = None
        if self.current_token.type == AdarshTokenType.WARNA:
            self.eat(AdarshTokenType.WARNA)
            if self.current_token.type == AdarshTokenType.AGAR:
                else_block = self.parse_agar()
            else:
                else_block = self.parse_block()
        return AdarshIfNode(condition, if_block, else_block)

    def parse_jabtak(self):
        self.eat(AdarshTokenType.JABTAK)
        self.eat(AdarshTokenType.LPAREN)
        condition = self.parse_expression()
        self.eat(AdarshTokenType.RPAREN)
        block = self.parse_block()
        return AdarshWhileNode(condition, block)

    def parse_ginnati(self):
        self.eat(AdarshTokenType.GINNATI)
        self.eat(AdarshTokenType.LPAREN)
        init_stmt = None
        if self.current_token.type != AdarshTokenType.SEMI:
            if self.current_token.type == AdarshTokenType.BADLO:
                init_stmt = self.parse_badlo_decl()
            else:
                init_stmt = self.parse_assignment_expression()
                self.eat(AdarshTokenType.SEMI)
        else:
            self.eat(AdarshTokenType.SEMI)
        condition = None
        if self.current_token.type != AdarshTokenType.SEMI:
            condition = self.parse_expression()
        self.eat(AdarshTokenType.SEMI)
        update_expr = None
        if self.current_token.type != AdarshTokenType.RPAREN:
            update_expr = self.parse_assignment_expression()
        self.eat(AdarshTokenType.RPAREN)
        block = self.parse_block()
        return AdarshForNode(init_stmt, condition, update_expr, block)

    def parse_ke_liye(self):
        self.eat(AdarshTokenType.KE_LIYE)
        self.eat(AdarshTokenType.LPAREN)
        declare_new = False
        if self.current_token.type == AdarshTokenType.BADLO:
            self.eat(AdarshTokenType.BADLO)
            declare_new = True
        iter_var = self.current_token.value
        self.eat(AdarshTokenType.IDENT)
        if self.current_token.type == AdarshTokenType.IN:
            self.eat(AdarshTokenType.IN)
        else:
            self.eat(AdarshTokenType.COLON)
        iterable_expr = self.parse_expression()
        self.eat(AdarshTokenType.RPAREN)
        block = self.parse_block()
        return AdarshForEachNode((iter_var, declare_new), iterable_expr, block)

    def parse_chuno(self):
        self.eat(AdarshTokenType.CHUNO)
        self.eat(AdarshTokenType.LPAREN)
        subject = self.parse_expression()
        self.eat(AdarshTokenType.RPAREN)
        self.eat(AdarshTokenType.LBRACE)
        cases = []
        default_block = None
        while self.current_token.type in (
            AdarshTokenType.CASE,
            AdarshTokenType.WARNA_CASE,
        ):
            if self.current_token.type == AdarshTokenType.CASE:
                self.eat(AdarshTokenType.CASE)
                match_exprs = [self.parse_expression()]
                while self.current_token.type == AdarshTokenType.COMMA:
                    self.eat(AdarshTokenType.COMMA)
                    match_exprs.append(self.parse_expression())
                self.eat(AdarshTokenType.COLON)
                block = self.parse_case_body()
                cases.append(AdarshSwitchCase(match_exprs, block))
            else:
                self.eat(AdarshTokenType.WARNA_CASE)
                self.eat(AdarshTokenType.COLON)
                default_block = self.parse_case_body()
                break
        self.eat(AdarshTokenType.RBRACE)
        return AdarshSwitchNode(subject, cases, default_block)

    def parse_case_body(self):
        if self.current_token.type == AdarshTokenType.LBRACE:
            return self.parse_block()
        statement = self.parse_statement()
        return AdarshBlockNode([statement])

    def parse_pakdo(self):
        self.eat(AdarshTokenType.PAKDO)
        try_block = self.parse_block()
        self.eat(AdarshTokenType.CHHODDO)
        self.eat(AdarshTokenType.LPAREN)
        err_name = self.current_token.value
        self.eat(AdarshTokenType.IDENT)
        self.eat(AdarshTokenType.RPAREN)
        catch_block = self.parse_block()
        return AdarshTryCatchNode(try_block, err_name, catch_block)

    def parse_throw_statement(self):
        self.eat(AdarshTokenType.CHHODDO)
        self.eat(AdarshTokenType.LPAREN)
        expr = self.parse_expression()
        self.eat(AdarshTokenType.RPAREN)
        self.eat(AdarshTokenType.SEMI)
        return AdarshThrowNode(expr)

    def parse_import_statement(self):
        self.eat(AdarshTokenType.LAO)
        module_expr = self.parse_expression()
        self.eat(AdarshTokenType.SEMI)
        return AdarshImportNode(module_expr)

    def parse_dhacha_def(self):
        self.eat(AdarshTokenType.DHACHA)
        type_name = self.current_token.value
        self.eat(AdarshTokenType.IDENT)
        self.eat(AdarshTokenType.LBRACE)
        fields = []
        if self.current_token.type != AdarshTokenType.RBRACE:
            fields.append(self.current_token.value)
            self.eat(AdarshTokenType.IDENT)
            while self.current_token.type == AdarshTokenType.COMMA:
                self.eat(AdarshTokenType.COMMA)
                fields.append(self.current_token.value)
                self.eat(AdarshTokenType.IDENT)
        self.eat(AdarshTokenType.RBRACE)
        self.eat(AdarshTokenType.SEMI)
        return AdarshDhachaDefNode(type_name, fields)


__all__ = ['AdarshParser', 'AdarshParserError']
