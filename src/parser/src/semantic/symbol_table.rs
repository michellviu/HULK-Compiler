use std::collections::HashMap;
use crate::tokens::Span;
use super::types::HulkType;

// ═══════════════════════════════════════════════════════════════════
// Function & class metadata
// ═══════════════════════════════════════════════════════════════════

/// Metadata for a declared function (global or method).
#[derive(Debug, Clone)]
pub struct FuncInfo {
    pub name: String,
    pub params: Vec<(String, HulkType)>,
    pub return_type: HulkType,
    pub span: Span,
}

/// Metadata for a declared class attribute.
#[derive(Debug, Clone)]
pub struct AttrInfo {
    pub name: String,
    pub hulk_type: HulkType,
    pub span: Span,
}

/// Metadata for a declared class.
#[derive(Debug, Clone)]
pub struct ClassInfo {
    pub name: String,
    /// Constructor parameters.
    pub params: Vec<(String, HulkType)>,
    /// Parent class name (None for root `Object`).
    pub parent: Option<String>,
    /// Declared attributes (own, not inherited).
    pub attributes: Vec<AttrInfo>,
    /// Declared methods (own, not inherited).
    pub methods: HashMap<String, FuncInfo>,
    pub span: Span,
}

impl ClassInfo {
    /// Looks up an attribute by name in this class only (not inherited).
    pub fn get_attribute(&self, name: &str) -> Option<&AttrInfo> {
        self.attributes.iter().find(|a| a.name == name)
    }

    /// Looks up a method by name in this class only (not inherited).
    pub fn get_method(&self, name: &str) -> Option<&FuncInfo> {
        self.methods.get(name)
    }
}

// ═══════════════════════════════════════════════════════════════════
// Variable scope
// ═══════════════════════════════════════════════════════════════════

/// A single variable binding.
#[derive(Debug, Clone)]
pub struct VarInfo {
    pub name: String,
    pub hulk_type: HulkType,
    pub span: Span,
    /// Whether this variable has been read at least once.
    pub used: bool,
}

/// A single lexical scope containing variable bindings.
#[derive(Debug, Clone)]
struct Scope {
    vars: HashMap<String, VarInfo>,
}

impl Scope {
    fn new() -> Self {
        Scope { vars: HashMap::new() }
    }
}

// ═══════════════════════════════════════════════════════════════════
// Symbol Table
// ═══════════════════════════════════════════════════════════════════

/// Central symbol table for the HULK compiler.
///
/// Stores:
/// - Global function declarations
/// - Class declarations (with attributes and methods)
/// - A scope stack for local variable resolution
#[derive(Debug)]
pub struct SymbolTable {
    /// Global functions: name → FuncInfo.
    pub functions: HashMap<String, FuncInfo>,
    /// Class/type declarations: name → ClassInfo.
    pub classes: HashMap<String, ClassInfo>,
    /// Stack of lexical scopes for variable lookup.
    scopes: Vec<Scope>,
    /// The class currently being analyzed (for `self` resolution).
    pub current_class: Option<String>,
}

impl SymbolTable {
    /// Creates a new symbol table pre-populated with built-in types and functions.
    pub fn new() -> Self {
        let mut st = SymbolTable {
            functions: HashMap::new(),
            classes: HashMap::new(),
            scopes: vec![Scope::new()], // global scope
            current_class: None,
        };
        st.register_builtins();
        st
    }

    // ── Built-in registration ───────────────────────────────────

    fn register_builtins(&mut self) {
        let builtin_span = Span::new(0, 0);

        // Built-in types
        for name in &["Object", "Number", "String", "Boolean"] {
            self.classes.insert(name.to_string(), ClassInfo {
                name: name.to_string(),
                params: vec![],
                parent: if *name == "Object" { None } else { Some("Object".to_string()) },
                attributes: vec![],
                methods: HashMap::new(),
                span: builtin_span,
            });
        }

        // Built-in functions
        let builtin_fns: Vec<(&str, Vec<(&str, HulkType)>, HulkType)> = vec![
            ("print", vec![("value", HulkType::Object)], HulkType::Void),
            ("sin", vec![("x", HulkType::Number)], HulkType::Number),
            ("cos", vec![("x", HulkType::Number)], HulkType::Number),
            ("sqrt", vec![("x", HulkType::Number)], HulkType::Number),
            ("exp", vec![("x", HulkType::Number)], HulkType::Number),
            ("log", vec![("base", HulkType::Number), ("x", HulkType::Number)], HulkType::Number),
            ("rand", vec![], HulkType::Number),
        ];

        for (name, params, ret) in builtin_fns {
            self.functions.insert(name.to_string(), FuncInfo {
                name: name.to_string(),
                params: params.into_iter().map(|(n, t)| (n.to_string(), t)).collect(),
                return_type: ret,
                span: builtin_span,
            });
        }
    }

    // ── Scope management ────────────────────────────────────────

    /// Pushes a new lexical scope.
    pub fn push_scope(&mut self) {
        self.scopes.push(Scope::new());
    }

    /// Pops the innermost scope, returning all its variable bindings.
    pub fn pop_scope(&mut self) -> Vec<VarInfo> {
        let scope = self.scopes.pop().expect("Cannot pop the global scope");
        scope.vars.into_values().collect()
    }

    /// Defines a variable in the current (innermost) scope.
    /// Returns `false` if the variable already exists in this scope.
    pub fn define_var(&mut self, name: &str, hulk_type: HulkType, span: Span) -> bool {
        let scope = self.scopes.last_mut().expect("No scope available");
        if scope.vars.contains_key(name) {
            return false;
        }
        scope.vars.insert(name.to_string(), VarInfo {
            name: name.to_string(),
            hulk_type,
            span,
            used: false,
        });
        true
    }

