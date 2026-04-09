//! Code generation for class types.
//!
//! Each HULK class is compiled to:
//! 1. An LLVM struct type with a vtable pointer slot (index 0) followed
//!    by one slot per attribute (including inherited ones).
//! 2. A vtable — an array of function pointers for all methods
//!    (inherited methods filled in from parent).
//! 3. A constructor function `__ClassName_new(params) → ptr`.
//! 4. Method functions `__ClassName_methodName(self, params) → ret`.

use std::collections::HashMap;

use inkwell::types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum};
use inkwell::values::BasicValue;

use parser::ast;
use parser::semantic::types::HulkType;

use super::context::CodegenContext;

impl<'ctx> CodegenContext<'ctx> {
    /// Generates LLVM IR for all class declarations.
    pub fn gen_classes(&mut self, classes: &[ast::ClassDecl]) {
        // Phase 1: Create opaque struct types and collect the full
        //          attribute/method lists (including inherited) in
        //          topological order (parents before children).
        let ordered = self.topo_sort_classes(classes);

        for class_name in &ordered {
            let class = classes.iter().find(|c| &c.name == class_name).unwrap();
            self.define_class_struct(class);
        }

        // Phase 2: Build vtables.
        for class_name in &ordered {
            let class = classes.iter().find(|c| &c.name == class_name).unwrap();
            self.build_vtable(class);
        }

        // Phase 3: Forward-declare all methods.
        for class_name in &ordered {
            let class = classes.iter().find(|c| &c.name == class_name).unwrap();
            self.declare_methods(class);
        }

        // Phase 4: Generate constructor functions.
        for class_name in &ordered {
            let class = classes.iter().find(|c| &c.name == class_name).unwrap();
            self.gen_constructor(class);
        }

        // Phase 5: Generate method bodies.
        for class_name in &ordered {
            let class = classes.iter().find(|c| &c.name == class_name).unwrap();
            self.gen_method_bodies(class);
        }
    }

    /// Topologically sorts classes so parents come before children.
    fn topo_sort_classes(&self, classes: &[ast::ClassDecl]) -> Vec<String> {
        let mut result = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let class_map: HashMap<&str, &ast::ClassDecl> =
            classes.iter().map(|c| (c.name.as_str(), c)).collect();

        fn visit<'a>(
            name: &str,
            class_map: &HashMap<&str, &'a ast::ClassDecl>,
            visited: &mut std::collections::HashSet<String>,
            result: &mut Vec<String>,
        ) {
            if visited.contains(name) {
                return;
            }
            visited.insert(name.to_string());
            if let Some(class) = class_map.get(name) {
                if let Some(ref parent) = class.parent {
                    visit(parent, class_map, visited, result);
                }
            }
            result.push(name.to_string());
        }

        for class in classes {
            visit(&class.name, &class_map, &mut visited, &mut result);
        }

