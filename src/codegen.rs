//register use table
use crate::parserlib::*;
use crate::validation::*;

use lazy_static::lazy_static; // 1.4.0
use std::cmp::max;
use std::cmp::min;
use std::collections::LinkedList;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::Mutex;

//global mutable arrays must be guarded with a mutex :(
//TODO Assignment statement can be optimized
//recursive call
const MAX_REGISTERS: usize = 21;
pub const XSM_STACK_OFFSET: i64 = 4096;
pub const LABEL_NOT_FOUND: usize = 10000;

//Global variables used
lazy_static! {
    pub static ref REGISTERS: Mutex<Vec<(bool, i64)>> = Mutex::new(vec![(false, 0); MAX_REGISTERS]);
    //Label assigner
    pub static ref LABEL_COUNT: Mutex<usize> = Mutex::new(0);
    //for continue and break statements
    pub static ref WHILE_TRACKER: Mutex<Vec<usize>> = Mutex::new(Vec::default());
    //Need a stack to call F(F(F(5))) type calls
    pub static ref REGISTER_STACK: Mutex<Vec<Vec<(bool, i64)>>> = Mutex::new(Vec::default());
    //TODO remove this stack
    pub static ref FSTACK: Mutex<(String,i64)> = Mutex::new((String::default(),0));
}

//Wrap error and write to file
fn write_line(mut writer: &File, args: std::fmt::Arguments) {
    if let Err(e) = writeln!(writer, "{}", args) {
        exit_on_err(e.to_string());
    }
}
/*
 * Get the size of the local declaration
 */
fn __get_function_storage(fname: &String) -> i64 {
    let ft = FUNCTION_TABLE.lock().unwrap();
    let mut max_size = 0;
    if let Some(entry) = ft.get(fname) {
        for (
            _k,
            LSymbol::Var {
                vartype: _,
                varid,
                varindices: _,
            },
        ) in entry.iter()
        {
            max_size = max(varid.clone(), max_size);
        }
        return max_size;
    } else {
        return 0;
    }
}
/*
 * Function to assign a register which has the lowest index
 */
pub fn get_reg() -> usize {
    let mut register = REGISTERS.lock().unwrap();
    for i in 0..MAX_REGISTERS {
        //lowest register number free is returned
        if register[i].0 == false {
            register[i].0 = true;
            return i.try_into().unwrap();
        }
    }
    return MAX_REGISTERS.try_into().unwrap();
}
// * Error handler
pub fn exit_on_err(err: String) {
    log::error!("{}", err);
    std::process::exit(-1);
}
/*
 * Function to free a given register, typically the highest
 * index is passed
 */
