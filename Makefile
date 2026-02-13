# Makefile para Hulk Compiler

ifeq ($(OS),Windows_NT)
    EXE = .exe
else
    EXE =
endif

CLANG = clang
SCRIPT = script.hulk
BUILD_DIR = hulk

.PHONY: parse compile execute clean

parse:
	@if [ ! -f $(SCRIPT) ]; then echo "ERROR: Falta $(SCRIPT) en el directorio actual." && exit 1; fi
	@echo "Parseando y mostrando AST..."
	@cargo run -- $(SCRIPT)

compile: parse

execute: parse
	@echo "Nota: No hay generacion de LLVM/ejecutable aun. Solo se imprime el AST."

clean:
	@if [ -d $(BUILD_DIR) ]; then rm -rf $(BUILD_DIR); fi