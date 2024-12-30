#include <stdio.h>
#include <stdlib.h>
#include <ctype.h>

#define BINARY_CHECK_BYTES 2048

// UTF-8の検証関数
static int is_valid_utf8(const unsigned char *buf, size_t len) {
    size_t i = 0;
    while (i < len) {
        if (buf[i] <= 0x7F) { // ASCII
            i++;
        }
        else if ((buf[i] & 0xE0) == 0xC0) { // 2byte
            if (i + 1 >= len || (buf[i+1] & 0xC0) != 0x80)
                return 0;
            i += 2;
        }
        else if ((buf[i] & 0xF0) == 0xE0) { // 3byte
            if (i + 2 >= len ||
                (buf[i+1] & 0xC0) != 0x80 ||
                (buf[i+2] & 0xC0) != 0x80)
                return 0;
            i += 3;
        }
        else if ((buf[i] & 0xF8) == 0xF0) { // 4byte
            if (i + 3 >= len ||
                (buf[i+1] & 0xC0) != 0x80 ||
                (buf[i+2] & 0xC0) != 0x80 ||
                (buf[i+3] & 0xC0) != 0x80)
                return 0;
            i += 4;
        }
        else {
            return 0;
        }
    }
    return 1;
}

// Shift-JISの検証関数
static int is_valid_shift_jis(const unsigned char *buf, size_t len) {
    size_t i = 0;
    while (i < len) {
        unsigned char c = buf[i];
        if (c <= 0x7F || (0xA1 <= c && c <= 0xDF)) { // ASCII or half-width katakana
            i++;
        }
        else if ((0x81 <= c && c <= 0x9F) || (0xE0 <= c && c <= 0xEF)) { // First byte of double-byte character
            if (i + 1 >= len) return 0;
            unsigned char c2 = buf[i + 1];
            if (!((0x40 <= c2 && c2 <= 0x7E) || (0x80 <= c2 && c2 <= 0xFC))) {
                return 0;
            }
            i += 2;
        }
        else {
            return 0;
        }
    }
    return 1;
}

int is_binary(const char *path) {
    FILE *fp = fopen(path, "rb");
    if (!fp) return -1;

    unsigned char *buf = malloc(BINARY_CHECK_BYTES);
    if (!buf) {
        fclose(fp);
        return -1;
    }

    size_t read_len = fread(buf, 1, BINARY_CHECK_BYTES, fp);
    fclose(fp);

    // UTF-8 Check
    if (is_valid_utf8(buf, read_len)) {
        free(buf);
        return 0; // Text file
    }

    // Shift-JIS Check
    if (is_valid_shift_jis(buf, read_len)) {
        free(buf);
        return 0; // Text file
    }

    // Calculate the percentage of hidden characters
    size_t non_printable = 0;
    for (size_t i = 0; i < read_len; i++) {
        if (buf[i] == 0) {
            free(buf);
            return 1; // Binary
        }
        if (!isprint(buf[i]) && buf[i] != '\n' && buf[i] != '\r' && buf[i] != '\t') {
            non_printable++;
        }
    }
    free(buf);

    if (read_len > 0 && ((double)non_printable / (double)read_len) > 0.25) {
        return 1; // Binary
    }

    return 0; // Binary
}