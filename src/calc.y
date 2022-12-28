%start Start 
%avoid_insert "INT"
%avoid_insert "FLOAT"
%avoid_insert "WORD"
%avoid_insert "SINGLE_COMMENT"
%avoid_insert "MULTI_COMMENT"
%avoid_insert "SPEC"
%token "VAR"
%token "COND"
%token "S"
%token "{"
%token "}"
%token "("
%token ")"
%token "IF"
%token "FI"

%%
Start -> Result<String , ()>:
	Expr { $1 }
	;

Factor -> Result<String, ()>:
	"VAR"
	{
		let v = $1.map_err(|_| ())?;
		parse_string($lexer.span_str(v.span()))
	}
	| "INT"
	{
		let v = $1.map_err(|_| ())?;
		parse_string($lexer.span_str(v.span()))
	}
	| '(' Expr ')' 
	{
		$2
	}
    ;
	
Expr -> Result<String, ()>:
	Expr '+' Term 
	{
		match $1 {
			Ok(expr) => {
				match $3 {
					Ok(term) => {
						let mut res = String::from(expr).clone();
						res.push_str(&term[..]);
						res.push_str("+");
						res.push_str(" ");
						Ok(res)
					}
					Err(e) => {
						Err(())
					}
				}
			}
			Err(e) => {
				Err(())
			}
		}	
	} 
	| Expr '-' Term
	{
		match $1 {
			Ok(expr) => {
				match $3 {
					Ok(term) => {
						let mut res = String::from(expr).clone();
						res.push_str(&term[..]);
						res.push_str("-");
						res.push_str(" ");
						Ok(res)
					}
					Err(e) => {
						Err(())
					}
				}
			}
			Err(e) => {
				Err(())
			}
		}	
		
	}
	| Term
	{
		$1
	}
	;

Term -> Result<String, ()>:
	Term '*' Factor
	{
		match $1 {
			Ok(term) => {
				match $3 {
					Ok(factor) => {
						let mut res = String::from(term).clone();
						res.push_str(&factor[..]);
						res.push_str("*");
						res.push_str(" ");
						Ok(res)
					}
					Err(e)=>{
						Err(())
					}
				}
			}
			Err(e) => {
				Err(())
			}
		}	

	}
	| Term '/' Factor
	{
		match $1 {
			Ok(term) => {
				match $3 {
					Ok(factor) => {
						let mut res = String::from(term).clone();
						res.push_str(&factor[..]);
						res.push_str("/");
						res.push_str(" ");
						Ok(res)
					}
					Err(e) => {
						Err(())
					}
				}
			}
			Err(e) => {
				Err(())
			}
		}	


	}
	|
	Factor 
	{
		$1
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
	Ok(s.to_owned() + " ")
}

