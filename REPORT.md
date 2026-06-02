# Informe Técnico de Arquitectura, Especificación e Implementación de HULK-Compiler

## Contexto y Fundamentación del Lenguaje HULK

Havana University Language for Kompilers (HULK) es un lenguaje de programación diseñado con fines didácticos en el ámbito de la Ciencia de la Computación. Este lenguaje unifica características de múltiples paradigmas de programación, presentándose como una herramienta orientada a objetos, estáticamente tipada, expresiva y segura. A pesar de su carácter pedagógico, HULK incorpora una serie de abstracciones complejas, tales como herencia de clases, despacho dinámico de métodos, polimorfismo, inferencia automática de tipos y expresiones avanzadas de control de flujo.

La implementación del compilador objeto de este informe se desarrolla en el lenguaje de programación **Rust** y adopta como estrategia de compilación la traducción dirigida por la sintaxis hacia la Representación Intermedia de LLVM (**LLVM IR**), delegando las optimizaciones de bajo nivel y la generación de código máquina al framework LLVM. Para soportar características integradas y operaciones básicas, el compilador enlaza el ejecutable resultante con un **runtime en lenguaje C**.

El compilador está estructurado en una tubería secuencial y modular donde cada etapa procesa la salida de la anterior: Análisis Léxico y Sintáctico, Análisis Semántico (en múltiples pasadas) y Generación de Código.

---

## Análisis Léxico y Sintáctico

La fase de frontend del compilador (Lexer y Parser) se implementa utilizando **LALRPOP**, un potente generador de analizadores sintácticos para Rust. Esta decisión de diseño permite definir gramáticas limpias y mantener el código libre de la complejidad del manejo manual de estados.

### Análisis Léxico (Lexer)

El análisis léxico en HULK-Compiler aprovecha el motor de expresiones regulares integrado de LALRPOP.

En el archivo de gramática (`grammar.lalrpop`), se define un bloque `match` que establece rigurosamente la prioridad de reconocimiento de tokens, resolviendo el problema de emparejamiento de prefijos:

1. **Palabras clave**: Tienen la máxima prioridad (ej. `let`, `if`, `while`, `function`, `type`, `is`, `as`, `true`, `false`).
2. **Operadores compuestos**: Se evalúan antes que los simples para evitar ambigüedades (ej. `:=`, `==`, `<=`, `=>`, `@@`).
3. **Operadores simples y puntuación**: Operadores matemáticos (`+`, `-`, `*`), lógicos (`&`, `|`), relacionales y símbolos de puntuación.
4. **Expresiones regulares (Regex)**: Se utilizan para capturar componentes variables:
   - Numéricos: `r"[0-9]+(\.[0-9]+)?"` (asignado a `NUMBER`).
   - Cadenas: `r#""([^"\\]|\\.)*""#` (asignado a `STRING`).
   - Identificadores: `r"[a-zA-Z][a-zA-Z0-9_]*"` (asignado a `ID`).
5. **Espacios en blanco**: Se definen para ser ignorados automáticamente (`r"\s+" => {}`).

LALRPOP genera un lexer eficiente subyacente que tokeniza el código fuente y provee de forma transparente la ubicación de cada token, facilitando reportes de error precisos.

### Análisis Sintáctico (Parser) LALR(1)

El analizador sintáctico construido por LALRPOP es de tipo **LALR(1)** (Look-Ahead Left-to-Right, con un token de prebúsqueda). Este enfoque ascendente (bottom-up) es ideal para manejar la rica sintaxis orientada a expresiones de HULK.

La gramática de HULK define **15 niveles explícitos de precedencia** para los operadores, resolviendo los conflictos Shift/Reduce clásicos al estructurar las expresiones de menor a mayor precedencia, garantizando la asociatividad correcta:

- **Nivel 1-2**: `let`, `if`, `while`, `for`, `case` y asignación destructiva (`:=` asociativa a la derecha).
- **Nivel 3-6**: Operadores lógicos y de comparación (`|`, `&`, `==`, `!=`, `<`, etc.).
- **Nivel 7-8**: Pruebas de tipo (`is`) y casting (`as`).
- **Nivel 9**: Concatenación (`@`, `@@`).
- **Nivel 10-11**: Aritmética aditiva y multiplicativa (`+`, `-`, `*`, `/`, `%`).
- **Nivel 12**: Potencia (`^` asociativa a la derecha).
- **Nivel 13**: Operadores unarios prefijo (`-`, `!`).
- **Nivel 14**: Operadores postfijos (acceso a miembros `.`, llamadas a métodos, indexación `[]`).
- **Nivel 15**: Expresiones primarias (literales, variables, llamadas a funciones, instanciación con `new`, y agrupaciones con paréntesis).