pub fn free_reg(register: usize) -> u64 {
    if register > 21 {
        return MAX_REGISTERS.try_into().unwrap();
    }
    let mut registers = REGISTERS.lock().unwrap();
    registers[register].0 = false;
    return MAX_REGISTERS.try_into().unwrap();
}
//function to push arguments
fn __push_args(file: &File, arglist: &LinkedList<ASTNode>, refr: bool) {
    for arg in arglist {
        let argreg = __code_gen(&arg, file, refr);
        write_line(file, format_args!("PUSH R{}", argreg));
    }
}
//function to backup live registers
fn __backup_registers(file: &File) {
    let mut registers = REGISTERS.lock().unwrap();
    let mut rs = REGISTER_STACK.lock().unwrap();
    rs.push(registers.clone());

    for i in 0..MAX_REGISTERS {
        if registers[i].0 == true {
            log::info!("R{} is being saved", i);
            write_line(file, format_args!("PUSH R{}", i));
        }
        registers[i].0 = false;
    }
}
//function to get a safe register for return_value of a function
fn __get_safe_register() -> usize {
    let mut rs = REGISTER_STACK.lock().unwrap();
    let registers = rs.last_mut().unwrap();
    for i in 0..MAX_REGISTERS {
        //lowest register number free is returned
        if registers[i].0 == false {
            registers[i].0 = true;
            return i;
        }
    }
    exit_on_err("Out of registers".to_string());
    0
}
//function to restore register context
fn __restore_register(file: &File, safe_register: usize) {
    let mut rs = REGISTER_STACK.lock().unwrap();
    let mut registers = rs.last().unwrap().clone();
    for i in (0..MAX_REGISTERS).rev() {
        if registers[i].0 == true && i != safe_register {
            write_line(file, format_args!("POP R{}", i));
        }
    }
    rs.pop();
    registers[safe_register].0 = true;
    let mut reg = REGISTERS.lock().unwrap();
    *reg = registers.clone();
    std::mem::drop(registers);
    //reset to not used
}
fn __load_variable(mut file: &File, vname: &String) -> usize {
    let lst = LOCALSYMBOLTABLE.lock().unwrap();
    if let Some(LSymbol::Var {
        vartype: _,
        varid,
        varindices: _,
    }) = lst.get(vname)
    {
        let vreg = get_reg();
        write_line(file, format_args!("MOV R{}, BP", vreg));
        if *varid < 0 {
            write_line(file, format_args!("SUB R{}, {}", vreg, -1 * varid));
        } else {
            write_line(file, format_args!("ADD R{}, {}", vreg, varid));
        }
        return vreg;
    }
    let gst = GLOBALSYMBOLTABLE.lock().unwrap();
    if let Some(GSymbol::Var {
        vartype: _,
        varid,
        varindices: _,
    }) = gst.get(vname)
    {
        let vreg = get_reg();
        if let Err(e) = writeln!(
            file,
            "MOV R{}, {}",
            vreg,
            XSM_STACK_OFFSET + i64::try_from(varid.clone()).unwrap()
        ) {
            exit_on_err(e.to_string())
        }
        return vreg;
    }

    exit_on_err("Variable not declared".to_string());
    0
}
/*
 * Meta function which recursively generates assembly lines
 * in xsm for arithmetic operations
 */
