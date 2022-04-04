pub mod attributes;
pub mod constant_pool;
pub mod descriptors;
pub mod fields;
pub mod methods;

use deku::prelude::*;

pub use attributes::AttributeInfo;
pub use constant_pool::ConstantPool;
pub use fields::FieldInfo;
pub use methods::MethodInfo;

// Note: these are access modifiers from classes, fields, and methods mixed
pub const ACC_PUBLIC: u16 = 0x0001;
// pub const ACC_PRIVATE: u16 = 0x0002;
// pub const ACC_PROTECTED: u16 = 0x0004;
// pub const ACC_STATIC: u16 = 0x0008;
pub const ACC_FINAL: u16 = 0x0010;
pub const ACC_SUPER: u16 = 0x0020;
// pub const ACC_VOLATILE: u16 = 0x0040;
// pub const ACC_TRANSIENT: u16 = 0x0080;
// pub const ACC_NATIVE: u16 = 0x0100;
pub const ACC_INTERFACE: u16 = 0x0200;
pub const ACC_ABSTRACT: u16 = 0x0400;
// pub const ACC_STRICT: u16 = 0x0800;
pub const ACC_SYNTHETIC: u16 = 0x1000;
pub const ACC_ANNOTATION: u16 = 0x2000;
pub const ACC_ENUM: u16 = 0x4000;
pub const ACC_MODULE: u16 = 0x8000;

#[derive(Debug, DekuRead)]
#[deku(endian = "big")]
pub struct ClassFile {
    pub magic: u32,
    pub minor_version: u16,
    pub major_version: u16,
    pub constant_pool: ConstantPool,
    pub access_flags: u16,
    pub this_class: u16,
    pub super_class: u16,
    pub interfaces_count: u16,
    #[deku(count = "interfaces_count")]
    pub interfaces: Vec<u16>,
    pub fields_count: u16,
    #[deku(count = "fields_count")]
    pub fields: Vec<FieldInfo>,
    pub methods_count: u16,
    #[deku(count = "methods_count")]
    pub methods: Vec<MethodInfo>,
    pub attributes_count: u16,
    #[deku(count = "attributes_count")]
    pub attributes: Vec<AttributeInfo>,
}
