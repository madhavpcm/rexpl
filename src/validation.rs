use crate::codegen::*;
use crate::parserlib::*;
use std::collections::LinkedList;
pub fn getexprtype(node: &ASTNode) -> Option<ASTExprType> {
    match node {
        ASTNode::ErrorNode { err } => match err {
            ASTError::TypeError(s) => {
                exit_on_err(s.to_owned());
                Some(ASTExprType::Null)
            }
        },
        ASTNode::STR(_) => Some(ASTExprType::String),
        ASTNode::INT(_) => Some(ASTExprType::Int),
        ASTNode::VAR { name, indices: _ } => getvartype(name),
        ASTNode::UnaryNode { op, ptr } => match op {
            ASTNodeType::Deref => getexprtype(ptr),
            ASTNodeType::Ref => getexprtype(ptr),
            _ => Some(ASTExprType::Null),
        },
        ASTNode::BinaryNode {
            op,
            mut exprtype,
            lhs,
            rhs,
        } => match op {
            ASTNodeType::Gt => {
                if exprtype == None {
                    let lhs_t = getexprtype(lhs)?;
                    let rhs_t = getexprtype(rhs)?;

                    exprtype = Some(match (lhs_t, rhs_t) {
                        (ASTExprType::Int, ASTExprType::Int) => ASTExprType::Bool,
                        (ASTExprType::IntRef, ASTExprType::IntRef) => ASTExprType::Bool,
                        (ASTExprType::StringRef, ASTExprType::StringRef) => ASTExprType::Bool,
                        _ => ASTExprType::Null,
                    });
                    Some(exprtype.unwrap())
                } else {
                    Some(exprtype.unwrap())
                }
            }
            ASTNodeType::Gte => {
                if exprtype == None {
                    let lhs_t = getexprtype(lhs)?;
                    let rhs_t = getexprtype(rhs)?;

                    exprtype = Some(match (lhs_t, rhs_t) {
                        (ASTExprType::Int, ASTExprType::Int) => ASTExprType::Bool,
                        (ASTExprType::IntRef, ASTExprType::IntRef) => ASTExprType::Bool,
                        (ASTExprType::StringRef, ASTExprType::StringRef) => ASTExprType::Bool,
                        _ => ASTExprType::Null,
                    });
                    Some(exprtype.unwrap())
                } else {
                    Some(exprtype.unwrap())
                }
            }
            ASTNodeType::Lt => {
                if exprtype == None {
                    let lhs_t = getexprtype(lhs)?;
                    let rhs_t = getexprtype(rhs)?;

                    exprtype = Some(match (lhs_t, rhs_t) {
                        (ASTExprType::Int, ASTExprType::Int) => ASTExprType::Bool,
                        (ASTExprType::IntRef, ASTExprType::IntRef) => ASTExprType::Bool,
                        (ASTExprType::StringRef, ASTExprType::StringRef) => ASTExprType::Bool,
                        _ => ASTExprType::Null,
                    });
                    Some(exprtype.unwrap())
                } else {
                    Some(exprtype.unwrap())
                }
            }
            ASTNodeType::Lte => {
                if exprtype == None {
                    let lhs_t = getexprtype(lhs)?;
                    let rhs_t = getexprtype(rhs)?;

                    exprtype = Some(match (lhs_t, rhs_t) {
                        (ASTExprType::Int, ASTExprType::Int) => ASTExprType::Bool,
                        (ASTExprType::IntRef, ASTExprType::IntRef) => ASTExprType::Bool,
                        (ASTExprType::StringRef, ASTExprType::StringRef) => ASTExprType::Bool,
                        _ => ASTExprType::Null,
                    });
                    Some(exprtype.unwrap())
                } else {
                    Some(exprtype.unwrap())
                }
            }
            ASTNodeType::Ne => {
                if exprtype == None {
                    let lhs_t = getexprtype(lhs)?;
                    let rhs_t = getexprtype(rhs)?;

                    exprtype = Some(match (lhs_t, rhs_t) {
                        (ASTExprType::Int, ASTExprType::Int) => ASTExprType::Bool,
                        (ASTExprType::IntRef, ASTExprType::IntRef) => ASTExprType::Bool,
                        (ASTExprType::StringRef, ASTExprType::StringRef) => ASTExprType::Bool,
                        (ASTExprType::String, ASTExprType::String) => ASTExprType::Bool,
                        _ => ASTExprType::Null,
                    });
                    Some(exprtype.unwrap())
                } else {
                    Some(exprtype.unwrap())
                }
            }
            ASTNodeType::Ee => {
                if exprtype == None {
                    let lhs_t = getexprtype(lhs)?;
                    let rhs_t = getexprtype(rhs)?;

                    exprtype = Some(match (lhs_t, rhs_t) {
                        (ASTExprType::Int, ASTExprType::Int) => ASTExprType::Bool,
                        (ASTExprType::IntRef, ASTExprType::IntRef) => ASTExprType::Bool,
                        (ASTExprType::StringRef, ASTExprType::StringRef) => ASTExprType::Bool,
                        (ASTExprType::String, ASTExprType::String) => ASTExprType::Bool,
                        _ => ASTExprType::Null,
                    });
                    Some(exprtype.unwrap())
                } else {
                    Some(exprtype.unwrap())
                }
            }
            ASTNodeType::Mod => {
                if exprtype == None {
                    let lhs_t = getexprtype(lhs)?;
                    let rhs_t = getexprtype(rhs)?;

                    exprtype = Some(match (lhs_t, rhs_t) {
                        (ASTExprType::Int, ASTExprType::Int) => ASTExprType::Int,
                        _ => ASTExprType::Null,
                    });
                    Some(exprtype.unwrap().clone())
                } else {
                    Some(exprtype.unwrap().clone())
                }
            }
            ASTNodeType::Slash => {
                if exprtype == None {
                    let lhs_t = getexprtype(lhs)?;
                    let rhs_t = getexprtype(rhs)?;

                    exprtype = Some(match (lhs_t, rhs_t) {
                        (ASTExprType::Int, ASTExprType::Int) => ASTExprType::Int,
                        _ => ASTExprType::Null,
                    });
                    Some(exprtype.unwrap().clone())
                } else {
                    Some(exprtype.unwrap().clone())
                }
            }
            ASTNodeType::Star => {
                if exprtype == None {
                    let lhs_t = getexprtype(lhs)?;
                    let rhs_t = getexprtype(rhs)?;

                    exprtype = Some(match (lhs_t, rhs_t) {
                        (ASTExprType::Int, ASTExprType::Int) => ASTExprType::Int,
                        _ => ASTExprType::Null,
                    });
                    Some(exprtype.unwrap().clone())
                } else {
                    Some(exprtype.unwrap().clone())
                }
            }
            ASTNodeType::Minus => {
                if exprtype == None {
                    let lhs_t = getexprtype(lhs)?;
                    let rhs_t = getexprtype(rhs)?;

                    exprtype = Some(match (lhs_t, rhs_t) {
                        (ASTExprType::Int, ASTExprType::Int) => ASTExprType::Int,
                        (ASTExprType::Int, ASTExprType::IntRef) => ASTExprType::IntRef,
                        (ASTExprType::IntRef, ASTExprType::Int) => ASTExprType::IntRef,
                        (ASTExprType::StringRef, ASTExprType::Int) => ASTExprType::StringRef,
                        (ASTExprType::Int, ASTExprType::StringRef) => ASTExprType::StringRef,
                        _ => ASTExprType::Null,
                    });
                    Some(exprtype.unwrap().clone())
                } else {
                    Some(exprtype.unwrap().clone())
                }
            }
            ASTNodeType::Plus => {
                if exprtype == None {
                    let lhs_t = getexprtype(lhs)?;
                    let rhs_t = getexprtype(rhs)?;

                    exprtype = Some(match (lhs_t, rhs_t) {
                        (ASTExprType::Int, ASTExprType::Int) => ASTExprType::Int,
                        (ASTExprType::Int, ASTExprType::IntRef) => ASTExprType::IntRef,
                        (ASTExprType::IntRef, ASTExprType::Int) => ASTExprType::IntRef,
                        (ASTExprType::StringRef, ASTExprType::Int) => ASTExprType::StringRef,
                        (ASTExprType::Int, ASTExprType::StringRef) => ASTExprType::StringRef,
                        _ => ASTExprType::Null,
                    });
                    Some(exprtype.unwrap().clone())
                } else {
                    Some(exprtype.unwrap().clone())
                }
            }
            _ => None,
        },
        ASTNode::FuncCallNode { fname, arglist: _ } => {
            let gst = GLOBALSYMBOLTABLE.lock().unwrap();
            if let Some(entry) = gst.get(fname) {
                match entry {
                    GSymbol::Func {
                        ret_type,
                        paramlist: _,
                        flabel: _,
                    } => Some(ret_type.clone()),
                    _ => None,
                }
            } else {
                None
            }
        }
        _ => None,
    }
}
pub fn getvarindices(name: &String) -> Option<Vec<usize>> {
    let lst = LOCALSYMBOLTABLE.lock().unwrap();
    if let Some(LSymbol::Var {
        vartype: _,
        varid: _,
        varindices,
    }) = lst.get(name)
    {
        return Some(varindices.clone());
    }
    let gst = GLOBALSYMBOLTABLE.lock().unwrap();
    if let Some(GSymbol::Var {
        vartype: _,
        varid: _,
        varindices,
    }) = gst.get(name)
    {
        return Some(varindices.clone());
    }
    None
}
pub fn get_ldecl_storage(decllist: &Box<LinkedList<ASTNode>>) -> usize {
    let mut size = 0;
    for i in decllist.iter() {
        match i {
            ASTNode::DeclNode { var_type: _, list } => {
                let mut ptr = &**list;
                loop {
                    match ptr {
                        VarList::Node {
                            var: _,
                            refr: _,
                            indices,
                            next,
                        } => {
                            let mut varsize = 1;
                            for i in indices {
                                varsize = varsize * i;
                            }
                            size += varsize;
                            ptr = &**next;
                        }
                        VarList::Null => {
                            break;
                        }
                    }
                }
            }
            _ => {
                panic!("err");
            }
        }
    }
    size
}

