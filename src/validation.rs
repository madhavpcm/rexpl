use crate::codegen::*;
use crate::parserlib::*;
use std::collections::LinkedList;

pub fn getvartype(name: &String) -> Option<ASTExprType> {
    let lst = LOCALSYMBOLTABLE.lock().unwrap();
    if let Some(LSymbol::Var {
        vartype,
        varid: _,
        varindices,
    }) = lst.get(name)
    {
        let mut vtype = vartype.clone();
        for _ in varindices.iter() {
            vtype = vtype.refr().unwrap();
        }
        return Some(vtype);
    }
    let gst = GLOBALSYMBOLTABLE.lock().unwrap();
    if let Some(GSymbol::Var {
        vartype,
        varid: _,
        varindices,
    }) = gst.get(name)
    {
        let mut vtype = vartype.clone();
        for _ in varindices.iter() {
            vtype = vtype.refr().unwrap();
        }
        return Some(vtype);
    }
    None
}
#[allow(dead_code)]
fn validate_field_array_access(
    array_name: &String,
    parent_type: &ASTExprType,
    array_access: &mut Vec<Box<ASTNode>>,
) -> Result<(), String> {
    let actual_array_type = parent_type.get_field_type(array_name)?;
    for ei in 0..array_access.len() {
        array_access[ei].validate()?;
        if let Some(ei_type) = array_access[ei].getexprtype() {
            match ei_type {
                ASTExprType::Primitive(PrimitiveType::Int) => {
                    continue;
                }
                _ => {
                    return Err("Invalid type used to index variable [".to_owned()
                        + array_name.as_str()
                        + "] at "
                        + "[]".repeat(ei).as_str());
                }
            }
        } else {
            return Err("Invalid type used to index variable [".to_owned()
                + array_name.as_str()
                + "] at "
                + "[]".repeat(ei).as_str());
        }
    }
    Ok(())
}
impl ASTNode {
    pub fn validate(&mut self) -> Result<(), String> {
        match self {
            ASTNode::VAR {
                name,
                array_access,
                dot_field_access,
                arrow_field_access,
            } => {
                varinscope(&name)?;
                let dind = getvarindices(&name).unwrap();
                if array_access.len() > dind.len() {
                    return Err("Index dimension error for variable [".to_owned()
                        + name.as_str()
                        + "]");
                }
                //validate array access
                for ei in 0..array_access.len() {
                    array_access[ei].validate()?;
                    if let Some(ei_type) = array_access[ei].getexprtype() {
                        match ei_type {
                            ASTExprType::Primitive(PrimitiveType::Int) => {
                                continue;
                            }
                            _ => {
                                return Err("Invalid type used to index variable [".to_owned()
                                    + name.as_str()
                                    + "] at "
                                    + "[]".repeat(ei).as_str());
                            }
                        }
                    } else {
                        return Err("Invalid type used to index variable [".to_owned()
                            + name.as_str()
                            + "] at "
                            + "[]".repeat(ei).as_str());
                    }
                }

                let mut currtype: ASTExprType = getvartype(name).unwrap();

                for _ in 0..array_access.len() {
                    currtype = currtype.derefr().unwrap();
                }
                // check if dot field type is
                let mut dotptr = &mut **dot_field_access;
                let mut arrowptr = &mut **arrow_field_access;
                loop {
                    if dotptr == &ASTNode::Null && arrowptr == &ASTNode::Null {
                        break;
                    }
                    if dotptr != &ASTNode::Null {
                        if let ASTNode::VAR {
                            name: nname,
                            array_access,
                            dot_field_access,
                            arrow_field_access,
                        } = dotptr
                        {
                            if array_access.len() > 0 {
                                return Err(
                                    "Arrays inside struct is not implemented yet!".to_owned()
                                );
                            }
                            currtype.get_field_id(&nname)?;
                            //validate_field_array_access(nname, &currtype, array_access)?;

                            currtype = currtype.get_field_type(nname)?;
                            for _ in 0..array_access.len() {
                                currtype = currtype.derefr().unwrap();
                            }
                            //get next field type
                            dotptr = &mut **dot_field_access;
                            arrowptr = &mut **arrow_field_access;
                            continue;
                        } else {
                            return Err("Dot operator can only be used to access [struct_t] types"
                                .to_owned());
                        }
                    }
                    // check if dot field type is
                    if arrowptr != &ASTNode::Null {
                        if let ASTNode::VAR {
                            name: nname,
                            array_access,
                            dot_field_access,
                            arrow_field_access,
                        } = arrowptr
                        {
                            if array_access.len() > 0 {
                                return Err(
                                    "Arrays inside struct is not implemented yet!".to_owned()
                                );
                            }
                            if let ASTExprType::Pointer(etype) = &currtype {
                                etype.get_field_id(&nname)?;
                                currtype = etype.get_field_type(nname)?;
                                for _ in 0..array_access.len() {
                                    currtype = currtype.derefr().unwrap();
                                }
                                dotptr = &mut **dot_field_access;
                                arrowptr = &mut **arrow_field_access;
                                continue;
                            } else {
                                return Err("Dot operator expects a variable name".to_owned());
                            }
                            //TODO validate array access
                            //validate_field_array_access(nname, &currtype, array_access)?;
                        } else {
                            return Err(
                                "Arrow operator can only be used to access [struct_t*] types"
                                    .to_owned(),
                            );
                        }
                    }
                }
                Ok(())
            }
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
                let ct = RET_TYPE.lock().unwrap().clone();
                expr.validate()?;
                let b = expr.getexprtype();
                if b != Some(ct) {
                    return Err("Invalid return type.".to_owned());
                }
                Ok(())
            }
            ASTNode::UnaryNode {
                op,
                exprtype: _,
                ptr,
                depth,
            } => match op {
                ASTNodeType::Deref => {
                    ptr.validate()?;

                    if let Some(ptrtype) = ptr.getexprtype() {
                        if ptrtype.depth() < depth.unwrap() {
                            return Err("Dereferencing non pointer type.".to_owned());
                        }
                    }
                    self.getexprtype();
                    Ok(())
                }
                ASTNodeType::Ref => {
                    ptr.validate()?;
                    match &**ptr {
                        ASTNode::VAR {
                            name,
                            array_access,
                            dot_field_access,
                            arrow_field_access,
                        } => {
                            let varindices = getvarindices(name).unwrap();
                            if array_access.len() != varindices.len() {
                                return Err("Reference operator can only reference to the basetype of an array.".to_owned());
                            }
                            Ok(())
                        }
                        _ => Err("Reference operator expects a declared variable.".to_owned()),
                    }
                }
                ASTNodeType::Write => {
                    ptr.validate()?;
                    match &**ptr {
                        ASTNode::VAR {
                            name: _,
                            array_access: _,
                            dot_field_access: _,
                            arrow_field_access: _,
                        } => Ok(()),
                        ASTNode::INT(_) => Ok(()),
                        ASTNode::STR(_) => Ok(()),
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
                            Some(ASTExprType::Pointer(_)) => {
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
                let mut p;
                if let Some(entry) = gst.get(fname) {
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
                compare_arglist_paramlist(arglist, &mut p)
            }
            ASTNode::FuncDefNode {
                fname,
                ret_type: r1,
                body: _,
                paramlist: a,
            } => {
                let gst = GLOBALSYMBOLTABLE.lock().unwrap();
                if let Some(entry) = gst.get(&fname.clone()) {
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
                            if r1 != r2 {
                                return Err("Function [".to_owned()
                                    + fname.as_str()
                                    + "]'s return type doesn't match in it declaration");
                            }
                            if a != b {
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
    pub fn getexprtype(&mut self) -> Option<ASTExprType> {
        match self {
            ASTNode::ErrorNode { err } => match err {
                ASTError::TypeError(s) => {
                    exit_on_err(s.to_owned());
                    Some(ASTExprType::Primitive(PrimitiveType::Null))
                }
            },
            ASTNode::STR(_) => Some(ASTExprType::Primitive(PrimitiveType::String)),
            ASTNode::INT(_) => Some(ASTExprType::Primitive(PrimitiveType::Int)),
            ASTNode::VAR {
                name,
                array_access,
                dot_field_access,
                arrow_field_access,
            } => {
                if let Some(mut vtype) = getvartype(&name) {
                    for _ in 0..array_access.len() {
                        vtype = vtype.derefr().unwrap();
                    }
                    let mut dotptr = &**dot_field_access;
                    let mut arrowptr = &**arrow_field_access;
                    loop {
                        if dotptr == &ASTNode::Null && arrowptr == &ASTNode::Null {
                            break;
                        }
                        if dotptr != &ASTNode::Null {
                            if let ASTNode::VAR {
                                name: nname,
                                array_access: _,
                                dot_field_access,
                                arrow_field_access,
                            } = dotptr
                            {
                                if array_access.len() > 0 {
                                    exit_on_err(
                                        "Arrays inside structs are not implemented yet.".to_owned(),
                                    );
                                }
                                if let Err(e) = vtype.get_field_id(nname) {
                                    exit_on_err(e.to_owned());
                                }
                                vtype = vtype.get_field_type(nname).unwrap();
                                dotptr = &**dot_field_access;
                                arrowptr = &**arrow_field_access;
                            }
                        }
                        if arrowptr != &ASTNode::Null {
                            if let ASTNode::VAR {
                                name: nname,
                                array_access,
                                dot_field_access,
                                arrow_field_access,
                            } = arrowptr
                            {
                                if array_access.len() > 0 {
                                    exit_on_err(
                                        "Arrays inside structs are not implemented yet.".to_owned(),
                                    );
                                }
                                if let ASTExprType::Pointer(etype) = &vtype {
                                    if let Err(e) = etype.get_field_id(&nname) {
                                        exit_on_err(e.to_owned());
                                    }
                                    vtype = etype.get_field_type(nname).unwrap();
                                    dotptr = &**dot_field_access;
                                    arrowptr = &**arrow_field_access;
                                } else {
                                    exit_on_err(
                                        "Arrow operator expects a variable of struct_t*".to_owned(),
                                    );
                                }
                            }
                        }
                    }
                    return Some(vtype);
                } else {
                    return None;
                }
            }
            ASTNode::UnaryNode {
                op,
                exprtype,
                ptr,
                depth,
            } => match op {
                ASTNodeType::Deref => {
                    if exprtype == &None {
                        let mut ptrtype = ptr.getexprtype().unwrap();
                        for _i in 0..depth.unwrap() {
                            ptrtype = ptrtype.derefr().unwrap();
                        }
                        *exprtype = Some(ptrtype);
                        exprtype.clone()
                    } else {
                        exprtype.clone()
                    }
                }
                ASTNodeType::Ref => {
                    if exprtype == &None {
                        if let Some(base) = ptr.getexprtype() {
                            *exprtype = base.refr();
                            base.refr()
                        } else {
                            exprtype.clone()
                        }
                    } else {
                        exprtype.clone()
                    }
                }
                _ => Some(ASTExprType::Primitive(PrimitiveType::Null)),
            },
            ASTNode::BinaryNode {
                op,
                exprtype,
                lhs,
                rhs,
            } => match op {
                ASTNodeType::Gt | ASTNodeType::Lt | ASTNodeType::Gte | ASTNodeType::Lte => {
                    if *exprtype == None {
                        let lhs_t = lhs.getexprtype()?;
                        let rhs_t = rhs.getexprtype()?;
                        *exprtype = match (lhs_t, rhs_t) {
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
                    if *exprtype == None {
                        let lhs_t = lhs.getexprtype()?;
                        let rhs_t = rhs.getexprtype()?;

                        *exprtype = match (lhs_t, rhs_t) {
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
                        exprtype.as_ref().cloned()
                    } else {
                        exprtype.as_ref().cloned()
                    }
                }
                ASTNodeType::Mod | ASTNodeType::Star | ASTNodeType::Slash => {
                    if *exprtype == None {
                        let lhs_t = lhs.getexprtype()?;
                        let rhs_t = rhs.getexprtype()?;

                        *exprtype = match (lhs_t, rhs_t) {
                            (
                                ASTExprType::Primitive(PrimitiveType::Int),
                                ASTExprType::Primitive(PrimitiveType::Int),
                            ) => Some(ASTExprType::Primitive(PrimitiveType::Int)),
                            _ => Some(ASTExprType::Error),
                        };
                        exprtype.as_ref().cloned()
                    } else {
                        exprtype.as_ref().cloned()
                    }
                }
                ASTNodeType::Minus | ASTNodeType::Plus => {
                    if *exprtype == None {
                        let lhs_t = lhs.getexprtype()?;
                        let rhs_t = rhs.getexprtype()?;

                        *exprtype = match (lhs_t, rhs_t) {
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
                if let Some(entry) = gst.get(&fname.clone()) {
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
    arglist: &mut LinkedList<ASTNode>,
    paramlist: &mut LinkedList<VarNode>,
) -> Result<(), String> {
    if arglist.len() != paramlist.len() {
        return Err(
            "Function call arguments and declaration arguments dont match in length.".to_owned(),
        );
    }
    let mut aiter = arglist.iter_mut();
    let mut piter = paramlist.iter_mut();

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
        ctr = ctr + 1;
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
