%start Start 
%avoid_insert "INT"
%avoid_insert "MAIN"
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
%token "BREAKPOINT"
%token "CONTINUE"
%token "MAIN"
%token "DECL"
%token "ENDDECL"
%token "RETURN"
%token ";"
%token "="
%nonassoc ">" "<" ">=" '<=' "==" "!="
%left '+' '-'
%left '*' '/' '%'
%%

PtrPtr -> Result<ASTExprType, ()>:
	PtrPtr '*'
	{
		Ok(ASTExprType::Pointer(Box::new($1?)))
	}
	| '*'
	{
		Ok(ASTExprType::Pointer(Box::new(ASTExprType::Primitive(PrimitiveType::Void))))
	}
	;
Type -> Result<ASTExprType, ()>:
	'INT_T'
	{
		Ok(ASTExprType::Primitive(PrimitiveType::Int))
	} 
	| 'STR_T'
	{
		Ok(ASTExprType::Primitive(PrimitiveType::String))
	}
    ;
DeclType -> Result<ASTExprType, ()>:
	'INT_T'
	{
		let mut dt = DECL_TYPE.lock().unwrap();
		*dt = ASTExprType::Primitive(PrimitiveType::Int);
		Ok(dt.clone())
	} 
	| 'STR_T'
	{
		let mut dt = DECL_TYPE.lock().unwrap();
		*dt = ASTExprType::Primitive(PrimitiveType::String);
		Ok(dt.clone())
	}
    ;

//Big Picture
Start -> Result<ASTNode, ()>:
	GDeclBlock FDefBlock MainBlock
	{
		Ok(ASTNode::BinaryNode{
			op : ASTNodeType::Connector,
            exprtype : Some(ASTExprType::Primitive(PrimitiveType::Null)),
			lhs : Box::new(ASTNode::BinaryNode{
                op: ASTNodeType::Connector,
				exprtype : Some(ASTExprType::Primitive(PrimitiveType::Null)),
                lhs: Box::new($3?),
                rhs: Box::new($2?),
            }),
			rhs : Box::new(ASTNode::Null),
		})
	}
	| GDeclBlock MainBlock 
	{
		Ok(ASTNode::BinaryNode{
			op : ASTNodeType::Connector,
            exprtype : Some(ASTExprType::Primitive(PrimitiveType::Null)),
			lhs : Box::new($2?),
			rhs : Box::new(ASTNode::Null),
		})
	}
    | MainBlock 
    {
        Ok(ASTNode::BinaryNode{
            op: ASTNodeType::Connector,
            exprtype : Some(ASTExprType::Primitive(PrimitiveType::Null)),
            lhs: Box::new(ASTNode::Null),
            rhs: Box::new($1?),
        })
    }
	;
MainBlock -> Result<ASTNode,()>:
	DeclType "MAIN" '('  ')' '{' LDeclBlock BeginBlock '}'
	{
		let ldecl_ = $6.map_err(|_| ())?;
		let body_ = $7.map_err(|_| ())?;
		let node = ASTNode::MainNode{
			decl: Box::new(ldecl_),
			body: Box::new(body_),
		};
		let mut ft = FUNCTION_TABLE.lock().unwrap();
		let lst = LOCALSYMBOLTABLE.lock().unwrap();
		ft.insert(
			"main".to_string(),
			lst.clone()
		);
		Ok(node)
	}
	;
BeginBlock -> Result<ASTNode,()>:
	"BEGIN" StmtList "END" 
	{
		$2
	}
	| "BEGIN" "END" 
	{
		Ok(ASTNode::Null)
	}
	;
GDeclBlock -> ():
	"DECL" GDeclList "ENDDECL" 
	{
		$2;
	}
	| "DECL" "ENDDECL" 
	{
	}
	;
LDeclList -> Result<LinkedList<ASTNode>, ()>:
	LDeclList LDecl 
	{
		let mut list  = LinkedList::new();
		list.append(&mut ($1)?);
		list.append(&mut ($2)?);
		Ok(list)
	}
	|
	LDecl
	{
		let mut list = LinkedList::new();
		list.append(&mut $1?);
		Ok(list)
	}
	;
GDeclList -> ():
	GDecl GDeclList 
	{
		$1;
		$2;
	}
	| GDecl
	{
		$1;
	}
	;
LDecl ->  Result<LinkedList<ASTNode>,()>:
	DeclType LLine ';'
	{
		let vtype = $1?;
		let list = $2?;
		let node = ASTNode::DeclNode{
			var_type: vtype,
			list: Box::new(list)
		};
		Ok(LinkedList::from(node))
	}
	;
