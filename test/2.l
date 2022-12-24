
/*Declarations section start here*/

/* Auxiliary declarations start here*/

%{
	#include <stdio.h>
	#include <stdlib.h>
	int global_variable;
	static int nCount,lbCount, sCount, wCount, cCount, fCount;
%}

/*Auxiliary declarations end & Regular definitions start here*/

number [0-9]+
floating [0-9]+[.][0-9]+
op [=|-|*|^|/|+]
lbreak [\n]
space [\b \b]
word [a-z|A-Z]+
char [a-z|A-Z|0-9|\n|\b \b]+

/*Declarations section ends here*/

%%

{floating} {
	fCount++;cCount+=yyleng;
}
{number}  { 
	nCount++;cCount+=yyleng;
}
{lbreak} {
	lbCount++;cCount+=yyleng;
}
{space} {
	sCount++;cCount+=yyleng;
}
{word} {
	wCount++;cCount+=yyleng;
}
{op}      {printf(" operator");}

%%
int yywrap(){
	return -1;
}

int main(int argc , char** argv)
{

	if( argc > 1 ){
		FILE *fp = fopen(argv[1], "r");
		if( fp != 0x0 ){
			yyin = fp;
		}
	}
	
	int val = yylex();
	printf("Numbers :: %d\nLines ::%d\nSpaces ::%d\nWords ::%d\nChars ::%d\nFloating ::%d\n",nCount,lbCount,sCount,wCount,cCount,fCount);
    return 1;
}
