%%
/\*[^*]*\*+(?:[^/*][^*]*\*+)*/ "MULTI_COMMENT"
//[^\n]* "SINGLE_COMMENT"

if "IF"
fi "FI"
[sS][0-9]* "S"
==|<=|>=|!=|<|> "COND"

[A-Za-z][a-zA-Z0-9]* "VAR"
[A][a-z|A-Z][a-z|A-Z] "SPEC"
[0-9]+[.][0-9]+ "FLOAT"
[0-9]+ "INT"
\w+ "WORD"
\+ "+"
\* "*"
/ "/"
\- "-"
\( "("
\) ")"
\{ "{"
\} "}"
[\t\n ]+ ;

