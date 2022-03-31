use crate::class_file::ConstantPool;
use crate::class_loader::{ClassId, ClassLoader};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug)]
pub struct Field {
    pub access_flags: u16,
}

#[derive(Debug)]
pub struct Method {
    pub code: Option<Arc<[u8]>>,
    pub access_flags: u16,
}

#[derive(Debug)]
pub struct Class {
    pub constant_pool: ConstantPool,
    pub defining_loader: ClassLoader,
    pub name: String,
    pub super_class: Option<ClassId>,
    pub interfaces: Vec<ClassId>,
    pub access_flags: u16,
    pub methods: HashMap<String, Method>,
}
