import sys
import re
import os
import math
import random
import time

# =====================================================
# =============== LEXICAL ANALYSIS ====================
# =====================================================

class AdarshTokenType:
    EOF         = 'EOF'
    IDENT       = 'IDENT'
    NUMBER      = 'NUMBER'
    STRING      = 'STRING'

    # Keywords
    BADLO        = 'BADLO'        # badlo
    KAAM         = 'KAAM'         # kaam
    WAPAS        = 'WAPAS'        # wapas
    AGAR         = 'AGAR'         # agar
    WARNA        = 'WARNA'        # warna
    JABTAK       = 'JABTAK'       # jabtak
    DIKHAO       = 'DIKHAO'       # dikhao
    BAS          = 'BAS'          # bas
    AAGE_BADHO   = 'AAGE_BADHO'   # aage_badho
    SAHI_HAI_BE  = 'SAHI_HAI_BE'  # sahi_hai_be
    JHUTH        = 'JHUTH'        # jhuth
    GINNATI      = 'GINNATI'      # ginnati
    KE_LIYE      = 'KE_LIYE'      # ke_liye
    CHUNO        = 'CHUNO'        # chuno
    CASE         = 'CASE'         # case
    WARNA_CASE   = 'WARNA_CASE'   # warna_case
    PAKDO        = 'PAKDO'        # pakdo
    CHHODDO      = 'CHHODDO'      # chhoddo
    LAO          = 'LAO'          # lao
    DHACHA       = 'DHACHA'       # dhacha
    BAAKI        = 'BAAKI'        # baaki (varargs marker)
    BADLO       = 'BADLO'       # badlo
    KAAM        = 'KAAM'        # kaam
    WAPAS       = 'WAPAS'       # wapas
    AGAR        = 'AGAR'        # agar
    WARNA       = 'WARNA'       # warna
    JABTAK      = 'JABTAK'      # jabtak
    DIKHAO      = 'DIKHAO'      # dikhao
    BAS         = 'BAS'         # bas
    AAGE_BADHO  = 'AAGE_BADHO'  # aage_badho
    SAHI_HAI_BE = 'SAHI_HAI_BE' # sahi_hai_be
    JHUTH       = 'JHUTH'       # jhuth

    # Operators
    PLUS        = '+'
    MINUS       = '-'
    MUL         = '*'
    DIV         = '/'
    ASSIGN      = '='
    EQ          = '=='
    NEQ         = '!='
    LT          = '<'
    GT          = '>'
    LTE         = '<='
    GTE         = '>='
    AUR         = '&&'   # aur
    YA          = '||'   # ya
    NAHIN       = '!'    # nahin
    SEMI        = ';'
    COMMA       = ','
    COLON       = ':'
    DOT         = '.'
    LPAREN      = '('
    RPAREN      = ')'
    LBRACE      = '{'
    RBRACE      = '}'
    LBRACKET    = '['
    RBRACKET    = ']'

HINGLISH_KEYWORDS = {
    'badlo':       AdarshTokenType.BADLO,
    'kaam':        AdarshTokenType.KAAM,
    'wapas':       AdarshTokenType.WAPAS,
    'agar':        AdarshTokenType.AGAR,
    'warna':       AdarshTokenType.WARNA,
    'jabtak':      AdarshTokenType.JABTAK,
    'dikhao':      AdarshTokenType.DIKHAO,
    'bas':         AdarshTokenType.BAS,
    'aage_badho':  AdarshTokenType.AAGE_BADHO,
    'sahi_hai_be': AdarshTokenType.SAHI_HAI_BE,
    'jhuth':       AdarshTokenType.JHUTH,
    'ginnati':     AdarshTokenType.GINNATI,
    'ke_liye':     AdarshTokenType.KE_LIYE,
    'chuno':       AdarshTokenType.CHUNO,
    'case':        AdarshTokenType.CASE,
    'warna_case':  AdarshTokenType.WARNA_CASE,
    'pakdo':       AdarshTokenType.PAKDO,
    'chhoddo':     AdarshTokenType.CHHODDO,
    'lao':         AdarshTokenType.LAO,
    'dhacha':      AdarshTokenType.DHACHA,
    'baaki':       AdarshTokenType.BAAKI,
}

HINGLISH_OPERATORS = {
    '+':  AdarshTokenType.PLUS,
    '-':  AdarshTokenType.MINUS,
    '*':  AdarshTokenType.MUL,
    '/':  AdarshTokenType.DIV,
    '=':  AdarshTokenType.ASSIGN,
    ';':  AdarshTokenType.SEMI,
    ',':  AdarshTokenType.COMMA,
    ':':  AdarshTokenType.COLON,
    '.':  AdarshTokenType.DOT,
    '(':  AdarshTokenType.LPAREN,
    ')':  AdarshTokenType.RPAREN,
    '{':  AdarshTokenType.LBRACE,
    '}':  AdarshTokenType.RBRACE,
    '[':  AdarshTokenType.LBRACKET,
    ']':  AdarshTokenType.RBRACKET,
    '!':  AdarshTokenType.NAHIN,
}

HINGLISH_MULTI_OPERATORS = {
    '==': AdarshTokenType.EQ,
    '!=': AdarshTokenType.NEQ,
    '<=': AdarshTokenType.LTE,
    '>=': AdarshTokenType.GTE,
    '&&': AdarshTokenType.AUR,
    '||': AdarshTokenType.YA,
    '<':  AdarshTokenType.LT,
    '>':  AdarshTokenType.GT
}

class AdarshToken:
    def __init__(self, ttype, value, line, column):
        self.type = ttype
        self.value = value
        self.line = line
        self.column = column

    def __repr__(self):
        return f"AdarshToken({self.type}, {self.value}, line={self.line}, col={self.column})"

