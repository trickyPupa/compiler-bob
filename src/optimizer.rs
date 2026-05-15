use crate::expression::Expression;
use crate::statement::{
    BlockStatement, ExpressionStatement, FunctionStatement, IfStatement, PrintStatement,
    ReturnStatement, Statement, VarStatement, WhileStatement,
};
use crate::token::TokenType;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq)]
enum ConstValue {
    Number(f64),
    String(String),
    Boolean(bool),
}

impl ConstValue {
    fn is_truthy(&self) -> bool {
        match self {
            ConstValue::Boolean(value) => *value,
            ConstValue::Number(value) => *value != 0.0,
            ConstValue::String(value) => !value.is_empty(),
        }
    }
}

#[derive(Debug, Default)]
pub struct Optimizer {
    scopes: Vec<HashMap<String, ConstValue>>,
}

impl Optimizer {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
        }
    }

    pub fn optimize_statements(&mut self, statements: Vec<Statement>) -> Vec<Statement> {
        let (optimized, _) = self.optimize_list(statements);
        optimized
    }

    fn optimize_list(&mut self, statements: Vec<Statement>) -> (Vec<Statement>, bool) {
        let mut result = Vec::new();
        let mut did_return = false;

        for st in statements {
            if did_return {
                break;
            }
            let (optimized, returns) = self.optimize_statement(st);
            if let Some(statement) = optimized {
                result.push(statement);
            }
            if returns {
                did_return = true;
            }
        }

        (result, did_return)
    }

    fn optimize_statement(&mut self, st: Statement) -> (Option<Statement>, bool) {
        match st {
            Statement::Expression(st) => self.optimize_expression_statement(st),
            Statement::Var(st) => self.optimize_var_statement(st),
            Statement::Print(st) => self.optimize_print_statement(st),
            Statement::Block(st) => self.optimize_block_statement(st),
            Statement::If(st) => self.optimize_if_statement(st),
            Statement::While(st) => self.optimize_while_statement(st),
            Statement::Function(st) => self.optimize_function_statement(st),
            Statement::Return(st) => self.optimize_return_statement(st),
        }
    }

    fn optimize_var_statement(&mut self, st: VarStatement) -> (Option<Statement>, bool) {
        let mut initializer = None;
        if let Some(expr) = st.initializer {
            let (expr, const_value) = self.optimize_expression(expr);
            initializer = Some(expr);
            if let Some(value) = const_value {
                self.define_const(st.name.clone(), value);
            } else {
                self.clear_const(&st.name);
            }
        } else {
            self.clear_const(&st.name);
        }

        (
            Some(Statement::Var(VarStatement {
                name: st.name,
                initializer,
                line: st.line,
                column: st.column,
            })),
            false,
        )
    }

    fn optimize_expression_statement(
        &mut self,
        st: ExpressionStatement,
    ) -> (Option<Statement>, bool) {
        let (expression, const_value) = self.optimize_expression(st.expression);

        if let Expression::Assign(name, _) = &expression {
            if let Some(value) = const_value {
                self.set_const(name, value);
            } else {
                self.clear_const(name);
            }
        }

        (
            Some(Statement::Expression(ExpressionStatement {
                expression,
                line: st.line,
                column: st.column,
            })),
            false,
        )
    }

    fn optimize_print_statement(&mut self, st: PrintStatement) -> (Option<Statement>, bool) {
        let (expression, _) = self.optimize_expression(st.expression);
        (
            Some(Statement::Print(PrintStatement {
                expression,
                line: st.line,
                column: st.column,
            })),
            false,
        )
    }

    fn optimize_block_statement(&mut self, st: BlockStatement) -> (Option<Statement>, bool) {
        self.push_scope();
        let (statements, did_return) = self.optimize_list(st.statements);
        self.pop_scope();

        (
            Some(Statement::Block(BlockStatement {
                statements,
                line: st.line,
                column: st.column,
            })),
            did_return,
        )
    }

    fn optimize_if_statement(&mut self, st: IfStatement) -> (Option<Statement>, bool) {
        let (condition, condition_const) = self.optimize_expression(st.condition);
        let condition_bool = if is_pure_expression(&condition) {
            condition_const.as_ref().map(ConstValue::is_truthy)
        } else {
            None
        };

        if let Some(is_true) = condition_bool {
            if is_true {
                return self.optimize_statement(*st.then_branch);
            }

            if let Some(else_branch) = st.else_branch {
                return self.optimize_statement(*else_branch);
            }

            return (None, false);
        }

        let base_env = self.scopes.clone();

        let mut then_optimizer = Optimizer {
            scopes: base_env.clone(),
        };
        let (then_branch_opt, then_returns) = then_optimizer.optimize_statement(*st.then_branch);
        let then_env = then_optimizer.scopes;

        let has_else = st.else_branch.is_some();
        let (else_branch_opt, else_returns, else_env) = if let Some(else_branch) = st.else_branch {
            let mut else_optimizer = Optimizer {
                scopes: base_env.clone(),
            };
            let (else_branch_opt, else_returns) = else_optimizer.optimize_statement(*else_branch);
            (else_branch_opt, else_returns, else_optimizer.scopes)
        } else {
            (None, false, base_env.clone())
        };

        self.scopes = merge_envs(base_env, then_env, else_env);

        let then_branch = then_branch_opt.unwrap_or_else(|| empty_block(st.line, st.column));
        let else_branch = else_branch_opt.map(Box::new);

        (
            Some(Statement::If(IfStatement {
                condition,
                then_branch: Box::new(then_branch),
                else_branch,
                line: st.line,
                column: st.column,
            })),
            has_else && then_returns && else_returns,
        )
    }

    fn optimize_while_statement(&mut self, st: WhileStatement) -> (Option<Statement>, bool) {
        let (condition, condition_const) = self.optimize_expression(st.condition);
        let condition_bool = if is_pure_expression(&condition) {
            condition_const.as_ref().map(ConstValue::is_truthy)
        } else {
            None
        };

        if let Some(is_true) = condition_bool
            && !is_true
        {
            return (None, false);
        }

        let (body, body_returns) = self.optimize_statement(*st.body);
        let body = body.unwrap_or_else(|| empty_block(st.line, st.column));

        let mut assigned = HashSet::new();
        collect_assigned_names(&body, &mut assigned);
        for name in assigned {
            self.clear_const(&name);
        }

        (
            Some(Statement::While(WhileStatement {
                condition,
                body: Box::new(body),
                line: st.line,
                column: st.column,
            })),
            body_returns,
        )
    }

    fn optimize_function_statement(&mut self, st: FunctionStatement) -> (Option<Statement>, bool) {
        self.clear_const(&st.name);

        let mut func_optimizer = Optimizer::new();
        let (statements, _) = func_optimizer.optimize_list(st.body.statements);

        let body = BlockStatement {
            statements,
            line: st.body.line,
            column: st.body.column,
        };

        (
            Some(Statement::Function(FunctionStatement {
                name: st.name,
                params: st.params,
                body,
                line: st.line,
                column: st.column,
            })),
            false,
        )
    }

    fn optimize_return_statement(&mut self, st: ReturnStatement) -> (Option<Statement>, bool) {
        let value = st.value.map(|expr| self.optimize_expression(expr).0);
        (
            Some(Statement::Return(ReturnStatement {
                value,
                line: st.line,
                column: st.column,
            })),
            true,
        )
    }

    fn optimize_expression(&mut self, expr: Expression) -> (Expression, Option<ConstValue>) {
        match expr {
            Expression::Number(value) => {
                (Expression::Number(value), Some(ConstValue::Number(value)))
            }
            Expression::String(value) => (
                Expression::String(value.clone()),
                Some(ConstValue::String(value)),
            ),
            Expression::Variable(name) => {
                if let Some(value) = self.get_const(&name) {
                    match value {
                        ConstValue::Number(num) => {
                            return (Expression::Number(*num), Some(value.clone()));
                        }
                        ConstValue::String(text) => {
                            return (Expression::String(text.clone()), Some(value.clone()));
                        }
                        ConstValue::Boolean(_) => {
                            return (Expression::Variable(name), Some(value.clone()));
                        }
                    }
                }
                (Expression::Variable(name), None)
            }
            Expression::Assign(name, value) => {
                let (value_expr, const_value) = self.optimize_expression(*value);
                if let Some(value) = const_value.clone() {
                    self.set_const(&name, value);
                } else {
                    self.clear_const(&name);
                }
                (Expression::Assign(name, Box::new(value_expr)), const_value)
            }
            Expression::Call(name, args) => {
                let mut optimized_args = Vec::with_capacity(args.len());
                for arg in args {
                    optimized_args.push(self.optimize_expression(arg).0);
                }
                (Expression::Call(name, optimized_args), None)
            }
            Expression::Unary(operator, inner) => {
                let (inner_expr, const_value) = self.optimize_expression(*inner);
                if let Some(ConstValue::Number(num)) = const_value
                    && operator == TokenType::MINUS
                {
                    return (Expression::Number(-num), Some(ConstValue::Number(-num)));
                }
                if let Some(value) = const_value
                    && operator == TokenType::EXCL
                {
                    return (
                        Expression::Unary(operator, Box::new(inner_expr)),
                        Some(ConstValue::Boolean(!value.is_truthy())),
                    );
                }

                (Expression::Unary(operator, Box::new(inner_expr)), None)
            }
            Expression::Binary(left, operator, right) => {
                let (left_expr, left_const) = self.optimize_expression(*left);
                let (right_expr, right_const) = self.optimize_expression(*right);

                if let (Some(ConstValue::Number(l)), Some(ConstValue::Number(r))) =
                    (left_const.as_ref(), right_const.as_ref())
                    && let Some((expr, value)) = fold_numeric(*l, operator, *r)
                {
                    return (expr, Some(ConstValue::Number(value)));
                }

                if let (Some(ConstValue::String(l)), Some(ConstValue::String(r))) =
                    (left_const.as_ref(), right_const.as_ref())
                    && operator == TokenType::PLUS
                {
                    let combined = format!("{l}{r}");
                    return (
                        Expression::String(combined.clone()),
                        Some(ConstValue::String(combined)),
                    );
                }

                let const_value =
                    eval_binary_const(operator, left_const.as_ref(), right_const.as_ref());

                (
                    Expression::Binary(Box::new(left_expr), operator, Box::new(right_expr)),
                    const_value,
                )
            }
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

    fn get_const(&self, name: &str) -> Option<&ConstValue> {
        self.scopes.iter().rev().find_map(|scope| scope.get(name))
    }

    fn define_const(&mut self, name: String, value: ConstValue) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, value);
        }
    }

    fn set_const(&mut self, name: &str, value: ConstValue) {
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(name) {
                scope.insert(name.to_string(), value);
                return;
            }
        }
        self.define_const(name.to_string(), value);
    }

    fn clear_const(&mut self, name: &str) {
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(name) {
                scope.remove(name);
                return;
            }
        }
    }
}