GDecl ->  ():
	DeclType GLine ';'
	{
		$1;
		$2;
	}
	;
GLine -> ():
	GItem ',' GLine
	{
		$1;
		$2;
	}
	| GItem
	{
		$1
	}
	;
GItem -> ():
	"VAR" '(' ParamList ')' 
	{
		let returntype = DECL_TYPE.lock().unwrap().clone();
		let v = $1.map_err(|_| ())?;
		let functionname= parse_string($lexer.span_str(v.span())).unwrap();
		let paramlist = $3?;
		install_func_to_gst(functionname,returntype,&paramlist);
	}
	| VarItem
	{
		let node = $1?;
		let dt = DECL_TYPE.lock().unwrap();
		node.vartype.set_base_type(dt.clone());
		node.install_to_gst();
		std::mem::drop(dt);
	}
	;

LLine -> ():
	VarItem ',' LLine
	{
		let node = $1?;
		let dt = DECL_TYPE.lock().unwrap();
		node.vartype.set_base_type(dt.clone());
		node.install_to_lst();
		std::mem::drop(dt);
		$3?;
	}
	| VarItem 
	{
		let node =$1?;
		let dt = DECL_TYPE.lock().unwrap();
		node.vartype.set_base_type(dt.clone());
		node.install_to_lst();
		std::mem::drop(dt);
	}
	;
VarItem -> Result<VarNode, ()>:
	VariableDef 
	{
		$1?
	} 
	| PtrPtr VariableDef
	{
		let dt = DECL_TYPE.lock().unwrap();
		let mut node= $2?;
		node.vartype = $1?;
		Ok(node);
	}
    ;
FDefBlock -> Result<ASTNode,()>:
	FDefBlock FDef 
	{
		Ok(ASTNode::BinaryNode{
			op: ASTNodeType::Connector,
            exprtype : Some(ASTExprType::Primitive(PrimitiveType::Null)),
			lhs: Box::new($1?),
			rhs: Box::new($2?),
		})
	}
	| FDef
	{
		$1
	}
	;
FDef ->Result<ASTNode,()>:
	DeclType "VAR" '(' ParamListBlock ')' '{' LDeclBlock BeginBlock '}'
	{
		let v = $2.map_err(|_| ())?;
		let funcname = parse_string($lexer.span_str(v.span())).unwrap();
		let ldecl_ = $7?;
		let paramlist_ = $4?;
		let node = ASTNode::FuncDefNode{
			fname: funcname.clone(),
			ret_type: $1?,
			paramlist: Box::new(paramlist_),
			decl: Box::new(ldecl_),
			body: Box::new($8?),
		};
		if validate_ast_node(&node) == Ok(false) {
			return Ok(ASTNode::ErrorNode{
				err : ASTError::TypeError("Function [".to_owned() + funcname.as_str() + "] declaration does not match with definition"),
			});
		}
		let mut lst = LOCALSYMBOLTABLE.lock().unwrap();
		let mut ft = FUNCTION_TABLE.lock().unwrap();
		let mut lv = LOCALVARID.lock().unwrap();
		*lv = 1;
		ft.insert(
			funcname,
			lst.clone()
		);
		lst.clear();
		std::mem::drop(lv);
		std::mem::drop(lst);
		std::mem::drop(ft);
		Ok(node)
	}
	;

LDeclBlock -> Result<LinkedList<ASTNode>,()>:
	"DECL" LDeclList "ENDDECL"
	{
		$2
	}
	| "DECL" "ENDDECL"
	{
        Ok(LinkedList::new())
	}
	;

ParamListBlock -> Result<LinkedList<VarNode>,()>:
	ParamList 
	{
		let node = $1?;
		__lst_install_params(&node);
		Ok(node)
	}
	;

ParamList -> Result<LinkedList<VarNode>,()>:
	ParamList ',' Param
	{
        let mut paramlist = $3?;
		paramlist.append(&mut $1?);
		Ok(paramlist)
	}
	| Param
	{
		$1
	}
	;

Param -> Result<LinkedList<VarNode>,()>:
	DeclType VariableDef 
    {
		let var = $2?;
        let vtype = $1?;
		if var.varindices.len() != 0 {
			exit_on_err("Arrays cannot be used as a function parameter. Use a pointer instead.".to_owned());
		}
		var.vartype= vtype;
		Ok(LinkedList::from(var))
    }
	;
ArgList -> Result<LinkedList<ASTNode>,()>:
	ArgList ',' Expr
	{
		let expr = $3?;
		let mut arglist = $1?;
		arglist.push_back(expr);
		Ok(arglist)
	}
	| Expr
	{
		Ok(LinkedList::from($1?))
	}
	;
