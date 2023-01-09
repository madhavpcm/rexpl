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
/*
 * Error handler
 */
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
fn __code_gen(root: &ASTNode, mut file: &File) -> usize {
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
                eprintln!("[code_gen] Write Error to file : {}", e);
            }
            registers[register].1 = *n;
            register
        }
        ASTNode::VAR(var) => {
            let gst = GLOBALSYMBOLTABLE.lock().unwrap();
            if gst.contains_key(var) == false {
                __exit_on_err("Variable : [".to_owned() + var.as_str() + "] is not declared");
            }
            let vardetails = gst.get(var).expect("[rexplc] : Global symbol table error");

            let temp_register = get_reg();
            let result = temp_register.clone();

            let mut hashmap = VARIABLE_REGISTER_MAP.lock().unwrap();
            let temp_operand = "R".to_owned() + temp_register.to_string().as_str();
            if let Err(e) = writeln!(
                file,
                "MOV {}, [{}]",
                temp_operand,
                XSM_OFFSET_STACK + vardetails.varid
            ) {
                eprintln!("[code_gen] Write Error to file : {}", e);
            }
            hashmap.insert(temp_register, XSM_OFFSET_STACK + vardetails.varid);
            result
        }
        ASTNode::BinaryNode {
            op,
            exprtype: _,
            lhs,
            rhs,
        } => {
            let left_register: usize = __code_gen(lhs, file).try_into().unwrap();
            let right_register: usize = __code_gen(rhs, file).try_into().unwrap();
            let left_operand: String = "R".to_owned() + left_register.to_string().as_str();
            let right_operand: String = "R".to_owned() + right_register.to_string().as_str();
            let mut registers = REGISTERS.lock().unwrap();
            let result = match op {
                ASTNodeType::Gt => {
                    if let Err(e) = writeln!(file, "GT {}, {}", left_operand, right_operand) {
                        eprintln!("[code_gen] Write Error to file : {}", e);
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
                    if let Err(e) = writeln!(file, "LT {}, {}", left_operand, right_operand) {
                        eprintln!("[code_gen] Write Error to file : {}", e);
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
                    if let Err(e) = writeln!(file, "GTE {}, {}", left_operand, right_operand) {
                        eprintln!("[code_gen] Write Error to file : {}", e);
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
                    if let Err(e) = writeln!(file, "LTE {}, {}", left_operand, right_operand) {
                        eprintln!("[code_gen] Write Error to file : {}", e);
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
                    if let Err(e) = writeln!(file, "EQ {}, {}", left_operand, right_operand) {
                        eprintln!("[code_gen] Write Error to file : {}", e);
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
                    if let Err(e) = writeln!(file, "NE {}, {}", left_operand, right_operand) {
                        eprintln!("[code_gen] Write Error to file : {}", e);
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
                    if let Err(e) = writeln!(file, "ADD {}, {}", left_operand, right_operand) {
                        eprintln!("[code_gen] Write Error to file : {}", e);
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
                    if let Err(e) = writeln!(file, "SUB {}, {}", left_operand, right_operand) {
                        eprintln!("[code_gen] Write Error to file : {}", e);
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
                    if let Err(e) = writeln!(file, "MUL {}, {}", left_operand, right_operand) {
                        eprintln!("[code_gen] Write Error to file : {}", e);
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
                    if let Err(e) = writeln!(file, "DIV {}, {}", left_operand, right_operand) {
                        eprintln!("[code_gen] Write Error to file : {}", e);
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
                    let mut hashmap = VARIABLE_REGISTER_MAP.lock().unwrap();
                    if hashmap.contains_key(&left_register) == false {
                        eprintln!("[code_gen] Too many variables to handle");
                        return 1;
                    }
                    registers[left_register] = registers[right_register];
                    if let Err(e) = writeln!(file, "MOV R{},R{}", left_register, right_register) {
                        eprintln!("[code_gen] Write Error to file : {}", e);
                    }
                    if let Err(e) = writeln!(
                        file,
                        "MOV [{}],R{}",
                        hashmap.get(&left_register).unwrap(),
                        left_register
                    ) {
                        eprintln!("[code_gen] Write Error to file : {}", e);
                    }
                    std::mem::drop(registers);
                    for k in hashmap.keys() {
                        free_reg(*k);
                    }
                    free_reg(left_register);
                    free_reg(right_register);
                    hashmap.clear();
                    0
                }
                ASTNodeType::Connector => 0,
                _ => 0,
            };
            result
        }
        ASTNode::UnaryNode { op, ptr } => match op {
            ASTNodeType::Read => {
                let register: usize = __code_gen(ptr, file).try_into().unwrap();
                let mut hashmap = VARIABLE_REGISTER_MAP.lock().unwrap();
                __xsm_read_syscall(file, *hashmap.get(&register).unwrap());
                for k in hashmap.keys() {
                    free_reg(*k);
                }
                hashmap.clear();
                0
            }
            ASTNodeType::Write => {
                let variable: usize = __code_gen(ptr, file).try_into().unwrap();
                let mut hashmap = VARIABLE_REGISTER_MAP.lock().unwrap();
                __xsm_write_syscall(file, variable);
                for k in hashmap.keys() {
                    free_reg(*k);
                }
                hashmap.clear();
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
            let result: usize = __code_gen(expr, file).try_into().unwrap();
            //Generate code for the expression
            if let Err(e) = writeln!(file, "JZ R{}, L{}", result, l1) {
                eprintln!("[code_gen] wrerror : {}", e);
            }
            //Free the register
            free_reg(result);
            //Drop label_count so that nested cases can be handled
            //generate if case flow
            __code_gen(xif, file);
            //result is 0 as xif is a stmtlist
            //Jmp to L2 if its else case
            if let Err(e) = writeln!(file, "JMP L{}", l2) {
                eprintln!("[code_gen] wrerror : {}", e);
            }
            //add label count for exit case
            if let Err(e) = writeln!(file, "L{}:", l1) {
                eprintln!("[code_gen] wrerror : {}", e);
            }
            __code_gen(xelse, file);
            if let Err(e) = writeln!(file, "L{}:", l2) {
                eprintln!("[code_gen] wrerror : {}", e);
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
                eprintln!("[code_gen] wrerror : {}", e);
            }
            (*label_count) += 1;

            let l2 = label_count.clone();
            (*label_count) += 1;

            while_tracker.push(l1);
            while_tracker.push(l2);
            //Drop label_count so that nested cases can be handled
            std::mem::drop(label_count);
            std::mem::drop(while_tracker);
            let result: usize = __code_gen(expr, file).try_into().unwrap();
            //Generate code for the expression
            if let Err(e) = writeln!(file, "JZ R{}, L{}", result, l2) {
                eprintln!("[code_gen] wrerror : {}", e);
            }
            //Free the register
            free_reg(result);
            //generate if case flow
            //result is 0 as xif is a stmtlist
            __code_gen(xdo, file);
            //while loop it back to top condition

            let mut while_tracker = WHILE_TRACKER.lock().unwrap();
            while_tracker.pop();
            while_tracker.pop();
            if let Err(e) = writeln!(file, "JMP L{}", l1) {
                eprintln!("[code_gen] wrerror : {}", e);
            }
            //add label count for exit case
            if let Err(e) = writeln!(file, "L{}:", l2) {
                eprintln!("[code_gen] wrerror : {}", e);
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
            let result: usize = __code_gen(expr, file).try_into().unwrap();
            //Generate code for the expression
            if let Err(e) = writeln!(file, "JZ R{}, L{}", result, l1) {
                eprintln!("[code_gen] wrerror : {}", e);
            }
            //Free the register
            free_reg(result);
            //generate if case flow
            __code_gen(xif, file);
            //result is 0 as xif is a stmtlist
            if let Err(e) = writeln!(file, "L{}:", l1) {
                eprintln!("[code_gen] wrerror : {}", e);
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
        eprintln!("[code_gen] Write error : {}", e);
    }
}

/*
 * Meta function to generate xsm code for Write Syscall of expos
 */
fn __xsm_write_syscall(mut file: &File, var: usize) {
    let register = get_reg();
    if let Err(e) = writeln!(file, "MOV R{},\"Write\"\nPUSH R{}\nMOV R{},-2\nPUSH R{}\nMOV R{},R{}\nPUSH R{}\nPUSH R0\nPUSH R0\nCALL 0\nPOP R0\nPOP R{}\nPOP R{}\nPOP R{}\nPOP R{}",register,register,register,register,register,var,register,register,register,register,register) {
        eprintln!("[code_gen] Write error : {}", e);
    }
    free_reg(register);
}

/*
 * Meta function to generate xsm code for Read Syscall of expos
 */
fn __xsm_read_syscall(mut file: &File, var: usize) {
    let register = get_reg();
    if let Err(e) = writeln!(file, "MOV R{},\"Read\"\nPUSH R{}\nMOV R{},-1\nPUSH R{}\nMOV R{},{}\nPUSH R{}\nPUSH R0\nPUSH R0\nCALL 0\nPOP R0\nPOP R{}\nPOP R{}\nPOP R{}\nPOP R{}",register,register,register,register,register,var,register,register,register,register,register) {
        eprintln!("[code_gen] Write error : {}", e);
    }
    free_reg(register);
}

/*
 * Meta function to generate xsm code for Exit Syscall of expos
 */
fn __xsm_exit_syscall(mut file: &File) {
    let register = get_reg();
    if let Err(e) = writeln!(file, "INT 10") {
        eprintln!("[code_gen] Write error : {}", e);
    }
    free_reg(register);
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
            let result: usize = __code_gen(root, &file);
            if let Err(e) = writeln!(file, "PUSH R{}", result) {
                eprintln!("[code_gen] Write error : {}", e);
            }
            __xsm_exit_syscall(&file);
            println!("Generated Object file: {}", filename.as_str());
            result
        }
        Err(e) => {
            eprintln!("[code_gen] Error opening output file : {}", e);
            1
        }
    }
}
