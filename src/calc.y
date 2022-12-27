%start Start 
%avoid_insert "INT"
%avoid_insert "FLOAT"
%avoid_insert "WORD"
%avoid_insert "SINGLE_COMMENT"
%avoid_insert "MULTI_COMMENT"
%avoid_insert "SPEC"
%avoid_insert "IF"
%avoid_insert "COND"
%avoid_insert "S"
%avoid_insert "{"
%avoid_insert "}"
%avoid_insert "("
%avoid_insert ")"

%%
Start -> Result<u64, ()>:
	If_Statement { Ok(0) }
	| Cond_Statement { Ok(0) }
	| Term { Ok(0) }
	;

If_Statement -> Result<u64, ()>:
	"S" { Ok(get_count().try_into().unwrap()) }
	| "IF" "(" Cond_Statement ")" "{" If_Statement "}" 
	{ 
		let v = $1.map_err(|_| ())?;
		let is_valid = $lexer.span_str(v.span());
		if is_valid == "if" {
			println!("{}", is_valid);
			inc_count(); 
		}
		Ok(0)
	}
	;

Cond_Statement -> Result<u64, ()>:
	Term "COND" Term 
	{
		Ok(0)
	}
	;

Term -> Result<String, ()>:
	"WORD"
	{
		let v = $1.map_err(|_| ())?;
		parse_string($lexer.span_str(v.span()))
	}
	| "INT"
	{
		let v = $1.map_err(|_| ())?;
		parse_string($lexer.span_str(v.span()))
	}
    ;

%%
// Any functions here are in scope for all the grammar actions above.
use lazy_static::lazy_static;
use std::sync::Mutex;

lazy_static!{
    static ref COUNT: Mutex<i32> = Mutex::new(0);
}



pub fn get_count() -> i32 {
	COUNT.lock().unwrap().clone()
}

pub fn inc_count() {
	let mut l :i32 = get_count();
	l = l+ 1;
	println!("{}",l);
    *COUNT.lock().unwrap() = l;
}
fn parse_int(s: &str) -> Result<u64, ()> {
    match s.parse::<u64>() {
        Ok(val) => Ok(val),
        Err(_) => {
            eprintln!("{} cannot be represented as a u64", s);
            Err(())
        }
    }
}

fn parse_string(s: &str) -> Result<String, ()> {
	match s.parse::<String>() {
		Ok(val) => Ok(val),
		Err(_) => {
			eprintln!("{} cannot be represented as a String", s);
			Err(())
		}
	}
}

