/* Minimal ctype.h shim for wasm32-unknown-unknown. */
#ifndef _CTYPE_H
#define _CTYPE_H

int isalpha(int c);
int isdigit(int c);
int isalnum(int c);
int isspace(int c);
int isupper(int c);
int islower(int c);
int toupper(int c);
int tolower(int c);

#endif
