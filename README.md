# HULK-Compiler
Compiler for HULK language

## Probar parser y AST printer

1. Edita `script.hulk` con una expresion valida (por ejemplo: `2+3` o `-(1+2)`)
2. Ejecuta el parser y el AST printer con Makefile:

```bash
make parse
```

Si quieres, `make compile` y `make execute` tambien ejecutan el parseo y muestran el AST.
