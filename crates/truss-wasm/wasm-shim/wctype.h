/* Minimal wctype.h shim for wasm32-unknown-unknown. */
#ifndef _WCTYPE_H
#define _WCTYPE_H

typedef unsigned int wint_t;
typedef int wchar_t;

int iswspace(wint_t wc);
int iswdigit(wint_t wc);
int iswalpha(wint_t wc);
int iswalnum(wint_t wc);

#endif
