//register use table
use crate::parser_y::{Node, Operator};
use lazy_static::lazy_static; // 1.4.0
use std::cmp::max;
use std::cmp::min;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::Mutex;

//global mutable arrays must be guarded with a mutex :(
//TODO maybe initalize this array somewhere and then pass the array as reference to each code_gen
//recursive call
const MAX_REGISTERS: usize = 21;
lazy_static! {
    static ref REGISTERS: Mutex<Vec<(bool, i64)>> = Mutex::new(vec![(false, 0); MAX_REGISTERS]);
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
    let mut registers = REGISTERS.lock().unwrap();
    registers[register].0 = false;
    return MAX_REGISTERS.try_into().unwrap();
}
/*
 * Meta function which recursively generates assembly lines
 * in xsm for arithmetic operations
 */
fn __code_gen(root: &Node, mut file: &File) -> usize {
    match root {
        Node::INT(n) => {
            let register = get_reg();
            let mut registers = REGISTERS.lock().unwrap();

            if let Err(e) = writeln!(file, "MOV R{},{}", register, n) {
                eprintln!("[code_gen] Write Error to file : {}", e);
            }
            registers[register].1 = *n;
            register
        }
        Node::BinaryExpr { op, lhs, rhs } => {
            let left_register: usize = __code_gen(lhs, file).try_into().unwrap();
            let right_regsiter: usize = __code_gen(rhs, file).try_into().unwrap();

            let mut registers = REGISTERS.lock().unwrap();

            let result = match op {
                Operator::Plus => {
                    if let Err(e) = writeln!(file, "ADD R{}, R{}", left_register, right_regsiter) {
                        eprintln!("[code_gen] Write Error to file : {}", e);
                    }

                    registers[left_register].1 + registers[right_regsiter].1
                }
                Operator::Minus => {
                    if let Err(e) = writeln!(file, "SUB R{}, R{}", left_register, right_regsiter) {
                        eprintln!("[code_gen] Write Error to file : {}", e);
                    }

                    registers[left_register].1 - registers[right_regsiter].1
                }
                Operator::Star => {
                    if let Err(e) = writeln!(file, "MUL R{}, R{}", left_register, right_regsiter) {
                        eprintln!("[code_gen] Write Error to file : {}", e);
                    }

                    registers[left_register].1 * registers[right_regsiter].1
                }
                Operator::Slash => {
                    if let Err(e) = writeln!(file, "DIV R{}, R{}", left_register, right_regsiter) {
                        eprintln!("[code_gen] Write Error to file : {}", e);
                    }

                    registers[left_register].1 / registers[right_regsiter].1
                }
            };
            let lower_register = min(left_register, right_regsiter);

            registers[lower_register].1 = result;
            // release mutex for global array so that register can be freed
            std::mem::drop(registers);
            free_reg(left_register + right_regsiter - lower_register);
            lower_register
        }
    }
}

fn __header_gen(mut file: &File) {
    if let Err(e) = writeln!(file, "0\n2056\n0\n0\n0\n0\n0\n0\nMOV SP,4095") {
        eprintln!("[code_gen] Write error : {}", e);
    }
}

fn __xsm_write_syscall(mut file: &File) {
    let register = get_reg();
    if let Err(e) = writeln!(file, "MOV R{},\"Write\"\nPUSH R{}\nMOV R{},-2\nPUSH R{}\nMOV R{},[4096]\nPUSH R{}\nPUSH R0\nPUSH R0\nCALL 0\nPOP R0\nPOP R{}\nPOP R{}\nPOP R{}\nPOP R{}",register,register,register,register,register,register,register,register,register,register) {
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

pub fn code_gen(root: &Node) -> usize {
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
            __xsm_write_syscall(&file);
            __xsm_exit_syscall(&file);
            result
        }
        Err(e) => {
            eprintln!("[code_gen] Error opening output file : {}", e);
            1
        }
    }
}
