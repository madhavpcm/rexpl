%start Start 
%avoid_insert "INT" "MAIN" "STR" "SINGLE_COMMENT" "MULTI_COMMENT" "STR_T" "INT_T"
%token 'SINGLE_COMMENT' "BEGIN" "END" "READ" "SYSCALL" "WRITE" "IF" "THEN" "ELSE" "ENDIF" "WHILE" "DO" "ENDWHILE" 'VAR' "BREAK" "BREAKPOINT" "CONTINUE" "MAIN" "DECL" "ENDDECL" "RETURN" ";" "DOT" "ARROW" "=" 

%nonassoc ">" "<" ">=" '<=' "==" "!="
%left '+' '-'
%left '*' '/' '%'
%%

PtrPtr -> Result<ASTExprType,String>: 
	PtrPtr '*' { Ok(ASTExprType::Pointer(Box::new($1?))) }
	| '*' { Ok(ASTExprType::Pointer(Box::new(ASTExprType::Primitive(PrimitiveType::Void)))) }
	;

Type -> Result<ASTExprType,String>: 
	'INT_T' { Ok(ASTExprType::Primitive(PrimitiveType::Int)) } 
	| 'STR_T' { Ok(ASTExprType::Primitive(PrimitiveType::String)) }
	| 'VAR' {
		let v = $1.map_err(|_| "VAR Err".to_string())?; 
		let typename= parse_string($lexer.span_str(v.span())).unwrap();
		let mtt = TYPE_TABLE.lock().unwrap();
		Ok(mtt.tt_get_type(&typename)?)
	}

    ;
ParamType -> Result<ASTExprType,String>: 
	Type { let t = $1?;Ok(t) }
	| Type PtrPtr { let mut ptr = $2?;ptr.set_base_type($1?.get_base_type());Ok(ptr) }
    ;

FType-> Result<ASTExprType,String>: 
	Type
	{
		let t = $1?;
		let mut rt = RET_TYPE.lock().unwrap();
		//TODO verify
		*rt = t.clone();
		std::mem::drop(rt);
		Ok(t)
	}
	| Type PtrPtr
	{
		let mut ptr = $2?;
		let t = $1?;
		ptr.set_base_type(t.get_base_type());
		let mut rt = RET_TYPE.lock().unwrap();
		//TODO verify
		*rt = ptr.clone();
		std::mem::drop(rt);
		Ok(ptr)
	}
    ;
DeclType -> Result<ASTExprType,String>: 
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
	| 'VAR'
	{
		let v = $1.map_err(|_| "VAR Err".to_string())?; 
		let typename= parse_string($lexer.span_str(v.span())).unwrap();
		let mut dt = DECL_TYPE.lock().unwrap();
		let mtt = TYPE_TABLE.lock().unwrap();
		*dt = mtt.tt_get_type(&typename)?;
		Ok(dt.clone())
	}
    ;

//Big Picture
Start -> Result<ASTNode,String>:
	TypeDefBlock ClassDefBlock GDeclBlock FDefBlock MainBlock
	{
		$1?;
		$3?;
		Ok(ASTNode::BinaryNode{
			op : ASTNodeType::Connector,
            exprtype : Some(ASTExprType::Primitive(PrimitiveType::Void)),
			lhs : Box::new(ASTNode::BinaryNode{
				op: ASTNodeType::Connector,
				exprtype: Some(ASTExprType::Primitive(PrimitiveType::Void)),
				lhs: Box::new($2?),
				rhs: Box::new($4?),
			}),
			rhs : Box::new($5?),
		})
	}
	| TypeDefBlock ClassDefBlock GDeclBlock MainBlock
	{
		$1?;
		$3?;
		Ok(ASTNode::BinaryNode{
			op : ASTNodeType::Connector,
            exprtype : Some(ASTExprType::Primitive(PrimitiveType::Void)),
			lhs : Box::new($2?),
			rhs : Box::new($4?),
		})
	}
	;


MainBlock -> Result<ASTNode,String>:
	FType "MAIN" '('  ')' '{' LDeclBlock BeginBlock '}'
	{
		let body_ = $7?;
		let type_ = $1?;
		if type_ != ASTExprType::Primitive(PrimitiveType::Int) {
			return Err("Main should return an integer".to_string());
		}
		let node = ASTNode::MainNode{
			body: Box::new(body_),
		};
		let mut ft = FUNCTION_TABLE.lock().unwrap();
		let mut lst = LOCALSYMBOLTABLE.lock().unwrap();
		ft.insert(
			"main#".to_string(),
			lst.clone()
		);
		lst.clear();
		Ok(node)
	}
	;

