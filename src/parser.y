%start Start 
%avoid_insert "INT"
%avoid_insert "STR"
%avoid_insert "SINGLE_COMMENT"
%avoid_insert "MULTI_COMMENT"
%avoid_insert "STR_T"
%avoid_insert "INT_T"
%token "BEGIN"
%token "END"
%token "READ"
%token "WRITE"
%token "IF"
%token "THEN"
%token "ELSE"
%token "ENDIF"
%token "WHILE"
%token "DO"
%token "ENDWHILE"
%token "VAR"
%token "BREAK"
%token "CONTINUE"
%token "DECL"
%token "ENDDECL"
%token ";"
%token "="
%nonassoc ">" "<" ">=" '<=' "==" "!="
%left '+' '-'
%left '*' '/' '%'

%%
DeclBlock -> Result<ASTNode,()>:
	"DECL" DeclList "ENDDECL" ';'
	{
		$2
	}
	| "DECL" "ENDDECL" ';'
	{
		Ok(ASTNode::Null)
	}
	;

DeclList -> Result<ASTNode, ()>:
	DeclList Decl 
	{
		let decl = $2?;

		__gentypehash(&decl);

		Ok(ASTNode::BinaryNode{
			op : ASTNodeType::Connector,
            exprtype : ASTExprType::Null,
			lhs : Box::new($1?),
			rhs : Box::new(decl),
		})
	}
	|
	Decl
	{
		let decl =$1?;
		__gentypehash(&decl);
		Ok(decl)
	}
	;

Decl ->  Result<ASTNode,()>:
	Type VarList ';'
	{
		Ok(ASTNode::DeclNode{
			var_type: $1?,
			list: Box::new($2?)
		})
	}
	;

Type -> Result<ASTExprType, ()>:
	"INT_T"
	{
		Ok(ASTExprType::Int)
	} 
	| "STR_T"
	{
		Ok(ASTExprType::String)
	}
	;

VarList -> Result<VarList,()>:
	VarList ',' "VAR" 
	{
		let v = $3.map_err(|_| ())?;
		let var = parse_string($lexer.span_str(v.span())).unwrap();

		Ok(VarList::Node{
			var: var,
			size: 1,
			next: Box::new($1?),
		})
	}
	| VarList ',' "VAR" "[" "INT" "]"
	{
		let v = $3.map_err(|_| ())?;
		let var = parse_string($lexer.span_str(v.span())).unwrap();

		let v = $5.map_err(|_| ())?;
        let varsize = parse_usize($lexer.span_str(v.span())).unwrap();

		Ok(VarList::Node{
			var: var,
			size: varsize,
			next: Box::new($1?),
		})
	
	}
	| "VAR" "[" "INT" "]"
	{
		let v = $1.map_err(|_| ())?;
		let var = parse_string($lexer.span_str(v.span())).unwrap();

		let v = $3.map_err(|_| ())?;
        let varsize = parse_usize($lexer.span_str(v.span())).unwrap();

		Ok(VarList::Node{
			var: var,
			size: varsize,
			next: Box::new(VarList::Null),
		})
	}
	| "VAR" 
	{
		let v = $1.map_err(|_| ())?;
		let var = parse_string($lexer.span_str(v.span())).unwrap();

		Ok(VarList::Node{
			var: var,
			size: 1,
			next: Box::new(VarList::Null),
		})
	}
    ;


	
WhileStmt -> Result<ASTNode, ()>:
    "WHILE" '(' Expr ')' "DO" StmtList "ENDWHILE" ';'
    {
        let expr = $3?;
        if validate_condition_expression(&expr) == Ok(false) {
            return Ok(ASTNode::ErrorNode{
                err : ASTError::TypeError("Expected a boolean expression".to_owned()),
            });
        }
        Ok(ASTNode::WhileNode{
            expr: Box::new(expr),
            xdo: Box::new($6?),
        })
    }
    ;

IfStmt -> Result<ASTNode, ()>:
	"IF" '(' Expr ')' "THEN" StmtList "ELSE" StmtList "ENDIF" ';'
	{
        let expr = $3?;

        if validate_condition_expression(&expr) == Ok(false) {
            return Ok(ASTNode::ErrorNode{
                err : ASTError::TypeError("Expected a boolean expression".to_owned()),
            });
        }
        Ok(ASTNode::IfElseNode{
            expr: Box::new(expr),
            xif: Box::new($6?),
            xelse: Box::new($8?),
        })
	}
	| "IF" '(' Expr ')' "THEN" StmtList "ENDIF" ';'
	{
        let expr = $3?;

        if validate_condition_expression(&expr) == Ok(false) {
            return Ok(ASTNode::ErrorNode{
                err : ASTError::TypeError("Expected a boolean expression".to_owned()),
            });
        }
        Ok(ASTNode::IfNode{
            expr: Box::new(expr),
            xif: Box::new($6?),
        })
	}
	;

Start -> Result<ASTNode, ()>:
	DeclBlock BeginBlock
	{
		Ok(ASTNode::BinaryNode{
			op : ASTNodeType::Connector,
            exprtype : ASTExprType::Null,
			lhs : Box::new($1?),
			rhs : Box::new($2?),
		})
	}
	;
