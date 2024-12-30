#include "traverse.h"
#include "utils.h"
#include <string.h>

#ifdef _WIN32
#include <windows.h>
#else
#include <dirent.h>
#include <unistd.h>
#include <sys/stat.h>
#endif

void traverse_directory(const char *dir_path, FILE *out, int depth, int max_depth, const char *output_file_path) {
    if (max_depth != -1 && depth > max_depth) {
        return;
    }

#ifdef _WIN32
    // Windows
    char search_path[MAX_PATH_LENGTH];
    snprintf(search_path, sizeof(search_path), "%s/*", dir_path);
    normalize_path(search_path);

    WIN32_FIND_DATAA fd;
    HANDLE hFind = FindFirstFileA(search_path, &fd);
    if (hFind == INVALID_HANDLE_VALUE) {
        fprintf(out, "Cannot open directory: %s\n", dir_path);
        return;
    }

    do {
        const char *name = fd.cFileName;
        if (strcmp(name, ".") == 0 || strcmp(name, "..") == 0) {
            continue;
        }

        char path[MAX_PATH_LENGTH];
        snprintf(path, sizeof(path), "%s/%s", dir_path, name);
        normalize_path(path);

        if (fd.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) {
            if (max_depth == -1 || depth < max_depth) {
                traverse_directory(path, out, depth + 1, max_depth, output_file_path);
            }
        } else {
            if (strcmp(path, output_file_path) == 0) {
                continue;
            }
            print_file_content(path, out);
        }
    } while (FindNextFileA(hFind, &fd) != 0);

    FindClose(hFind);
#else
    // POSIX (Linux, macOS, etc.)
    DIR *dir = opendir(dir_path);
    if (!dir) {
        fprintf(out, "Cannot open directory: %s\n", dir_path);
        return;
    }

    struct dirent *entry;
    while ((entry = readdir(dir)) != NULL) {
        const char *name = entry->d_name;
        if (strcmp(name, ".") == 0 || strcmp(name, "..") == 0) {
            continue;
        }

        char path[MAX_PATH_LENGTH];
        snprintf(path, sizeof(path), "%s/%s", dir_path, name);
        normalize_path(path);

        struct stat st;
        if (stat(path, &st) == -1) {
            perror("stat");
            continue;
        }

        if (S_ISDIR(st.st_mode)) {
            if (max_depth == -1 || depth < max_depth) {
                traverse_directory(path, out, depth + 1, max_depth, output_file_path);
            }
        } else {
            if (strcmp(path, output_file_path) == 0) {
                continue;
            }
            print_file_content(path, out);
        }
    }
    closedir(dir);
#endif
}
