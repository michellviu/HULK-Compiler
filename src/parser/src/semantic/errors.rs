use std::fmt;
use crate::tokens::Span;

/// Severity level for compiler diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

/// A unified compiler error covering all post-parse phases.
#[derive(Debug, Clone)]
pub struct CompilerError {
    /// Unique error code (e.g. "E100" for semantic, "E200" for type errors).
    pub code: &'static str,
    /// Human-readable error message.
    pub message: String,
    /// Source span where the error occurred.
    pub span: Span,
    /// Severity level.
    pub severity: Severity,
    /// Optional hint/suggestion for fixing the error.
    pub hint: Option<String>,
}

impl CompilerError {
    // ── Semantic errors (E1xx) ───────────────────────────────────

    pub fn undefined_variable(name: &str, span: Span) -> Self {
        CompilerError {
            code: "E100",
            message: format!("Variable '{}' no está definida", name),
            span,
            severity: Severity::Error,
            hint: Some("Verifique que la variable esté declarada antes de usarla.".into()),
        }
    }

    pub fn undefined_function(name: &str, span: Span) -> Self {
        CompilerError {
            code: "E101",
            message: format!("Función '{}' no está definida", name),
            span,
            severity: Severity::Error,
            hint: None,
        }
    }

    pub fn undefined_type(name: &str, span: Span) -> Self {
        CompilerError {
            code: "E102",
            message: format!("Tipo '{}' no está definido", name),
            span,
            severity: Severity::Error,
            hint: None,
        }
    }

    pub fn duplicate_definition(kind: &str, name: &str, span: Span) -> Self {
        CompilerError {
            code: "E103",
            message: format!("{} '{}' ya fue definida anteriormente", kind, name),
            span,
            severity: Severity::Error,
            hint: None,
        }
    }

    pub fn wrong_arity(name: &str, expected: usize, got: usize, span: Span) -> Self {
        CompilerError {
            code: "E104",
            message: format!(
                "'{}' espera {} argumento(s), pero se proporcionaron {}",
                name, expected, got
            ),
            span,
            severity: Severity::Error,
            hint: None,
        }
    }

    pub fn invalid_assign_target(span: Span) -> Self {
        CompilerError {
            code: "E105",
            message: "El lado izquierdo de ':=' debe ser una variable, atributo o indexación".into(),
            span,
            severity: Severity::Error,
            hint: None,
        }
    }

    pub fn self_outside_class(span: Span) -> Self {
        CompilerError {
            code: "E106",
            message: "'self' solo puede usarse dentro de un método de clase".into(),
            span,
            severity: Severity::Error,
            hint: None,
        }
    }

    pub fn cyclic_inheritance(name: &str, span: Span) -> Self {
        CompilerError {
            code: "E107",
            message: format!("Herencia circular detectada en la clase '{}'", name),
            span,
            severity: Severity::Error,
            hint: None,
        }
    }

    pub fn undefined_member(class_name: &str, member: &str, span: Span) -> Self {
        CompilerError {
            code: "E108",
            message: format!("El tipo '{}' no tiene un miembro '{}'", class_name, member),
            span,
            severity: Severity::Error,
            hint: None,
        }
    }

    pub fn undefined_method(class_name: &str, method: &str, span: Span) -> Self {
        CompilerError {
            code: "E109",
            message: format!("El tipo '{}' no tiene un método '{}'", class_name, method),
            span,
            severity: Severity::Error,
            hint: None,
        }
    }

    // ── Type errors (E2xx) ──────────────────────────────────────

    pub fn type_mismatch(expected: &str, got: &str, span: Span) -> Self {
        CompilerError {
            code: "E200",
            message: format!("Se esperaba tipo '{}', pero se obtuvo '{}'", expected, got),
            span,
            severity: Severity::Error,
            hint: None,
        }
    }

    pub fn binary_op_type_error(op: &str, left: &str, right: &str, span: Span) -> Self {
        CompilerError {
            code: "E201",
            message: format!(
                "Operador '{}' no se puede aplicar a tipos '{}' y '{}'",
                op, left, right
            ),
            span,
            severity: Severity::Error,
            hint: None,
        }
    }

    pub fn unary_op_type_error(op: &str, operand: &str, span: Span) -> Self {
        CompilerError {
            code: "E202",
            message: format!(
                "Operador unario '{}' no se puede aplicar al tipo '{}'",
                op, operand
            ),
            span,
            severity: Severity::Error,
            hint: None,
        }
    }

