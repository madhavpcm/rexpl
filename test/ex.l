%option noyywrap

/*Declarations section start here*/

/* Auxiliary declarations start here*/

%{
	#include <stdio.h>
	int global_variable;
%}

/*Auxiliary declarations end & Regular definitions start here*/

number [0-9]+
op [=|-|*|^|/|+]

/*Declarations section ends here*/

%%

{number}  {printf(" number");}
{op}      {printf(" operator");}

%%

int main()
{
    yylex();
    return 1;
}
