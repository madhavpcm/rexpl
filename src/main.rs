// Libraries
use lrlex::lrlex_mod;
use lrpar::lrpar_mod;
use std::{
    env,
    fs::File,
    io::{stderr, Read, Write},
    process,
};

//Modules

lrlex_mod!("lexer.l");
lrpar_mod!("parser.y");

mod codegen;
mod exprtree;
mod parserlib;

fn read_file(path: &str) -> String {
    let mut f = match File::open(path) {
        Ok(r) => r,
        Err(e) => {
            writeln!(stderr(), "Can't open file {}: {}", path, e).ok();
            process::exit(1);
        }
    };
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    s
}

fn main() {
    let lexerdef = lexer_l::lexerdef();
    let args: Vec<String> = env::args().collect();

    let input = &read_file(&args[1]);
    println!("Input is {}", input);
    let lexer_ = lexerdef.lexer(input);
    let (expr_res, errs) = parser_y::parse(&lexer_);
    for e in errs {
        println!("{}", e.pp(&lexer_, &parser_y::token_epp));
    }
    if let Some(Ok(r)) = expr_res {
        codegen::code_gen(&r);
    }

    return;
}
