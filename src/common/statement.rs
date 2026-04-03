use super::expression::Expression;

#[derive(Debug, Clone)]
pub struct ExpressionStatement {
    pub expression: Expression,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub struct VarStatement {
    pub name: String,
    pub initializer: Option<Expression>,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub struct PrintStatement {
    pub expression: Expression,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub struct BlockStatement {
    pub statements: Vec<Statement>,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub struct IfStatement {
    pub condition: Expression,
    pub then_branch: Box<Statement>,
    pub else_branch: Option<Box<Statement>>,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub struct WhileStatement {
    pub condition: Expression,
    pub body: Box<Statement>,
    pub line: usize,
    pub column: usize,
}

impl PartialEq for ExpressionStatement {
    fn eq(&self, other: &Self) -> bool {
        self.expression == other.expression
    }
}

impl PartialEq for VarStatement {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.initializer == other.initializer
    }
}

impl PartialEq for PrintStatement {
    fn eq(&self, other: &Self) -> bool {
        self.expression == other.expression
    }
}

impl PartialEq for BlockStatement {
    fn eq(&self, other: &Self) -> bool {
        self.statements == other.statements
    }
}

impl PartialEq for IfStatement {
    fn eq(&self, other: &Self) -> bool {
        self.condition == other.condition
            && self.then_branch == other.then_branch
            && self.else_branch == other.else_branch
    }
}

impl PartialEq for WhileStatement {
    fn eq(&self, other: &Self) -> bool {
        self.condition == other.condition && self.body == other.body
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Expression(ExpressionStatement),
    Var(VarStatement),
    Print(PrintStatement),
    Block(BlockStatement),
    If(IfStatement),
    While(WhileStatement),
}
