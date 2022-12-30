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
%left '+' '-'
%left '*' '/'

%%
Start -> Result<Node, ()>:
	Expr { $1 }
	;
Expr -> Result<Node,()>:
	'+' Expr Expr 
	{
        Ok(Node::BinaryExpr{
            op : Operator::Plus,
            lhs : Box::new($2?),
            rhs : Box::new($3?),
        })
	}
	| '-' Expr Expr
	{
        Ok(Node::BinaryExpr{
            op : Operator::Minus,
            lhs : Box::new($2?),
            rhs : Box::new($3?),
        })
	}
	| '*' Expr Expr
	{
        Ok(Node::BinaryExpr{
            op : Operator::Star,
            lhs : Box::new($2?),
            rhs : Box::new($3?),
        })
	}
	| '/' Expr Expr
	{
        Ok(Node::BinaryExpr{
            op : Operator::Slash,
            lhs : Box::new($2?),
            rhs : Box::new($3?),
        })
	}
    | '(' Expr ')'
    {
        $2
    }
    | 
	"INT"
	{
		let v = $1.map_err(|_| ())?;
        let num  = parse_int($lexer.span_str(v.span())).unwrap();
        Ok(Node::INT(num))
	}
    ; 

%%
// Any functions here are in scope for all the grammar actions above.
use lazy_static::lazy_static;
use std::sync::Mutex;
use std::fmt::Debug;

lazy_static!{
    pub static ref COUNT: Mutex<i32> = Mutex::new(0);
}

#[derive(Debug)]
pub enum Operator {
    Plus,
    Minus,
    Star,
    Slash,
}
#[derive(Debug)]

pub enum Node {
    INT(i64),
    BinaryExpr {
        op: Operator,
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
}
fn parse_int(s: &str) -> Result<i64, ()> {
    match s.parse::<i64>() {
        Ok(val) => Ok(val),
        Err(_) => {
            eprintln!("{} cannot be represented as a i64", s);
            Err(())
        }
    }
}

pub fn parse_string(s: &str) -> Result<String, ()> {
	Ok(s.to_owned() + " ")
}

