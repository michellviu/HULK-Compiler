//! Semantic analysis module for the HULK compiler.
//!
//! This module implements three analysis passes (all run after parsing):
//!
//! 1. **Collector** ([`collector`]) — registers all top-level class and
//!    function declarations in the symbol table.
//! 2. **Semantic checker** ([`semantic_checker`]) — validates scoping,
//!    name resolution, arity, and structural rules.
//! 3. **Type checker** ([`type_checker`]) — infers expression types and
//!    verifies type conformance across the entire program.

pub mod types;
pub mod errors;
pub mod symbol_table;
pub mod collector;
pub mod semantic_checker;
pub mod type_checker;

pub use types::HulkType;
pub use errors::{CompilerError, format_compiler_error, Severity};
pub use symbol_table::SymbolTable;

use crate::ast;

/// Result of running all semantic analysis passes.
pub struct AnalysisResult {
    /// All errors and warnings collected across all passes.
    pub diagnostics: Vec<CompilerError>,
    /// The populated symbol table (available even if there are errors).
    pub symbols: SymbolTable,
}

impl AnalysisResult {
    /// Returns `true` if there are no errors (warnings are OK).
    pub fn is_ok(&self) -> bool {
        !self.diagnostics.iter().any(|d| d.severity == Severity::Error)
    }

    /// Returns only the errors (not warnings).
    pub fn errors(&self) -> Vec<&CompilerError> {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .collect()
    }

    /// Returns only the warnings.
    pub fn warnings(&self) -> Vec<&CompilerError> {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Warning)
            .collect()
    }
}

/// Runs the full semantic analysis pipeline on a parsed program.
///
/// Pipeline:
/// 1. Collect declarations → populate symbol table
/// 2. Semantic check → validate scoping, resolution, arity
/// 3. Type check → infer and verify types
///
/// Each pass accumulates errors independently.  If pass 1 produces
/// errors, passes 2 and 3 are still run (for maximum error reporting),
/// but the final result will contain all accumulated diagnostics.
pub fn analyze(program: &ast::Program) -> AnalysisResult {
    let mut symbols = SymbolTable::new();
    let mut diagnostics = Vec::new();

    // Pass 1: Collect declarations.
    let collect_errors = collector::collect_declarations(program, &mut symbols);
    let has_collect_errors = collect_errors.iter().any(|e| e.severity == Severity::Error);
    diagnostics.extend(collect_errors);

    // Pass 2: Semantic check (only if collection had no fatal errors).
    if !has_collect_errors {
        let sem_errors = semantic_checker::check_semantics(program, &mut symbols);
        diagnostics.extend(sem_errors);

        // Pass 3: Type check — always run even if there are semantic errors,
        // so that type mismatches (e.g. `let x: Number = "hello"`) are
        // reported alongside resolution errors.
        let type_errors = type_checker::check_types(program, &mut symbols);
        diagnostics.extend(type_errors);
    }

    AnalysisResult {
        diagnostics,
        symbols,
    }
}
