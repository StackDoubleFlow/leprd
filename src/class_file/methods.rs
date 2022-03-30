use deku::prelude::*;
use super::attributes::AttributeInfo;

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