Para resolver ambigüedades estructurales como el *dangling-else*, la gramática de HULK define cuerpos de bloques (`ExprBody`) y obliga a la construcción jerárquica explícita de expresiones. Si el parser detecta un error de sintaxis, genera una estructura de error enriquecida (`SyntaxError`) indicando la línea, columna y los tokens esperados, permitiendo una retroalimentación clara al usuario.

---

## Análisis Semántico y Verificación de Tipos

Una vez construido el Árbol de Sintaxis Abstracta (AST), el compilador inicia la fase semántica. Dado que HULK permite referencias cruzadas entre funciones y clases sin requerir un orden estricto de declaración, el análisis semántico se ejecuta en **cuatro pasadas secuenciales**.

La información se centraliza en una **Tabla de Símbolos (`SymbolTable`)**, la cual almacena información de funciones globales, clases (con su jerarquía, atributos y métodos), y mantiene una pila de ámbitos léxicos (scopes) para la resolución local de variables.

### Pasada 1: Colector de Declaraciones (`collector.rs`)

El colector recorre el AST e inscribe en la tabla de símbolos todas las definiciones a nivel superior:
- Verifica colisiones de nombres (clases y funciones globales duplicadas).
- Registra firmas de constructores, métodos y funciones, junto con sus tipos explícitos (si están definidos).
- Una vez procesadas todas las clases, valida la jerarquía de herencia: verifica que las clases padres existan y emplea una búsqueda en grafo para detectar y rechazar **ciclos de herencia**.
- Resuelve los parámetros efectivos de los constructores (manejando la herencia implícita de constructores base cuando aplica).

### Pasada 2: Verificador Semántico (`semantic_checker.rs`)

Con todas las firmas reconocidas, esta pasada se adentra en el cuerpo de funciones y métodos para validar la integridad léxica y estructural:
- **Validación de Scopes:** Asegura que toda variable referenciada haya sido declarada previamente.
- **Validación de Invocaciones:** Comprueba que cualquier llamada a función, método o constructor referencie un símbolo válido y coincida en la aridad (cantidad de argumentos).
- **Consistencia de Clases:** Verifica que la palabra clave `self` se utilice exclusivamente dentro de métodos y valida las llamadas a constructores base (mediante la verificación de argumentos pasados al padre).
- **Verificación de Firmas Sobrescritas:** Garantiza que si una clase hija anula un método, su firma (tipos de parámetros y retorno) coincida exactamente con la de la clase padre.
- Genera advertencias útiles, como el reporte de variables declaradas que nunca son utilizadas.

### Pasada 3: Inferencia de Tipos (`type_inferer.rs`)

HULK permite la omisión de anotaciones de tipo (tipado implícito), dejando al compilador la tarea de deducir tipos estáticos seguros. El mecanismo implementado se basa en la **recopilación y unificación de restricciones por uso**.

1. **Recolección de Restricciones:** El inferidor explora las expresiones. Si una variable (o parámetro) sin tipo participa en una operación, se registra una restricción. Por ejemplo, si se encuentra `x + 5`, el sistema deduce que `x` debe ser de tipo `Number`. Si la variable se pasa a una función como `sin(x)`, se deduce el tipo esperado por la función.
2. **Resolución:** Se analizan todas las restricciones para un símbolo. Si todas coinciden, el tipo se asigna. Si existe un conflicto (ej., restringido a `Number` y `String` simultáneamente), el tipo permanece ambiguo y será detectado como un error en la siguiente fase. Si las restricciones apuntan a diferentes clases de la misma jerarquía, se calcula el Ancestro Común Más Bajo (LCA).
3. **Atributos y Let:** Los tipos de atributos de clases y declaraciones `let` no anotados se infieren evaluando directamente el tipo estático de su expresión de inicialización antes de la pasada completa de tipos.

### Pasada 4: Verificación de Tipos (`type_checker.rs`)

Es la validación final y más estricta. Utiliza los tipos inferidos y declarados para garantizar la seguridad total del programa:
- Atraviesa recursivamente cada nodo del AST, verificando que los operandos de cada operación (matemática, lógica o de comparación) tengan los tipos compatibles.
- Para las instrucciones de control (`if`, `while`), asegura que la condición sea estrictamente `Boolean`.
- Para estructuras con múltiples ramas (`if-elif-else`), determina el tipo resultante calculando recursivamente el Ancestro Común Más Bajo (LCA) entre los tipos devueltos por cada rama.
- En asignaciones y retornos de funciones, comprueba la relación de **subtipado**, garantizando que el tipo de la expresión evaluada sea compatible (o derivado) con el tipo esperado de la variable destino o la firma de la función.

Cualquier discrepancia de tipos en este punto genera un error, abortando la compilación antes de la generación de código.

