use compiler::{
    expression::Expression,
    lexer::Lexer,
    parser::Parser,
    statement::Statement,
    token::TokenType,
};

fn get_parser(src: &str) -> Parser<Lexer> {
    let lexer = Lexer::new(src);
    Parser::new(lexer)
}

#[test]
fn parse_print_statement() {
    let mut parser = get_parser("print 1;");

    let goal = Some(Statement::Print(Expression::Number(1.0f64)));

    assert_eq!(parser.next(), goal);
}

#[test]
fn parse_var_statement() {
    let mut parser = get_parser("var x = 1;");

    let goal = Some(Statement::Var(
        String::from("x"),
        Some(Expression::Number(1.0f64)),
    ));

    assert_eq!(parser.next(), goal);
}

#[test]
fn parse_block_statement() {
    let source = "{print 1;var x = 2;}";
    let mut parser = get_parser(source);

    let goal = Some(Statement::Block(vec![
        Statement::Print(Expression::Number(1.0f64)),
        Statement::Var(String::from("x"), Some(Expression::Number(2.0f64)))
    ]));

    assert_eq!(parser.next(), goal);
}

#[test]
fn parse_if_statement() {
    let source = "if (1 > 0) {print 1;}";
    let mut parser = get_parser(source);

    let goal = Some(Statement::If(
        Expression::Binary(
            Box::new(Expression::Number(1.0f64)),
            TokenType::LT,
            Box::new(Expression::Number(0.0f64)),
        ),
        Box::new(Statement::Block(vec![Statement::Print(
            Expression::Number(1.0f64),
        )])),
        None,
    ));

    assert_eq!(parser.next(), goal);
}

#[test]
fn parse_if_else_statement() {
    let source = "if (1 > 0) {print 1;} else {var x = 2;print x;}";
    let mut parser = get_parser(source);

    let goal = Some(Statement::If(
        Expression::Binary(
            Box::new(Expression::Number(1.0f64)),
            TokenType::LT,
            Box::new(Expression::Number(0.0f64)),
        ),
        Box::new(Statement::Block(vec![
            Statement::Print(Expression::Number(1.0f64)),
        ])),
        Some(Box::new(Statement::Block(vec![
            Statement::Var(String::from("x"), Some(Expression::Number(2.0f64))),
            Statement::Print(Expression::Variable(String::from("x"))),
        ]))),
    ));

    assert_eq!(parser.next(), goal);
}