pub fn getvarid(name: &String) -> Option<i64> {
    let lst = LOCALSYMBOLTABLE.lock().unwrap();
    if let Some(LSymbol::Var {
        vartype: _,
        varid,
        varindices: _,
    }) = lst.get(name)
    {
        return Some(*varid);
    }
    let gst = GLOBALSYMBOLTABLE.lock().unwrap();
    if let Some(GSymbol::Var {
        vartype: _,
        varid,
        varindices: _,
    }) = gst.get(name)
    {
        return Some(i64::try_from(*varid).unwrap());
    }
    None
}
pub fn getvartype(name: &String) -> Option<ASTExprType> {
    let lst = LOCALSYMBOLTABLE.lock().unwrap();
    if let Some(LSymbol::Var {
        vartype,
        varid: _,
        varindices: _,
    }) = lst.get(name)
    {
        return Some(*vartype);
    }
    let gst = GLOBALSYMBOLTABLE.lock().unwrap();
    if let Some(GSymbol::Var {
        vartype,
        varid: _,
        varindices: _,
    }) = gst.get(name)
    {
        return Some(*vartype);
    }
    None
}
pub fn varinscope(name: &String) -> Result<bool, ()> {
    let lst = LOCALSYMBOLTABLE.lock().unwrap();
    if lst.contains_key(name) {
        Ok(true)
    } else {
        let gst = GLOBALSYMBOLTABLE.lock().unwrap();
        if let Some(entry) = gst.get(name) {
            match entry {
                GSymbol::Var {
                    vartype: _,
                    varid: _,
                    varindices: _,
                } => Ok(true),
                GSymbol::Func {
                    ret_type: _,
                    paramlist: _,
                    flabel: _,
                } => Ok(false),
            }
        } else {
            Ok(false)
        }
    }
}
pub fn validate_ast_node(node: &ASTNode) -> Result<bool, ()> {
    //while
    match node {
        ASTNode::VAR { name, indices: _ } => varinscope(name),
        ASTNode::INT(_) => Ok(true),
        ASTNode::STR(_) => Ok(true),
        ASTNode::BreakNode => {
            let w = WHILE_TRACKER.lock().unwrap();
            if w.len() > 1 {
                Ok(true)
            } else {
                Ok(false)
            }
        }
        ASTNode::ContinueNode => {
            let w = WHILE_TRACKER.lock().unwrap();
            if w.len() > 1 {
                Ok(true)
            } else {
                Ok(false)
            }
        }
        ASTNode::DeclNode {
            var_type: _,
            list: _,
        } => Ok(true),
        ASTNode::WhileNode { expr, xdo: _ } => {
            if getexprtype(&expr) != Some(ASTExprType::Bool) {
                exit_on_err("Invalid Expression inside while".to_owned())
            }
            Ok(true)
        }
        ASTNode::IfNode { expr, xif: _ } => {
            if getexprtype(&expr) != Some(ASTExprType::Bool) {
                exit_on_err("Invalid Expression inside if".to_owned());
            }
            Ok(true)
        }
        ASTNode::IfElseNode {
            expr,
            xif: _,
            xelse: _,
        } => {
            if getexprtype(&expr) != Some(ASTExprType::Bool) {
                exit_on_err("Invalid Expression inside ifelse".to_owned());
            }
            Ok(true)
        }
        ASTNode::ReturnNode { expr } => {
            let ct = CURR_TYPE.lock().unwrap();

            if getexprtype(expr).unwrap() != *ct {
                Ok(false)
            } else {
                Ok(true)
            }
        }
        ASTNode::UnaryNode { op, ptr } => match op {
            ASTNodeType::Ref => {
                let var = &**ptr;
                match var {
                    ASTNode::VAR { name, indices: _ } => {
                        if varinscope(&name) == Ok(false) {
                            Ok(false)
                        } else {
                            let vartype = getvartype(&name).unwrap();
                            if vartype == ASTExprType::Int || vartype == ASTExprType::String {
                                Ok(true)
                            } else {
                                Ok(false)
                            }
                        }
                    }
                    _ => Ok(false),
                }
            }
            ASTNodeType::Deref => {
                let var = &**ptr;
                match var {
                    ASTNode::VAR { name, indices: _ } => {
                        if varinscope(&name) == Ok(false) {
                            Ok(false)
                        } else {
                            let vartype = getvartype(&name).unwrap();
                            if vartype == ASTExprType::IntRef || vartype == ASTExprType::StringRef {
                                Ok(true)
                            } else {
                                Ok(false)
                            }
                        }
                    }
                    _ => Ok(false),
                }
            }
            ASTNodeType::Write => {
                let expr = &**ptr;
                match expr {
                    ASTNode::BinaryNode {
                        op: _,
                        exprtype,
                        lhs: _,
                        rhs: _,
                    } => match exprtype {
                        Some(ASTExprType::Bool) => Ok(false),
                        _ => Ok(true),
                    },
                    _ => Ok(true),
                }
            }
            _ => Ok(true),
        },
        ASTNode::BinaryNode {
            op,
            exprtype: _,
            lhs,
            rhs,
        } => match op {
            ASTNodeType::Equals => {
                let lhs_t = getexprtype(lhs);
                let rhs_t = getexprtype(rhs);

                if lhs_t == rhs_t {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            ASTNodeType::Gt => {
                if getexprtype(node) != Some(ASTExprType::Bool) {
                    Ok(false)
                } else {
                    Ok(true)
                }
            }
            ASTNodeType::Gte => {
                if getexprtype(node) != Some(ASTExprType::Bool) {
                    Ok(false)
                } else {
                    Ok(true)
                }
            }
            ASTNodeType::Lt => {
                if getexprtype(node) != Some(ASTExprType::Bool) {
                    Ok(false)
                } else {
                    Ok(true)
                }
            }
            ASTNodeType::Lte => {
                if getexprtype(node) != Some(ASTExprType::Bool) {
                    Ok(false)
                } else {
                    Ok(true)
                }
            }
            ASTNodeType::Ee => {
                if getexprtype(node) != Some(ASTExprType::Bool) {
                    Ok(false)
                } else {
                    Ok(true)
                }
            }
            ASTNodeType::Ne => {
                if getexprtype(node) != Some(ASTExprType::Bool) {
                    Ok(false)
                } else {
                    Ok(true)
                }
            }
            ASTNodeType::Plus => {
                if getexprtype(node) != None {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            ASTNodeType::Minus => {
                if getexprtype(node) != None {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            ASTNodeType::Star => {
                if getexprtype(node) != None {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            ASTNodeType::Slash => {
                if getexprtype(node) != None {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            ASTNodeType::Mod => {
                if getexprtype(node) != None {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            _ => Ok(false),
        },
        ASTNode::FuncCallNode { fname, arglist } => {
            let gst = GLOBALSYMBOLTABLE.lock().unwrap();
            let p;
            if let Some(entry) = gst.get(fname) {
                match entry {
                    GSymbol::Func {
                        ret_type: _,
                        paramlist,
                        flabel: _,
                    } => {
                        p = paramlist.clone();
                    }
                    _ => return Ok(false),
                }
            } else {
                return Ok(false);
            }
            std::mem::drop(gst);
            Ok(compare_arglist_paramlist(arglist, &p))
        }
        ASTNode::FuncDeclNode {
            fname,
            ret_type: _,
            paramlist: _,
        } => {
            if fname.as_str() == "main" {
                exit_on_err("`main` cannot be redeclared".to_owned());
                Ok(false)
            } else {
                let gst = GLOBALSYMBOLTABLE.lock().unwrap();
                if gst.contains_key(fname) == true {
                    Ok(false)
                } else {
                    Ok(true)
                }
            }
        }
        ASTNode::FuncDefNode {
            fname,
            ret_type: r1,
            paramlist: a,
            decl: _,
            body: _,
        } => {
            let gst = GLOBALSYMBOLTABLE.lock().unwrap();
            if let Some(entry) = gst.get(fname) {
                match entry {
                    GSymbol::Var {
                        vartype: _,
                        varid: _,
                        varindices: _,
                    } => Ok(false),
                    GSymbol::Func {
                        ret_type: r2,
                        paramlist: b,
                        flabel: _,
                    } => Ok((&**a == b) && (r1 == r2)),
                    _ => Ok(false),
                }
            } else {
                Ok(false)
            }
        }
        _ => Ok(true),
    }
}
/*
 * Function to validate the pamalist in declaration to definition
 */
fn compare_arglist_paramlist(arglist: &LinkedList<ASTNode>, paramlist: &LinkedList<Param>) -> bool {
    if arglist.len() != paramlist.len() {
        return false;
    }
    let mut aiter = arglist.iter();
    let mut piter = paramlist.iter();

    while let (Some(arg), Some(param)) = (aiter.next(), piter.next()) {
        if getexprtype(arg).unwrap() != param.vartype {
            return false;
        }
    }
    true
}
/*
 * Function to validate the type of argument list to paramlist
 */
pub fn validate_arglist(fname: &String, arglist: &LinkedList<ASTNode>) -> Result<bool, ()> {
    let gst = GLOBALSYMBOLTABLE.lock().unwrap();
    if let Some(functable) = gst.get(fname) {
        match functable {
            GSymbol::Func {
                ret_type: _,
                paramlist,
                flabel: _,
            } => Ok(compare_arglist_paramlist(arglist, paramlist)),
            _ => Err(exit_on_err(
                fname.clone() + " is declared as variable, not a function",
            )),
        }
    } else {
        Ok(false)
    }
}
//Gets the label of a function
pub fn get_function_label(fname: &String) -> usize {
    let gst = GLOBALSYMBOLTABLE.lock().unwrap();
    if let Some(entry) = gst.get(fname) {
        return match entry {
            GSymbol::Func {
                ret_type: _,
                paramlist: _,
                flabel,
            } => flabel.clone(),
            _ => LABEL_NOT_FOUND,
        };
    } else {
        LABEL_NOT_FOUND
    }
}
/*
 * Validates if a variable is in scope
 */
pub fn validate_locality(vname: String) {
    let lst = LOCALSYMBOLTABLE.lock().unwrap();
    let gst = GLOBALSYMBOLTABLE.lock().unwrap();
    if let Some(entry) = gst.get(&vname) {
        match entry {
            GSymbol::Func {
                ret_type: _,
                paramlist: _,
                flabel: _,
            } => {
                //exit if a function with similar name exists
                exit_on_err(
                    "Parameter Symbol ".to_owned()
                        + vname.as_str()
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
                    vname
                );
            }
        }
    }
    if lst.contains_key(&vname) == true {
        exit_on_err("Parameter Symbol ".to_owned() + vname.as_str() + " is already declared ");
    }
}
