# Makefile para Hulk Compiler

ifeq ($(OS),Windows_NT)
    EXE = .exe
else
    EXE =
endif

CLANG = clang
SCRIPT = script.hulk
BUILD_DIR = hulk

.PHONY: compile execute clean

compile:
	@if [ ! -f $(SCRIPT) ]; then echo "ERROR: Falta $(SCRIPT) en el directorio actual." && exit 1; fi
	@mkdir -p $(BUILD_DIR)
	@echo "Compilando script.hulk..."
	@cargo run -- $(SCRIPT)

execute: clean compile
	@echo "Ejecutando compilador Hulk..."
	@echo "Generando ejecutable con clang..."
	@$(CLANG) hulk/script.ll -o hulk/script$(EXE)
	@$(if $(findstring Windows_NT,$(OS)), \
		$(BUILD_DIR)\script.exe $(SCRIPT), \
		$(BUILD_DIR)/script $(SCRIPT))

clean:
	@if [ -d $(BUILD_DIR) ]; then rm -rf $(BUILD_DIR); fi