fn __code_gen(root: &ASTNode, mut file: &File, refr: bool) -> usize {
    match root {
        ASTNode::ErrorNode { err } => {
            let err: String = match err {
                ASTError::TypeError(s) => s.to_owned(),
            };
            exit_on_err(err);
            25
        }
        ASTNode::BreakpointNode => {
            write_line(file, format_args!("BRKP"));
            25
        }
        ASTNode::STR(s) => {
            let register = get_reg();
            let mut registers = REGISTERS.lock().unwrap();
            write_line(file, format_args!("MOV R{}, {}", register, s));
            registers[register].1 = 0;
            register
        }
        ASTNode::INT(n) => {
            let register = get_reg();
            let mut registers = REGISTERS.lock().unwrap();
            write_line(file, format_args!("MOV R{}, {}", register, n));
            registers[register].1 = *n;
            register
        }
        ASTNode::VAR {
            name,
            array_access: indices,
            dot_field_access,
            arrow_field_access,
        } => {
            let varid = getvarid(name).expect("Error in variable tables");
            let varindices = getvarindices(name).expect("Error in variable tables");

            let baseaddrreg = __load_variable(file, name);

            let mut registers = REGISTERS.lock().unwrap();
            registers[baseaddrreg].1 =
                i64::try_from(XSM_STACK_OFFSET).unwrap() + i64::try_from(varid).unwrap();
            std::mem::drop(registers);

            for i in 0..indices.len() {
                //Generate code for first index
                let offsetreg = __code_gen(&*indices[i], file, false);
                //Multiple unless its the last index
                //varindices because we need to handle a[2][2] with a[1] access as pointer
                if i != varindices.len() - 1 {
                    //Get register for multiplication
                    let indexmulreg = get_reg();
                    //Multiply with the corresponding declared index
                    write_line(file, format_args!("MUL R{}, {}", offsetreg, varindices[i]));
                    //free this for reuse
                    free_reg(indexmulreg);
                }
                //Add the offset
                let mut registers = REGISTERS.lock().unwrap();
                write_line(file, format_args!("ADD R{}, R{}", baseaddrreg, offsetreg));
                registers[baseaddrreg].1 += registers[offsetreg].1;
                std::mem::drop(registers);
                //Free this for reuse
                free_reg(offsetreg);
            }
            let mut dotptr = &**dot_field_access;
            let mut arrowptr = &**arrow_field_access;
            let mut currtype = getvartype(name).unwrap();
            for _ in 0..indices.len() {
                currtype = currtype.derefr().unwrap();
            }
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
                        let field_offset = currtype.get_field_id(nname).unwrap();
                        currtype = currtype.get_field_type(nname).unwrap();
                        write_line(file, format_args!("ADD R{}, {}", baseaddrreg, field_offset));
                        dotptr = &**dot_field_access;
                        arrowptr = &**arrow_field_access;
                        continue;
                    }
                }
                // check if dot field type is
                if arrowptr != &ASTNode::Null {
                    if let ASTNode::VAR {
                        name: nname,
                        array_access: _,
                        dot_field_access,
                        arrow_field_access,
                    } = arrowptr
                    {
                        if let ASTExprType::Pointer(etype) = &currtype {
                            write_line(
                                file,
                                format_args!("MOV R{}, [R{}]", baseaddrreg, baseaddrreg),
                            );
                            let field_offset = etype.get_field_id(nname).unwrap();
                            currtype = etype.get_field_type(nname).unwrap();
                            write_line(
                                file,
                                format_args!("ADD R{}, {}", baseaddrreg, field_offset),
                            );
                            dotptr = &**dot_field_access;
                            arrowptr = &**arrow_field_access;
                            continue;
                        }
                    }
                }
            }
            if refr == false && varindices.len() == indices.len() {
                write_line(
                    file,
                    format_args!("MOV R{}, [R{}]", baseaddrreg, baseaddrreg),
                );
            }
            return baseaddrreg;
        }
        ASTNode::BinaryNode {
            op,
            exprtype: _,
            lhs,
            rhs,
        } => {
            let result = match op {
                ASTNodeType::Gt => {
                    let left_register: usize = __code_gen(lhs, file, false).try_into().unwrap();
                    let right_register: usize = __code_gen(rhs, file, false).try_into().unwrap();
                    let mut registers = REGISTERS.lock().unwrap();
                    write_line(
                        file,
                        format_args!("GT R{}, R{}", left_register, right_register),
                    );
                    let result: i64 = (registers[left_register].1 > registers[right_register].1)
                        .try_into()
                        .unwrap();
                    let lower_register = min(left_register, right_register);
                    registers[lower_register].1 = result;
                    // release mutex for global array so that register can be freed
                    std::mem::drop(registers);
                    free_reg(left_register + right_register - lower_register);
                    lower_register
                }
                ASTNodeType::Lt => {
                    let left_register: usize = __code_gen(lhs, file, false).try_into().unwrap();
                    let right_register: usize = __code_gen(rhs, file, false).try_into().unwrap();
                    let mut registers = REGISTERS.lock().unwrap();
                    write_line(
                        file,
                        format_args!("LT R{}, R{}", left_register, right_register),
                    );
                    let result: i64 = (registers[left_register].1 < registers[right_register].1)
                        .try_into()
                        .unwrap();
                    let lower_register = min(left_register, right_register);
                    registers[lower_register].1 = result;
                    // release mutex for global array so that register can be freed
                    std::mem::drop(registers);
                    free_reg(left_register + right_register - lower_register);
                    lower_register
                }
                ASTNodeType::Gte => {
                    let left_register: usize = __code_gen(lhs, file, false).try_into().unwrap();
                    let right_register: usize = __code_gen(rhs, file, false).try_into().unwrap();
                    let mut registers = REGISTERS.lock().unwrap();
                    write_line(
                        file,
                        format_args!("GE R{}, R{}", left_register, right_register),
                    );
                    let result: i64 = (registers[left_register].1 >= registers[right_register].1)
                        .try_into()
                        .unwrap();
                    let lower_register = min(left_register, right_register);
                    registers[lower_register].1 = result;
                    // release mutex for global array so that register can be freed
                    std::mem::drop(registers);
                    free_reg(left_register + right_register - lower_register);
                    lower_register
                }
                ASTNodeType::Lte => {
                    let left_register: usize = __code_gen(lhs, file, false).try_into().unwrap();
                    let right_register: usize = __code_gen(rhs, file, false).try_into().unwrap();
                    let mut registers = REGISTERS.lock().unwrap();
                    write_line(
                        file,
                        format_args!("LE R{}, R{}", left_register, right_register),
                    );
                    let result: i64 = (registers[left_register].1 <= registers[right_register].1)
                        .try_into()
                        .unwrap();
                    let lower_register = min(left_register, right_register);
                    registers[lower_register].1 = result;
                    // release mutex for global array so that register can be freed
                    std::mem::drop(registers);
                    free_reg(left_register + right_register - lower_register);
                    lower_register
                }
                ASTNodeType::Ee => {
                    let left_register: usize = __code_gen(lhs, file, false).try_into().unwrap();
                    let right_register: usize = __code_gen(rhs, file, false).try_into().unwrap();
                    let mut registers = REGISTERS.lock().unwrap();
                    write_line(
                        file,
                        format_args!("EQ R{}, R{}", left_register, right_register),
                    );
                    let result: i64 = (registers[left_register].1 == registers[right_register].1)
                        .try_into()
                        .unwrap();
                    let lower_register = min(left_register, right_register);
                    registers[lower_register].1 = result;
                    // release mutex for global array so that register can be freed
                    std::mem::drop(registers);
                    free_reg(left_register + right_register - lower_register);
                    lower_register
                }
                ASTNodeType::Ne => {
                    let left_register: usize = __code_gen(lhs, file, false).try_into().unwrap();
                    let right_register: usize = __code_gen(rhs, file, false).try_into().unwrap();
                    let mut registers = REGISTERS.lock().unwrap();
                    write_line(
                        file,
                        format_args!("NE R{}, R{}", left_register, right_register),
                    );
                    let result: i64 = (registers[left_register].1 != registers[right_register].1)
                        .try_into()
                        .unwrap();
                    let lower_register = min(left_register, right_register);
                    registers[lower_register].1 = result;
                    // release mutex for global array so that register can be freed
                    std::mem::drop(registers);
                    free_reg(left_register + right_register - lower_register);
                    lower_register
                }
                ASTNodeType::Plus => {
                    let left_register: usize = __code_gen(lhs, file, false).try_into().unwrap();
                    let right_register: usize = __code_gen(rhs, file, false).try_into().unwrap();
                    let mut registers = REGISTERS.lock().unwrap();
                    write_line(
                        file,
                        format_args!("ADD R{}, R{}", left_register, right_register),
                    );
                    let result: i64 = registers[left_register].1 + registers[right_register].1;
                    let lower_register = min(left_register, right_register);
                    registers[lower_register].1 = result;
                    // release mutex for global array so that register can be freed
                    std::mem::drop(registers);
                    free_reg(left_register + right_register - lower_register);
                    lower_register
                }
                ASTNodeType::Minus => {
                    let left_register: usize = __code_gen(lhs, file, false).try_into().unwrap();
                    let right_register: usize = __code_gen(rhs, file, false).try_into().unwrap();
                    let mut registers = REGISTERS.lock().unwrap();
                    write_line(
                        file,
                        format_args!("SUB R{}, R{}", left_register, right_register),
                    );
                    let result: i64 = registers[left_register].1 - registers[right_register].1;
                    let lower_register = min(left_register, right_register);
                    registers[lower_register].1 = result;
                    // release mutex for global array so that register can be freed
                    std::mem::drop(registers);
                    free_reg(left_register + right_register - lower_register);
                    lower_register
                }
                ASTNodeType::Star => {
                    let left_register: usize = __code_gen(lhs, file, false).try_into().unwrap();
                    let right_register: usize = __code_gen(rhs, file, false).try_into().unwrap();
                    let mut registers = REGISTERS.lock().unwrap();
                    write_line(
                        file,
                        format_args!("MUL R{}, R{}", left_register, right_register),
                    );
                    let result: i64 = registers[left_register].1 * registers[right_register].1;
                    let lower_register = min(left_register, right_register);
                    registers[lower_register].1 = result;
                    // release mutex for global array so that register can be freed
                    std::mem::drop(registers);
                    free_reg(left_register + right_register - lower_register);
                    lower_register
                }
                ASTNodeType::Slash => {
                    let left_register: usize = __code_gen(lhs, file, false).try_into().unwrap();
                    let right_register: usize = __code_gen(rhs, file, false).try_into().unwrap();
                    let mut registers = REGISTERS.lock().unwrap();
                    write_line(
                        file,
                        format_args!("DIV R{}, R{}", left_register, right_register),
                    );
                    let result: i64 = registers[left_register].1 / registers[right_register].1;
                    let lower_register = min(left_register, right_register);
                    registers[lower_register].1 = result;
                    // release mutex for global array so that register can be freed
                    std::mem::drop(registers);
                    free_reg(left_register + right_register - lower_register);
                    lower_register
                }
                ASTNodeType::Mod => {
                    let left_register: usize = __code_gen(lhs, file, false).try_into().unwrap();
                    let right_register: usize = __code_gen(rhs, file, false).try_into().unwrap();
                    let mut registers = REGISTERS.lock().unwrap();
                    write_line(
                        file,
                        format_args!("MOD R{}, R{}", left_register, right_register),
                    );
                    let result: i64 = registers[left_register].1 % registers[right_register].1;
                    let lower_register = min(left_register, right_register);
                    registers[lower_register].1 = result;
                    // release mutex for global array so that register can be freed
                    std::mem::drop(registers);
                    free_reg(left_register + right_register - lower_register);
                    lower_register
                }
                ASTNodeType::Equals => {
                    let left_register: usize = __code_gen(lhs, file, true).try_into().unwrap();
                    let right_register: usize = __code_gen(rhs, file, false).try_into().unwrap();
                    write_line(
                        file,
                        format_args!("MOV [R{}], R{}", left_register, right_register),
                    );
                    free_reg(left_register);
                    free_reg(right_register);
                    25
                }
                ASTNodeType::Connector => {
                    let res = __code_gen(lhs, file, false);
                    free_reg(res);
                    let res = __code_gen(rhs, file, false);
                    free_reg(res);
                    25
                }
                _ => 0,
            };
            result
        }
        ASTNode::UnaryNode {
            op,
            exprtype: _,
            ptr,
            depth,
        } => match op {
            ASTNodeType::Read => {
                __backup_registers(file);
                let register = get_reg();
                write_line(file, format_args!("MOV R{}, \"Read\"", register));
                write_line(file, format_args!("PUSH R{}", register));
                write_line(file, format_args!("MOV R{}, -1", register));
                write_line(file, format_args!("PUSH R{}", register));
                free_reg(register);
                let register: usize = __code_gen(ptr, file, true).try_into().unwrap();
                write_line(file, format_args!("PUSH R{}", register));
                write_line(file, format_args!("ADD SP, 2"));
                write_line(file, format_args!("CALL 0"));
                let ret_reg = __get_safe_register();
                write_line(file, format_args!("POP R{}", ret_reg));
                write_line(file, format_args!("SUB SP, 4"));
                __restore_register(file, ret_reg);
                ret_reg
            }
            ASTNodeType::Write => {
                __backup_registers(file);
                let register = get_reg();
                write_line(file, format_args!("MOV R{}, \"Write\"", register));
                write_line(file, format_args!("PUSH R{}", register));
                write_line(file, format_args!("MOV R{}, -2", register));
                write_line(file, format_args!("PUSH R{}", register));
                free_reg(register);
                let variable: usize = __code_gen(ptr, file, false).try_into().unwrap();
                write_line(file, format_args!("PUSH R{}", variable));
                write_line(file, format_args!("ADD SP, 2"));
                write_line(file, format_args!("CALL 0"));
                let ret_reg = __get_safe_register();
                write_line(file, format_args!("POP R{}", ret_reg));
                write_line(file, format_args!("SUB SP, 4"));
                __restore_register(file, ret_reg);
                ret_reg
            }
            ASTNodeType::Ref => {
                match &**ptr {
                    ASTNode::VAR {
                        name: _,
                        array_access: _,
                        dot_field_access: _,
                        arrow_field_access: _,
                    } => {
                        let regaddr: usize = __code_gen(ptr, file, true).try_into().unwrap();
                        return regaddr;
                    }
                    _ => {
                        exit_on_err("Reference to a non variable is not allowed".to_string());
                    }
                }
                0
            }
            ASTNodeType::Deref => match &**ptr {
                ASTNode::VAR {
                    name: _,
                    array_access: _,
                    dot_field_access: _,
                    arrow_field_access: _,
                } => {
                    let regaddr: usize = __code_gen(ptr, file, refr).try_into().unwrap();
                    for _i in 0..depth.unwrap() {
                        write_line(file, format_args!("MOV R{},[R{}]", regaddr, regaddr));
                    }
                    return regaddr;
                }
                _ => {
                    exit_on_err("Cannot dereference a variable".to_string());
                    0
                }
            },
            _ => 0,
        },
        ASTNode::FuncCallNode { fname, arglist } => {
            //Save Live registers except ret_reg
            __backup_registers(file);
            //Push Arguments
            __push_args(file, arglist, refr);
            //Push return value
            write_line(file, format_args!("ADD SP, {}", 1));
            let gst = GLOBALSYMBOLTABLE.lock().unwrap();
            if let Some(entry) = gst.get(fname) {
                match entry {
                    GSymbol::Func {
                        ret_type: _,
                        paramlist: _,
                        flabel,
                    } => {
                        write_line(file, format_args!("CALL L{}", flabel));
                    }
                    _ => exit_on_err("Function not declared".to_string()),
                }
            }
            let ret_reg = __get_safe_register();
            //extract return register
            write_line(file, format_args!("POP R{}", ret_reg));
            //remove arguments
            write_line(file, format_args!("SUB SP, {}", (&**arglist).len()));
            //Restore live registers except_ret_reg
            __restore_register(file, ret_reg);
            ret_reg
        }
        ASTNode::ReturnNode { expr } => {
            let fs = FSTACK.lock().unwrap();
            let (_fname, storage) = fs.clone();
            std::mem::drop(fs);

            let retreg = __code_gen(expr, file, refr);
            let reg = get_reg();

            write_line(file, format_args!("MOV R{}, BP", reg));
            write_line(file, format_args!("SUB R{}, 2", reg));
            write_line(file, format_args!("MOV [R{}], R{}", reg, retreg));
            write_line(file, format_args!("SUB SP, {}\nPOP BP", storage));
            write_line(file, format_args!("RET"));

            let mut registers = REGISTERS.lock().unwrap();
            *registers = vec![(false, 0); MAX_REGISTERS];
            std::mem::drop(registers);
            25
        }
        ASTNode::MainNode { body } => {
            //this node is traverse after all function def nodes,
            write_line(
                file,
                format_args!("L{}:", get_function_label(&"main".to_owned())),
            );
            let ft = FUNCTION_TABLE.lock().unwrap();
            if let Some(local_table) = ft.get("main") {
                let mut lst = LOCALSYMBOLTABLE.lock().unwrap();
                *lst = local_table.clone();
                std::mem::drop(ft);
                std::mem::drop(lst);

                write_line(file, format_args!("PUSH BP\nMOV BP,SP",));
                write_line(
                    file,
                    format_args!("ADD SP, {}", __get_function_storage(&"main".to_owned())),
                );
                //idk
                let mut fs = FSTACK.lock().unwrap();
                *fs = (
                    "main".to_string(),
                    __get_function_storage(&"main".to_owned()),
                );
                std::mem::drop(fs);

                __backup_registers(file);

                __code_gen(body, file, false);

                let mut registers = REGISTERS.lock().unwrap();
                *registers = vec![(false, 0); MAX_REGISTERS];
                std::mem::drop(registers);

                __xsm_exit_syscall(file);
            } else {
                exit_on_err("main not defined".to_string())
            }
            25
        }
        /*
         * L{funclabel}:
         *    Subtract SP by declvars.size()
         *    <body>
         *    ret
         */
        ASTNode::FuncDefNode {
            fname,
            ret_type: _,
            paramlist: _,
            body,
        } => {
            write_line(
                file,
                format_args!("L{}:\nBRKP\nPUSH BP\nMOV BP,SP", get_function_label(fname)),
            );
            write_line(
                file,
                format_args!("ADD SP, {}", __get_function_storage(fname)),
            );
            let ft = FUNCTION_TABLE.lock().unwrap();
            if let Some(_local_table) = ft.get(fname) {
                let mut lst = LOCALSYMBOLTABLE.lock().unwrap();
                *lst = _local_table.clone();
                let mut fs = FSTACK.lock().unwrap();
                std::mem::drop(ft);
                std::mem::drop(lst);
                *fs = (fname.clone(), __get_function_storage(fname));
                std::mem::drop(fs);

                __code_gen(&**body, file, false);
            }
            25
        }
        /*
         * <expr>
         * <cond>
         * <jz> L1
         * <if>
         * jmp L2
         * L1:
         * <else>
         * L2:
         */
        ASTNode::IfElseNode { expr, xif, xelse } => {
            let mut label_count = LABEL_COUNT.lock().unwrap();
            let l1 = label_count.clone();
            (*label_count) += 1;
            let l2 = label_count.clone();
            (*label_count) += 1;
            //drop for handling nested cases
            std::mem::drop(label_count);
            let result: usize = __code_gen(expr, file, false).try_into().unwrap();
            //Generate code for the expression
            write_line(file, format_args!("JZ R{}, L{}", result, l1));
            //Free the register
            free_reg(result);
            //Drop label_count so that nested cases can be handled
            //generate if case flow
            __code_gen(xif, file, false);
            //result is 0 as xif is a stmtlist
            //Jmp to L2 if its else case
            write_line(file, format_args!("JMP L{}", l2));
            //add label count for exit case
            write_line(file, format_args!("L{}:", l1));
            __code_gen(xelse, file, false);
            write_line(file, format_args!("L{}:", l2));
            25
        }
        /* While Node
         * L1:
         * <expr>
         * <cond>
         * <jz> L2
         * <do>
         * <jmp> L1
         * L2:
         */
        ASTNode::WhileNode { expr, xdo } => {
            let mut label_count = LABEL_COUNT.lock().unwrap();
            let mut while_tracker = WHILE_TRACKER.lock().unwrap();

            let l1 = label_count.clone();
            //Create a new label
            write_line(file, format_args!("L{}:", l1));
            (*label_count) += 1;

            let l2 = label_count.clone();
            (*label_count) += 1;

            while_tracker.push(l1);
            while_tracker.push(l2);
            //Drop label_count so that nested cases can be handled
            std::mem::drop(label_count);
            std::mem::drop(while_tracker);
            let result: usize = __code_gen(expr, file, false).try_into().unwrap();
            //Generate code for the expression
            write_line(file, format_args!("JZ R{}, L{}", result, l2));
            //Free the register
            free_reg(result);
            //generate if case flow
            //result is 0 as xif is a stmtlist
            __code_gen(xdo, file, false);
            //while loop it back to top condition

            let mut while_tracker = WHILE_TRACKER.lock().unwrap();
            while_tracker.pop();
            while_tracker.pop();
            write_line(file, format_args!("JMP L{}", l1));
            //add label count for exit case
            write_line(file, format_args!("L{}:", l2));
            //increment label_count
            25
        }
        /* If Node
         * <expr>
         * <cond>
         * <jz> L1
         * <ifcase>
         * L1:
         */
        ASTNode::IfNode { expr, xif } => {
            let mut label_count = LABEL_COUNT.lock().unwrap();
            let l1 = label_count.clone();
            (*label_count) += 1;
            //Drop label_count so that nested cases can be handled
            std::mem::drop(label_count);
            let result: usize = __code_gen(expr, file, false).try_into().unwrap();
            //Generate code for the expression
            write_line(file, format_args!("JZ R{}, L{}", result, l1));
            //Free the register
            free_reg(result);
            //generate if case flow
            __code_gen(xif, file, false);
            //result is 0 as xif is a stmtlist
            write_line(file, format_args!("L{}:", l1));
            //increment label_count
            25
        }
        /*
         * L1:
         * ...
         * <expr>
         * <cond>
         * <jz> L1
         * L2:
         */
        ASTNode::BreakNode => {
            let while_tracker = WHILE_TRACKER.lock().unwrap();
            writeln!(file, "JMP L{}", while_tracker[while_tracker.len() - 1])
                .expect("[code_gen] Write error");
            25
        }
        ASTNode::ContinueNode => {
            let while_tracker = WHILE_TRACKER.lock().unwrap();
            writeln!(file, "JMP L{}", while_tracker[while_tracker.len() - 2])
                .expect("[code_gen] Write error");
            25
        }
        ASTNode::Null => 25,
    }
}