        result
    }

    /// Collects all attributes for a class including inherited ones,
    /// in order: parent attributes first, then own attributes.
    fn collect_all_attributes(&self, class: &ast::ClassDecl, all_classes: &[ast::ClassDecl]) -> Vec<(String, HulkType)> {
        let mut attrs = Vec::new();

        // Inherited attributes.
        if let Some(ref parent_name) = class.parent {
            if let Some(parent_class) = all_classes.iter().find(|c| &c.name == parent_name) {
                attrs = self.collect_all_attributes(parent_class, all_classes);
            }
        }

        // Own attributes.
        for attr in &class.attributes {
            let ht = match &attr.type_ann {
                Some(ann) => HulkType::from_name(ann),
                None => HulkType::Unknown,
            };
            attrs.push((attr.name.clone(), ht));
        }

        attrs
    }

    /// Defines the LLVM struct type for a class.
    fn define_class_struct(&mut self, class: &ast::ClassDecl) {
        let ptr_ty = self.ptr_type();

        // Slot 0: vtable pointer (ptr).
        let mut field_types: Vec<BasicTypeEnum<'ctx>> = vec![ptr_ty.into()];
        let mut field_index: u32 = 1;

        // Get all attributes from symbol table (includes inherited via resolve).
        if let Some(class_info) = self.symbols.get_class(&class.name).cloned() {
            // Walk ancestor chain to get all attributes in order.
            let ancestors = self.symbols.ancestors(&class.name);
            for ancestor_name in ancestors.iter().rev() {
                if let Some(ancestor_info) = self.symbols.get_class(ancestor_name) {
                    for attr in &ancestor_info.attributes {
                        let llvm_ty = self.hulk_type_to_llvm(&attr.hulk_type);
                        field_types.push(llvm_ty);
                        self.class_field_indices
                            .insert((class.name.clone(), attr.name.clone()), field_index);
                        field_index += 1;
                    }
                }
            }
            // Own attributes.
            for attr in &class_info.attributes {
                // Skip if already added from ancestors.
                if self
                    .class_field_indices
                    .contains_key(&(class.name.clone(), attr.name.clone()))
                {
                    continue;
                }
                let llvm_ty = self.hulk_type_to_llvm(&attr.hulk_type);
                field_types.push(llvm_ty);
                self.class_field_indices
                    .insert((class.name.clone(), attr.name.clone()), field_index);
                field_index += 1;
            }
        }

        let struct_ty = self.context.opaque_struct_type(&class.name);
        struct_ty.set_body(&field_types, false);
        self.class_structs.insert(class.name.clone(), struct_ty);
    }

    /// Collects all methods (including inherited) and builds a vtable global.
    fn build_vtable(&mut self, class: &ast::ClassDecl) {
        // Collect method list: start from parent's vtable, override as needed.
        let mut method_list: Vec<(String, String)> = Vec::new(); // (method_name, owner_class)

        if let Some(ref parent_name) = class.parent {
            // Copy parent's vtable entries.
            if let Some(parent_class_info) = self.symbols.get_class(parent_name).cloned() {
                let ancestors = self.symbols.ancestors(parent_name);
                for ancestor in ancestors.iter().rev() {
                    if let Some(info) = self.symbols.get_class(ancestor) {
                        for (method_name, _) in &info.methods {
                            if !method_list.iter().any(|(m, _)| m == method_name) {
                                method_list.push((method_name.clone(), ancestor.to_string()));
                            }
                        }
                    }
                }
                // Parent's own methods.
                for (method_name, _) in &parent_class_info.methods {
                    if !method_list.iter().any(|(m, _)| m == method_name) {
                        method_list.push((method_name.clone(), parent_name.clone()));
                    }
                }
            }
        }

        // Override or add own methods.
        for method in &class.methods {
            if let Some(entry) = method_list.iter_mut().find(|(m, _)| m == &method.name) {
                entry.1 = class.name.clone(); // override
            } else {
                method_list.push((method.name.clone(), class.name.clone()));
            }
        }

        // Store vtable indices.
        for (idx, (method_name, _owner)) in method_list.iter().enumerate() {
            self.vtable_indices
                .insert((class.name.clone(), method_name.clone()), idx);
        }

        // We'll create the actual vtable global after methods are declared.
        // For now, store the method list and create later in gen_constructor.
    }

    /// Forward-declares all methods of a class as LLVM functions.
    fn declare_methods(&mut self, class: &ast::ClassDecl) {
        for method in &class.methods {
            let method_llvm_name = format!("__{}__{}", class.name, method.name);

            // First parameter is always `self` (ptr).
            let mut param_types: Vec<BasicMetadataTypeEnum<'ctx>> =
                vec![self.ptr_type().into()];

            for p in &method.params {
                let ht = match &p.type_ann {
                    Some(ann) => HulkType::from_name(ann),
                    None => HulkType::Number,
                };
                param_types.push(self.hulk_type_to_meta(&ht));
            }

            let ret_type = match &method.return_type {
                Some(ann) => HulkType::from_name(ann),
                None => HulkType::Void,
            };

            let fn_type = if Self::is_void_type(&ret_type) {
                self.void_type().fn_type(&param_types, false)
            } else {
                self.hulk_type_to_llvm(&ret_type)
                    .fn_type(&param_types, false)
            };

            let llvm_func = self.module.add_function(&method_llvm_name, fn_type, None);
            self.functions.insert(method_llvm_name, llvm_func);
        }
    }

    /// Generates the constructor function for a class.
    pub fn gen_constructor(&mut self, class: &ast::ClassDecl) {
        let ctor_name = format!("__{}_new", class.name);
        let ptr_ty = self.ptr_type();

        // Constructor params.
        let param_types: Vec<BasicMetadataTypeEnum<'ctx>> = class
            .params
            .iter()
            .map(|p| {
                let ht = match &p.type_ann {
                    Some(ann) => HulkType::from_name(ann),
                    None => HulkType::Number,
                };
                self.hulk_type_to_meta(&ht)
            })
            .collect();

        let fn_type = ptr_ty.fn_type(&param_types, false);
        let ctor_func = self.module.add_function(&ctor_name, fn_type, None);
        self.functions.insert(ctor_name.clone(), ctor_func);

        let entry_bb = self.context.append_basic_block(ctor_func, "entry");
        self.builder.position_at_end(entry_bb);

        self.push_scope();
        self.current_class = Some(class.name.clone());

        // Bind constructor params.
        for (i, param) in class.params.iter().enumerate() {
            let ht = match &param.type_ann {
                Some(ann) => HulkType::from_name(ann),
                None => HulkType::Number,
            };
            let llvm_ty = self.hulk_type_to_llvm(&ht);
            let alloca =
                self.create_entry_block_alloca(ctor_func, &param.name, llvm_ty);
            self.builder
                .build_store(alloca, ctor_func.get_nth_param(i as u32).unwrap())
                .unwrap();
            self.set_variable(&param.name, alloca, llvm_ty);
        }

        // Allocate the instance.
        let struct_ty = self.class_structs.get(&class.name).unwrap().clone();
        let size = struct_ty.size_of().unwrap();
        let alloc_fn = *self.functions.get("__hulk_alloc").unwrap();
        let raw_ptr = self
            .builder
            .build_call(alloc_fn, &[size.into()], "instance")
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap();

        let instance_ptr = raw_ptr.into_pointer_value();

        // Store `self` for attribute initializers.
        let self_alloca = self.create_entry_block_alloca(ctor_func, "self", ptr_ty.into());
        self.builder.build_store(self_alloca, instance_ptr).unwrap();
        self.set_variable("self", self_alloca, ptr_ty.into());

        // Initialize attributes.
        for attr in &class.attributes {
            let init_val = self.gen_expression(&attr.init);
            if let Some(val) = init_val {
                if let Some(&field_idx) =
                    self.class_field_indices.get(&(class.name.clone(), attr.name.clone()))
                {
                    let field_ptr = self
                        .builder
                        .build_struct_gep(struct_ty, instance_ptr, field_idx, &attr.name)
                        .unwrap();
                    self.builder.build_store(field_ptr, val).unwrap();
                }
            }
        }

        // Return the instance pointer.
        self.builder
            .build_return(Some(&instance_ptr as &dyn BasicValue))
            .unwrap();

        self.current_class = None;
        self.pop_scope();
    }

    /// Generates method bodies for a class.
    fn gen_method_bodies(&mut self, class: &ast::ClassDecl) {
        for method in &class.methods {
            self.gen_method_body(class, method);
        }
    }

    fn gen_method_body(&mut self, class: &ast::ClassDecl, method: &ast::Method) {
        let method_llvm_name = format!("__{}__{}", class.name, method.name);
        let llvm_func = *self.functions.get(&method_llvm_name).unwrap();
        let entry_bb = self.context.append_basic_block(llvm_func, "entry");
        self.builder.position_at_end(entry_bb);

        self.push_scope();
        self.current_class = Some(class.name.clone());

        // Param 0 is `self`.
        let self_ptr = llvm_func.get_nth_param(0).unwrap().into_pointer_value();
        let ptr_ty_enum: BasicTypeEnum = self.ptr_type().into();
        let self_alloca =
            self.create_entry_block_alloca(llvm_func, "self", ptr_ty_enum);
        self.builder.build_store(self_alloca, self_ptr).unwrap();
        self.set_variable("self", self_alloca, ptr_ty_enum);

        // Bind other parameters.
        for (i, param) in method.params.iter().enumerate() {
            let ht = match &param.type_ann {
                Some(ann) => HulkType::from_name(ann),
                None => HulkType::Number,
            };
            let llvm_ty = self.hulk_type_to_llvm(&ht);
            let alloca =
                self.create_entry_block_alloca(llvm_func, &param.name, llvm_ty);
            self.builder
                .build_store(
                    alloca,
                    llvm_func.get_nth_param((i + 1) as u32).unwrap(),
                )
                .unwrap();
            self.set_variable(&param.name, alloca, llvm_ty);
        }

        // Generate body.
        let body_val = self.gen_body(&method.body);

        // Build return.
        let ret_type = match &method.return_type {
            Some(ann) => HulkType::from_name(ann),
            None => HulkType::Void,
        };

        let current_bb = self.builder.get_insert_block().unwrap();
        if current_bb.get_terminator().is_none() {
            if Self::is_void_type(&ret_type) {
                self.builder.build_return(None).unwrap();
            } else if let Some(val) = body_val {
                self.builder.build_return(Some(&val)).unwrap();
            } else {
                let default = self.default_value(&ret_type);
                self.builder.build_return(Some(&default)).unwrap();
            }
        }

        self.current_class = None;
        self.pop_scope();
    }
}
