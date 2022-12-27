%%
/\*[^*]*\*+(?:[^/*][^*]*\*+)*/ "MULTI_COMMENT"
//[^\n]* "SINGLE_COMMENT"
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
[\t\n ]+ ;
