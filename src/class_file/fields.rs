use super::attributes::AttributeInfo;
use deku::prelude::*;

pub mod acc {
    pub const PUBLIC: u16 = 0x0001;
    pub const PRIVATE: u16 = 0x0002;
    pub const PROTECTED: u16 = 0x0004;
    pub const STATIC: u16 = 0x0008;
    pub const FINAL: u16 = 0x0010;
    pub const VOLATILE: u16 = 0x0040;
    pub const TRANSIENT: u16 = 0x0080;
    pub const SYNTHETIC: u16 = 0x100;
    pub const ENUM: u16 = 0x4000;
}

#[derive(DekuRead, Debug)]
#[deku(endian = "endian", ctx = "endian: deku::ctx::Endian")]
pub struct FieldInfo {
    pub access_flags: u16,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes_count: u16,
    #[deku(count = "attributes_count")]
    pub attributes: Vec<AttributeInfo>,
}