class AdarshLexer:
    def __init__(self, text):
        self.text = text
        self.pos = 0
        self.line = 1
        self.column = 1
        self.current_char = self.text[self.pos] if self.text else None
    
    def advance(self):
        if self.current_char == '\n':
            self.line += 1
            self.column = 0
        self.pos += 1
        self.column += 1
        if self.pos >= len(self.text):
            self.current_char = None
        else:
            self.current_char = self.text[self.pos]

    def peek(self):
        peek_pos = self.pos + 1
        if peek_pos >= len(self.text):
            return None
        return self.text[peek_pos]

    def skip_whitespace(self):
        while self.current_char is not None and self.current_char.isspace():
            self.advance()

    def skip_comment(self):
        while self.current_char is not None and self.current_char != '\n':
            self.advance()

    def make_number(self):
        start_line = self.line
        start_col = self.column
        number_str = ''
        dot_count = 0

        while self.current_char is not None and (self.current_char.isdigit() or self.current_char == '.'):
            if self.current_char == '.':
                dot_count += 1
                if dot_count > 1:
                    break
            number_str += self.current_char
            self.advance()
        if '.' in number_str:
            return AdarshToken(AdarshTokenType.NUMBER, float(number_str), start_line, start_col)
        else:
            return AdarshToken(AdarshTokenType.NUMBER, int(number_str), start_line, start_col)

    def make_string(self):
        start_line = self.line
        start_col = self.column
        self.advance()  # skip the initial quote
        result = ''
        while self.current_char is not None and self.current_char != '"':
            result += self.current_char
            self.advance()
        self.advance()  # skip the closing quote
        return AdarshToken(AdarshTokenType.STRING, result, start_line, start_col)

    def make_identifier(self):
        start_line = self.line
        start_col = self.column
        ident_str = ''
        while (self.current_char is not None and 
               (self.current_char.isalnum() or self.current_char == '_')):
            ident_str += self.current_char
            self.advance()

        token_type = HINGLISH_KEYWORDS.get(ident_str, AdarshTokenType.IDENT)
        return AdarshToken(token_type, ident_str, start_line, start_col)

    def handle_multi_char_operator(self):
        start_line = self.line
        start_col = self.column
        op = self.current_char
        next_ch = self.peek()
        if next_ch:
            potential_op = op + next_ch
            if potential_op in HINGLISH_MULTI_OPERATORS:
                self.advance()
                self.advance()
                return AdarshToken(HINGLISH_MULTI_OPERATORS[potential_op], potential_op, start_line, start_col)
        self.advance()
        if op in HINGLISH_MULTI_OPERATORS:
            return AdarshToken(HINGLISH_MULTI_OPERATORS[op], op, start_line, start_col)
        elif op in HINGLISH_OPERATORS:
            return AdarshToken(HINGLISH_OPERATORS[op], op, start_line, start_col)
        return AdarshToken(None, op, start_line, start_col)

    def get_next_token(self):
        while self.current_char is not None:
            if self.current_char.isspace():
                self.skip_whitespace()
                continue
            if self.current_char == '#':
                self.skip_comment()
                continue
            if self.current_char == '"':
                return self.make_string()
            if self.current_char.isdigit():
                return self.make_number()
            if self.current_char.isalpha() or self.current_char == '_':
                return self.make_identifier()
            return self.handle_multi_char_operator()
        return AdarshToken(AdarshTokenType.EOF, None, self.line, self.column)

    def tokenize(self):
        tokens = []
        while True:
            tok = self.get_next_token()
            tokens.append(tok)
            if tok.type == AdarshTokenType.EOF:
                break
        return tokens

# =====================================================
# ================ PARSE NODES (AST) ==================
# =====================================================

class AdarshASTNode: pass

class AdarshProgramNode(AdarshASTNode):
    def __init__(self, statements):
        self.statements = statements

class BadloNode(AdarshASTNode):
    def __init__(self, var_name, init_expr):
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
        self.entries = entries  # list of (key_expr, value_expr)

class AdarshDhachaConstructNode(AdarshASTNode):
    def __init__(self, type_name, field_exprs):
        self.type_name = type_name
        self.field_exprs = field_exprs  # list of (field_name, expr)

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

class AdarshCaseClause:
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

class AdarshKaamDefNode(AdarshASTNode):
    def __init__(self, func_name, params, body):
        self.func_name = func_name
        self.params = params  # list of AdarshParam
        self.body = body

class AdarshDhachaDefNode(AdarshASTNode):
    def __init__(self, name, fields):
        self.name = name
        self.fields = fields

class AdarshWapasNode(AdarshASTNode):
    def __init__(self, expr):
        self.expr = expr

class AdarshFunctionExprNode(AdarshASTNode):
    def __init__(self, params, body):
        self.params = params
        self.body = body

class AdarshCallNode(AdarshASTNode):
    def __init__(self, callee, args):
        self.callee = callee
        self.args = args