VariableDef -> Result<VarNode,()>:
	"VAR" 
	{
		let v = $1.map_err(|_| ())?;
		let var_ = parse_string($lexer.span_str(v.span())).unwrap();
		let mut ll = LinkedList::new();
		VarNode{
			varname: var_,
			vartype: ASTExprType::Primitive(PrimitiveType::Void),
			varindices: vec![],
		}
	}
	| "VAR" "[" "INT" "]"
	{
		let v = $1.map_err(|_| ())?;
		let var_ = parse_string($lexer.span_str(v.span())).unwrap();
		let v = $3.map_err(|_| ())?;
        let i= parse_usize($lexer.span_str(v.span())).unwrap();
		VarNode{
			varname: var_,
			vartype: ASTExprType::Primitive(PrimitiveType::Void),
			varindices: vec![i],
		}
	}
	| "VAR" "[" "INT" "]" "[" "INT" "]"
	{
		let v = $1.map_err(|_| ())?;
		let var_ = parse_string($lexer.span_str(v.span())).unwrap();
		let v = $3.map_err(|_| ())?;
        let i= parse_usize($lexer.span_str(v.span())).unwrap();
		let v = $6.map_err(|_| ())?;
        let j= parse_usize($lexer.span_str(v.span())).unwrap();
		VarNode{
			varname: var_,
			vartype: ASTExprType::Primitive(PrimitiveType::Void),
			varindices: vec![i,j],
		}
	}
	;	
