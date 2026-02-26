/* Minimal unistd.h shim for wasm32-unknown-unknown. */
#ifndef _UNISTD_H
#define _UNISTD_H

#include <stddef.h>

int dup(int fd);

#endif
