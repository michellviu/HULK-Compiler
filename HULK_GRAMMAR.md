# Gramática de HULK

> Gramática libre de contexto para el lenguaje HULK, con **precedencia y asociatividad
> explícitas** en las expresiones. Diseñada para ser implementada directamente en un
> generador de parser LALR (lalrpop).

---

## 1. Terminales (Tokens)

```
NUMBER   := [0-9]+ ( '.' [0-9]+ )?
STRING   := '"' UNICODE* '"'            // Caracteres unicode válidos, con escapes \" \\
BOOLEAN  := "true" | "false"
ID       := [a-zA-Z_][a-zA-Z0-9_]*
```

### Palabras reservadas

```
let  in  if  elif  else  while  for  case  of
new  class  is  function  true  false
```

### Operadores y puntuación

```
+  -  *  /  %  ^                         // Aritméticos
==  !=  <  <=  >  >=                     // Comparación
&  |  !                                  // Lógicos
@  @@                                    // Concatenación de strings
:=                                       // Asignación destructiva
=                                        // Inicialización
->                                       // Flecha (cuerpo inline)
;  ,  :  .                               // Puntuación
(  )  {  }  [  ]                         // Agrupación
```

---

## 2. Programa (raíz)

```
<program> := <class>* <function>* [ <expr> ";" ]
```

Un programa es una secuencia de cero o más clases, seguida de cero o más funciones
globales, seguida opcionalmente de una expresión terminada en `;`.

---

## 3. Clases

```
<class> := "class" ID [ "(" <params> ")" ] [ "is" ID [ "(" <args> ")" ] ]
           "{" <attr>* <method>* "}"
```

### Parámetros (para constructores y funciones)

```
<params>  := <param> ( "," <param> )*
           | ε

<param>   := ID [ ":" ID ]
```

### Argumentos (expresiones pasadas a funciones/constructores)

```
<args>    := <expr> ( "," <expr> )*
           | ε
```

### Atributos

```
<attr>    := ID [ ":" ID ] "=" <expr> ";"
```

### Métodos

```
<method>  := ID "(" <params> ")" [ ":" ID ] <body>
```

---

## 4. Funciones globales

```
<function> := "function" ID "(" <params> ")" [ ":" ID ] <body>
```

---

## 5. Cuerpo (body) — compartido por funciones y métodos

```
<body> := "->" <expr> ";"
        | "{" ( <expr> ";" )+ "}"
```

### Cuerpo de expresión (para let, if, while, etc.)

```
<expr-body> := <expr>
             | "{" ( <expr> ";" )+ "}"
```

---

## 6. Expresiones — Jerarquía de precedencia (menor a mayor)

Las expresiones se descomponen en **niveles de precedencia** para eliminar la ambigüedad.
Cada nivel referencia al siguiente, garantizando que los operadores de menor precedencia
se evalúen últimos.

```
<expr> := <let-expr>
        | <if-expr>
        | <while-expr>
        | <case-expr>
        | <assign-expr>
```

> **Nota:** `let`, `if`, `while`, `case` y la asignación destructiva tienen la menor
> precedencia y no son asociativos (no se pueden anidar sin paréntesis como operandos
> de operadores binarios).

---

### 6.1. Expresión `let`

```
<let-expr> := "let" <decls> "in" <expr-body>

<decls>    := <decl> ( "," <decl> )*

<decl>     := ID [ ":" ID ] "=" <expr>
```

---

### 6.2. Expresión `if`

```
<if-expr> := "if" "(" <expr> ")" <expr-body>
             ( "elif" "(" <expr> ")" <expr-body> )*
             [ "else" <expr-body> ]
```

---

### 6.3. Expresión `while`

```
<while-expr> := "while" "(" <expr> ")" <expr-body>
                [ "else" <expr-body> ]
```

---

### 6.4. Expresión `case` (pattern matching por tipo)

```
<case-expr> := "case" <expr> "of" <case-branches>

<case-branches> := ID ":" ID "->" <expr-body>
                 | "{" ( ID ":" ID "->" <expr-body> ";" )* "}"
```

---

### 6.5. Asignación destructiva

