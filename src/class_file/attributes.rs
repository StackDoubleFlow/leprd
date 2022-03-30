use deku::prelude::*;

#[derive(DekuRead, Debug)]
#[deku(endian = "endian", ctx = "endian: deku::ctx::Endian")]
pub struct AttributeInfo {
    pub attribute_name_index: u16,
    pub attribute_length: u32,
    #[deku(count = "attribute_length")]
    pub info: Vec<u8>,
}
