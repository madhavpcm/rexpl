//register use table
use crate::parserlib::*;
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
const XSM_OFFSET_STACK: usize = 4096;

lazy_static! {
    static ref REGISTERS: Mutex<Vec<(bool, i64)>> = Mutex::new(vec![(false, 0); MAX_REGISTERS]);
    static ref VARIABLE_REGISTER_MAP: Mutex<HashMap<usize, usize>> = Mutex::new(HashMap::default());
    static ref LABEL_COUNT: Mutex<usize> = Mutex::new(0);
    static ref WHILE_TRACKER: Mutex<Vec<usize>> = Mutex::new(Vec::default());
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
fn __exit_on_err(err: String) {
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
            __exit_on_err(err);
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
                __exit_on_err(e.to_string());
            }
            registers[register].1 = *n;
            register
        }
        ASTNode::VAR { name, index1 } => {
            let gst = GLOBALSYMBOLTABLE.lock().unwrap();
            if gst.contains_key(name) == false {
                __exit_on_err("Variable : [".to_owned() + name.as_str() + "] is not declared");
            }
            let vardetails = gst
                .get(name)
                .expect("[rexplc] : Global symbol table error")
                .clone();

            std::mem::drop(gst);

            match **index1 {
                ASTNode::STR(_) => {
                    __exit_on_err(
                        "str Type cannot be used to index variable [".to_owned() + name + "]",
                    );
                }
                ASTNode::INT(num) => {
                    let register = get_reg();
                    let mut registers = REGISTERS.lock().unwrap();
                    if let Err(e) = writeln!(file, "MOV R{},{}", register, num) {
                        __exit_on_err(e.to_string());
                    }
                    registers[register].1 = num
                        + i64::try_from(XSM_OFFSET_STACK).unwrap()
                        + i64::try_from(vardetails.varid).unwrap();

                    return register;
                }
                ASTNode::VAR { name: _, index1: _ } => {
                    let baseaddrreg = get_reg();

                    if let Err(e) = writeln!(
                        file,
                        "MOV R{}, {}",
                        baseaddrreg,
                        XSM_OFFSET_STACK + vardetails.varid
                    ) {
                        __exit_on_err(e.to_string());
                    }

                    let offsetreg = __code_gen(&index1, file, false);

                    let mut registers = REGISTERS.lock().unwrap();
                    if let Err(e) = writeln!(file, "ADD R{}, R{}", baseaddrreg, offsetreg) {
                        __exit_on_err(e.to_string());
                    }
                    if refr == false {
                        if let Err(e) = writeln!(file, "MOV R{}, [R{}]", baseaddrreg, baseaddrreg) {
                            __exit_on_err(e.to_string());
                        }
                    }
                    registers[baseaddrreg].1 = i64::try_from(XSM_OFFSET_STACK).unwrap()
                        + i64::try_from(vardetails.varid).unwrap()
                        + registers[offsetreg].1;

                    std::mem::drop(registers);
                    free_reg(offsetreg);
                    return baseaddrreg;
                }
                //Normal case
                ASTNode::Null => {
                    let baseaddrreg = get_reg();

                    let mut registers = REGISTERS.lock().unwrap();
                    registers[baseaddrreg].1 = i64::try_from(XSM_OFFSET_STACK).unwrap()
                        + i64::try_from(vardetails.varid).unwrap();
                    if refr == false {
                        if let Err(e) = writeln!(
                            file,
                            "MOV R{}, [{}]",
                            baseaddrreg,
                            XSM_OFFSET_STACK + vardetails.varid
                        ) {
                            __exit_on_err(e.to_string());
                        }
                    } else {
                        if let Err(e) = writeln!(
                            file,
                            "MOV R{}, {}",
                            baseaddrreg,
                            XSM_OFFSET_STACK + vardetails.varid
                        ) {
                            __exit_on_err(e.to_string());
                        }
                    }
                    return baseaddrreg;
                }
                ASTNode::BinaryNode {
                    op: _,
                    exprtype,
                    lhs: _,
                    rhs: _,
                } => {
                    let temp_register = get_reg();
                    if exprtype != ASTExprType::Int {
                        __exit_on_err(
                            "An expression of bool Type cannot be used to index variable ["
                                .to_owned()
                                + name
                                + "]",
                        );
                    }
                    if let Err(e) = writeln!(
                        file,
                        "MOV R{}, {}",
                        temp_register,
                        XSM_OFFSET_STACK + vardetails.varid
                    ) {
                        __exit_on_err(e.to_string());
                    }
                    let evalreg = __code_gen(index1, file, false);
                    if let Err(e) = writeln!(file, "ADD R{}, R{}", temp_register, evalreg) {
                        __exit_on_err(e.to_string());
                    }
                    free_reg(evalreg);
                    return temp_register;
                }
                _ => __exit_on_err("Invalid token as index".to_string()),
            };
            0
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
                        __exit_on_err(e.to_string());
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
                        __exit_on_err(e.to_string());
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
                    if let Err(e) = writeln!(file, "GTE {}, {}", left_operand, right_operand) {
                        __exit_on_err(e.to_string());
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
                    if let Err(e) = writeln!(file, "LTE {}, {}", left_operand, right_operand) {
                        __exit_on_err(e.to_string());
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
                        __exit_on_err(e.to_string());
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
                        __exit_on_err(e.to_string());
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
                        __exit_on_err(e.to_string());
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
                        __exit_on_err(e.to_string());
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
                        __exit_on_err(e.to_string());
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
                        __exit_on_err(e.to_string());
                    }
                    let result: i64 = registers[left_register].1 / registers[right_register].1;
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
                        __exit_on_err(e.to_string());
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
            _ => 0,
        },
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
                __exit_on_err(e.to_string());
            }
            //Free the register
            free_reg(result);
            //Drop label_count so that nested cases can be handled
            //generate if case flow
            __code_gen(xif, file, false);
            //result is 0 as xif is a stmtlist
            //Jmp to L2 if its else case
            if let Err(e) = writeln!(file, "JMP L{}", l2) {
                __exit_on_err(e.to_string());
            }
            //add label count for exit case
            if let Err(e) = writeln!(file, "L{}:", l1) {
                __exit_on_err(e.to_string());
            }
            __code_gen(xelse, file, false);
            if let Err(e) = writeln!(file, "L{}:", l2) {
                __exit_on_err(e.to_string());
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
                __exit_on_err(e.to_string());
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
                __exit_on_err(e.to_string());
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
                __exit_on_err(e.to_string());
            }
            //add label count for exit case
            if let Err(e) = writeln!(file, "L{}:", l2) {
                __exit_on_err(e.to_string());
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
                __exit_on_err(e.to_string());
            }
            //Free the register
            free_reg(result);
            //generate if case flow
            __code_gen(xif, file, false);
            //result is 0 as xif is a stmtlist
            if let Err(e) = writeln!(file, "L{}:", l1) {
                __exit_on_err(e.to_string());
            }
            //increment label_count
            0
        }
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
    }
}