BeginBlock -> Result<ASTNode,String>:
	"BEGIN" StmtList "END" { $2 }
	| "BEGIN" "END" { Ok(ASTNode::Void) }
	| { Ok(ASTNode::Void) }
	;

GDeclBlock -> Result<(),String>:
	"DECL" GDeclList "ENDDECL" { $2 }
	| "DECL" "ENDDECL" { Ok(()) }
	| { Ok(()) }
	;

LDeclList -> Result<(),String>:
	LDeclList LDecl { $1?;$2?;Ok(()) }
	| LDecl { $1?;Ok(()) }
	;

GDeclList -> Result<(),String>:
	GDecl GDeclList { $1?;$2?;Ok(()) }
	| GDecl { $1?;Ok(()) }
	;

LDecl ->  Result<(),String>:
	DeclType LLine ';' { $1?;$2?;Ok(()) }
	;

GDecl ->  Result<(),String>:
	DeclType GLine ';' { $1?;$2?;Ok(()) }
	;

GLine -> Result<(),String>:
	GItem ',' GLine { $1?;$3?;Ok(()) }
	| GItem { $1?;Ok(()) }
	;

GParamList -> Result<LinkedList<VarNode>, String>:
	ParamList { $1 }
	| { Ok(LinkedList::new()) }
	;

GItem -> Result<(),String>:
	'VAR' '(' GParamList ')' 
	{
		let returntype = DECL_TYPE.lock().unwrap().clone();
		let v = $1.map_err(|_| "VAR Err".to_string()).unwrap();
		let functionname= parse_string($lexer.span_str(v.span())).unwrap();
		let paramlist = $3?;
		install_func_to_gst(functionname,returntype,&paramlist);
		Ok(())
	}
	| PtrPtr 'VAR' '(' GParamList ')'
	{
		let base= DECL_TYPE.lock().unwrap().clone();
		let mut returntype = $1?;
		returntype.set_base_type(base.get_base_type());
		let v = $2.map_err(|_| "VAR Err".to_string()).unwrap();
		let functionname= parse_string($lexer.span_str(v.span())).unwrap();
		let paramlist = $4?;
		install_func_to_gst(functionname,returntype,&paramlist);
		Ok(())
	}
	| VarItem
	{
		let mut node = $1?;
		let dt = DECL_TYPE.lock().unwrap().clone();
		node.vartype.set_base_type(dt.get_base_type());
		node.install_to_gst();
		Ok(())
	}
	;

LLine -> Result<(),String>:
	VarItem ',' LLine
	{
		let mut node = $1?;
		let dt = DECL_TYPE.lock().unwrap().clone();
		node.vartype.set_base_type(dt.get_base_type());
		node.install_to_lst();
		Ok(())
	}
	| VarItem 
	{
		let mut node =$1.unwrap();
		let dt = DECL_TYPE.lock().unwrap().clone();
		node.vartype.set_base_type(dt.get_base_type());
		node.install_to_lst();
		Ok(())
	}
	;

VarItem -> Result<VarNode,String>: 
	VariableDef { $1 } 
	| PtrPtr VariableDef { let mut node= $2?;node.vartype = $1?;Ok(node) }
    ;

FBlock -> Result<ASTNode,String>:
	FDefBlock { $1 }
	| { Ok(ASTNode::Void) }
	;

FDefBlock -> Result<ASTNode,String>:
	FDefBlock FDef 
	{
		let f1 = $1?;
		let mut lst = LOCALSYMBOLTABLE.lock().unwrap();
		*lst = HashMap::default();
		std::mem::drop(lst);
		let f2 = $2?;
		Ok(ASTNode::BinaryNode{
			op: ASTNodeType::Connector,
            exprtype : Some(ASTExprType::Primitive(PrimitiveType::Void)),
			lhs: Box::new(f1),
			rhs: Box::new(f2),
		})
	}
	| FDef { $1 }
	;

