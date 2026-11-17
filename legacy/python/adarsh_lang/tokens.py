"""Token definitions for AdarshLang."""


class AdarshTokenType:
    EOF = 'EOF'
    IDENT = 'IDENT'
    NUMBER = 'NUMBER'
    STRING = 'STRING'

    # Keywords
    BADLO = 'BADLO'        # badlo
    KAAM = 'KAAM'          # kaam
    WAPAS = 'WAPAS'        # wapas
    AGAR = 'AGAR'          # agar
    WARNA = 'WARNA'        # warna
    JABTAK = 'JABTAK'      # jabtak
    DIKHAO = 'DIKHAO'      # dikhao
    BAS = 'BAS'            # bas
    AAGE_BADHO = 'AAGE_BADHO'  # aage_badho
    SAHI_HAI_BE = 'SAHI_HAI_BE'  # sahi_hai_be
    JHUTH = 'JHUTH'        # jhuth
    GINNATI = 'GINNATI'    # ginnati
    KE_LIYE = 'KE_LIYE'    # ke_liye
    IN = 'IN'              # in
    CHUNO = 'CHUNO'        # chuno
    CASE = 'CASE'          # case
    WARNA_CASE = 'WARNA_CASE'  # warna_case
    PAKDO = 'PAKDO'        # pakdo
    CHHODDO = 'CHHODDO'    # chhoddo
    LAO = 'LAO'            # lao
    DHACHA = 'DHACHA'      # dhacha
    BAAKI = 'BAAKI'        # baaki (varargs marker)
    KHALI = 'KHALI'        # khali (null)

    # Operators
    PLUS = '+'
    MINUS = '-'
    MUL = '*'
    DIV = '/'
    MOD = '%'
    POWER = '**'
    ASSIGN = '='
    PLUS_ASSIGN = '+='
    MINUS_ASSIGN = '-='
    MUL_ASSIGN = '*='
    DIV_ASSIGN = '/='
    MOD_ASSIGN = '%='
    EQ = '=='
    NEQ = '!='
    LT = '<'
    GT = '>'
    LTE = '<='
    GTE = '>='
    AUR = '&&'   # aur
    YA = '||'    # ya
    NAHIN = '!'  # nahin
    SEMI = ';'
    COMMA = ','
    COLON = ':'
    DOT = '.'
    LPAREN = '('
    RPAREN = ')'
    LBRACE = '{'
    RBRACE = '}'
    LBRACKET = '['
    RBRACKET = ']'


HINGLISH_KEYWORDS = {
    'badlo': AdarshTokenType.BADLO,
    'kaam': AdarshTokenType.KAAM,
    'wapas': AdarshTokenType.WAPAS,
    'agar': AdarshTokenType.AGAR,
    'warna': AdarshTokenType.WARNA,
    'jabtak': AdarshTokenType.JABTAK,
    'dikhao': AdarshTokenType.DIKHAO,
    'bas': AdarshTokenType.BAS,
    'aage_badho': AdarshTokenType.AAGE_BADHO,
    'sahi_hai_be': AdarshTokenType.SAHI_HAI_BE,
    'jhuth': AdarshTokenType.JHUTH,
    'ginnati': AdarshTokenType.GINNATI,
    'ke_liye': AdarshTokenType.KE_LIYE,
    'in': AdarshTokenType.IN,
    'chuno': AdarshTokenType.CHUNO,
    'case': AdarshTokenType.CASE,
    'warna_case': AdarshTokenType.WARNA_CASE,
    'pakdo': AdarshTokenType.PAKDO,
    'chhoddo': AdarshTokenType.CHHODDO,
    'lao': AdarshTokenType.LAO,
    'dhacha': AdarshTokenType.DHACHA,
    'baaki': AdarshTokenType.BAAKI,
    'khali': AdarshTokenType.KHALI,
}


HINGLISH_OPERATORS = {
    '+': AdarshTokenType.PLUS,
    '-': AdarshTokenType.MINUS,
    '*': AdarshTokenType.MUL,
    '/': AdarshTokenType.DIV,
    '%': AdarshTokenType.MOD,
    '=': AdarshTokenType.ASSIGN,
    ';': AdarshTokenType.SEMI,
    ',': AdarshTokenType.COMMA,
    ':': AdarshTokenType.COLON,
    '.': AdarshTokenType.DOT,
    '(': AdarshTokenType.LPAREN,
    ')': AdarshTokenType.RPAREN,
    '{': AdarshTokenType.LBRACE,
    '}': AdarshTokenType.RBRACE,
    '[': AdarshTokenType.LBRACKET,
    ']': AdarshTokenType.RBRACKET,
    '!': AdarshTokenType.NAHIN,
}


HINGLISH_MULTI_OPERATORS = {
    '**': AdarshTokenType.POWER,
    '+=': AdarshTokenType.PLUS_ASSIGN,
    '-=': AdarshTokenType.MINUS_ASSIGN,
    '*=': AdarshTokenType.MUL_ASSIGN,
    '/=': AdarshTokenType.DIV_ASSIGN,
    '%=': AdarshTokenType.MOD_ASSIGN,
    '==': AdarshTokenType.EQ,
    '!=': AdarshTokenType.NEQ,
    '<=': AdarshTokenType.LTE,
    '>=': AdarshTokenType.GTE,
    '&&': AdarshTokenType.AUR,
    '||': AdarshTokenType.YA,
    '<': AdarshTokenType.LT,
    '>': AdarshTokenType.GT,
}

__all__ = [
    'AdarshTokenType',
    'HINGLISH_KEYWORDS',
    'HINGLISH_OPERATORS',
    'HINGLISH_MULTI_OPERATORS',
]
