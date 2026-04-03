use crate::expression::Expression;
use crate::semantic::environment::{EnvRef, Environment};
use crate::statement::{
    BlockStatement, ExpressionStatement, IfStatement, PrintStatement, Statement, VarStatement,
    WhileStatement,
};
use std::rc::Rc;

pub struct Analyzer<T: Iterator<Item = Statement>> {
    env: EnvRef,
    errors: Vec<String>,
    statements: T,
}

impl<T: Iterator<Item = Statement>> Analyzer<T> {
    pub fn new(parser: T) -> Self {
        Analyzer {
            env: Environment::new(None),
            errors: Vec::new(),
            statements: parser,
        }
    }

    pub fn analyze(&mut self) {
        loop {
            let flag = self.analyze_next();
            if !flag {
                break;
            }
        }
    }

    pub fn analyze_next(&mut self) -> bool {
        if let Some(st) = self.statements.next() {
            self.analyze_statement(&st);
            true
        } else {
            false
        }
    }

    fn analyze_statement(&mut self, st: &Statement) {
        match st {
            Statement::Expression(st) => self.analyze_expression_statement(st),
            Statement::Var(st) => self.analyze_var_statement(st),
            Statement::Print(st) => self.analyze_print_statement(st),
            Statement::Block(st) => self.analyze_block_statement(st),
            Statement::If(st) => self.analyze_if_statement(st),
            Statement::While(st) => self.analyze_while_statement(st),
        }
    }

    fn analyze_var_statement(&mut self, st: &VarStatement) {
        if let Some(expr) = &st.initializer {
            self.analyze_expression(expr);
        }

        let initialized = st.initializer.is_some();
        if !Environment::define_variable(&self.env, st.name.clone(), initialized) {
            self.errors.push(format!(
                "[Line {}, Col {}] Variable '{}' is already defined in this scope",
                st.line, st.column, st.name
            ));
        }
    }

    fn analyze_expression_statement(&mut self, st: &ExpressionStatement) {
        self.analyze_expression(&st.expression);
    }

    fn analyze_block_statement(&mut self, st: &BlockStatement) {
        let prev_env = Rc::clone(&self.env);
        self.env = Environment::new(Some(&prev_env));

        for nested in &st.statements {
            self.analyze_statement(nested);
        }

        self.check_unused_variables(st.line, st.column);
        self.env = prev_env;
    }

    fn analyze_if_statement(&mut self, st: &IfStatement) {
        self.analyze_expression(&st.condition);
        self.analyze_statement(&st.then_branch);

        if let Some(else_branch) = st.else_branch.as_deref() {
            self.analyze_statement(else_branch);
        }
    }

    fn analyze_while_statement(&mut self, st: &WhileStatement) {
        self.analyze_expression(&st.condition);
        self.analyze_statement(&st.body);
    }

    fn analyze_print_statement(&mut self, st: &PrintStatement) {
        self.analyze_expression(&st.expression);
        self.check_unused_variables(st.line, st.column);
    }

    fn analyze_expression(&mut self, expr: &Expression) {
        match expr {
            Expression::Number(_) | Expression::String(_) => {}
            Expression::Variable(name) => self.analyze_var_expression(name),
            Expression::Binary(left, _, right) => {
                self.analyze_expression(left);
                self.analyze_expression(right);
            }
            Expression::Unary(_, inner) => self.analyze_expression(inner),
            Expression::Assign(name, value) => self.analyze_assign_expression(name, value),
        }
    }

    fn check_unused_variables(&mut self, line: usize, column: usize) {
        Environment::for_each_local_variable(&self.env, |name, is_used| {
            if !is_used {
                self.errors.push(format!(
                    "[Line {}, Col {}] [Semantic Warning] Variable '{}' declared, but has not used.",
                    line, column, name
                ));
            }
        });
    }

    fn analyze_var_expression(&mut self, name: &str) {
        if Environment::is_variable_defined(&self.env, name) {
            Environment::set_used(&self.env, name);

            if !Environment::is_variable_initialized(&self.env, name) {
                self.errors
                    .push(format!("Uninitialized variable used {name}."));
            }
        } else {
            self.errors
                .push(format!("Undeclared variable used {name}."));
        }
    }

    fn analyze_assign_expression(&mut self, name: &str, value: &Expression) {
        self.analyze_expression(value);

        if Environment::is_variable_defined(&self.env, name) {
            Environment::set_initialized(&self.env, name);
        } else {
            self.errors
                .push(format!("Cannot assign to undefined variable '{}'", name));
        }
    }

    pub fn errors(&self) -> &[String] {
        &self.errors
    }
}

#[cfg(test)]
mod tests {
    use super::Analyzer;
    use crate::expression::Expression;
    use crate::statement::{
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

        assert!(analyzer.errors().iter().any(|e| {
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
}