FDef ->Result<ASTNode,String>:
	FType 'VAR' '(' ParamListBlock ')' '{' LDeclBlock BeginBlock '}'
	{
		let v = $2.map_err(|_| "VAR Err".to_string())?; 
		$7?;
		let funcname = parse_string($lexer.span_str(v.span())).unwrap();
		let mut node = ASTNode::FuncDefNode{
			fname: funcname.clone(),
			ret_type: $1?,
			body: Box::new($8?),
			paramlist: $4?, 
		};
		node.validate()?;

		let mut lst = LOCALSYMBOLTABLE.lock().unwrap();
		let mut ft = FUNCTION_TABLE.lock().unwrap();
		let mut lv = LOCALVARID.lock().unwrap();
		let cname = CLASSNAME.lock().unwrap();
		ft.insert(
			funcname + "#" + cname.as_str(),
			lst.clone()
		);
		//reset data structures
		*lst = HashMap::default();
		*lv = 1;
		Ok(node)
	}
	;

LDeclBlock -> Result<(),String>:
	"DECL" LDeclList "ENDDECL" { $2 }
	| "DECL" "ENDDECL" { Ok(()) }
	| { Ok(()) }
	;

ParamListBlock -> Result<LinkedList<VarNode>,String>:
	ParamList 
	{
		let mtt = TYPE_TABLE.lock().unwrap();
		let mut ll: LinkedList<VarNode> = LinkedList::new();
		if CLASSNAME.lock().unwrap().len() > 0 {
			ll = LinkedList::from( VarNode {
				varname: "self".to_owned(),
				vartype: ASTExprType::Pointer(Box::new(mtt.tt_get_type(&*CLASSNAME.lock().unwrap())?)),
				varindices: vec![],
			});
		}
		ll.append(&mut $1?);
		let mut lst = LOCALSYMBOLTABLE.lock().unwrap();
		*lst = HashMap::default();
		std::mem::drop(mtt);
		std::mem::drop(lst);
		__lst_install_params(&mut ll)?;
		Ok(ll)
	}
	|
	{
		let mtt = TYPE_TABLE.lock().unwrap();
		let mut ll: LinkedList<VarNode> = LinkedList::new();
		if CLASSNAME.lock().unwrap().len() > 0 {
			ll = LinkedList::from(VarNode {
				varname: "self".to_owned(),
				vartype: ASTExprType::Pointer(Box::new(mtt.tt_get_type(&*CLASSNAME.lock().unwrap())?)),
				varindices: vec![],
			});
		}
		let mut lst = LOCALSYMBOLTABLE.lock().unwrap();
		*lst = HashMap::default();
		std::mem::drop(mtt);
		std::mem::drop(lst);
		__lst_install_params(&mut ll)?;
		Ok(ll)
	}
	;

ParamList -> Result<LinkedList<VarNode>,String>:
	ParamList ',' Param { let mut paramlist = $1?;paramlist.append(&mut $3?);Ok(paramlist) }
	| Param { $1 }
	;

Param -> Result<LinkedList<VarNode>,String>:
	ParamType VariableDef 
    {
		let mut var = $2?;let vtype = $1?;
		if var.varindices.len() != 0 {
			return Err("Arrays cannot be used as a function parameter. Use a pointer instead.".to_owned());
		}
		var.vartype= vtype;
		Ok(LinkedList::from(var))
    }
	;

ArgList -> Result<LinkedList<ASTNode>,String>:
	ArgList ',' Expr { let expr = $3?;let mut arglist = $1?;arglist.push_back(expr); Ok(arglist) }
	| Expr { Ok(LinkedList::from($1?)) }
	;

VariableDef -> Result<VarNode,String>:
	'VAR' 
	{
		let v = $1.map_err(|_| "VAR Err".to_string())?;
		let var_ = parse_string($lexer.span_str(v.span())).unwrap();
		Ok(VarNode{
			varname: var_,
			vartype: ASTExprType::Primitive(PrimitiveType::Void),
			varindices: vec![],
		})
	}
	| 'VAR' "[" "INT" "]"
	{
		let v = $1.map_err(|_| "VAR[] Err".to_string())?;
		let var_ = parse_string($lexer.span_str(v.span())).unwrap();
		let v = $3.map_err(|_| "[INT] Err".to_string())?;
        let i= parse_usize($lexer.span_str(v.span())).unwrap();
		Ok(VarNode{
			varname: var_,
			vartype: ASTExprType::Primitive(PrimitiveType::Void),
			varindices: vec![i],
		})
	}
	| 'VAR' "[" "INT" "]" "[" "INT" "]"
	{
		let v = $1.map_err(|_| "VAR[][] Err".to_string())?;
		let var_ = parse_string($lexer.span_str(v.span())).unwrap();
		let v = $3.map_err(|_| "VAR[INT] Err".to_string())?;
        let i= parse_usize($lexer.span_str(v.span())).unwrap();
		let v = $3.map_err(|_| "VAR[][INT] Err".to_string())?;
        let j= parse_usize($lexer.span_str(v.span())).unwrap();
		Ok(VarNode{
			varname: var_,
			vartype: ASTExprType::Primitive(PrimitiveType::Void),
			varindices: vec![i,j],
		})
	}
	;