fn fold_numeric(left: f64, operator: TokenType, right: f64) -> Option<(Expression, f64)> {
    let value = match operator {
        TokenType::PLUS => left + right,
        TokenType::MINUS => left - right,
        TokenType::STAR => left * right,
        TokenType::SLASH => {
            if right == 0.0 {
                return None;
            }
            left / right
        }
        _ => return None,
    };

    Some((Expression::Number(value), value))
}

fn eval_binary_const(
    operator: TokenType,
    left: Option<&ConstValue>,
    right: Option<&ConstValue>,
) -> Option<ConstValue> {
    match operator {
        TokenType::EQEQ => match (left, right) {
            (Some(l), Some(r)) => Some(ConstValue::Boolean(l == r)),
            _ => None,
        },
        TokenType::NEQ => match (left, right) {
            (Some(l), Some(r)) => Some(ConstValue::Boolean(l != r)),
            _ => None,
        },
        TokenType::GT | TokenType::GTEQ | TokenType::LT | TokenType::LTEQ => match (left, right) {
            (Some(ConstValue::Number(l)), Some(ConstValue::Number(r))) => {
                let result = match operator {
                    TokenType::GT => l > r,
                    TokenType::GTEQ => l >= r,
                    TokenType::LT => l < r,
                    TokenType::LTEQ => l <= r,
                    _ => false,
                };
                Some(ConstValue::Boolean(result))
            }
            _ => None,
        },
        TokenType::AND | TokenType::OR => match (left, right) {
            (Some(l), Some(r)) => {
                let l_val = l.is_truthy();
                let r_val = r.is_truthy();
                let result = if operator == TokenType::AND {
                    l_val && r_val
                } else {
                    l_val || r_val
                };
                Some(ConstValue::Boolean(result))
            }
            _ => None,
        },
        _ => None,
    }
}