---

## Generación de Código y Entorno de Ejecución

La etapa final traduce el AST semánticamente validado a código ejecutable. Para ello, utiliza la biblioteca **Inkwell**, un wrapper seguro de Rust sobre el API en C++ de **LLVM**. Esta decisión permite producir ejecutables nativos de alto rendimiento, delegando las optimizaciones nativas a LLVM.

### Flujo General de Emisión de LLVM IR

La generación de código se organiza como un recorrido del AST que produce **LLVM IR** de manera estructurada y determinista. La implementación se distribuye por módulos especializados en el directorio `codegen/`, lo cual permite separar responsabilidades y simplificar el mantenimiento:

- **`context.rs`** configura el `Context`, `Module` y `Builder` de LLVM, encapsulando utilidades comunes como creación de tipos LLVM, constantes y funciones auxiliares.
- **`types.rs`** define el mapeo entre tipos del lenguaje HULK y tipos LLVM, asegurando coherencia al generar firmas, valores literales y conversiones.
- **`functions.rs`** se encarga de registrar funciones y métodos, declarar prototipos y emitir el cuerpo de cada función.
- **`expressions.rs`** materializa la semántica de expresiones, operadores y literales, generando instrucciones LLVM correctas para cada nodo del AST.
- **`classes.rs`** gestiona la representación de objetos, construcción de instancias y el despacho dinámico.

En términos de flujo, el generador procede en dos niveles: primero declara las estructuras y prototipos necesarios (funciones, métodos, vtables, variables globales) y luego emite los cuerpos, garantizando que todo símbolo llamado exista previamente en el módulo LLVM. Esta estrategia evita referencias a símbolos inexistentes y facilita la resolución de llamadas recursivas.

### Emisión de Expresiones y Operadores

Las expresiones se traducen a una secuencia de instrucciones LLVM que preservan el orden de evaluación de HULK. El generador crea valores temporales cuando es necesario y usa instrucciones específicas según el tipo estático:

- **Aritmética y comparación:** Se emiten instrucciones enteras o de punto flotante según corresponda, asegurando que `+`, `-`, `*`, `/` y los comparadores estén tipados correctamente.
- **Operadores lógicos:** Se implementan con cortocircuito cuando aplica, construyendo bloques de control para evaluar solo lo necesario.
- **Conversión de tipos:** Cuando el AST requiere cast o coerciones seguras, se insertan conversiones LLVM explícitas, evitando mezclar representaciones incompatibles.
- **Literales y constantes:** Se materializan como constantes LLVM del tipo apropiado, lo que permite optimizaciones tempranas por parte de LLVM.

### Control de Flujo y Bloques Básicos

Las construcciones de control (`if`, `while`, `for`) se traducen a **bloques básicos** conectados por ramas condicionales. Cada bloque termina en una instrucción de salto (`br`) o retorno (`ret`), y el generador usa nodos `phi` cuando es necesario para consolidar valores provenientes de ramas distintas (por ejemplo, el valor resultante de un `if` con expresión en cada rama).

Esta estrategia asegura que el flujo de control sea explícito y correcto en LLVM IR, manteniendo la semántica del lenguaje y permitiendo optimizaciones del backend.

### Funciones, Llamadas y Métodos

Las funciones globales y métodos de clase se emiten como funciones LLVM con una convención de llamada uniforme. Para los métodos, el primer parámetro suele ser el receptor (`self`), lo que permite implementar el acceso a atributos y la invocación de otros métodos de manera consistente.

El sistema distingue entre llamadas directas (funciones globales o métodos conocidos estáticamente) y llamadas indirectas (despacho dinámico mediante vtables). En el segundo caso, el puntero a función se obtiene desde la tabla virtual y se invoca mediante una llamada indirecta, respetando el índice de método precomputado.

### Manejo de Tipos y Representación en Memoria

Para mantener la seguridad de tipos durante la generación de IR, cada valor en LLVM posee una representación fija y coherente con la tabla de símbolos. Esto incluye:

- **Layouts de objetos** con slots estables para `vtable`, `type_id` y atributos.
- **Acceso a campos** mediante offsets conocidos, calculados a partir del tipo de la clase.
- **Verificaciones dinámicas** para `is` y `as`, comparando el `type_id` cuando corresponde y fallando de forma segura si el cast no es válido.

Este diseño permite que la fase de generación de código sea un reflejo directo del modelo semántico del lenguaje.

### Estructuración de Objetos y Despacho Dinámico

Para mapear la orientación a objetos de HULK a los primitivos de bajo nivel de LLVM, el generador de código (`classes.rs`) implementa explícitamente el patrón de Tablas de Funciones Virtuales (**VTable**).