# =====================================================
# =============== RECURSIVE DESCENT PARSER ============
# =====================================================

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
                    raise AdarshParserError("Multiple 'baaki' varargs parameters are not allowed in AdarshLang.")
                self.eat(AdarshTokenType.BAAKI)
                is_varargs = True
                seen_varargs = True
            name = self.current_token.value
            self.eat(AdarshTokenType.IDENT)
            default_expr = None
            if self.current_token.type == AdarshTokenType.ASSIGN:
                if is_varargs:
                    raise AdarshParserError("Varargs parameter cannot have a default value in AdarshLang.")
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
            expr = self.parse_expression()
            if self.current_token.type == AdarshTokenType.ASSIGN:
                self.eat(AdarshTokenType.ASSIGN)
                right_expr = self.parse_expression()
                self.eat(AdarshTokenType.SEMI)
                if isinstance(expr, AdarshVarReferenceNode):
                    return AdarshAssignNode(expr.var_name, right_expr)
                elif isinstance(expr, AdarshIndexAccessNode):
                    return AdarshIndexAssignNode(expr.collection, expr.index_expr, right_expr)
                elif isinstance(expr, AdarshAttributeAccessNode):
                    return AdarshAttributeAssignNode(expr.target, expr.attribute, right_expr)
                if isinstance(expr, AdarshVarReferenceNode):
                    var_name = expr.var_name
                    self.eat(AdarshTokenType.ASSIGN)
                    right_expr = self.parse_expression()
                    self.eat(AdarshTokenType.SEMI)
                    return AdarshAssignNode(var_name, right_expr)
                elif isinstance(expr, AdarshIndexAccessNode):
                    self.eat(AdarshTokenType.ASSIGN)
                    value_expr = self.parse_expression()
                    self.eat(AdarshTokenType.SEMI)
                    return AdarshIndexAssignNode(expr.collection, expr.index_expr, value_expr)
                else:
                    raise AdarshParserError("Invalid assignment target in AdarshLang.")
            else:
                self.eat(AdarshTokenType.SEMI)
                return expr

    def parse_badlo_decl(self, expect_semi=True):
        self.eat(AdarshTokenType.BADLO)
        var_name = self.current_token.value
        self.eat(AdarshTokenType.IDENT)
        init_expr = None
        if self.current_token.type == AdarshTokenType.ASSIGN:
            self.eat(AdarshTokenType.ASSIGN)
            init_expr = self.parse_expression()
        if expect_semi:
            self.eat(AdarshTokenType.SEMI)
        return BadloNode(var_name, init_expr)

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
        block_node = self.parse_block()
        return AdarshWhileNode(condition, block_node)

    def parse_ginnati(self):
        self.eat(AdarshTokenType.GINNATI)
        self.eat(AdarshTokenType.LPAREN)
        init_stmt = None
        if self.current_token.type != AdarshTokenType.SEMI:
            if self.current_token.type == AdarshTokenType.BADLO:
                init_stmt = self.parse_badlo_decl(expect_semi=False)
            elif self.current_token.type == AdarshTokenType.IDENT and self.peek().type == AdarshTokenType.ASSIGN:
                var_name = self.current_token.value
                self.eat(AdarshTokenType.IDENT)
                self.eat(AdarshTokenType.ASSIGN)
                expr = self.parse_expression()
                init_stmt = AdarshAssignNode(var_name, expr)
            else:
                init_stmt = self.parse_expression()
            self.eat(AdarshTokenType.SEMI)
        else:
            self.eat(AdarshTokenType.SEMI)

        condition = None
        if self.current_token.type != AdarshTokenType.SEMI:
            condition = self.parse_expression()
        self.eat(AdarshTokenType.SEMI)

        update_expr = None
        if self.current_token.type != AdarshTokenType.RPAREN:
            if self.current_token.type == AdarshTokenType.IDENT and self.peek().type == AdarshTokenType.ASSIGN:
                var_name = self.current_token.value
                self.eat(AdarshTokenType.IDENT)
                self.eat(AdarshTokenType.ASSIGN)
                expr = self.parse_expression()
                update_expr = AdarshAssignNode(var_name, expr)
            else:
                update_expr = self.parse_expression()
        self.eat(AdarshTokenType.RPAREN)
        block = self.parse_block()
        return AdarshForNode(init_stmt, condition, update_expr, block)

    def parse_ke_liye(self):
        self.eat(AdarshTokenType.KE_LIYE)
        self.eat(AdarshTokenType.LPAREN)
        declare_new = False
        if self.current_token.type == AdarshTokenType.BADLO:
            declare_new = True
            self.eat(AdarshTokenType.BADLO)
        iter_var = self.current_token.value
        self.eat(AdarshTokenType.IDENT)
        if declare_new:
            iter_var_node = BadloNode(iter_var, None)
        else:
            iter_var_node = AdarshVarReferenceNode(iter_var)
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
        while self.current_token.type != AdarshTokenType.RBRACE:
            if self.current_token.type == AdarshTokenType.CASE:
                self.eat(AdarshTokenType.CASE)
                match_exprs = [self.parse_expression()]
                while self.current_token.type == AdarshTokenType.COMMA:
                    self.eat(AdarshTokenType.COMMA)
                    match_exprs.append(self.parse_expression())
                self.eat(AdarshTokenType.COLON)
                statements = []
                while self.current_token.type not in (AdarshTokenType.CASE, AdarshTokenType.WARNA_CASE, AdarshTokenType.RBRACE):
                    statements.append(self.parse_statement())
                cases.append(AdarshCaseClause(match_exprs, AdarshBlockNode(statements)))
            elif self.current_token.type == AdarshTokenType.WARNA_CASE:
                self.eat(AdarshTokenType.WARNA_CASE)
                self.eat(AdarshTokenType.COLON)
                statements = []
                while self.current_token.type != AdarshTokenType.RBRACE:
                    statements.append(self.parse_statement())
                default_block = AdarshBlockNode(statements)
                break
            else:
                raise AdarshParserError("Expected 'case' or 'warna_case' inside chuno block in AdarshLang.")
        self.eat(AdarshTokenType.RBRACE)
        return AdarshSwitchNode(subject, cases, default_block)

    def parse_dikhao(self):
        self.eat(AdarshTokenType.DIKHAO)
        self.eat(AdarshTokenType.LPAREN)
        expr = self.parse_expression()
        self.eat(AdarshTokenType.RPAREN)
        self.eat(AdarshTokenType.SEMI)
        return AdarshDikhaoNode(expr)

    def parse_pakdo(self):
        self.eat(AdarshTokenType.PAKDO)
        try_block = self.parse_block()
        if self.current_token.type != AdarshTokenType.CHHODDO:
            raise AdarshParserError("'pakdo' must be followed by a 'chhoddo' catch block in AdarshLang.")
        self.eat(AdarshTokenType.CHHODDO)
        self.eat(AdarshTokenType.LPAREN)
        err_name = self.current_token.value
        self.eat(AdarshTokenType.IDENT)
        self.eat(AdarshTokenType.RPAREN)
        catch_block = self.parse_block()
        return AdarshTryCatchNode(try_block, err_name, catch_block)

    def parse_throw_statement(self):
        self.eat(AdarshTokenType.CHHODDO)
        if self.current_token.type == AdarshTokenType.LPAREN:
            self.eat(AdarshTokenType.LPAREN)
            expr = self.parse_expression()
            self.eat(AdarshTokenType.RPAREN)
        else:
            expr = self.parse_expression()
        self.eat(AdarshTokenType.SEMI)
        return AdarshThrowNode(expr)

    def parse_import_statement(self):
        self.eat(AdarshTokenType.LAO)
        module_expr = self.parse_expression()
        self.eat(AdarshTokenType.SEMI)
        return AdarshImportNode(module_expr)

    def parse_dhacha_def(self):
        self.eat(AdarshTokenType.DHACHA)
        name = self.current_token.value
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
        return AdarshDhachaDefNode(name, fields)

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
        while self.current_token.type in (AdarshTokenType.LT, AdarshTokenType.GT,
                                          AdarshTokenType.LTE, AdarshTokenType.GTE):
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
    def parse_kaam_call_or_var_or_literal(self):
        node = self.parse_primary()
        while self.current_token.type == AdarshTokenType.LBRACKET:
            self.eat(AdarshTokenType.LBRACKET)
            index_expr = self.parse_expression()
            self.eat(AdarshTokenType.RBRACKET)
            node = AdarshIndexAccessNode(node, index_expr)
        return node

    def parse_primary(self):
        token = self.current_token
        if token.type == AdarshTokenType.IDENT:
            next_tok = self.peek()
            if next_tok.type == AdarshTokenType.LPAREN:
                func_name = token.value
                self.eat(AdarshTokenType.IDENT)
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
            elif self.current_token.type == AdarshTokenType.LBRACE and isinstance(node, AdarshVarReferenceNode):
                node = self.parse_dhacha_construction(node.var_name)
            else:
                break
        return node

    def parse_primary(self):
        token = self.current_token
        if token.type == AdarshTokenType.IDENT:
            self.eat(AdarshTokenType.IDENT)
            node = AdarshVarReferenceNode(token.value)
                node = AdarshKaamCallNode(func_name, args)
            else:
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

