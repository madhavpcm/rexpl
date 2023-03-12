// TODO call to undefined functions
// TODO Strict Ownership things for type system
// TODO user types and function name check
use env_logger::{Builder, Env};
use lrlex::lrlex_mod;
use lrpar::lrpar_mod;
use std::{
    env,
    fs::File,
    io::{Read, Write},
    process,
};

//Modules

lrlex_mod!("lexer.l");
lrpar_mod!("parser.y");

mod codegen;
mod exprtree;
mod linker;
mod parserlib;
mod validation;

fn read_file(path: &str) -> String {
    let mut f = match File::open(path) {
        Ok(r) => r,
        Err(e) => {
            log::error!("Can't open file {}: {}", path, e);
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

    // Log formatting
    Builder::from_env(Env::default().default_filter_or("trace"))
        .format(|buf, record| {
            let mut style = buf.style();
            style.set_color(match record.level() {
                log::Level::Error => env_logger::fmt::Color::Red,
                log::Level::Warn => env_logger::fmt::Color::Yellow,
                log::Level::Info => env_logger::fmt::Color::Blue,
                log::Level::Trace => env_logger::fmt::Color::Green,
                log::Level::Debug => env_logger::fmt::Color::Magenta,
            });
            writeln!(buf, "[{}] {}", style.value(record.level()), record.args())
        })
        .init();

    let input = &read_file(&args[1]);
    let lexer_ = lexerdef.lexer(input);

    let filename = *args[1]
        .split('.')
        .collect::<Vec<&str>>()
        .first()
        .expect("Extension error");

    let (expr_res, errs) = parser_y::parse(&lexer_);
    for e in errs {
        log::error!("{}", e.pp(&lexer_, &parser_y::token_epp));
        process::exit(1);
    }
    if let Some(Ok(r)) = expr_res {
        codegen::code_gen(&r, filename.to_owned() + ".o");
    } else {
        if let Some(Err(e)) = expr_res {
            log::error!("{}", e);
            process::exit(1);
        }
    }
    linker::linker(filename).expect("Linking failed");

    return;
}
