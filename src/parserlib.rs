use lazy_static::lazy_static; // 1.4.0
use std::collections::LinkedList;
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Formatter};
use std::sync::Mutex;

use crate::codegen::exit_on_err;
use crate::codegen::LABEL_COUNT;
use crate::validation::compare_arglist_paramlist;

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
    pub static ref CLASS_RET_TYPE: Mutex<FieldType> =
        Mutex::new(FieldType::Primitive(PrimitiveType::Null));
    pub static ref RET_TYPE: Mutex<ASTExprType> =
        Mutex::new(ASTExprType::Primitive(PrimitiveType::Null));
    pub static ref DECL_TYPE: Mutex<ASTExprType> =
        Mutex::new(ASTExprType::Primitive(PrimitiveType::Null));
    pub static ref INITFLAG: Mutex<bool> = Mutex::new(false);
    pub static ref CLASSNAME: Mutex<String> = Mutex::new(String::new());
    pub static ref PCLASSNAME: Mutex<String> = Mutex::new(String::new());
    pub static ref VFT_ID: Mutex<usize> = Mutex::new(0);
}
pub struct TypeTable {
    pub table: HashMap<String, ASTExprType>,
}
pub struct GlobalSymbolTable {
    pub table: HashMap<String, GSymbol>,
}
#[derive(Debug, Clone)]
pub struct ClassSymbolTable {
    pub table: HashMap<String, CSymbol>,
}
impl Default for ClassSymbolTable {
    fn default() -> ClassSymbolTable {
        ClassSymbolTable {
            table: (HashMap::default()),
        }
    }
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
impl Iterator for TypeTable {
    type Item = (String, ASTExprType);