    pub fn condition_not_boolean(span: Span) -> Self {
        CompilerError {
            code: "E203",
            message: "La condición debe ser de tipo 'Boolean'".into(),
            span,
            severity: Severity::Error,
            hint: None,
        }
    }

    pub fn not_indexable(type_name: &str, span: Span) -> Self {
        CompilerError {
            code: "E204",
            message: format!("El tipo '{}' no es indexable", type_name),
            span,
            severity: Severity::Error,
            hint: Some("Solo los arrays pueden ser indexados con [].".into()),
        }
    }

    pub fn index_not_number(span: Span) -> Self {
        CompilerError {
            code: "E205",
            message: "El índice de un array debe ser de tipo 'Number'".into(),
            span,
            severity: Severity::Error,
            hint: None,
        }
    }

    pub fn not_callable(name: &str, span: Span) -> Self {
        CompilerError {
            code: "E206",
            message: format!("'{}' no es una función invocable", name),
            span,
            severity: Severity::Error,
            hint: None,
        }
    }

    pub fn return_type_mismatch(func: &str, expected: &str, got: &str, span: Span) -> Self {
        CompilerError {
            code: "E207",
            message: format!(
                "La función '{}' debe retornar '{}', pero retorna '{}'",
                func, expected, got
            ),
            span,
            severity: Severity::Error,
            hint: None,
        }
    }

    pub fn type_does_not_conform(got: &str, expected: &str, span: Span) -> Self {
        CompilerError {
            code: "E208",
            message: format!(
                "El tipo '{}' no conforma al tipo esperado '{}'",
                got, expected
            ),
            span,
            severity: Severity::Error,
            hint: None,
        }
    }

    // ── Warnings (W0xx) ─────────────────────────────────────────

    pub fn unused_variable(name: &str, span: Span) -> Self {
        CompilerError {
            code: "W001",
            message: format!("Variable '{}' declarada pero nunca usada", name),
            span,
            severity: Severity::Warning,
            hint: Some("Si es intencional, prefije el nombre con '_'.".into()),
        }
    }
}

impl fmt::Display for CompilerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let prefix = match self.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
        };
        write!(f, "{}[{}]: {}", prefix, self.code, self.message)
    }
}

// ── Colored diagnostic formatting ───────────────────────────────────

const RED: &str = "\x1b[1;31m";
const YELLOW: &str = "\x1b[1;33m";
const CYAN: &str = "\x1b[1;36m";
const BLUE: &str = "\x1b[1;34m";
const BOLD: &str = "\x1b[1m";
const RESET: &str = "\x1b[0m";

/// Converts a byte offset to (line, column), both 1-based.
fn offset_to_line_col(source: &str, offset: usize) -> (usize, usize) {
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
    (line, col)
}

/// Formats a `CompilerError` as a rich terminal diagnostic.
pub fn format_compiler_error(err: &CompilerError, source: &str, filename: &str) -> String {
    let (line, col) = offset_to_line_col(source, err.span.start);
    let source_line = source.lines().nth(line - 1).unwrap_or("");
    let line_num_width = format!("{}", line).len().max(3);

    let color = match err.severity {
        Severity::Error => RED,
        Severity::Warning => YELLOW,
    };
    let prefix = match err.severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
    };

    let mut out = String::new();

    // Header
    out.push_str(&format!(
        "{color}{prefix}[{}]{RESET}: {BOLD}{}{RESET}\n",
        err.code, err.message
    ));

    // Location
    out.push_str(&format!(
        "  {BLUE}-->{RESET} {}:{}:{}\n",
        filename, line, col
    ));

    // Separator
    out.push_str(&format!(
        "  {BLUE}{:>width$} |{RESET}\n",
        "",
        width = line_num_width
    ));

    // Source line
    out.push_str(&format!(
        "  {BLUE}{:>width$} |{RESET} {}\n",
        line,
        source_line,
        width = line_num_width
    ));

    // Underline
    let span_len = (err.span.end - err.span.start).max(1);
    let padding = " ".repeat(col.saturating_sub(1));
    let underline = "^".repeat(span_len.min(source_line.len().saturating_sub(col - 1)).max(1));
    out.push_str(&format!(
        "  {BLUE}{:>width$} |{RESET} {padding}{color}{underline}{RESET}\n",
        "",
        width = line_num_width
    ));

    // Hint
    if let Some(ref hint) = err.hint {
        out.push_str(&format!(
            "\n  {CYAN}ayuda:{RESET} {hint}\n"
        ));
    }

    out
}
