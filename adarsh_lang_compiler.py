import sys
import re

# =====================================================
# =============== LEXICAL ANALYSIS ====================
# =====================================================

class AdarshTokenType:
    EOF         = 'EOF'
    IDENT       = 'IDENT'
    NUMBER      = 'NUMBER'
    STRING      = 'STRING'

    # Keywords
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
}

HINGLISH_OPERATORS = {
    '+':  AdarshTokenType.PLUS,
    '-':  AdarshTokenType.MINUS,
    '*':  AdarshTokenType.MUL,
    '/':  AdarshTokenType.DIV,
    '=':  AdarshTokenType.ASSIGN,
    ';':  AdarshTokenType.SEMI,
    ',':  AdarshTokenType.COMMA,
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

class AdarshIndexAssignNode(AdarshASTNode):
    def __init__(self, collection, index_expr, value_expr):
        self.collection = collection
        self.index_expr = index_expr
        self.value_expr = value_expr

class AdarshVarReferenceNode(AdarshASTNode):
    def __init__(self, var_name):
        self.var_name = var_name

class AdarshIndexAccessNode(AdarshASTNode):
    def __init__(self, collection, index_expr):
        self.collection = collection
        self.index_expr = index_expr

class AdarshDikhaoNode(AdarshASTNode):
    def __init__(self, expr):
        self.expr = expr

class AdarshBreakNode(AdarshASTNode):
    pass

class AdarshContinueNode(AdarshASTNode):
    pass

class AdarshIfNode(AdarshASTNode):
    def __init__(self, condition, if_block, else_block=None):
        self.condition = condition
        self.if_block = if_block
        self.else_block = else_block

class AdarshWhileNode(AdarshASTNode):
    def __init__(self, condition, block):
        self.condition = condition
        self.block = block

class AdarshBlockNode(AdarshASTNode):
    def __init__(self, statements):
        self.statements = statements

class AdarshKaamDefNode(AdarshASTNode):
    def __init__(self, func_name, params, body):
        self.func_name = func_name
        self.params = params
        self.body = body

class AdarshWapasNode(AdarshASTNode):
    def __init__(self, expr):
        self.expr = expr

class AdarshKaamCallNode(AdarshASTNode):
    def __init__(self, func_name, args):
        self.func_name = func_name
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
        if self.current_token.type == AdarshTokenType.IDENT:
            params = self.parse_param_list()
        self.eat(AdarshTokenType.RPAREN)
        body = self.parse_block()
        return AdarshKaamDefNode(func_name, params, body)

    def parse_param_list(self):
        params = [self.current_token.value]
        self.eat(AdarshTokenType.IDENT)
        while self.current_token.type == AdarshTokenType.COMMA:
            self.eat(AdarshTokenType.COMMA)
            params.append(self.current_token.value)
            self.eat(AdarshTokenType.IDENT)
        return params

    def parse_statement(self):
        if self.current_token.type == AdarshTokenType.BADLO:
            return self.parse_badlo_decl()
        elif self.current_token.type == AdarshTokenType.AGAR:
            return self.parse_agar()
        elif self.current_token.type == AdarshTokenType.JABTAK:
            return self.parse_jabtak()
        elif self.current_token.type == AdarshTokenType.DIKHAO:
            return self.parse_dikhao()
        elif self.current_token.type == AdarshTokenType.BAS:
            return self.parse_bas()
        elif self.current_token.type == AdarshTokenType.AAGE_BADHO:
            return self.parse_aage_badho()
        elif self.current_token.type == AdarshTokenType.WAPAS:
            return self.parse_wapas()
        elif self.current_token.type == AdarshTokenType.LBRACE:
            return self.parse_block()
        else:
            expr = self.parse_expression()
            if self.current_token.type == AdarshTokenType.ASSIGN:
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

    def parse_dikhao(self):
        self.eat(AdarshTokenType.DIKHAO)
        self.eat(AdarshTokenType.LPAREN)
        expr = self.parse_expression()
        self.eat(AdarshTokenType.RPAREN)
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
            return self.parse_kaam_call_or_var_or_literal()

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
        elif token.type == AdarshTokenType.LPAREN:
            self.eat(AdarshTokenType.LPAREN)
            node = self.parse_expression()
            self.eat(AdarshTokenType.RPAREN)
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

    def declare_function(self, name, param_count):
        self.functions[name] = param_count

    def has_function(self, name):
        if name in self.functions:
            return True
        if self.parent:
            return self.parent.has_function(name)
        return False

    def get_function_param_count(self, name):
        if name in self.functions:
            return self.functions[name]
        if self.parent:
            return self.parent.get_function_param_count(name)
        raise AdarshSemanticError(f"Function '{name}' not declared in AdarshLang scope.")

class AdarshSemanticAnalyzer:
    def __init__(self):
        self.builtin_functions = {
            'length': 1,
            'push': 2,
            'pop': 1,
        }

    def analyze(self, node, scope=None, loop_depth=0):
        if scope is None:
            scope = AdarshSymbolTable()
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
        elif isinstance(node, AdarshIfNode):
            self.analyze(node.condition, scope, loop_depth)
            self.analyze(node.if_block, AdarshSymbolTable(parent=scope), loop_depth)
            if node.else_block:
                self.analyze(node.else_block, AdarshSymbolTable(parent=scope), loop_depth)
        elif isinstance(node, AdarshWhileNode):
            self.analyze(node.condition, scope, loop_depth)
            self.analyze(node.block, AdarshSymbolTable(parent=scope), loop_depth + 1)
        elif isinstance(node, AdarshBlockNode):
            for s in node.statements:
                self.analyze(s, scope, loop_depth)
        elif isinstance(node, AdarshKaamDefNode):
            scope.declare_function(node.func_name, len(node.params))
            func_scope = AdarshSymbolTable(parent=scope)
            for p in node.params:
                func_scope.declare_variable(p)
            self.analyze(node.body, func_scope, 0)
        elif isinstance(node, AdarshWapasNode):
            if node.expr is not None:
                self.analyze(node.expr, scope, loop_depth)
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

class AdarshReturnSignal:
    def __init__(self, value):
        self.value = value

class AdarshBreakSignal:
    pass

class AdarshContinueSignal:
    pass

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

    def visit_AdarshBinOpNode(self, node, scope):
        left_val = self.visit(node.left, scope)
        right_val = self.visit(node.right, scope)
        op_type = node.op
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
        else:
            raise RuntimeError(f"Unknown binary operator '{op_type}' in AdarshLang.")

    def visit_AdarshUnaryOpNode(self, node, scope):
        val = self.visit(node.factor, scope)
        op_type = node.op
        if op_type == AdarshTokenType.NAHIN:
            return not val
        elif op_type == AdarshTokenType.MINUS:
            return -val
        else:
            raise RuntimeError(f"Unknown unary operator '{op_type}' in AdarshLang.")

    def visit_AdarshNumLiteralNode(self, node, scope):
        return node.value

    def visit_AdarshBoolLiteralNode(self, node, scope):
        return node.value

    def visit_AdarshStringLiteralNode(self, node, scope):
        return node.value

    def visit_AdarshListLiteralNode(self, node, scope):
        return [self.visit(element, scope) for element in node.elements]

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

    def visit_AdarshIfNode(self, node, scope):
        cond_val = self.visit(node.condition, scope)
        if cond_val:
            return self.visit(node.if_block, AdarshSymbolTable(parent=scope))
        else:
            if node.else_block:
                return self.visit(node.else_block, AdarshSymbolTable(parent=scope))
        return None

    def visit_AdarshWhileNode(self, node, scope):
        result = None
        while True:
            cond_val = self.visit(node.condition, scope)
            if not cond_val:
                break
            result = self.visit(node.block, AdarshSymbolTable(parent=scope))
            if isinstance(result, AdarshReturnSignal):
                return result
            if isinstance(result, AdarshBreakSignal):
                return None
            if isinstance(result, AdarshContinueSignal):
                continue
        return result

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
        self.function_definitions[node.func_name] = node
        scope.declare_function(node.func_name, len(node.params))
        return None

    def visit_AdarshWapasNode(self, node, scope):
        val = None
        if node.expr is not None:
            val = self.visit(node.expr, scope)
        return AdarshReturnSignal(val)

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

    #print("aaaaaaaaaaaaaaaeeeeeeeeeeeittttt pura coderun ho rha")

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

def main():
    if len(sys.argv) < 2:
        print("Usage: python adarsh_lang_compiler.py <source_file>")
        sys.exit(1)

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