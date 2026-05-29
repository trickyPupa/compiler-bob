use compiler::code_generator::generate_random_program;
use compiler::expression::Expression;
use compiler::lexer::Lexer;
use compiler::parser::Parser;
use compiler::statement::{
    BlockStatement, ExpressionStatement, FunctionStatement, IfStatement, PrintStatement,
    ReturnStatement, Statement, VarStatement,
};
use compiler::token::TokenType;

fn get_parser(src: &str) -> Parser<Lexer> {
    let lexer = Lexer::new(src);
    Parser::new_debug(lexer)
}

#[test]
fn parse_print_statement() {
    let mut parser = get_parser("print 1;");

    let goal = Some(Statement::Print(PrintStatement {
        expression: Expression::Number(1.0f64),
        line: 0,
        column: 0,
    }));

    assert_eq!(parser.next(), goal);
}

#[test]
fn parse_complex_print_statement() {
    let mut parser = get_parser("print 1 + 2;");

    let goal = Some(Statement::Print(PrintStatement {
        expression: Expression::Binary(
            Box::new(Expression::Number(1.0f64)),
            TokenType::PLUS,
            Box::new(Expression::Number(2.0f64)),
        ),
        line: 0,
        column: 0,
    }));

    assert_eq!(parser.next(), goal);
}

#[test]
fn parse_var_statement() {
    let mut parser = get_parser("var x = 1;");

    let goal = Some(Statement::Var(VarStatement {
        name: String::from("x"),
        initializer: Some(Expression::Number(1.0f64)),
        line: 0,
        column: 0,
    }));

    assert_eq!(parser.next(), goal);
}

#[test]
fn parse_assignment() {
    let mut parser = get_parser("x = 1;");

    let goal = Some(Statement::Expression(ExpressionStatement {
        expression: Expression::Assign(String::from("x"), Box::new(Expression::Number(1f64))),
        line: 0,
        column: 0,
    }));

    assert_eq!(parser.next(), goal);
}

#[test]
fn parse_simple_combo() {
    let mut parser = get_parser("var x = 2;x = 1;print x;");

    let s1 = Some(Statement::Var(VarStatement {
        name: String::from("x"),
        initializer: Some(Expression::Number(2.0f64)),
        line: 0,
        column: 0,
    }));
    let s2 = Some(Statement::Expression(ExpressionStatement {
        expression: Expression::Assign(String::from("x"), Box::new(Expression::Number(1f64))),
        line: 0,
        column: 0,
    }));
    let s3 = Some(Statement::Print(PrintStatement {
        expression: Expression::Variable(String::from("x")),
        line: 0,
        column: 0,
    }));

    let mut goal = Vec::new();
    goal.push(s1);
    goal.push(s2);
    goal.push(s3);

    for s in goal {
        assert_eq!(parser.next(), s);
    }
}

#[test]
fn parse_simple_expression_statement() {
    let mut parser = get_parser("1 < 4;");

    let goal = Some(Statement::Expression(ExpressionStatement {
        expression: Expression::Binary(
            Box::new(Expression::Number(1.0f64)),
            TokenType::LT,
            Box::new(Expression::Number(4.0f64)),
        ),
        line: 0,
        column: 0,
    }));

    assert_eq!(parser.next(), goal);
}

#[test]
fn parse_array_literal() {
    let mut parser = get_parser("var xs = [1, 2, 3];");

    let goal = Some(Statement::Var(VarStatement {
        name: String::from("xs"),
        initializer: Some(Expression::ArrayLiteral(vec![
            Expression::Number(1.0f64),
            Expression::Number(2.0f64),
            Expression::Number(3.0f64),
        ])),
        line: 0,
        column: 0,
    }));

    assert_eq!(parser.next(), goal);
}

#[test]
fn parse_array_index_expression_statement() {
    let mut parser = get_parser("xs[1];");

    let goal = Some(Statement::Expression(ExpressionStatement {
        expression: Expression::Index(
            Box::new(Expression::Variable(String::from("xs"))),
            Box::new(Expression::Number(1.0f64)),
        ),
        line: 0,
        column: 0,
    }));

    assert_eq!(parser.next(), goal);
}

#[test]
fn parse_array_index_assignment() {
    let mut parser = get_parser("xs[1] = 10;");

    let goal = Some(Statement::Expression(ExpressionStatement {
        expression: Expression::AssignIndex(
            Box::new(Expression::Variable(String::from("xs"))),
            Box::new(Expression::Number(1.0f64)),
            Box::new(Expression::Number(10.0f64)),
        ),
        line: 0,
        column: 0,
    }));

    assert_eq!(parser.next(), goal);
}

