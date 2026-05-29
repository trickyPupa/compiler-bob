use compiler::expression::Expression;
use compiler::interpreter::{RuntimeInterpreter, RuntimeValue};
use compiler::statement::{
    BlockStatement, ExpressionStatement, FunctionStatement, IfStatement, PrintStatement,
    ReturnStatement, Statement, VarStatement, WhileStatement,
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
    runtime.execute_program().expect("program should run");

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
    runtime.execute_program().expect("program should run");

    assert_eq!(
        runtime.get_value("counter"),
        Some(&RuntimeValue::Number(3.0))
    );
    assert_eq!(runtime.output(), ["10", "3"]);
}

#[test]
fn errors_on_unknown_variable() {
    let mut runtime =
        RuntimeInterpreter::new(vec![print(Expression::Variable("x".to_string()))].into_iter());
    let result = runtime.execute_program();

    assert!(result.is_err());
}

#[test]
fn executes_function_call_and_return() {
    let function = Statement::Function(FunctionStatement {
        name: "add".to_string(),
        params: vec!["a".to_string(), "b".to_string()],
        body: BlockStatement {
            statements: vec![Statement::Return(ReturnStatement {
                value: Some(Expression::Binary(
                    Box::new(Expression::Variable("a".to_string())),
                    TokenType::PLUS,
                    Box::new(Expression::Variable("b".to_string())),
                )),
                line: 1,
                column: 1,
            })],
            line: 1,
            column: 1,
        },
        line: 1,
        column: 1,
    });

    let program = vec![
        function,
        print(Expression::Call(
            "add".to_string(),
            vec![Expression::Number(1.0), Expression::Number(2.0)],
        )),
    ];

    let mut runtime = RuntimeInterpreter::new(program.into_iter());
    runtime.execute_program().expect("program should run");

    assert_eq!(runtime.output(), ["3"]);
}

#[test]
fn executes_array_indexing_and_assignment() {
    let program = vec![
        var(
            "xs",
            Some(Expression::ArrayLiteral(vec![
                Expression::Number(1.0),
                Expression::Number(2.0),
            ])),
        ),
        print(Expression::Index(
            Box::new(Expression::Variable("xs".to_string())),
            Box::new(Expression::Number(1.0)),
        )),
        expr(Expression::AssignIndex(
            Box::new(Expression::Variable("xs".to_string())),
            Box::new(Expression::Number(0.0)),
            Box::new(Expression::Number(5.0)),
        )),
        print(Expression::Index(
            Box::new(Expression::Variable("xs".to_string())),
            Box::new(Expression::Number(0.0)),
        )),
    ];

    let mut runtime = RuntimeInterpreter::new(program.into_iter());
    runtime.execute_program().expect("program should run");

    assert_eq!(runtime.output(), ["2", "5"]);
    assert_eq!(
        runtime.get_value("xs"),
        Some(&RuntimeValue::Array(vec![
            RuntimeValue::Number(5.0),
            RuntimeValue::Number(2.0)
        ]))
    );
}
