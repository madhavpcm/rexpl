use crate::parser_y::{ASTNode, ASTNodeType};
use std::fmt::Formatter;
use std::io::Write;

impl std::fmt::Display for ASTNodeType {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            ASTNodeType::Plus => write!(f, "+"),
            ASTNodeType::Minus => write!(f, "-"),
            ASTNodeType::Star => write!(f, "*"),
            ASTNodeType::Slash => write!(f, "/"),
            ASTNodeType::Read => write!(f, "Read"),
            ASTNodeType::Write => write!(f, "Write"),
            ASTNodeType::Connector => write!(f, "< >"),
            ASTNodeType::Equals => write!(f, "="),
        }
    }
}
//pub fn prefix_tree(root: &ASTNode) {
//    match root {
//        ASTNode::INT(n) => {
//            print!("{} ", n);
//            std::io::stdout().flush().expect("flush err");
//        }
//        ASTNode::BinaryExpr { op, lhs, rhs } => {
//            print!("{} ", op);
//            std::io::stdout().flush().expect("flush err");
//            postfix_tree(lhs);
//            prefix_tree(rhs);
//        }
//    }
//}
//pub fn postfix_tree(root: &ASTNode) {
//    match root {
//        ASTNode::INT(n) => {
//            print!("{} ", n);
//            std::io::stdout().flush().expect("flush err");
//        }
//        ASTNode::BinaryExpr { op, lhs, rhs } => {
//            postfix_tree(lhs);
//            postfix_tree(rhs);
//            print!("{} ", op);
//            std::io::stdout().flush().expect("flush err");
//        }
//    }
//}
//
//pub fn evaluate_ast(root: ASTNode) -> i64 {
//    match root {
//        ASTNode::INT(n) => n,
//        ASTNode::BinaryExpr { op, lhs, rhs } => match op {
//            ASTNodeType::Plus => evaluate_ast(*lhs) + evaluate_ast(*rhs),
//            ASTNodeType::Minus => evaluate_ast(*lhs) - evaluate_ast(*rhs),
//            ASTNodeType::Star => evaluate_ast(*lhs) * evaluate_ast(*rhs),
//            ASTNodeType::Slash => evaluate_ast(*lhs) / evaluate_ast(*rhs),
//        },
//    }
//}
