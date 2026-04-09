# Makefile para Hulk Compiler

ifeq ($(OS),Windows_NT)
    EXE = .exe
else
    EXE =
endif

CLANG = clang
SCRIPT ?= script.hulk
BUILD_DIR = build
SCRIPT_NAME = $(notdir $(SCRIPT))
SCRIPT_STEM = $(basename $(SCRIPT_NAME))

.PHONY: build parse test compile execute clean prepare

build:
	@echo "Compilando el proyecto..."
	@cargo build

parse:
	@if [ ! -f $(SCRIPT) ]; then echo "ERROR: Falta $(SCRIPT) en el directorio actual." && exit 1; fi
	@$(MAKE) prepare
	@echo "Parseando '$(SCRIPT)' y generando artefactos en '$(BUILD_DIR)/'..."
	@cd $(BUILD_DIR) && cargo run --manifest-path ../Cargo.toml -- ../$(SCRIPT)

test:
	@echo "Ejecutando tests..."
	@cargo test

compile: parse

execute: parse
	@echo "Ejecutando './$(SCRIPT_STEM)$(EXE)' desde '$(BUILD_DIR)/'..."
	@cd $(BUILD_DIR) && ./$(SCRIPT_STEM)$(EXE)

prepare:
	@mkdir -p $(BUILD_DIR)

clean:
	@echo "Limpiando artefactos de build..."
	@if [ -d $(BUILD_DIR) ]; then rm -rf $(BUILD_DIR); fi