//StateMents
StmtList -> Result<ASTNode, String>:
	StmtList Stmt 
	{
		Ok(ASTNode::BinaryNode{
			op : ASTNodeType::Connector,
            exprtype : Some(ASTExprType::Primitive(PrimitiveType::Void)),
			lhs : Box::new($1?),
			rhs : Box::new($2?),
		})
	}
	| Stmt { $1 }
	;
Stmt -> Result<ASTNode,String>:
	InputStmt { $1 }
	| OutputStmt { $1 }
	| AssgStmt { $1 }
	| WhileStmt { $1 }
    | IfStmt { $1 }
	| "BREAKPOINT" ';' { Ok(ASTNode::BreakpointNode) }
	| "BREAK" ';' { Ok(ASTNode::BreakNode) }
	| "CONTINUE" ';' { Ok(ASTNode::ContinueNode) }
	| "RETURN" Expr ';'
	{
		let mut node = ASTNode::ReturnNode{
			expr: Box::new($2?)
		};
		node.validate()?;
		Ok(node)
	}
	| "INIT" '(' ')' ';'
	{
		let mut node = ASTNode::UnaryNode{
			op: ASTNodeType::Initialize,
			exprtype: Some(ASTExprType::Primitive(PrimitiveType::Void)),
			ptr: Box::new(ASTNode::Void),
			depth: None,
		};
		node.validate()?;
		Ok(node)
	}
	| 'SYSCALL' '(' ArgList ')' ';'
	{
		let mut node = ASTNode::StdFuncCallNode{
			func: STDLibFunction::Syscall,
			arglist: Box::new($3?),
		};
		node.validate()?;
		Ok(node)
	}
	| 'SETADDR' '(' ArgList ')' ';'
	{
		let mut node = ASTNode::StdFuncCallNode{
			func: STDLibFunction::Setaddr,
			arglist: Box::new($3?),
		};
		node.validate()?;
		Ok(node)
	}
	;
WhileStmt -> Result<ASTNode,String>:
    "WHILE" '(' Expr ')' "DO" StmtList "ENDWHILE" ';'
    {
        let expr = $3?;
        Ok(ASTNode::WhileNode{
            expr: Box::new(expr),
            xdo: Box::new($6?),
        })
    }
    ;
IfStmt -> Result<ASTNode,String>:
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
OutputStmt -> Result<ASTNode,String>:
	"WRITE" '(' Expr ')' ';' 
	{
		Ok(ASTNode::UnaryNode{
			op : ASTNodeType::Write,
			exprtype : Some(ASTExprType::Primitive(PrimitiveType::Void)),
			ptr : Box::new($3?),
			depth : None,
		})
	}
	;
AssgStmt -> Result<ASTNode,String>:
	VariableExpr '=' Expr ';'
	{
		let lhs = $1?;
		let rhs = $3?;
		let mut node = ASTNode::BinaryNode{
			op : ASTNodeType::Equals,
            exprtype : Some(ASTExprType::Primitive(PrimitiveType::Void)),
			lhs : Box::new(lhs),
			rhs : Box::new(rhs),
		};
		node.validate()?;
		Ok(node)
	}
	| VariableExpr '=' 'ALLOC' '(' ')' ';'
	{
		let mut node = ASTNode::UnaryNode{
			op: ASTNodeType::Alloc,
			exprtype: Some(ASTExprType::Primitive(PrimitiveType::Void)),
			ptr : Box::new($1?),
			depth: None
		};
		node.validate()?;
		Ok(node)
	}
	| VariableExpr '=' 'NEW' '(' 'VAR' ')' ';'
	{
		let v = $5.map_err(|_| "VAR Err".to_string())?;  

		let mut node_ = ASTNode::StdFuncCallNode{
			func: STDLibFunction::New,
			arglist: Box::new(LinkedList::from(ASTNode::VAR{
				name: $lexer.span_str(v.span()).to_string(), 
				array_access: Vec::default(),
				dot_field_access: Box::new(ASTNode::Void),
				arrow_field_access: Box::new(ASTNode::Void),
			})),
		};
		node_.validate()?;
		let mut node = ASTNode::BinaryNode{
			op : ASTNodeType::Equals,
            exprtype : Some(ASTExprType::Primitive(PrimitiveType::Void)),
			lhs : Box::new($1?),
			rhs : Box::new(node_),
		};
		node.validate()?;
		Ok(node)
	}
	;