/*
 * Meta function to generate header compatible to XSM ABI Standard
 */
fn __header_gen(mut file: &File) {
    let mut gst = GLOBALSYMBOLTABLE.lock().unwrap();
    log::info!("Global Symbol Table Size : {}", gst.len());
    let mut label_count = LABEL_COUNT.lock().unwrap();
    let l = label_count.clone();
    *label_count += 1;
    //in case main() is recursively called, we need the label of main
    gst.insert(
        "main".to_string(),
        GSymbol::Func {
            ret_type: (ASTExprType::Primitive(PrimitiveType::Int)),
            paramlist: LinkedList::new(),
            flabel: (l),
        },
    );
    let mut baseaddr = 0;
    for (_k, v) in gst.iter() {
        match v {
            GSymbol::Var {
                vartype: _,
                varid: _,
                varindices,
            } => {
                let mut size = 1;
                for index in varindices {
                    size *= index;
                }
                baseaddr = baseaddr + size;
            }
            _ => continue,
        }
    }
    writeln!(
        file,
        "0\n2056\n0\n0\n0\n0\n0\n0\nBRKP\nMOV SP, 4095\nADD SP, {baseaddr}\nMOV BP,SP\nADD SP,1\nCALL L{l}\nSUB SP,1\nPUSH R0\nINT 10",
    )
    .unwrap();
}

