use super::expression::Expression;

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Expression(Expression),
    Var(String, Option<Expression>),
    Print(Expression),
    Block(Vec<Statement>),
    If(Expression, Box<Statement>, Box<Statement>),
    While(String, Box<Statement>),
}
