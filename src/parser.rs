use crate::expression::Expression;
use crate::statement::Statement;
use crate::token::{Token, TokenType};

pub struct Parser<T: Iterator<Item = Token>> {
    tokens: T,
    current: Option<Token>,
    debug_flag: bool,
}

// utils functions
impl<T: Iterator<Item = Token>> Parser<T> {
    fn advance(&mut self) {
        self.current = self.tokens.next();
        if self.debug_flag {
            println!("debug: token {:?}", self.current);
        }
    }

    fn is_at_end(&self) -> bool {
        self.check_type(&[TokenType::EOF])
    }

    fn check_type(&self, ttypes: &[TokenType]) -> bool {
        self.current.is_some() && (ttypes.contains(&self.current.as_ref().unwrap().ttype))
    }

    fn advance_check_type(&mut self, ttypes: &[TokenType]) -> bool {
        self.advance();
        self.check_type(ttypes)
    }

    fn check_type_advance(&mut self, ttype: TokenType) -> bool {
        let res = self.check_type(&[ttype]);
        if res {
            self.advance();
        }
        res
    }

    fn peek_type_or_panic(&mut self, ttype: TokenType, message: &str) -> Token {
        if self.check_type(&[ttype]) {
            let cur = self.current.take();
            return cur.unwrap();
        }

        panic!("{}", self.generate_panic_message(message));
    }

    #[inline]
    fn generate_panic_message(&self, message: &str) -> String {
        format!(
            "[Parser Error] Line {}, Col {}: {message}",
            self.current.as_ref().unwrap().line,
            self.current.as_ref().unwrap().column
        )
    }
}

// parsing expressions
impl<T: Iterator<Item = Token>> Parser<T> {
    fn parse_expression(&mut self) -> Option<Expression> {
        self.parse_assignment()
    }

    // going from lowest priority (assignment) to highest (primarities)
    // 1. Assignment
    fn parse_assignment(&mut self) -> Option<Expression> {
        let expr = self.parse_logical_or();

        let line = self.current.as_ref().unwrap().line; // TODO replace unwrap

        if self.check_type_advance(TokenType::EQ) {
            let value = self.parse_assignment(); // recursion for a = b = 2

            if let Some(Expression::Variable(name)) = expr {
                return Some(Expression::Assign(
                    name,
                    Box::from(value.expect("Right-hand side of assignment is expected")),
                ));
            } else {
                panic!("[Parser Error] Line {line}: Invalid assigment target.");
            }
        }

        expr
    }

    // 2. (||)
    fn parse_logical_or(&mut self) -> Option<Expression> {
        let mut expr = self.parse_logical_and();

        while self.check_type(&[TokenType::OR]) {
            let op = self.current.take();
            self.advance();

            let right = self.parse_logical_and();
            if expr.is_none() && right.is_none() {
                return None;
            }
            expr = Some(Expression::Binary(
                Box::new(expr.unwrap()),
                op.unwrap().ttype,
                Box::new(right.unwrap()),
            ));
        }

        expr
    }

    // 3. (&&)
    fn parse_logical_and(&mut self) -> Option<Expression> {
        let mut expr = self.parse_equality();

        while self.check_type(&[TokenType::AND]) {
            let op = self.current.take();
            self.advance();

            let right = self.parse_equality();
            if expr.is_none() && right.is_none() {
                return None;
            }
            expr = Some(Expression::Binary(
                Box::new(expr.unwrap()),
                op.unwrap().ttype,
                Box::new(right.unwrap()),
            ));
        }

        expr
    }

    // 4. (==, !=)
    fn parse_equality(&mut self) -> Option<Expression> {
        let mut expr = self.parse_comparison();

        while self.check_type(&[TokenType::EQEQ, TokenType::NEQ]) {
            let op = self.current.take();
            self.advance();

            let right = self.parse_comparison();
            if expr.is_none() && right.is_none() {
                return None;
            }
            expr = Some(Expression::Binary(
                Box::new(expr.unwrap()),
                op.unwrap().ttype,
                Box::new(right.unwrap()),
            ));
        }

        expr
    }

