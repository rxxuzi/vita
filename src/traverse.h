#ifndef TRAVERSE_H
#define TRAVERSE_H

#include <stdio.h>
#include "options.h"

void traverse_directory(const char *dir_path, FILE *out, int depth, int max_depth, const char *output_file_path);

#endif // TRAVERSE_H
