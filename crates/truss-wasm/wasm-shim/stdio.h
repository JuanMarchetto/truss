/* Minimal stdio.h shim for wasm32-unknown-unknown. */
#ifndef _STDIO_H
#define _STDIO_H

#include <stddef.h>

typedef struct FILE FILE;
extern FILE *stderr;

int fprintf(FILE *stream, const char *format, ...);
int snprintf(char *str, size_t size, const char *format, ...);
int vsnprintf(char *str, size_t size, const char *format, ...);

#endif
