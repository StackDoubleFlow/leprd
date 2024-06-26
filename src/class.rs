use crate::class_file::attributes::CodeAttribute;
use crate::class_file::descriptors::{FieldDescriptor, FieldType, MethodDescriptor};
use crate::class_file::{fields, ConstantPool};
use crate::class_loader::{method_area, ClassId, ClassLoader, FieldId, MethodArea, MethodId};
use crate::heap::{heap, ObjectRef};
use crate::value::Value;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug)]
pub enum FieldBacking {
    StaticValue(Value),
    /// Stored in the object at the given offset
    Instance(u32),
}

#[derive(Debug)]
pub struct Field {
    pub name: String,
    pub defining_class: ClassId,
    pub access_flags: u16,
    pub descriptor: FieldDescriptor,
    pub backing: FieldBacking,
}

impl Field {
    pub fn load_static(&self) -> Value {
        match &self.backing {
            FieldBacking::StaticValue(static_value) => static_value.extend_32(),
            _ => {
                panic!("tried to store value statically into non-static field");
            }
        }
    }

    pub fn store_static(&mut self, val: Value) {
        let ty = &self.descriptor.0;
        match &mut self.backing {
            FieldBacking::StaticValue(static_value) => {
                *static_value = val.store_ty(ty);
            }
            _ => {
                panic!("tried to store value statically into non-static field");
            }
        }
    }

    pub fn is_static(&self) -> bool {
        self.access_flags & fields::acc::STATIC != 0
    }
}

#[derive(Debug)]
pub struct Method {
    pub defining_class: ClassId,
    pub name: String,
    pub descriptor: MethodDescriptor,
    pub code: Option<Arc<CodeAttribute>>,
    pub access_flags: u16,
}

#[derive(Copy, Clone, Debug)]
pub enum Reference {
    Unresolved,
    Field(FieldId),
    Method(MethodId),
    Class(ClassId),
}

impl Reference {
    fn is_unresolved(self) -> bool {
        matches!(self, Reference::Unresolved)
    }
    fn field(self) -> Option<FieldId> {
        match self {
            Reference::Field(id) => Some(id),
            _ => None,
        }
    }

    fn method(self) -> Option<MethodId> {
        match self {
            Reference::Method(id) => Some(id),
            _ => None,
        }
    }

    fn class(self) -> Option<ClassId> {
        match self {
            Reference::Class(id) => Some(id),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct Class {
    pub constant_pool: ConstantPool,
    /// Keyed using constant pool index to reference entry
    pub references: HashMap<u16, Reference>,
    pub initialized: bool,
    pub class_obj: Option<ObjectRef>,

    pub defining_loader: ClassLoader,
    pub name: String,
    pub super_class: Option<ClassId>,
    pub interfaces: Vec<ClassId>,
    pub access_flags: u16,
    pub methods: Vec<MethodId>,
    pub fields: Vec<FieldId>,

    /// Array element type. Only used for array classes.
    pub elem_ty: Option<FieldType>,

    pub size: u32,
    pub alignment: u8,
}

impl Class {
    pub fn class_reference(id: ClassId, cp_idx: u16) -> ClassId {
        let ma = method_area();
        let self_class = &ma.classes[id];
        match self_class.references[&cp_idx] {
            Reference::Unresolved => {
                let name = self_class.constant_pool.class_name(cp_idx);
                drop(ma);
                let class = method_area().resolve_class(&name);

                let self_class = &mut method_area().classes[id];
                *self_class.references.get_mut(&cp_idx).unwrap() = Reference::Class(class);

                class
            }
            Reference::Class(class) => class,
            _ => unreachable!(),
        }
    }

    pub fn field_reference(id: ClassId, cp_idx: u16) -> FieldId {
        let ma = method_area();
        let self_class = &ma.classes[id];
        match self_class.references[&cp_idx] {
            Reference::Unresolved => {
                let (class_idx, nat) = self_class.constant_pool.any_ref(cp_idx);
                let (name, ..) = self_class.constant_pool.nat(nat);
                drop(ma);
                let class_id = Class::class_reference(id, class_idx);

                let mut ma = method_area();
                let field = ma.resolve_field(class_id, &name);
                let self_class = &mut ma.classes[id];
                *self_class.references.get_mut(&cp_idx).unwrap() = Reference::Field(field);

                field
            }
            Reference::Field(field) => field,
            _ => unreachable!(),
        }
    }

    pub fn method_reference(id: ClassId, cp_idx: u16) -> MethodId {
        let ma = method_area();
        let self_class = &ma.classes[id];
        match self_class.references[&cp_idx] {
            Reference::Unresolved => {
                let (class_idx, nat) = self_class.constant_pool.any_ref(cp_idx);
                let (name, descriptor) = self_class.constant_pool.nat(nat);
                drop(ma);
                let class_id = Class::class_reference(id, class_idx);

                let mut ma = method_area();
                let method =
                    ma.resolve_method(class_id, &name, &MethodDescriptor::read(&descriptor));
                let self_class = &mut ma.classes[id];
                *self_class.references.get_mut(&cp_idx).unwrap() = Reference::Method(method);
                method
            }
            Reference::Method(method) => method,
            _ => unreachable!(),
        }
    }

    pub fn obj(id: ClassId) -> ObjectRef {
        let mut ma = method_area();
        if let Some(obj_id) = ma.classes[id].class_obj {
            return obj_id;
        }

        let class_class = ma.resolve_class("java/lang/Class");
        let obj = heap().new_object(&mut ma, class_class);
        ma.class_objs.insert(obj, id);
        // TODO: Initialize fields such as classLoader
        obj
    }

    fn of_field_ty(ma: &mut MethodArea, field_ty: FieldType) -> ClassId {
        match field_ty {
            FieldType::ArrayType(arr_ty) => {
                let elem_ty = arr_ty.0 .0;
                ma.resolve_arr_class(&elem_ty)
            }
            FieldType::ObjectType(obj_ty) => ma.resolve_class(&obj_ty.class_name),
            _ => todo!(),
        }
    }

    pub fn instance_of(this: ClassId, of: ClassId) -> bool {
        let mut ma = method_area();

        if let Some(this_elem_ty) = &ma.classes[this].elem_ty {
            if let Some(of_elem_ty) = &ma.classes[of].elem_ty {
                return match (this_elem_ty, of_elem_ty) {
                    (FieldType::BaseType(this_prim), FieldType::BaseType(of_prim)) => {
                        this_prim == of_prim
                    }
                    _ => {
                        // FIXME: how do I get rid of these clones?
                        let this_elem_ty = this_elem_ty.clone();
                        let of_elem_ty = this_elem_ty.clone();
                        Self::instance_of(
                            Self::of_field_ty(&mut ma, this_elem_ty),
                            Self::of_field_ty(&mut ma, of_elem_ty),
                        )
                    }
                };
            } else {
                // TODO: also check if `of` is one of the interfaces implemented by array (JLS
                // 4.10.3)
                return of == ma.resolve_class("java/lang/Object");
            }
        }

        let mut cur_class = this;
        loop {
            let class = &ma.classes[cur_class];
            if cur_class == of || class.interfaces.contains(&of) {
                break true;
            }

            if let Some(parent) = class.super_class {
                cur_class = parent;
            } else {
                break false;
            }
        }
    }
}
