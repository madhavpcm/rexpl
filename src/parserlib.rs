use lazy_static::lazy_static; // 1.4.0
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Mutex;

lazy_static! {
    pub static ref GLOBALSYMBOLTABLE: Mutex<HashMap<String, Variable>> =
        Mutex::new(HashMap::default());
    pub static ref VARID: Mutex<usize> = Mutex::new(0);
}

pub struct Variable {
    pub vartype: ASTExprType,
    pub varid: usize,
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

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum ASTExprType {
    Int,
    String,
    Bool,
    Null,
}

#[derive(Debug, Eq, PartialEq)]
pub enum ASTError {
    TypeError(String),
}
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum VarList {
    Node { var: String, next: Box<VarList> },
    Null,
}

#[derive(Debug, Eq, PartialEq)]
pub enum ASTNode {
    INT(i64),
    STR(String),
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
        ASTNode::VAR(var) => {
            let hashmap = GLOBALSYMBOLTABLE.lock().unwrap();
            if let Some(value) = hashmap.get(var.as_str()) {
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
        ASTNode::VAR(var) => {
            let hashmap = GLOBALSYMBOLTABLE.lock().unwrap();
            if let Some(value) = hashmap.get(var.as_str()) {
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
        ASTNode::VAR(_a) => false,
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
                    VarList::Node { var, next } => {
                        if gst.contains_key(&var) == true {
                            log::error!("Variable : [{}] is already declared", var);
                            std::process::exit(1);
                        }
                        gst.insert(
                            var,
                            Variable {
                                vartype: var_type.clone(),
                                varid: var_id.clone(),
                            },
                        );
                        *var_id = *var_id + 1;
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
