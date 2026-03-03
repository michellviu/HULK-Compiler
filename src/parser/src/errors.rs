use lalrpop_util::ParseError;
use lalrpop_util::lexer::Token;

/// Información de posición legible (línea y columna, 1-based).
#[derive(Debug, Clone)]
pub struct SourceLocation {
    pub line: usize,
    pub column: usize,
    pub offset: usize,
}

/// Representa un error sintáctico con toda la información necesaria para
/// generar un diagnóstico detallado.
#[derive(Debug, Clone)]
pub struct SyntaxError {
    /// Tipo de error
    pub kind: SyntaxErrorKind,
    /// Posición en el código fuente (línea, columna)
    pub location: SourceLocation,
    /// Línea de código fuente donde ocurrió el error
    pub source_line: String,
    /// Tokens que se esperaban
    pub expected: Vec<String>,
    /// Token que se encontró (si aplica)
    pub found: Option<String>,
}

#[derive(Debug, Clone)]
pub enum SyntaxErrorKind {
    /// Se encontró un token inesperado
    UnrecognizedToken,
    /// Se encontró un token que el lexer no reconoce
    InvalidToken,
    /// Se llegó al final del archivo inesperadamente
    UnexpectedEOF,
    /// Carácter extra no esperado
    ExtraToken,
}

impl std::fmt::Display for SyntaxErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyntaxErrorKind::UnrecognizedToken => write!(f, "Token inesperado"),
            SyntaxErrorKind::InvalidToken => write!(f, "Token inválido"),
            SyntaxErrorKind::UnexpectedEOF => write!(f, "Fin de archivo inesperado"),
            SyntaxErrorKind::ExtraToken => write!(f, "Token extra no esperado"),
        }
    }
}