/*
 * Meta function to generate xsm code for Write Syscall of expos
 */
fn __xsm_write_syscall(file: &File, var: usize) -> usize {
    __backup_registers(file);
    let register = get_reg();
    write_line(file, format_args!("MOV R{}, \"Write\"", register));
    write_line(file, format_args!("PUSH R{}", register));
    write_line(file, format_args!("MOV R{}, -2", register));
    write_line(file, format_args!("PUSH R{}", register));
    write_line(file, format_args!("PUSH R{}", var));
    write_line(file, format_args!("ADD SP,2"));
    write_line(file, format_args!("CALL 0"));
    write_line(file, format_args!("POP R{}", register));
    write_line(file, format_args!("SUB SP,4"));
    __restore_register(file, register);
    register
}

/*
 * Meta function to generate xsm code for Read Syscall of expos
 */

/*
 * Meta function to generate xsm code for Exit Syscall of expos
 */
fn __xsm_exit_syscall(file: &File) {
    let register = get_reg();
    write_line(file, format_args!("PUSH R0\nINT 10"));
    free_reg(register);
}
fn __print_gst() {
    let gst = GLOBALSYMBOLTABLE.lock().unwrap();

    log::info!("Global symbol table has {} symbols", gst.len());
}
pub fn code_gen(root: &ASTNode, filename: String) -> usize {
    let f = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(filename.as_str());

    match f {
        Ok(file) => {
            file.set_len(0)
                .expect("[code_gen] Error truncating existing file");
            __header_gen(&file);
            let result: usize = __code_gen(root, &file, false);
            write_line(&file, format_args!("PUSH R{}", result));
            __xsm_exit_syscall(&file);
            log::trace!("Generated Object file: {}", filename.as_str());
            result
        }
        Err(e) => {
            exit_on_err(e.to_string());
            1
        }
    }
}