#[test]
fn parse_complex_expression_statement() {
    let mut parser = get_parser("1 + 2 < 4 && 23 == 24;");

    let left = Expression::Binary(
        Box::new(Expression::Binary(
            Box::new(Expression::Number(1f64)),
            TokenType::PLUS,
            Box::new(Expression::Number(2f64)),
        )),
        TokenType::LT,
        Box::new(Expression::Number(4f64)),
    );
    let right = Expression::Binary(
        Box::new(Expression::Number(23f64)),
        TokenType::EQEQ,
        Box::new(Expression::Number(24f64)),
    );
    let goal = Some(Statement::Expression(ExpressionStatement {
        expression: Expression::Binary(Box::new(left), TokenType::AND, Box::new(right)),
        line: 0,
        column: 0,
    }));

    assert_eq!(parser.next(), goal);
}

#[test]
fn parse_block_statement() {
    let source = "{print 1;var x = 2;}";
    let mut parser = get_parser(source);

    let goal = Some(Statement::Block(BlockStatement {
        statements: vec![
            Statement::Print(PrintStatement {
                expression: Expression::Number(1.0f64),
                line: 0,
                column: 0,
            }),
            Statement::Var(VarStatement {
                name: String::from("x"),
                initializer: Some(Expression::Number(2.0f64)),
                line: 0,
                column: 0,
            }),
        ],
        line: 0,
        column: 0,
    }));

    assert_eq!(parser.next(), goal);
}

#[test]
fn parse_if_statement() {
    let source = "if (1 > 0) {print 1;}";
    let mut parser = get_parser(source);

    let goal = Some(Statement::If(IfStatement {
        condition: Expression::Binary(
            Box::new(Expression::Number(1.0f64)),
            TokenType::GT,
            Box::new(Expression::Number(0.0f64)),
        ),
        then_branch: Box::new(Statement::Block(BlockStatement {
            statements: vec![Statement::Print(PrintStatement {
                expression: Expression::Number(1.0f64),
                line: 0,
                column: 0,
            })],
            line: 0,
            column: 0,
        })),
        else_branch: None,
        line: 0,
        column: 0,
    }));

    assert_eq!(parser.next(), goal);
}

#[test]
fn parse_if_else_statement() {
    let source = "if (1 > 0) {print 1;} else {var x = 2;print x;}";
    let mut parser = get_parser(source);

    let goal = Some(Statement::If(IfStatement {
        condition: Expression::Binary(
            Box::new(Expression::Number(1.0f64)),
            TokenType::GT,
            Box::new(Expression::Number(0.0f64)),
        ),
        then_branch: Box::new(Statement::Block(BlockStatement {
            statements: vec![Statement::Print(PrintStatement {
                expression: Expression::Number(1.0f64),
                line: 0,
                column: 0,
            })],
            line: 0,
            column: 0,
        })),
        else_branch: Some(Box::new(Statement::Block(BlockStatement {
            statements: vec![
                Statement::Var(VarStatement {
                    name: String::from("x"),
                    initializer: Some(Expression::Number(2.0f64)),
                    line: 0,
                    column: 0,
                }),
                Statement::Print(PrintStatement {
                    expression: Expression::Variable(String::from("x")),
                    line: 0,
                    column: 0,
                }),
            ],
            line: 0,
            column: 0,
        }))),
        line: 0,
        column: 0,
    }));

    assert_eq!(parser.next(), goal);
}

#[test]
fn parse_function_declaration() {
    let source = "fn add(a, b) { return a + b; }";
    let mut parser = get_parser(source);

    let goal = Some(Statement::Function(FunctionStatement {
        name: String::from("add"),
        params: vec![String::from("a"), String::from("b")],
        body: BlockStatement {
            statements: vec![Statement::Return(ReturnStatement {
                value: Some(Expression::Binary(
                    Box::new(Expression::Variable(String::from("a"))),
                    TokenType::PLUS,
                    Box::new(Expression::Variable(String::from("b"))),
                )),
                line: 0,
                column: 0,
            })],
            line: 0,
            column: 0,
        },
        line: 0,
        column: 0,
    }));

    assert_eq!(parser.next(), goal);
}

#[test]
fn parse_function_call_expression_statement() {
    let source = "add(1, 2);";
    let mut parser = get_parser(source);

    let goal = Some(Statement::Expression(ExpressionStatement {
        expression: Expression::Call(
            String::from("add"),
            vec![Expression::Number(1.0), Expression::Number(2.0)],
        ),
        line: 0,
        column: 0,
    }));

    assert_eq!(parser.next(), goal);
}

#[test]
#[ignore = "note ready yet"]
fn parse_full_code() {
    for i in 0..10 {
        println!("attempt {i}:\n");
        let source = generate_random_program(3);
        println!("CODE\n{}\nCODE", source);

        let parser = get_parser(&source);

        for st in parser {
            println!("{:?}", st);
        }
        println!()
    }
}
