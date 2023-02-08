//register use table
use crate::parserlib::*;
use crate::validation::*;

use lazy_static::lazy_static; // 1.4.0
use std::cmp::min;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::Mutex;

//global mutable arrays must be guarded with a mutex :(
//TODO maybe initalize this array somewhere and then pass the array as reference to each code_gen
//TODO Assignment statement can be optimized
//recursive call
const MAX_REGISTERS: usize = 21;
pub const XSM_STACK_OFFSET: i64 = 4096;
pub const LABEL_NOT_FOUND: usize = 10000;

lazy_static! {
    pub static ref REGISTERS: Mutex<Vec<(bool, i64)>> = Mutex::new(vec![(false, 0); MAX_REGISTERS]);
    pub static ref VARIABLE_REGISTER_MAP: Mutex<HashMap<usize, usize>> =
        Mutex::new(HashMap::default());
    pub static ref LABEL_COUNT: Mutex<usize> = Mutex::new(0);
    pub static ref WHILE_TRACKER: Mutex<Vec<usize>> = Mutex::new(Vec::default());
    pub static ref SCOPE_STACK: Mutex<Vec<HashMap<String, LSymbol>>> = Mutex::new(Vec::default());
    pub static ref FUNCTION_STACK: Mutex<Vec<String>> = Mutex::new(Vec::default());
    pub static ref BASE_POINTER: Mutex<i64> = Mutex::new(0);
    pub static ref REGISTER_STACK: Mutex<Vec<Vec<(bool, i64)>>> = Mutex::new(Vec::default());
}
/*
 * Function to assign a register which has the lowest index
 */