    // 5. (>, <, >=, <=)
    fn parse_comparison(&mut self) -> Option<Expression> {
        let mut expr = self.parse_term();

        while self.check_type(&[
            TokenType::LT,
            TokenType::GT,
            TokenType::GTEQ,
            TokenType::LTEQ,
        ]) {
            let op = self.current.take();
            self.advance();

            let right = self.parse_term();
            if expr.is_none() && right.is_none() {
                return None;
            }
            expr = Some(Expression::Binary(
                Box::new(expr.unwrap()),
                op.unwrap().ttype,
                Box::new(right.unwrap()),
            ));
        }

        expr
    }

    // 6. (+, -)
    fn parse_term(&mut self) -> Option<Expression> {
        let mut expr = self.parse_factor();

        while self.check_type(&[TokenType::PLUS, TokenType::MINUS]) {
            let op = self.current.take();
            self.advance();

            let right = self.parse_factor();
            if expr.is_none() && right.is_none() {
                return None;
            }
            expr = Some(Expression::Binary(
                Box::new(expr.unwrap()),
                op.unwrap().ttype,
                Box::new(right.unwrap()),
            ));
        }

        expr
    }

    // 7. (*, /)
    fn parse_factor(&mut self) -> Option<Expression> {
        let mut expr = self.parse_unary();
        self.advance();

        while self.check_type(&[TokenType::STAR, TokenType::SLASH]) {
            let op = self.current.take();
            self.advance();

            let right = self.parse_unary();
            self.advance();
            if expr.is_none() && right.is_none() {
                return None;
            }
            expr = Some(Expression::Binary(
                Box::new(expr.unwrap()),
                op.unwrap().ttype,
                Box::new(right.unwrap()),
            ));
        }

        expr
    }

    // 8. (!, -)
    fn parse_unary(&mut self) -> Option<Expression> {
        if self.check_type(&[TokenType::EXCL, TokenType::MINUS]) {
            let op = self.current.take();
            self.advance();

            let expr = self.parse_unary();

            return expr.map(|token| Expression::Unary(op.unwrap().ttype, Box::new(token)));
        }

        self.parse_primary()
    }

    // 9. numbers, variables, brackets
    fn parse_primary(&mut self) -> Option<Expression> {
        match self.current.as_ref() {
            Some(token) => match token.ttype {
                TokenType::NUMBER => {
                    let double: f64 = token.value.parse().unwrap_or_else(|_| {
                        panic!("{}", self.generate_panic_message("Invalid number format."))
                    });
                    Some(Expression::Number(double))
                }
                TokenType::ID => Some(Expression::Variable(token.value.clone())),
                TokenType::LPAREN => {
                    self.advance();
                    let expr = self.parse_expression();
                    self.peek_type_or_panic(
                        TokenType::RPAREN,
                        "\")\" is required after an expression.",
                    );
                    expr
                }
                _ => panic!("{}", self.generate_panic_message("Expression is expected.")),
            },
            None => None,
        }
    }
}

// main block. parsing statements
impl<T: Iterator<Item = Token>> Parser<T> {
    fn next_statement(&mut self) -> Option<Statement> {
        self.advance();

        if self.is_at_end() || self.current.is_none() {
            return None;
        }

        let res = self
            .parse_declaration()
            .or_else(|| self.parse_statement())
            .or_else(|| self.parse_expression_statement());

        if self.debug_flag {
            println!("debug: statement {:?}", res);
        }

        res
    }

    fn parse_declaration(&mut self) -> Option<Statement> {
        if self.check_type_advance(TokenType::VAR) {
            self.parse_var_declaration()
        } else {
            None
        }
    }