1. **Tipos Struct en LLVM:** Cada clase de HULK se traduce a un tipo opaco de estructura (`StructType`) en LLVM. El bloque de memoria de un objeto incluye:
   - Slot 0: Puntero a la VTable (despacho dinámico).
   - Slot 1: Un ID de tipo en tiempo de ejecución (`u64`) para chequeos rápidos de tipado y casteo dinámico (`is` / `as`).
   - Slot 2+: Los atributos de instancia, incorporando primero los heredados de clases base, manteniendo el alineamiento correcto para garantizar que un objeto hijo pueda ser tratado limpiamente como un puntero a un objeto padre.

2. **Tablas Virtuales (VTable):** Cada clase posee una variable global constante que almacena un array de punteros a función. Los métodos sobrescritos reemplazan a sus predecesores en el índice correspondiente, permitiendo que la invocación `obj.method()` se resuelva indirectamente en tiempo de ejecución obteniendo el puntero desde la tabla en base a un índice estático conocido en tiempo de compilación.

### El Runtime en C

El sistema se apoya en un código nativo compilado de forma transparente junto al programa: el **Runtime en C** (`hulk_runtime.c`). Durante la generación de IR, el compilador declara las firmas de estas funciones externas (`builtins.rs`) y las invoca cuando es necesario:

- **E/S y Utilidades:** Operaciones de impresión dinámica (`print`) se mapean a rutinas de formato de `stdio.h`.
- **Matemáticas y Aleatoriedad:** Invocaciones a `sin`, `cos`, `sqrt`, `exp` y `pow` se conectan directamente a `libm`, mientras que `rand()` se asocia al generador aleatorio.
- **Cadenas de Texto:** Operadores como `@` y `@@` generan llamadas a rutinas de asignación de memoria y concatenación dinámica (`hulk_concat`, `hulk_concat_spaced`), incluyendo utilidades para conversión de números y booleanos a cadenas.
- **Gestión de Memoria y Manejo de Errores:** La instanciación de objetos en HULK se traduce a invocaciones a `hulk_alloc` (un wrapper sobre `calloc`). Si se presenta un fallo irrecuperable en tiempo de ejecución (por ejemplo, fallo en un casteo dinámico `as`), se invoca `hulk_cast_error` para abortar la ejecución, la cual reporta la localización exacta (línea, columna y el fragmento del código fuente original) donde ocurrió el error.

### Tubería de Enlazado (Linking)

El proceso de compilación del compilador (definido en `main.rs` y `codegen/mod.rs`) finaliza el ciclo de traducción orquestando las herramientas del sistema operativo subyacente:
1. Verifica el módulo final de LLVM y opcionalmente escribe su código intermedio (`.ll`) a disco.
2. Utiliza el `TargetMachine` asociado a LLVM para emitir un archivo binario objeto nativo de la máquina (`.o`).
3. Invoca internamente al compilador de C del sistema base (por defecto `cc`) para procesar el entorno de ejecución `hulk_runtime.c`.
4. Ejecuta la herramienta de enlazado (`linker`), anexando el objeto del programa, el del runtime nativo y dependencias de la biblioteca estándar de C y de funciones matemáticas (`-lm`), produciendo por consiguiente el binario ejecutable óptimo.

---

## Conclusiones del Informe Técnico

La arquitectura y especificación del **HULK-Compiler** representan una solución ingenieril, moderna y fundamentada en buenas prácticas en la construcción de lenguajes de programación e ingeniería de software de bajo nivel. 

La integración del generador **LALRPOP** garantiza un pipeline de entrada eficiente, determinista, libre de ambigüedades lógicas, y preparado para proveer diagnósticos sintácticos humanizados y muy detallados al nivel de la línea de comandos. Separar el **Análisis Semántico en un ecosistema de cuatro pasadas consecutivas e independientes** (Recolección, Control Semántico, Inferencia de Restricciones y Validación de Tipos) viabiliza el soporte completo a constructos cíclicos en las declaraciones y permite disfrutar de una infraestructura de tipado implícito mediante ecuaciones lógicas de resolución unificada de tipos, un avance muy notable que mejora enormemente la experiencia del desarrollador HULK.

Finalmente, el acoplamiento final con el backend de **LLVM** por medio del puente `inkwell`, aunado a la gestión interna y estratégica de las Tablas de Métodos Virtuales (VTable) para el soporte algorítmico al paradigma orientado a objetos, permite disfrutar de los provechos de un compilador de grado industrial. El sistema delega la gestión de las pesadas y tediosas pasadas de optimización intermediales y enfocando el intelecto de la arquitectura en una traducción y validación de tipos óptima con un resultado compilado en binario nativo, asistido con destreza por un entorno de ejecución dinámico incorporado de lenguaje C.