InputStmt -> Result<ASTNode, String> :
	"READ" '(' Variable ')' ';'
	{
		Ok(ASTNode::UnaryNode{
			op : ASTNodeType::Read,
			exprtype: Some(ASTExprType::Primitive(PrimitiveType::Void)),
			ptr : Box::new($3?),
			depth: None
		})
	}
	| "READ" '(' PtrPtr Variable ')' ';'
	{
		let var = $4?;
		Ok(ASTNode::UnaryNode{
			op : ASTNodeType::Read,
			exprtype: Some(ASTExprType::Primitive(PrimitiveType::Void)),
			ptr : Box::new(ASTNode::UnaryNode{
				op: ASTNodeType::Deref,
				exprtype: None,
				ptr: Box::new(var),
				depth: Some($3?.depth()),
			}),
			depth: None,
		})
	}
	;
Expr -> Result<ASTNode,String>:
	Expr '<' Expr 
	{
        let lhs = $1?;
        let rhs = $3?;
		
		let mut node = ASTNode::BinaryNode{
			op : ASTNodeType::Lt,
			exprtype : None,
			lhs : Box::new(lhs),
			rhs : Box::new(rhs),
		};
		node.validate()?;
		Ok(node)
	}
	| Expr '>' Expr 
	{
        let lhs = $1?;
        let rhs = $3?;
		let mut node = ASTNode::BinaryNode{
			op : ASTNodeType::Gt,
			exprtype : None,
			lhs : Box::new(lhs),
			rhs : Box::new(rhs),
		};
		node.validate()?;
		Ok(node)
	}
	| Expr '<=' Expr 
	{
        let lhs = $1?;
        let rhs = $3?;
		let mut node = ASTNode::BinaryNode{
			op : ASTNodeType::Lte,
			exprtype : None,
			lhs : Box::new(lhs),
			rhs : Box::new(rhs),
		};
		node.validate()?;
		Ok(node)
	}
	| Expr '>=' Expr 
	{
        let lhs = $1?;
        let rhs = $3?;
		let mut node = ASTNode::BinaryNode{
			op : ASTNodeType::Gte,
			exprtype : None,
			lhs : Box::new(lhs),
			rhs : Box::new(rhs),
		};
		node.validate()?;
		Ok(node)
	}
	| Expr '!=' Expr 
	{
        let lhs = $1?;
        let rhs = $3?;
		let mut node = ASTNode::BinaryNode{
			op : ASTNodeType::Ne,
			exprtype : None,
			lhs : Box::new(lhs),
			rhs : Box::new(rhs),
		};
		node.validate()?;
		Ok(node)
	}
	| Expr '==' Expr 
	{
        let lhs = $1?;
        let rhs = $3?;
		let mut node = ASTNode::BinaryNode{
			op : ASTNodeType::Ee,
			exprtype : None,
			lhs : Box::new(lhs),
			rhs : Box::new(rhs),
		};
		node.validate()?;
		Ok(node)
	}
	| Expr '+' Expr
	{
        let lhs = $1?;
        let rhs = $3?;
		let mut node = ASTNode::BinaryNode{
			op : ASTNodeType::Plus,
			exprtype : None,
			lhs : Box::new(lhs),
			rhs : Box::new(rhs),
		};
		node.validate()?;
		Ok(node)
	}
	| Expr '-' Expr
	{
        let lhs = $1?;
        let rhs = $3?;
		let mut node = ASTNode::BinaryNode{
			op : ASTNodeType::Minus,
			exprtype : None,
			lhs : Box::new(lhs),
			rhs : Box::new(rhs),
		};
		node.validate()?;
		Ok(node)
	}
	| Expr '*' Expr
	{
        let lhs = $1?;
        let rhs = $3?;
		let mut node = ASTNode::BinaryNode{
			op : ASTNodeType::Star,
			exprtype : None,
			lhs : Box::new(lhs),
			rhs : Box::new(rhs),
		};
		node.validate()?;
		Ok(node)
	}
	| Expr '/' Expr
	{
        let lhs = $1?;
        let rhs = $3?;
		let mut node = ASTNode::BinaryNode{
			op : ASTNodeType::Slash,
			exprtype : None,
			lhs : Box::new(lhs),
			rhs : Box::new(rhs),
		};
		node.validate()?;
		Ok(node)
	}
	| Expr '%' Expr
	{
        let lhs = $1?;
        let rhs = $3?;
		let mut node = ASTNode::BinaryNode{
			op : ASTNodeType::Mod,
			exprtype : None,
			lhs : Box::new(lhs),
			rhs : Box::new(rhs),
		};
		node.validate()?;
		Ok(node)
	}
	| "INT"
	{
		let v = $1.map_err(|_| "INT Err".to_string())?;  
        let num  = parse_int($lexer.span_str(v.span())).unwrap();
        Ok(ASTNode::INT(num))
	}
	| "STR"
	{
		let v = $1.map_err(|_| "STR Err".to_string())?;  
		let str = parse_string($lexer.span_str(v.span())).unwrap();
		Ok(ASTNode::STR(str))
	}
    | '(' Expr ')' { $2 } 
	| VariableExpr { $1 } 
	| 'NULL' { Ok(ASTNode::Null) }
	| StdFuncCall { $1 }
	;

