//! Core code generation context.
//!
//! Wraps LLVM types from inkwell and provides the central builder
//! plus helpers shared across all codegen sub-modules.

use std::collections::HashMap;
use std::path::Path;

use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::{BasicTypeEnum, StructType};
use inkwell::values::{BasicValueEnum, FunctionValue, PointerValue};
use inkwell::AddressSpace;

use parser::semantic::SymbolTable;
use parser::tokens::Span;

/// Value produced by HULK expression codegen.
///
/// `None` represents `Void` (e.g. `print` returns nothing usable).
pub type HulkValue<'ctx> = Option<BasicValueEnum<'ctx>>;

/// The central codegen context that threads through every code-generation
/// function.  It owns the LLVM context, module and builder and keeps
/// lookup tables for variables, functions, class struct layouts, and
/// vtables.
pub struct CodegenContext<'ctx> {
    // ── LLVM core ────────────────────────────────────────────────
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,

    // ── Semantic info ────────────────────────────────────────────
    pub symbols: SymbolTable,
    pub source_filename: String,
    pub source_text: String,

    // ── Variable scopes ──────────────────────────────────────────
    /// Stack of lexical scopes mapping variable names → (alloca pointer, LLVM type).
    pub scopes: Vec<HashMap<String, (PointerValue<'ctx>, BasicTypeEnum<'ctx>)>>,

    // ── Functions ────────────────────────────────────────────────
    /// All compiled LLVM functions by name.
    pub functions: HashMap<String, FunctionValue<'ctx>>,

    // ── Classes ──────────────────────────────────────────────────
    /// Struct type layouts per class name.
    pub class_structs: HashMap<String, StructType<'ctx>>,
    /// Map from class name → vtable global pointer.
    pub vtables: HashMap<String, PointerValue<'ctx>>,
    /// Per-class ordered vtable entries: (method name, owner class).
    pub vtable_layouts: HashMap<String, Vec<(String, String)>>,
    /// Stable runtime type IDs by class name.
    pub class_type_ids: HashMap<String, u64>,
    /// Map from (class, method_name) → index in vtable.
    pub vtable_indices: HashMap<(String, String), usize>,
    /// Ordered list of attribute names per class (for GEP indexing).
    /// Index 0 is the vtable pointer; attributes start at 1.
    pub class_field_indices: HashMap<(String, String), u32>,

    // ── String interning ─────────────────────────────────────────
    /// Cached LLVM global string constants.
    pub string_constants: HashMap<String, PointerValue<'ctx>>,

    // ── Current class (for `self` resolution) ────────────────────
    pub current_class: Option<String>,
    pub current_method: Option<String>,
}

impl<'ctx> CodegenContext<'ctx> {
    pub fn new(
        context: &'ctx Context,
        module_name: &str,
        symbols: SymbolTable,
        source_filename: &str,
        source_text: &str,
    ) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();

