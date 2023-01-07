%start Start 
%avoid_insert "INT"
%avoid_insert "FLOAT"
%avoid_insert "SINGLE_COMMENT"
%avoid_insert "MULTI_COMMENT"
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
%token ";"
%token "="
%nonassoc ">" "<" ">=" '<=' "==" "!="
%left '+' '-'
%left '*' '/'

%%
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
		Ok(ASTNode::BinaryNode{
			op : ASTNodeType::Equals,
            exprtype : ASTExprType::Null,
			lhs : Box::new($1?),
			rhs : Box::new($3?),
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

        if validate_ast_binary_node(&lhs,&rhs,&ASTExprType::Bool) == Ok(false){
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

        if validate_ast_binary_node(&lhs,&rhs,&ASTExprType::Bool) == Ok(false){
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

        if validate_ast_binary_node(&lhs,&rhs,&ASTExprType::Bool) == Ok(false){
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

        if validate_ast_binary_node(&lhs,&rhs,&ASTExprType::Bool) == Ok(false){
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
use crate::parserlib::{*};
