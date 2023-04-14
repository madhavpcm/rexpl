use crate::parserlib::ASTNodeType;
use std::fmt::Formatter;

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
            _ => {
                write!(f, "Node")
            }
        }
    }
}
