use lrlex::DefaultLexeme;
use lrpar::{LexError, Lexeme, Lexer, Span};
use regex::Regex;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{stderr, Read, Write};

lrlex::lrlex_mod!("linker.l");
const XSM_START: u32 = 2056;
// > get tokens
// > calclulate lineno of each token
// > convert lineno of each token to xsm relative address
// > replace every occurance to xsm relative address

fn read_file(path: &str) -> String {
    let mut f = match File::open(path) {
        Ok(r) => r,
        Err(e) => {
            writeln!(stderr(), "Can't open file {}: {}", path, e).ok();
            std::process::exit(1);
        }
    };
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    s
}
/*
 * Line no to xsm address converter function
 */
#[inline]
fn __get_xsm_address(line: u32) -> u32 {
    XSM_START + (line - (8 - 1)) * 2
}
/*
 * Linker, accepts a file, detects labels, replaces them xsm addresses
 */
pub fn linker(filename: &str) -> Result<bool, ()> {
    let lexerdef = linker_l::lexerdef();

    let mut input = read_file(filename);
    // O(n) first pass
    let lexer = lexerdef.lexer(&input);

    let label_regex = Regex::new(r"L(\d+)").unwrap();
    let mut label_map: HashMap<String, u32>;
    label_map = HashMap::default();

    let mut lexerrev: Vec<Result<DefaultLexeme<u32>, LexError>> = Vec::default();
    for r in lexer.iter() {
        lexerrev.push(r);
    }
    let mut tags: Vec<Vec<String>> = Vec::default();
    let mut locations: Vec<Span> = Vec::default();

    // O(Unique Addresses)
    for r in lexerrev.iter().rev() {
        let tag: Vec<String> = match r {
            //got span of the label here, now find each token inside the label, set it to hashmap
            //replace these tokens with emptyness
            //
            Ok(l) => {
                locations.push(l.span().clone());
                label_regex
                    .find_iter(&input[l.span().start()..l.span().end()].to_owned())
                    .filter_map(|labels| Some(labels.as_str().to_owned()))
                    .collect()
            }
            Err(e) => {
                println!("{:?}", e);
                std::process::exit(1);
            }
        };
        tags.push(tag);
    }
    let mut lineno: u32 = 1;
    // assign address to each label
    for line in input.lines() {
        for tag in tags.iter().rev() {
            // if any label is matched with the tag
            let mut flag = false;
            for label in tag {
                if label_map.contains_key(label) == true {
                    // this tag is already calculate, ignore
                    break;
                }
                // if not matched, check if this is the definition of the label
                let search = label.clone() + ":";
                if line.find(search.as_str()) != None {
                    flag = true;
                    break;
                }
            }
            if flag == true {
                for label in tag {
                    label_map.insert(label.clone(), __get_xsm_address(lineno));
                }
            }
            // all labels in the tag get the same address
        }
        lineno += 1;
    }

    //remove the labels
    // O(Unique Addresses)
    for location in locations {
        input = input
            .replace(&input[location.start()..location.end()], "")
            .to_owned();
    }

    let mut f = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open("a.xsm")
        .expect("[linker] xsm file write error");

    // Pass 3, replace label with address
    for line in input.lines() {
        let mut flag: bool = false;
        for (k, v) in label_map.iter() {
            //find if there is a label
            if line.find(k.as_str()) != None {
                //replace with address in hashmap
                writeln!(f, "{}", line.replace(k.as_str(), v.to_string().as_str()))
                    .expect("[linker] xsm file write error");
                flag = true;
                break;
            }
        }
        if flag == false {
            writeln!(f, "{}", line).expect("[linker] xsm file write error");
        }
    }

    println!("{}", input);
    Ok(false)
}
