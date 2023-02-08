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
Start -> Result<ASTNode, ()>:
	GDeclBlock FDefBlock MainBlock
	{
		Ok(ASTNode::BinaryNode{
			op : ASTNodeType::Connector,
            exprtype : Some(ASTExprType::Null),
			lhs : Box::new(ASTNode::BinaryNode{
                op: ASTNodeType::Connector,
				exprtype : Some(ASTExprType::Null),
                lhs: Box::new($1?),
                rhs: Box::new($2?),
            }),
			rhs : Box::new($3?),
		})
	}
	| GDeclBlock MainBlock 
	{
		Ok(ASTNode::BinaryNode{
			op : ASTNodeType::Connector,
            exprtype : Some(ASTExprType::Null),
			lhs : Box::new($1?),
			rhs : Box::new($2?),
		})
	}
    | MainBlock 
    {
        Ok(ASTNode::BinaryNode{
            op: ASTNodeType::Connector,
            exprtype : Some(ASTExprType::Null),
            lhs: Box::new(ASTNode::Null),
            rhs: Box::new($1?),
        })
    }
	;
MainBlock -> Result<ASTNode,()>:
	"INT_T" "MAIN" '('  ')' '{' LDeclBlock BeginBlock '}'
	{
		let mut ss = SCOPE_STACK.lock().unwrap();
		let mut fs = FUNCTION_STACK.lock().unwrap();
		fs.push("main".to_string());
		ss.push(HashMap::default().clone());
		log::info!("before size ofscope : {}", ss.last().unwrap().len());

		std::mem::drop(ss);
		std::mem::drop(fs);

		log::info!("main()");
		let ldecl_ = $6.map_err(|_| ())?;
		let body_ = $7.map_err(|_| ())?;
		__gen_local_symbol_table(&ldecl_, &ParamList::Null);

		let node = ASTNode::MainNode{
			decl: Box::new(ldecl_),
			body: Box::new(body_),
		};

		let mut ss =SCOPE_STACK.lock().unwrap();
		log::info!("before size ofscope : {}", ss.last().unwrap().len());
		let mut ft = FUNCTION_TABLE.lock().unwrap();
		ft.insert(
			"main".to_string(),
			ss.last().unwrap().clone()
		);
    log::info!("in the end size ofscope : {}", ss.len());
		ss.pop();
		Ok(node)
	}
	;
BeginBlock -> Result<ASTNode,()>:
	"BEGIN" StmtList "END" 
	{
		let ss = SCOPE_STACK.lock().unwrap();
		if let Some(sz) = ss.last() {
			log::info!("before size ofscope : {}", sz.len());
		}
		else{
			log::error!("err {}" , ss.len());
		}
		std::mem::drop(ss);
		$2
	}
	| "BEGIN" "END" 
	{
		log::error!("Empty beginend");
		Ok(ASTNode::Null)
	}
	;
GDeclBlock -> Result<ASTNode,()>:
	"DECL" GDeclList "ENDDECL" 
	{
		$2
	}
	| "DECL" "ENDDECL" 
	{
		Ok(ASTNode::Null)
	}
	;

LDeclList -> Result<ASTNode, ()>:
	LDeclList LDecl 
	{
		let decl = $2?;

		Ok(ASTNode::BinaryNode{
			op : ASTNodeType::Connector,
            exprtype : Some(ASTExprType::Null),
			lhs : Box::new($1?),
			rhs : Box::new(decl),
		})
	}
	|
	LDecl
	{
		let decl =$1?;
		Ok(decl)
	}
	;

GDeclList -> Result<ASTNode, ()>:
	GDeclList GDecl 
	{
		let decl = $2?;

		__gen_global_symbol_table(&decl);

		Ok(ASTNode::BinaryNode{
			op : ASTNodeType::Connector,
            exprtype : Some(ASTExprType::Null),
			lhs : Box::new($1?),
			rhs : Box::new(decl),
		})
	}
	| GDecl
	{
		let decl =$1?;
		__gen_global_symbol_table(&decl);
		Ok(decl)
	}
	;

LDecl ->  Result<ASTNode,()>:
	Type VarList ';'
	{
		Ok(ASTNode::DeclNode{
			var_type: $1?,
			list: Box::new($2?)
		})
	}
	;

