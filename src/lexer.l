%%
/\*[^*]*\*+(?:[^/*][^*]*\*+)*/ "MULTI_COMMENT"
[\t ]*//[^\n]* "SINGLE_COMMENT"

endwhile "ENDWHILE"
while "WHILE"
do "DO"
if "IF"
else "ELSE"
endif "ENDIF"
begin "BEGIN"
end "END"
read "READ"
write "WRITE"
then "THEN"
breakpoint "BREAKPOINT"
break "BREAK"
continue "CONTINUE"
decl "DECL"
enddecl "ENDDECL"
main "MAIN"
return "RETURN"
str "STR_T"
int "INT_T"
type "TYPE"
endtype "ENDTYPE"
alloc "ALLOC"
free "FREE"
null|NULL 'NULL'
initialize "INIT"
syscall "SYSCALL"
setaddr 'SETADDR'
getaddr 'GETADDR'
class 'CLASS'
endclass 'ENDCLASS'
this 'THIS'
div 'DIV'
extends 'EXTENDS'
new 'NEW'

[.] "DOT"
-> "ARROW"
== "=="
<= "<="
>= ">="
!= "!="
< "<"
> ">"

[a-z|A-Z|_][a-z|A-Z|_|0-9]* "VAR"
[0-9]+ "INT"
["]([^"])*["] "STR"

& "&"
= "="
\+ "+"
\* "*"
/ "/"
% "%"
\- "-"
\( "("
\) ")"
\{ "{"
\} "}"
\[ "["
\] "]"
; ";"
, ','
[\t\n ]+ ;