# =====================================================
# =============== SEMANTIC ANALYSIS ==================
# =====================================================

class AdarshSemanticError(Exception):
    pass

class AdarshSymbolTable:
    def __init__(self, parent=None):
        self.parent = parent
        self.variables = {}
        self.functions = {}
        self.types = {}

    def declare_variable(self, name):
        if name in self.variables:
            pass
        else:
            self.variables[name] = None

    def set_variable(self, name, value):
        if name in self.variables:
            self.variables[name] = value
        else:
            if self.parent:
                self.parent.set_variable(name, value)
            else:
                raise AdarshSemanticError(f"Variable '{name}' not declared in AdarshLang scope.")

    def get_variable(self, name):
        if name in self.variables:
            return self.variables[name]
        else:
            if self.parent:
                return self.parent.get_variable(name)
            else:
                raise AdarshSemanticError(f"Variable '{name}' not declared in AdarshLang scope.")

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
        raise AdarshSemanticError(f"Function '{name}' not declared in AdarshLang scope.")

    def declare_type(self, name, fields):
        self.types[name] = list(fields)

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
        raise AdarshSemanticError(f"Dhacha '{name}' not declared in AdarshLang scope.")
    def get_function_param_count(self, name):
        if name in self.functions:
            return self.functions[name]
        if self.parent:
            return self.parent.get_function_param_count(name)
        raise AdarshSemanticError(f"Function '{name}' not declared in AdarshLang scope.")

