use crate::parser_y::{Node, Operator};
use std::fmt::Formatter;
use std::io::Write;

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
pub fn prefix_tree(root: &Node) {
    match root {
        Node::INT(n) => {
            print!("{} ", n);
            std::io::stdout().flush().expect("flush err");
        }
        Node::BinaryExpr { op, lhs, rhs } => {
            print!("{} ", op);
            std::io::stdout().flush().expect("flush err");
            postfix_tree(lhs);
            prefix_tree(rhs);
        }
    }
}
pub fn postfix_tree(root: &Node) {
    match root {
        Node::INT(n) => {
            print!("{} ", n);
            std::io::stdout().flush().expect("flush err");
        }
        Node::BinaryExpr { op, lhs, rhs } => {
            postfix_tree(lhs);
            postfix_tree(rhs);
            print!("{} ", op);
            std::io::stdout().flush().expect("flush err");
        }
    }
}

pub fn evaluate_ast(root: Node) -> i64 {
    match root {
        Node::INT(n) => n,
        Node::BinaryExpr { op, lhs, rhs } => match op {
            Operator::Plus => evaluate_ast(*lhs) + evaluate_ast(*rhs),
            Operator::Minus => evaluate_ast(*lhs) - evaluate_ast(*rhs),
            Operator::Star => evaluate_ast(*lhs) * evaluate_ast(*rhs),
            Operator::Slash => evaluate_ast(*lhs) / evaluate_ast(*rhs),
        },
    }
}
