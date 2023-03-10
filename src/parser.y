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

PtrPtr -> Result<ASTExprType,String>: 
	PtrPtr '*'
	{
		Ok(ASTExprType::Pointer(Box::new($1?)))
	}
	| '*'
	{
		Ok(ASTExprType::Pointer(Box::new(ASTExprType::Primitive(PrimitiveType::Void))))
	}
	;
Type -> Result<ASTExprType,String>: 
	'INT_T'
	{
		Ok(ASTExprType::Primitive(PrimitiveType::Int))
	} 
	| 'STR_T'
	{
		Ok(ASTExprType::Primitive(PrimitiveType::String))
	}
	| 'VAR'
	{
		let v = $1.map_err(|_| "VAR Err".to_string())?; 
		let typename= parse_string($lexer.span_str(v.span())).unwrap();
		Ok(ASTExprType::Struct(ASTStructType{
			name: typename,
			size: 0,
			fields: LinkedList::default(),
		}))
	}
    ;
ParamType -> Result<ASTExprType,String>: 
	Type
	{
		let t = $1?;
		//TODO verify
		Ok(t)
	}
	| Type PtrPtr
	{
		let mut ptr = $2?;
		let t = $1?;
		//TODO verify
		ptr.set_base_type(t.get_base_type());
		Ok(ptr)
	}
    ;
FType-> Result<ASTExprType,String>: 
	Type
	{
		let t = $1?;
		let mut rt = RET_TYPE.lock().unwrap();
		//TODO verify
		*rt = t.clone();
		Ok(t)
	}
	| Type PtrPtr
	{
		let mut ptr = $2?;
		let t = $1?;
		let mut rt = RET_TYPE.lock().unwrap();
		//TODO verify
		ptr.set_base_type(t.get_base_type());
		*rt = ptr.clone();
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
    ;

//Big Picture
Start -> Result<ASTNode,String>:
	TypeDefBlock GDeclBlock FDefBlock MainBlock
	{
		$1?;
		Ok(ASTNode::BinaryNode{
			op : ASTNodeType::Connector,
            exprtype : Some(ASTExprType::Primitive(PrimitiveType::Null)),
			lhs : Box::new($3?),
			rhs : Box::new($4?),
		})
	}
	| TypeDefBlock GDeclBlock MainBlock
	{
		$1?;
		Ok(ASTNode::BinaryNode{
			op : ASTNodeType::Connector,
            exprtype : Some(ASTExprType::Primitive(PrimitiveType::Null)),
			lhs : Box::new($3?),
			rhs : Box::new(ASTNode::Null),
		})
	}
	;
//	| GDeclBlock MainBlock 
//	{
//		Ok(ASTNode::BinaryNode{
//			op : ASTNodeType::Connector,
//            exprtype : Some(ASTExprType::Primitive(PrimitiveType::Null)),
//			lhs : Box::new($2?),
//			rhs : Box::new(ASTNode::Null),
//		})
//	}
//    | MainBlock 
//    {
//        Ok(ASTNode::BinaryNode{
//            op: ASTNodeType::Connector,
//            exprtype : Some(ASTExprType::Primitive(PrimitiveType::Null)),
//            lhs: Box::new(ASTNode::Null),
//            rhs: Box::new($1?),
//        })
//    }
//	;
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
			"main".to_string(),
			lst.clone()
		);
		lst.clear();
		Ok(node)
	}
	;
BeginBlock -> Result<ASTNode,String>:
	"BEGIN" StmtList "END" 
	{
		$2
	}
	| "BEGIN" "END" 
	{
		Ok(ASTNode::Null)
	}
	|
	{
		Ok(ASTNode::Null)
	}
	;
GDeclBlock -> ():
	"DECL" GDeclList "ENDDECL" 
	{
	}
	| "DECL" "ENDDECL" 
	{
	}
	|
	{

	}
	;
LDeclList -> ():
	LDeclList LDecl 
	{
	}
	| LDecl
	{
	}
	;
GDeclList -> ():
	GDecl GDeclList
	{
	}
	| GDecl
	{
	}
	;
LDecl ->  ():
	DeclType LLine ';'
	{
	}
	;
GDecl ->  ():
	DeclType GLine ';'
	{
	}
	;
GLine -> ():
	GItem ',' GLine
	{
	}
	| GItem
	{
	}
	;
GItem -> ():
	"VAR" '(' ParamList ')' 
	{
		let returntype = DECL_TYPE.lock().unwrap().clone();
		let v = $1.map_err(|_| ()).unwrap();
		let functionname= parse_string($lexer.span_str(v.span())).unwrap();
		let paramlist = $3.unwrap();
		install_func_to_gst(functionname,returntype,&paramlist);
	}
	| PtrPtr "VAR" '(' ParamList ')'
	{
		let base= DECL_TYPE.lock().unwrap().clone();
		let mut returntype = $1.unwrap();
		returntype.set_base_type(base.get_base_type());
		let v = $2.map_err(|_| ()).unwrap();
		let functionname= parse_string($lexer.span_str(v.span())).unwrap();
		let paramlist = $4.unwrap();
		install_func_to_gst(functionname,returntype,&paramlist);
	}
	| VarItem
	{
		let mut node = $1.unwrap();
		let dt = DECL_TYPE.lock().unwrap().clone();
		node.vartype.set_base_type(dt.get_base_type());
		node.install_to_gst();
	}
	;