FuncCall -> Result<ASTNode, String>:
	'VAR' '(' ')'
	{
		let v = $1.map_err(|_| "VAR()".to_string())?;
		let functionname= parse_string($lexer.span_str(v.span())).unwrap();
		let mut node = ASTNode::FuncCallNode{
			fname: functionname, 
			arglist: Box::new(LinkedList::new()),
		};
		node.validate()?;
		Ok(node)
	}
	| 'VAR' '(' ArgList ')'
	{
		let v = $1.map_err(|_| "VAR( ArgList )".to_string())?;
		let functionname= parse_string($lexer.span_str(v.span())).unwrap();
		let mut node = ASTNode::FuncCallNode{
			fname: functionname, 
			arglist: Box::new($3?),
		};
		node.validate()?;
		Ok(node)
	}
    ; 

StdFuncCall -> Result<ASTNode,String>:
	'FREE' '(' VariableExpr ')'
	{
		let mut node = ASTNode::UnaryNode{
			op: ASTNodeType::Free,
			exprtype: Some(ASTExprType::Primitive(PrimitiveType::Int)),
			ptr: Box::new($3?),
			depth: None,
		};
		node.validate()?;
		Ok(node)
	}
	| 'GETADDR' '(' ArgList ')'
	{
		let mut node = ASTNode::StdFuncCallNode{
			func: STDLibFunction::Getaddr,
			arglist: Box::new($3?),
		};
		node.validate()?;
		Ok(node)
	}
	;
//Variables around the code
Variable -> Result<ASTNode,String>:
	'VAR'
	{
		let v = $1.map_err(|_| "VAR Err".to_string())?;
		let var = parse_string($lexer.span_str(v.span())).unwrap();
		Ok(ASTNode::VAR{
			name: var,
			array_access: Vec::default(),
			dot_field_access: Box::new(ASTNode::Void),
			arrow_field_access: Box::new(ASTNode::Void),
		})
	}
	| 'VAR' VariableArray
	{
		let v = $1.map_err(|_| "VAR Err".to_string())?;
		let var = parse_string($lexer.span_str(v.span())).unwrap();
		Ok(ASTNode::VAR{
			name: var,
			array_access: $2?,
			dot_field_access: Box::new(ASTNode::Void),
			arrow_field_access: Box::new(ASTNode::Void),
		})
	}
	| 'VAR' 'DOT' Variable
	{
		let v = $1.map_err(|_| "VAR Err".to_string())?;
		let var = parse_string($lexer.span_str(v.span())).unwrap();
		Ok(ASTNode::VAR{
			name: var,
			array_access: vec![],
			dot_field_access: Box::new($3?),
			arrow_field_access: Box::new(ASTNode::Void),
		})
	}
	| 'VAR' 'ARROW' Variable
	{
		let v = $1.map_err(|_| "VAR Err".to_string())?;
		let var = parse_string($lexer.span_str(v.span())).unwrap();
		Ok(ASTNode::VAR{
			name: var,
			array_access: vec![],
			dot_field_access: Box::new(ASTNode::Void),
			arrow_field_access: Box::new($3?),
		})
	}
	| FuncCall
	{
		$1
	}
	;