class AdarshSemanticAnalyzer:
    def __init__(self):
        self.builtin_functions = {
            'length': AdarshFunctionSignature([AdarshParam('_value')]),
            'push': AdarshFunctionSignature([AdarshParam('_list'), AdarshParam('_value')]),
            'pop': AdarshFunctionSignature([AdarshParam('_list')]),
            'rakho': AdarshFunctionSignature([AdarshParam('_target'), AdarshParam('_key'), AdarshParam('_value')]),
            'nikalo': AdarshFunctionSignature([AdarshParam('_target'), AdarshParam('_key'), AdarshParam('_default', AdarshBoolLiteralNode(False))]),
            'map': AdarshFunctionSignature([AdarshParam('_fn'), AdarshParam('_iterable')]),
            'filter': AdarshFunctionSignature([AdarshParam('_fn'), AdarshParam('_iterable')]),
            'reduce': AdarshFunctionSignature([AdarshParam('_fn'), AdarshParam('_iterable'), AdarshParam('_initial', AdarshBoolLiteralNode(False))]),
            'random_number': AdarshFunctionSignature([AdarshParam('_start', AdarshNumLiteralNode(0)), AdarshParam('_end', AdarshNumLiteralNode(1))]),
            'current_time': AdarshFunctionSignature([]),
            'abs': AdarshFunctionSignature([AdarshParam('_value')]),
            'floor': AdarshFunctionSignature([AdarshParam('_value')]),
            'ceil': AdarshFunctionSignature([AdarshParam('_value')]),
            'upper': AdarshFunctionSignature([AdarshParam('_value')]),
            'lower': AdarshFunctionSignature([AdarshParam('_value')]),
            'join': AdarshFunctionSignature([AdarshParam('_iterable'), AdarshParam('_sep', AdarshStringLiteralNode(''))]),
            'split': AdarshFunctionSignature([AdarshParam('_value'), AdarshParam('_sep', AdarshStringLiteralNode(' '))]),
        }
        self.imported_modules = set()
            'length': 1,
            'push': 2,
            'pop': 1,
        }

    def analyze(self, node, scope=None, loop_depth=0):
        if scope is None:
            scope = AdarshSymbolTable()
            for name, signature in self.builtin_functions.items():
                scope.declare_function(name, signature)
                scope.declare_variable(name)
            for name, count in self.builtin_functions.items():
                scope.declare_function(name, count)

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
        elif isinstance(node, AdarshIndexAssignNode):
            self.analyze(node.collection, scope, loop_depth)
            self.analyze(node.index_expr, scope, loop_depth)
            self.analyze(node.value_expr, scope, loop_depth)
        elif isinstance(node, AdarshAttributeAssignNode):
            self.analyze(node.target, scope, loop_depth)
            self.analyze(node.value_expr, scope, loop_depth)
        elif isinstance(node, AdarshBinOpNode):
            self.analyze(node.left, scope, loop_depth)
            self.analyze(node.right, scope, loop_depth)
        elif isinstance(node, AdarshUnaryOpNode):
            self.analyze(node.factor, scope, loop_depth)
        elif isinstance(node, AdarshNumLiteralNode):
            pass
        elif isinstance(node, AdarshBoolLiteralNode):
            pass
        elif isinstance(node, AdarshStringLiteralNode):
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
                raise AdarshSemanticError(f"Dhacha '{node.type_name}' not declared in AdarshLang.")
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
        elif isinstance(node, AdarshVarReferenceNode):
            scope.get_variable(node.var_name)
        elif isinstance(node, AdarshIndexAccessNode):
            self.analyze(node.collection, scope, loop_depth)
            self.analyze(node.index_expr, scope, loop_depth)
        elif isinstance(node, AdarshDikhaoNode):
            self.analyze(node.expr, scope, loop_depth)
        elif isinstance(node, AdarshBreakNode):
            if loop_depth == 0:
                raise AdarshSemanticError("'bas' (break) can only be used inside loops in AdarshLang.")
        elif isinstance(node, AdarshContinueNode):
            if loop_depth == 0:
                raise AdarshSemanticError("'aage_badho' (continue) can only be used inside loops in AdarshLang.")
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
            for s in node.statements:
                self.analyze(s, scope, loop_depth)
        elif isinstance(node, AdarshKaamDefNode):
            signature = AdarshFunctionSignature(node.params)
            scope.declare_function(node.func_name, signature)
            scope.declare_variable(node.func_name)
            func_scope = AdarshSymbolTable(parent=scope)
            for p in node.params:
                func_scope.declare_variable(p.name)
                func_scope.declare_variable(p)
            self.analyze(node.body, func_scope, 0)
        elif isinstance(node, AdarshWapasNode):
            if node.expr is not None:
                self.analyze(node.expr, scope, loop_depth)
        elif isinstance(node, AdarshFunctionExprNode):
            func_scope = AdarshSymbolTable(parent=scope)
            for p in node.params:
                func_scope.declare_variable(p.name)
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
        elif isinstance(node, AdarshKaamCallNode):
            if not scope.has_function(node.func_name):
                raise AdarshSemanticError(f"Function '{node.func_name}' not declared in AdarshLang.")
            expected_count = scope.get_function_param_count(node.func_name)
            if len(node.args) != expected_count:
                raise AdarshSemanticError(
                    f"Function '{node.func_name}' expects {expected_count} args, got {len(node.args)}."
                )
            for arg in node.args:
                self.analyze(arg, scope, loop_depth)
        else:
            pass

# =====================================================
# =============== INTERPRETER (EXECUTION) =============
# =====================================================

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
        return result

class AdarshBuiltinFunction(AdarshCallable):
    def __init__(self, name, func, signature):
        super().__init__(signature)
        self.name = name
        self.func = func

    def call(self, interpreter, args):
        if not self.signature.allows(len(args)):
            raise AdarshRuntimeError(
                f"Builtin '{self.name}' received invalid argument count {len(args)} in AdarshLang."
            )
        values, provided = interpreter._resolve_arguments(self.signature.params, args, interpreter.global_scope)
        return self.func(interpreter, values, provided)

