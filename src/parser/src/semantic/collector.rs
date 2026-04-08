//! Pass 1: Declaration collector.
//!
//! Walks the AST once and registers every top-level class and function
//! in the symbol table **without** analysing bodies.  This allows later
//! passes (semantic check, type check) to reference any symbol regardless
//! of declaration order.

use std::collections::{HashMap, HashSet};

use crate::ast::{self, Visitable, Visitor};
use crate::tokens;

use super::errors::CompilerError;
use super::symbol_table::{AttrInfo, ClassInfo, FuncInfo, SymbolTable};
use super::types::HulkType;

/// Collects all top-level declarations into the symbol table.
/// Returns accumulated errors (duplicate names, unknown parents, cycles).
pub fn collect_declarations(
    program: &ast::Program,
    symbols: &mut SymbolTable,
) -> Vec<CompilerError> {
    let mut collector = CollectorVisitor {
        symbols,
        errors: Vec::new(),
    };
    program.accept(&mut collector);

    // After collecting all classes, check for inheritance cycles.
    let cycle_errors = check_inheritance_cycles(&collector.symbols);
    collector.errors.extend(cycle_errors);

    collector.errors
}

// ═══════════════════════════════════════════════════════════════════
// Visitor implementation
// ═══════════════════════════════════════════════════════════════════

struct CollectorVisitor<'a> {
    symbols: &'a mut SymbolTable,
    errors: Vec<CompilerError>,
}

impl<'a> Visitor for CollectorVisitor<'a> {
    fn visit_program(&mut self, program: &ast::Program) {
        for class in &program.classes {
            class.accept(self);
        }
        for func in &program.functions {
            func.accept(self);
        }
        // Entry expression is analysed in later passes.
    }

    fn visit_class_decl(&mut self, class: &ast::ClassDecl) {
        // Check for duplicate class name.
        if self.symbols.classes.contains_key(&class.name) {
            self.errors.push(CompilerError::duplicate_definition(
                "Tipo",
                &class.name,
                class.span,
            ));
            return;
        }

        // Check that the parent type exists (if specified).
        if let Some(ref parent) = class.parent {
            if !self.symbols.type_exists(parent) && !is_forward_class(parent, &class.name) {
                // The parent might be declared later; we allow it for now
                // and validate after all classes are collected.
            }
        }

        // Resolve constructor parameter types.
        let params: Vec<(String, HulkType)> = class
            .params
            .iter()
            .map(|p| {
                let t = match &p.type_ann {
                    Some(ann) => HulkType::from_name(ann),
                    None => HulkType::Unknown,
                };
                (p.name.clone(), t)
            })
            .collect();

        // Collect explicitly declared attributes.
        let mut attributes: Vec<AttrInfo> = Vec::new();
        // When the initializer is a variable that matches a constructor param,
        // use the constructor param's type.
        for a in &class.attributes {
            let hulk_type = match &a.type_ann {
                Some(ann) => HulkType::from_name(ann),
                None => {
                    let inferred = infer_type_from_expr(&a.init);
                    if inferred == HulkType::Unknown {
                        // Try to match init variable to a constructor param.
                        if let ast::Expression::Atom(atom) = &a.init {
                            if let ast::atoms::atom::Atom::Variable(var) = atom.as_ref() {
                                params
                                    .iter()
                                    .find(|(n, _)| n == &var.name)
                                    .map(|(_, t)| t.clone())
                                    .unwrap_or(HulkType::Unknown)
                            } else {
                                HulkType::Unknown
                            }
                        } else {
                            HulkType::Unknown
                        }
                    } else {
                        inferred
                    }
                }
            };
            attributes.push(AttrInfo {
                name: a.name.clone(),
                hulk_type,
                span: a.span,
            });
        }

        // Collect methods (signatures only — bodies are not analysed here).
        let mut methods = HashMap::new();
        for m in &class.methods {
            let method_params: Vec<(String, HulkType)> = m
                .params
                .iter()
                .map(|p| {
                    let t = match &p.type_ann {
                        Some(ann) => HulkType::from_name(ann),
                        None => HulkType::Unknown,
                    };
                    (p.name.clone(), t)
                })
                .collect();
            let ret = match &m.return_type {
                Some(ann) => HulkType::from_name(ann),
                None => HulkType::Unknown,
            };

            if methods.contains_key(&m.name) {
                self.errors.push(CompilerError::duplicate_definition(
                    "Método",
                    &m.name,
                    m.span,
                ));
                continue;
            }
            methods.insert(
                m.name.clone(),
                FuncInfo {
                    name: m.name.clone(),
                    params: method_params,
                    return_type: ret,
                    span: m.span,
                },
            );
        }

        let parent = class.parent.clone().or_else(|| {
            // Every user-defined class implicitly inherits from Object.
            Some("Object".to_string())
        });

        self.symbols.classes.insert(
            class.name.clone(),
            ClassInfo {
                name: class.name.clone(),
                params,
                parent,
                attributes,
                methods,
                span: class.span,
            },
        );
    }

