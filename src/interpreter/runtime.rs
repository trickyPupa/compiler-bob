use std::collections::HashMap;

use crate::expression::Expression;
use crate::statement::Statement;
use crate::token::TokenType;

#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeValue {
    Number(f64),
    Boolean(bool),
    String(String),
    Array(Vec<RuntimeValue>),
    Function {
        params: Vec<String>,
        body: crate::statement::BlockStatement,
    },
    Nil,
}

impl RuntimeValue {
    fn is_truthy(&self) -> bool {
        match self {
            RuntimeValue::Boolean(value) => *value,
            RuntimeValue::Number(value) => *value != 0.0,
            RuntimeValue::String(value) => !value.is_empty(),
            RuntimeValue::Array(values) => !values.is_empty(),
            RuntimeValue::Nil => false,
            RuntimeValue::Function { .. } => true,
        }
    }
}

impl std::fmt::Display for RuntimeValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeValue::Number(value) => write!(f, "{value}"),
            RuntimeValue::Boolean(value) => write!(f, "{value}"),
            RuntimeValue::String(value) => write!(f, "{value}"),
            RuntimeValue::Array(values) => {
                let joined = values
                    .iter()
                    .map(|value| value.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "[{joined}]")
            }
            RuntimeValue::Nil => write!(f, "nil"),
            RuntimeValue::Function { params, .. } => {
                write!(f, "<fn/{}>", params.len())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeError {
    UndefinedVariable(String),
    TypeError(String),
    DivisionByZero,
    IndexOutOfBounds { index: usize, len: usize },
    UnsupportedOperator(TokenType),
    Return(RuntimeValue),
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeError::UndefinedVariable(name) => {
                write!(f, "Variable '{name}' is not defined")
            }
            RuntimeError::TypeError(message) => write!(f, "Type error: {message}"),
            RuntimeError::DivisionByZero => write!(f, "Division by zero"),
            RuntimeError::IndexOutOfBounds { index, len } => {
                write!(f, "Index {index} out of bounds (len {len})")
            }
            RuntimeError::UnsupportedOperator(operator) => {
                write!(f, "Unsupported operator: {operator:?}")
            }
            RuntimeError::Return(runtime_value) => {
                write!(f, "Return({runtime_value})")
            }
        }
    }
}

impl std::error::Error for RuntimeError {}

#[derive(Debug, Default)]
pub struct RuntimeInterpreter<I: Iterator<Item = Statement>> {
    scopes: Vec<HashMap<String, RuntimeValue>>,
    output: Vec<String>,
    statements: I,
}

impl<I: Iterator<Item = Statement>> RuntimeInterpreter<I> {
    pub fn new(statements: I) -> Self {
        Self {
            scopes: vec![HashMap::new()],
            output: Vec::new(),
            statements,
        }
    }

    pub fn execute_program(&mut self) -> Result<(), RuntimeError> {
        while let Some(statement) = self.statements.next() {
            self.execute_statement(&statement)?;
        }
        Ok(())
    }

    fn execute_statement(&mut self, statement: &Statement) -> Result<(), RuntimeError> {
        match statement {
            Statement::Expression(st) => {
                self.evaluate_expression(&st.expression)?;
                Ok(())
            }
            Statement::Var(st) => {
                let value = match &st.initializer {
                    Some(initializer) => self.evaluate_expression(initializer)?,
                    None => RuntimeValue::Nil,
                };
                self.define_value(st.name.clone(), value);
                Ok(())
            }
            Statement::Print(st) => {
                let value = self.evaluate_expression(&st.expression)?;
                self.output.push(value.to_string());
                Ok(())
            }
            Statement::Function(st) => {
                self.define_value(
                    st.name.clone(),
                    RuntimeValue::Function {
                        params: st.params.clone(),
                        body: st.body.clone(),
                    },
                );
                Ok(())
            }
            Statement::Return(st) => {
                let value = match &st.value {
                    Some(expr) => self.evaluate_expression(expr)?,
                    None => RuntimeValue::Nil,
                };
                Err(RuntimeError::Return(value))
            }
            Statement::Block(st) => self.execute_block(&st.statements),
            Statement::If(st) => {
                if self.evaluate_expression(&st.condition)?.is_truthy() {
                    self.execute_statement(&st.then_branch)?;
                } else if let Some(else_branch) = &st.else_branch {
                    self.execute_statement(else_branch)?;
                }
                Ok(())
            }
            Statement::While(st) => {
                while self.evaluate_expression(&st.condition)?.is_truthy() {
                    self.execute_statement(&st.body)?;
                }
                Ok(())
            }
        }
    }