class AdarshInterpreter:
    def __init__(self):
        self.global_scope = AdarshSymbolTable()
        self.global_scope.types = {}
        self.loaded_modules = set()
        self.builtins = self._build_builtins()
        for name, builtin in self.builtins.items():
            self.global_scope.declare_variable(name)
            self.global_scope.set_variable(name, builtin)

    def _build_builtins(self):
        builtins = {}
        def register(name, func, signature):
            builtins[name] = AdarshBuiltinFunction(name, func, signature)

        register('length', self._builtin_length, AdarshFunctionSignature([AdarshParam('_value')]))
        register('push', self._builtin_push, AdarshFunctionSignature([AdarshParam('_list'), AdarshParam('_value')]))
        register('pop', self._builtin_pop, AdarshFunctionSignature([AdarshParam('_list')]))
        register('rakho', self._builtin_rakho, AdarshFunctionSignature([
            AdarshParam('_target'), AdarshParam('_key'), AdarshParam('_value')
        ]))
        register('nikalo', self._builtin_nikalo, AdarshFunctionSignature([
            AdarshParam('_target'), AdarshParam('_key'), AdarshParam('_default', AdarshBoolLiteralNode(False))
        ]))
        register('map', self._builtin_map, AdarshFunctionSignature([
            AdarshParam('_fn'), AdarshParam('_iterable')
        ]))
        register('filter', self._builtin_filter, AdarshFunctionSignature([
            AdarshParam('_fn'), AdarshParam('_iterable')
        ]))
        register('reduce', self._builtin_reduce, AdarshFunctionSignature([
            AdarshParam('_fn'), AdarshParam('_iterable'), AdarshParam('_initial', AdarshBoolLiteralNode(False))
        ]))
        register('random_number', self._builtin_random_number, AdarshFunctionSignature([
            AdarshParam('_start', AdarshNumLiteralNode(0)), AdarshParam('_end', AdarshNumLiteralNode(1))
        ]))
        register('current_time', self._builtin_current_time, AdarshFunctionSignature([]))
        register('abs', self._builtin_abs, AdarshFunctionSignature([AdarshParam('_value')]))
        register('floor', self._builtin_floor, AdarshFunctionSignature([AdarshParam('_value')]))
        register('ceil', self._builtin_ceil, AdarshFunctionSignature([AdarshParam('_value')]))
        register('upper', self._builtin_upper, AdarshFunctionSignature([AdarshParam('_value')]))
        register('lower', self._builtin_lower, AdarshFunctionSignature([AdarshParam('_value')]))
        register('join', self._builtin_join, AdarshFunctionSignature([
            AdarshParam('_iterable'), AdarshParam('_sep', AdarshStringLiteralNode(''))
        ]))
        register('split', self._builtin_split, AdarshFunctionSignature([
            AdarshParam('_value'), AdarshParam('_sep', AdarshStringLiteralNode(' '))
        ]))
        return builtins
