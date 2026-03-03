# Makefile para Hulk Compiler

ifeq ($(OS),Windows_NT)
    EXE = .exe
else
    EXE =
endif

CLANG = clang
SCRIPT ?= script.hulk
BUILD_DIR = hulk

.PHONY: build parse test compile execute clean

build:
	@echo "Compilando el proyecto..."
	@cargo build

parse:
	@if [ ! -f $(SCRIPT) ]; then echo "ERROR: Falta $(SCRIPT) en el directorio actual." && exit 1; fi
	@echo "Parseando '$(SCRIPT)' y mostrando AST..."
	@cargo run -- $(SCRIPT)

test:
	@echo "Ejecutando tests..."
	@cargo test

compile: parse

execute: parse
	@echo "Nota: No hay generacion de LLVM/ejecutable aun. Solo se imprime el AST."

clean:
	@echo "Limpiando artefactos de build..."
	@cargo clean
	@if [ -d $(BUILD_DIR) ]; then rm -rf $(BUILD_DIR); fi