VariableArray -> Result<Vec<Box<ASTNode>>,String>:
	'[' Expr ']' VariableArray
	{
		let mut i = $2?;
		i.validate()?;
		if i.getexprtype() != Some(ASTExprType::Primitive(PrimitiveType::Int)) {
			return Err(
				"Invalid expression type used to index".to_owned()
					+ "[x]",
			);
		}
		let mut v: Vec<Box<ASTNode>> = vec![Box::new(i)];v.append(&mut $4?);
		Ok(v)
	}
	| '[' Expr ']'
	{
		let mut i = $2?;
		i.validate()?;
		if i.getexprtype() != Some(ASTExprType::Primitive(PrimitiveType::Int)) {
			return Err(
				"Invalid expression type used to index".to_owned()
					+ "[x]",
			);
		}
		Ok(vec![Box::new(i)])
	}
	;

//Variables which could appear in expressions
VariableExpr -> Result<ASTNode,String>:
	Variable
	{
		let mut node = $1?;
		node.validate()?;
		Ok(node)
	}
	| '&' Variable
	{
		let mut node = ASTNode::UnaryNode{
			op: ASTNodeType::Ref,
			exprtype: None,
			ptr: Box::new($2?),
			depth: None,
		};
		node.validate()?;
		Ok(node)
	}
	| PtrPtr Variable 
	{
		let mut node = ASTNode::UnaryNode{
			op: ASTNodeType::Deref,
			exprtype: None,
			ptr: Box::new($2?),
			depth: Some($1?.depth()),
		};
		node.validate()?;
		Ok(node)
	}
	;

//UserDefined Types
TypeDefBlock -> Result<(),String>:
	"TYPE" TypeDefList "ENDTYPE" { $2?;Ok(()) }
	| { Ok(()) }
	;

TypeDefList -> Result<(),String>:
	TypeDef TypeDefList { $1?;$2?;Ok(()) }
	| TypeDef { $1?;Ok(()) }
	;

TypeDef -> Result<(),String>:
	'VAR' '{' FieldDeclList '}' ';'
	{
		let v = $1.map_err(|_| "VAR Err".to_string())?;
		let typename = parse_string($lexer.span_str(v.span())).unwrap();
		let fields = $3?;
		let mut mtt = TYPE_TABLE.lock().unwrap();
		mtt.tinstall_struct(typename, fields)?;
		Ok(())
	}
	;

FieldDeclList -> Result<LinkedList<Field>,String>:
	FieldDecl FieldDeclList
	{
		let mut f1 = LinkedList::from($1?);
		let mut f2 = $2?;
		f1.append(&mut f2);
		Ok(f1)
	}
	| FieldDecl { let field = $1?; Ok(LinkedList::from(field)) }
	;

FieldDecl -> Result<Field,String>:
	CFType 'VAR' ';'
	{
		let v = $2.map_err(|_| "VAR Err".to_string())?; 
		Ok(Field{
			name:parse_string($lexer.span_str(v.span())).unwrap(),
			field_type: $1?,
			array_access: vec![],
		})
	}
	;
FieldType -> Result<FieldType,String>: 
	'INT_T'	{ Ok(FieldType::Primitive(PrimitiveType::Int)) } 
	| 'STR_T' { Ok(FieldType::Primitive(PrimitiveType::String)) }
	| 'VAR'
	{
		let v = $1.map_err(|_| "VAR Err".to_string())?; 
		let typename= parse_string($lexer.span_str(v.span())).unwrap();
		Ok(FieldType::Struct(typename))
	}
    ;

FieldPtr-> Result<FieldType,String>: 
	FieldPtr '*' { 	Ok(FieldType::Pointer(Box::new($1?))) }
	| '*' { Ok(FieldType::Pointer(Box::new(FieldType::Primitive(PrimitiveType::Void)))) }
	;

ClassDefBlock -> Result<ASTNode,String>:
	'CLASS' ClassDefList 'ENDCLASS' { $2 }
	| 'CLASS' 'ENDCLASS' { Ok(ASTNode::Void) }
	| { Ok(ASTNode::Void) }
	;

ClassDefList -> Result<ASTNode,String>:
	ClassDefList ClassDef 
	{ 
		Ok(ASTNode::BinaryNode {
			op : ASTNodeType::Connector,
            exprtype : Some(ASTExprType::Primitive(PrimitiveType::Void)),
			lhs : Box::new($1?),
			rhs : Box::new($2?),
		} )
	}
	| ClassDef { $1 }
	;

