# HULK-Compiler

Compilador para el lenguaje HULK, implementado en Rust con [LALRPOP](https://github.com/lalrpop/lalrpop).

## Estructura del proyecto

```
HULK-Compiler/
├── Cargo.toml              # Workspace raíz
├── Makefile                # Comandos de construcción y pruebas
├── HULK_GRAMMAR.md         # Gramática de referencia
├── script.hulk             # Script de entrada de ejemplo
├── runtime/
│   └── hulk_runtime.c      # Runtime en C enlazado al binario final
├── tests/
│   ├── test_class.hulk
│   ├── test_errors.hulk
│   └── test_full.hulk
└── src/
    ├── main.rs             # Pipeline: parseo, semántica y generación LLVM
    ├── codegen/            # Generación de IR LLVM y salida ejecutable
    └── parser/             # Crate del parser + análisis semántico
        ├── build.rs        # Script de build que invoca lalrpop
        ├── Cargo.toml
        └── src/
            ├── grammar.lalrpop   # Gramática LALRPOP (producciones)
            ├── lib.rs            # API pública del parser
            ├── ast/              # Nodos del AST
            │   ├── atoms/
            │   ├── declarations/
            │   ├── expressions/
            │   ├── program.rs
            │   └── visitor/      # Trait Visitor + AstPrinterVisitor
            ├── semantic/         # Tipado, chequeos y diagnósticos
            └── tokens/           # Tokens: Literal, Identifier, Operator, etc.
```

## Gramática de referencia

Consulta [HULK_GRAMMAR.md](./HULK_GRAMMAR.md) para la gramática completa desambiguada
con tabla de precedencia de operadores y clases de nodos AST.

---

## Requisitos

- **Rust** (edición 2024) — instalar con [rustup](https://rustup.rs/)
- **GNU Make** (opcional, para usar los targets del Makefile)

---

## Cómo ejecutar el compilador — Paso a paso

### Paso 1: Editar `script.hulk`

Escribe un programa en `script.hulk` en la raíz del proyecto.

Ejemplos válidos:

```hulk
print("Hello World!");
```

### Paso 2: Compilar y ejecutar con el Makefile

```bash
# Compilar el proyecto (sin ejecutar)
make compile

# Parsear y compilar script.hulk (genera artefactos en build/)
make parse

# Parsear un archivo diferente
make parse SCRIPT=otro_archivo.hulk

# Ejecutar el binario generado
make execute SCRIPT=otro_archivo.hulk
```

### Paso 3: Interpretar la salida

La salida muestra diagnósticos de compilación y el resultado final.

```
→ Generando código LLVM…
✓ Compilación exitosa → build/script
```

---

## Targets del Makefile

| Target            | Descripción                                              |
|-------------------|----------------------------------------------------------|
| `make build`      | Compila el proyecto sin ejecutar                         |
| `make compile`    | Alias de `make parse`                                    |
| `make parse`      | Parsea y compila `script.hulk` (o `SCRIPT=...`)         |
| `make execute`    | Ejecuta el binario generado en `build/`                 |
| `make test`       | Ejecuta los tests del workspace                          |
| `make clean`      | Limpia artefactos de build (cargo clean)                 |

---

## Flujo de desarrollo para agregar nuevas producciones

1. **Consulta** `HULK_GRAMMAR.md` para la producción que quieres implementar.
2. **Agrega los nodos AST** necesarios en `src/parser/src/ast/` (atoms o expressions).
3. **Agrega los tokens** necesarios en `src/parser/src/tokens/` si hacen falta.
4. **Agrega las reglas** en `src/parser/src/grammar.lalrpop`.
5. **Actualiza semántica y/o codegen** en `src/parser/src/semantic/` y `src/codegen/`.
6. **Compila** con `make build` — LALRPOP generará el parser automáticamente.
7. **Prueba** editando `script.hulk` y ejecutando `make parse` o `make execute`.
