use lazy_static::lazy_static; // 1.4.0
use std::collections::LinkedList;
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Formatter};
use std::sync::Mutex;

use crate::codegen::exit_on_err;
use crate::codegen::LABEL_COUNT;

lazy_static! {
    pub static ref TYPE_TABLE: Mutex<TypeTable> = Mutex::new(TypeTable::default());
    pub static ref FUNCTION_TABLE: Mutex<HashMap<String, HashMap<String, LSymbol>>> =
        Mutex::new(HashMap::default());
    pub static ref GLOBALSYMBOLTABLE: Mutex<HashMap<String, GSymbol>> =
        Mutex::new(GlobalSymbolTable::default().table);
    pub static ref LOCALVARID: Mutex<i64> = Mutex::new(1);
    pub static ref VARID: Mutex<usize> = Mutex::new(0);
    pub static ref LOCALSYMBOLTABLE: Mutex<HashMap<String, LSymbol>> =
        Mutex::new(HashMap::default());
    pub static ref RET_TYPE: Mutex<ASTExprType> =
        Mutex::new(ASTExprType::Primitive(PrimitiveType::Null));
    pub static ref DECL_TYPE: Mutex<ASTExprType> =
        Mutex::new(ASTExprType::Primitive(PrimitiveType::Null));
    pub static ref INITFLAG: Mutex<bool> = Mutex::new(false);
}
pub struct TypeTable {
    pub table: HashMap<String, ASTExprType>,
}
pub struct GlobalSymbolTable {
    pub table: HashMap<String, GSymbol>,
}
impl Default for GlobalSymbolTable {
    fn default() -> GlobalSymbolTable {
        let mut table: HashMap<String, GSymbol> = HashMap::default();
        table.insert(
            "write".to_owned(),
            GSymbol::Func {
                ret_type: ASTExprType::Primitive(PrimitiveType::Int),
                paramlist: (LinkedList::from(VarNode {
                    varname: "var".to_owned(),
                    vartype: ASTExprType::Primitive(PrimitiveType::Int),
                    varindices: vec![],
                })),
                flabel: (0),
            },
        );
        table.insert(
            "read".to_owned(),
            GSymbol::Func {
                ret_type: ASTExprType::Primitive(PrimitiveType::Int),
                paramlist: (LinkedList::from(VarNode {
                    varname: "var".to_owned(),
                    vartype: ASTExprType::Primitive(PrimitiveType::Int),
                    varindices: vec![],
                })),
                flabel: (0),
            },
        );
        table.insert(
            "free".to_owned(),
            GSymbol::Func {
                ret_type: ASTExprType::Pointer(Box::new(ASTExprType::Primitive(
                    PrimitiveType::Void,
                ))),
                paramlist: (LinkedList::from(VarNode {
                    varname: "ptr".to_owned(),
                    vartype: ASTExprType::Primitive(PrimitiveType::Int),
                    varindices: vec![],
                })),
                flabel: (0),
            },
        );
        table.insert(
            "initialize".to_owned(),
            GSymbol::Func {
                ret_type: ASTExprType::Primitive(PrimitiveType::Void),
                paramlist: (LinkedList::default()),
                flabel: (0),
            },
        );
        table.insert(
            "syscall".to_owned(),
            GSymbol::Func {
                ret_type: ASTExprType::Primitive(PrimitiveType::Int),
                paramlist: (LinkedList::default()),
                flabel: (0),
            },
        );
        table.insert(
            "alloc".to_owned(),
            GSymbol::Func {
                ret_type: ASTExprType::Pointer(Box::new(ASTExprType::Primitive(
                    PrimitiveType::Void,
                ))),
                paramlist: (LinkedList::default()),
                flabel: (0),
            },
        );
        GlobalSymbolTable { table: (table) }
    }
}
impl Default for TypeTable {
    fn default() -> TypeTable {
        let mut table: HashMap<String, ASTExprType> = HashMap::default();
        table.insert("int".to_owned(), ASTExprType::Primitive(PrimitiveType::Int));
        table.insert(
            "str".to_owned(),
            ASTExprType::Primitive(PrimitiveType::String),
        );
        TypeTable { table: (table) }
    }
}
impl TypeTable {
    pub fn tt_get_type(&self, tname: &String) -> Result<ASTExprType, String> {
        let table = &self.table;
        if let Some(entry) = table.get(tname) {
            Ok(entry.clone())
        } else {
            Err("Type [".to_owned() + tname.as_str() + "] is not declared/valid.")
        }
    }
    pub fn tt_exists(&self, tname: &String) -> bool {
        let table = &self.table;
        if let Some(_) = table.get(tname) {
            true
        } else {
            false
        }
    }
    pub fn tinstall(&mut self, tname: String, tfields: LinkedList<Field>) -> Result<(), String> {
        let map = &mut self.table;
        if map.contains_key(&tname) {
            return Err("Type [".to_owned() + &tname + "] is already declared.");
        }
        if tfields.len() > 8 {
            return Err("Type [".to_owned() + &tname + "] has more than 8 fields.");
        }
        let mut fieldcheck: HashSet<String> = HashSet::new();
        map.insert(tname.clone(), ASTExprType::Error);

        for i in tfields.iter() {
            //validate the type
            match &i.field_type {
                FieldType::Primitive(_) => {}
                FieldType::Pointer(p) => {
                    let base = p.get_base_type();
                    match base {
                        FieldType::Primitive(_) => {}
                        FieldType::Struct(s) => {
                            if map.contains_key(&s) == false {
                                return Err("Type [".to_owned() + &s + "] is not declared.");
                            }
                        }
                        _ => {
                            return Err("Some error".to_owned());
                        }
                    }
                }
                FieldType::Struct(s) => {
                    //We can choose to disallow this
                    if *s == tname {
                        return Err("Type [".to_owned() + &s + "] is incomplete.");
                    } else {
                        if map.contains_key(s) == false {
                            return Err("Type [".to_owned() + &s + "] is not declared.");
                        }
                    }
                }
            };
            if fieldcheck.contains(&i.name) {
                return Err("In Type [".to_owned()
                    + &tname
                    + "], field ["
                    + &i.name
                    + "] is declared more than once.");
            }
            fieldcheck.insert(i.name.clone());
        }
        map.insert(
            tname.clone(),
            ASTExprType::Struct(ASTStructType {
                name: tname.clone(),
                size: tfields.len(),
                fields: tfields,
            }),
        );
        Ok(())
    }
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
    //Heap
    Alloc,
    Free,
    Initialize,
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
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum STDLibFunction {
    Heapset,
    Alloc,
    Free,
    Read,
    Write,
    Syscall,
    Setaddr,
    Getaddr,
}
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum PrimitiveType {
    Int,
    String,
    Bool,
    Void,
    Null,
}
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum FieldType {
    Primitive(PrimitiveType),
    Pointer(Box<FieldType>),
    Struct(String),
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Field {
    pub name: String,
    pub field_type: FieldType,
    pub array_access: Vec<Box<Field>>,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct ASTStructType {
    pub name: String,
    pub size: usize,
    pub fields: LinkedList<Field>,
}
// an expression could be a primitive type or a pointer to a primitive type or so on..
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum ASTExprType {
    Primitive(PrimitiveType),
    Pointer(Box<ASTExprType>),
    Struct(ASTStructType),
    Error,
}
impl FieldType {
    pub fn as_astexprtype(&self) -> Result<ASTExprType, String> {
        match self {
            FieldType::Primitive(p) => Ok(ASTExprType::Primitive(p.clone())),
            FieldType::Pointer(p) => Ok(ASTExprType::Pointer(Box::new((&**p).as_astexprtype()?))),
            FieldType::Struct(p) => Ok(TYPE_TABLE.lock().unwrap().tt_get_type(p)?),
        }
    }
}
impl ASTExprType {
    pub fn size(&self) -> Result<usize, String> {
        match self {
            ASTExprType::Primitive(_) => Ok(1),
            ASTExprType::Pointer(_) => Ok(1),
            ASTExprType::Struct(s) => Ok(s.size),
            ASTExprType::Error => {
                unreachable!()
            }
        }
    }
    pub fn get_field_type(&self, fname: &String) -> Result<ASTExprType, String> {
        match self {
            ASTExprType::Struct(s) => {
                for i in s.fields.iter() {
                    if &i.name == fname {
                        return i.field_type.as_astexprtype();
                    }
                }
                Err("Field [".to_owned()
                    + fname.as_str()
                    + "] not declared inside type ["
                    + fname.as_str()
                    + "]")
            }
            _ => Err("Expression of this type cannot be accessed.".to_owned()),
        }
    }
    pub fn get_field_id(&self, fname: &String) -> Result<usize, String> {
        match self {
            ASTExprType::Struct(s) => {
                let mut len = 1;
                for i in s.fields.iter() {
                    if &i.name == fname {
                        return Ok(len);
                    }
                    len += 1;
                }
                Err("Field [".to_owned()
                    + fname.as_str()
                    + "] not declared inside type ["
                    + fname.as_str()
                    + "]")
            }
            _ => Err("Expression of this type cannot be accessed.".to_owned()),
        }
    }
    pub fn refr(&self) -> Option<ASTExprType> {
        match self {
            ASTExprType::Error => None,
            _ => Some(ASTExprType::Pointer(Box::new(self.clone()))),
        }
    }
    pub fn derefr(&self) -> Option<ASTExprType> {
        match self {
            ASTExprType::Primitive(_) => None,
            ASTExprType::Pointer(p) => Some((**p).clone()),
            ASTExprType::Struct { .. } => None,
            ASTExprType::Error => None,
        }
    }
    pub fn set_base_type(&mut self, p: ASTExprType) {
        match self {
            ASTExprType::Primitive(_) => *self = p,
            ASTExprType::Pointer(b) => {
                b.set_base_type(p);
            }
            ASTExprType::Struct { .. } => *self = p,
            ASTExprType::Error => {}
        }
    }
    pub fn get_base_type(&self) -> ASTExprType {
        match self {
            ASTExprType::Primitive(_) => self.clone(),
            ASTExprType::Pointer(p) => Self::get_base_type(p),
            ASTExprType::Struct { .. } => self.clone(),
            ASTExprType::Error => ASTExprType::Primitive(PrimitiveType::Void),
        }
    }
    pub fn depth(&self) -> usize {
        match self {
            ASTExprType::Pointer(p) => Self::depth(p) + 1,
            ASTExprType::Primitive(_) => 0,
            ASTExprType::Struct { .. } => 0,
            ASTExprType::Error => 0,
        }
    }
}

impl FieldType {
    pub fn set_base_type(&mut self, p: FieldType) {
        match self {
            FieldType::Primitive(_) => *self = p,
            FieldType::Pointer(b) => {
                b.set_base_type(p);
            }
            FieldType::Struct { .. } => *self = p,
        }
    }
    pub fn get_base_type(&self) -> FieldType {
        match self {
            FieldType::Primitive(_) => self.clone(),
            FieldType::Pointer(p) => Self::get_base_type(p),
            FieldType::Struct(_) => self.clone(),
        }
    }
    pub fn depth(&self) -> usize {
        match self {
            FieldType::Pointer(p) => Self::depth(p) + 1,
            FieldType::Primitive(_) => 0,
            FieldType::Struct(_) => 0,
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

impl From<Field> for LinkedList<Field> {
    fn from(param: Field) -> Self {
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
    pub fn validate_locality(&mut self) {
        let lst = LOCALSYMBOLTABLE.lock().unwrap();
        let gst = GLOBALSYMBOLTABLE.lock().unwrap();
        if let Some(entry) = gst.get(&self.varname) {
            match entry {
                GSymbol::Func {
                    ret_type: _,
                    paramlist: _,
                    flabel: _,
                } => {
                    //exit if a function with similar name exists
                    exit_on_err(
                        "Parameter Symbol ".to_owned()
                            + &self.varname.as_str()
                            + " is already declared as a function",
                    );
                }
                GSymbol::Var {
                    vartype: _,
                    varid: _,
                    varindices: _,
                } => {
                    //Shadow global variable after warning user
                    log::warn!(
                        "Parameter Symbol {} is already declared as a variable in global scope",
                        &self.varname.as_str()
                    );
                }
            }
        }
        if lst.contains_key(&self.varname) == true {
            exit_on_err(
                "Parameter Symbol ".to_owned() + &self.varname.as_str() + " is already declared ",
            );
        }
    }
    pub fn install_to_lst(&mut self) {
        //check if this is already used
        Self::validate_locality(self);
        let mut lst = LOCALSYMBOLTABLE.lock().unwrap();
        let mut varid = LOCALVARID.lock().unwrap();
        if TYPE_TABLE.lock().unwrap().tt_exists(&self.varname) == true {
            exit_on_err(
                "Name [".to_owned()
                    + self.varname.as_str()
                    + "]  exists as a user defined type and cannot be used to declare a local variable.",
            );
        }

        lst.insert(
            self.varname.clone(),
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
        if TYPE_TABLE.lock().unwrap().tt_exists(&self.varname) == true {
            exit_on_err(
                "Name [".to_owned()
                    + self.varname.as_str()
                    + "]  exists as a user defined type and cannot be used to declare a global variable.",
            );
        }
        if gst.contains_key(self.varname.as_str()) {
            exit_on_err(
                "Global symbol [".to_owned() + self.varname.as_str() + "] is already declared.",
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
        let mut size = match &self.vartype {
            ASTExprType::Primitive(_) => 1,
            ASTExprType::Pointer(_) => 1,
            ASTExprType::Struct(s) => s.size.clone(),
            ASTExprType::Error => 0,
        };
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
    let mut label_count = LABEL_COUNT.lock().unwrap();
    //check if this is already  used
    if TYPE_TABLE.lock().unwrap().tt_exists(&funcname) == true {
        exit_on_err(
            "Name [".to_owned()
                + funcname.as_str()
                + "]  exists as a user defined type and cannot be used to declare a function.",
        );
    }
    if gst.contains_key(funcname.as_str()) {
        exit_on_err("Global symbol [".to_owned() + funcname.as_str() + "] is already declared.")
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
        array_access: Vec<Box<ASTNode>>,
        dot_field_access: Box<ASTNode>,
        arrow_field_access: Box<ASTNode>,
    },
    BinaryNode {
        op: ASTNodeType,
        exprtype: Option<ASTExprType>,
        lhs: Box<ASTNode>,
        rhs: Box<ASTNode>,
    },
    UnaryNode {
        op: ASTNodeType,
        exprtype: Option<ASTExprType>,
        ptr: Box<ASTNode>,
        depth: Option<usize>,
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
        paramlist: LinkedList<VarNode>,
    },
    FuncCallNode {
        fname: String,
        arglist: Box<LinkedList<ASTNode>>,
    },
    StdFuncCallNode {
        func: STDLibFunction,
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
    Void,
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
pub fn __lst_install_params(paramlist: &mut LinkedList<VarNode>) -> Result<(), String> {
    //Check if this variable is in Global Symbol Table
    let mut localid = -3;
    for param in paramlist.iter_mut().rev() {
        param.validate_locality();
        let mut lst = LOCALSYMBOLTABLE.lock().unwrap();
        let mut siz = param.vartype.size()?;
        for i in &param.varindices {
            siz *= i;
        }
        lst.insert(
            param.varname.clone(),
            LSymbol::Var {
                vartype: (param.vartype.clone()),
                varid: (localid - (i64::try_from(siz).unwrap() - 1)),
                varindices: (param.varindices.clone()),
            },
        );
        localid -= i64::try_from(siz).unwrap();
    }
    Ok(())
}
pub fn __parse_debug() {
    log::warn!("Im here");
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
impl std::fmt::Display for ASTExprType {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            ASTExprType::Error => write!(f, "ASTExprType :: Error"),
            ASTExprType::Primitive(p) => write!(f, "{}", p),
            ASTExprType::Struct(s) => write!(f, "struct_{}_t", s.name),
            ASTExprType::Pointer(p) => write!(f, "{}{}", "*".repeat(p.depth()), p.get_base_type()),
        }
    }
}
