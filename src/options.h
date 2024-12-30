#ifndef OPTIONS_H
#define OPTIONS_H

#define MAX_OUTPUT_FILE 256
//#define MAX_PATH_LENGTH 4096

typedef struct {
    char output_file[MAX_OUTPUT_FILE];
    int max_depth;
} Options;

#endif // OPTIONS_H