GDecl ->  Result<ASTNode,()>:
	Type VarList ';'
	{
		Ok(ASTNode::DeclNode{
			var_type: $1?,
			list: Box::new($2?)
		})
	}
	| Type "VAR" '(' ParamList ')' ';'
	{
		let returntype = $1?;
		let v = $2.map_err(|_| ())?;
		let functionname= parse_string($lexer.span_str(v.span())).unwrap();
		let paramlist = $4?;

		let node = ASTNode::FuncDeclNode{
			fname: functionname.clone(),
			ret_type: returntype,
			paramlist: Box::new(paramlist.clone()),
		};

		if validate_ast_node(&node) == Ok(false){
			return Ok(ASTNode::ErrorNode{
                err : ASTError::TypeError("Function [".to_owned() + functionname.as_str() + "] is redeclared."),
            });
		}

		let mut gst = GLOBALSYMBOLTABLE.lock().unwrap();
		gst.insert(
			functionname.clone(),
			GSymbol::Func {
				ret_type: (returntype.clone()),
				paramlist: Box::new(paramlist),
				flabel: (0),
			},
		);

		Ok(node)
	}
	| PtrType "VAR" '(' ParamList ')' ';'
	{
		let returntype = $1?;
		let v = $2.map_err(|_| ())?;
		let functionname= parse_string($lexer.span_str(v.span())).unwrap();
		let paramlist = $4?;

		let node = ASTNode::FuncDeclNode{
			fname: functionname.clone(),
			ret_type: returntype,
			paramlist: Box::new(paramlist.clone()),
		};

		if validate_ast_node(&node) == Ok(false){
			return Ok(ASTNode::ErrorNode{
                err : ASTError::TypeError("Function [".to_owned() + functionname.as_str() + "] is redeclared."),
            });
		}

		let mut gst = GLOBALSYMBOLTABLE.lock().unwrap();
		gst.insert(
			functionname.clone(),
			GSymbol::Func {
				ret_type: (returntype.clone()),
				paramlist: Box::new(paramlist),
				flabel: (0),
			},
		);

		Ok(node)
	}
	;

