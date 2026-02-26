/* Minimal stdlib.h shim for wasm32-unknown-unknown.
 * Provides just enough for tree-sitter's C code to compile. */
#ifndef _STDLIB_H
#define _STDLIB_H

#include <stddef.h>

void *malloc(size_t size);
void *calloc(size_t nmemb, size_t size);
void *realloc(void *ptr, size_t size);
void free(void *ptr);
_Noreturn void abort(void);
long strtol(const char *nptr, char **endptr, int base);

#endif
