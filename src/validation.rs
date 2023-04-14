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
    let _actual_array_type = parent_type.get_field_type(array_name)?;
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
            ASTNode::StdFuncCallNode { func, arglist } => match func {
                STDLibFunction::Syscall => {
                    //check if first value is an integer
                    if arglist.len() != 5 {
                        log::error!("got {}", arglist.len());
                        return Err("[Syscall] system call expects 5 arguments. ".to_owned());
                    }
                    let mut iter = arglist.iter_mut();
                    if let Some(i) = iter.next() {
                        if i.getexprtype() != Some(ASTExprType::Primitive(PrimitiveType::Int)) {
                            return Err(
                                "[Syscall] system call number must be an int type.".to_owned()
                            );
                        }
                    }
                    if let Some(i) = iter.next() {
                        if let ASTNode::INT(_) = i {
                        } else {
                            return Err(
                                "[Syscall] interrupt routine number must be an integer.".to_owned()
                            );
                        }
                    }
                    Ok(())
                    //check if sec
                }
                STDLibFunction::Setaddr => {
                    if arglist.len() != 2 {
                        return Err("[Setaddr] expects 2 arguments.".to_owned());
                    }
                    let mut iter = arglist.iter_mut();
                    if let Some(i) = iter.next() {
                        if let Some(t) = i.getexprtype() {
                            if t.get_base_type() != ASTExprType::Primitive(PrimitiveType::Int) {
                                return Err("[Setaddr] the first argument is not a pointer or raw address expression.".to_owned());
                            }
                        }
                    }
                    Ok(())
                }
                STDLibFunction::Getaddr => {
                    if arglist.len() != 1 {
                        return Err("[Getaddr] expects 1 argument.".to_owned());
                    }
                    let mut iter = arglist.iter_mut();
                    if let Some(i) = iter.next() {
                        if let Some(t) = i.getexprtype() {
                            if t.get_base_type() != ASTExprType::Primitive(PrimitiveType::Int) {
                                return Err("[Getaddr] the first argument is not a pointer or raw address expression.".to_owned());
                            }
                        }
                    }
                    Ok(())
                }
                STDLibFunction::New => {
                    if arglist.len() != 1 {
                        return Err("[New] expects 1 argument.".to_owned());
                    }
                    match arglist.front().unwrap() {
                        ASTNode::VAR {
                            name,
                            array_access,
                            dot_field_access,
                            arrow_field_access,
                        } => {
                            if !(**dot_field_access == ASTNode::Void
                                && **arrow_field_access == ASTNode::Void
                                && array_access.len() == 0)
                            {
                                return Err("[New] Expected a class_name".to_owned());
                            }
                            if let Ok(ASTExprType::Class(_)) =
                                TYPE_TABLE.lock().unwrap().tt_get_type(&name)
                            {
                                return Ok(());
                            }
                            return Err("[New] got a class_name which is not declared".to_owned());
                        }
                        _ => return Err("[New] Expected a declared class_name".to_owned()),
                    }
                }
                _ => Err("Std function Unimplemented!".to_owned()),
            },
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
                    if dotptr == &ASTNode::Void && arrowptr == &ASTNode::Void {
                        break;
                    }
                    match dotptr {
                        ASTNode::VAR {
                            name: nname,
                            array_access,
                            dot_field_access,
                            arrow_field_access,
                        } => {
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
                        }
                        ASTNode::FuncCallNode { fname, arglist } => {
                            //check if currtype is class
                            //check if fname is in currtype
                            //set currtype as return value of fname
                            if !currtype.is_class() {
                                return Err("Type [".to_owned()
                                    + name.as_str()
                                    + "] is not a class type to call methods.");
                            }
                            currtype.is_method(fname, arglist)?;
                            break;
                        }
                        ASTNode::Void => {}
                        _ => {
                            return Err("Dot operator can only be used to access [struct_t] types"
                                .to_owned());
                        }
                    }
                    match arrowptr {
                        ASTNode::VAR {
                            name: nname,
                            array_access,
                            dot_field_access,
                            arrow_field_access,
                        } => {
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
                                return Err(
                                    "Arrow operator can only be used to pointer types".to_owned()
                                );
                            }
                            //validate_field_array_access(nname, &currtype, array_access)?;
                        }
                        ASTNode::FuncCallNode { fname, arglist } => {
                            //check if currtype is class
                            //check if fname is in currtype
                            if let ASTExprType::Pointer(etype) = &currtype {
                                if !etype.is_class() {
                                    return Err("Type [".to_owned()
                                        + name.as_str()
                                        + "] is not a class type to call methods.");
                                }
                                etype.is_method(fname, arglist)?;
                                break;
                            } else {
                                return Err(
                                    "Arrow operator can only be used to pointer types".to_owned()
                                );
                            }
                        }
                        ASTNode::Void => {}
                        _ => return Err("Arrow operator expects a field/method".to_owned()),
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
            ASTNode::IfElseNode { expr, .. } => {
                if expr.getexprtype() != Some(ASTExprType::Primitive(PrimitiveType::Bool)) {
                    return Err("Invalid expression inside if else's condition.".to_owned());
                }
                Ok(())
            }
            ASTNode::ReturnNode { expr } => {
                let ct = RET_TYPE.lock().unwrap().clone();
                let b = expr.getexprtype();
                if let ASTExprType::Class(_) = b.as_ref().unwrap().get_base_type() {
                    return Err("Stage 8 doesnt allow classes to be returned".to_owned());
                }
                if b == Some(ASTExprType::Primitive(PrimitiveType::Null)) {
                    if let ASTExprType::Pointer(_) = ct {
                        return Ok(());
                    }
                }
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
                ASTNodeType::Free => {
                    if let Some(ASTExprType::Pointer(_)) = ptr.getexprtype() {
                        Ok(())
                    } else {
                        Err("Free expects a pointer type.".to_owned())
                    }
                }
                ASTNodeType::Initialize => {
                    if *INITFLAG.lock().unwrap() {
                        return Err("Initialize should only be called once".to_owned());
                    }
                    *INITFLAG.lock().unwrap() = true;
                    Ok(())
                }
                ASTNodeType::Alloc => match &**ptr {
                    ASTNode::VAR { .. } => {
                        if let Some(ASTExprType::Pointer(_)) = ptr.getexprtype() {
                            Ok(())
                        } else {
                            Err("Alloc can only be used on pointer types.".to_owned())
                        }
                    }
                    _ => Err("Alloc expects a declared variable.".to_owned()),
                },
                ASTNodeType::Deref => {
                    if let Some(ptrtype) = ptr.getexprtype() {
                        if ptrtype.depth() < depth.unwrap() {
                            return Err("Dereferencing non pointer type.".to_owned());
                        }
                    }
                    self.getexprtype();
                    Ok(())
                }
                ASTNodeType::Ref => match &**ptr {
                    ASTNode::VAR {
                        name, array_access, ..
                    } => {
                        let varindices = getvarindices(name).unwrap();
                        if array_access.len() != varindices.len() {
                            return Err("Reference operator can only reference to the basetype of an array.".to_owned());
                        }
                        Ok(())
                    }
                    _ => Err("Reference operator expects a declared variable.".to_owned()),
                },
                ASTNodeType::Write => {
                    ptr.validate()?;
                    match &**ptr {
                        ASTNode::VAR { .. } => Ok(()),
                        ASTNode::INT(_) => Ok(()),
                        ASTNode::STR(_) => Ok(()),
                        ASTNode::BinaryNode {
                            op: _, exprtype, ..
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
                        match (lhs_t, rhs_t) {
                            (
                                Some(ASTExprType::Pointer(..)),
                                Some(ASTExprType::Primitive(PrimitiveType::Null)),
                            ) => Ok(()),
                            (Some(ASTExprType::Pointer(c1)), Some(ASTExprType::Pointer(c2))) => {
                                match (&*c1, &*c2) {
                                    (ASTExprType::Class(a), ASTExprType::Class(_)) => {
                                        if c2.is_ancestor(Some(a.name.clone())) {
                                            Ok(())
                                        } else {
                                            Err("Assignment of invalid type.".to_owned())
                                        }
                                    }
                                    _ => Err("Assignment of invalid type.".to_owned()),
                                }
                            }
                            _ => Err("Assignment of invalid type.".to_owned()),
                        }
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
                    if expr != None && expr != Some(ASTExprType::Primitive(PrimitiveType::Void)) {
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
                            ..
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
                std::mem::drop(gst);
                compare_arglist_paramlist(fname, arglist, &mut p)
            }
            ASTNode::FuncDefNode {
                fname,
                ret_type: r1,
                body: _,
                paramlist: a,
            } => {
                let cn = CLASSNAME.lock().unwrap();
                if cn.len() > 0 {
                    let classentry = TYPE_TABLE.lock().unwrap().tt_get_type(&*cn)?;
                    let ce = classentry.clone();
                    match classentry {
                        ASTExprType::Class(c) => {
                            if let Some(m) = c.symbol_table.table.get(fname) {
                                match m {
                                    CSymbol::Func {
                                        name: _,
                                        ret_type: r2,
                                        paramlist: b,
                                        ..
                                    } => {
                                        if r1 != r2 {
                                            return Err("Function [".to_owned()
                                        + fname.as_str()
                                        + "]'s return type doesn't match in it declaration");
                                        }
                                        let mut l = b.clone();
                                        l.push_front(VarNode {
                                            varname: "self".to_owned(),
                                            vartype: ASTExprType::Pointer(Box::new(ce)),
                                            varindices: vec![],
                                        });
                                        if a != &l {
                                            return Err("Function [".to_owned()
                                        + fname.as_str()
                                        + "]'s parameter list doesn't match in it declaration");
                                        }
                                        Ok(())
                                    }
                                    _ => {
                                        return Err("Function [".to_owned()
                                            + fname.as_str()
                                            + "] is declared as a field.");
                                    }
                                }
                            } else {
                                Err("Func with name [".to_owned()
                                    + fname
                                    + "] is not declared in class ["
                                    + cn.as_str()
                                    + "]")
                            }
                        }
                        _ => Err("Func def must be inside classdef.".to_owned()),
                    }
                } else {
                    let gst = GLOBALSYMBOLTABLE.lock().unwrap();
                    if let Some(entry) = gst.get(&fname.clone()) {
                        match entry {
                            GSymbol::Var { .. } => Err("Function with name [".to_owned()
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
                        Err("Function with name [".to_owned()
                            + fname.as_str()
                            + "] is not declared")
                    }
                }
            }
            _ => Ok(()),
        }
    }
    pub fn getexprtype(&mut self) -> Option<ASTExprType> {
        match self {
            ASTNode::StdFuncCallNode { func, arglist } => match func {
                STDLibFunction::Heapset => Some(ASTExprType::Primitive(PrimitiveType::Int)),
                STDLibFunction::Free => Some(ASTExprType::Primitive(PrimitiveType::Int)),
                STDLibFunction::Alloc => Some(ASTExprType::Primitive(PrimitiveType::Int)),
                STDLibFunction::Getaddr => Some(ASTExprType::Primitive(PrimitiveType::Int)),
                STDLibFunction::Setaddr => Some(ASTExprType::Primitive(PrimitiveType::Void)),
                STDLibFunction::Syscall => Some(ASTExprType::Primitive(PrimitiveType::Void)),
                STDLibFunction::Read => Some(ASTExprType::Primitive(PrimitiveType::Void)),
                STDLibFunction::Write => Some(ASTExprType::Primitive(PrimitiveType::Void)),
                STDLibFunction::New => match (&**arglist).front().unwrap() {
                    ASTNode::VAR { name, .. } => {
                        if let Ok(p) = TYPE_TABLE.lock().unwrap().tt_get_type(&name) {
                            return Some(ASTExprType::Pointer(Box::new(p)));
                        }
                        return None;
                    }
                    _ => {
                        unreachable!()
                    }
                },
            },
            ASTNode::ErrorNode { err } => match err {
                ASTError::TypeError(s) => {
                    exit_on_err(s.to_owned());
                    Some(ASTExprType::Primitive(PrimitiveType::Void))
                }
            },
            ASTNode::Null => Some(ASTExprType::Primitive(PrimitiveType::Null)),
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
                        if dotptr == &ASTNode::Void && arrowptr == &ASTNode::Void {
                            break;
                        }
                        match dotptr {
                            ASTNode::VAR {
                                name: nname,
                                array_access,
                                dot_field_access,
                                arrow_field_access,
                            } => {
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
                            ASTNode::FuncCallNode { fname, arglist } => {
                                vtype = vtype.is_method(fname, arglist).unwrap();
                                break;
                            }
                            ASTNode::Void => {}
                            _ => {
                                unreachable!()
                            }
                        }
                        match arrowptr {
                            ASTNode::VAR {
                                name: nname,
                                array_access,
                                dot_field_access,
                                arrow_field_access,
                            } => {
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
                            ASTNode::FuncCallNode { fname, arglist } => {
                                if let ASTExprType::Pointer(etype) = &vtype {
                                    vtype = etype.is_method(fname, arglist).unwrap();
                                    break;
                                } else {
                                    exit_on_err(
                                        "Arrow operator expects a variable of struct_t*".to_owned(),
                                    );
                                }
                            }
                            _ => unreachable!(),
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
                _ => Some(ASTExprType::Primitive(PrimitiveType::Void)),
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
                                ASTExprType::Primitive(PrimitiveType::Null),
                                ASTExprType::Pointer(..),
                            ) => Some(ASTExprType::Primitive(PrimitiveType::Bool)),
                            (
                                ASTExprType::Pointer(..),
                                ASTExprType::Primitive(PrimitiveType::Null),
                            ) => Some(ASTExprType::Primitive(PrimitiveType::Bool)),
                            (
                                ASTExprType::Primitive(PrimitiveType::Int),
                                ASTExprType::Primitive(PrimitiveType::Int),
                            ) => Some(ASTExprType::Primitive(PrimitiveType::Bool)),
                            (
                                ASTExprType::Primitive(PrimitiveType::String),
                                ASTExprType::Primitive(PrimitiveType::String),
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
                _ => Some(ASTExprType::Primitive(PrimitiveType::Void)),
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
        vartype: _, varid, ..
    }) = lst.get(name)
    {
        return Some(*varid);
    }
    let gst = GLOBALSYMBOLTABLE.lock().unwrap();
    if let Some(GSymbol::Var {
        vartype: _, varid, ..
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
                GSymbol::Var { .. } => Ok(()),
                GSymbol::Func { .. } => {
                    Err("Symbol [".to_owned() + name.as_str() + "] declared as a function.")
                }
            }
        } else {
            Err("Symbol [".to_owned() + name.as_str() + "] is not declared.")
        }
    }
}
/*
 * Function to validate the pamalist in declaration to definition
 */
pub fn compare_arglist_paramlist(
    fname: &mut String,
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
            return Err("Function [".to_owned()
                + fname.as_str()
                + "] call arguments and declaration arguments dont match in type at ["
                + ctr.to_string().as_str()
                + "] position.");
        }
        ctr = ctr + 1;
    }
    Ok(())
}
