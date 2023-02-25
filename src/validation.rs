use crate::codegen::*;
use crate::parserlib::*;
use std::collections::LinkedList;

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
impl ASTNode {
    pub fn validate(self) -> Result<(), String> {
        //while
        match self {
            ASTNode::VAR { name, indices: _ } => varinscope(&name),
            ASTNode::INT(_) => Ok(()),
            ASTNode::STR(_) => Ok(()),
            ASTNode::BreakNode => {
                let w = WHILE_TRACKER.lock().unwrap();
                if w.len() < 2 {
                    return Err("Break statement must be used inside a while loop.".to_owned());
                }
                Ok(())
            }
            ASTNode::ContinueNode => {
                let w = WHILE_TRACKER.lock().unwrap();
                if w.len() < 2 {
                    return Err("Continue statement must be used inside a while loop.".to_owned());
                }
                Ok(())
            }
            ASTNode::WhileNode { expr, xdo: _ } => {
                if expr.getexprtype() != Some(ASTExprType::Primitive(PrimitiveType::Bool)) {
                    return Err("Invalid expression inside while's condition.".to_owned());
                }
                Ok(())
            }
            ASTNode::IfNode { expr, xif: _ } => {
                if expr.getexprtype() != Some(ASTExprType::Primitive(PrimitiveType::Bool)) {
                    return Err("Invalid expression inside if's condition.".to_owned());
                }
                Ok(())
            }
            ASTNode::IfElseNode {
                expr,
                xif: _,
                xelse: _,
            } => {
                if expr.getexprtype() != Some(ASTExprType::Primitive(PrimitiveType::Bool)) {
                    return Err("Invalid expression inside if else's condition.".to_owned());
                }
                Ok(())
            }
            ASTNode::ReturnNode { expr } => {
                let ct = CURR_TYPE.lock().unwrap();
                if expr.getexprtype() != Some(*ct) {
                    return Err("Invalid expression inside if else's condition.".to_owned());
                }
                Ok(())
            }
            ASTNode::UnaryNode { op, ptr } => match op {
                ASTNodeType::Ref => {
                    let var = &*ptr;
                    match var {
                            ASTNode::VAR { name, indices: _ } => {
                                varinscope(&name);
                                let vartype = getvartype(&name).unwrap();
                                match vartype {
                                    ASTExprType::Primitive(PrimitiveType::Int) => Ok(()),
                                    ASTExprType::Primitive(PrimitiveType::String) => Ok(()),
                                    _ => Err("Reference operator got a variable of invalid type"
                                        .to_owned()),
                                }
                            }
                            _ => Err(
                                "Reference operator expects a declared variable of Primitive Type or User Defined Type."
                                    .to_owned(),
                            ),
                        }
                }
                ASTNodeType::Deref => {
                    let var = &*ptr;
                    match var {
                        ASTNode::VAR { name, indices: _ } => {
                            varinscope(name);
                            let vartype = getvartype(&name).unwrap();
                            match vartype {
                                ASTExprType::Pointer(_) => Ok(()),
                                _ => Err("Dereference operator expects a variable with pointer data type.".to_owned()),
                            }
                        }
                        _ => Err("Dereference operator expects a variable".to_owned()),
                    }
                }
                ASTNodeType::Write => {
                    let expr = &*ptr;
                    match expr {
                        ASTNode::BinaryNode {
                            op: _,
                            exprtype,
                            lhs: _,
                            rhs: _,
                        } => match exprtype {
                            Some(ASTExprType::Primitive(PrimitiveType::Bool)) => {
                                Err("Write statement expects a str or int type.".to_owned())
                            }
                            Some(ASTExprType::Primitive(PrimitiveType::Void)) => {
                                Err("Write statement expects a str or int type.".to_owned())
                            }
                            Some(ASTExprType::Primitive(PrimitiveType::Null)) => {
                                Err("Write statement expects a str or int type.".to_owned())
                            }
                            Some(ASTExprType::Pointer(p)) => {
                                log::warn!("Writing a pointer!");
                                Ok(())
                            }
                            _ => Ok(()),
                        },
                        _ => Ok(()),
                    }
                }
                _ => Ok(()),
            },
            ASTNode::BinaryNode {
                op,
                exprtype: _,
                lhs,
                rhs,
            } => match op {
                ASTNodeType::Equals => {
                    let lhs_t = lhs.getexprtype();
                    let rhs_t = rhs.getexprtype();

                    if lhs_t == rhs_t {
                        Ok(())
                    } else {
                        Err("Assignment of invalid type.".to_owned())
                    }
                }
                ASTNodeType::Gt
                | ASTNodeType::Gte
                | ASTNodeType::Ne
                | ASTNodeType::Ee
                | ASTNodeType::Lt
                | ASTNodeType::Lte => {
                    if self.getexprtype() != Some(ASTExprType::Primitive(PrimitiveType::Bool)) {
                        Err("Boolean operator got invalid types.".to_owned())
                    } else {
                        Ok(())
                    }
                }
                ASTNodeType::Plus
                | ASTNodeType::Minus
                | ASTNodeType::Star
                | ASTNodeType::Slash
                | ASTNodeType::Mod => {
                    let expr = self.getexprtype();
                    if expr != None && expr != Some(ASTExprType::Primitive(PrimitiveType::Null)) {
                        Ok(())
                    } else {
                        Err("Operator +-/*% got invalid types.".to_owned())
                    }
                }
                _ => Ok(()),
            },
            ASTNode::FuncCallNode { fname, arglist } => {
                let gst = GLOBALSYMBOLTABLE.lock().unwrap();
                let p;
                if let Some(entry) = gst.get(&fname) {
                    match entry {
                        GSymbol::Func {
                            ret_type: _,
                            paramlist,
                            flabel: _,
                        } => {
                            p = paramlist.clone();
                        }
                        _ => {
                            return Err("Function name [".to_owned()
                                + fname.as_str()
                                + "] is not declared")
                        }
                    }
                } else {
                    return Ok(());
                }
                compare_arglist_paramlist(&*arglist, &p)
            }
            ASTNode::FuncDefNode {
                fname,
                ret_type: r1,
                body: _,
                paramlist: a,
            } => {
                let gst = GLOBALSYMBOLTABLE.lock().unwrap();
                if let Some(entry) = gst.get(&fname) {
                    match entry {
                        GSymbol::Var {
                            vartype: _,
                            varid: _,
                            varindices: _,
                        } => Err("Function with name [".to_owned()
                            + fname.as_str()
                            + "]is already declared as a variable"),
                        GSymbol::Func {
                            ret_type: r2,
                            paramlist: b,
                            flabel: _,
                        } => {
                            if &r1 != r2 {
                                return Err("Function [".to_owned()
                                    + fname.as_str()
                                    + "]'s return type doesn't match in it declaration");
                            }
                            if &a != b {
                                return Err("Function [".to_owned()
                                    + fname.as_str()
                                    + "]'s parameter list doesn't match in it declaration");
                            }
                            Ok(())
                        }
                    }
                } else {
                    Err("Function with name [".to_owned() + fname.as_str() + "] is not declared")
                }
            }
            _ => Ok(()),
        }
    }
    pub fn getexprtype(self) -> Option<ASTExprType> {
        match self {
            ASTNode::ErrorNode { err } => match err {
                ASTError::TypeError(s) => {
                    exit_on_err(s.to_owned());
                    Some(ASTExprType::Primitive(PrimitiveType::Null))
                }
            },
            ASTNode::STR(_) => Some(ASTExprType::Primitive(PrimitiveType::String)),
            ASTNode::INT(_) => Some(ASTExprType::Primitive(PrimitiveType::Int)),
            ASTNode::VAR { name, indices: _ } => getvartype(&name),
            ASTNode::UnaryNode { op, ptr } => match op {
                ASTNodeType::Deref => ptr.getexprtype(),
                ASTNodeType::Ref => ptr.getexprtype(),
                _ => Some(ASTExprType::Primitive(PrimitiveType::Null)),
            },
            ASTNode::BinaryNode {
                op,
                mut exprtype,
                lhs,
                rhs,
            } => match op {
                ASTNodeType::Gt | ASTNodeType::Lt | ASTNodeType::Gte | ASTNodeType::Lte => {
                    if exprtype == None {
                        let lhs_t = lhs.getexprtype()?;
                        let rhs_t = rhs.getexprtype()?;
                        exprtype = match (lhs_t, rhs_t) {
                            (
                                ASTExprType::Primitive(PrimitiveType::Int),
                                ASTExprType::Primitive(PrimitiveType::Int),
                            ) => Some(ASTExprType::Primitive(PrimitiveType::Bool)),
                            _ => Some(ASTExprType::Error),
                        };
                        exprtype.clone()
                    } else {
                        exprtype.clone()
                    }
                }
                ASTNodeType::Ee | ASTNodeType::Ne => {
                    if exprtype == None {
                        let lhs_t = lhs.getexprtype()?;
                        let rhs_t = rhs.getexprtype()?;

                        exprtype = match (lhs_t, rhs_t) {
                            (
                                ASTExprType::Primitive(PrimitiveType::Int),
                                ASTExprType::Primitive(PrimitiveType::Int),
                            ) => Some(ASTExprType::Primitive(PrimitiveType::Bool)),
                            (ASTExprType::Pointer(ptr1), ASTExprType::Pointer(ptr2)) => {
                                if ptr1.depth() == ptr2.depth()
                                    && ptr1.get_base_type() == ptr2.get_base_type()
                                {
                                    Some(*ptr1.clone())
                                } else {
                                    Some(ASTExprType::Error)
                                }
                            }
                            _ => None,
                        };
                        exprtype.clone()
                    } else {
                        exprtype.clone()
                    }
                }
                ASTNodeType::Mod | ASTNodeType::Star | ASTNodeType::Slash => {
                    if exprtype == None {
                        let lhs_t = lhs.getexprtype()?;
                        let rhs_t = rhs.getexprtype()?;

                        exprtype = match (lhs_t, rhs_t) {
                            (
                                ASTExprType::Primitive(PrimitiveType::Int),
                                ASTExprType::Primitive(PrimitiveType::Int),
                            ) => Some(ASTExprType::Primitive(PrimitiveType::Bool)),
                            _ => Some(ASTExprType::Error),
                        };
                        exprtype.clone()
                    } else {
                        exprtype.clone()
                    }
                }
                ASTNodeType::Minus | ASTNodeType::Plus => {
                    if exprtype == None {
                        let lhs_t = lhs.getexprtype()?;
                        let rhs_t = rhs.getexprtype()?;

                        exprtype = match (lhs_t, rhs_t) {
                            (
                                ASTExprType::Primitive(PrimitiveType::Int),
                                ASTExprType::Primitive(PrimitiveType::Int),
                            ) => Some(ASTExprType::Primitive(PrimitiveType::Int)),
                            (
                                ASTExprType::Primitive(PrimitiveType::Int),
                                ASTExprType::Pointer(p),
                            ) => Some(*p.clone()),
                            (
                                ASTExprType::Pointer(p),
                                ASTExprType::Primitive(PrimitiveType::Int),
                            ) => Some(*p.clone()),
                            _ => Some(ASTExprType::Error),
                        };
                        exprtype.clone()
                    } else {
                        exprtype.clone()
                    }
                }
                _ => Some(ASTExprType::Primitive(PrimitiveType::Null)),
            },
            ASTNode::FuncCallNode { fname, arglist: _ } => {
                let gst = GLOBALSYMBOLTABLE.lock().unwrap();
                if let Some(entry) = gst.get(&fname) {
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
pub fn varinscope(name: &String) -> Result<(), String> {
    let lst = LOCALSYMBOLTABLE.lock().unwrap();
    if lst.contains_key(name) {
        Ok(())
    } else {
        let gst = GLOBALSYMBOLTABLE.lock().unwrap();
        if let Some(entry) = gst.get(name) {
            match entry {
                GSymbol::Var {
                    vartype: _,
                    varid: _,
                    varindices: _,
                } => Ok(()),
                GSymbol::Func {
                    ret_type: _,
                    paramlist: _,
                    flabel: _,
                } => Err("Symbol [".to_owned() + name.as_str() + "] declared as a function."),
            }
        } else {
            Err("Symbol [".to_owned() + name.as_str() + "] is not declared.")
        }
    }
}
/*
 * Function to validate the pamalist in declaration to definition
 */
fn compare_arglist_paramlist(
    arglist: &LinkedList<ASTNode>,
    paramlist: &LinkedList<VarNode>,
) -> Result<(), String> {
    if arglist.len() != paramlist.len() {
        return Err(
            "Function call arguments and declaration arguments dont match in length.".to_owned(),
        );
    }
    let mut aiter = arglist.iter();
    let mut piter = paramlist.iter();

    let mut ctr = 1;
    while let (Some(arg), Some(param)) = (aiter.next(), piter.next()) {
        if arg.getexprtype().unwrap() != param.vartype {
            return Err(
                "Function call arguments and declaration arguments dont match in type at ["
                    .to_owned()
                    + ctr.to_string().as_str()
                    + "] position.",
            );
        }
    }
    Ok(())
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