    /// Looks up a variable by name, searching from innermost to outermost scope.
    /// Returns a reference to the `VarInfo` if found.
    pub fn lookup_var(&self, name: &str) -> Option<&VarInfo> {
        for scope in self.scopes.iter().rev() {
            if let Some(var) = scope.vars.get(name) {
                return Some(var);
            }
        }
        None
    }

    /// Marks a variable as used. Returns `true` if found.
    pub fn mark_var_used(&mut self, name: &str) -> bool {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(var) = scope.vars.get_mut(name) {
                var.used = true;
                return true;
            }
        }
        false
    }

    /// Looks up the type of a variable.
    pub fn var_type(&self, name: &str) -> Option<HulkType> {
        self.lookup_var(name).map(|v| v.hulk_type.clone())
    }

    // ── Type queries ────────────────────────────────────────────

    /// Checks if a type name is defined (built-in or user-defined class).
    pub fn type_exists(&self, name: &str) -> bool {
        matches!(name, "Number" | "String" | "Boolean" | "Void" | "Object")
            || self.classes.contains_key(name)
    }

    /// Returns the `ClassInfo` for a given type name.
    pub fn get_class(&self, name: &str) -> Option<&ClassInfo> {
        self.classes.get(name)
    }

    /// Returns the `FuncInfo` for a global function.
    pub fn get_function(&self, name: &str) -> Option<&FuncInfo> {
        self.functions.get(name)
    }

    /// Tests whether `child` conforms to (is a subtype of) `parent`.
    pub fn conforms_to(&self, child: &HulkType, parent: &HulkType) -> bool {
        // Error types conform to everything (for recovery).
        if child.is_error() || parent.is_error() {
            return true;
        }
        // Same type.
        if child == parent {
            return true;
        }
        // Everything conforms to Object.
        if *parent == HulkType::Object {
            return true;
        }
        // Check class hierarchy.
        match (child, parent) {
            (HulkType::Class(c), HulkType::Class(p)) => {
                self.is_subclass(c, p)
            }
            // Number, String, Boolean conform to Object (handled above).
            (HulkType::Number | HulkType::String | HulkType::Boolean, HulkType::Object) => true,
            _ => false,
        }
    }

    /// Checks if `child_class` is a (transitive) subclass of `parent_class`.
    fn is_subclass(&self, child_class: &str, parent_class: &str) -> bool {
        let mut current = child_class.to_string();
        let mut visited = std::collections::HashSet::new();
        loop {
            if current == parent_class {
                return true;
            }
            if !visited.insert(current.clone()) {
                // Cyclic inheritance — stop.
                return false;
            }
            match self.classes.get(&current) {
                Some(info) => match &info.parent {
                    Some(p) => current = p.clone(),
                    None => return false, // reached root (Object)
                },
                None => return false,
            }
        }
    }

    /// Computes the Lowest Common Ancestor (LCA) of two types in the
    /// inheritance hierarchy. Falls back to `Object` if no better match.
    pub fn lca(&self, a: &HulkType, b: &HulkType) -> HulkType {
        if a == b {
            return a.clone();
        }
        if a.is_error() {
            return b.clone();
        }
        if b.is_error() {
            return a.clone();
        }
        // If either is Object, LCA is Object.
        if *a == HulkType::Object || *b == HulkType::Object {
            return HulkType::Object;
        }
        // For class types, walk ancestors of `a` and check if `b` conforms.
        match (a, b) {
            (HulkType::Class(ca), HulkType::Class(_cb)) => {
                let ancestors = self.ancestors(ca);
                for ancestor in &ancestors {
                    if self.conforms_to(b, &HulkType::Class(ancestor.clone())) {
                        return HulkType::Class(ancestor.clone());
                    }
                }
                HulkType::Object
            }
            _ => HulkType::Object,
        }
    }

    /// Returns the chain of ancestors: [self, parent, grandparent, ..., Object].
    fn ancestors(&self, class_name: &str) -> Vec<String> {
        let mut chain = vec![];
        let mut current = class_name.to_string();
        let mut visited = std::collections::HashSet::new();
        loop {
            if !visited.insert(current.clone()) {
                break;
            }
            chain.push(current.clone());
            match self.classes.get(&current) {
                Some(info) => match &info.parent {
                    Some(p) => current = p.clone(),
                    None => break,
                },
                None => break,
            }
        }
        chain
    }

    /// Looks up an attribute in a class or its ancestors.
    pub fn resolve_attribute(&self, class_name: &str, attr_name: &str) -> Option<(String, AttrInfo)> {
        let mut current = class_name.to_string();
        let mut visited = std::collections::HashSet::new();
        loop {
            if !visited.insert(current.clone()) {
                return None;
            }
            if let Some(class) = self.classes.get(&current) {
                if let Some(attr) = class.get_attribute(attr_name) {
                    return Some((current, attr.clone()));
                }
                match &class.parent {
                    Some(p) => current = p.clone(),
                    None => return None,
                }
            } else {
                return None;
            }
        }
    }

    /// Looks up a method in a class or its ancestors.
    pub fn resolve_method(&self, class_name: &str, method_name: &str) -> Option<(String, FuncInfo)> {
        let mut current = class_name.to_string();
        let mut visited = std::collections::HashSet::new();
        loop {
            if !visited.insert(current.clone()) {
                return None;
            }
            if let Some(class) = self.classes.get(&current) {
                if let Some(method) = class.get_method(method_name) {
                    return Some((current, method.clone()));
                }
                match &class.parent {
                    Some(p) => current = p.clone(),
                    None => return None,
                }
            } else {
                return None;
            }
        }
    }
}
