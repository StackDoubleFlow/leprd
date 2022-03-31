use deku::prelude::*;
use deku::DekuContainerRead;

#[derive(DekuRead, Debug)]
#[deku(endian = "endian", ctx = "endian: deku::ctx::Endian")]
pub struct AttributeInfo {
    pub attribute_name_index: u16,
    pub attribute_length: u32,
    #[deku(count = "attribute_length")]
    pub info: Vec<u8>,
}

impl AttributeInfo {
    pub fn code(&self) -> CodeAttribute {
        CodeAttribute::from_bytes((&self.info, 0))
            .expect("ClassFormatError")
            .1
    }
}

#[derive(DekuRead, Debug)]
#[deku(endian = "endian", ctx = "endian: deku::ctx::Endian")]
pub struct ExceptionTableEntry {
    pub start_pc: u16,
    pub end_pc: u16,
    pub handler_pc: u16,
    pub catch_type: u16,
}

#[derive(DekuRead, Debug)]
#[deku(endian = "big")]
pub struct CodeAttribute {
    pub max_stack: u16,
    pub max_locals: u16,
    pub code_length: u32,
    #[deku(count = "code_length")]
    pub code: Vec<u8>,
    pub exception_table_length: u16,
    #[deku(count = "exception_table_length")]
    pub exception_table: Vec<ExceptionTableEntry>,
    pub attributes_count: u16,
    #[deku(count = "attributes_count")]
    pub attributes: Vec<AttributeInfo>,
}