```
<assign-expr> := <or-expr> ":=" <assign-expr>       // Asociativa a la derecha
               | <or-expr>
```

> La asignación es asociativa a la **derecha**: `a := b := 5` equivale a `a := (b := 5)`.

---

## 7. Expresiones elementales — Operadores por nivel de precedencia

Cada nivel es una producción que consume operadores de su nivel y delega al nivel
inmediatamente superior en precedencia. Todos los operadores binarios aritméticos,
lógicos y de comparación son **asociativos a la izquierda** salvo que se indique lo contrario.

```
Nivel   Operadores              Asociatividad   Nombre de producción
─────   ──────────              ─────────────   ────────────────────
  1     |                       izquierda       <or-expr>
  2     &                       izquierda       <and-expr>
  3     == !=                   izquierda       <equality-expr>
  4     < <= > >=               izquierda       <comparison-expr>
  5     @ @@                    izquierda       <concat-expr>
  6     + -                     izquierda       <additive-expr>
  7     * / %                   izquierda       <multiplicative-expr>
  8     ^                       derecha         <power-expr>
  9     - (unario)  !           —  (prefijo)    <unary-expr>
 10     . [] ()                 izquierda       <postfix-expr>
 11     new / literales / ID    —               <primary-expr>
```

---

### 7.1. OR lógico — Nivel 1 (menor precedencia entre elem)

```
<or-expr> := <or-expr> "|" <and-expr>
           | <and-expr>
```

### 7.2. AND lógico — Nivel 2

```
<and-expr> := <and-expr> "&" <equality-expr>
            | <equality-expr>
```

### 7.3. Igualdad — Nivel 3

```
<equality-expr> := <equality-expr> ( "==" | "!=" ) <comparison-expr>
                 | <comparison-expr>
```

### 7.4. Comparación — Nivel 4

```
<comparison-expr> := <comparison-expr> ( "<" | "<=" | ">" | ">=" ) <concat-expr>
                   | <concat-expr>
```

### 7.5. Concatenación de strings — Nivel 5

```
<concat-expr> := <concat-expr> ( "@" | "@@" ) <additive-expr>
               | <additive-expr>
```

### 7.6. Suma y resta — Nivel 6

```
<additive-expr> := <additive-expr> ( "+" | "-" ) <multiplicative-expr>
                 | <multiplicative-expr>
```

### 7.7. Multiplicación, división, módulo — Nivel 7

```
<multiplicative-expr> := <multiplicative-expr> ( "*" | "/" | "%" ) <power-expr>
                       | <power-expr>
```

### 7.8. Potencia — Nivel 8 (asociativa a la derecha)

```
<power-expr> := <unary-expr> "^" <power-expr>
              | <unary-expr>
```

> `2 ^ 3 ^ 2` se evalúa como `2 ^ (3 ^ 2) = 512`, no `(2 ^ 3) ^ 2 = 64`.

### 7.9. Operadores unarios prefijo — Nivel 9

```
<unary-expr> := "-" <unary-expr>
              | "!" <unary-expr>
              | <postfix-expr>
```

### 7.10. Operadores postfijo (acceso, indexación, llamada) — Nivel 10

```
<postfix-expr> := <postfix-expr> "." ID [ "(" <args> ")" ]     // Acceso a miembro / llamada a método
                | <postfix-expr> "[" <expr> "]"                 // Indexación de array
                | <primary-expr>
```

---

## 8. Expresiones primarias — Nivel 11

```
<primary-expr> := NUMBER
                | STRING
                | BOOLEAN
                | ID [ "(" <args> ")" ]       // Variable o llamada a función global
                | "(" <expr> ")"              // Expresión agrupada
                | <new-expr>                  // Instanciación o creación de array
```

### 8.1. Expresión `new` (instanciación y arrays)

```
<new-expr> := "new" ID "(" <args> ")"                              // Instanciación de clase
            | "new" [ ID ] "[" <expr> "]" [ "{" ID "->" <expr> "}" ] // Creación de array
```

---

## 9. Resumen — Tabla de precedencia completa

