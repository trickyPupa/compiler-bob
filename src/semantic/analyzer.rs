use crate::expression::Expression;
use crate::semantic::environment::{EnvRef, Environment};
use crate::statement::Statement;
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
            Statement::Expression(expr) => self.analyze_expression_statement(expr),
            Statement::Var(name, initializer) => self.analyze_var_statement(name, initializer),
            Statement::Print(expr) => self.analyze_print_statement(expr),
            Statement::Block(statements) => self.analyze_block_statement(statements),
            Statement::If(condition, then_branch, else_branch) => {
                self.analyze_if_statement(condition, then_branch, else_branch.as_deref())
            }
            Statement::While(condition, body) => self.analyze_while_statement(condition, body),
        }
    }

    fn analyze_var_statement(&mut self, name: &str, initializer: &Option<Expression>) {
        if let Some(expr) = initializer {
            self.analyze_expression(expr);
        }

        let initialized = initializer.is_some();
        if !Environment::define_variable(&self.env, name.to_string(), initialized) {
            self.errors
                .push(format!("Variable '{}' is already defined in this scope", name));
        }
    }

    fn analyze_expression_statement(&mut self, expr: &Expression) {
        self.analyze_expression(expr);
    }

    fn analyze_block_statement(&mut self, statements: &[Statement]) {
        let prev_env = Rc::clone(&self.env);
        self.env = Environment::new(Some(&prev_env));

        for st in statements {
            self.analyze_statement(st);
        }

        self.check_unused_variables();
        self.env = prev_env;
    }

    fn analyze_if_statement(
        &mut self,
        condition: &Expression,
        then_branch: &Statement,
        else_branch: Option<&Statement>,
    ) {
        self.analyze_expression(condition);
        self.analyze_statement(then_branch);

        if let Some(else_branch) = else_branch {
            self.analyze_statement(else_branch);
        }
    }

    fn analyze_while_statement(&mut self, condition: &Expression, body: &Statement) {
        self.analyze_expression(condition);
        self.analyze_statement(body);
    }

    fn analyze_print_statement(&mut self, expr: &Expression) {
        self.analyze_expression(expr);
        self.check_unused_variables();
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

    fn check_unused_variables(&mut self) {
        Environment::for_each_local_variable(&self.env, |name, is_used| {
            if !is_used {
                self.errors.push(format!(
                    "[Semantic Warning] Variable '{}' declared, but has not used.",
                    name
                ));
            }
        });
    }

    fn analyze_var_expression(&mut self, name: &str) {
        if Environment::is_variable_defined(&self.env, name) {
            Environment::set_used(&self.env, name);

            if !Environment::is_variable_initialized(&self.env, name) {
                self.errors.push(format!("Uninitialized variable used {name}."));
            }
        } else {
            self.errors.push(format!("Undeclared variable used {name}."));
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
