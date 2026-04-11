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

/* ── Diagnostic formatting (runtime) ───────────────────────────── */

#define RED "\x1b[1;31m"
#define BLUE "\x1b[1;34m"
#define CYAN "\x1b[1;36m"
#define BOLD "\x1b[1m"
#define RESET "\x1b[0m"

static void hulk_runtime_diagnostic(
    const char *code,
    const char *message,
    const char *hint,
    const char *source_filename,
    int line,
    int col,
    const char *source_line,
    int marker_col,
    int marker_len
) {
    const char *safe_code = code ? code : "R000";
    const char *safe_message = message ? message : "Error de ejecucion desconocido";

    fprintf(stderr, RED "error[%s]" RESET ": " BOLD "%s" RESET "\n", safe_code, safe_message);

    if (source_filename && source_filename[0] != '\0' && line > 0 && col > 0) {
        int width = snprintf(NULL, 0, "%d", line);
        if (width < 3) {
            width = 3;
        }

        fprintf(stderr, "  " BLUE "-->" RESET " %s:%d:%d\n", source_filename, line, col);
        fprintf(stderr, "  " BLUE "%*s |" RESET "\n", width, "");

        if (!source_line) {
            source_line = "";
        }
        fprintf(stderr, "  " BLUE "%*d |" RESET " %s\n", width, line, source_line);

        if (marker_col < 1) {
            marker_col = 1;
        }
        if (marker_len < 1) {
            marker_len = 1;
        }

        fprintf(stderr, "  " BLUE "%*s |" RESET " ", width, "");
        for (int i = 1; i < marker_col; i++) {
            fputc(' ', stderr);
        }
        fprintf(stderr, RED);
        for (int i = 0; i < marker_len; i++) {
            fputc('^', stderr);
        }
        fprintf(stderr, RESET "\n");
    } else {
        fprintf(stderr, "  " BLUE "-->" RESET " runtime\n");
        fprintf(stderr, "  " BLUE "  |" RESET "\n");
        fprintf(stderr, "  " BLUE "  =" RESET " contexto: ejecucion del programa\n");
    }

    if (hint && hint[0] != '\0') {
        fprintf(stderr, "  " CYAN "ayuda:" RESET " %s\n", hint);
    }
}

static void hulk_runtime_abort(
    const char *code,
    const char *message,
    const char *hint,
    const char *source_filename,
    int line,
    int col,
    const char *source_line,
    int marker_col,
    int marker_len
) {
    hulk_runtime_diagnostic(
        code,
        message,
        hint,
        source_filename,
        line,
        col,
        source_line,
        marker_col,
        marker_len
    );
    exit(1);
}

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
        hulk_runtime_abort(
            "R001",
            "Memoria insuficiente al crear el iterador de rango",
            "Reduce el uso de memoria o simplifica la entrada del programa.",
            NULL,
            0,
            0,
            NULL,
            0,
            0
        );
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
        hulk_runtime_abort(
            "R001",
            "Memoria insuficiente al reservar memoria para un objeto",
            "Reduce el uso de memoria o simplifica la entrada del programa.",
            NULL,
            0,
            0,
            NULL,
            0,
            0
        );
    }
    return ptr;
}

/* ── Runtime type/cast errors ──────────────────────────────────── */

void hulk_cast_error(
    const char *message,
    const char *source_filename,
    int line,
    int col,
    const char *source_line,
    int marker_col,
    int marker_len
) {
    if (!message) {
        message = "Conversion de tipo invalida";
    }
    hulk_runtime_abort(
        "R002",
        message,
        "Verifique el tipo con 'is' antes de usar 'as'.",
        source_filename,
        line,
        col,
        source_line,
        marker_col,
        marker_len
    );
}
