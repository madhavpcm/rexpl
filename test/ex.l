%option noyywrap

/*Declarations section start here*/

/* Auxiliary declarations start here*/

%{
	#include <stdio.h>
	#include <stdlib.h>
	int global_variable;
%}

/*Auxiliary declarations end & Regular definitions start here*/

number [0-9]+
op [=|-|*|^|/|+]

/*Declarations section ends here*/

%%

{number}  { atoi(yytext) % 2 == 0? printf("Even number detected of length %d", yyleng) : printf("Odd number detected of length %d",yyleng);}
{op}      {printf(" operator");}

%%

int main(int argc , char** argv)
{
	if( argc > 1 ){
		FILE *fp = fopen(argv[1], "r");
		if( fp != 0x0 ){
			yyin = fp;
		}
	}
	
    yylex();
    return 1;
}
