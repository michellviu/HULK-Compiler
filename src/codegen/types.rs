//! HULK → LLVM type mapping.
//!
//! Converts `HulkType` to LLVM `BasicTypeEnum` and generates struct
//! layouts for user-defined classes (including vtable pointer slot).

use inkwell::types::{BasicMetadataTypeEnum, BasicTypeEnum};

use parser::semantic::types::HulkType;

use super::context::CodegenContext;

impl<'ctx> CodegenContext<'ctx> {
    /// Maps a semantic `HulkType` to the corresponding LLVM type.
    ///
    /// - `Number`  → `f64`
    /// - `Boolean` → `i1`
    /// - `String`  → `ptr` (i8*)
    /// - `Class(_)` → `ptr` (pointer to class struct)
    /// - `Object`  → `ptr`
    /// - `Void` / `Error` / `Unknown` → `ptr` (fallback)
    pub fn hulk_type_to_llvm(&self, ht: &HulkType) -> BasicTypeEnum<'ctx> {
        match ht {
            HulkType::Number => self.f64_type().into(),
            HulkType::Boolean => self.bool_type().into(),
            HulkType::String => self.ptr_type().into(),
            HulkType::Class(_) => self.ptr_type().into(),
            HulkType::Object => self.ptr_type().into(),
            HulkType::Array(_) => self.ptr_type().into(),
            HulkType::SelfType => self.ptr_type().into(),
            HulkType::Void => self.ptr_type().into(),
            HulkType::Error | HulkType::Unknown => self.ptr_type().into(),
        }
    }

    /// Converts a `HulkType` to a `BasicMetadataTypeEnum` suitable for
    /// function parameter / return lists.
    pub fn hulk_type_to_meta(&self, ht: &HulkType) -> BasicMetadataTypeEnum<'ctx> {
        self.hulk_type_to_llvm(ht).into()
    }

    /// Returns `true` if the given HULK type should be returned as void
    /// in LLVM IR (i.e. the function returns nothing).
    pub fn is_void_type(ht: &HulkType) -> bool {
        matches!(ht, HulkType::Void)
    }
}