    fn set_value(&mut self, name: &str, value: RuntimeValue) {
        if self.assign_value(name, value.clone()).is_err() {
            self.define_value(name.to_string(), value);
        }
    }

    pub fn get_value(&self, name: &str) -> Option<&RuntimeValue> {
        self.scopes.iter().rev().find_map(|scope| scope.get(name))
    }

    pub fn values(&self) -> &HashMap<String, RuntimeValue> {
        self.scopes
            .first()
            .expect("RuntimeInterpreter must have a global scope")
    }

    pub fn output(&self) -> &[String] {
        &self.output
    }

    fn evaluate_expression(
        &mut self,
        expression: &Expression,
    ) -> Result<RuntimeValue, RuntimeError> {
        match expression {
            Expression::Number(value) => Ok(RuntimeValue::Number(*value)),
            Expression::String(value) => Ok(RuntimeValue::String(value.clone())),
            Expression::Variable(name) => self
                .get_value(name)
                .cloned()
                .ok_or_else(|| RuntimeError::UndefinedVariable(name.clone())),
            Expression::ArrayLiteral(values) => {
                let mut items = Vec::with_capacity(values.len());
                for value in values {
                    items.push(self.evaluate_expression(value)?);
                }
                Ok(RuntimeValue::Array(items))
            }
            Expression::Index(target, index) => {
                let target_value = self.evaluate_expression(target)?;
                let index_value = self.evaluate_expression(index)?;
                self.evaluate_index(target_value, index_value)
            }
            Expression::Assign(name, value_expression) => {
                let value = self.evaluate_expression(value_expression)?;
                self.assign_value(name, value.clone())?;
                Ok(value)
            }
            Expression::AssignIndex(target, index, value_expression) => {
                let value = self.evaluate_expression(value_expression)?;
                self.assign_index(target, index, value.clone())?;
                Ok(value)
            }
            Expression::Call(name, args) => {
                let mut arg_values: Vec<RuntimeValue> = Vec::new();
                for a in args {
                    arg_values.push(self.evaluate_expression(a)?);
                }

                let func_opt = self.get_value(name).cloned();
                match func_opt {
                    Some(RuntimeValue::Function { params, body }) => {
                        if params.len() != arg_values.len() {
                            return Err(RuntimeError::TypeError(format!(
                                "Function '{}' expects {} args, got {}",
                                name,
                                params.len(),
                                arg_values.len()
                            )));
                        }

                        self.push_scope();

                        for (p, v) in params.iter().zip(arg_values.into_iter()) {
                            self.define_value(p.clone(), v);
                        }

                        let result = self.execute_block(&body.statements);
                        let ret_val = match result {
                            Ok(_) => Ok(RuntimeValue::Nil),
                            Err(RuntimeError::Return(val)) => Ok(val),
                            Err(e) => Err(e),
                        };

                        self.pop_scope();
                        ret_val
                    }
                    Some(_) => Err(RuntimeError::TypeError(format!(
                        "'{}' is not a function",
                        name
                    ))),
                    None => Err(RuntimeError::UndefinedVariable(name.clone())),
                }
            }
            Expression::Unary(operator, inner_expression) => {
                let value = self.evaluate_expression(inner_expression)?;
                self.evaluate_unary(*operator, value)
            }
            Expression::Binary(left_expression, operator, right_expression) => {
                self.evaluate_binary(left_expression, *operator, right_expression)
            }
        }
    }

    fn evaluate_unary(
        &self,
        operator: TokenType,
        value: RuntimeValue,
    ) -> Result<RuntimeValue, RuntimeError> {
        match operator {
            TokenType::MINUS => match value {
                RuntimeValue::Number(number) => Ok(RuntimeValue::Number(-number)),
                other => Err(RuntimeError::TypeError(format!(
                    "Unary '-' expects number, got {other:?}"
                ))),
            },
            TokenType::EXCL => Ok(RuntimeValue::Boolean(!value.is_truthy())),
            _ => Err(RuntimeError::UnsupportedOperator(operator)),
        }
    }

