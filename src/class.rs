use crate::class_file::ConstantPool;
use crate::class_file::attributes::CodeAttribute;
use crate::class_loader::{ClassId, ClassLoader, MethodId, FieldId};
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
    pub references: HashMap<usize, Reference>,

    pub defining_loader: ClassLoader,
    pub name: String,
    pub super_class: Option<ClassId>,
    pub interfaces: Vec<ClassId>,
    pub access_flags: u16,
    pub methods: Vec<MethodId>,
    pub fields: Vec<FieldId>,
}
