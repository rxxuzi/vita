#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <getopt.h>
#include <sys/stat.h>
#include <limits.h>
#include <unistd.h>
#include "options.h"
#include "utils.h"
#include "traverse.h"

int main(int argc, char *argv[]) {
    if (argc < 2) {
        print_usage();
        return EXIT_FAILURE;
    }

    Options opts;
    opts.output_file[0] = '\0'; // デフォルトは標準出力
    opts.max_depth = -1; // デフォルトは無制限

    struct option long_options[] = {
            {"output", required_argument, 0, 'o'},
            {"depth",  required_argument, 0, 'd'},
            {"help",   no_argument,       0, 'h'},
            {0, 0, 0, 0}
    };

    int opt;
    while ((opt = getopt_long(argc, argv, "o:d:h", long_options, NULL)) != -1) {
        switch (opt) {
            case 'o': // 出力ファイル
                strncpy(opts.output_file, optarg, MAX_OUTPUT_FILE - 1);
                opts.output_file[MAX_OUTPUT_FILE - 1] = '\0';
                break;
            case 'd': // 最大深度
                opts.max_depth = atoi(optarg);
                if (opts.max_depth < 0) {
                    fprintf(stderr, "Error: Invalid depth value '%s'. Must be a non-negative integer.\n", optarg);
                    return EXIT_FAILURE;
                }
                break;
            case 'h': // ヘルプ
                print_usage();
                return EXIT_SUCCESS;
            case '?':
                if (optopt) {
                    fprintf(stderr, "vita: unknown option -- %c\n", optopt);
                } else {
                    fprintf(stderr, "vita: unknown option\n");
                }
                print_usage();
                return EXIT_FAILURE;
            default:
                print_usage();
                return EXIT_FAILURE;
        }
    }

    if (optind >= argc) {
        fprintf(stderr, "Error: Directory path not specified.\n");
        print_usage();
        return EXIT_FAILURE;
    }
    const char *dir_path = argv[optind];

    struct stat path_stat;
    if (stat(dir_path, &path_stat) != 0) {
        fprintf(stderr, "Error: Directory '%s' does not exist.\n", dir_path);
        return EXIT_FAILURE;
    }
    if (!S_ISDIR(path_stat.st_mode)) {
        fprintf(stderr, "Error: '%s' is not a directory.\n", dir_path);
        return EXIT_FAILURE;
    }

    FILE *out;
    char output_file_path[MAX_PATH_LENGTH] = {0};

    if (opts.output_file[0] != '\0') {
        // 絶対パスに変換
        char *resolved = realpath(opts.output_file, output_file_path);
        if (resolved == NULL) {
            char cwd[MAX_PATH_LENGTH];
            if (getcwd(cwd, sizeof(cwd)) != NULL) {
                if (strlen(cwd) + strlen(opts.output_file) + 1 >= sizeof(output_file_path)) {
                    fprintf(stderr, "Error: Output file path is too long.\n");
                    return EXIT_FAILURE;
                }
                snprintf(output_file_path, sizeof(output_file_path), "%s/%s", cwd, opts.output_file);

                normalize_path(output_file_path);
            } else {
                fprintf(stderr, "Error: Cannot get current working directory.\n");
                return EXIT_FAILURE;
            }

        }

        out = fopen(opts.output_file, "w");
        if (!out) {
            fprintf(stderr, "Error: Cannot open output file: %s\n", opts.output_file);
            return EXIT_FAILURE;
        }
    } else {
        out = stdout;
    }

    char normalized_root[MAX_PATH_LENGTH];
    snprintf(normalized_root, sizeof(normalized_root), "%s", dir_path);
    normalize_path(normalized_root);

    fprintf(out, "================================================================================\n");
    fprintf(out, "%s/\n", normalized_root);
    fprintf(out, "================================================================================\n");

    traverse_directory(normalized_root, out, 0, opts.max_depth, output_file_path);

    if (out != stdout) {
        fclose(out);
    }

    return EXIT_SUCCESS;
}