    fn evaluate_binary(
        &mut self,
        left_expression: &Expression,
        operator: TokenType,
        right_expression: &Expression,
    ) -> Result<RuntimeValue, RuntimeError> {
        match operator {
            TokenType::AND => {
                let left = self.evaluate_expression(left_expression)?;
                if !left.is_truthy() {
                    return Ok(RuntimeValue::Boolean(false));
                }
                let right = self.evaluate_expression(right_expression)?;
                Ok(RuntimeValue::Boolean(right.is_truthy()))
            }
            TokenType::OR => {
                let left = self.evaluate_expression(left_expression)?;
                if left.is_truthy() {
                    return Ok(RuntimeValue::Boolean(true));
                }
                let right = self.evaluate_expression(right_expression)?;
                Ok(RuntimeValue::Boolean(right.is_truthy()))
            }
            _ => {
                let left = self.evaluate_expression(left_expression)?;
                let right = self.evaluate_expression(right_expression)?;
                self.evaluate_binary_values(left, operator, right)
            }
        }
    }

    fn evaluate_binary_values(
        &self,
        left: RuntimeValue,
        operator: TokenType,
        right: RuntimeValue,
    ) -> Result<RuntimeValue, RuntimeError> {
        use RuntimeValue::{Boolean, Number, String};

        match operator {
            TokenType::PLUS => match (left, right) {
                (Number(l), Number(r)) => Ok(Number(l + r)),
                (String(l), String(r)) => Ok(String(format!("{l}{r}"))),
                (l, r) => Err(RuntimeError::TypeError(format!(
                    "'+' expects two numbers or two strings, got {l:?} and {r:?}"
                ))),
            },
            TokenType::MINUS => self.numeric_operation(left, right, |l, r| Number(l - r), "-"),
            TokenType::STAR => self.numeric_operation(left, right, |l, r| Number(l * r), "*"),
            TokenType::SLASH => match (left, right) {
                (Number(_), Number(0.0)) => Err(RuntimeError::DivisionByZero),
                (Number(l), Number(r)) => Ok(Number(l / r)),
                (l, r) => Err(RuntimeError::TypeError(format!(
                    "'/' expects two numbers, got {l:?} and {r:?}"
                ))),
            },
            TokenType::GT => self.numeric_compare(left, right, |l, r| l > r, ">"),
            TokenType::GTEQ => self.numeric_compare(left, right, |l, r| l >= r, ">="),
            TokenType::LT => self.numeric_compare(left, right, |l, r| l < r, "<"),
            TokenType::LTEQ => self.numeric_compare(left, right, |l, r| l <= r, "<="),
            TokenType::EQEQ => Ok(Boolean(left == right)),
            TokenType::NEQ => Ok(Boolean(left != right)),
            _ => Err(RuntimeError::UnsupportedOperator(operator)),
        }
    }

    fn numeric_operation<F>(
        &self,
        left: RuntimeValue,
        right: RuntimeValue,
        operation: F,
        operator_name: &str,
    ) -> Result<RuntimeValue, RuntimeError>
    where
        F: FnOnce(f64, f64) -> RuntimeValue,
    {
        match (left, right) {
            (RuntimeValue::Number(l), RuntimeValue::Number(r)) => Ok(operation(l, r)),
            (l, r) => Err(RuntimeError::TypeError(format!(
                "'{operator_name}' expects two numbers, got {l:?} and {r:?}"
            ))),
        }
    }