pub fn get_reg() -> usize {
    let mut register = REGISTERS.lock().unwrap();
    for i in 10..MAX_REGISTERS {
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
//function to backup live registers
fn __backup_registers(mut file: &File) {
    let mut registers = REGISTERS.lock().unwrap();
    let mut rs = REGISTER_STACK.lock().unwrap();
    rs.push(registers.clone());

    //reset to not used
    for i in 0..MAX_REGISTERS {
        if registers[i].0 == true {
            if let Err(e) = writeln!(file, "PUSH R{}", i) {
                exit_on_err(e.to_string());
            }
        }
        registers[i].0 = false;
    }
}
//function to get a safe register for return_value of a function
fn __get_safe_register() -> usize {
    let rs = REGISTER_STACK.lock().unwrap();
    let registers = rs.last().unwrap();
    for i in 0..MAX_REGISTERS {
        //lowest register number free is returned
        if registers[i].0 == false {
            return i;
        }
    }
    0
}
//function to restore register context
fn __restore_register(mut file: &File, safe_register: usize) {
    let mut registers = REGISTERS.lock().unwrap();
    for i in 0..MAX_REGISTERS {
        registers[i].0 = false;
    }
    let mut rs = REGISTER_STACK.lock().unwrap();
    *registers = rs.last().unwrap().clone();
    for i in MAX_REGISTERS - 1..=0 {
        if registers[i].0 == true && i != safe_register {
            if let Err(e) = writeln!(file, "POP R{}", i) {
                exit_on_err(e.to_string());
            }
        }
    }
    rs.pop();
    //reset to not used
}
fn __load_variable(mut file: &File, vname: &String, refr: bool) -> usize {
    let ss = SCOPE_STACK.lock().unwrap();
    if let Some(lst) = ss.last() {
        if let Some(entry) = lst.get(vname) {
            match entry {
                LSymbol::Var {
                    vartype,
                    varid,
                    varindices: _,
                } => {
                    let vreg = get_reg();
                    if let Err(e) = writeln!(file, "MOV R{}, BP", vreg) {
                        exit_on_err(e.to_string())
                    }
                    if *varid < 0 {
                        if let Err(e) = writeln!(file, "SUB R{}, {}", vreg, -1 * varid) {
                            exit_on_err(e.to_string())
                        }
                    } else {
                        if let Err(e) = writeln!(file, "ADD R{}, {}", vreg, varid) {
                            exit_on_err(e.to_string())
                        }
                    }
                    vreg
                }
                _ => {
                    exit_on_err("Error".to_string());
                    0
                }
            }
        } else {
            let gst = GLOBALSYMBOLTABLE.lock().unwrap();
            if let Some(entry) = gst.get(vname) {
                match entry {
                    GSymbol::Var {
                        vartype: _,
                        varid,
                        varindices: _,
                    } => {
                        let vreg = get_reg();
                        if let Err(e) = writeln!(
                            file,
                            "MOV R{}, {}",
                            vreg,
                            XSM_STACK_OFFSET + i64::try_from(varid.clone()).unwrap()
                        ) {
                            exit_on_err(e.to_string())
                        }
                        vreg
                    }
                    _ => {
                        exit_on_err("Error".to_string());
                        0
                    }
                }
            } else {
                exit_on_err("Error".to_string());
                0
            }
        }
    } else {
        let gst = GLOBALSYMBOLTABLE.lock().unwrap();
        if let Some(entry) = gst.get(vname) {
            match entry {
                GSymbol::Var {
                    vartype: _,
                    varid,
                    varindices: _,
                } => {
                    let vreg = get_reg();
                    if let Err(e) = writeln!(
                        file,
                        "MOV R{}, {}",
                        vreg,
                        XSM_STACK_OFFSET + i64::try_from(varid.clone()).unwrap()
                    ) {
                        exit_on_err(e.to_string())
                    }
                    vreg
                }
                _ => {
                    exit_on_err("Error".to_string());
                    0
                }
            }
        } else {
            exit_on_err("Error".to_string());
            0
        }
    }
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
            0
        }
        ASTNode::STR(s) => {
            let register = get_reg();
            let mut registers = REGISTERS.lock().unwrap();
            if let Err(e) = writeln!(file, "MOV R{},{}", register, s) {
                log::error!("[code_gen] Write Error to file : {}", e);
            }
            registers[register].1 = 0;
            register
        }
        ASTNode::INT(n) => {
            let register = get_reg();
            let mut registers = REGISTERS.lock().unwrap();
            if let Err(e) = writeln!(file, "MOV R{},{}", register, n) {
                exit_on_err(e.to_string());
            }
            registers[register].1 = *n;
            register
        }
        ASTNode::VAR { name, indices } => {
            if varinscope(name) == Ok(false) {
                exit_on_err("Variable : [".to_owned() + name.as_str() + "] is not declared");
            }
            let varid = getvarid(name).expect("Error in variable tables");
            let varindices = getvarindices(name).expect("Error in variable tables");

            let baseaddrreg = __load_variable(file, name, refr);

            let mut registers = REGISTERS.lock().unwrap();
            registers[baseaddrreg].1 =
                i64::try_from(XSM_STACK_OFFSET).unwrap() + i64::try_from(varid).unwrap();
            std::mem::drop(registers);

            for i in 0..indices.len() {
                match *indices[i] {
                    ASTNode::STR(_) => {
                        exit_on_err(
                            "str Type cannot be used to index variable [".to_owned() + name + "]",
                        );
                    }
                    ASTNode::INT(num) => {
                        let offsetreg = get_reg();

                        if let Err(e) = writeln!(file, "MOV R{},{}", offsetreg, num) {
                            exit_on_err(e.to_string());
                        }

                        if i != indices.len() - 1 {
                            let indexmulreg = get_reg();
                            let mut registers = REGISTERS.lock().unwrap();

                            if let Err(e) =
                                writeln!(file, "MOV R{}, {}", indexmulreg, varindices[i])
                            {
                                exit_on_err(e.to_string());
                            }

                            if let Err(e) = writeln!(file, "MUL R{}, R{}", offsetreg, indexmulreg) {
                                exit_on_err(e.to_string());
                            }

                            registers[offsetreg].1 *= i64::try_from(varindices[i]).unwrap();

                            std::mem::drop(registers);
                            free_reg(indexmulreg);
                        }

                        let mut registers = REGISTERS.lock().unwrap();
                        if let Err(e) = writeln!(file, "ADD R{}, R{}", baseaddrreg, offsetreg) {
                            exit_on_err(e.to_string());
                        }
                        registers[baseaddrreg].1 += registers[offsetreg].1;

                        std::mem::drop(registers);
                        free_reg(offsetreg);
                    }
                    ASTNode::VAR {
                        name: _,
                        indices: _,
                    } => {
                        if getexprtype(&*indices[i]) != Ok(ASTExprType::Int) {
                            exit_on_err(
                                "Variable with invalid type used to index".to_owned()
                                    + name
                                    + ("[]".repeat(i)).as_str()
                                    + "[x]",
                            );
                        }
                        let offsetreg = __code_gen(&*indices[i], file, false);

                        if i != indices.len() - 1 {
                            let indexmulreg = get_reg();
                            let mut registers = REGISTERS.lock().unwrap();

                            if let Err(e) =
                                writeln!(file, "MOV R{}, {}", indexmulreg, varindices[i])
                            {
                                exit_on_err(e.to_string());
                            }

                            if let Err(e) = writeln!(file, "MUL R{}, R{}", offsetreg, indexmulreg) {
                                exit_on_err(e.to_string());
                            }

                            registers[offsetreg].1 *= i64::try_from(varindices[i]).unwrap();

                            std::mem::drop(registers);
                            free_reg(indexmulreg);
                        }
                        let mut registers = REGISTERS.lock().unwrap();

                        if let Err(e) = writeln!(file, "ADD R{}, R{}", baseaddrreg, offsetreg) {
                            exit_on_err(e.to_string());
                        }
                        registers[baseaddrreg].1 += registers[offsetreg].1;

                        std::mem::drop(registers);
                        free_reg(offsetreg);
                    }
                    //Normal case
                    ASTNode::BinaryNode {
                        op: _,
                        exprtype: _,
                        lhs: _,
                        rhs: _,
                    } => {
                        let exprtype = getexprtype(&*indices[i]);
                        if exprtype != Ok(ASTExprType::Int) {
                            exit_on_err(
                                "Invalid expression type used to index".to_owned()
                                    + name
                                    + ("[]".repeat(i)).as_str()
                                    + "[x]",
                            );
                        }
                        let offsetreg = __code_gen(&*indices[i], file, false);

                        if i != indices.len() - 1 {
                            let indexmulreg = get_reg();
                            let mut registers = REGISTERS.lock().unwrap();

                            if let Err(e) =
                                writeln!(file, "MOV R{}, {}", indexmulreg, varindices[i])
                            {
                                exit_on_err(e.to_string());
                            }

                            if let Err(e) = writeln!(file, "MUL R{}, R{}", offsetreg, indexmulreg) {
                                exit_on_err(e.to_string());
                            }

                            registers[offsetreg].1 *= i64::try_from(varindices[i]).unwrap();

                            std::mem::drop(registers);
                            free_reg(indexmulreg);
                        }

                        let mut registers = REGISTERS.lock().unwrap();
                        if let Err(e) = writeln!(file, "ADD R{}, R{}", baseaddrreg, offsetreg) {
                            exit_on_err(e.to_string());
                        }
                        registers[baseaddrreg].1 += registers[offsetreg].1;

                        std::mem::drop(registers);
                        free_reg(offsetreg);
                    }
                    _ => exit_on_err("Invalid token as index".to_string()),
                };
            }
            if refr == false {
                if let Err(e) = writeln!(file, "MOV R{}, [R{}]", baseaddrreg, baseaddrreg) {
                    exit_on_err(e.to_string());
                }
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
                    let left_operand: String = "R".to_owned() + left_register.to_string().as_str();
                    let right_operand: String =
                        "R".to_owned() + right_register.to_string().as_str();
                    let mut registers = REGISTERS.lock().unwrap();
                    if let Err(e) = writeln!(file, "GT {}, {}", left_operand, right_operand) {
                        exit_on_err(e.to_string());
                    }
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
                    let left_operand: String = "R".to_owned() + left_register.to_string().as_str();
                    let right_operand: String =
                        "R".to_owned() + right_register.to_string().as_str();
                    let mut registers = REGISTERS.lock().unwrap();
                    if let Err(e) = writeln!(file, "LT {}, {}", left_operand, right_operand) {
                        exit_on_err(e.to_string());
                    }
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
                    let left_operand: String = "R".to_owned() + left_register.to_string().as_str();
                    let right_operand: String =
                        "R".to_owned() + right_register.to_string().as_str();
                    let mut registers = REGISTERS.lock().unwrap();
                    if let Err(e) = writeln!(file, "GE {}, {}", left_operand, right_operand) {
                        exit_on_err(e.to_string());
                    }
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
                    let left_operand: String = "R".to_owned() + left_register.to_string().as_str();
                    let right_operand: String =
                        "R".to_owned() + right_register.to_string().as_str();
                    let mut registers = REGISTERS.lock().unwrap();
                    if let Err(e) = writeln!(file, "LE {}, {}", left_operand, right_operand) {
                        exit_on_err(e.to_string());
                    }
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
                    let left_operand: String = "R".to_owned() + left_register.to_string().as_str();
                    let right_operand: String =
                        "R".to_owned() + right_register.to_string().as_str();
                    let mut registers = REGISTERS.lock().unwrap();
                    if let Err(e) = writeln!(file, "EQ {}, {}", left_operand, right_operand) {
                        exit_on_err(e.to_string());
                    }
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
                    let left_operand: String = "R".to_owned() + left_register.to_string().as_str();
                    let right_operand: String =
                        "R".to_owned() + right_register.to_string().as_str();
                    let mut registers = REGISTERS.lock().unwrap();
                    if let Err(e) = writeln!(file, "NE {}, {}", left_operand, right_operand) {
                        exit_on_err(e.to_string());
                    }
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
                    let left_operand: String = "R".to_owned() + left_register.to_string().as_str();
                    let right_operand: String =
                        "R".to_owned() + right_register.to_string().as_str();
                    let mut registers = REGISTERS.lock().unwrap();
                    if let Err(e) = writeln!(file, "ADD {}, {}", left_operand, right_operand) {
                        exit_on_err(e.to_string());
                    }
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
                    let left_operand: String = "R".to_owned() + left_register.to_string().as_str();
                    let right_operand: String =
                        "R".to_owned() + right_register.to_string().as_str();
                    let mut registers = REGISTERS.lock().unwrap();
                    if let Err(e) = writeln!(file, "SUB {}, {}", left_operand, right_operand) {
                        exit_on_err(e.to_string());
                    }
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
                    let left_operand: String = "R".to_owned() + left_register.to_string().as_str();
                    let right_operand: String =
                        "R".to_owned() + right_register.to_string().as_str();
                    let mut registers = REGISTERS.lock().unwrap();
                    if let Err(e) = writeln!(file, "MUL {}, {}", left_operand, right_operand) {
                        exit_on_err(e.to_string());
                    }
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
                    let left_operand: String = "R".to_owned() + left_register.to_string().as_str();
                    let right_operand: String =
                        "R".to_owned() + right_register.to_string().as_str();
                    let mut registers = REGISTERS.lock().unwrap();
                    if let Err(e) = writeln!(file, "DIV {}, {}", left_operand, right_operand) {
                        exit_on_err(e.to_string());
                    }
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
                    if let Err(e) = writeln!(file, "MOD R{}, R{}", left_register, right_register) {
                        exit_on_err(e.to_string());
                    }
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
                    if let Err(e) = writeln!(file, "MOV [R{}],R{}", left_register, right_register) {
                        exit_on_err(e.to_string());
                    }
                    free_reg(left_register);
                    free_reg(right_register);
                    0
                }
                ASTNodeType::Connector => {
                    __code_gen(lhs, file, false);
                    __code_gen(rhs, file, false);
                    0
                }
                _ => 0,
            };
            result
        }
        ASTNode::UnaryNode { op, ptr } => match op {
            ASTNodeType::Read => {
                let register: usize = __code_gen(ptr, file, true).try_into().unwrap();
                __xsm_read_syscall(file, usize::try_from(register).unwrap());
                free_reg(register);
                0
            }
            ASTNodeType::Write => {
                let variable: usize = __code_gen(ptr, file, false).try_into().unwrap();
                __xsm_write_syscall(file, variable);
                free_reg(variable);
                0
            }
            ASTNodeType::Ref => {
                match &**ptr {
                    ASTNode::VAR {
                        name: _,
                        indices: _,
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
                    indices: _,
                } => {
                    let regaddr: usize = __code_gen(ptr, file, refr).try_into().unwrap();
                    if let Err(e) = writeln!(file, "MOV R{},[R{}]", regaddr, regaddr) {
                        exit_on_err(e.to_string());
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
            let mut ptr = &**arglist;
            //Save Live registers
            __backup_registers(file);
            //Push Arguments
            loop {
                match ptr {
                    ArgList::Node { expr, next } => {
                        let retvaladdr = __code_gen(expr, file, refr);
                        if let Err(e) = writeln!(file, "PUSH R{}", retvaladdr) {
                            exit_on_err(e.to_string());
                        }
                        free_reg(retvaladdr);
                        ptr = &**next;
                    }
                    ArgList::Null => break,
                }
            }
            //Push return value
            let pushaddr = get_reg();
            if let Err(e) = writeln!(file, "PUSH R{}", pushaddr) {
                exit_on_err(e.to_string());
            }
            free_reg(pushaddr);
            let gst = GLOBALSYMBOLTABLE.lock().unwrap();
            if let Some(entry) = gst.get(fname) {
                match entry {
                    GSymbol::Func {
                        ret_type,
                        paramlist,
                        flabel,
                    } => {
                        if let Err(e) = writeln!(file, "CALL L{}", flabel) {
                            exit_on_err(e.to_string())
                        }
                    }
                    _ => exit_on_err("Function not declared".to_string()),
                }
            }
            //get safe register
            let safe_reg = __get_safe_register();
            if let Err(e) = writeln!(file, "POP R{}", safe_reg) {
                exit_on_err(e.to_string());
            }
            //Restore live registers
            __restore_register(file, safe_reg);
            0
        }
        ASTNode::ReturnNode { expr } => {
            let reg = __code_gen(expr, file, refr);
            if let Err(e) = writeln!(file, "RET R{}", reg) {
                exit_on_err(e.to_string());
            }
            0
        }
        ASTNode::MainNode { decl, body } => {
            //this node is traverse after all function def nodes,
            let mut label_count = LABEL_COUNT.lock().unwrap();
            let mut gst = GLOBALSYMBOLTABLE.lock().unwrap();
            let l = label_count.clone();
            *label_count += 1;
            //in case main() is recursively called, we need the label of main
            gst.insert(
                "main".to_string(),
                GSymbol::Func {
                    ret_type: (ASTExprType::Int),
                    paramlist: Box::new(ParamList::Null),
                    flabel: (l),
                },
            );

            std::mem::drop(label_count);
            //
            //get the initial stack pointer location
            let mut baseaddr = 4095;
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
                };
            }
            std::mem::drop(gst);
            let ft = FUNCTION_TABLE.lock().unwrap();
            if let Some(local_table) = ft.get("main") {
                let mut ss = SCOPE_STACK.lock().unwrap();
                ss.push(local_table.clone());
                std::mem::drop(ss);
                std::mem::drop(ft);

                if let Err(e) = writeln!(file, "SUB SP, {}", get_ldecl_storage(decl)) {
                    exit_on_err(e.to_string());
                }
                if let Err(e) = writeln!(file, "MOV SP, {}\nMOV BP,SP", baseaddr) {
                    exit_on_err(e.to_string());
                }
                __code_gen(body, file, false);

                let mut ss = SCOPE_STACK.lock().unwrap();
                ss.pop();
            } else {
                exit_on_err("main not defined".to_string())
            }
            0
        }
        ASTNode::FuncDeclNode {
            fname,
            ret_type,
            paramlist,
        } => {
            let mut label_count = LABEL_COUNT.lock().unwrap();
            let l = label_count.clone();
            *label_count += 1;
            let mut gst = GLOBALSYMBOLTABLE.lock().unwrap();

            gst.insert(
                fname.clone(),
                GSymbol::Func {
                    ret_type: (ret_type.clone()),
                    paramlist: (paramlist.clone()),
                    flabel: (l),
                },
            );

            0
        }
        /*
         * L{funclabel}:
         *    Subtract SP by declvars.size()
         *    <body>
         *    ret
         */
        ASTNode::FuncDefNode {
            fname,
            ret_type,
            paramlist,
            decl,
            body,
        } => {
            if let Err(e) = writeln!(file, "L{}:\nPUSH BP\nMOV BP,SP", get_function_label(fname)) {
                exit_on_err(e.to_string());
            }
            if let Err(e) = writeln!(file, "SUB SP, {}", get_ldecl_storage(decl)) {
                exit_on_err(e.to_string());
            }
            let ft = FUNCTION_TABLE.lock().unwrap();
            if let Some(local_table) = ft.get(fname) {
                let mut ss = SCOPE_STACK.lock().unwrap();
                ss.push(local_table.clone());
                std::mem::drop(ss);
                std::mem::drop(ft);

                __code_gen(&**body, file, false);

                let mut ss = SCOPE_STACK.lock().unwrap();
                ss.pop();
            }
            0
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
            if let Err(e) = writeln!(file, "JZ R{}, L{}", result, l1) {
                exit_on_err(e.to_string());
            }
            //Free the register
            free_reg(result);
            //Drop label_count so that nested cases can be handled
            //generate if case flow
            __code_gen(xif, file, false);
            //result is 0 as xif is a stmtlist
            //Jmp to L2 if its else case
            if let Err(e) = writeln!(file, "JMP L{}", l2) {
                exit_on_err(e.to_string());
            }
            //add label count for exit case
            if let Err(e) = writeln!(file, "L{}:", l1) {
                exit_on_err(e.to_string());
            }
            __code_gen(xelse, file, false);
            if let Err(e) = writeln!(file, "L{}:", l2) {
                exit_on_err(e.to_string());
            }
            0
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
            if let Err(e) = writeln!(file, "L{}:", l1) {
                exit_on_err(e.to_string());
            }
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
            if let Err(e) = writeln!(file, "JZ R{}, L{}", result, l2) {
                exit_on_err(e.to_string());
            }
            //Free the register
            free_reg(result);
            //generate if case flow
            //result is 0 as xif is a stmtlist
            __code_gen(xdo, file, false);
            //while loop it back to top condition

            let mut while_tracker = WHILE_TRACKER.lock().unwrap();
            while_tracker.pop();
            while_tracker.pop();
            if let Err(e) = writeln!(file, "JMP L{}", l1) {
                exit_on_err(e.to_string());
            }
            //add label count for exit case
            if let Err(e) = writeln!(file, "L{}:", l2) {
                exit_on_err(e.to_string());
            }
            //increment label_count
            0
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
            if let Err(e) = writeln!(file, "JZ R{}, L{}", result, l1) {
                exit_on_err(e.to_string());
            }
            //Free the register
            free_reg(result);
            //generate if case flow
            __code_gen(xif, file, false);
            //result is 0 as xif is a stmtlist
            if let Err(e) = writeln!(file, "L{}:", l1) {
                exit_on_err(e.to_string());
            }
            //increment label_count
            0
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
            0
        }
        ASTNode::ContinueNode => {
            let while_tracker = WHILE_TRACKER.lock().unwrap();
            writeln!(file, "JMP L{}", while_tracker[while_tracker.len() - 2])
                .expect("[code_gen] Write error");
            0
        }
        ASTNode::DeclNode {
            var_type: _,
            list: _,
        } => 0,
        ASTNode::Null => 0,
        _ => 0,
    }
}

/*
 * Meta function to generate header compatible to XSM ABI Standard
 */
fn __header_gen(mut file: &File) {
    let gst = GLOBALSYMBOLTABLE.lock().unwrap();
    log::info!("Global Symbol Table Size : {}", gst.len());
    if let Err(e) = writeln!(file, "0\n2056\n0\n0\n0\n0\n0\n0\nBRKP",) {
        exit_on_err(e.to_string());
    }
}

/*
 * Meta function to generate xsm code for Write Syscall of expos
 */
fn __xsm_write_syscall(mut file: &File, var: usize) {
    let register = get_reg();
    if let Err(e) = writeln!(file, "MOV R{},\"Write\"\nPUSH R{}\nMOV R{},-2\nPUSH R{}\nMOV R{},R{}\nPUSH R{}\nPUSH R0\nPUSH R0\nCALL 0\nPOP R0\nPOP R{}\nPOP R{}\nPOP R{}\nPOP R{}",register,register,register,register,register,var,register,register,register,register,register) {
        exit_on_err(e.to_string());
    }
    free_reg(register);
}

/*
 * Meta function to generate xsm code for Read Syscall of expos
 */
fn __xsm_read_syscall(mut file: &File, var: usize) {
    let register = get_reg();
    if let Err(e) = writeln!(file, "MOV R{},\"Read\"\nPUSH R{}\nMOV R{},-1\nPUSH R{}\nMOV R{},R{}\nPUSH R{}\nPUSH R0\nPUSH R0\nCALL 0\nPOP R0\nPOP R{}\nPOP R{}\nPOP R{}\nPOP R{}",register,register,register,register,register,var,register,register,register,register,register) {
        exit_on_err(e.to_string());
    }
    free_reg(register);
}

/*
 * Meta function to generate xsm code for Exit Syscall of expos
 */
fn __xsm_exit_syscall(mut file: &File) {
    let register = get_reg();
    if let Err(e) = writeln!(file, "INT 10") {
        exit_on_err(e.to_string());
    }
    free_reg(register);
}
fn __print_gst() {
    let gst = GLOBALSYMBOLTABLE.lock().unwrap();

    log::info!("Global symbol table has {} symbols", gst.len());
    for (k, v) in gst.iter() {}
}
pub fn code_gen(root: &ASTNode, filename: String) -> usize {
    let f = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(filename.as_str());

    match f {
        Ok(mut file) => {
            file.set_len(0)
                .expect("[code_gen] Error truncating existing file");
            __header_gen(&file);
            let result: usize = __code_gen(root, &file, false);
            if let Err(e) = writeln!(file, "PUSH R{}", result) {
                exit_on_err(e.to_string());
            }
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
