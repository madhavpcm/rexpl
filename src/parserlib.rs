use std::fmt::Debug;

#[derive(Debug, Eq, PartialEq)]
pub enum ASTNodeError {
    TypeError(String),
}
#[derive(Debug, Eq, PartialEq)]
pub enum ASTNodeType {
    Plus,
    Minus,
    Star,
    Slash,

    Equals,

    Read,
    Write,
    Connector,

    Gt,
    Lt,
    Gte,
    Lte,
    Ee,
    Ne,
}

#[derive(Debug, Eq, PartialEq)]
pub enum ASTExprType {
    Int,
    Bool,
    Null,
}

#[derive(Debug, Eq, PartialEq)]
pub enum ASTError {
    TypeError(String),
}

#[derive(Debug, Eq, PartialEq)]
pub enum ASTNode {
    INT(i64),
    VAR(String),
    BinaryNode {
        op: ASTNodeType,
        exprtype: ASTExprType,
        lhs: Box<ASTNode>,
        rhs: Box<ASTNode>,
    },
    UnaryNode {
        op: ASTNodeType,
        ptr: Box<ASTNode>,
    },
    IfNode {
        expr: Box<ASTNode>,
        xif: Box<ASTNode>,
    },
    IfElseNode {
        expr: Box<ASTNode>,
        xif: Box<ASTNode>,
        xelse: Box<ASTNode>,
    },
    WhileNode {
        expr: Box<ASTNode>,
        xdo: Box<ASTNode>,
    },
    ErrorNode {
        err: ASTError,
    },
    BreakNode,
    ContinueNode,
    Null(i64),
}
pub fn parse_int(s: &str) -> Result<i64, ()> {
    match s.parse::<i64>() {
        Ok(val) => Ok(val),
        Err(_) => {
            eprintln!("{} cannot be represented as a i64", s);
            Err(())
        }
    }
}

pub fn parse_string(s: &str) -> Result<String, ()> {
    Ok(s.to_owned())
}

pub fn validate_ast_binary_node(
    lhs: &ASTNode,
    rhs: &ASTNode,
    sign: &ASTExprType,
) -> Result<bool, ()> {
    let left: bool = match lhs {
        ASTNode::BinaryNode {
            op: _,
            exprtype,
            lhs: _,
            rhs: _,
        } => {
            if sign == exprtype {
                true
            } else {
                false
            }
        }
        ASTNode::INT(_a) => true,
        ASTNode::VAR(_a) => true,
        _ => false,
    };
    let right: bool = match rhs {
        ASTNode::BinaryNode {
            op: _,
            exprtype,
            lhs: _,
            rhs: _,
        } => {
            if sign == exprtype {
                true
            } else {
                false
            }
        }
        ASTNode::INT(_a) => true,
        ASTNode::VAR(_a) => true,
        _ => false,
    };
    return Ok(right && left);
}

pub fn validate_condition_expression(expr: &ASTNode) -> Result<bool, ()> {
    let result: bool = match expr {
        ASTNode::BinaryNode {
            op: _,
            exprtype,
            lhs: _,
            rhs: _,
        } => match exprtype {
            ASTExprType::Bool => true,
            _ => false,
        },
        ASTNode::INT(_a) => true,
        ASTNode::VAR(_a) => true,
        _ => false,
    };

    return Ok(result);
}
