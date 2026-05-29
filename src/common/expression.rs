use crate::token::TokenType;

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Number(f64),
    String(String),
    Variable(String),
    ArrayLiteral(Vec<Expression>),
    Index(Box<Expression>, Box<Expression>),
    Binary(Box<Expression>, TokenType, Box<Expression>),
    Unary(TokenType, Box<Expression>),
    Assign(String, Box<Expression>),
    AssignIndex(Box<Expression>, Box<Expression>, Box<Expression>),
    Call(String, Vec<Expression>),
}
