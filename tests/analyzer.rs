
use compiler::semantic::analyzer::Analyzer;
use compiler::expression::Expression;
use compiler::statement::{
    BlockStatement, ExpressionStatement, PrintStatement, Statement, VarStatement,
};

fn var(name: &str, initializer: Option<Expression>, line: usize, column: usize) -> Statement {
    Statement::Var(VarStatement {
        name: name.to_string(),
        initializer,
        line,
        column,
    })
}

fn print(expression: Expression, line: usize, column: usize) -> Statement {
    Statement::Print(PrintStatement {
        expression,
        line,
        column,
    })
}

fn expr(expression: Expression, line: usize, column: usize) -> Statement {
    Statement::Expression(ExpressionStatement {
        expression,
        line,
        column,
    })
}

fn block(statements: Vec<Statement>, line: usize, column: usize) -> Statement {
    Statement::Block(BlockStatement {
        statements,
        line,
        column,
    })
}

#[test]
fn reports_undeclared_variable_usage() {
    let program = vec![print(Expression::Variable("x".to_string()), 1, 1)];
    let mut analyzer = Analyzer::new(program.into_iter());

    analyzer.analyze();

    assert!(
        analyzer
            .errors()
            .iter()
            .any(|e| e.contains("Undeclared variable used x"))
    );
}

#[test]
fn reports_uninitialized_variable_usage() {
    let program = vec![
        var("x", None, 1, 1),
        print(Expression::Variable("x".to_string()), 2, 1),
    ];
    let mut analyzer = Analyzer::new(program.into_iter());

    analyzer.analyze();

    assert!(
        analyzer
            .errors()
            .iter()
            .any(|e| e.contains("Uninitialized variable used x"))
    );
}

#[test]
fn reports_duplicate_variable_in_same_scope_with_position() {
    let program = vec![
        var("x", Some(Expression::Number(1.0)), 1, 1),
        var("x", Some(Expression::Number(2.0)), 3, 7),
    ];
    let mut analyzer = Analyzer::new(program.into_iter());

    analyzer.analyze();

    assert!(analyzer.errors().iter().any(|e| {
        e.contains("[Line 3, Col 7]") && e.contains("already defined in this scope")
    }));
}

#[test]
fn reports_assignment_to_undefined_variable() {
    let program = vec![expr(
        Expression::Assign("x".to_string(), Box::new(Expression::Number(10.0))),
        1,
        1,
    )];
    let mut analyzer = Analyzer::new(program.into_iter());

    analyzer.analyze();

    assert!(
        analyzer
            .errors()
            .iter()
            .any(|e| e.contains("Cannot assign to undefined variable 'x'"))
    );
}

#[test]
fn warns_about_unused_variable_in_block() {
    let program = vec![block(
        vec![var("x", Some(Expression::Number(1.0)), 2, 3)],
        1,
        1,
    )];
    let mut analyzer = Analyzer::new(program.into_iter());

    analyzer.analyze();

    assert!(analyzer.warnings().iter().any(|e| {
        e.contains("[Line 1, Col 1]") && e.contains("declared, but has not used")
    }));
}

#[test]
fn no_errors_for_initialized_then_used_variable() {
    let program = vec![
        var("x", Some(Expression::Number(1.0)), 1, 1),
        print(Expression::Variable("x".to_string()), 2, 1),
    ];
    let mut analyzer = Analyzer::new(program.into_iter());

    analyzer.analyze();

    assert!(analyzer.errors().is_empty());
}