LLine -> ():
	VarItem ',' LLine
	{
		let mut node = $1.unwrap();
		let dt = DECL_TYPE.lock().unwrap().clone();
		node.vartype.set_base_type(dt.get_base_type());
		node.install_to_lst();
	}
	| VarItem 
	{
		let mut node =$1.unwrap();
		let dt = DECL_TYPE.lock().unwrap().clone();
		node.vartype.set_base_type(dt.get_base_type());
		node.install_to_lst();
	}
	;
VarItem -> Result<VarNode,String>: 
	VariableDef 
	{
		$1
	} 
	| PtrPtr VariableDef
	{
		let mut node= $2?;
		node.vartype = $1?;
		Ok(node)
	}
    ;
FBlock -> Result<ASTNode,String>:
	FDefBlock
	{
		$1
	}
	|
	{
		Ok(ASTNode::Null)
	}
	;
FDefBlock -> Result<ASTNode,String>:
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
FDef ->Result<ASTNode,String>:
	FType "VAR" '(' ParamListBlock ')' '{' LDeclBlock BeginBlock '}'
	{
		let v = $2.map_err(|_| "VAR Err".to_string())?; 
		let funcname = parse_string($lexer.span_str(v.span())).unwrap();
		let paramlist_ = $4?;
		let mut node = ASTNode::FuncDefNode{
			fname: funcname.clone(),
			ret_type: $1?,
			body: Box::new($8?),
			paramlist: paramlist_.clone()
		};
		node.validate()?;

		let mut lst = LOCALSYMBOLTABLE.lock().unwrap();
		let mut ft = FUNCTION_TABLE.lock().unwrap();
		let mut lv = LOCALVARID.lock().unwrap();
		//save localsymbol table for accessing in backend
		ft.insert(
			funcname,
			lst.clone()
		);
		//reset data structures
		lst.clear();
		*lv = 1;
		std::mem::drop(lv);
		std::mem::drop(lst);
		std::mem::drop(ft);
		Ok(node)
	}
	;

LDeclBlock -> ():
	"DECL" LDeclList "ENDDECL"
	{
	}
	| "DECL" "ENDDECL"
	{
	}
	|
	{
	}
	;

ParamListBlock -> Result<LinkedList<VarNode>,String>:
	ParamList 
	{
		let mut node = $1?;
		__lst_install_params(&mut node);
		Ok(node)
	}
	;

ParamList -> Result<LinkedList<VarNode>,String>:
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

Param -> Result<LinkedList<VarNode>,String>:
	ParamType VariableDef 
    {
		let mut var = $2?;
        let vtype = $1?;
		if var.varindices.len() != 0 {
			exit_on_err("Arrays cannot be used as a function parameter. Use a pointer instead.".to_owned());
		}
		var.vartype= vtype;
		Ok(LinkedList::from(var))
    }
	;
ArgList -> Result<LinkedList<ASTNode>,String>:
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
VariableDef -> Result<VarNode,String>:
	"VAR" 
	{
		let v = $1.map_err(|_| "VAR Err".to_string())?;
		let var_ = parse_string($lexer.span_str(v.span())).unwrap();
		Ok(VarNode{
			varname: var_,
			vartype: ASTExprType::Primitive(PrimitiveType::Void),
			varindices: vec![],
		})
	}
	| "VAR" "[" "INT" "]"
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
	| "VAR" "[" "INT" "]" "[" "INT" "]"
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
Stmt -> Result<ASTNode,String>:
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
		let mut node = ASTNode::ReturnNode{
			expr: Box::new($2?)
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
			exprtype : Some(ASTExprType::Primitive(PrimitiveType::Null)),
			ptr : Box::new($3?),
			depth : None,
		})
	}
	;
