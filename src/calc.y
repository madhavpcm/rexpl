%start Expr
%avoid_insert "INT"

%%
Diver -> Result<u64, ()>:
      Diver '/' Expr { Ok($1? / $3?) }
    | Expr { $1 }
    ;
Expr -> Result<u64, ()>:
      Expr '+' Term { Ok($1? + $3?) }
    | Term { $1 }
    ;

Term -> Result<u64, ()>:
      Term '*' Factor { Ok($1? * $3?) }
    | Worm { $1 }
    ;

Worm -> Result<u64, ()>:
      Worm '-' Factor { Ok($1? - $3?) }
    | Factor { $1 }
    ;

Factor -> Result<u64, ()>:
      '(' Expr ')' { $2 }
    | 'INT'
      {
          let v = $1.map_err(|_| ())?;
          parse_int($lexer.span_str(v.span()))
      }
    ;
%%
// Any functions here are in scope for all the grammar actions above.

fn parse_int(s: &str) -> Result<u64, ()> {
    match s.parse::<u64>() {
        Ok(val) => Ok(val),
        Err(_) => {
            eprintln!("{} cannot be represented as a u64", s);
            Err(())
        }
    }
}
