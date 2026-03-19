#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenType {
    NUMBER,
    ID,
    STRING,
    VAR,

    PRINT,
    IF,
    ELSE,
    WHILE, // while

    // Operators
    PLUS,  // +
    MINUS, // -
    STAR,  // *
    SLASH, // /
    EQ,    // =
    EQEQ,  // ==
    EXCL,  // !
    NEQ,   // !=
    LT,    // <
    GT,    // >
    LTEQ,  // <=
    GTEQ,  // >=
    AND,   // &&
    OR,    // ||

    // Grouping & Punctuation
    LPAREN,    // (
    RPAREN,    // )
    LBRACE,    // {
    RBRACE,    // }
    SEMICOLON, // ;

    EOF,
}

#[derive(Debug)]
pub struct Token {
    pub ttype: TokenType,
    pub value: String,
    pub position: usize,
    pub line: usize,
    pub column: usize,
}