/// Convierte un offset de bytes a línea y columna (1-based).
fn offset_to_location(source: &str, offset: usize) -> SourceLocation {
    let mut line = 1;
    let mut col = 1;
    for (i, ch) in source.char_indices() {
        if i >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    SourceLocation {
        line,
        column: col,
        offset,
    }
}

/// Obtiene la línea de código fuente dado un número de línea (1-based).
fn get_source_line(source: &str, line_number: usize) -> String {
    source
        .lines()
        .nth(line_number - 1)
        .unwrap_or("")
        .to_string()
}

/// Traduce los nombres internos de tokens a nombres legibles en español.
fn humanize_token(token: &str) -> String {
    match token.trim_matches('"') {
        ";" => "\";\" (punto y coma)".to_string(),
        ":" => "\":\" (dos puntos)".to_string(),
        "," => "\",\" (coma)".to_string(),
        "." => "\".\" (punto)".to_string(),
        "(" => "\"(\" (paréntesis de apertura)".to_string(),
        ")" => "\")\" (paréntesis de cierre)".to_string(),
        "{" => "\"{\" (llave de apertura)".to_string(),
        "}" => "\"}\" (llave de cierre)".to_string(),
        "[" => "\"[\" (corchete de apertura)".to_string(),
        "]" => "\"]\" (corchete de cierre)".to_string(),
        "=" => "\"=\" (igual)".to_string(),
        ":=" => "\":=\" (asignación destructiva)".to_string(),
        "->" => "\"->\" (flecha)".to_string(),
        "+" => "\"+\" (suma)".to_string(),
        "-" => "\"-\" (resta/negación)".to_string(),
        "*" => "\"*\" (multiplicación)".to_string(),
        "/" => "\"/\" (división)".to_string(),
        "%" => "\"%\" (módulo)".to_string(),
        "^" => "\"^\" (potencia)".to_string(),
        "==" => "\"==\" (igualdad)".to_string(),
        "!=" => "\"!=\" (desigualdad)".to_string(),
        "<" => "\"<\" (menor que)".to_string(),
        "<=" => "\"<=\" (menor o igual)".to_string(),
        ">" => "\">\" (mayor que)".to_string(),
        ">=" => "\">=\" (mayor o igual)".to_string(),
        "&" => "\"&\" (y lógico)".to_string(),
        "|" => "\"|\" (o lógico)".to_string(),
        "!" => "\"!\" (negación)".to_string(),
        "@" => "\"@\" (concatenación)".to_string(),
        "@@" => "\"@@\" (concatenación con espacio)".to_string(),
        "let" => "\"let\" (declaración de variable)".to_string(),
        "in" => "\"in\"".to_string(),
        "if" => "\"if\" (condicional)".to_string(),
        "elif" => "\"elif\" (condicional alternativa)".to_string(),
        "else" => "\"else\" (alternativa)".to_string(),
        "while" => "\"while\" (bucle)".to_string(),
        "for" => "\"for\" (bucle)".to_string(),
        "case" => "\"case\" (coincidencia de patrones)".to_string(),
        "of" => "\"of\"".to_string(),
        "new" => "\"new\" (instanciación)".to_string(),
        "class" => "\"class\" (declaración de clase)".to_string(),
        "is" => "\"is\" (herencia)".to_string(),
        "function" => "\"function\" (declaración de función)".to_string(),
        "true" => "\"true\" (literal booleano)".to_string(),
        "false" => "\"false\" (literal booleano)".to_string(),
        other => format!("\"{}\"", other),
    }
}

/// Genera una sugerencia contextual basada en el token encontrado y los esperados.
fn generate_hint(error: &SyntaxError) -> Option<String> {
    let expected_raw: Vec<String> = error
        .expected
        .iter()
        .map(|s| s.trim_matches('"').to_string())
        .collect();
    let found_raw = error.found.as_deref().unwrap_or("");

    // Falta punto y coma
    if expected_raw.contains(&";".to_string()) && (found_raw == "}" || found_raw == ")") {
        return Some("¿Falta un \";\" al final de la expresión anterior?".to_string());
    }

    // Falta paréntesis de apertura
    if found_raw == ":" && expected_raw.contains(&"(".to_string()) {
        return Some(
            "El tipo de retorno va después de los parámetros. \
             Sintaxis correcta: function nombre(params): tipo { ... }"
                .to_string(),
        );
    }

    // Falta cerrar paréntesis
    if expected_raw.contains(&")".to_string()) {
        return Some("¿Falta cerrar un paréntesis \")\"?".to_string());
    }

    // Falta cerrar llave
    if expected_raw.contains(&"}".to_string()) {
        return Some("¿Falta cerrar una llave \"}\"?".to_string());
    }

    // Falta 'else' en if
    if expected_raw.contains(&"else".to_string()) {
        return Some(
            "En HULK, las expresiones \"if\" requieren una rama \"else\" obligatoria."
                .to_string(),
        );
    }

    // Falta 'in' después de let
    if expected_raw.contains(&"in".to_string()) {
        return Some("¿Falta \"in\" después de la declaración \"let\"?".to_string());
    }

    // EOF inesperado
    if matches!(error.kind, SyntaxErrorKind::UnexpectedEOF) {
        return Some(
            "El archivo terminó antes de lo esperado. \
             Verifique que todas las llaves, paréntesis y expresiones estén completas."
                .to_string(),
        );
    }

    None
}

/// Agrupa los tokens esperados para mostrar un resumen legible.
fn format_expected(expected: &[String]) -> String {
    if expected.is_empty() {
        return "ningún token reconocido".to_string();
    }

    let humanized: Vec<String> = expected.iter().map(|t| humanize_token(t)).collect();

    if humanized.len() == 1 {
        return humanized[0].clone();
    }

    if humanized.len() <= 5 {
        let (init, last) = humanized.split_at(humanized.len() - 1);
        return format!("{} o {}", init.join(", "), last[0]);
    }

    // Más de 5: mostrar los primeros 5 y cuántos quedan
    let shown = &humanized[..5];
    let remaining = humanized.len() - 5;
    format!(
        "{}, ... (y {} opciones más)",
        shown.join(", "),
        remaining
    )
}

/// Convierte un `ParseError` de lalrpop en un `SyntaxError` detallado.
pub fn build_syntax_error<'input>(
    source: &str,
    err: &ParseError<usize, Token<'input>, &'static str>,
) -> SyntaxError {
    match err {
        ParseError::UnrecognizedToken {
            token: (start, token, _end),
            expected,
        } => {
            let location = offset_to_location(source, *start);
            let source_line = get_source_line(source, location.line);
            SyntaxError {
                kind: SyntaxErrorKind::UnrecognizedToken,
                location,
                source_line,
                expected: expected.clone(),
                found: Some(format!("{}", token)),
            }
        }
        ParseError::UnrecognizedEof { location, expected } => {
            let loc = offset_to_location(source, *location);
            let source_line = get_source_line(source, loc.line);
            SyntaxError {
                kind: SyntaxErrorKind::UnexpectedEOF,
                location: loc,
                source_line,
                expected: expected.clone(),
                found: None,
            }
        }
        ParseError::InvalidToken { location } => {
            let loc = offset_to_location(source, *location);
            let source_line = get_source_line(source, loc.line);
            SyntaxError {
                kind: SyntaxErrorKind::InvalidToken,
                location: loc,
                source_line,
                expected: vec![],
                found: None,
            }
        }
        ParseError::ExtraToken {
            token: (start, token, _end),
        } => {
            let location = offset_to_location(source, *start);
            let source_line = get_source_line(source, location.line);
            SyntaxError {
                kind: SyntaxErrorKind::ExtraToken,
                location,
                source_line,
                expected: vec![],
                found: Some(format!("{}", token)),
            }
        }
        ParseError::User { error } => {
            // Caso genérico para errores del usuario
            SyntaxError {
                kind: SyntaxErrorKind::InvalidToken,
                location: SourceLocation {
                    line: 1,
                    column: 1,
                    offset: 0,
                },
                source_line: source.lines().next().unwrap_or("").to_string(),
                expected: vec![],
                found: Some(error.to_string()),
            }
        }
    }
}