class AdarshInterpreter:
    def __init__(self):
        self.global_scope = AdarshSymbolTable()
        self.function_definitions = {}
        self.builtins = {
            'length': (self._builtin_length, 1),
            'push': (self._builtin_push, 2),
            'pop': (self._builtin_pop, 1),
        }
        for name, (_, arity) in self.builtins.items():
            self.global_scope.declare_function(name, arity)

    def visit(self, node, scope=None):
        if scope is None:
            scope = self.global_scope
        method_name = f"visit_{type(node).__name__}"
        visitor = getattr(self, method_name, self.generic_visit)
        return visitor(node, scope)

    def generic_visit(self, node, scope):
        raise Exception(f"No visit_{type(node).__name__} method defined for AdarshLang.")

    def interpret(self, node):
        return self.visit(node, self.global_scope)

    def visit_AdarshProgramNode(self, node, scope):
        result = None
        for stmt in node.statements:
            result = self.visit(stmt, scope)
        return result

    def visit_BadloNode(self, node, scope):
        scope.declare_variable(node.var_name)
        init_val = None
        if node.init_expr is not None:
            init_val = self.visit(node.init_expr, scope)
        scope.set_variable(node.var_name, init_val)
        return None

    def visit_AdarshAssignNode(self, node, scope):
        value = self.visit(node.expr, scope)
        scope.set_variable(node.var_name, value)
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
        left_val = self.visit(node.left, scope)
        right_val = self.visit(node.right, scope)
        op_type = node.op
        try:
            if op_type == AdarshTokenType.PLUS:
                return left_val + right_val
            elif op_type == AdarshTokenType.MINUS:
                return left_val - right_val
            elif op_type == AdarshTokenType.MUL:
                return left_val * right_val
            elif op_type == AdarshTokenType.DIV:
                return left_val / right_val
            elif op_type == AdarshTokenType.EQ:
                return left_val == right_val
            elif op_type == AdarshTokenType.NEQ:
                return left_val != right_val
            elif op_type == AdarshTokenType.LT:
                return left_val < right_val
            elif op_type == AdarshTokenType.GT:
                return left_val > right_val
            elif op_type == AdarshTokenType.LTE:
                return left_val <= right_val
            elif op_type == AdarshTokenType.GTE:
                return left_val >= right_val
            elif op_type == AdarshTokenType.AUR:
                return bool(left_val) and bool(right_val)
            elif op_type == AdarshTokenType.YA:
                return bool(left_val) or bool(right_val)
        except TypeError as exc:
            raise AdarshRuntimeError(str(exc)) from exc
        raise AdarshRuntimeError(f"Unknown binary operator '{op_type}' in AdarshLang.")

    def visit_AdarshUnaryOpNode(self, node, scope):
        val = self.visit(node.factor, scope)
        op_type = node.op
        if op_type == AdarshTokenType.NAHIN:
            return not val
        elif op_type == AdarshTokenType.MINUS:
            return -val
        raise AdarshRuntimeError(f"Unknown unary operator '{op_type}' in AdarshLang.")

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
            raise AdarshRuntimeError(f"Dhacha '{node.type_name}' not declared before use in AdarshLang.")
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
        raise AdarshRuntimeError("Attribute access only supported on dhacha objects or dictionaries in AdarshLang.")

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
            raise AdarshRuntimeError("Only indexable values like lists, strings, or dicts support indexing in AdarshLang.")
        except IndexError:
            raise AdarshRuntimeError("Index out of range while accessing collection in AdarshLang.")
    def visit_AdarshVarReferenceNode(self, node, scope):
        return scope.get_variable(node.var_name)

    def visit_AdarshIndexAccessNode(self, node, scope):
        collection = self.visit(node.collection, scope)
        index_value = self.visit(node.index_expr, scope)
        index_value = self._normalize_index(index_value)
        try:
            return collection[index_value]
        except TypeError:
            raise RuntimeError("Only indexable values (like lists or strings) support indexing in AdarshLang.")
        except IndexError:
            raise RuntimeError("Index out of range while accessing collection in AdarshLang.")

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
            raise AdarshRuntimeError("Only mutable collections like lists or dicts support indexed assignment in AdarshLang.")
        except IndexError:
            raise AdarshRuntimeError("Index out of range while assigning into collection in AdarshLang.")

        index_value = self._normalize_index(index_value)

        try:
            collection[index_value] = value
        except TypeError:
            raise RuntimeError("Only mutable indexable values like lists support assignment in AdarshLang.")
        except IndexError:
            raise RuntimeError("Index out of range while assigning into collection in AdarshLang.")

        return value

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
            self.visit(node.init_stmt, loop_scope)
        result = None
        while True:
            if node.condition is not None and not self.visit(node.condition, loop_scope):
                break
            body_scope = AdarshSymbolTable(parent=loop_scope)
            result = self.visit(node.block, body_scope)
            if isinstance(result, AdarshReturnSignal):
                return result
            if isinstance(result, AdarshBreakSignal):
                return None
            if isinstance(result, AdarshContinueSignal):
                pass
            if node.update_expr is not None:
                self.visit(node.update_expr, loop_scope)
        return result

    def visit_AdarshForEachNode(self, node, scope):
        iter_var, declare_new = node.iter_var
        iterable = self.visit(node.iterable_expr, scope)
        if not hasattr(iterable, '__iter__'):
            raise AdarshRuntimeError("'ke_liye' expects an iterable value in AdarshLang.")
        loop_scope = AdarshSymbolTable(parent=scope)
        if declare_new:
            loop_scope.declare_variable(iter_var)
        result = None
        for item in iterable:
            if declare_new:
                loop_scope.set_variable(iter_var, item)
            else:
                scope.set_variable(iter_var, item)
            body_scope = AdarshSymbolTable(parent=loop_scope)
            result = self.visit(node.block, body_scope)
            result = self.visit(node.block, AdarshSymbolTable(parent=scope))
            if isinstance(result, AdarshReturnSignal):
                return result
            if isinstance(result, AdarshBreakSignal):
                return None
            if isinstance(result, AdarshContinueSignal):
                continue
        return result

    def visit_AdarshSwitchNode(self, node, scope):
        subject_value = self.visit(node.subject, scope)
        executed = False
        result = None
        for case in node.cases:
            for match in case.match_exprs:
                if self.visit(match, scope) == subject_value:
                    case_scope = AdarshSymbolTable(parent=scope)
                    result = self.visit(case.block, case_scope)
                    if isinstance(result, AdarshBreakSignal):
                        return None
                    if isinstance(result, (AdarshReturnSignal, AdarshContinueSignal)):
                        return result
                    executed = True
                    break
            if executed:
                break
        if not executed and node.default_block:
            result = self.visit(node.default_block, AdarshSymbolTable(parent=scope))
            if isinstance(result, AdarshBreakSignal):
                return None
            if isinstance(result, (AdarshReturnSignal, AdarshContinueSignal)):
                return result
        return result

    def visit_AdarshTryCatchNode(self, node, scope):
        try:
            try_scope = AdarshSymbolTable(parent=scope)
            result = self.visit(node.try_block, try_scope)
            return result
        except (AdarshRuntimeError, AdarshUserException) as exc:
            catch_scope = AdarshSymbolTable(parent=scope)
            catch_scope.declare_variable(node.err_name)
            value = exc.value if isinstance(exc, AdarshUserException) else str(exc)
            catch_scope.set_variable(node.err_name, value)
            return self.visit(node.catch_block, catch_scope)

    def visit_AdarshBlockNode(self, node, scope):
        result = None
        for stmt in node.statements:
            result = self.visit(stmt, scope)
            if isinstance(result, (AdarshReturnSignal, AdarshBreakSignal, AdarshContinueSignal)):
                return result
        return result

    def visit_AdarshBreakNode(self, node, scope):
        return AdarshBreakSignal()

    def visit_AdarshContinueNode(self, node, scope):
        return AdarshContinueSignal()

    def visit_AdarshKaamDefNode(self, node, scope):
        function_value = AdarshFunctionValue(node.func_name, node.params, node.body, scope)
        scope.declare_variable(node.func_name)
        scope.set_variable(node.func_name, function_value)
        return None

    def visit_AdarshDhachaDefNode(self, node, scope):
        scope.declare_type(node.name, node.fields)
        return None

    def visit_AdarshWapasNode(self, node, scope):
        val = None
        if node.expr is not None:
            val = self.visit(node.expr, scope)
        return AdarshReturnSignal(val)

    def visit_AdarshFunctionExprNode(self, node, scope):
        return AdarshFunctionValue(None, node.params, node.body, scope)

    def visit_AdarshCallNode(self, node, scope):
        callee = self.visit(node.callee, scope)
        if not isinstance(callee, AdarshCallable):
            raise AdarshRuntimeError("Only functions or builtins can be called in AdarshLang.")
        args = [self.visit(arg, scope) for arg in node.args]
        return callee.call(self, args)

    def _resolve_arguments(self, params, args, default_scope):
        values = []
        provided_flags = []
        arg_index = 0
        total_args = len(args)
        for param in params:
            if param.is_varargs:
                values.append(list(args[arg_index:]))
                provided_flags.append(True)
                arg_index = total_args
            else:
                if arg_index < total_args:
                    values.append(args[arg_index])
                    provided_flags.append(True)
                    arg_index += 1
                else:
                    if param.default_expr is not None:
                        values.append(self.visit(param.default_expr, default_scope))
                        provided_flags.append(False)
                    else:
                        raise AdarshRuntimeError(
                            f"Missing argument for parameter '{param.name}' in AdarshLang."
                        )
        if arg_index < total_args:
            raise AdarshRuntimeError("Too many arguments passed to function in AdarshLang.")
        return values, provided_flags

    def _builtin_length(self, interpreter, values, provided):
        value = values[0]
        try:
            return len(value)
        except TypeError:
            raise AdarshRuntimeError("Builtin 'length' expects an indexable value like a list or string in AdarshLang.")

    def _builtin_push(self, interpreter, values, provided):
        target, value = values
        if not isinstance(target, list):
            raise AdarshRuntimeError("Builtin 'push' expects the first argument to be a list in AdarshLang.")
        target.append(value)
        return target

    def _builtin_pop(self, interpreter, values, provided):
        target = values[0]
        if not isinstance(target, list):
            raise AdarshRuntimeError("Builtin 'pop' expects the argument to be a list in AdarshLang.")
        if not target:
            raise AdarshRuntimeError("Cannot pop from an empty list in AdarshLang.")
        return target.pop()

    def _builtin_rakho(self, interpreter, values, provided):
        target, key, value = values
        if isinstance(target, dict):
            target[key] = value
            return target
        if isinstance(target, list) and isinstance(key, int):
            index = self._normalize_index(key)
            if index < 0 or index >= len(target):
                raise AdarshRuntimeError("List index out of range for 'rakho' in AdarshLang.")
            target[index] = value
            return target
        raise AdarshRuntimeError("'rakho' expects a dictionary or list target in AdarshLang.")

    def _builtin_nikalo(self, interpreter, values, provided):
        target, key = values[0], values[1]
        has_default = len(provided) > 2 and provided[2]
        default_value = values[2] if has_default else None
        if isinstance(target, dict):
            if key in target:
                return target[key]
            if has_default:
                return default_value
            raise AdarshRuntimeError("Key missing during 'nikalo' without default in AdarshLang.")
        if isinstance(target, (list, str)):
            index = self._normalize_index(key)
            try:
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
                raise AdarshRuntimeError("'reduce' requires at least one item when no initial value is provided in AdarshLang.")
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
                raise AdarshRuntimeError("Index expressions must evaluate to whole numbers in AdarshLang.")
            index_value = int(index_value)
        if not isinstance(index_value, int):
            raise AdarshRuntimeError("Index expressions must evaluate to integers in AdarshLang.")
    def visit_AdarshKaamCallNode(self, node, scope):
        arg_values = [self.visit(arg, scope) for arg in node.args]
        func_node = self.function_definitions.get(node.func_name)
        if func_node:
            func_scope = AdarshSymbolTable(parent=self.global_scope)

            # Declare each parameter before setting
            for param_name, arg_val in zip(func_node.params, arg_values):
                func_scope.declare_variable(param_name)
                func_scope.set_variable(param_name, arg_val)

            result = self.visit(func_node.body, func_scope)
            if isinstance(result, AdarshReturnSignal):
                return result.value
            return result

        builtin = self.builtins.get(node.func_name)
        if builtin:
            func, expected_arity = builtin
            if len(arg_values) != expected_arity:
                raise RuntimeError(
                    f"Builtin '{node.func_name}' expects {expected_arity} arguments but got {len(arg_values)} in AdarshLang."
                )
            return func(arg_values)

        raise RuntimeError(f"Kaam (function) '{node.func_name}' not defined in AdarshLang.")

    def _builtin_length(self, args):
        value = args[0]
        try:
            return len(value)
        except TypeError:
            raise RuntimeError("Builtin 'length' expects an indexable value like a list or string in AdarshLang.")

    def _builtin_push(self, args):
        target, value = args
        if not isinstance(target, list):
            raise RuntimeError("Builtin 'push' expects the first argument to be a list in AdarshLang.")
        target.append(value)
        return target

    def _builtin_pop(self, args):
        target = args[0]
        if not isinstance(target, list):
            raise RuntimeError("Builtin 'pop' expects the argument to be a list in AdarshLang.")
        if not target:
            raise RuntimeError("Cannot pop from an empty list in AdarshLang.")
        return target.pop()

    def _normalize_index(self, index_value):
        if isinstance(index_value, bool):
            raise RuntimeError("Index must be an integer value in AdarshLang.")

        if isinstance(index_value, float):
            if not index_value.is_integer():
                raise RuntimeError("Index expressions must evaluate to whole numbers in AdarshLang.")
            index_value = int(index_value)

        if not isinstance(index_value, int):
            raise RuntimeError("Index expressions must evaluate to integers in AdarshLang.")

        return index_value

# =====================================================
# =============== MAIN: COMPILATION PIPELINE ==========
# =====================================================

def adarshlang_compile_and_run(source_code):

    # 1. Lexical Analysis
    lexer = AdarshLexer(source_code)
    tokens = lexer.tokenize()

    # 2. Parsing
    parser = AdarshParser(tokens)
    ast = parser.parse_program()

    # 3. Semantic Analysis
    sema = AdarshSemanticAnalyzer()
    sema.analyze(ast)

    # 4. Interpretation
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

def main():
    if len(sys.argv) < 2 or sys.argv[1] == '--repl':
        run_repl()
        return

    filename = sys.argv[1]
    with open(filename, 'r', encoding='utf-8') as f:
        source_code = f.read()

    adarshlang_compile_and_run(source_code)

if __name__ == '__main__':
    main()

# def main():
#     print("Enter AdarshLang code. Press Ctrl+D (or Ctrl+Z on Windows) to finish:")
#     import sys
#     source_code = sys.stdin.read()  # read until EOF
#     adarshlang_compile_and_run(source_code)