| Prec. | Operador(es)                          | Aridad   | Asociatividad |
|-------|---------------------------------------|----------|---------------|
| 1     | `let...in`, `if`, `while`, `case`     | especial | —             |
| 2     | `:=`                                  | binario  | derecha       |
| 3     | `\|`                                  | binario  | izquierda     |
| 4     | `&`                                   | binario  | izquierda     |
| 5     | `==`  `!=`                            | binario  | izquierda     |
| 6     | `<`  `<=`  `>`  `>=`                  | binario  | izquierda     |
| 7     | `@`  `@@`                             | binario  | izquierda     |
| 8     | `+`  `-`                              | binario  | izquierda     |
| 9     | `*`  `/`  `%`                         | binario  | izquierda     |
| 10    | `^`                                   | binario  | derecha       |
| 11    | `-` (unario)  `!`                     | prefijo  | —             |
| 12    | `.`  `[]`  `()`                       | postfijo | izquierda     |
| 13    | literales, `ID`, `(expr)`, `new`      | primario | —             |

---

## 10. Notas para la implementación en LALRPOP

### Mapeo de producciones a reglas lalrpop

Cada nivel de precedencia se convierte en una regla lalrpop independiente:

```
pub Expr       = { LetExpr, IfExpr, WhileExpr, CaseExpr, AssignExpr }
AssignExpr     = { <OrExpr> ":=" <AssignExpr>, OrExpr }
OrExpr         = { <OrExpr> "|" <AndExpr>, AndExpr }
AndExpr        = { <AndExpr> "&" <EqualityExpr>, EqualityExpr }
EqualityExpr   = { <EqualityExpr> EqOp <CompExpr>, CompExpr }
CompExpr       = { <CompExpr> CmpOp <ConcatExpr>, ConcatExpr }
ConcatExpr     = { <ConcatExpr> CatOp <AddExpr>, AddExpr }
AddExpr        = { <AddExpr> AddOp <MulExpr>, MulExpr }
MulExpr        = { <MulExpr> MulOp <PowExpr>, PowExpr }
PowExpr        = { <UnaryExpr> "^" <PowExpr>, UnaryExpr }
UnaryExpr      = { UnaryPrefixOp <UnaryExpr>, PostfixExpr }
PostfixExpr    = { <PostfixExpr> "." ID [ "(" Args ")" ],
                   <PostfixExpr> "[" Expr "]",
                   PrimaryExpr }
PrimaryExpr    = { NUMBER, STRING, BOOLEAN,
                   ID [ "(" Args ")" ],
                   "(" Expr ")",
                   NewExpr }
```

### Clases de nodos AST sugeridas

```
── Program
   ├── ClassDecl          // Declaración de clase
   │   ├── Param          // Parámetro con tipo opcional
   │   ├── Attribute       // Atributo con inicialización
   │   └── Method         // Método
   ├── FunctionDecl       // Declaración de función global
   └── Expression         // Nodo raíz de expresiones
       ├── LetExpr        // let ... in ...
       ├── IfExpr         // if / elif / else
       ├── WhileExpr      // while ... else ...
       ├── CaseExpr       // case ... of ...
       ├── AssignExpr     // loc := expr
       ├── BinaryOp       // operadores binarios (+, -, *, /, %, ^, ==, etc.)
       ├── UnaryOp        // operadores unarios (-, !)
       ├── MemberAccess   // expr.id
       ├── MethodCall     // expr.id(args)
       ├── IndexAccess    // expr[expr]
       ├── FunctionCall   // id(args)
       ├── NewInstance    // new Type(args)
       ├── NewArray       // new Type?[size] { init? }
       └── Atom           // Literales (Number, String, Bool) e Identificadores
```

---

## 11. Ejemplo completo

```hulk
class Point(x: Number, y: Number) {
    x: Number = x;
    y: Number = y;

    translate(dx: Number, dy: Number): Point -> new Point(x + dx, y + dy);

    norm(): Number {
        (x ^ 2 + y ^ 2) ^ 0.5;
    }
}

function greet(name: String): String -> "Hello " @@ name;

let p: Point = new Point(3, 4),
    msg: String = greet("world")
in {
    if (p.norm() > 5) p.translate(1, 1)
    elif (p.norm() == 5) p
    else new Point(0, 0);
};
```

