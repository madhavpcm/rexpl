use lazy_static::lazy_static; // 1.4.0
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::sync::Mutex;

lazy_static! {
    pub static ref GLOBALSYMBOLTABLE: Mutex<HashMap<String, Variable>> =
        Mutex::new(HashMap::default());
    pub static ref VARID: Mutex<usize> = Mutex::new(0);
}

#[derive(Copy, Debug, Clone)]
pub struct Variable {
    pub vartype: ASTExprType,
    pub varid: usize,
    pub varsize: usize,
}
#[derive(Clone, Debug, Eq, PartialEq)]
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

impl std::fmt::Display for ASTExprType {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            ASTExprType::Int => write!(f, "int_t"),
            ASTExprType::String => write!(f, "str_t"),
            ASTExprType::Bool => write!(f, "bool_t"),
            _ => {
                write!(f, "Node")
            }
        }
    }
}
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum ASTExprType {
    Int,
    String,
    Bool,
    Null,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ASTError {
    TypeError(String),
}
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum VarList {
    Node {
        var: String,
        size: usize,
        next: Box<VarList>,
    },
    Null,
}

#[derive(Clone, Eq, PartialEq)]
pub enum ASTNode {
    INT(i64),
    STR(String),
    VAR {
        name: String,
        index1: Box<ASTNode>,
    },
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
    DeclNode {
        var_type: ASTExprType,
        list: Box<VarList>,
    },
    ErrorNode {
        err: ASTError,
    },
    BreakNode,
    ContinueNode,
    Null,
}
/*
 * Convert string to integer
 */
pub fn parse_int(s: &str) -> Result<i64, ()> {
    match s.parse::<i64>() {
        Ok(val) => Ok(val),
        Err(_) => {
            eprintln!("{} cannot be represented as a i64", s);
            Err(())
        }
    }
}

/*
 * Convert string to usize
 */
pub fn parse_usize(s: &str) -> Result<usize, ()> {
    match s.parse::<usize>() {
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

/*
 * Condition to check if an abstract binary node is valid
 */
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
        ASTNode::VAR { name, index1: _ } => {
            let hashmap = GLOBALSYMBOLTABLE.lock().unwrap();
            if let Some(value) = hashmap.get(name.as_str()) {
                log::info!("vartype {}", value.vartype);
                if value.vartype != ASTExprType::Int {
                    false
                } else {
                    true
                }
            } else {
                false
            }
        }
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
        ASTNode::VAR { name, index1: _ } => {
            let hashmap = GLOBALSYMBOLTABLE.lock().unwrap();
            if let Some(value) = hashmap.get(name.as_str()) {
                if value.vartype != ASTExprType::Int {
                    false
                } else {
                    true
                }
            } else {
                false
            }
        }
        _ => false,
    };
    return Ok(right && left);
}
/*
 * Function to check if a condition expression returns boolean
 */
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
        ASTNode::INT(_a) => false,
        ASTNode::VAR { name: _, index1: _ } => false,
        _ => false,
    };

    return Ok(result);
}
/*
 * Function to check if index is valid
 */
pub fn validate_index(expr: &ASTNode) -> Result<bool, ()> {
    let result: bool = match expr {
        ASTNode::BinaryNode {
            op: _,
            exprtype,
            lhs: _,
            rhs: _,
        } => match exprtype {
            ASTExprType::Int => true,
            _ => false,
        },
        ASTNode::INT(_a) => true,
        ASTNode::VAR { name: _, index1: _ } => true,
        _ => false,
    };
    return Ok(result);
}
/*
 * Meta function to map each variable to its type
 *
 * This hash determins the virtual address of the variable in xsm assembly
 *
 * Type checking is also performed with the data generated here
 */
pub fn __gentypehash(declnode: &ASTNode) {
    match declnode {
        ASTNode::DeclNode { var_type, list } => {
            let mut ptr = *list.clone();
            let mut gst = GLOBALSYMBOLTABLE.lock().unwrap();
            let mut var_id = VARID.lock().unwrap();

            loop {
                match ptr {
                    VarList::Node { var, size, next } => {
                        if gst.contains_key(&var) == true {
                            log::error!("Variable : [{}] is already declared", var);
                            std::process::exit(1);
                        }
                        gst.insert(
                            var,
                            Variable {
                                vartype: var_type.clone(),
                                varid: var_id.clone(),
                                varsize: size,
                            },
                        );
                        *var_id = *var_id + size;
                        ptr = *next;
                    }
                    VarList::Null => {
                        break;
                    }
                }
            }
        }
        _ => {
            eprintln!("[parser] Decl Block error");
        }
    };
}
/*
 * Parser debug
 */
pub fn __parse_debug() {
    log::warn!("Im here");
}
