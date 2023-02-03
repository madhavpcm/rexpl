use crate::codegen::*;
use crate::parserlib::*;
pub fn getexprtype(node: &ASTNode) -> Result<ASTExprType, ()> {
    match node {
        ASTNode::STR(_) => Ok(ASTExprType::String),
        ASTNode::INT(_) => Ok(ASTExprType::Int),
        ASTNode::VAR { name, indices: _ } => getvartype(name),
        ASTNode::UnaryNode { op, ptr } => match op {
            ASTNodeType::Deref => getexprtype(ptr),
            ASTNodeType::Ref => getexprtype(ptr),
            _ => Ok(ASTExprType::Null),
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
                    Ok(exprtype.unwrap())
                } else {
                    Ok(exprtype.unwrap())
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
                    Ok(exprtype.unwrap())
                } else {
                    Ok(exprtype.unwrap())
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
                    Ok(exprtype.unwrap())
                } else {
                    Ok(exprtype.unwrap())
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
                    Ok(exprtype.unwrap())
                } else {
                    Ok(exprtype.unwrap())
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
                    Ok(exprtype.unwrap())
                } else {
                    Ok(exprtype.unwrap())
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
                    Ok(exprtype.unwrap())
                } else {
                    Ok(exprtype.unwrap())
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
                    Ok(exprtype.unwrap().clone())
                } else {
                    Ok(exprtype.unwrap().clone())
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
                    Ok(exprtype.unwrap().clone())
                } else {
                    Ok(exprtype.unwrap().clone())
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
                    Ok(exprtype.unwrap().clone())
                } else {
                    Ok(exprtype.unwrap().clone())
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
                    Ok(exprtype.unwrap().clone())
                } else {
                    Ok(exprtype.unwrap().clone())
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
                    Ok(exprtype.unwrap().clone())
                } else {
                    Ok(exprtype.unwrap().clone())
                }
            }
            _ => Ok(ASTExprType::Null),
        },
        _ => Ok(ASTExprType::Null),
    }
}
pub fn getvarindices(name: &String) -> Result<Vec<usize>, ()> {
    let ss = SCOPE_STACK.lock().unwrap();
    if let Some(lst) = ss.last() {
        if let Some(entry) = lst.get(name) {
            match entry {
                LSymbol::Var {
                    vartype: _,
                    varid: _,
                    varindices,
                } => Ok(varindices.clone()),
                LSymbol::Null => Err(()),
            }
        } else {
            let gst = GLOBALSYMBOLTABLE.lock().unwrap();
            if let Some(entry) = gst.get(name) {
                match entry {
                    GSymbol::Var {
                        vartype: _,
                        varid: _,
                        varindices,
                    } => Ok(varindices.clone()),
                    _ => Err(()),
                }
            } else {
                Err(())
            }
        }
    } else {
        let gst = GLOBALSYMBOLTABLE.lock().unwrap();
        if let Some(entry) = gst.get(name) {
            match entry {
                GSymbol::Var {
                    vartype: _,
                    varid: _,
                    varindices,
                } => Ok(varindices.clone()),
                _ => Err(()),
            }
        } else {
            Err(())
        }
    }
}
pub fn getvarid(name: &String) -> Result<i64, ()> {
    let ss = SCOPE_STACK.lock().unwrap();
    if let Some(lst) = ss.last() {
        if let Some(entry) = lst.get(name) {
            match entry {
                LSymbol::Var {
                    vartype: _,
                    varid,
                    varindices: _,
                } => Ok(varid.clone()),
                LSymbol::Null => Err(()),
            }
        } else {
            let gst = GLOBALSYMBOLTABLE.lock().unwrap();
            if let Some(entry) = gst.get(name) {
                match entry {
                    GSymbol::Var {
                        vartype: _,
                        varid,
                        varindices: _,
                    } => Ok(i64::try_from(varid.clone()).unwrap()),
                    _ => Err(()),
                }
            } else {
                Err(())
            }
        }
    } else {
        let gst = GLOBALSYMBOLTABLE.lock().unwrap();
        if let Some(entry) = gst.get(name) {
            match entry {
                GSymbol::Var {
                    vartype: _,
                    varid,
                    varindices: _,
                } => Ok(i64::try_from(varid.clone()).unwrap()),
                _ => Err(()),
            }
        } else {
            Err(())
        }
    }
}
pub fn getvartype(name: &String) -> Result<ASTExprType, ()> {
    let ss = SCOPE_STACK.lock().unwrap();
    if let Some(lst) = ss.last() {
        if let Some(entry) = lst.get(name) {
            match entry {
                LSymbol::Var {
                    vartype,
                    varid: _,
                    varindices: _,
                } => Ok(vartype.clone()),
                LSymbol::Null => Ok(ASTExprType::Null),
            }
        } else {
            let gst = GLOBALSYMBOLTABLE.lock().unwrap();
            if let Some(entry) = gst.get(name) {
                match entry {
                    GSymbol::Var {
                        vartype,
                        varid: _,
                        varindices: _,
                    } => Ok(vartype.clone()),
                    _ => Ok(ASTExprType::Null),
                }
            } else {
                Ok(ASTExprType::Null)
            }
        }
    } else {
        let gst = GLOBALSYMBOLTABLE.lock().unwrap();
        if let Some(entry) = gst.get(name) {
            match entry {
                GSymbol::Var {
                    vartype,
                    varid: _,
                    varindices: _,
                } => Ok(vartype.clone()),
                _ => Ok(ASTExprType::Null),
            }
        } else {
            Ok(ASTExprType::Null)
        }
    }
}
pub fn varinscope(name: &String) -> Result<bool, ()> {
    let ss = SCOPE_STACK.lock().unwrap();
    if let Some(lst) = ss.last() {
        if let Some(_entry) = lst.get(name) {
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
                    _ => Ok(false),
                }
            } else {
                Ok(false)
            }
        }
    } else {
        let gst = GLOBALSYMBOLTABLE.lock().unwrap();
        if let Some(entry) = gst.get(name) {
            match entry {
                GSymbol::Var {
                    vartype: _,
                    varid: _,
                    varindices: _,
                } => Ok(true),
                _ => Ok(false),
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
            if getexprtype(&expr) != Ok(ASTExprType::Bool) {
                exit_on_err("Invalid Expression inside while".to_owned())
            }
            Ok(true)
        }
        ASTNode::IfNode { expr, xif: _ } => {
            if getexprtype(&expr) != Ok(ASTExprType::Bool) {
                exit_on_err("Invalid Expression inside if".to_owned());
            }
            Ok(true)
        }
        ASTNode::IfElseNode {
            expr,
            xif: _,
            xelse: _,
        } => {
            if getexprtype(&expr) != Ok(ASTExprType::Bool) {
                exit_on_err("Invalid Expression inside if".to_owned());
            }
            Ok(true)
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
                let lhs_t = getexprtype(lhs)?;
                let rhs_t = getexprtype(rhs)?;

                if lhs_t == rhs_t {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            ASTNodeType::Gt => {
                if getexprtype(node) != Ok(ASTExprType::Bool) {
                    Ok(false)
                } else {
                    Ok(true)
                }
            }
            ASTNodeType::Gte => {
                if getexprtype(node) != Ok(ASTExprType::Bool) {
                    Ok(false)
                } else {
                    Ok(true)
                }
            }
            ASTNodeType::Lt => {
                if getexprtype(node) != Ok(ASTExprType::Bool) {
                    Ok(false)
                } else {
                    Ok(true)
                }
            }
            ASTNodeType::Lte => {
                if getexprtype(node) != Ok(ASTExprType::Bool) {
                    Ok(false)
                } else {
                    Ok(true)
                }
            }
            ASTNodeType::Ee => {
                if getexprtype(node) != Ok(ASTExprType::Bool) {
                    Ok(false)
                } else {
                    Ok(true)
                }
            }
            ASTNodeType::Ne => {
                if getexprtype(node) != Ok(ASTExprType::Bool) {
                    Ok(false)
                } else {
                    Ok(true)
                }
            }
            ASTNodeType::Plus => {
                if getexprtype(node) != Ok(ASTExprType::Null) {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            ASTNodeType::Minus => {
                if getexprtype(node) != Ok(ASTExprType::Null) {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            ASTNodeType::Star => {
                if getexprtype(node) != Ok(ASTExprType::Null) {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            ASTNodeType::Slash => {
                if getexprtype(node) != Ok(ASTExprType::Null) {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            ASTNodeType::Mod => {
                if getexprtype(node) != Ok(ASTExprType::Null) {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            _ => Ok(false),
        },
        _ => Ok(true),
    }
}
fn compare_arglist_paramlist(arglist: &ArgList, paramlist: &ParamList) -> bool {
    match (arglist, paramlist) {
        (
            ArgList::Node { expr, next: anext },
            ParamList::Node {
                var: _,
                vartype,
                indices: _,
                next: pnext,
            },
        ) => {
            if &getexprtype(expr).unwrap() != vartype {
                return false;
            }
            compare_arglist_paramlist(anext, pnext)
        }
        (ArgList::Null, ParamList::Null) => true,
        _ => false,
    }
}

pub fn validate_arglist(fname: &String, arglist: &ArgList) -> Result<bool, ()> {
    let gst = GLOBALSYMBOLTABLE.lock().unwrap();
    if let Some(functable) = gst.get(fname) {
        match functable {
            GSymbol::Func {
                ret_type,
                paramlist,
                flabel,
            } => Ok(compare_arglist_paramlist(arglist, paramlist)),
            _ => Err(exit_on_err(
                fname.clone() + " is declared as variable, not a function",
            )),
        }
    } else {
        Ok(false)
    }
}
