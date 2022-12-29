use calc_y::Node;
use calc_y::Operator;
use lrlex::{lrlex_mod, LexerDef};
use lrpar::lrpar_mod;
use lrpar::{Lexeme, Lexer};
use std::{
    env,
    fs::File,
    io::{stderr, Read, Write},
    process,
};

// Using `lrlex_mod!` brings the lexer for `calc.l` into scope. By default the
// module name will be `calc_l` (i.e. the file name, minus any extensions,
// with a suffix of `_l`).
lrlex_mod!("calc.l");
// Using `lrpar_mod!` brings the parser for `calc.y` into scope. By default the
// module name will be `calc_y` (i.e. the file name, minus any extensions,
// with a suffix of `_y`).
lrpar_mod!("calc.y");

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

fn evaluateAST(root: Node) -> i64 {
    match root {
        Node::INT(n) => n,
        Node::BinaryExpr { op, lhs, rhs } => match op {
            Operator::Plus => evaluateAST(*lhs) + evaluateAST(*rhs),
            Operator::Minus => evaluateAST(*lhs) - evaluateAST(*rhs),
            Operator::Star => evaluateAST(*lhs) * evaluateAST(*rhs),
            Operator::Slash => evaluateAST(*lhs) / evaluateAST(*rhs),
        },
        Node::UnaryExpr { op, child } => todo!(),
    }
}

fn main() {
    // Get the `LexerDef` for the `calc` language.
    let lexerdef = calc_l::lexerdef();
    let args: Vec<String> = env::args().collect();
    // Now we create a lexer with the `lexer` method with which we can lex an
    // input.
    //

    let input = &read_file(&args[1]);
    println!("Input is {}", input);
    let strf = calc_y::parse_string("g");
    let lexer_ = lexerdef.lexer(input);
    let (expr_res, errs) = calc_y::parse(&lexer_);
    for e in errs {
        println!("{}", e.pp(&lexer_, &calc_y::token_epp));
    }
    if let Some(Ok(r)) = expr_res {
        println!("Result is {}", evaluateAST(r));
    }

    return;
}
