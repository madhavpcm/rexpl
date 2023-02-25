use lazy_static::lazy_static; // 1.4.0
use std::collections::HashMap;
use std::collections::LinkedList;
use std::fmt::{Debug, Formatter};
use std::sync::Mutex;

use crate::codegen::exit_on_err;
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
    pub static ref CURR_TYPE: Mutex<ASTExprType> =
        Mutex::new(ASTExprType::Primitive(PrimitiveType::Null));
    pub static ref DECL_TYPE: Mutex<ASTExprType> =
        Mutex::new(ASTExprType::Primitive(PrimitiveType::Null));
}
#[derive(Debug, Clone)]
pub enum GSymbol {
    Func {
        ret_type: ASTExprType,
        paramlist: LinkedList<VarNode>,
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
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum PrimitiveType {
    Int,
    String,
    Bool,
    Void,
    Null,
}
//Overload for printing exprtype
impl std::fmt::Display for PrimitiveType {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            PrimitiveType::Int => write!(f, "int_t"),
            PrimitiveType::String => write!(f, "str_t"),
            PrimitiveType::Bool => write!(f, "bool_t"),
            PrimitiveType::Void => write!(f, "void_t"),
            _ => {
                write!(f, "null_t")
            }
        }
    }
}

// an expression could be a primitive type or a pointer to a primitive type or so on..
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum ASTExprType {
    Primitive(PrimitiveType),
    Pointer(Box<ASTExprType>),
}

impl ASTExprType {
    //Only used by parser
    pub fn set_base_type(&mut self, p: PrimitiveType) {
        match self {
            ASTExprType::Primitive(t) => {
                *t = p.clone();
            }
            ASTExprType::Pointer(b) => {
                b.set_base_type(p);
            }
        }
    }
    pub fn get_base_type(&self) -> PrimitiveType {
        match self {
            ASTExprType::Primitive(p) => p.clone(),
            ASTExprType::Pointer(p) => Self::get_base_type(p),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ASTError {
    TypeError(String),
}
impl From<VarNode> for LinkedList<VarNode> {
    fn from(param: VarNode) -> Self {
        let mut list = LinkedList::new();
        list.push_back(param);
        list
    }
}
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct VarNode {
    pub varname: String,
    pub vartype: ASTExprType,
    pub varindices: Vec<usize>,
}
impl VarNode {
    pub fn install_to_lst(self) {
        //check if this is already used
        validate_locality(self.varname.clone());
        let mut lst = LOCALSYMBOLTABLE.lock().unwrap();
        let mut varid = LOCALVARID.lock().unwrap();

        lst.insert(
            self.varname,
            LSymbol::Var {
                vartype: (self.vartype.clone()),
                varid: (varid.clone()),
                varindices: (self.varindices.clone()),
            },
        );
        let mut size = 1;
        for i in self.varindices.iter() {
            size *= i;
        }
        *varid += i64::try_from(size).unwrap();
    }
    pub fn install_to_gst(self) {
        let mut gst = GLOBALSYMBOLTABLE.lock().unwrap();
        let mut varid = VARID.lock().unwrap();
        //check if this is already  used
        if gst.contains_key(self.varname.as_str()) {
            exit_on_err(
                "Global symbol + ".to_owned() + self.varname.as_str() + " is already declared.",
            )
        }
        gst.insert(
            self.varname,
            GSymbol::Var {
                vartype: (self.vartype.clone()),
                varid: (varid.clone()),
                varindices: (self.varindices.clone()),
            },
        );
        let mut size = 1;
        for i in self.varindices.iter() {
            size *= i;
        }
        *varid += size;
    }
}

pub fn install_func_to_gst(
    funcname: String,
    returntype: ASTExprType,
    paramlist: &LinkedList<VarNode>,
) {
    let mut gst = GLOBALSYMBOLTABLE.lock().unwrap();
    let mut varid = VARID.lock().unwrap();
    let mut label_count = LABEL_COUNT.lock().unwrap();
    //check if this is already  used
    if gst.contains_key(funcname.as_str()) {
        exit_on_err("Global symbol + ".to_owned() + funcname.as_str() + " is already declared.")
    }
    gst.insert(
        funcname,
        GSymbol::Func {
            ret_type: (returntype.clone()),
            paramlist: (paramlist.clone()),
            flabel: (label_count.clone()),
        },
    );
    *label_count += 1;
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
    FuncDefNode {
        fname: String,
        ret_type: ASTExprType,
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
pub fn __lst_install_params(paramlist: &LinkedList<VarNode>) {
    //Check if this variable is in Global Symbol Table
    let mut localid = -3;
    for param in paramlist {
        validate_locality(param.varname.clone());
        let mut lst = LOCALSYMBOLTABLE.lock().unwrap();
        lst.insert(
            param.varname.clone(),
            LSymbol::Var {
                vartype: (param.vartype),
                varid: (localid),
                varindices: (param.varindices.clone()),
            },
        );
        let mut siz = 1;
        for i in &param.varindices {
            siz *= i;
        }
        localid -= i64::try_from(siz).unwrap();
    }
}
pub fn __parse_debug() {
    log::warn!("Im here");
}
