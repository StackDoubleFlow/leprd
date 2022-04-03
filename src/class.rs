use crate::class_file::attributes::CodeAttribute;
use crate::class_file::ConstantPool;
use crate::class_loader::{ClassId, ClassLoader, FieldId, MethodId, method_area, resolve_class};
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
    pub fn field_reference(&mut self, cp_idx: u16) -> FieldId {
        match self.references[&cp_idx] {
            Reference::Unresolved => {
                let (class_idx, nat) = self.constant_pool.any_ref(cp_idx);
                let class_id = resolve_class(&self.constant_pool.class_name(class_idx));
                let (name, ..) = self.constant_pool.nat(nat);
                let field = find_field(class_id, &name).unwrap();
                *self.references.get_mut(&cp_idx).unwrap() = Reference::Field(field);
                field
            }
            Reference::Field(field) => field,
            _ => unreachable!()
        }
    }

    pub fn method_reference(&mut self, cp_idx: u16) -> MethodId {
        match self.references[&cp_idx] {
            Reference::Unresolved => {
                let (class_idx, nat) = self.constant_pool.any_ref(cp_idx);
                let class_id = resolve_class(&self.constant_pool.class_name(class_idx));
                let (name, descriptor) = self.constant_pool.nat(nat);
                let method = find_method(class_id, &name, &descriptor).unwrap();
                *self.references.get_mut(&cp_idx).unwrap() = Reference::Method(method);
                method
            }
            Reference::Method(method) => method,
            _ => unreachable!()
        }
    }
}

pub fn find_method(class: ClassId, name: &str, descriptor: &str) -> Option<MethodId> {
    let method_area = method_area();
    let class = &method_area.classes[class];
    class.methods.iter().copied().find(|&id| {
        let method = &method_area.methods[id];
        method.name == name && method.descriptor == descriptor
    })
}

pub fn find_field(class: ClassId, name: &str) -> Option<FieldId> {
    let method_area = method_area();
    let class = &method_area.classes[class];
    class.fields.iter().copied().find(|&id| {
        let field = &method_area.fields[id];
        field.name == name
    })
}