        Self {
            context,
            module,
            builder,
            symbols,
            source_filename: source_filename.to_string(),
            source_text: source_text.to_string(),
            scopes: vec![HashMap::new()],
            functions: HashMap::new(),
            class_structs: HashMap::new(),
            vtables: HashMap::new(),
            vtable_layouts: HashMap::new(),
            class_type_ids: HashMap::new(),
            vtable_indices: HashMap::new(),
            class_field_indices: HashMap::new(),
            string_constants: HashMap::new(),
            current_class: None,
            current_method: None,
        }
    }

    // ── Scope management ─────────────────────────────────────────

    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn set_variable(&mut self, name: &str, ptr: PointerValue<'ctx>, ty: BasicTypeEnum<'ctx>) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), (ptr, ty));
        }
    }

    pub fn get_variable(&self, name: &str) -> Option<(PointerValue<'ctx>, BasicTypeEnum<'ctx>)> {
        for scope in self.scopes.iter().rev() {
            if let Some(entry) = scope.get(name) {
                return Some(*entry);
            }
        }
        None
    }

    // ── LLVM type helpers ────────────────────────────────────────

    /// f64 type (HULK Number).
    pub fn f64_type(&self) -> inkwell::types::FloatType<'ctx> {
        self.context.f64_type()
    }

    /// i1 type (HULK Boolean).
    pub fn bool_type(&self) -> inkwell::types::IntType<'ctx> {
        self.context.bool_type()
    }

    /// i8* type (C string / HULK String).
    pub fn string_ptr_type(&self) -> inkwell::types::PointerType<'ctx> {
        self.context.ptr_type(AddressSpace::default())
    }

    /// Generic pointer type (for class instances, etc.).
    pub fn ptr_type(&self) -> inkwell::types::PointerType<'ctx> {
        self.context.ptr_type(AddressSpace::default())
    }

    /// void type.
    pub fn void_type(&self) -> inkwell::types::VoidType<'ctx> {
        self.context.void_type()
    }

    // ── Alloca helper ────────────────────────────────────────────

    /// Creates an alloca instruction in the entry block of the current
    /// function.  This is the standard way to allocate locals.
    pub fn create_entry_block_alloca(
        &self,
        function: FunctionValue<'ctx>,
        name: &str,
        ty: BasicTypeEnum<'ctx>,
    ) -> PointerValue<'ctx> {
        let entry = function.get_first_basic_block().unwrap();
        let tmp_builder = self.context.create_builder();
        match entry.get_first_instruction() {
            Some(first) => tmp_builder.position_before(&first),
            None => tmp_builder.position_at_end(entry),
        }
        tmp_builder.build_alloca(ty, name).unwrap()
    }

    // ── String constant helper ───────────────────────────────────

    /// Returns a pointer to a global string constant, caching it.
    pub fn get_or_create_string(&mut self, s: &str) -> PointerValue<'ctx> {
        if let Some(&ptr) = self.string_constants.get(s) {
            return ptr;
        }
        let global = self.builder.build_global_string_ptr(s, "str").unwrap();
        let ptr = global.as_pointer_value();
        self.string_constants.insert(s.to_string(), ptr);
        ptr
    }

    // ── Current function helper ──────────────────────────────────

    pub fn current_function(&self) -> FunctionValue<'ctx> {
        self.builder
            .get_insert_block()
            .unwrap()
            .get_parent()
            .unwrap()
    }

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

    pub fn runtime_site_from_span(&self, span: Span) -> (u32, u32, String, u32, u32) {
        let (line, col) = Self::offset_to_line_col(&self.source_text, span.start);
        let source_line = self
            .source_text
            .lines()
            .nth(line.saturating_sub(1))
            .unwrap_or("")
            .to_string();

        let span_len = span.end.saturating_sub(span.start).max(1);
        let line_len = source_line.chars().count();
        let available = line_len.saturating_sub(col.saturating_sub(1)).max(1);
        let marker_len = span_len.min(available).max(1);

        (line as u32, col as u32, source_line, col as u32, marker_len as u32)
    }

    // ── Output ───────────────────────────────────────────────────

    /// Writes the LLVM IR to a `.ll` file for debugging.
    pub fn write_ir(&self, path: &Path) {
        self.module.print_to_file(path).unwrap();
    }

    /// Writes a native object file via LLVM's target machine.
    pub fn write_object_file(&self, path: &Path) -> Result<(), String> {
        use inkwell::targets::*;
        Target::initialize_native(&InitializationConfig::default())
            .map_err(|e| e.to_string())?;

        let triple = TargetMachine::get_default_triple();
        let target = Target::from_triple(&triple).map_err(|e| e.to_string())?;
        let machine = target
            .create_target_machine(
                &triple,
                "generic",
                "",
                inkwell::OptimizationLevel::Default,
                RelocMode::PIC,
                CodeModel::Default,
            )
            .ok_or("Could not create target machine")?;

        // Set data layout on the module.
        self.module
            .set_data_layout(&machine.get_target_data().get_data_layout());
        self.module.set_triple(&triple);

        machine
            .write_to_file(&self.module, FileType::Object, path)
            .map_err(|e| e.to_string())
    }
}
