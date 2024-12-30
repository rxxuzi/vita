#include "utils.h"
#include <string.h>
#include <stdlib.h>

#ifdef _WIN32
#include <windows.h>
#include <direct.h> // _getcwd
#endif

void print_usage() {
    printf("Usage: vita [OPTIONS] <DIR_PATH>\n");
    printf("Options:\n");
    printf("  -o <FILE>   Write output to <FILE> instead of standard output.\n");
    printf("  -d <DEPTH>  Set maximum depth for directory traversal (default: unlimited).\n");
    printf("  -h, --help Show this help message and exit.\n");
}

void normalize_path(char *path) {
    // バックスラッシュ -> スラッシュ
    for (char *p = path; *p; ++p) {
        if (*p == '\\') {
            *p = '/';
        }
    }
    size_t len = strlen(path);
    while (len > 0 && (path[len - 1] == '/' || path[len - 1] == '\\')) {
        path[--len] = '\0';
    }
}

void print_file_content(const char *path, FILE *out) {
    int bin = is_binary(path);
    // Header
    fprintf(out, "--------------------------------------------------------------------------------\n");
    fprintf(out, "%s:\n", path);
    fprintf(out, "--------------------------------------------------------------------------------\n");

    if (bin == -1) {
        fprintf(out, "Cannot open file: %s\n\n", path);
        return;
    }
    if (bin == 1) {
        fprintf(out, "This is binary file\n\n");
        return;
    }

    FILE *fp = fopen(path, "r");
    if (!fp) {
        fprintf(out, "Cannot open file: %s\n\n", path);
        return;
    }

    char buffer[BUFFER_SIZE];
    while (fgets(buffer, sizeof(buffer), fp)) {
        fprintf(out, "%s", buffer);
    }
    fprintf(out, "\n");
    fclose(fp);
}

#ifdef _WIN32
char *realpath(const char *path, char *resolved_path) {
    if (resolved_path != NULL) {
        return _fullpath(resolved_path, path, MAX_PATH_LENGTH);
    } else {
        char *buffer = (char *)malloc(MAX_PATH_LENGTH);
        if (buffer == NULL) {
            return NULL;
        }
        char *result = _fullpath(buffer, path, MAX_PATH_LENGTH);
        if (result == NULL) {
            free(buffer);
            return NULL;
        }
        return buffer;
    }
}
#endif