fn empty_block(line: usize, column: usize) -> Statement {
    Statement::Block(BlockStatement {
        statements: Vec::new(),
        line,
        column,
    })
}

fn merge_envs(
    base: Vec<HashMap<String, ConstValue>>,
    then_env: Vec<HashMap<String, ConstValue>>,
    else_env: Vec<HashMap<String, ConstValue>>,
) -> Vec<HashMap<String, ConstValue>> {
    let mut merged = Vec::with_capacity(base.len());

    for idx in 0..base.len() {
        let mut scope = HashMap::new();
        let base_scope = &base[idx];
        let then_scope = &then_env[idx];
        let else_scope = &else_env[idx];

        for (name, value) in base_scope {
            if let (Some(t), Some(e)) = (then_scope.get(name), else_scope.get(name))
                && t == e
                && t == value
            {
                scope.insert(name.clone(), value.clone());
            }
        }

        for (name, t) in then_scope {
            if let Some(e) = else_scope.get(name)
                && t == e
                && !base_scope.contains_key(name)
            {
                scope.insert(name.clone(), t.clone());
            }
        }

        merged.push(scope);
    }

    merged
}

fn collect_assigned_names(statement: &Statement, out: &mut HashSet<String>) {
    match statement {
        Statement::Var(st) => {
            out.insert(st.name.clone());
            if let Some(expr) = &st.initializer {
                collect_assigned_in_expr(expr, out);
            }
        }
        Statement::Expression(st) => {
            collect_assigned_in_expr(&st.expression, out);
        }
        Statement::Print(st) => {
            collect_assigned_in_expr(&st.expression, out);
        }
        Statement::Return(st) => {
            if let Some(expr) = &st.value {
                collect_assigned_in_expr(expr, out);
            }
        }
        Statement::Block(st) => {
            for nested in &st.statements {
                collect_assigned_names(nested, out);
            }
        }
        Statement::If(st) => {
            collect_assigned_in_expr(&st.condition, out);
            collect_assigned_names(&st.then_branch, out);
            if let Some(else_branch) = &st.else_branch {
                collect_assigned_names(else_branch, out);
            }
        }
        Statement::While(st) => {
            collect_assigned_in_expr(&st.condition, out);
            collect_assigned_names(&st.body, out);
        }
        Statement::Function(st) => {
            for nested in &st.body.statements {
                collect_assigned_names(nested, out);
            }
        }
    }
}

fn collect_assigned_in_expr(expr: &Expression, out: &mut HashSet<String>) {
    match expr {
        Expression::Assign(name, value) => {
            out.insert(name.clone());
            collect_assigned_in_expr(value, out);
        }
        Expression::Binary(left, _, right) => {
            collect_assigned_in_expr(left, out);
            collect_assigned_in_expr(right, out);
        }
        Expression::Unary(_, inner) => collect_assigned_in_expr(inner, out),
        Expression::Call(_, args) => {
            for arg in args {
                collect_assigned_in_expr(arg, out);
            }
        }
        _ => {}
    }
}

fn is_pure_expression(expr: &Expression) -> bool {
    match expr {
        Expression::Assign(_, _) => false,
        Expression::Call(_, _) => false,
        Expression::Binary(left, _, right) => is_pure_expression(left) && is_pure_expression(right),
        Expression::Unary(_, inner) => is_pure_expression(inner),
        _ => true,
    }
}

pub fn optimize_statements(statements: Vec<Statement>) -> Vec<Statement> {
    Optimizer::new().optimize_statements(statements)
}
