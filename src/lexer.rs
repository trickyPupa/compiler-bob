use std::collections::HashMap;
use std::sync::LazyLock;

use super::token::{Token, TokenType};

static OPERATOPS_MAP: LazyLock<HashMap<&'static str, TokenType>> = LazyLock::new(|| {
    HashMap::from([
        ("==", TokenType::EQEQ),
        ("!=", TokenType::NEQ),
        ("<=", TokenType::LTEQ),
        (">=", TokenType::GTEQ),
        ("&&", TokenType::AND),
        ("||", TokenType::OR),
        ("+", TokenType::PLUS),
        ("-", TokenType::MINUS),
        ("*", TokenType::STAR),
        ("/", TokenType::SLASH),
        ("=", TokenType::EQ),
        ("<", TokenType::LT),
        (">", TokenType::GT),
        ("!", TokenType::EXCL),
        ("(", TokenType::LPAREN),
        (")", TokenType::RPAREN),
        ("{", TokenType::LBRACE),
        ("}", TokenType::RBRACE),
        (";", TokenType::SEMICOLON),
    ])
});

static KEYWORDS_MAP: LazyLock<HashMap<&'static str, TokenType>> = LazyLock::new(|| {
    HashMap::from([
        ("var", TokenType::VAR),
        ("print", TokenType::PRINT),
        ("if", TokenType::IF),
        ("else", TokenType::ELSE),
        ("while", TokenType::WHILE),
    ])
});

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Lexer {
    input: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(inp: &str) -> Self {
        Self {
            input: inp.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
        }
    }

    fn next_token(&mut self) -> Option<Token> {
        if self.position > self.input.len() {
            // todo refactor conditions
            return None;
        }
        let mut current = self.peek();

        while current.is_whitespace() {
            self.next();
            current = self.peek();
        }

        if self.position == self.input.len() {
            self.position += 1;
            return Some(Token {
                ttype: TokenType::EOF,
                value: "\0".to_string(),
                position: self.position,
                line: self.line,
                column: self.column,
            });
        }
        if self.position > self.input.len() {
            return None;
        }

        if current.is_numeric() {
            Some(self.read_number())
        } else if current.is_alphabetic() {
            Some(self.read_word())
        } else {
            Some(self.read_operator_or_punctuation())
        }
    }

    fn read_number(&mut self) -> Token {
        let start_pos = self.position;
        let start_line = self.line;
        let start_col = self.column;

        while self.peek().is_numeric() {
            self.next();
        }

        let text = self.input[start_pos..self.position].iter().collect();

        Token {
            ttype: TokenType::NUMBER,
            value: text,
            position: start_pos + 1,
            line: start_line,
            column: start_col,
        }
    }

    fn read_word(&mut self) -> Token {
        let start_pos = self.position;
        let start_line = self.line;
        let start_col = self.column;

        while self.peek().is_alphanumeric() {
            self.next();
        }

        let text: String = self.input[start_pos..self.position].iter().collect();
        let ttype = *KEYWORDS_MAP.get(text.as_str()).unwrap_or(&TokenType::ID);

        Token {
            ttype,
            value: text,
            position: start_pos + 1,
            line: start_line,
            column: start_col,
        }
    }

    fn read_operator_or_punctuation(&mut self) -> Token {
        let start_pos = self.position;
        let start_line = self.line;
        let start_col = self.column;

        if self.position + 1 < self.input.len() {
            let two_chars: String = self.input[self.position..self.position + 2]
                .iter()
                .collect();

            if let Some(&ttype) = OPERATOPS_MAP.get(two_chars.as_str()) {
                self.next();
                self.next();
                return Token {
                    ttype,
                    value: two_chars,
                    position: start_pos + 1,
                    line: start_line,
                    column: start_col,
                };
            }
        }

        let one_char: String = self.input[self.position].to_string();
        if let Some(&ttype) = OPERATOPS_MAP.get(one_char.as_str()) {
            self.next();
            return Token {
                ttype,
                value: one_char,
                position: start_pos,
                line: start_line,
                column: start_col,
            };
        }

        let bad_char = self.peek();
        panic!(
            "[Lexer Error] Unexpected character '{}' at Line {}, Column {}",
            bad_char, start_line, start_col
        );
    }

    fn peek(&mut self) -> char {
        if self.position >= self.input.len() {
            '\0'
        } else {
            self.input[self.position]
        }
    }

    fn next(&mut self) -> char {
        if self.position >= self.input.len() {
            return '\0';
        }
        let current = self.input[self.position];
        self.position += 1;

        if current == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }

        current
    }
}

impl Iterator for Lexer {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}
