use crate::token::TokenType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DataType {
    String,
    Numeric,
    Boolean,
    Unknown,
}

pub const NUMERIC_SUPPORTED_OPERATIONS: [crate::token::TokenType; 8] = [
    TokenType::PLUS,
    TokenType::MINUS,
    TokenType::GT,
    TokenType::GTEQ,
    TokenType::LT,
    TokenType::LTEQ,
    TokenType::STAR,
    TokenType::SLASH,
];
