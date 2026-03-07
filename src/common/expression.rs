#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ExpressionType {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Expression {
    stype: ExpressionType,
}
