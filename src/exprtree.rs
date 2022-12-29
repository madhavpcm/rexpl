use crate::parser_y::{Node, Operator};
use std::fmt::Formatter;
use std::io::{stderr, Read, Write};

impl std::fmt::Display for Operator {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Operator::Plus => write!(f, "+"),
            Operator::Minus => write!(f, "-"),
            Operator::Star => write!(f, "*"),
            Operator::Slash => write!(f, "/"),
        }
    }
}
pub fn prefixTree(root: &Node) {
    match root {
        Node::INT(n) => {
            print!("{} ", n);
            std::io::stdout().flush();
        }
        Node::BinaryExpr { op, lhs, rhs } => {
            print!("{} ", op);
            std::io::stdout().flush();
            prefixTree(lhs);
            prefixTree(rhs);
        }
        Node::UnaryExpr { op, child } => todo!(),
    }
}
pub fn postfixTree(root: &Node) {
    match root {
        Node::INT(n) => {
            print!("{} ", n);
            std::io::stdout().flush();
        }
        Node::BinaryExpr { op, lhs, rhs } => {
            postfixTree(lhs);
            postfixTree(rhs);
            print!("{} ", op);
            std::io::stdout().flush();
        }
        Node::UnaryExpr { op, child } => todo!(),
    }
}

pub fn evaluateAST(root: Node) -> i64 {
    match root {
        Node::INT(n) => n,
        Node::BinaryExpr { op, lhs, rhs } => match op {
            Operator::Plus => evaluateAST(*lhs) + evaluateAST(*rhs),
            Operator::Minus => evaluateAST(*lhs) - evaluateAST(*rhs),
            Operator::Star => evaluateAST(*lhs) * evaluateAST(*rhs),
            Operator::Slash => evaluateAST(*lhs) / evaluateAST(*rhs),
        },
        Node::UnaryExpr { op, child } => todo!(),
    }
}
