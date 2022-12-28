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

fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>());
}

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
    // Get the `LexerDef` for the `calc` language.
    let lexerdef = calc_l::lexerdef();
    let args: Vec<String> = env::args().collect();
    // Now we create a lexer with the `lexer` method with which we can lex an
    // input.
    //

    let input = &read_file(&args[1]);

    let lexer_ = lexerdef.lexer(input);
    for lexeme in lexer_.iter() {
        match lexeme {
            Ok(l) => {
                let rule = lexerdef.get_rule_by_id(l.tok_id()).name.as_ref().unwrap();
                if rule == "VAR" {
                    println!("{} {}", rule, &input[l.span().start()..l.span().end()]);
                }
                println!()
            }
            Err(e) => {
                println!("{:?}", e);
                continue;
            }
        }
    }
    let (res, errs) = calc_y::parse(&lexer_);
    for e in errs {
        println!("{}", e.pp(&lexer_, &calc_y::token_epp));
    }
    match res {
        Some(Ok(r)) => println!("Result: {:?}", r),
        _ => eprintln!("Unable to evaluate expression."),
    }
    // Pass the lexer to the parser and lex and parse the input.
    //let (res, errs) = calc_y::parse(&lexer);
}
