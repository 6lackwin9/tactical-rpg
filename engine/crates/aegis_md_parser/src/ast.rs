use aegis_core::Variant;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GameFile {
    pub header: FileHeader,
    pub properties: Vec<PropertyDef>,
    pub signals: Vec<SignalDef>,
    pub methods: Vec<MethodDef>,
    pub tables: Vec<TableDef>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct FileHeader {
    pub class_name: String,
    pub extends: String,
    pub tags: Vec<String>,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PropertyDef {
    pub name: String,
    pub type_name: String,
    pub default_value: Option<Variant>,
    pub is_readonly: bool,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SignalParam {
    pub name: String,
    pub type_name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SignalDef {
    pub name: String,
    pub params: Vec<SignalParam>,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MethodParam {
    pub name: String,
    pub type_name: String,
    pub default_value: Option<Variant>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MethodDef {
    pub name: String,
    pub params: Vec<MethodParam>,
    pub return_type: String,
    pub is_static: bool,
    pub description: String,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Block {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TableDef {
    pub title: String,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Statement {
    VarDecl {
        name: String,
        type_name: Option<String>,
        init: Option<Expression>,
    },
    Assignment {
        target: Expression,
        value: Expression,
    },
    If {
        condition: Expression,
        then_branch: Block,
        elif_branches: Vec<(Expression, Block)>,
        else_branch: Option<Block>,
    },
    While {
        condition: Expression,
        body: Block,
    },
    ForIn {
        var_name: String,
        iterable: Expression,
        body: Block,
    },
    Return(Option<Expression>),
    Emit {
        signal_name: String,
        args: Vec<Expression>,
    },
    Expr(Expression),
    Comment(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expression {
    Literal(Variant),
    Identifier(String),
    MemberAccess {
        object: Box<Expression>,
        member: String,
    },
    IndexAccess {
        object: Box<Expression>,
        index: Box<Expression>,
    },
    Call {
        callee: Box<Expression>,
        args: Vec<Expression>,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Expression>,
    },
    Binary {
        left: Box<Expression>,
        op: BinaryOp,
        right: Box<Expression>,
    },
    ArrayLiteral(Vec<Expression>),
    DictLiteral(Vec<(String, Expression)>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnaryOp {
    Neg,
    Not,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Neq,
    Lt,
    Lte,
    Gt,
    Gte,
    And,
    Or,
}
