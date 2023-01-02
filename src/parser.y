%start Start 
%avoid_insert "INT"
%avoid_insert "FLOAT"
%avoid_insert "SINGLE_COMMENT"
%avoid_insert "MULTI_COMMENT"
%token "BEGIN"
%token "END"
%token "READ"
%token "WRITE"
%token "VAR"
%token "COND"
%token "{"
%token "}"
%token "("
%token ")"
%token ";"
%token "IF"
%token "FI"
%token "="
%left '+' '-'
%left '*' '/'

%%
Start -> Result<ASTNode, ()>:
	"BEGIN" StmtList "END" ';'
	{
		$2	
	}
	| "BEGIN" "END" ';'
	{
		Ok(ASTNode::Null(0))	
	}
	;

StmtList -> Result<ASTNode, ()>:
	StmtList Stmt 
	{
		Ok(ASTNode::BinaryNode{
			op : ASTNodeType::Connector,
			lhs : Box::new($1?),
			rhs : Box::new($2?),
		})
	}
	| Stmt
	{
		$1
	}
	;

Stmt -> Result<ASTNode,()>:
	InputStmt 
	{
		$1
	}
	| OutputStmt 
	{
		$1
	}
	| AssgStmt 
	{
		$1
	}
	;

InputStmt -> Result<ASTNode, ()>:
	"READ" '(' Variable ')' ';'
	{
		Ok(ASTNode::UnaryNode{
			op : ASTNodeType::Read,
			ptr : Box::new($3?),
		})

	}
	;

OutputStmt -> Result<ASTNode, ()>:
	"WRITE" '(' Expr ')' ';' 
	{
		Ok(ASTNode::UnaryNode{
			op : ASTNodeType::Write,
			ptr : Box::new($3?),
		})
	}
	;
AssgStmt -> Result<ASTNode, ()>:
	Variable '=' Expr ';'
	{
		Ok(ASTNode::BinaryNode{
			op : ASTNodeType::Equals,
			lhs : Box::new($1?),
			rhs : Box::new($3?),
		})
	}
	;

Expr -> Result<ASTNode,()>:
	Expr '+' Expr 
	{
        Ok(ASTNode::BinaryNode{
            op : ASTNodeType::Plus,
            lhs : Box::new($1?),
            rhs : Box::new($3?),
        })
	}
	| Expr '-' Expr
	{
        Ok(ASTNode::BinaryNode{
            op : ASTNodeType::Minus,
            lhs : Box::new($1?),
            rhs : Box::new($3?),
        })
	}
	| Expr '*' Expr
	{
        Ok(ASTNode::BinaryNode{
            op : ASTNodeType::Star,
            lhs : Box::new($1?),
            rhs : Box::new($3?),
        })
	}
	| Expr '/' Expr
	{
        Ok(ASTNode::BinaryNode{
            op : ASTNodeType::Slash,
            lhs : Box::new($1?),
            rhs : Box::new($3?),
        })
	}
    | '(' Expr ')'
    {
        $2
    }
    | "INT"
	{
		let v = $1.map_err(|_| ())?;
        let num  = parse_int($lexer.span_str(v.span())).unwrap();
        Ok(ASTNode::INT(num))
	}
	| Variable
	{
		$1
	}
    ; 

Variable -> Result<ASTNode,()>:
	"VAR"
	{
		let v = $1.map_err(|_| ())?;
		let var = parse_string($lexer.span_str(v.span())).unwrap();
		Ok(ASTNode::VAR(var))
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
pub enum ASTNodeType {
    Plus,
    Minus,
    Star,
    Slash,
	Equals,
    Read,
    Write,
    Connector,
}

#[derive(Debug)]

pub enum ASTNode {
    INT(i64),
    VAR(String),
    BinaryNode {
        op: ASTNodeType,
        lhs: Box<ASTNode>,
        rhs: Box<ASTNode>,
    },
	UnaryNode{
		op: ASTNodeType,
		ptr: Box<ASTNode>,
	},
	Null(i64),
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
	Ok(s.to_owned())
}