    fn next(&mut self) -> Option<Self::Item> {
        self.table
            .iter()
            .next()
            .map(|(k, v)| (k.clone(), v.clone()))
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
    fn validate_field_type(&self, this: &String, t: &FieldType) -> Result<(), String> {
        let map = &self.table;
        match t {
            FieldType::Primitive(_) => Ok(()),
            FieldType::Pointer(p) => {
                let base = p.get_base_type();
                match base {
                    FieldType::Primitive(_) => Ok(()),
                    FieldType::Struct(s) => {
                        if map.contains_key(&s) == false {
                            return Err("Type [".to_owned() + &s + "] is not declared.");
                        }
                        Ok(())
                    }
                    _ => {
                        return Err("Some error".to_owned());
                    }
                }
            }
            FieldType::Struct(s) => {
                //We can choose to disallow this
                if map.contains_key(s) == false {
                    return Err("Type [".to_owned() + &s + "] is not declared.");
                }
                Ok(())
            }
            FieldType::Class(s) => {
                if s == this {
                    return Err("Type [".to_owned() + &s + "] is incomplete.");
                } else {
                    if map.contains_key(s) == false {
                        return Err("Type [".to_owned() + &s + "] is not declared.");
                    }
                    Ok(())
                }
            }
        }
    }
    pub fn tinstall_class_methods(
        &mut self,
        tmethods: &mut LinkedList<CSymbol>,
    ) -> Result<(), String> {
        let tname = &*CLASSNAME.lock().unwrap();
        let classentry = self.tt_get_type(tname)?;
        let map = &mut self.table;
        let mut label_count = LABEL_COUNT.lock().unwrap();
        let mut cstruct;
        if let ASTExprType::Class(c) = classentry {
            cstruct = c;
        } else {
            return Err("not a class?.".to_owned());
        }
        let mut fieldid: i64 = cstruct.methodsize;
        for i in tmethods.iter_mut() {
            match i {
                CSymbol::Func {
                    name,
                    ret_type,
                    paramlist,
                    flabel,
                    fid,
                } => {
                    if let Some(entry) = cstruct.symbol_table.table.get(name) {
                        match entry {
                            CSymbol::Var { .. } => {
                                return Err("In class [".to_owned()
                                    + tname
                                    + "], Method ["
                                    + &name
                                    + "] is already declared as field");
                            }
                            CSymbol::Func { name, .. } => {
                                log::warn!("In class {} overriding method {}", tname, name)
                            }
                        }
                    }
                    *flabel = *label_count;
                    *label_count += 1;
                    *fid = fieldid;
                    fieldid += 1;
                    cstruct.symbol_table.table.insert(
                        name.clone(),
                        CSymbol::Func {
                            name: (name.clone()),
                            ret_type: (ret_type.clone()),
                            paramlist: paramlist.clone(),
                            flabel: (flabel.clone()),
                            fid: (fid.clone()),
                        },
                    );
                }
                _ => {
                    unreachable!()
                }
            }
        }
        map.insert(
            tname.clone(),
            ASTExprType::Class(ASTClassType {
                name: (tname.clone()),
                fieldsize: (cstruct.fieldsize),
                methodsize: (fieldid - cstruct.fieldsize),
                symbol_table: (cstruct.symbol_table),
                parent: cstruct.parent,
                vft_id: cstruct.vft_id,
            }),
        );
        Ok(())
    }
    //meant to be called first
    pub fn tinstall_class_fields(
        &mut self,
        tfields: &mut LinkedList<CSymbol>,
    ) -> Result<(), String> {
        let tname = &*CLASSNAME.lock().unwrap();
        let pname = &*PCLASSNAME.lock().unwrap();
        let mut vid = VFT_ID.lock().unwrap();
        let v_ = *vid;
        *vid += 1;
        std::mem::drop(vid);
        let map = &mut self.table;
        if map.contains_key(tname) {
            return Err("Type [".to_owned() + tname + "] is already declared.");
        }
        if tfields.len() > 8 {
            return Err("Type [".to_owned() + tname + "] has more than 8 .");
        }
        let mut fieldid: i64 = 0;
        let mut methods: i64 = 0;
        //get this from parent
        let mut ctable: ClassSymbolTable;
        if pname == "" {
            ctable = ClassSymbolTable::default();
        } else {
            ctable = match map.get(pname).unwrap() {
                ASTExprType::Class(c) => {
                    methods = c.methodsize;
                    c.symbol_table.clone()
                }
                _ => return Err("Type [".to_owned() + tname + "] has more than 8 ."),
            }
        }
        std::mem::drop(map);
        for i in tfields.iter_mut() {
            match i {
                CSymbol::Var {
                    name,
                    vartype,
                    varid,
                    ..
                } => {
                    self.validate_field_type(tname, &vartype)?;
                    if ctable.table.contains_key(name) {
                        return Err("In Type [".to_owned()
                            + tname
                            + "], field ["
                            + &name
                            + "] is declared more than once.");
                    }
                    *varid = fieldid;
                    fieldid += 1;
                    ctable.table.insert(name.to_owned(), i.to_owned());
                }
                CSymbol::Func { .. } => unreachable!(),
            }
            //validate the type
        }
        let map = &mut self.table;
        map.insert(
            tname.clone(),
            ASTExprType::Class(ASTClassType {
                name: (tname.clone()),
                fieldsize: (fieldid),
                methodsize: (methods),
                symbol_table: (ctable),
                parent: match pname.as_str() {
                    "" => None,
                    _ => Some(pname.clone()),
                },
                vft_id: v_,
            }),
        );
        Ok(())
    }
    pub fn tinstall_struct(
        &mut self,
        tname: String,
        tfields: LinkedList<Field>,
    ) -> Result<(), String> {
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
                FieldType::Class(_) => {
                    return Err("Classes are not allowed inside structs.".to_owned())
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
                name: tname,
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

#[derive(Debug, Clone)]
pub struct Class {
    pub symbol_table: HashMap<String, CSymbol>,
    pub num_methods: usize,
    pub num_fields: usize,
}

#[derive(Debug, Clone)]
pub enum CSymbol {
    Func {
        name: String,
        ret_type: ASTExprType,
        paramlist: LinkedList<VarNode>,
        flabel: usize,
        fid: i64,
    },
    Var {
        name: String,
        vartype: FieldType,
        varid: i64,
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
    New,
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
    Class(String),
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
#[derive(Debug, Clone)]
pub struct ASTClassType {
    pub name: String,
    pub fieldsize: i64,
    pub methodsize: i64,
    pub symbol_table: ClassSymbolTable,
    pub parent: Option<String>,
    pub vft_id: usize,
}

impl PartialEq for ASTClassType {
    fn eq(&self, _other: &Self) -> bool {
        return self.name == _other.name;
    }
}
impl Eq for ASTClassType {}
// an expression could be a primitive type or a pointer to a primitive type or so on..
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ASTExprType {
    Class(ASTClassType),
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
            FieldType::Class(p) => Ok(TYPE_TABLE.lock().unwrap().tt_get_type(p)?),
        }
    }
}
impl ASTExprType {
    pub fn get_vftid(&self) -> Option<usize> {
        match self {
            ASTExprType::Class(c) => Some(c.vft_id),
            _ => None,
        }
    }
    pub fn is_ancestor(&self, parent: Option<String>) -> bool {
        if parent == None {
            return false;
        }
        match self {
            ASTExprType::Class(c) => {
                if Some(c.name.clone()) == parent {
                    true
                } else {
                    let ptype = TYPE_TABLE
                        .lock()
                        .unwrap()
                        .tt_get_type(&c.parent.as_ref().unwrap())
                        .unwrap();
                    ptype.is_ancestor(parent)
                }
            }
            _ => unreachable!(),
        }
    }
    pub fn is_class(&self) -> bool {
        match self {
            ASTExprType::Class(_) => true,
            _ => false,
        }
    }
    pub fn size(&self) -> Result<usize, String> {
        match self {
            ASTExprType::Primitive(_) => Ok(1),
            ASTExprType::Pointer(p) => match &**p {
                ASTExprType::Class(_) => Ok(2),
                _ => Ok(1),
            },
            ASTExprType::Struct(s) => Ok(s.size),
            ASTExprType::Class(s) => Ok(usize::try_from(s.methodsize + s.fieldsize).unwrap()),
            ASTExprType::Error => {
                unreachable!()
            }
        }
    }
    pub fn is_method(
        &self,
        mname: &String,
        arglist: &LinkedList<ASTNode>,
    ) -> Result<ASTExprType, String> {
        match self {
            ASTExprType::Class(c) => {
                if let Some(entry) = c.symbol_table.table.get(mname) {
                    match entry {
                        CSymbol::Func {
                            name: _,
                            ret_type,
                            paramlist,
                            ..
                        } => {
                            compare_arglist_paramlist(
                                &mut mname.clone(),
                                &mut arglist.clone(),
                                &mut paramlist.clone(),
                            )?;
                            Ok(ret_type.clone())
                        }
                        _ => Err("[".to_owned() + mname + "] is declared as a field."),
                    }
                } else {
                    Err("Method not found.".to_owned())
                }
            }
            _ => Err("Methods are only allowed inside classes.".to_owned()),
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
                    + s.name.as_str()
                    + "]")
            }
            ASTExprType::Class(c) => {
                if let Some(entry) = c.symbol_table.table.get(fname) {
                    match entry {
                        CSymbol::Var {
                            name: _, vartype, ..
                        } => vartype.as_astexprtype(),
                        CSymbol::Func {
                            name: _, ret_type, ..
                        } => Ok(ret_type.clone()),
                    }
                } else {
                    Err("Field [".to_owned()
                        + fname.as_str()
                        + "] not present in ["
                        + c.name.as_str()
                        + "] class")
                }
            }
            _ => Err("Expression of this type cannot be accessed.".to_owned()),
        }
    }
    pub fn validate_method(
        &self,
        fname: &String,
        arglist: &LinkedList<ASTNode>,
    ) -> Result<bool, String> {
        match self {
            ASTExprType::Class(c) => {
                if let Some(CSymbol::Func {
                    name: _,
                    ret_type: _,
                    paramlist,
                    ..
                }) = c.symbol_table.table.get(fname)
                {
                    let mut fname = fname.clone();
                    let mut arglist = arglist.clone();
                    let mut paramlist = paramlist.clone();
                    compare_arglist_paramlist(&mut fname, &mut arglist, &mut paramlist)?;
                    Ok(true)
                } else {
                    Err("Variable / Method error.".to_owned())
                }
            }
            _ => Err("Method called to non class type.".to_owned()),
        }
    }
    pub fn get_type_name(&self) -> Result<String, String> {
        match self {
            ASTExprType::Struct(s) => Ok(s.name.clone()),
            ASTExprType::Class(c) => Ok(c.name.clone()),
            _ => unreachable!(),
        }
    }
    pub fn get_field_id(&self, fname: &String) -> Result<usize, String> {
        match self {
            ASTExprType::Struct(s) => {
                let mut len = 0;
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
            ASTExprType::Class(c) => {
                if let Some(entry) = c.symbol_table.table.get(fname) {
                    let val = match entry {
                        CSymbol::Var {
                            name: _,
                            vartype: _,
                            varid,
                            ..
                        } => varid,
                        CSymbol::Func {
                            name: _,
                            ret_type: _,
                            paramlist: _,
                            flabel: _,
                            fid,
                        } => fid,
                    };
                    Ok(usize::try_from(*val).unwrap())
                } else {
                    Err("Symbol [".to_owned() + fname + "] not declared.")
                }
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
            ASTExprType::Class { .. } => None,
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
            ASTExprType::Class { .. } => *self = p,
            ASTExprType::Error => {}
        }
    }
    pub fn get_base_type(&self) -> ASTExprType {
        match self {
            ASTExprType::Primitive(_) => self.clone(),
            ASTExprType::Pointer(p) => Self::get_base_type(p),
            ASTExprType::Struct { .. } => self.clone(),
            ASTExprType::Class { .. } => self.clone(),
            ASTExprType::Error => ASTExprType::Primitive(PrimitiveType::Void),
        }
    }
    pub fn depth(&self) -> usize {
        match self {
            ASTExprType::Pointer(p) => Self::depth(p) + 1,
            ASTExprType::Primitive(_) => 0,
            ASTExprType::Struct { .. } => 0,
            ASTExprType::Class { .. } => 0,
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
            FieldType::Class { .. } => *self = p,
        }
    }
    pub fn get_base_type(&self) -> FieldType {
        match self {
            FieldType::Primitive(_) => self.clone(),
            FieldType::Pointer(p) => Self::get_base_type(p),
            FieldType::Struct(_) => self.clone(),
            FieldType::Class(_) => self.clone(),
        }
    }
    pub fn depth(&self) -> usize {
        match self {
            FieldType::Pointer(p) => Self::depth(p) + 1,
            FieldType::Primitive(_) => 0,
            FieldType::Struct(_) => 0,
            FieldType::Class(_) => 0,
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

impl From<CSymbol> for LinkedList<CSymbol> {
    fn from(param: CSymbol) -> Self {
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
                GSymbol::Func { .. } => {
                    //exit if a function with similar name exists
                    exit_on_err(
                        "Parameter Symbol ".to_owned()
                            + &self.varname.as_str()
                            + " is already declared as a function",
                    );
                }
                GSymbol::Var { .. } => {
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
                "Parameter Symbol [".to_owned() + &self.varname.as_str() + "] is already declared ",
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
            ASTExprType::Class(s) => {
                usize::try_from(s.methodsize.clone() + s.fieldsize.clone()).unwrap()
            }
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
    if returntype.get_base_type().is_class() {
        exit_on_err("Classes are not allowed as return types.".to_owned());
    }
    for param in paramlist {
        if param.vartype.get_base_type().is_class() {
            exit_on_err("Classes are not allowed as return types.".to_owned());
        }
    }
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
    ClassNode {
        cname: String,
        methods: Box<LinkedList<ASTNode>>,
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
            ASTExprType::Class(s) => write!(f, "class_{}_t", s.name),
            ASTExprType::Pointer(p) => write!(f, "{}{}", "*".repeat(p.depth()), p.get_base_type()),
        }
    }
}
