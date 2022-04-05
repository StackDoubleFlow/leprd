use super::attributes::AttributeInfo;
use deku::prelude::*;

pub mod acc {
    pub const PUBLIC: u16 = 0x0001;
    pub const PRIVATE: u16 = 0x0002;
    pub const PROTECTED: u16 = 0x0004;
    pub const STATIC: u16 = 0x0008;
    pub const FINAL: u16 = 0x0010;
    pub const SYNCHRONIZED: u16 = 0x0020;
    pub const BRIDGE: u16 = 0x0040;
    pub const VARARGS: u16 = 0x0080;
    pub const NATIVE: u16 = 0x0100;
    pub const ABSTRACT: u16 = 0x0400;
    pub const STRICT: u16 = 0x0800;
    pub const SYNTHETIC: u16 = 0x1000;
}

#[derive(DekuRead, Debug)]
#[deku(endian = "endian", ctx = "endian: deku::ctx::Endian")]
pub struct MethodInfo {
    pub access_flags: u16,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes_count: u16,
    #[deku(count = "attributes_count")]
    pub attributes: Vec<AttributeInfo>,
}