PtrType -> Result<ASTExprType,()>:
	"STRPTR_T"
	{
		Ok(ASTExprType::StringRef)
	}
	| "INTPTR_T"
	{
		Ok(ASTExprType::IntRef)
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
	VarList ',' VariableDef 
	{
		let var = $3?;
		match var {
			VarList::Node { var,refr,indices,next}=> {
				Ok(VarList::Node{
					var:var,
					refr:refr,
					indices:indices,
					next:Box::new($1?),
				})
			},
			VarList::Null => {
				Ok(VarList::Null)
			}
		}
	}
	| VarList ',' '*' VariableDef
	{
		let var = $4?;
		match var {
			VarList::Node { var,refr:_,indices,next}=> {
				Ok(VarList::Node{
					var:var,
					refr:true,
					indices:indices,
					next:Box::new($1?),
				})
			},
			VarList::Null => {
				Ok(VarList::Null)
			}
		}
	}
	| VariableDef 
	{
		$1
	} 
	| '*' VariableDef
	{
		let var = $2?;
		match var {
			VarList::Node { var,refr:_,indices,next}=> {
				Ok(VarList::Node{
					var:var,
					refr:true,
					indices:indices,
					next:next
				})
			},
			_ =>{
				Ok(VarList::Null)
			}
		}
	}
    ;

FDefBlock -> Result<ASTNode,()>:
	FDefBlock FDef 
	{
		Ok(ASTNode::BinaryNode{
			op: ASTNodeType::Connector,
            exprtype : Some(ASTExprType::Null),
			lhs: Box::new($1?),
			rhs: Box::new($2?),
		})
	}
	| FDef
	{
		$1
	}
	;

FDef -> Result<ASTNode,()>:
	PtrType "VAR" '(' ParamList ')' '{' LDeclBlock BeginBlock '}'
	{
		let v = $2.map_err(|_| ())?;
		let funcname = parse_string($lexer.span_str(v.span())).unwrap();

		let gst = GLOBALSYMBOLTABLE.lock().unwrap();
		if gst.contains_key(&funcname) == false {
			return Ok(ASTNode::ErrorNode{
                err : ASTError::TypeError("Function [".to_owned() + funcname.as_str() + "] is not declared."),
            });
		}

		let mut ss = SCOPE_STACK.lock().unwrap();
		let mut fs = FUNCTION_STACK.lock().unwrap();
		fs.push(funcname.clone());
		ss.push(HashMap::default());
		log::info!("()");

		std::mem::drop(ss);
		std::mem::drop(gst);
		std::mem::drop(fs);

		log::info!("asdf");
		let ldecl_ = $7?;
		let paramlist_ = $4?;

		__gen_local_symbol_table( &ldecl_, &paramlist_ );

		let node = ASTNode::FuncDefNode{
			fname: funcname.clone(),
			ret_type: $1?,
			paramlist: Box::new(paramlist_),
			decl: Box::new(ldecl_),
			body: Box::new($8?),
		};

		let mut ss =SCOPE_STACK.lock().unwrap();
		let mut ft = FUNCTION_TABLE.lock().unwrap();
		ft.insert(
			funcname,
			ss.last().unwrap().clone()
		);
		ss.pop();

		Ok(node)
	}
	| Type "VAR" '(' ParamList ')' '{' LDeclBlock BeginBlock '}'
	{
		let v = $2.map_err(|_| ())?;
		let funcname = parse_string($lexer.span_str(v.span())).unwrap();

		let gst = GLOBALSYMBOLTABLE.lock().unwrap();
		if gst.contains_key(&funcname) == false {
			return Ok(ASTNode::ErrorNode{
                err : ASTError::TypeError("Function [".to_owned() + funcname.as_str() + "] is not declared."),
            });
		}

		let mut ss = SCOPE_STACK.lock().unwrap();
		let mut fs = FUNCTION_STACK.lock().unwrap();
		fs.push(funcname.clone());
		ss.push(HashMap::default().clone());

		std::mem::drop(ss);
		std::mem::drop(gst);
		std::mem::drop(fs);

		log::info!("sadf");
		let ldecl_ = $7?;
		let paramlist_ = $4?;

		__gen_local_symbol_table(&ldecl_, &paramlist_ );

		let body_ = $8?;
		let node = ASTNode::FuncDefNode{
			fname: funcname.clone(),
			ret_type: $1?,
			paramlist: Box::new(paramlist_),
			decl: Box::new(ldecl_),
			body: Box::new(body_),
		};

		let mut ss =SCOPE_STACK.lock().unwrap();
		let mut ft = FUNCTION_TABLE.lock().unwrap();
		ft.insert(
			funcname,
			ss.last().unwrap().clone()
		);
		ss.pop();

		Ok(node)
	}
	;

LDeclBlock -> Result<ASTNode,()>:
	"DECL" LDeclList "ENDDECL"
	{
		log::warn!("ss");
	    $2
	}
	| "DECL" "ENDDECL"
	{
		log::warn!("Empty Declaration");
        Ok(ASTNode::Null)
	}
	;

ParamList -> Result<ParamList,()>:
	ParamList ',' Param
	{
        let param = $3?;
        match param {
            ParamList::Node {var, vartype, indices, next} => {
                Ok(ParamList::Node{
                    var:var,
                    vartype:vartype,
                    indices:indices,
                    next:Box::new($1?)
                })
            },
            ParamList::Null =>{
                Ok(param)
            }
        }
	}
	| Param
	{
        $1
	}
	;

Param -> Result<ParamList,()>:
	PtrType VariableDef
    {
		let var = $2?;

        let vtype = $1?;
		match var {
			VarList::Node { var,refr:_,indices,next}=> {
				if indices != Vec::default() {
					exit_on_err("Arrays cannot be used as a function parameter. Use a pointer instead.".to_owned());
				}
				Ok(ParamList::Node{
					var:var,
                    vartype:vtype,
					indices:indices,
					next:Box::new(ParamList::Null),
				})
			},
			VarList::Null => {
				Ok(ParamList::Null)
			}
		}
        
    }
	| Type VariableDef
    {
		let var = $2?;

        let vtype = $1?;
		match var {
			VarList::Node { var,refr:_,indices,next}=> {
				if indices != Vec::default() {
					exit_on_err("Arrays cannot be used as a function parameter. Use a pointer instead.".to_owned());
				}
				Ok(ParamList::Node{
					var:var,
                    vartype:vtype,
					indices:indices,
					next:Box::new(ParamList::Null),
				})
			},
			VarList::Null => {
				Ok(ParamList::Null)
			}
		}
        
    }
	| 
	{
		Ok(ParamList::Null)
	}
	;


ArgList -> Result<ArgList,()>:
	ArgList ',' Expr
	{
		let expr = $3?;
		Ok(ArgList::Node{
			expr: expr,
			next: Box::new($1?),
		})
	}
	| Expr
	{
		let expr = $1?;
		Ok(ArgList::Node{
			expr: expr,
			next: Box::new(ArgList::Null),
		})
	}
	;

VariableDef -> Result<VarList,()>:
	"VAR" "[" "INT" "]"
	{
		let v = $1.map_err(|_| ())?;
		let var_ = parse_string($lexer.span_str(v.span())).unwrap();

		let v = $3.map_err(|_| ())?;
        let i= parse_usize($lexer.span_str(v.span())).unwrap();

        let mut indices_ :Vec<usize>= Vec::default();
        indices_.push(i);

		Ok(VarList::Node{
			var: var_,
			refr:false,
			indices: indices_,
			next: Box::new(VarList::Null),
		})
	}
	| "VAR" 
	{
		let v = $1.map_err(|_| ())?;
		let var_ = parse_string($lexer.span_str(v.span())).unwrap();

		Ok(VarList::Node{
			var: var_,
			refr:false,
			indices: Vec::default(),
			next: Box::new(VarList::Null),
		})
	}
	| "VAR" "[" "INT" "]" "[" "INT" "]"
	{
		let v = $1.map_err(|_| ())?;
		let var_ = parse_string($lexer.span_str(v.span())).unwrap();

		let v = $3.map_err(|_| ())?;
        let i= parse_usize($lexer.span_str(v.span())).unwrap();

		let v = $6.map_err(|_| ())?;
        let j= parse_usize($lexer.span_str(v.span())).unwrap();

        let mut indices_ : Vec<usize> = Vec::default();
        indices_.push(i);
        indices_.push(j);

		Ok(VarList::Node{
			var: var_,
			refr:false,
			indices: indices_,
			next: Box::new(VarList::Null),
		})
	}
	;	


//StateMents	

StmtList -> Result<ASTNode, ()>:
	StmtList Stmt 
	{
		Ok(ASTNode::BinaryNode{
			op : ASTNodeType::Connector,
            exprtype : Some(ASTExprType::Null),
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
	| "RETURN" Expr ';'
	{
		Ok(ASTNode::ReturnNode{
			expr: Box::new($2?)
		})
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
		let v = $2.map_err(|_| ())?;
		let var = parse_string($lexer.span_str(v.span())).unwrap();

		let lhs = $1?;
		let rhs = $3?;
		Ok(ASTNode::BinaryNode{
			op : ASTNodeType::Equals,
            exprtype : Some(ASTExprType::Null),
			lhs : Box::new(lhs),
			rhs : Box::new(rhs),
		})
	}
	| '*' Variable '=' Expr ';'
	{
		let v = $3.map_err(|_| ())?;
		let var = parse_string($lexer.span_str(v.span())).unwrap();

		let lhs = $2?;
		let rhs = $4?;

		Ok(ASTNode::BinaryNode{
			op : ASTNodeType::Equals,
            exprtype : Some(ASTExprType::Null),
			lhs : Box::new(ASTNode::UnaryNode{
				op: ASTNodeType::Deref,
				ptr: Box::new(lhs)
			}),
			rhs : Box::new(rhs),
		})
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
		let v = $2.map_err(|_| ())?;
		let var = parse_string($lexer.span_str(v.span())).unwrap();

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
                err : ASTError::TypeError("Type mismatch".to_owned()),
            });
		}

		Ok(node)
	}
	| Expr '>' Expr 
	{
		let v = $2.map_err(|_| ())?;
		let var = parse_string($lexer.span_str(v.span())).unwrap();

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
		let v = $2.map_err(|_| ())?;
		let var = parse_string($lexer.span_str(v.span())).unwrap();

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
		let v = $2.map_err(|_| ())?;
		let var = parse_string($lexer.span_str(v.span())).unwrap();

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
		let v = $2.map_err(|_| ())?;
		let var = parse_string($lexer.span_str(v.span())).unwrap();

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
		let v = $2.map_err(|_| ())?;
		let var = parse_string($lexer.span_str(v.span())).unwrap();

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
		let v = $2.map_err(|_| ())?;
		let var = parse_string($lexer.span_str(v.span())).unwrap();

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
		let v = $2.map_err(|_| ())?;
		let var = parse_string($lexer.span_str(v.span())).unwrap();

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
		let v = $2.map_err(|_| ())?;
		let var = parse_string($lexer.span_str(v.span())).unwrap();
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
		let v = $2.map_err(|_| ())?;
		let var = parse_string($lexer.span_str(v.span())).unwrap();

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
		let v = $2.map_err(|_| ())?;
		let var = parse_string($lexer.span_str(v.span())).unwrap();

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
			arglist: Box::new(ArgList::Null),
		};

		if validate_ast_node(&node) == Ok(false) {
			return Ok(ASTNode::ErrorNode{
				err: ASTError::TypeError("Arglist mismatch in type".to_string())
            });
		}

		Ok(node)
	}
	| "VAR" '(' ArgList ')'
	{
		log::info!("arglist");
		let v = $1.map_err(|_| ())?;
		let functionname= parse_string($lexer.span_str(v.span())).unwrap();

		
		let node = ASTNode::FuncCallNode{
			fname: functionname, 
			arglist: Box::new($3?),
		};

		if validate_ast_node(&node) == Ok(false) {
			return Ok(ASTNode::ErrorNode{
				err: ASTError::TypeError("Arglist mismatch in type".to_string())
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
use crate::codegen::SCOPE_STACK;
use crate::codegen::FUNCTION_STACK;
use crate::codegen::exit_on_err;
use std::collections::HashMap;