    fn parse_var_declaration(&mut self) -> Option<Statement> {
        let name = self
            .peek_type_or_panic(TokenType::ID, "Variable name is required")
            .value;
        self.advance();
        let mut initializer = None;

        if self.check_type_advance(TokenType::EQ) {
            initializer = self.parse_expression();
        }

        self.peek_type_or_panic(
            TokenType::SEMICOLON,
            "\";\" is required after variable declaration",
        );
        Some(Statement::Var(name, initializer))
    }

    fn parse_statement(&mut self) -> Option<Statement> {
        let ttype = self.current.as_ref().map(|token| token.ttype)?;

        match ttype {
            TokenType::IF => {
                self.advance();
                self.parse_if_statement()
            }
            TokenType::PRINT => {
                self.advance();
                self.parse_print_statement()
            }
            TokenType::WHILE => {
                self.advance();
                self.parse_while_statement()
            }
            TokenType::LBRACE => self.parse_block(),
            _ => None,
        }
    }

    fn parse_expression_statement(&mut self) -> Option<Statement> {
        let expr = self.parse_expression();
        self.peek_type_or_panic(TokenType::SEMICOLON, "\";\" is required after expression.");
        expr.map(Statement::Expression)
    }

    fn parse_if_statement(&mut self) -> Option<Statement> {
        self.peek_type_or_panic(TokenType::LPAREN, "\"(\" is required after \"if\".");
        self.advance();

        let condition = self.parse_expression().unwrap(); // todo

        self.peek_type_or_panic(
            TokenType::RPAREN,
            "\")\" is required after \"if\" condition.",
        );
        self.advance();

        let then_branch = self
            .parse_statement()
            .unwrap_or_else(|| panic!("{}", self.generate_panic_message("if")));
        self.advance();
        let else_branch = if self.check_type_advance(TokenType::ELSE) {
            self.parse_statement()
        } else {
            None
        };

        Some(Statement::If(
            condition,
            Box::new(then_branch),
            else_branch.map(Box::new),
        ))
    }

    fn parse_while_statement(&mut self) -> Option<Statement> {
        self.peek_type_or_panic(TokenType::LPAREN, "\"(\" is required after \"while\".");
        self.advance();

        let condition = self.parse_expression().unwrap(); // todo

        self.peek_type_or_panic(
            TokenType::RPAREN,
            "\")\" is required after \"while\" condition.",
        );
        self.advance();

        let body = self.parse_statement().unwrap_or_else(|| {
            panic!(
                "{}",
                self.generate_panic_message("while block can't be empty")
            )
        });
        Some(Statement::While(condition, Box::new(body)))
    }

    fn parse_print_statement(&mut self) -> Option<Statement> {
        let value = self.parse_expression().unwrap_or_else(|| {
            panic!(
                "{}",
                self.generate_panic_message("print block can't be empty")
            )
        });
        self.peek_type_or_panic(TokenType::SEMICOLON, "\";\" is required after value");

        Some(Statement::Print(value))
    }

    fn parse_block(&mut self) -> Option<Statement> {
        let mut statements: Vec<Statement> = Vec::new();

        while !self.is_at_end() && !self.advance_check_type(&[TokenType::RBRACE]) {
            let st = self
                .parse_declaration()
                .or_else(|| self.parse_statement())
                .or_else(|| self.parse_expression_statement());

            if st.is_none() {
                break;
            }
            statements.push(st.unwrap());
        }

        if statements.is_empty() {
            panic!("{}", self.generate_panic_message("block can't be empty"))
        }

        Some(Statement::Block(statements))
    }
}

// public functions
impl<T: Iterator<Item = Token>> Parser<T> {
    pub fn new(lexer: T) -> Self {
        Parser {
            tokens: lexer,
            current: None,
            debug_flag: false,
        }
    }

    pub fn new_debug(lexer: T) -> Self {
        Parser {
            tokens: lexer,
            current: None,
            debug_flag: true,
        }
    }
}

impl<T: Iterator<Item = Token>> Iterator for Parser<T> {
    type Item = Statement;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_statement()
    }
}
