use crate::common::data_type::{DataType, NUMERIC_SUPPORTED_OPERATIONS};
use crate::expression::Expression;
use crate::semantic::environment::{EnvOps, EnvRef, Environment};
use crate::statement::*;
use crate::token::TokenType;
use std::rc::Rc;

pub struct Analyzer<T: Iterator<Item = Statement>> {
    env: EnvRef,
    errors: Vec<String>,
    warnings: Vec<String>,
    statements: T,
}

impl<T: Iterator<Item = Statement>> Analyzer<T> {
    pub fn new(parser: T) -> Self {
        Analyzer {
            env: Environment::new(None),
            errors: Vec::new(),
            warnings: Vec::new(),
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
        let mut dtype = DataType::Unknown;
        if let Some(expr) = &st.initializer {
            dtype = self.analyze_expression(expr);
        }

        let initialized = st.initializer.is_some();
        if !self
            .env
            .define_variable(st.name.clone(), initialized, dtype)
        {
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

    fn analyze_expression(&mut self, expr: &Expression) -> DataType {
        match expr {
            Expression::Number(_) => DataType::Numeric,
            Expression::String(_) => DataType::String,
            Expression::Variable(name) => self.analyze_var_expression(name),
            Expression::Binary(left, tt, right) => self.analyze_binary_expression(left, tt, right),
            Expression::Unary(_tt, inner) => self.analyze_expression(inner),
            Expression::Assign(name, value) => self.analyze_assign_expression(name, value),
        }
    }

    fn check_unused_variables(&mut self, line: usize, column: usize) {
        self.env.for_each_local_variable(|name, is_used| {
            if !is_used {
                self.warnings.push(format!(
                    "[Line {}, Col {}] Variable '{}' declared, but has not used.",
                    line, column, name
                ));
            }
        });
    }

    fn analyze_var_expression(&mut self, name: &str) -> DataType {
        match self.env.with_variable_mut(name, |symbol| {
            symbol.is_used = true;
            (symbol.dtype, symbol.is_initialized)
        }) {
            Some((dtype, is_initialized)) => {
                if !is_initialized {
                    self.errors
                        .push(format!("Uninitialized variable used {name}."));
                }
                dtype
            }
            None => {
                self.errors
                    .push(format!("Undeclared variable used {name}."));
                DataType::Unknown
            }
        }
    }

    fn analyze_binary_expression(
        &mut self,
        left: &Expression,
        tt: &TokenType,
        right: &Expression,
    ) -> DataType {
        let left = self.analyze_expression(left);
        let right = self.analyze_expression(right);

        let error_msg = format!(
            "Unsupported operation for {:?} and {:?} data types",
            left, right
        );

        if left == DataType::Unknown || right == DataType::Unknown {
            DataType::Unknown
        } else if left != right {
            self.errors.push(error_msg);
            DataType::Unknown
        } else {
            if left == DataType::Numeric && !NUMERIC_SUPPORTED_OPERATIONS.contains(tt) {
                self.errors.push(error_msg);
            }
            // todo
            left
        }
    }

    fn analyze_assign_expression(&mut self, name: &str, value: &Expression) -> DataType {
        let dtype = self.analyze_expression(value);

        if self
            .env
            .with_variable_mut(name, |symbol| symbol.is_initialized = true)
            .is_none()
        {
            self.errors
                .push(format!("Cannot assign to undefined variable '{}'", name));
        }

        dtype
    }

    pub fn errors(&self) -> &[String] {
        &self.errors
    }

    pub fn warnings(&self) -> &[String] {
        &self.warnings
    }
}
