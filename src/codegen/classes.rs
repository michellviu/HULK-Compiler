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

        // Assign stable runtime IDs for dynamic type tests/casts.
        for (idx, class_name) in ordered.iter().enumerate() {
            self.class_type_ids
                .insert(class_name.clone(), (idx + 1) as u64);
        }

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

        // Phase 4: Materialize vtable globals now that all methods exist.
        for class_name in &ordered {
            let class = classes.iter().find(|c| &c.name == class_name).unwrap();
            self.materialize_vtable(class);
        }

        // Phase 5: Generate constructor functions.
        for class_name in &ordered {
            let class = classes.iter().find(|c| &c.name == class_name).unwrap();
            self.gen_constructor(class);
        }

        // Phase 6: Generate method bodies.
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
        // Slot 1: runtime class type id (i64).
        let mut field_types: Vec<BasicTypeEnum<'ctx>> =
            vec![ptr_ty.into(), self.context.i64_type().into()];
        let mut field_index: u32 = 2;

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

    /// Collects all methods (including inherited) and stores the vtable layout.
    fn build_vtable(&mut self, class: &ast::ClassDecl) {
        // Collect method list: copy parent layout first, then override/add own methods.
        let mut method_list: Vec<(String, String)> = class
            .parent
            .as_ref()
            .and_then(|p| self.vtable_layouts.get(p).cloned())
            .unwrap_or_default();

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

        self.vtable_layouts
            .insert(class.name.clone(), method_list);
    }

    fn materialize_vtable(&mut self, class: &ast::ClassDecl) {
        let ptr_ty = self.ptr_type();
        let entries = self
            .vtable_layouts
            .get(&class.name)
            .cloned()
            .unwrap_or_default();

        let array_ty = ptr_ty.array_type(entries.len() as u32);
        let mut values = Vec::with_capacity(entries.len());

        for (method_name, owner_class) in entries {
            let method_llvm_name = format!("__{}__{}", owner_class, method_name);
            if let Some(&llvm_fn) = self.functions.get(&method_llvm_name) {
                values.push(llvm_fn.as_global_value().as_pointer_value());
            } else {
                values.push(ptr_ty.const_null());
            }
        }

        let init = ptr_ty.const_array(&values);
        let global = self
            .module
            .add_global(array_ty, None, &format!("__vtable_{}", class.name));
        global.set_initializer(&init);
        global.set_constant(true);

        self.vtables
            .insert(class.name.clone(), global.as_pointer_value());
    }

    /// Forward-declares all methods of a class as LLVM functions.
    fn declare_methods(&mut self, class: &ast::ClassDecl) {
        let class_info = self.symbols.get_class(&class.name).cloned();

        for method in &class.methods {
            let method_llvm_name = format!("__{}__{}", class.name, method.name);
            let method_info = class_info
                .as_ref()
                .and_then(|info| info.get_method(&method.name))
                .cloned();

            // First parameter is always `self` (ptr).
            let mut param_types: Vec<BasicMetadataTypeEnum<'ctx>> =
                vec![self.ptr_type().into()];

            if let Some(info) = &method_info {
                for (_, ht) in &info.params {
                    param_types.push(self.hulk_type_to_meta(ht));
                }
            } else {
                for p in &method.params {
                    let ht = match &p.type_ann {
                        Some(ann) => HulkType::from_name(ann),
                        None => HulkType::Number,
                    };
                    param_types.push(self.hulk_type_to_meta(&ht));
                }
            }

            let ret_type = method_info
                .as_ref()
                .map(|m| m.return_type.clone())
                .unwrap_or_else(|| match &method.return_type {
                    Some(ann) => HulkType::from_name(ann),
                    None => HulkType::Void,
                });

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
        let class_info = self.symbols.get_class(&class.name).cloned();
        let ctor_params = class_info
            .as_ref()
            .map(|c| c.params.clone())
            .unwrap_or_default();

        // Constructor params.
        let param_types: Vec<BasicMetadataTypeEnum<'ctx>> = ctor_params
            .iter()
            .map(|(_, ht)| self.hulk_type_to_meta(ht))
            .collect();

        let fn_type = ptr_ty.fn_type(&param_types, false);
        let ctor_func = self.module.add_function(&ctor_name, fn_type, None);
        self.functions.insert(ctor_name.clone(), ctor_func);

        let entry_bb = self.context.append_basic_block(ctor_func, "entry");
        self.builder.position_at_end(entry_bb);

        self.push_scope();
        self.current_class = Some(class.name.clone());

        // Bind constructor params.
        for (i, (param_name, ht)) in ctor_params.iter().enumerate() {
            let llvm_ty = self.hulk_type_to_llvm(&ht);
            let alloca =
                self.create_entry_block_alloca(ctor_func, param_name, llvm_ty);
            self.builder
                .build_store(alloca, ctor_func.get_nth_param(i as u32).unwrap())
                .unwrap();
            self.set_variable(param_name, alloca, llvm_ty);
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

        // Initialize inherited attributes by running parent constructor first.
        if let Some(parent_name) = class.parent.as_ref().filter(|p| p.as_str() != "Object") {
            let parent_ctor_name = format!("__{}_new", parent_name);
            if let Some(&parent_ctor_fn) = self.functions.get(&parent_ctor_name) {
                let parent_info = self.symbols.get_class(parent_name).cloned();
                let mut parent_args_vals: Vec<inkwell::values::BasicMetadataValueEnum<'ctx>> =
                    Vec::new();

                if !class.parent_args.is_empty() {
                    for arg in &class.parent_args {
                        if let Some(v) = self.gen_expression(arg) {
                            parent_args_vals.push(v.into());
                        }
                    }
                } else if let Some(parent_info) = &parent_info {
                    for (idx, (parent_param_name, parent_param_type)) in
                        parent_info.params.iter().enumerate()
                    {
                        let fallback_name = ctor_params.get(idx).map(|(n, _)| n.as_str());
                        let source_name = if self.get_variable(parent_param_name).is_some() {
                            Some(parent_param_name.as_str())
                        } else {
                            fallback_name
                        };

                        let value = if let Some(name) = source_name {
                            if let Some((alloca, llvm_ty)) = self.get_variable(name) {
                                self.builder.build_load(llvm_ty, alloca, name).ok()
                            } else {
                                None
                            }
                        } else {
                            None
                        };

                        parent_args_vals.push(
                            value
                                .unwrap_or_else(|| self.default_value(parent_param_type))
                                .into(),
                        );
                    }
                }

                let parent_instance = self
                    .builder
                    .build_call(parent_ctor_fn, &parent_args_vals, "parent_instance")
                    .unwrap()
                    .try_as_basic_value()
                    .left()
                    .map(|v| v.into_pointer_value());

                if let (Some(parent_instance_ptr), Some(parent_info)) = (parent_instance, parent_info)
                {
                    let parent_struct_ty = self.class_structs.get(parent_name).unwrap().clone();
                    let ancestors = self.symbols.ancestors(parent_name);

                    for ancestor_name in ancestors.iter().rev() {
                        if let Some(ancestor_info) = self.symbols.get_class(ancestor_name) {
                            for attr in &ancestor_info.attributes {
                                let parent_idx = self
                                    .class_field_indices
                                    .get(&(parent_info.name.clone(), attr.name.clone()))
                                    .copied();
                                let child_idx = self
                                    .class_field_indices
                                    .get(&(class.name.clone(), attr.name.clone()))
                                    .copied();

                                if let (Some(pidx), Some(cidx)) = (parent_idx, child_idx) {
                                    let attr_ty = self.hulk_type_to_llvm(&attr.hulk_type);
                                    let parent_field_ptr = self
                                        .builder
                                        .build_struct_gep(
                                            parent_struct_ty,
                                            parent_instance_ptr,
                                            pidx,
                                            &format!("parent_{}", attr.name),
                                        )
                                        .unwrap();
                                    let child_field_ptr = self
                                        .builder
                                        .build_struct_gep(
                                            struct_ty,
                                            instance_ptr,
                                            cidx,
                                            &format!("child_{}", attr.name),
                                        )
                                        .unwrap();
                                    let value = self
                                        .builder
                                        .build_load(attr_ty, parent_field_ptr, "inherited_attr")
                                        .unwrap();
                                    self.builder.build_store(child_field_ptr, value).unwrap();
                                }
                            }
                        }
                    }
                }
            }
        }

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

        // Store class vtable pointer in slot 0.
        if let Some(&vtable_ptr) = self.vtables.get(&class.name) {
            let vtable_field_ptr = self
                .builder
                .build_struct_gep(struct_ty, instance_ptr, 0, "vtable")
                .unwrap();
            self.builder.build_store(vtable_field_ptr, vtable_ptr).unwrap();
        }

        // Store runtime class id in slot 1.
        if let Some(type_id) = self.class_type_ids.get(&class.name) {
            let type_id_field_ptr = self
                .builder
                .build_struct_gep(struct_ty, instance_ptr, 1, "type_id")
                .unwrap();
            let type_id_val = self.context.i64_type().const_int(*type_id, false);
            self.builder
                .build_store(type_id_field_ptr, type_id_val)
                .unwrap();
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
        let method_info = self
            .symbols
            .get_class(&class.name)
            .and_then(|info| info.get_method(&method.name))
            .cloned();
        let entry_bb = self.context.append_basic_block(llvm_func, "entry");
        self.builder.position_at_end(entry_bb);

        self.push_scope();
        self.current_class = Some(class.name.clone());
        self.current_method = Some(method.name.clone());

        // Param 0 is `self`.
        let self_ptr = llvm_func.get_nth_param(0).unwrap().into_pointer_value();
        let ptr_ty_enum: BasicTypeEnum = self.ptr_type().into();
        let self_alloca =
            self.create_entry_block_alloca(llvm_func, "self", ptr_ty_enum);
        self.builder.build_store(self_alloca, self_ptr).unwrap();
        self.set_variable("self", self_alloca, ptr_ty_enum);

        // Bind other parameters.
        for (i, param) in method.params.iter().enumerate() {
            let ht = method_info
                .as_ref()
                .and_then(|info| info.params.get(i).map(|(_, t)| t.clone()))
                .unwrap_or_else(|| match &param.type_ann {
                    Some(ann) => HulkType::from_name(ann),
                    None => HulkType::Number,
                });
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
        let ret_type = method_info
            .as_ref()
            .map(|m| m.return_type.clone())
            .unwrap_or_else(|| match &method.return_type {
                Some(ann) => HulkType::from_name(ann),
                None => HulkType::Void,
            });

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
        self.current_method = None;
        self.pop_scope();
    }
}