    fn numeric_compare<F>(
        &self,
        left: RuntimeValue,
        right: RuntimeValue,
        compare: F,
        operator_name: &str,
    ) -> Result<RuntimeValue, RuntimeError>
    where
        F: FnOnce(f64, f64) -> bool,
    {
        match (left, right) {
            (RuntimeValue::Number(l), RuntimeValue::Number(r)) => {
                Ok(RuntimeValue::Boolean(compare(l, r)))
            }
            (l, r) => Err(RuntimeError::TypeError(format!(
                "'{operator_name}' expects two numbers, got {l:?} and {r:?}"
            ))),
        }
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    fn define_value(&mut self, name: String, value: RuntimeValue) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, value);
        }
    }

    fn assign_value(&mut self, name: &str, value: RuntimeValue) -> Result<(), RuntimeError> {
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(name) {
                scope.insert(name.to_string(), value);
                return Ok(());
            }
        }
        Err(RuntimeError::UndefinedVariable(name.to_string()))
    }

    fn get_value_mut(&mut self, name: &str) -> Option<&mut RuntimeValue> {
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(name) {
                return scope.get_mut(name);
            }
        }
        None
    }

    fn evaluate_index(
        &self,
        target: RuntimeValue,
        index_value: RuntimeValue,
    ) -> Result<RuntimeValue, RuntimeError> {
        let index = self.expect_index(index_value)?;

        match target {
            RuntimeValue::Array(values) => {
                values
                    .get(index)
                    .cloned()
                    .ok_or(RuntimeError::IndexOutOfBounds {
                        index,
                        len: values.len(),
                    })
            }
            other => Err(RuntimeError::TypeError(format!(
                "Indexing expects array, got {other:?}"
            ))),
        }
    }

    fn expect_index(&self, value: RuntimeValue) -> Result<usize, RuntimeError> {
        match value {
            RuntimeValue::Number(number) => {
                if !number.is_finite() || number.fract() != 0.0 || number < 0.0 {
                    Err(RuntimeError::TypeError(
                        "Index must be a non-negative integer".to_string(),
                    ))
                } else {
                    Ok(number as usize)
                }
            }
            other => Err(RuntimeError::TypeError(format!(
                "Index must be a number, got {other:?}"
            ))),
        }
    }

    fn assign_index(
        &mut self,
        target: &Expression,
        index: &Expression,
        value: RuntimeValue,
    ) -> Result<(), RuntimeError> {
        let mut indices: Vec<&Expression> = Vec::new();
        let base_name = self
            .collect_index_chain(target, &mut indices)
            .ok_or_else(|| {
                RuntimeError::TypeError("Invalid index assignment target".to_string())
            })?;

        indices.push(index);

        let mut evaluated_indices: Vec<usize> = Vec::with_capacity(indices.len());
        for idx_expr in indices {
            let idx_value = self.evaluate_expression(idx_expr)?;
            evaluated_indices.push(self.expect_index(idx_value)?);
        }

        let base_value = self
            .get_value_mut(base_name)
            .ok_or_else(|| RuntimeError::UndefinedVariable(base_name.to_string()))?;

        Self::assign_nested_index(base_value, &evaluated_indices, value)
    }

    fn collect_index_chain<'a>(
        &self,
        expr: &'a Expression,
        indices: &mut Vec<&'a Expression>,
    ) -> Option<&'a str> {
        match expr {
            Expression::Variable(name) => Some(name.as_str()),
            Expression::Index(target, index) => {
                let base_name = self.collect_index_chain(target, indices)?;
                indices.push(index);
                Some(base_name)
            }
            _ => None,
        }
    }

    fn assign_nested_index(
        target: &mut RuntimeValue,
        indices: &[usize],
        value: RuntimeValue,
    ) -> Result<(), RuntimeError> {
        if indices.is_empty() {
            return Err(RuntimeError::TypeError(
                "Missing index for assignment".to_string(),
            ));
        }

        match target {
            RuntimeValue::Array(values) => {
                let index = indices[0];
                if index >= values.len() {
                    return Err(RuntimeError::IndexOutOfBounds {
                        index,
                        len: values.len(),
                    });
                }

                if indices.len() == 1 {
                    values[index] = value;
                    Ok(())
                } else {
                    Self::assign_nested_index(&mut values[index], &indices[1..], value)
                }
            }
            other => Err(RuntimeError::TypeError(format!(
                "Indexing expects array, got {other:?}"
            ))),
        }
    }

    fn execute_block(&mut self, statements: &[Statement]) -> Result<(), RuntimeError> {
        self.push_scope();

        for nested in statements {
            match self.execute_statement(nested) {
                Ok(_) => {}
                Err(e) => {
                    self.pop_scope();
                    return Err(e);
                }
            }
        }

        self.pop_scope();
        Ok(())
    }
}