/*
 * Meta function to generate header compatible to XSM ABI Standard
 */
fn __header_gen(mut file: &File) {
    let gst = GLOBALSYMBOLTABLE.lock().unwrap();
    log::info!("Global Symbol Table Size : {}", gst.len());
    if let Err(e) = writeln!(
        file,
        "0\n2056\n0\n0\n0\n0\n0\n0\nMOV SP,{}\nMOV BP,SP",
        4095 + gst.len()
    ) {
        __exit_on_err(e.to_string());
    }
}

/*
 * Meta function to generate xsm code for Write Syscall of expos
 */
fn __xsm_write_syscall(mut file: &File, var: usize) {
    let register = get_reg();
    if let Err(e) = writeln!(file, "MOV R{},\"Write\"\nPUSH R{}\nMOV R{},-2\nPUSH R{}\nMOV R{},R{}\nPUSH R{}\nPUSH R0\nPUSH R0\nCALL 0\nPOP R0\nPOP R{}\nPOP R{}\nPOP R{}\nPOP R{}",register,register,register,register,register,var,register,register,register,register,register) {
        __exit_on_err(e.to_string());
    }
    free_reg(register);
}

/*
 * Meta function to generate xsm code for Read Syscall of expos
 */
fn __xsm_read_syscall(mut file: &File, var: usize) {
    let register = get_reg();
    if let Err(e) = writeln!(file, "MOV R{},\"Read\"\nPUSH R{}\nMOV R{},-1\nPUSH R{}\nMOV R{},R{}\nPUSH R{}\nPUSH R0\nPUSH R0\nCALL 0\nPOP R0\nPOP R{}\nPOP R{}\nPOP R{}\nPOP R{}",register,register,register,register,register,var,register,register,register,register,register) {
        __exit_on_err(e.to_string());
    }
    free_reg(register);
}

/*
 * Meta function to generate xsm code for Exit Syscall of expos
 */
fn __xsm_exit_syscall(mut file: &File) {
    let register = get_reg();
    if let Err(e) = writeln!(file, "INT 10") {
        __exit_on_err(e.to_string());
    }
    free_reg(register);
}
fn __print_gst() {
    let gst = GLOBALSYMBOLTABLE.lock().unwrap();

    log::info!("Global symbol table has {} symbols", gst.len());
    for (k, v) in gst.iter() {
        log::info!(
            "GST Entry [{}] : {} | {} | {}",
            k,
            v.varid,
            v.varsize,
            v.vartype
        );
    }
}
pub fn code_gen(root: &ASTNode, filename: String) -> usize {
    let f = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(filename.as_str());

    __print_gst();

    match f {
        Ok(mut file) => {
            file.set_len(0)
                .expect("[code_gen] Error truncating existing file");
            __header_gen(&file);
            let result: usize = __code_gen(root, &file, false);
            if let Err(e) = writeln!(file, "PUSH R{}", result) {
                __exit_on_err(e.to_string());
            }
            __xsm_exit_syscall(&file);
            log::trace!("Generated Object file: {}", filename.as_str());
            result
        }
        Err(e) => {
            __exit_on_err(e.to_string());
            1
        }
    }
}