//StateMents	
StmtList -> Result<ASTNode, ()>:
	StmtList Stmt 
	{
		Ok(ASTNode::BinaryNode{
			op : ASTNodeType::Connector,
            exprtype : Some(ASTExprType::Primitive(PrimitiveType::Null)),
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
	| "BREAKPOINT" ';'
	{
		Ok(ASTNode::BreakpointNode)
	}
	| "BREAK" ';'
	{
		Ok(ASTNode::BreakNode)
	}
	| "CONTINUE" ';'
	{
		Ok(ASTNode::ContinueNode)
	}
	| "RETURN" Expr ';'
	{
		let node = ASTNode::ReturnNode{
			expr: Box::new($2?)
		};
		if validate_ast_node(&node) == Ok(false) {
			return Ok(ASTNode::ErrorNode{
                err : ASTError::TypeError("Return expression type mismatch".to_owned()),
            });
		}
		Ok(node)
	}
	;
WhileStmt -> Result<ASTNode, ()>:
    "WHILE" '(' Expr ')' "DO" StmtList "ENDWHILE" ';'
    {
        let expr = $3?;
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
        Ok(ASTNode::IfElseNode{
            expr: Box::new(expr),
            xif: Box::new($6?),
            xelse: Box::new($8?),
        })
	}
	| "IF" '(' Expr ')' "THEN" StmtList "ENDIF" ';'
	{
        let expr = $3?;
        Ok(ASTNode::IfNode{
            expr: Box::new(expr),
            xif: Box::new($6?),
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
		let lhs = $1?;
		let rhs = $3?;
		let node = ASTNode::BinaryNode{
			op : ASTNodeType::Equals,
            exprtype : Some(ASTExprType::Primitive(PrimitiveType::Null)),
			lhs : Box::new(lhs),
			rhs : Box::new(rhs),
		};
		if validate_ast_node(&node) == Ok(false) {
			return Ok(ASTNode::ErrorNode{
                err : ASTError::TypeError("Type mismatch at assignment".to_owned()),
            });
		}
		Ok(node)
	}
	| '*' Variable '=' Expr ';'
	{
		let lhs = $2?;
		let rhs = $4?;
		let node = ASTNode::BinaryNode{
			op : ASTNodeType::Equals,
            exprtype : Some(ASTExprType::Primitive(PrimitiveType::Null)),
			lhs : Box::new(ASTNode::UnaryNode{
				op: ASTNodeType::Deref,
				ptr: Box::new(lhs)
			}),
			rhs : Box::new(rhs),
		};
		if validate_ast_node(&node) == Ok(false) {
			return Ok(ASTNode::ErrorNode{
                err : ASTError::TypeError("Type mismatch at assignment".to_owned()),
            });
		}
		Ok(node)
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
	| "READ" '(' '*' Variable ')' ';'
	{
		let var = $4?;
		Ok(ASTNode::UnaryNode{
			op : ASTNodeType::Read,
			ptr : Box::new(ASTNode::UnaryNode{
				op: ASTNodeType::Deref,
				ptr: Box::new(var),
			}),
		})
	}
	;
Expr -> Result<ASTNode,()>:
	Expr '<' Expr 
	{
        let lhs = $1?;
        let rhs = $3?;
		
		let node = ASTNode::BinaryNode{
			op : ASTNodeType::Lt,
			exprtype : None,
			lhs : Box::new(lhs),
			rhs : Box::new(rhs),
		};
		if validate_ast_node(&node) == Ok(false) {
			return Ok(ASTNode::ErrorNode{
                err : ASTError::TypeError("Type mismatch at <".to_owned()),
            });
		}
		Ok(node)
	}
	| Expr '>' Expr 
	{
        let lhs = $1?;
        let rhs = $3?;
		let node = ASTNode::BinaryNode{
			op : ASTNodeType::Gt,
			exprtype : None,
			lhs : Box::new(lhs),
			rhs : Box::new(rhs),
		};
		if validate_ast_node(&node) == Ok(false) {
			return Ok(ASTNode::ErrorNode{
                err : ASTError::TypeError("Type error at >".to_owned()),
            });
		}
		Ok(node)
	}
	| Expr '<=' Expr 
	{
        let lhs = $1?;
        let rhs = $3?;
		let node = ASTNode::BinaryNode{
			op : ASTNodeType::Lte,
			exprtype : None,
			lhs : Box::new(lhs),
			rhs : Box::new(rhs),
		};
		if validate_ast_node(&node) == Ok(false) {
			return Ok(ASTNode::ErrorNode{
                err : ASTError::TypeError("Type error at <=".to_owned()),
            });
		}
		Ok(node)
	}
	| Expr '>=' Expr 
	{
        let lhs = $1?;
        let rhs = $3?;
		let node = ASTNode::BinaryNode{
			op : ASTNodeType::Gte,
			exprtype : None,
			lhs : Box::new(lhs),
			rhs : Box::new(rhs),
		};
		if validate_ast_node(&node) == Ok(false) {
			return Ok(ASTNode::ErrorNode{
                err : ASTError::TypeError("Type error at >=".to_owned()),
            });
		}
		Ok(node)
	}
	| Expr '!=' Expr 
	{
        let lhs = $1?;
        let rhs = $3?;
		let node = ASTNode::BinaryNode{
			op : ASTNodeType::Ne,
			exprtype : None,
			lhs : Box::new(lhs),
			rhs : Box::new(rhs),
		};
		if validate_ast_node(&node) == Ok(false) {
			return Ok(ASTNode::ErrorNode{
                err : ASTError::TypeError("Type error at !=".to_owned()),
            });
		}
		Ok(node)
	}
	| Expr '==' Expr 
	{
        let lhs = $1?;
        let rhs = $3?;
		let node = ASTNode::BinaryNode{
			op : ASTNodeType::Ee,
			exprtype : None,
			lhs : Box::new(lhs),
			rhs : Box::new(rhs),
		};
		if validate_ast_node(&node) == Ok(false) {
			return Ok(ASTNode::ErrorNode{
                err : ASTError::TypeError("Type error at ==".to_owned()),
            });
		}
		Ok(node)
	}
	| Expr '+' Expr
	{
        let lhs = $1?;
        let rhs = $3?;
		let node = ASTNode::BinaryNode{
			op : ASTNodeType::Plus,
			exprtype : None,
			lhs : Box::new(lhs),
			rhs : Box::new(rhs),
		};
		if validate_ast_node(&node) == Ok(false) {
			return Ok(ASTNode::ErrorNode{
                err : ASTError::TypeError("Type error at +".to_owned()),
            });
		}
		Ok(node)
	}
	| Expr '-' Expr
	{
        let lhs = $1?;
        let rhs = $3?;
		let node = ASTNode::BinaryNode{
			op : ASTNodeType::Minus,
			exprtype : None,
			lhs : Box::new(lhs),
			rhs : Box::new(rhs),
		};
		if validate_ast_node(&node) == Ok(false) {
			return Ok(ASTNode::ErrorNode{
                err : ASTError::TypeError("Type error at -".to_owned()),
            });
		}
		Ok(node)
	}
	| Expr '*' Expr
	{
        let lhs = $1?;
        let rhs = $3?;
		let node = ASTNode::BinaryNode{
			op : ASTNodeType::Star,
			exprtype : None,
			lhs : Box::new(lhs),
			rhs : Box::new(rhs),
		};
		if validate_ast_node(&node) == Ok(false) {
			return Ok(ASTNode::ErrorNode{
                err : ASTError::TypeError("Type error at *".to_owned()),
            });
		}
		Ok(node)
	}
	| Expr '/' Expr
	{
        let lhs = $1?;
        let rhs = $3?;
		let node = ASTNode::BinaryNode{
			op : ASTNodeType::Slash,
			exprtype : None,
			lhs : Box::new(lhs),
			rhs : Box::new(rhs),
		};
		if validate_ast_node(&node) == Ok(false) {
			return Ok(ASTNode::ErrorNode{
                err : ASTError::TypeError("Type error at /".to_owned()),
            });
		}
		Ok(node)
	}
	| Expr '%' Expr
	{
        let lhs = $1?;
        let rhs = $3?;
		let node = ASTNode::BinaryNode{
			op : ASTNodeType::Mod,
			exprtype : None,
			lhs : Box::new(lhs),
			rhs : Box::new(rhs),
		};
		if validate_ast_node(&node) == Ok(false) {
			return Ok(ASTNode::ErrorNode{
                err : ASTError::TypeError("Type error at %".to_owned()),
            });
		}
		Ok(node)
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
		Ok(ASTNode::STR(str))
	}
	| VariableExpr
	{
		$1
	}
	| "VAR" '(' ')'
	{
		let v = $1.map_err(|_| ())?;
		let functionname= parse_string($lexer.span_str(v.span())).unwrap();
		let node = ASTNode::FuncCallNode{
			fname: functionname, 
			arglist: Box::new(LinkedList::new()),
		};
		if validate_ast_node(&node) == Ok(false) {
			return Ok(ASTNode::ErrorNode{
				err: ASTError::TypeError("Function call does not match declaration".to_string())
            });
		}
		Ok(node)
	}
	| "VAR" '(' ArgList ')'
	{
		let v = $1.map_err(|_| ())?;
		let functionname= parse_string($lexer.span_str(v.span())).unwrap();
		let node = ASTNode::FuncCallNode{
			fname: functionname, 
			arglist: Box::new($3?),
		};
		if validate_ast_node(&node) == Ok(false) {
			return Ok(ASTNode::ErrorNode{
				err: ASTError::TypeError("Function call does not match declaration".to_string())
			})
		}
		Ok(node)
	}
    ; 

//Variables around the code
Variable -> Result<ASTNode, ()>:
	"VAR"
	{
		let v = $1.map_err(|_| ())?;
		let var = parse_string($lexer.span_str(v.span())).unwrap();
		Ok(ASTNode::VAR{
			name: var,
			indices: Vec::default(),
		})
	}
	| "VAR" "[" Expr "]"
	{
		let v = $1.map_err(|_| ())?;
		let var = parse_string($lexer.span_str(v.span())).unwrap();
		let expr = $3?;
        let mut ind : Vec<Box<ASTNode>> = Vec::default();
        ind.push(Box::new(expr));
		Ok(ASTNode::VAR{
			name: var,
			indices: ind,
		})
	}
	| "VAR" "[" Expr "]" "[" Expr "]"
	{
		let v = $1.map_err(|_| ())?;
		let var = parse_string($lexer.span_str(v.span())).unwrap();
		let i = $3?;
		let j = $6?;
        let mut ind : Vec<Box<ASTNode>> = Vec::default();
        ind.push(Box::new(i));
        ind.push(Box::new(j));
		Ok(ASTNode::VAR{
			name: var,
			indices:ind 
		})
	}
	;
//Variables which could appear in expressions
VariableExpr -> Result<ASTNode,()>:
	Variable
	{
		let node = $1?;
		if validate_ast_node(&node) == Ok(false) {
			return Ok(ASTNode::ErrorNode{
                err : ASTError::TypeError("Variable not defined or out of scope".to_owned()),
            });
		}
		Ok(node)
	}
	| '&' Variable
	{
		let var = $2?;
		let node = ASTNode::UnaryNode{
			op: ASTNodeType::Ref,
			ptr: Box::new(var),
		};
		if validate_ast_node(&node) == Ok(false) {
			return Ok(ASTNode::ErrorNode{
                err : ASTError::TypeError("Expected an Integer or String type for reference operator".to_owned()),
            });
		}
		Ok(node)
	}
	| '*' Variable
	{
		let var = $2?;
		let node = ASTNode::UnaryNode{
			op: ASTNodeType::Ref,
			ptr: Box::new(var),
		};
		if validate_ast_node(&node) == Ok(false) {
			return Ok(ASTNode::ErrorNode{
                err : ASTError::TypeError("Expected an IntegerReference or StringReference type for dereference operator".to_owned()),
            });
		}
		Ok(node)
	}
	;
%%
// Any functions here are in scope for all the grammar actions above.
use crate::parserlib::{*};
use crate::validation::{*};
use crate::codegen::exit_on_err;
use std::collections::LinkedList;