    fn visit_function_decl(&mut self, func: &ast::FunctionDecl) {
        if self.symbols.functions.contains_key(&func.name) {
            self.errors.push(CompilerError::duplicate_definition(
                "Función",
                &func.name,
                func.span,
            ));
            return;
        }

        let params: Vec<(String, HulkType)> = func
            .params
            .iter()
            .map(|p| {
                let t = match &p.type_ann {
                    Some(ann) => HulkType::from_name(ann),
                    None => HulkType::Unknown,
                };
                (p.name.clone(), t)
            })
            .collect();

        let ret = match &func.return_type {
            Some(ann) => HulkType::from_name(ann),
            None => HulkType::Unknown,
        };

        self.symbols.functions.insert(
            func.name.clone(),
            FuncInfo {
                name: func.name.clone(),
                params,
                return_type: ret,
                span: func.span,
            },
        );
    }

    // ── The remaining visitor methods are no-ops for the collector ──

    fn visit_method(&mut self, _method: &ast::Method) {}
    fn visit_attribute(&mut self, _attr: &ast::Attribute) {}
    fn visit_expression(&mut self, _expr: &ast::Expression) {}
    fn visit_atom(&mut self, _atom: &ast::atoms::atom::Atom) {}
    fn visit_binary_op(&mut self, _binop: &ast::expressions::binoperation::BinaryOp) {}
    fn visit_unary_op(&mut self, _unary_op: &ast::expressions::unaryoperation::UnaryOp) {}
    fn visit_let_expr(&mut self, _let_expr: &ast::LetExpr) {}
    fn visit_if_expr(&mut self, _if_expr: &ast::IfExpr) {}
    fn visit_while_expr(&mut self, _while_expr: &ast::WhileExpr) {}
    fn visit_case_expr(&mut self, _case_expr: &ast::CaseExpr) {}
    fn visit_assign_expr(&mut self, _assign: &ast::AssignExpr) {}
    fn visit_member_access(&mut self, _access: &ast::MemberAccess) {}
    fn visit_method_call(&mut self, _call: &ast::MethodCall) {}
    fn visit_index_access(&mut self, _access: &ast::IndexAccess) {}
    fn visit_function_call(&mut self, _call: &ast::FunctionCall) {}
    fn visit_new_instance(&mut self, _inst: &ast::NewInstance) {}
    fn visit_new_array(&mut self, _arr: &ast::NewArray) {}
    fn visit_identifier(&mut self, _identifier: &tokens::Identifier) {}
    fn visit_literal(&mut self, _literal: &tokens::Literal) {}
}

// ═══════════════════════════════════════════════════════════════════
// Post-collection validation
// ═══════════════════════════════════════════════════════════════════

/// Helper: we allow forward references for now, this just avoids false
/// positives during collection.
fn is_forward_class(_parent: &str, _child: &str) -> bool {
    true
}

/// Validates that all parent types exist and there are no inheritance cycles.
fn check_inheritance_cycles(symbols: &SymbolTable) -> Vec<CompilerError> {
    let mut errors = Vec::new();

    for (name, class) in &symbols.classes {
        // Skip built-in types.
        if class.span.start == 0 && class.span.end == 0 && matches!(name.as_str(), "Object" | "Number" | "String" | "Boolean") {
            continue;
        }

        // Check parent exists.
        if let Some(ref parent) = class.parent {
            if !symbols.type_exists(parent) {
                errors.push(CompilerError::undefined_type(parent, class.span));
            }
        }

        // Check for cycles: walk the parent chain.
        let mut visited = HashSet::new();
        let mut current = name.clone();
        loop {
            if !visited.insert(current.clone()) {
                errors.push(CompilerError::cyclic_inheritance(name, class.span));
                break;
            }
            match symbols.classes.get(&current) {
                Some(info) => match &info.parent {
                    Some(p) => current = p.clone(),
                    None => break,
                },
                None => break,
            }
        }
    }

    errors
}

/// Attempts to infer a `HulkType` from a simple initializer expression.
/// Handles literals, unary minus on number, and `new ClassName(...)`.
/// Returns `Unknown` for anything more complex.
fn infer_type_from_expr(expr: &ast::Expression) -> HulkType {
    match expr {
        ast::Expression::Atom(atom) => match atom.as_ref() {
            ast::atoms::atom::Atom::NumberLiteral(_) => HulkType::Number,
            ast::atoms::atom::Atom::StringLiteral(_) => HulkType::String,
            ast::atoms::atom::Atom::BooleanLiteral(_) => HulkType::Boolean,
            ast::atoms::atom::Atom::Group(g) => infer_type_from_expr(&g.expression),
            _ => HulkType::Unknown,
        },
        ast::Expression::UnaryOp(unary) => {
            match &unary.op {
                tokens::UnaryOp::Minus(_) => HulkType::Number,
                tokens::UnaryOp::Not(_) => HulkType::Boolean,
            }
        }
        ast::Expression::NewInstance(inst) => HulkType::Class(inst.type_name.clone()),
        _ => HulkType::Unknown,
    }
}
