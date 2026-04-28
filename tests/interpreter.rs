use compiler::expression::Expression;
use compiler::interpreter::{RuntimeInterpreter, RuntimeValue};
use compiler::statement::{
    ExpressionStatement, IfStatement, PrintStatement, Statement, VarStatement, WhileStatement,
};
use compiler::token::TokenType;

fn var(name: &str, initializer: Option<Expression>) -> Statement {
    Statement::Var(VarStatement {
        name: name.to_string(),
        initializer,
        line: 1,
        column: 1,
    })
}

fn expr(expression: Expression) -> Statement {
    Statement::Expression(ExpressionStatement {
        expression,
        line: 1,
        column: 1,
    })
}

fn print(expression: Expression) -> Statement {
    Statement::Print(PrintStatement {
        expression,
        line: 1,
        column: 1,
    })
}

#[test]
fn executes_statements_sequentially() {
    let program = vec![
        var("x", Some(Expression::Number(1.0))),
        expr(Expression::Assign(
            "x".to_string(),
            Box::new(Expression::Binary(
                Box::new(Expression::Variable("x".to_string())),
                TokenType::PLUS,
                Box::new(Expression::Number(2.0)),
            )),
        )),
        print(Expression::Variable("x".to_string())),
    ];

    let mut runtime = RuntimeInterpreter::new(program.into_iter());
    runtime
        .execute_program()
        .expect("program should run");

    assert_eq!(runtime.get_value("x"), Some(&RuntimeValue::Number(3.0)));
    assert_eq!(runtime.output(), ["3"]);
}

#[test]
fn executes_if_else_and_while() {
    let while_body = expr(Expression::Assign(
        "counter".to_string(),
        Box::new(Expression::Binary(
            Box::new(Expression::Variable("counter".to_string())),
            TokenType::PLUS,
            Box::new(Expression::Number(1.0)),
        )),
    ));

    let program = vec![
        var("counter", Some(Expression::Number(0.0))),
        Statement::If(IfStatement {
            condition: Expression::Binary(
                Box::new(Expression::Number(2.0)),
                TokenType::GT,
                Box::new(Expression::Number(1.0)),
            ),
            then_branch: Box::new(print(Expression::Number(10.0))),
            else_branch: Some(Box::new(print(Expression::Number(20.0)))),
            line: 1,
            column: 1,
        }),
        Statement::While(WhileStatement {
            condition: Expression::Binary(
                Box::new(Expression::Variable("counter".to_string())),
                TokenType::LT,
                Box::new(Expression::Number(3.0)),
            ),
            body: Box::new(while_body),
            line: 1,
            column: 1,
        }),
        print(Expression::Variable("counter".to_string())),
    ];

    let mut runtime = RuntimeInterpreter::new(program.into_iter());
    runtime
        .execute_program()
        .expect("program should run");

    assert_eq!(
        runtime.get_value("counter"),
        Some(&RuntimeValue::Number(3.0))
    );
    assert_eq!(runtime.output(), ["10", "3"]);
}

#[test]
fn errors_on_unknown_variable() {
    let mut runtime = RuntimeInterpreter::new(vec![print(Expression::Variable("x".to_string()))].into_iter());
    let result = runtime.execute_program();

    assert!(result.is_err());
}
