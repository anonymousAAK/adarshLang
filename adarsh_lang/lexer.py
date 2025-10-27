"""Lexical analysis for AdarshLang."""

from .tokens import (
    AdarshTokenType,
    HINGLISH_KEYWORDS,
    HINGLISH_OPERATORS,
    HINGLISH_MULTI_OPERATORS,
)


class AdarshToken:
    def __init__(self, ttype, value, line, column):
        self.type = ttype
        self.value = value
        self.line = line
        self.column = column

    def __repr__(self):
        return (
            f"AdarshToken({self.type}, {self.value}, line={self.line}, col={self.column})"
        )


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

        while self.current_char is not None and (
            self.current_char.isdigit() or self.current_char == '.'
        ):
            if self.current_char == '.':
                dot_count += 1
                if dot_count > 1:
                    break
            number_str += self.current_char
            self.advance()
        if '.' in number_str:
            return AdarshToken(
                AdarshTokenType.NUMBER, float(number_str), start_line, start_col
            )
        else:
            return AdarshToken(
                AdarshTokenType.NUMBER, int(number_str), start_line, start_col
            )

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
        while (
            self.current_char is not None
            and (self.current_char.isalnum() or self.current_char == '_')
        ):
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
                return AdarshToken(
                    HINGLISH_MULTI_OPERATORS[potential_op],
                    potential_op,
                    start_line,
                    start_col,
                )
        self.advance()
        if op in HINGLISH_MULTI_OPERATORS:
            return AdarshToken(HINGLISH_MULTI_OPERATORS[op], op, start_line, start_col)
        if op in HINGLISH_OPERATORS:
            return AdarshToken(HINGLISH_OPERATORS[op], op, start_line, start_col)
        raise RuntimeError(
            f"Unexpected character '{op}' at line {start_line}, col {start_col}"
        )

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
            if self.current_char in HINGLISH_OPERATORS or self.current_char in HINGLISH_MULTI_OPERATORS:
                return self.handle_multi_char_operator()
            raise RuntimeError(
                f"Unexpected character '{self.current_char}' at line {self.line}, col {self.column}"
            )
        return AdarshToken(AdarshTokenType.EOF, None, self.line, self.column)

    def tokenize(self):
        tokens = []
        while True:
            tok = self.get_next_token()
            tokens.append(tok)
            if tok.type == AdarshTokenType.EOF:
                break
        return tokens


__all__ = ['AdarshToken', 'AdarshLexer']