// ── Colores ANSI ────────────────────────────────────────────────────────────

const RED: &str = "\x1b[1;31m";
const YELLOW: &str = "\x1b[1;33m";
const CYAN: &str = "\x1b[1;36m";
const BLUE: &str = "\x1b[1;34m";
const GRAY: &str = "\x1b[90m";
const BOLD: &str = "\x1b[1m";
const RESET: &str = "\x1b[0m";

/// Formatea un `SyntaxError` como un diagnóstico legible para la terminal,
/// con colores ANSI, indicadores de posición, y sugerencias contextuales.
pub fn format_syntax_error(err: &SyntaxError, filename: &str) -> String {
    let mut out = String::new();

    // ── Encabezado ──────────────────────────────────────────────────────
    out.push_str(&format!(
        "{RED}error[E001]{RESET}: {BOLD}{}{RESET}\n",
        err.kind
    ));

    // ── Ubicación ───────────────────────────────────────────────────────
    out.push_str(&format!(
        "  {BLUE}-->{RESET} {}:{}:{}\n",
        filename, err.location.line, err.location.column
    ));

    // ── Contexto del código fuente ──────────────────────────────────────
    let line_num = err.location.line;
    let col = err.location.column;
    let line_num_width = format!("{}", line_num).len().max(3);

    // Línea separadora
    out.push_str(&format!(
        "  {BLUE}{:>width$} |{RESET}\n",
        "",
        width = line_num_width
    ));

    // Línea con el código fuente
    out.push_str(&format!(
        "  {BLUE}{:>width$} |{RESET} {}\n",
        line_num,
        err.source_line,
        width = line_num_width
    ));

    // Indicador que apunta al error
    let padding = " ".repeat(col.saturating_sub(1));
    let found_text = err
        .found
        .as_deref()
        .unwrap_or("EOF");

    // Determinar ancho del subrayado
    let underline_len = found_text.len().max(1);
    let underline = "^".repeat(underline_len);

    out.push_str(&format!(
        "  {BLUE}{:>width$} |{RESET} {padding}{RED}{underline}{RESET} {RED}{msg}{RESET}\n",
        "",
        width = line_num_width,
        msg = if err.found.is_some() {
            format!("Se encontró {} aquí", found_text)
        } else {
            "Fin de archivo inesperado aquí".to_string()
        },
    ));

    // ── Tokens esperados ────────────────────────────────────────────────
    if !err.expected.is_empty() {
        out.push_str(&format!(
            "  {BLUE}{:>width$} |{RESET}\n",
            "",
            width = line_num_width
        ));
        out.push_str(&format!(
            "  {BLUE}{:>width$} ={RESET} {CYAN}Se esperaba:{RESET} {}\n",
            "",
            format_expected(&err.expected),
            width = line_num_width
        ));
    }

    // ── Sugerencia ──────────────────────────────────────────────────────
    if let Some(hint) = generate_hint(err) {
        out.push_str(&format!(
            "\n  {YELLOW}ayuda:{RESET} {}{GRAY}{RESET}\n",
            hint
        ));
    }

    out
}

/// Función de conveniencia: parsea un error de lalrpop y devuelve el
/// diagnóstico formateado completo.
pub fn format_parse_error<'input>(
    source: &str,
    filename: &str,
    err: &ParseError<usize, Token<'input>, &'static str>,
) -> String {
    let syntax_error = build_syntax_error(source, err);
    format_syntax_error(&syntax_error, filename)
}

