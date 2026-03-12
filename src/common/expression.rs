use crate::token::TokenType;

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Number(f64),
    String(String),
    Variable(String),
    Binary(Box<Expression>, TokenType, Box<Expression>),
    Unary(TokenType, Box<Expression>),
    Assign(String, Box<Expression>),
}
