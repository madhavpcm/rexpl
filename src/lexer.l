%%
/\*[^*]*\*+(?:[^/*][^*]*\*+)*/ "MULTI_COMMENT"
//[^\n]* "SINGLE_COMMENT"

if "IF"
fi "FI"
begin "BEGIN"
end "END"
read "READ"
write "WRITE"

==|<=|>=|!=|<|> "COND"

[a-z] "VAR"
[0-9]+[.][0-9]+ "FLOAT"
[0-9]+ "INT"
= "="
\+ "+"
\* "*"
/ "/"
\- "-"
\( "("
\) ")"
\{ "{"
\} "}"
; ";"
[\t\n ]+ ;

