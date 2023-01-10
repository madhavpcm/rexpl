use lrlex::DefaultLexeme;
use lrpar::{LexError, Lexeme, Lexer, Span};
use regex::Regex;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{stderr, Read, Write};

lrlex::lrlex_mod!("linker.l");
const XSM_START: usize = 2056;

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
fn __get_xsm_address(line: usize) -> usize {
    if line < 9 {
        return 0;
    }
    XSM_START + (line - 9) * 2
}
/*
 * Linker, accepts a file, detects labels, replaces them xsm addresses
 * ALGORITHM
* > get tokens
* > calclulate lineno of each token
* > convert lineno of each token to xsm relative address
* > replace every occurance to xsm relative address
 */
pub fn linker(filename: &str) -> Result<bool, ()> {
    let lexerdef = linker_l::lexerdef();
    let mut input = read_file((filename.to_owned() + ".o").as_str());
    // O(n) first pass
    let lexer = lexerdef.lexer(&input);
    //to match labels which may consecutively occur together
    let label_regex = Regex::new(r"L(\d+)").unwrap();

    //Symbol table for <label> <address> pair
    let mut label_map: HashMap<String, usize>;
    label_map = HashMap::default();

    //we need the lexemes to be in reverse order to avoid index out of bounds when removing
    let mut lexerrev: Vec<Result<DefaultLexeme<u32>, LexError>> = Vec::default();
    for r in lexer.iter() {
        lexerrev.push(r);
    }
    //Collection of tags, where each tag may consist of multiple label but have same assembly address
    let mut tags: Vec<Vec<String>> = Vec::default();
    //Spans of the locations of each tag
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
    let mut lineno: usize = 1;
    // Pass 2 to determine the address of each tag
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
                lineno -= tag.len();
                break;
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
        .open(filename.to_owned() + ".xsm")
        .expect("[linker] xsm file write error");

    f.set_len(0)
        .expect("[linker] Error truncating existing file");
    // Pass 3, replace label with address and write to new file
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

    log::trace!("Generated XSM Assembly: {}.xsm", filename);

    Ok(false)
}
