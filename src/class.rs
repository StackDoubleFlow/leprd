use crate::class_file::attributes::CodeAttribute;
use crate::class_file::ConstantPool;
use crate::class_loader::{
    method_area, resolve_class, resolve_field, resolve_method, ClassId, ClassLoader, FieldId,
    MethodId,
};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug)]
pub struct Field {
    pub name: String,
    pub defining_class: ClassId,
    pub access_flags: u16,
}

#[derive(Debug)]
pub struct Method {
    pub defining_class: ClassId,
    pub name: String,
    pub descriptor: String,
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

    pub defining_loader: ClassLoader,
    pub name: String,
    pub super_class: Option<ClassId>,
    pub interfaces: Vec<ClassId>,
    pub access_flags: u16,
    pub methods: Vec<MethodId>,
    pub fields: Vec<FieldId>,
}

impl Class {
    pub fn class_reference(&mut self, cp_idx: u16) -> ClassId {
        match self.references[&cp_idx] {
            Reference::Unresolved => {
                let name = self.constant_pool.class_name(cp_idx);
                let class = resolve_class(&name);
                *self.references.get_mut(&cp_idx).unwrap() = Reference::Class(class);
                class
            }
            Reference::Class(class) => class,
            _ => unreachable!(),
        }
    }

    pub fn field_reference(&mut self, cp_idx: u16) -> FieldId {
        match self.references[&cp_idx] {
            Reference::Unresolved => {
                let (class_idx, nat) = self.constant_pool.any_ref(cp_idx);
                let class_id = self.class_reference(class_idx);
                let (name, ..) = self.constant_pool.nat(nat);
                let field = resolve_field(class_id, &name);
                *self.references.get_mut(&cp_idx).unwrap() = Reference::Field(field);
                field
            }
            Reference::Field(field) => field,
            _ => unreachable!(),
        }
    }

    pub fn method_reference(&mut self, cp_idx: u16) -> MethodId {
        match self.references[&cp_idx] {
            Reference::Unresolved => {
                let (class_idx, nat) = self.constant_pool.any_ref(cp_idx);
                let class_id = self.class_reference(class_idx);
                let (name, descriptor) = self.constant_pool.nat(nat);
                let method = resolve_method(class_id, &name, &descriptor);
                *self.references.get_mut(&cp_idx).unwrap() = Reference::Method(method);
                method
            }
            Reference::Method(method) => method,
            _ => unreachable!(),
        }
    }
}
