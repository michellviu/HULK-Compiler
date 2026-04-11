/*
 * HULK Runtime Library
 *
 * Provides the built-in functions called from HULK-generated LLVM IR.
 * Compiled separately and linked with the generated object file.
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>
#include <time.h>

typedef struct {
    long long cursor;
    long long end;
} HulkRange;

/* ── Print functions ─────────────────────────────────────────────── */

void hulk_print_number(double value) {
    /* Print integers without decimal point */
    if (value == (long long)value && !isinf(value) && !isnan(value)) {
        printf("%lld\n", (long long)value);
    } else {
        printf("%g\n", value);
    }
}

void hulk_print_string(const char *value) {
    if (value) {
        printf("%s\n", value);
    } else {
        printf("(null)\n");
    }
}

void hulk_print_bool(int value) {
    printf("%s\n", value ? "true" : "false");
}

/* ── Math functions ──────────────────────────────────────────────── */
/* sin, cos, sqrt, exp are already in libm — we link against -lm.    */

double hulk_log(double base, double x) {
    return log(x) / log(base);
}

static int _rand_initialized = 0;

double hulk_rand(void) {
    if (!_rand_initialized) {
        srand((unsigned int)time(NULL));
        _rand_initialized = 1;
    }
    return (double)rand() / (double)RAND_MAX;
}

double hulk_pow(double base, double exponent) {
    return pow(base, exponent);
}

/* ── Range iterable ─────────────────────────────────────────────── */

void *hulk_range(double start, double end) {
    HulkRange *it = (HulkRange *)malloc(sizeof(HulkRange));
    if (!it) {
        fprintf(stderr, "HULK runtime error: out of memory\n");
        exit(1);
    }
    it->cursor = (long long)start;
    it->end = (long long)end;
    return (void *)it;
}

int hulk_range_next(void *iterable) {
    HulkRange *it = (HulkRange *)iterable;
    if (!it) {
        return 0;
    }
    if (it->cursor < it->end) {
        it->cursor += 1;
        return 1;
    }
    return 0;
}

double hulk_range_current(void *iterable) {
    HulkRange *it = (HulkRange *)iterable;
    if (!it) {
        return 0.0;
    }
    return (double)(it->cursor - 1);
}

/* ── String functions ────────────────────────────────────────────── */

char *hulk_concat(const char *a, const char *b) {
    if (!a) a = "";
    if (!b) b = "";
    size_t la = strlen(a);
    size_t lb = strlen(b);
    char *result = (char *)malloc(la + lb + 1);
    memcpy(result, a, la);
    memcpy(result + la, b, lb);
    result[la + lb] = '\0';
    return result;
}

char *hulk_concat_spaced(const char *a, const char *b) {
    if (!a) a = "";
    if (!b) b = "";
    size_t la = strlen(a);
    size_t lb = strlen(b);
    char *result = (char *)malloc(la + 1 + lb + 1);
    memcpy(result, a, la);
    result[la] = ' ';
    memcpy(result + la + 1, b, lb);
    result[la + 1 + lb] = '\0';
    return result;
}

/* ── Conversion functions ────────────────────────────────────────── */

char *hulk_number_to_string(double value) {
    char *buf = (char *)malloc(64);
    if (value == (long long)value && !isinf(value) && !isnan(value)) {
        snprintf(buf, 64, "%lld", (long long)value);
    } else {
        snprintf(buf, 64, "%g", value);
    }
    return buf;
}

char *hulk_bool_to_string(int value) {
    const char *s = value ? "true" : "false";
    char *buf = (char *)malloc(strlen(s) + 1);
    strcpy(buf, s);
    return buf;
}

/* ── Memory allocation ───────────────────────────────────────────── */

void *hulk_alloc(long long size) {
    void *ptr = calloc(1, (size_t)size);
    if (!ptr) {
        fprintf(stderr, "HULK runtime error: out of memory\n");
        exit(1);
    }
    return ptr;
}
