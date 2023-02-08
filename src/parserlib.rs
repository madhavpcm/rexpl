use lazy_static::lazy_static; // 1.4.0
use std::collections::HashMap;
use std::collections::LinkedList;
use std::fmt::{Debug, Formatter};
use std::sync::Mutex;

use crate::codegen::exit_on_err;
use crate::codegen::FUNCTION_STACK;
use crate::codegen::SCOPE_STACK;

lazy_static! {
    pub static ref FUNCTION_TABLE: Mutex<HashMap<String, HashMap<String, LSymbol>>> =
        Mutex::new(HashMap::default());
    pub static ref GLOBALSYMBOLTABLE: Mutex<HashMap<String, GSymbol>> =
        Mutex::new(HashMap::default());
    pub static ref VARID: Mutex<usize> = Mutex::new(0);
}
#[derive(Debug, Clone)]
pub enum GSymbol {
    Func {
        ret_type: ASTExprType,
        paramlist: Box<ParamList>,
        flabel: usize,
    },
    Var {
        vartype: ASTExprType,
        varid: usize,
        varindices: Vec<usize>,
    },
    Null,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LSymbol {
    Var {
        vartype: ASTExprType,
        varid: i64,
        varindices: Vec<usize>,
    },
    Null,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ASTNodeType {
    //Operators
    Plus,
    Minus,
    Star,
    Slash,
    Mod,
    //Assignment
    Equals,
    //IO
    Read,
    Write,
    //Connector/Blank Node
    Connector,
    //Pointers
    Ref,
    Deref,
    //Logical Operators
    Gt,
    Lt,
    Gte,
    Lte,
    Ee,
    Ne,
}
//Overload for printing exprtype
impl std::fmt::Display for ASTExprType {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            ASTExprType::Int => write!(f, "int_t"),
            ASTExprType::String => write!(f, "str_t"),
            ASTExprType::Bool => write!(f, "bool_t"),
            ASTExprType::IntRef => write!(f, "intptr_t"),
            ASTExprType::StringRef => write!(f, "strptr_t"),
            _ => {
                write!(f, "null_t")
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
pub enum ParamList {
    Node {
        var: String,
        vartype: ASTExprType,
        indices: Vec<usize>,
        next: Box<ParamList>,
    },
    Null,
}
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum ArgList {
    Node { expr: ASTNode, next: Box<ArgList> },
    Null,
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

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ASTNode {
    INT(i64),
    STR(String),
    VAR {
        name: String,
        indices: Vec<Box<ASTNode>>,
    },
    BinaryNode {
        op: ASTNodeType,
        exprtype: Option<ASTExprType>,
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
    FuncDeclNode {
        fname: String,
        ret_type: ASTExprType,
        paramlist: Box<ParamList>,
    },
    FuncDefNode {
        fname: String,
        ret_type: ASTExprType,
        paramlist: Box<ParamList>,
        decl: Box<LinkedList<ASTNode>>,
        body: Box<ASTNode>,
    },
    FuncCallNode {
        fname: String,
        arglist: Box<ArgList>,
    },
    ErrorNode {
        err: ASTError,
    },
    ReturnNode {
        expr: Box<ASTNode>,
    },
    MainNode {
        decl: Box<LinkedList<ASTNode>>,
        body: Box<ASTNode>,
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
 * Meta function
 * Get the type of a Global Symbol
 */
pub fn __get_gsymbol_type(g: &GSymbol) -> &ASTExprType {
    let vartype = match g {
        GSymbol::Func {
            ret_type,
            paramlist: _,
            flabel: _,
        } => ret_type,
        GSymbol::Var {
            vartype,
            varid: _,
            varindices: _,
        } => vartype,
        GSymbol::Null => &ASTExprType::Null,
    };
    return vartype;
}
/*
 * Meta function
 * Get the type of a local symbol
 */
pub fn __get_lsymbol_type(l: &LSymbol) -> &ASTExprType {
    let vartype = match l {
        LSymbol::Var {
            vartype,
            varid: _,
            varindices: _,
        } => vartype,
        LSymbol::Null => &ASTExprType::Null,
    };
    return vartype;
}
/*64
 * Function to check if a condition expression returns boolean
 */
pub fn __gen_local_symbol_table(decllist: &LinkedList<ASTNode>, paramlist: &ParamList) {
    let mut ss = SCOPE_STACK.lock().unwrap();
    let local_table = ss.last_mut().unwrap();

    let gst = GLOBALSYMBOLTABLE.lock().unwrap();

    let mut local_var_id: i64 = 1;
    let mut param_offset: i64 = -3;

    let mut paramptr = paramlist;
    loop {
        match paramptr {
            ParamList::Node {
                var,
                vartype,
                indices,
                next,
            } => {
                if gst.contains_key(var) == true {
                    if let Some(entry) = gst.get(var) {
                        match entry {
                            GSymbol::Func {
                                ret_type: _,
                                paramlist: _,
                                flabel: _,
                            } => {
                                exit_on_err(
                                    "Argument name with ".to_owned()
                                        + var.as_str()
                                        + " is already declared as a function",
                                );
                            }
                            GSymbol::Var {
                                vartype: _,
                                varid: _,
                                varindices: _,
                            } => {
                                log::warn!(
                                    "Argument {} is already declared as a variable in global scope",
                                    var
                                );
                            }
                            GSymbol::Null => exit_on_err("GST error".to_owned()),
                        }
                    }
                    local_table.insert(
                        var.clone(),
                        LSymbol::Var {
                            vartype: vartype.clone(),
                            varid: param_offset.clone(),
                            varindices: indices.clone(),
                        },
                    );
                    let mut size = 1;
                    for i in indices {
                        size = size * i;
                    }
                    param_offset = param_offset - i64::try_from(size).unwrap();
                    paramptr = next;
                }
            }
            ParamList::Null => {
                break;
            }
        };
    }

    for i in decllist.iter() {
        match i {
            ASTNode::DeclNode { var_type, list } => {
                let mut ptr = *list.clone();

                loop {
                    match ptr {
                        VarList::Node {
                            var,
                            refr,
                            indices,
                            next,
                        } => {
                            if gst.contains_key(&var) == true {
                                if let Some(entry) = gst.get(&var) {
                                    match entry {
                                        GSymbol::Func {
                                            ret_type: _,
                                            paramlist: _,
                                            flabel: _,
                                        } => {
                                            exit_on_err(
                                                "Variable ".to_owned()
                                                    + var.as_str()
                                                    + " is already declared as a function",
                                            );
                                        }
                                        GSymbol::Var {
                                            vartype: _,
                                            varid: _,
                                            varindices: _,
                                        } => {
                                            log::warn!("Variable {} is already declared as a variable in global scope", var);
                                        }
                                        GSymbol::Null => exit_on_err("GST error".to_owned()),
                                    }
                                }
                            }
                            let mut vart = var_type.clone();
                            if refr == true {
                                if vart == ASTExprType::String {
                                    vart = ASTExprType::StringRef;
                                } else {
                                    vart = ASTExprType::IntRef;
                                }
                            }
                            local_table.insert(
                                var,
                                LSymbol::Var {
                                    vartype: vart.clone(),
                                    varid: local_var_id,
                                    varindices: indices.clone(),
                                },
                            );
                            let mut size = 1;
                            for i in indices {
                                size = size * i;
                            }
                            local_var_id = local_var_id + i64::try_from(size).unwrap();
                            ptr = *next;
                        }
                        VarList::Null => {
                            break;
                        }
                    }
                }
            }
            _ => exit_on_err("Invalid declaration in function ".to_owned()),
        }
    }
}
/*
 * Meta function to map each variable to its type
 *
 * This hash determins the virtual address of the variable in xsm assembly
 *
 * Type checking is also performed with the data generated here
 */
pub fn __gen_global_symbol_table(declnode: &ASTNode) {
    match declnode {
        ASTNode::FuncDeclNode {
            fname,
            ret_type: ret,
            paramlist: plist,
        } => {
            let mut gst = GLOBALSYMBOLTABLE.lock().unwrap();

            gst.insert(fname.to_string(), {
                GSymbol::Func {
                    ret_type: ret.clone(),
                    paramlist: plist.clone(),
                    flabel: 0,
                }
            });
        }
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
                            log::error!(
                                "Variable name: [{}] is already declared as a variable or function",
                                var
                            );
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
                            GSymbol::Var {
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
pub fn __parse_debug() {
    log::warn!("Im here");
}
