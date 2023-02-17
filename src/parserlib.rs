use lazy_static::lazy_static; // 1.4.0
use std::collections::HashMap;
use std::collections::LinkedList;
use std::fmt::{Debug, Formatter};
use std::sync::Mutex;

use crate::codegen::LABEL_COUNT;
use crate::validation::validate_locality;

lazy_static! {
    pub static ref FUNCTION_TABLE: Mutex<HashMap<String, HashMap<String, LSymbol>>> =
        Mutex::new(HashMap::default());
    pub static ref GLOBALSYMBOLTABLE: Mutex<HashMap<String, GSymbol>> =
        Mutex::new(HashMap::default());
    pub static ref LOCALVARID: Mutex<i64> = Mutex::new(1);
    pub static ref VARID: Mutex<usize> = Mutex::new(0);
    pub static ref LOCALSYMBOLTABLE: Mutex<HashMap<String, LSymbol>> =
        Mutex::new(HashMap::default());
    pub static ref CURR_TYPE: Mutex<ASTExprType> = Mutex::new(ASTExprType::Null);
}
#[derive(Debug, Clone)]
pub enum GSymbol {
    Func {
        ret_type: ASTExprType,
        paramlist: LinkedList<Param>,
        flabel: usize,
    },
    Var {
        vartype: ASTExprType,
        varid: usize,
        varindices: Vec<usize>,
    },
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LSymbol {
    Var {
        vartype: ASTExprType,
        varid: i64,
        varindices: Vec<usize>,
    },
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

impl ASTExprType {
    fn refrtype(&self) -> Result<Self, &'static str> {
        match self {
            ASTExprType::String => Ok(ASTExprType::StringRef),
            ASTExprType::Int => Ok(ASTExprType::IntRef),
            _ => Err("Cannot refr to this type"),
        }
    }
    fn derefrtype(&self) -> Result<Self, &'static str> {
        match self {
            ASTExprType::StringRef => Ok(ASTExprType::String),
            ASTExprType::IntRef => Ok(ASTExprType::Int),
            _ => Err("Cannot derefr to this type"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ASTError {
    TypeError(String),
}
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Param {
    pub var: String,
    pub vartype: ASTExprType,
    pub indices: Vec<usize>,
}
impl From<Param> for LinkedList<Param> {
    fn from(param: Param) -> Self {
        let mut list = LinkedList::new();
        list.push_back(param);
        list
    }
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
        paramlist: Box<LinkedList<Param>>,
    },
    FuncDefNode {
        fname: String,
        ret_type: ASTExprType,
        paramlist: Box<LinkedList<Param>>,
        decl: Box<LinkedList<ASTNode>>,
        body: Box<ASTNode>,
    },
    FuncCallNode {
        fname: String,
        arglist: Box<LinkedList<ASTNode>>,
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
    BreakpointNode,
    ContinueNode,
    Null,
}
impl From<ASTNode> for LinkedList<ASTNode> {
    fn from(node: ASTNode) -> Self {
        let mut linkedlist = LinkedList::new();
        linkedlist.push_back(node);
        linkedlist
        // convert the parserlib::ASTNode to your ASTNode type and return a LinkedList
    }
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
    };
    return vartype;
}
/*
 * Function to insert parameter list to local symbol table
 */
pub fn __lst_install_params(paramlist: &LinkedList<Param>) {
    //Check if this variable is in Global Symbol Table
    let mut localid = -3;
    for param in paramlist {
        validate_locality(param.var.clone());
        let mut lst = LOCALSYMBOLTABLE.lock().unwrap();
        lst.insert(
            param.var.clone(),
            LSymbol::Var {
                vartype: (param.vartype),
                varid: (localid),
                varindices: (param.indices.clone()),
            },
        );
        let mut siz = 1;
        for i in &param.indices {
            siz *= i;
        }
        localid -= i64::try_from(siz).unwrap();
    }
}
/*
 * Function to insert declared variabled to local symbol table
 */
pub fn __lst_install_variables(vtype: &ASTExprType, l: &VarList) {
    //Check if this variable is in Global Symbol Table
    let mut ptr = l;
    let mut localid = LOCALVARID.lock().unwrap();
    loop {
        match ptr {
            VarList::Node {
                var,
                refr,
                indices,
                next,
            } => {
                validate_locality(var.clone());
                let mut lst = LOCALSYMBOLTABLE.lock().unwrap();
                let itype;
                if refr == &true {
                    itype = vtype.refrtype().unwrap();
                } else {
                    itype = vtype.clone();
                }
                lst.insert(
                    var.clone(),
                    LSymbol::Var {
                        vartype: (itype),
                        varid: (*localid),
                        varindices: (indices.clone()),
                    },
                );
                let mut siz = 1;
                for i in indices {
                    siz *= i;
                }
                *localid += i64::try_from(siz).unwrap();
                ptr = &**next;
            }
            VarList::Null => {
                break;
            }
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
            let mut label_count = LABEL_COUNT.lock().unwrap();
            let l = label_count.clone();
            *label_count += 1;

            gst.insert(fname.to_string(), {
                GSymbol::Func {
                    ret_type: ret.clone(),
                    paramlist: *plist.clone(),
                    flabel: l,
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
