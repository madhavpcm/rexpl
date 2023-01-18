use lazy_static::lazy_static; // 1.4.0
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::sync::Mutex;

lazy_static! {
    pub static ref GLOBALSYMBOLTABLE: Mutex<HashMap<String, Variable>> =
        Mutex::new(HashMap::default());
    pub static ref VARID: Mutex<usize> = Mutex::new(0);
}

#[derive(Debug, Clone)]
pub struct Variable {
    pub vartype: ASTExprType,
    pub varid: usize,
    pub varindices: Vec<usize>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ASTNodeType {
    Plus,
    Minus,
    Star,
    Slash,
    Mod,

    Equals,

    Read,
    Write,
    Connector,

    Ref,
    Deref,

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
            ASTExprType::IntRef => write!(f, "intptr_t"),
            ASTExprType::StringRef => write!(f, "strptr_t"),
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
    IntRef,
    StringRef,
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
        refr: bool,
        indices: Vec<usize>,
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
        indices: Vec<Box<ASTNode>>,
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
            if exprtype == &ASTExprType::Int {
                true
            } else if sign == exprtype {
                true
            } else {
                false
            }
        }
        ASTNode::INT(_a) => true,
        ASTNode::VAR { name, indices: _ } => {
            let hashmap = GLOBALSYMBOLTABLE.lock().unwrap();
            if let Some(value) = hashmap.get(name.as_str()) {
                if &value.vartype == &ASTExprType::String {
                    false
                } else {
                    true
                }
            } else {
                false
            }
        }
        ASTNode::UnaryNode { op, ptr } => match *op {
            ASTNodeType::Ref => {
                let name = match &**ptr {
                    ASTNode::VAR { name, indices: _ } => name.clone(),
                    _ => "".to_owned(),
                };
                log::info!("{}", name);
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
        },
        _ => false,
    };
    let right: bool = match rhs {
        ASTNode::BinaryNode {
            op: _,
            exprtype,
            lhs: _,
            rhs: _,
        } => {
            if exprtype == &ASTExprType::Int {
                true
            } else if sign == exprtype {
                true
            } else {
                false
            }
        }
        ASTNode::INT(_a) => true,
        ASTNode::STR(_a) => true,
        ASTNode::UnaryNode { op, ptr } => match *op {
            ASTNodeType::Ref => {
                let name = match &**ptr {
                    ASTNode::VAR { name, indices: _ } => name.clone(),
                    _ => "".to_owned(),
                };
                log::info!("{}", name);
                let hashmap = GLOBALSYMBOLTABLE.lock().unwrap();
                if let Some(value) = hashmap.get(name.as_str()) {
                    if &value.vartype != sign {
                        false
                    } else {
                        true
                    }
                } else {
                    false
                }
            }
            _ => false,
        },
        ASTNode::VAR { name, indices: _ } => {
            let hashmap = GLOBALSYMBOLTABLE.lock().unwrap();
            if let Some(value) = hashmap.get(name.as_str()) {
                if &value.vartype == &ASTExprType::String {
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
        ASTNode::VAR {
            name: _,
            indices: _,
        } => true,
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
                    VarList::Node {
                        var,
                        refr,
                        indices,
                        next,
                    } => {
                        if gst.contains_key(&var) == true {
                            log::error!("Variable : [{}] is already declared", var);
                            std::process::exit(1);
                        }
                        let mut vart = var_type.clone();
                        if refr == true {
                            if vart == ASTExprType::String {
                                vart = ASTExprType::StringRef;
                            } else {
                                vart = ASTExprType::IntRef;
                            }
                        }
                        gst.insert(
                            var,
                            Variable {
                                vartype: vart.clone(),
                                varid: var_id.clone(),
                                varindices: indices.clone(),
                            },
                        );
                        let mut size = 1;
                        for i in indices {
                            size = size * i;
                        }
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
 * Validate whether a variable is type
 */
pub fn validate_var(node: &ASTNode) -> Result<bool, ()> {
    match node {
        ASTNode::VAR { name, indices: _ } => {
            let gst = GLOBALSYMBOLTABLE.lock().unwrap();
            if let Some(vardetails) = gst.get(name) {
                if vardetails.vartype != ASTExprType::Int
                    || vardetails.vartype != ASTExprType::String
                {
                    Ok(false)
                } else {
                    Ok(true)
                }
            } else {
                Ok(false)
            }
        }
        _ => Ok(false),
    }
}
/*
 * Validate whether a variable is reference type
 */
pub fn validate_refr(node: &ASTNode) -> Result<bool, ()> {
    match node {
        ASTNode::VAR { name, indices: _ } => {
            let gst = GLOBALSYMBOLTABLE.lock().unwrap();
            if let Some(vardetails) = gst.get(name) {
                if vardetails.vartype != ASTExprType::IntRef
                    || vardetails.vartype != ASTExprType::StringRef
                {
                    Ok(false)
                } else {
                    Ok(true)
                }
            } else {
                Ok(false)
            }
        }
        _ => Ok(false),
    }
}
/*
 * Parser debug
 */
pub fn __parse_debug() {
    log::warn!("Im here");
}
/*
 * Meta function to get variable type
 */

pub fn getvartype(name: &String) -> Result<ASTExprType, ()> {
    let gst = GLOBALSYMBOLTABLE.lock().unwrap();
    if let Some(value) = gst.get(name) {
        return Ok(value.vartype);
    } else {
        return Ok(ASTExprType::Null);
    }
}

pub fn validate_assg(rhs: &ASTNode, sign: &ASTExprType) -> Result<bool, ()> {
    let result = match rhs {
        ASTNode::BinaryNode {
            op: _,
            exprtype,
            lhs: _,
            rhs: _,
        } => {
            if exprtype != sign {
                false
            } else {
                true
            }
        }
        ASTNode::INT(_a) => {
            if sign == &ASTExprType::Int {
                true
            } else {
                false
            }
        }
        ASTNode::STR(_a) => {
            if sign == &ASTExprType::String {
                true
            } else {
                false
            }
        }
        ASTNode::VAR { name, indices: _ } => {
            let t = getvartype(name)?;
            if &t != sign {
                false
            } else {
                true
            }
        }
        ASTNode::UnaryNode { op, ptr } => match *op {
            ASTNodeType::Deref => {
                if let ASTNode::VAR { name, indices: _ } = &**ptr {
                    let t = getvartype(name)?;
                    if sign == &ASTExprType::Int && t == ASTExprType::IntRef
                        || sign == &ASTExprType::String && t == ASTExprType::StringRef
                    {
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            ASTNodeType::Ref => {
                if let ASTNode::VAR { name, indices: _ } = &**ptr {
                    let t = getvartype(name)?;
                    if sign == &ASTExprType::IntRef && t == ASTExprType::Int
                        || sign == &ASTExprType::StringRef && t == ASTExprType::String
                    {
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            _ => false,
        },
        _ => false,
    };
    return Ok(result);
}
