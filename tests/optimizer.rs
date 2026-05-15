use compiler::expression::Expression;
use compiler::optimizer::optimize_statements;
use compiler::statement::{
    ExpressionStatement, IfStatement, PrintStatement, ReturnStatement, Statement, VarStatement,
};
use compiler::token::TokenType;

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
fn removes_dead_code_after_return() {
    let program = vec![
        Statement::Return(ReturnStatement {
            value: Some(Expression::Number(1.0)),
            line: 1,
            column: 1,
        }),
        print(Expression::Number(2.0)),
    ];

    let optimized = optimize_statements(program);

    assert_eq!(optimized.len(), 1);
    assert!(matches!(optimized[0], Statement::Return(_)));
}

#[test]
fn folds_and_propagates_constants() {
    let program = vec![
        Statement::Var(VarStatement {
            name: "x".to_string(),
            initializer: Some(Expression::Binary(
                Box::new(Expression::Number(1.0)),
                TokenType::PLUS,
                Box::new(Expression::Number(2.0)),
            )),
            line: 1,
            column: 1,
        }),
        print(Expression::Binary(
            Box::new(Expression::Variable("x".to_string())),
            TokenType::PLUS,
            Box::new(Expression::Number(3.0)),
        )),
    ];

    let optimized = optimize_statements(program);

    match &optimized[0] {
        Statement::Var(st) => {
            assert!(matches!(st.initializer, Some(Expression::Number(3.0))));
        }
        _ => panic!("expected var statement"),
    }

    match &optimized[1] {
        Statement::Print(st) => {
            assert!(matches!(st.expression, Expression::Number(6.0)));
        }
        _ => panic!("expected print statement"),
    }
}

#[test]
fn removes_constant_false_branch() {
    let program = vec![Statement::If(IfStatement {
        condition: Expression::Binary(
            Box::new(Expression::Number(1.0)),
            TokenType::GT,
            Box::new(Expression::Number(2.0)),
        ),
        then_branch: Box::new(print(Expression::Number(10.0))),
        else_branch: Some(Box::new(print(Expression::Number(20.0)))),
        line: 1,
        column: 1,
    })];

    let optimized = optimize_statements(program);

    assert_eq!(optimized.len(), 1);
    match &optimized[0] {
        Statement::Print(st) => {
            assert!(matches!(st.expression, Expression::Number(20.0)));
        }
        _ => panic!("expected print statement"),
    }
}

#[test]
fn removes_while_with_false_condition() {
    let program = vec![Statement::While(compiler::statement::WhileStatement {
        condition: Expression::Binary(
            Box::new(Expression::Number(1.0)),
            TokenType::GT,
            Box::new(Expression::Number(2.0)),
        ),
        body: Box::new(expr(Expression::Assign(
            "x".to_string(),
            Box::new(Expression::Number(1.0)),
        ))),
        line: 1,
        column: 1,
    })];

    let optimized = optimize_statements(program);

    assert!(optimized.is_empty());
}