BeginBlock -> Result<ASTNode,()>:
	"BEGIN" StmtList "END" ';'
	{
		$2
	}
	| "BEGIN" "END" ';'
	{
		Ok(ASTNode::Null)
	}
	;

StmtList -> Result<ASTNode, ()>:
	StmtList Stmt 
	{
		Ok(ASTNode::BinaryNode{
			op : ASTNodeType::Connector,
            exprtype : ASTExprType::Null,
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
	| WhileStmt
	{
		$1
	}
    | IfStmt
	{
		$1
	}
	| "BREAK" ';'
	{
		Ok(ASTNode::BreakNode)
	}
	| "CONTINUE" ';'
	{
		Ok(ASTNode::ContinueNode)
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
		let v = $2.map_err(|_| ())?;
		let var = parse_string($lexer.span_str(v.span())).unwrap();

		let lhs = $1?;
		let rhs = $3?;

        if validate_ast_binary_node(&lhs,&rhs,&ASTExprType::Int) == Ok(false){
            return Ok(ASTNode::ErrorNode{ 
                err : ASTError::TypeError("TypeError :: at operator ".to_owned() + var.as_str()),
            });
        }
		Ok(ASTNode::BinaryNode{
			op : ASTNodeType::Equals,
            exprtype : ASTExprType::Null,
			lhs : Box::new(lhs),
			rhs : Box::new(rhs),
		})
	}
	;

Expr -> Result<ASTNode,()>:
	Expr '<' Expr 
	{
		let v = $2.map_err(|_| ())?;
		let var = parse_string($lexer.span_str(v.span())).unwrap();

        let lhs = $1?;
        let rhs = $3?;

        if validate_ast_binary_node(&lhs,&rhs,&ASTExprType::Bool) == Ok(false){
            return Ok(ASTNode::ErrorNode{ 
                err : ASTError::TypeError("TypeError :: at operator ".to_owned() + var.as_str()),
            });
        }
		Ok(ASTNode::BinaryNode{
			op : ASTNodeType::Lt,
			exprtype : ASTExprType::Bool,
			lhs : Box::new(lhs),
			rhs : Box::new(rhs),
		})
	}
	| Expr '>' Expr 
	{
		let v = $2.map_err(|_| ())?;
		let var = parse_string($lexer.span_str(v.span())).unwrap();

        let lhs = $1?;
        let rhs = $3?;

        if validate_ast_binary_node(&lhs,&rhs,&ASTExprType::Bool) == Ok(false){
            return Ok(ASTNode::ErrorNode{ 
                err : ASTError::TypeError("TypeError :: at operator ".to_owned() + var.as_str()),
            });
        }

		Ok(ASTNode::BinaryNode{
			op : ASTNodeType::Gt,
			exprtype : ASTExprType::Bool,
			lhs : Box::new(lhs),
			rhs : Box::new(rhs),
		})
	}
	| Expr '<=' Expr 
	{
		let v = $2.map_err(|_| ())?;
		let var = parse_string($lexer.span_str(v.span())).unwrap();

        let lhs = $1?;
        let rhs = $3?;

        if validate_ast_binary_node(&lhs,&rhs,&ASTExprType::Bool) == Ok(false){
            return Ok(ASTNode::ErrorNode{ 
                err : ASTError::TypeError("TypeError :: at operator ".to_owned() + var.as_str()),
            });
        }

		Ok(ASTNode::BinaryNode{
			op : ASTNodeType::Lte,
			exprtype : ASTExprType::Bool,
			lhs : Box::new(lhs),
			rhs : Box::new(rhs),
		})
	}
	| Expr '>=' Expr 
	{
		let v = $2.map_err(|_| ())?;
		let var = parse_string($lexer.span_str(v.span())).unwrap();

        let lhs = $1?;
        let rhs = $3?;

        if validate_ast_binary_node(&lhs,&rhs,&ASTExprType::Bool) == Ok(false){
            return Ok(ASTNode::ErrorNode{ 
                err : ASTError::TypeError("TypeError :: at operator ".to_owned() + var.as_str()),
            });
        }

		Ok(ASTNode::BinaryNode{
			op : ASTNodeType::Gte,
			exprtype : ASTExprType::Bool,
			lhs : Box::new(lhs),
			rhs : Box::new(rhs),
		})
	}
	| Expr '!=' Expr 
	{
		let v = $2.map_err(|_| ())?;
		let var = parse_string($lexer.span_str(v.span())).unwrap();

        let lhs = $1?;
        let rhs = $3?;

        if validate_ast_binary_node(&lhs,&rhs,&ASTExprType::Bool) == Ok(false){
            return Ok(ASTNode::ErrorNode{ 
                err : ASTError::TypeError("TypeError :: at operator ".to_owned() + var.as_str()),
            });
        }

		Ok(ASTNode::BinaryNode{
			op : ASTNodeType::Ne,
			exprtype : ASTExprType::Bool,
			lhs : Box::new(lhs),
			rhs : Box::new(rhs),
		})
	}
	| Expr '==' Expr 
	{
		let v = $2.map_err(|_| ())?;
		let var = parse_string($lexer.span_str(v.span())).unwrap();

        let lhs = $1?;
        let rhs = $3?;

        if validate_ast_binary_node(&lhs,&rhs,&ASTExprType::Bool) == Ok(false){
            return Ok(ASTNode::ErrorNode{ 
                err : ASTError::TypeError("TypeError :: at operator ".to_owned() + var.as_str()),
            });
        }

		Ok(ASTNode::BinaryNode{
			op : ASTNodeType::Ee,
			exprtype : ASTExprType::Bool,
			lhs : Box::new(lhs),
			rhs : Box::new(rhs),
		})
	}
	| Expr '+' Expr
	{
		let v = $2.map_err(|_| ())?;
		let var = parse_string($lexer.span_str(v.span())).unwrap();

        let lhs = $1?;
        let rhs = $3?;

        if validate_ast_binary_node(&lhs,&rhs,&ASTExprType::Int) == Ok(false){
            return Ok(ASTNode::ErrorNode{ 
                err : ASTError::TypeError("TypeError :: at operator ".to_owned() + var.as_str()),
            });
        }

		Ok(ASTNode::BinaryNode{
			op : ASTNodeType::Plus,
			exprtype : ASTExprType::Int,
			lhs : Box::new(lhs),
			rhs : Box::new(rhs),
		})
	}
	| Expr '-' Expr
	{
		let v = $2.map_err(|_| ())?;
		let var = parse_string($lexer.span_str(v.span())).unwrap();

        let lhs = $1?;
        let rhs = $3?;

        if validate_ast_binary_node(&lhs,&rhs,&ASTExprType::Int) == Ok(false){
            return Ok(ASTNode::ErrorNode{ 
                err : ASTError::TypeError("TypeError :: at operator ".to_owned() + var.as_str()),
            });
        }

		Ok(ASTNode::BinaryNode{
			op : ASTNodeType::Minus,
			exprtype : ASTExprType::Int,
			lhs : Box::new(lhs),
			rhs : Box::new(rhs),
		})
	}
	| Expr '*' Expr
	{
		let v = $2.map_err(|_| ())?;
		let var = parse_string($lexer.span_str(v.span())).unwrap();
        let lhs = $1?;
        let rhs = $3?;

        if validate_ast_binary_node(&lhs,&rhs,&ASTExprType::Int) == Ok(false){
            return Ok(ASTNode::ErrorNode{ 
                err : ASTError::TypeError("TypeError :: at operator ".to_owned() + var.as_str()),
            });
        }

		Ok(ASTNode::BinaryNode{
			op : ASTNodeType::Star,
			exprtype : ASTExprType::Int,
			lhs : Box::new(lhs),
			rhs : Box::new(rhs),
		})
	}
	| Expr '/' Expr
	{
		let v = $2.map_err(|_| ())?;
		let var = parse_string($lexer.span_str(v.span())).unwrap();

        let lhs = $1?;
        let rhs = $3?;

        if validate_ast_binary_node(&lhs,&rhs,&ASTExprType::Int) == Ok(false){
            return Ok(ASTNode::ErrorNode{ 
                err : ASTError::TypeError("TypeError :: at operator ".to_owned() + var.as_str()),
            });
        }

		Ok(ASTNode::BinaryNode{
			op : ASTNodeType::Slash,
			exprtype : ASTExprType::Int,
			lhs : Box::new(lhs),
			rhs : Box::new(rhs),
		})
	}
	| Expr '%' Expr
	{
		let v = $2.map_err(|_| ())?;
		let var = parse_string($lexer.span_str(v.span())).unwrap();

        let lhs = $1?;
        let rhs = $3?;

        if validate_ast_binary_node(&lhs,&rhs,&ASTExprType::Int) == Ok(false){
            return Ok(ASTNode::ErrorNode{ 
                err : ASTError::TypeError("TypeError :: at operator ".to_owned() + var.as_str()),
            });
        }

		Ok(ASTNode::BinaryNode{
			op : ASTNodeType::Mod,
			exprtype : ASTExprType::Int,
			lhs : Box::new(lhs),
			rhs : Box::new(rhs),
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
	| "STR"
	{
		let v = $1.map_err(|_| ())?;
		let str = parse_string($lexer.span_str(v.span())).unwrap();
		log::info!("String detected: {}",str);
		Ok(ASTNode::STR(str))
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
		Ok(ASTNode::VAR{
			name: var,
			index1: Box::new(ASTNode::Null),
		})
	}
	| "VAR" "[" Expr "]"
	{
		let v = $1.map_err(|_| ())?;
		let var = parse_string($lexer.span_str(v.span())).unwrap();

		let expr = $3?;
		if validate_index(&expr) == Ok(false) {
            return Ok(ASTNode::ErrorNode{ 
                err : ASTError::TypeError("Invalid Expression while indexing : ".to_owned() + var.as_str()),
            });
		}
		
		Ok(ASTNode::VAR{
			name: var,
			index1: Box::new(expr),
		})

	}
	;
%%
// Any functions here are in scope for all the grammar actions above.
use crate::parserlib::{*};
