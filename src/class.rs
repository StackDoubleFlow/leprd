use crate::class_file::attributes::CodeAttribute;
use crate::class_file::descriptors::MethodDescriptor;
use crate::class_file::ConstantPool;
use crate::class_loader::{method_area, ClassId, ClassLoader, FieldId, MethodId};
use crate::value::Value;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug)]
pub struct Field {
    pub name: String,
    pub defining_class: ClassId,
    pub access_flags: u16,
    pub static_value: Option<Value>,
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

    pub defining_loader: ClassLoader,
    pub name: String,
    pub super_class: Option<ClassId>,
    pub interfaces: Vec<ClassId>,
    pub access_flags: u16,
    pub methods: Vec<MethodId>,
    pub fields: Vec<FieldId>,
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
                let method = ma.resolve_method(class_id, &name, &descriptor);
                let self_class = &mut ma.classes[id];
                *self_class.references.get_mut(&cp_idx).unwrap() = Reference::Method(method);
                method
            }
            Reference::Method(method) => method,
            _ => unreachable!(),
        }
    }
}
