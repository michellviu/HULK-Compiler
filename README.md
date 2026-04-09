# HULK-Compiler

Compilador para el lenguaje HULK, implementado en Rust con [LALRPOP](https://github.com/lalrpop/lalrpop).

## Estructura del proyecto

```
HULK-Compiler/
├── Cargo.toml              # Workspace raíz
├── Makefile                # Comandos de construcción y pruebas
├── HULK_GRAMMAR.md         # Gramática de referencia
├── script.hulk             # Archivo de prueba para el parser
└── src/
    ├── main.rs             # Punto de entrada: lee script.hulk y muestra el AST
    └── parser/             # Crate del parser (LALRPOP)
        ├── build.rs        # Script de build que invoca lalrpop
        ├── Cargo.toml
        └── src/
            ├── grammar.lalrpop   # Gramática LALRPOP (producciones)
            ├── lib.rs            # API pública del parser
            ├── ast/              # Nodos del AST
            │   ├── atoms/        # Atom, Group
            │   ├── expressions/  # Expression, BinaryOp, UnaryOp
            │   └── visitor/      # Trait Visitor + AstPrinterVisitor
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

## Cómo probar el parser y el AST printer — Paso a paso

### Paso 1: Editar `script.hulk`

Escribe una expresión válida en el archivo `script.hulk` en la raíz del proyecto.
La gramática actualmente soporta expresiones aritméticas, booleanas, strings,
operadores unarios y paréntesis.

Ejemplos válidos:

```hulk
print("Hello World!");
```

### Paso 2: Compilar y ejecutar con el Makefile

```bash
# Compilar el proyecto (sin ejecutar)
make compile

# Parsear script.hulk y mostrar el AST por consola
make parse

# Parsear un archivo diferente
make parse SCRIPT=otro_archivo.hulk
```

### Paso 3: Interpretar la salida

La salida es un árbol AST indentado. Por ejemplo, para `2+3*(4-5)/6`:

```
BinaryOp: +
  NumberLiteral: 2
  BinaryOp: /
    BinaryOp: *
      NumberLiteral: 3
      Group:
        BinaryOp: -
          NumberLiteral: 4
          NumberLiteral: 5
    NumberLiteral: 6
```

---

## Targets del Makefile

| Target            | Descripción                                              |
|-------------------|----------------------------------------------------------|
| `make build`      | Compila el proyecto sin ejecutar                         |
| `make parse`      | Parsea `script.hulk` (o `SCRIPT=...`) y muestra el AST  |
| `make test`       | Ejecuta los tests del workspace                          |
| `make clean`      | Limpia artefactos de build (cargo clean)                 |

---

## Flujo de desarrollo para agregar nuevas producciones

1. **Consulta** `HULK_GRAMMAR.md` para la producción que quieres implementar.
2. **Agrega los nodos AST** necesarios en `src/parser/src/ast/` (atoms o expressions).
3. **Agrega los tokens** necesarios en `src/parser/src/tokens/` si hacen falta.
4. **Agrega las reglas** en `src/parser/src/grammar.lalrpop`.
5. **Actualiza el Visitor** en `src/parser/src/ast/visitor/visitor.rs` y el printer.
6. **Compila** con `make build` — LALRPOP generará el parser automáticamente.
7. **Prueba** editando `script.hulk` y ejecutando `make parse`.
