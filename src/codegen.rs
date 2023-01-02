//register use table
use crate::parser_y::{ASTNode, ASTNodeType};
use lazy_static::lazy_static; // 1.4.0
use std::cmp::min;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::Mutex;

//global mutable arrays must be guarded with a mutex :(
//TODO maybe initalize this array somewhere and then pass the array as reference to each code_gen
//recursive call
const MAX_REGISTERS: usize = 21;
const MAX_VARIABLES: usize = 26;
const XSM_OFFSET_STACK: usize = 4096;
lazy_static! {
    static ref REGISTERS: Mutex<Vec<(bool, i64)>> = Mutex::new(vec![(false, 0); MAX_REGISTERS]);
    static ref VARIABLES: Mutex<Vec<i64>> = Mutex::new(vec![0; MAX_VARIABLES]);
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
fn __code_gen(root: &ASTNode, mut file: &File) -> usize {
    match root {
        ASTNode::INT(n) => {
            let register = get_reg();
            let mut registers = REGISTERS.lock().unwrap();

            if let Err(e) = writeln!(file, "MOV R{},{}", register, n) {
                eprintln!("[code_gen] Write Error to file : {}", e);
            }
            registers[register].1 = *n;
            register
        }
        ASTNode::VAR(var) => {
            let var_location: u8 = var.chars().nth(0).unwrap() as u8;
            let var_location: usize = var_location as usize;
            (var_location - 97 + 4096 as usize).into()
        }
        ASTNode::BinaryNode { op, lhs, rhs } => {
            let left_register: usize = __code_gen(lhs, file).try_into().unwrap();
            let right_register: usize = __code_gen(rhs, file).try_into().unwrap();

            let mut left_operand: String = "R".to_owned() + left_register.to_string().as_str();
            let mut right_operand: String = "R".to_owned() + right_register.to_string().as_str();

            if left_register >= XSM_OFFSET_STACK {
                left_operand = "[".to_owned() + left_register.to_string().as_str() + "]";
            }

            if right_register >= XSM_OFFSET_STACK {
                right_operand = "[".to_owned() + right_register.to_string().as_str() + "]";
            }

            let mut registers = REGISTERS.lock().unwrap();
            let mut variables = VARIABLES.lock().unwrap();

            let result = match op {
                ASTNodeType::Plus => {
                    if let Err(e) = writeln!(file, "ADD {}, {}", left_operand, right_operand) {
                        eprintln!("[code_gen] Write Error to file : {}", e);
                    }

                    let mut result: i64 = 0;
                    if left_register >= XSM_OFFSET_STACK {
                        result += variables[left_register - XSM_OFFSET_STACK];
                    } else {
                        result += registers[left_register].1;
                    }
                    if right_register >= XSM_OFFSET_STACK {
                        result += variables[right_register - XSM_OFFSET_STACK];
                    } else {
                        result += registers[right_register].1;
                    }

                    let lower_register = min(left_register, right_register);
                    if lower_register < XSM_OFFSET_STACK {
                        registers[lower_register].1 = result;
                    }

                    // release mutex for global array so that register can be freed
                    std::mem::drop(registers);
                    free_reg(left_register + right_register - lower_register);
                    lower_register
                }
                ASTNodeType::Minus => {
                    if let Err(e) = writeln!(file, "SUB {}, {}", left_operand, right_operand) {
                        eprintln!("[code_gen] Write Error to file : {}", e);
                    }
                    let mut result: i64 = 0;
                    if left_register >= XSM_OFFSET_STACK {
                        result += variables[left_register - XSM_OFFSET_STACK];
                    } else {
                        result += registers[left_register].1;
                    }
                    if right_register >= XSM_OFFSET_STACK {
                        result -= variables[right_register - XSM_OFFSET_STACK];
                    } else {
                        result -= registers[right_register].1;
                    }

                    let lower_register = min(left_register, right_register);
                    if lower_register < XSM_OFFSET_STACK {
                        registers[lower_register].1 = result;
                    }

                    // release mutex for global array so that register can be freed
                    std::mem::drop(registers);
                    free_reg(left_register + right_register - lower_register);
                    lower_register
                }
                ASTNodeType::Star => {
                    if let Err(e) = writeln!(file, "MUL {}, {}", left_operand, right_operand) {
                        eprintln!("[code_gen] Write Error to file : {}", e);
                    }

                    let mut result: i64 = 0;
                    if left_register >= XSM_OFFSET_STACK {
                        result += variables[left_register - XSM_OFFSET_STACK];
                    } else {
                        result += registers[left_register].1;
                    }
                    if right_register >= XSM_OFFSET_STACK {
                        result *= variables[right_register - XSM_OFFSET_STACK];
                    } else {
                        result *= registers[right_register].1;
                    }

                    let lower_register = min(left_register, right_register);

                    if lower_register < XSM_OFFSET_STACK {
                        registers[lower_register].1 = result;
                    }
                    // release mutex for global array so that register can be freed
                    std::mem::drop(registers);
                    free_reg(left_register + right_register - lower_register);
                    lower_register
                }
                ASTNodeType::Slash => {
                    if let Err(e) = writeln!(file, "DIV {}, {}", left_operand, right_operand) {
                        eprintln!("[code_gen] Write Error to file : {}", e);
                    }

                    let mut result: i64 = 0;
                    if left_register >= XSM_OFFSET_STACK {
                        result += variables[left_register - XSM_OFFSET_STACK];
                    } else {
                        result += registers[left_register].1;
                    }
                    if right_register >= XSM_OFFSET_STACK {
                        result /= variables[right_register - XSM_OFFSET_STACK];
                    } else {
                        result /= registers[right_register].1;
                    }

                    let lower_register = min(left_register, right_register);

                    if lower_register < XSM_OFFSET_STACK {
                        registers[lower_register].1 = result;
                    }
                    // release mutex for global array so that register can be freed
                    std::mem::drop(registers);
                    free_reg(left_register + right_register - lower_register);
                    lower_register
                }

                ASTNodeType::Equals => {
                    if left_register < XSM_OFFSET_STACK || left_register > XSM_OFFSET_STACK + 25 {
                        eprintln!("[code_gen] Left hand side must be a variable");
                    }

                    if let Err(e) = writeln!(file, "MOV {},{}", left_operand, right_operand) {
                        eprintln!("[code_gen] Write Error to file : {}", e);
                    }
                    0
                }
                ASTNodeType::Read => 0,
                ASTNodeType::Write => 0,
                ASTNodeType::Connector => 0,
            };
            result
        }
        ASTNode::UnaryNode { op, ptr } => match op {
            ASTNodeType::Read => {
                let variable: usize = __code_gen(ptr, file).try_into().unwrap();
                __xsm_read_syscall(file, variable);
                0
            }
            ASTNodeType::Write => {
                let variable: usize = __code_gen(ptr, file).try_into().unwrap();
                __xsm_write_syscall(file, variable);
                0
            }
            ASTNodeType::Plus => 0,
            ASTNodeType::Minus => 0,
            ASTNodeType::Star => 0,
            ASTNodeType::Slash => 0,
            ASTNodeType::Equals => 0,
            ASTNodeType::Connector => 0,
        },
        ASTNode::Null(a) => 0,
    }
}

fn __header_gen(mut file: &File) {
    if let Err(e) = writeln!(file, "0\n2056\n0\n0\n0\n0\n0\n0\nMOV SP,4121") {
        eprintln!("[code_gen] Write error : {}", e);
    }
}

fn __xsm_write_syscall(mut file: &File, var: usize) {
    let register = get_reg();
    if let Err(e) = writeln!(file, "MOV R{},\"Write\"\nPUSH R{}\nMOV R{},-2\nPUSH R{}\nMOV R{},[{}]\nPUSH R{}\nPUSH R0\nPUSH R0\nCALL 0\nPOP R0\nPOP R{}\nPOP R{}\nPOP R{}\nPOP R{}",register,register,register,register,register,var,register,register,register,register,register) {
        eprintln!("[code_gen] Write error : {}", e);
    }
    free_reg(register);
}

fn __xsm_read_syscall(mut file: &File, var: usize) {
    let register = get_reg();
    if let Err(e) = writeln!(file, "MOV R{},\"Read\"\nPUSH R{}\nMOV R{},-1\nPUSH R{}\nMOV R{},{}\nPUSH R{}\nPUSH R0\nPUSH R0\nCALL 0\nPOP R0\nPOP R{}\nPOP R{}\nPOP R{}\nPOP R{}",register,register,register,register,register,var,register,register,register,register,register) {
        eprintln!("[code_gen] Write error : {}", e);
    }
    free_reg(register);
}
fn __xsm_exit_syscall(mut file: &File) {
    let register = get_reg();
    if let Err(e) = writeln!(file, "INT 10") {
        eprintln!("[code_gen] Write error : {}", e);
    }
    free_reg(register);
}

pub fn code_gen(root: &ASTNode) -> usize {
    let mut f = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open("a.xsm");

    match f {
        Ok(mut file) => {
            file.set_len(0)
                .expect("[code_gen] Error truncating existing file");
            __header_gen(&file);
            let result: usize = __code_gen(root, &file);

            if let Err(e) = writeln!(file, "PUSH R{}", result) {
                eprintln!("[code_gen] Write error : {}", e);
            }
            __xsm_exit_syscall(&file);
            result
        }
        Err(e) => {
            eprintln!("[code_gen] Error opening output file : {}", e);
            1
        }
    }
}