ClassDef -> Result<ASTNode,String>:
	ClassName '{'   ClassDeclBlock ClassMethodDefList '}' ';' 
	{ 
		let cname = $1?;$3?;let methods = $4?;
		//reset class after def
		let mut cn = CLASSNAME.lock().unwrap();
		*cn = "".to_owned();
		Ok(ASTNode::ClassNode{
			cname: cname,
			methods: Box::new(methods),
		}) 
	}
	;

ClassName -> Result<String,String>:
	'VAR' 
	{
		let mut cn = CLASSNAME.lock().unwrap();
		let v = $1.map_err(|_| "VARErr".to_owned())?;
		*cn = $lexer.span_str(v.span()).to_owned();
		Ok(cn.clone())
	}
	| 'VAR' 'EXTENDS' 'VAR'
	{
		let mut cn = CLASSNAME.lock().unwrap();
		let v = $3.map_err(|_| "VARErr".to_owned())?;
		let mut parent = PCLASSNAME.lock().unwrap();
		*parent = $lexer.span_str(v.span()).to_owned();
		let v = $1.map_err(|_| "VARErr".to_owned())?;
		*cn = $lexer.span_str(v.span()).to_owned();
		Ok(cn.clone())
	}
	;

ClassDeclBlock -> Result<(),String>:
	'DECL' ClassFieldBlock 'DIV' ClassMethodDeclList 'ENDDECL'
	{
		$2?;
		let mut methods = $4?;
		let mut mtt = TYPE_TABLE.lock().unwrap();
		mtt.tinstall_class_methods(&mut methods)?;
		Ok(())
	}
	;


ClassFieldBlock -> Result<(), String>:
	ClassFieldList { let mut fields = $1?;let mut mtt = TYPE_TABLE.lock().unwrap();mtt.tinstall_class_fields(&mut fields)?;Ok(()) }
    | { let mut mtt = TYPE_TABLE.lock().unwrap();mtt.tinstall_class_fields(&mut LinkedList::new())?;Ok(()) }
	;

ClassFieldList -> Result<LinkedList<CSymbol>,String>:
	ClassFieldList ClassField { let mut f1 = $1?;  f1.append(&mut $2?); Ok(f1) }
    | ClassField { $1 }
	;

ClassField -> Result<LinkedList<CSymbol>, String>:
	CFType 'VAR' ';' {
		let v = $2.map_err(|_| "VAR Err".to_string())?; 
		Ok(LinkedList::from( CSymbol::Var{
			name: $lexer.span_str(v.span()).to_owned(),
			vartype: $1?,
			varid: 0,
			varindices: vec![],
		} ))
	}
	;

ClassMethodBlock -> Result<LinkedList<CSymbol>,String>:
	ClassMethodDeclList { $1 }
	| { Ok(LinkedList::new()) }
	;

ClassMethodDeclList-> Result<LinkedList<CSymbol>,String>:
	ClassMethodDeclList ClassMethodDecl { let mut ll = $1?;ll.append(&mut $2?);Ok(ll) }
	| ClassMethodDecl { $1 }
	;

ClassMethodDecl -> Result<LinkedList<CSymbol>,String>:
	ParamType 'VAR' '(' GParamList ')' ';' {
		let v = $2.map_err(|_| "VAR Err".to_string())?; 
		Ok(LinkedList::from(CSymbol::Func{
			name: $lexer.span_str(v.span()).to_owned(), 
			ret_type: $1?,
			paramlist: $4?,
			flabel:0,
			fid:0,
		}))
	}
	;

ClassMethodDefList -> Result<LinkedList<ASTNode>,String>:
	ClassMethodDefList FDef { let mut l1 = LinkedList::from($1?);l1.append(&mut LinkedList::from($2?));Ok(l1) }
	| FDef { Ok(LinkedList::from($1?)) }
	;

CFType-> Result<FieldType,String>: 
	FieldType
	{
		let t = $1?;
		let mut rt = CLASS_RET_TYPE.lock().unwrap();
		//TODO verify
		*rt = t.clone();
		std::mem::drop(rt);
		Ok(t)
	}
	| FieldType FieldPtr
	{
		let mut ptr = $2?;
		let t = $1?;
		ptr.set_base_type(t.get_base_type());
		let mut rt = CLASS_RET_TYPE.lock().unwrap();
		//TODO verify
		*rt = ptr.clone();
		std::mem::drop(rt);
		Ok(ptr)
	}
    ;
%%
// Any functions here are in scope for all the grammar actions above.
use crate::parserlib::{*};
use std::collections::{LinkedList,HashMap};
