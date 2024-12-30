#ifndef UTILS_H
#define UTILS_H

#include "options.h"
#include <stdio.h>

#define BUFFER_SIZE 1024
#define BINARY_CHECK_BYTES 512
#define MAX_PATH_LENGTH 4096

void print_usage();
void normalize_path(char *path);
int is_binary(const char *path);
void print_file_content(const char *path, FILE *out);

#ifdef _WIN32
char *realpath(const char *path, char *resolved_path);
#endif

#endif // UTILS_H
