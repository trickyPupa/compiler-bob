use crate::{
    expression::Expression, statement::Statement, token::{Token, TokenType}
};

pub struct Parser<T: Iterator<Item = Token>> {
    tokens: T,
    current: Option<Token>,
}

impl<T: Iterator<Item = Token>> Parser<T> {
    pub fn new(lexer: T) -> Self {
        Parser {
            tokens: lexer,
            current: None,
        }
    }

    fn next_statement(&mut self) -> Option<Statement> {
        self.advance();

        if self.is_at_end() {
            return None;
        }

        self.parse_declaration()
            .or_else(|| self.parse_statement())
            .or_else(|| self.parse_expression_statement())
    }

    fn parse_declaration(&mut self) -> Option<Statement> {
        if self.check_type_advance(TokenType::VAR) {
            self.parse_var_declaration()
        } else {
            None
        }
    }

    fn parse_var_declaration(&mut self) -> Option<Statement> {
        let name = self.peek_type_or_panic(TokenType::ID, "Required variable name").value;
        let mut initializer = None;

        if self.check_type_advance(TokenType::EQ) {
            initializer = self.parse_expression();
        }

        self.peek_type_or_panic(TokenType::SEMICOLON, "Required \";\" after variable declaration");
        Some(Statement::Var(name, initializer))
    }

    fn parse_statement(&mut self) -> Option<Statement> {
        match self.current.as_ref() {
            Some(token) => match token.ttype {
                TokenType::IF => self.parse_if_statement(),
                TokenType::PRINT => self.parse_print_statement(),
                TokenType::WHILE => self.parse_while_statement(),
                TokenType::LBRACE => self.parse_block(),
                _ => None,
            },
            None => None,
        }
    }

    fn parse_expression_statement(&mut self) -> Option<Statement> {
        None
    }

    fn parse_if_statement(&mut self) -> Option<Statement> {
        None
    }

    fn parse_while_statement(&mut self) -> Option<Statement> {
        None
    }

    fn parse_print_statement(&mut self) -> Option<Statement> {
        None
    }

    fn parse_block(&mut self) -> Option<Statement> {
        None
    }

    fn parse_expression(&mut self) -> Option<Expression> {
        None
    }

    fn advance(&mut self) {
        self.current = self.tokens.next();
    }

    fn is_at_end(&self) -> bool {
        self.check_type(TokenType::EOF)
    }

    fn check_type(&self, ttype: TokenType) -> bool {
        self.current.is_none() || (self.current.as_ref().unwrap().ttype == ttype)
    }

    fn check_type_advance(&mut self, ttype: TokenType) -> bool {
        let res = self.check_type(ttype);
        if res {
            self.advance();
        }
        res
    }

    fn peek_type_or_panic(&mut self, ttype: TokenType, message: &str) -> Token {
        if self.check_type(ttype) {
            let cur = self.current.take();
            self.advance();
            return cur.unwrap();
        }

        panic!(
            "[Parser Error] Line {}, Col {}: {message}",
            self.current.as_ref().unwrap().line,
            self.current.as_ref().unwrap().column
        )
    }
}

impl<T: Iterator<Item = Token>> Iterator for Parser<T> {
    type Item = Statement;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_statement()
    }
}
