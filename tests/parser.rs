use compiler::code_generator::generate_random_program;
use compiler::expression::Expression;
use compiler::lexer::Lexer;
use compiler::parser::Parser;
use compiler::statement::Statement;
use compiler::token::TokenType;

fn get_parser(src: &str) -> Parser<Lexer> {
    let lexer = Lexer::new(src);
    Parser::new_debug(lexer)
}

#[test]
fn parse_print_statement() {
    let mut parser = get_parser("print 1;");

    let goal = Some(Statement::Print(Expression::Number(1.0f64)));

    assert_eq!(parser.next(), goal);
}

#[test]
fn parse_complex_print_statement() {
    let mut parser = get_parser("print 1 + 2;");

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
fn parse_assignment() {
    let mut parser = get_parser("x = 1;");

    let goal = Some(Statement::Expression(Expression::Assign(
        String::from("x"),
        Box::new(Expression::Number(1f64)),
    )));

    assert_eq!(parser.next(), goal);
}

#[test]
fn parse_simple_combo() {
    let mut parser = get_parser("var x = 2;x = 1;print x;");

    let s1 = Some(Statement::Var(
        String::from("x"),
        Some(Expression::Number(2.0f64)),
    ));
    let s2 = Some(Statement::Expression(Expression::Assign(
        String::from("x"),
        Box::new(Expression::Number(1f64)),
    )));
    let s3 = Some(Statement::Print(Expression::Variable(String::from("x"))));

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

    let goal = Some(Statement::Expression(Expression::Binary(
        Box::new(Expression::Number(1.0f64)),
        TokenType::LT,
        Box::new(Expression::Number(4.0f64)),
    )));

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
    let goal = Some(Statement::Expression(Expression::Binary(
        Box::new(left),
        TokenType::AND,
        Box::new(right),
    )));

    assert_eq!(parser.next(), goal);
}

#[test]
fn parse_block_statement() {
    let source = "{print 1;var x = 2;}";
    let mut parser = get_parser(source);

    let goal = Some(Statement::Block(vec![
        Statement::Print(Expression::Number(1.0f64)),
        Statement::Var(String::from("x"), Some(Expression::Number(2.0f64))),
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
            TokenType::GT,
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
            TokenType::GT,
            Box::new(Expression::Number(0.0f64)),
        ),
        Box::new(Statement::Block(vec![Statement::Print(
            Expression::Number(1.0f64),
        )])),
        Some(Box::new(Statement::Block(vec![
            Statement::Var(String::from("x"), Some(Expression::Number(2.0f64))),
            Statement::Print(Expression::Variable(String::from("x"))),
        ]))),
    ));

    assert_eq!(parser.next(), goal);
}

#[test]
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