AssgStmt -> Result<ASTNode,String>:
	Variable '=' Expr ';'
	{
		let lhs = $1?;
		let rhs = $3?;
		let mut node = ASTNode::BinaryNode{
			op : ASTNodeType::Equals,
            exprtype : Some(ASTExprType::Primitive(PrimitiveType::Null)),
			lhs : Box::new(lhs),
			rhs : Box::new(rhs),
		};
		node.validate()?;
		Ok(node)
	}
	| PtrPtr Variable '=' Expr ';'
	{
	//TODO
		let lhs = $2?;
		let rhs = $4?;
		let mut node = ASTNode::BinaryNode{
			op : ASTNodeType::Equals,
            exprtype : Some(ASTExprType::Primitive(PrimitiveType::Null)),
			lhs : Box::new(ASTNode::UnaryNode{
				op: ASTNodeType::Deref,
				exprtype: None,
				ptr: Box::new(lhs),
				depth: Some($1?.depth()),
			}),
			rhs : Box::new(rhs),
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
			exprtype: Some(ASTExprType::Primitive(PrimitiveType::Null)),
			ptr : Box::new($3?),
			depth: None
		})
	}
	| "READ" '(' PtrPtr Variable ')' ';'
	{
		let var = $4?;
		Ok(ASTNode::UnaryNode{
			op : ASTNodeType::Read,
			exprtype: Some(ASTExprType::Primitive(PrimitiveType::Null)),
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
    | '(' Expr ')'
    {
        $2
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
	| VariableExpr
	{
		$1
	}
	| "VAR" '(' ')'
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
	| "VAR" '(' ArgList ')'
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

//Variables around the code
Variable -> Result<ASTNode,String>:
	"VAR"
	{
		let v = $1.map_err(|_| "VAR Err".to_string())?;
		let var = parse_string($lexer.span_str(v.span())).unwrap();
		Ok(ASTNode::VAR{
			name: var,
			indices: Vec::default(),
		})
	}
	| "VAR" "[" Expr "]"
	{
		let v = $1.map_err(|_| "VAR[] Err".to_string())?;
		let var = parse_string($lexer.span_str(v.span())).unwrap();
		let mut expr = $3?;
		expr.validate()?;
		if expr.getexprtype() != Some(ASTExprType::Primitive(PrimitiveType::Int)) {
			exit_on_err(
				"Invalid expression type used to index".to_owned()
					+ var.as_str() 
					+ "[x]",
			);
		}
        let mut ind : Vec<Box<ASTNode>> = Vec::default();
        ind.push(Box::new(expr));
		Ok(ASTNode::VAR{
			name: var,
			indices: ind,
		})
	}
	| "VAR" "[" Expr "]" "[" Expr "]"
	{
		let v = $1.map_err(|_| "VAR[][] Err".to_string())?;
		let var = parse_string($lexer.span_str(v.span())).unwrap();
		let mut i = $3?;
		let mut j = $6?;
        let mut ind : Vec<Box<ASTNode>> = Vec::default();
		i.validate()?;
		if i.getexprtype() != Some(ASTExprType::Primitive(PrimitiveType::Int)) {
			exit_on_err(
				"Invalid expression type used to index".to_owned()
					+ var.as_str() 
					+ "[x]",
			);
		}
		j.validate()?;
		if j.getexprtype() != Some(ASTExprType::Primitive(PrimitiveType::Int)) {
			exit_on_err(
				"Invalid expression type used to index".to_owned()
					+ var.as_str() 
					+ "[][x]",
			);
		}
        ind.push(Box::new(i));
        ind.push(Box::new(j));
		Ok(ASTNode::VAR{
			name: var,
			indices:ind 
		})
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
	"TYPE" TypeDefList "ENDTYPE"
	{
		$2?;
		Ok(())
	}
	|
	{
		Ok(())
	}
	;


TypeDefList -> Result<(),String>:
	TypeDef TypeDefList
	{
		$1?;
		$2?;
		Ok(())	
	}
	| TypeDef
	{
		$1?;
		Ok(())
	}
	;

TypeDef -> Result<(),String>:
	"VAR" '{' FieldDeclList '}' ';'
	{
		let v = $1.map_err(|_| "VAR Err".to_string())?;
		let typename = parse_string($lexer.span_str(v.span())).unwrap();
		let fields = $3?;
		let mut mtt = MUTEX_TYPE_TABLE.lock().unwrap();
		mtt.tinstall(typename, fields)?;
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
	| FieldDecl
	{
		let field = $1?;
		Ok(LinkedList::from(field))
	}
	;

FieldDecl -> Result<Field,String>:
	FieldType "VAR" ';'
	{
		let v = $2.map_err(|_| "VAR Err".to_string())?; 
		Ok(Field{
		name:parse_string($lexer.span_str(v.span())).unwrap(),
			field_type: $1?,
		})
	}
	| FieldType FieldPtr "VAR" ';'
	{
		let basetype = $1?;
		let mut ptrtype = $2?;
		let v = $3.map_err(|_| "VAR Err".to_string())?; 
		ptrtype.set_base_type(basetype.get_base_type());
		Ok(Field{
		name :parse_string($lexer.span_str(v.span())).unwrap(),
			field_type: ptrtype
		})
	}
	;
FieldType -> Result<FieldType,String>: 
	'INT_T'
	{
		Ok(FieldType::Primitive(PrimitiveType::Int))
	} 
	| 'STR_T'
	{
		Ok(FieldType::Primitive(PrimitiveType::String))
	}
	| 'VAR'
	{
		let v = $1.map_err(|_| "VAR Err".to_string())?; 
		let typename= parse_string($lexer.span_str(v.span())).unwrap();
		Ok(FieldType::Struct(typename))
	}
    ;

FieldPtr-> Result<FieldType,String>: 
	FieldPtr '*'
	{
		Ok(FieldType::Pointer(Box::new($1?)))
	}
	| '*'
	{
		Ok(FieldType::Pointer(Box::new(FieldType::Primitive(PrimitiveType::Void))))
	}
	;

%%
// Any functions here are in scope for all the grammar actions above.
use crate::parserlib::{*};
use crate::codegen::exit_on_err;
use std::collections::LinkedList;
