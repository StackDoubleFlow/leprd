use deku::bitvec::{BitSlice, Msb0};
use deku::prelude::*;

#[derive(DekuRead, Debug)]
#[deku(endian = "endian", ctx = "endian: deku::ctx::Endian")]
pub struct ConstantPool {
    pub count: u16,
    #[deku(reader = "ConstantPool::read_table(deku::rest, *count, endian)")]
    pub table: Vec<CPInfo>,
}

impl ConstantPool {
    fn read_table(
        mut rest: &BitSlice<Msb0, u8>,
        count: u16,
        endian: deku::ctx::Endian,
    ) -> Result<(&BitSlice<Msb0, u8>, Vec<CPInfo>), DekuError> {
        let mut table = Vec::new();
        while table.len() < count as usize - 1 {
            let (new_rest, cp_info) = CPInfo::read(rest, endian)?;
            rest = new_rest;
            let long = matches!(cp_info, CPInfo::Long { .. } | CPInfo::Double { .. });
            table.push(cp_info);
            if long {
                table.push(CPInfo::Unusable);
            }
        }
        Ok((rest, table))
    }

    pub fn utf8(&self, idx: u16) -> String {
        match &self.table[idx as usize - 1] {
            CPInfo::Utf8 { bytes, .. } => String::from_utf8(bytes.clone()).unwrap(),
            _ => panic!("ClassFormatError"),
        }
    }

    pub fn class_name(&self, idx: u16) -> String {
        match self.table[idx as usize - 1] {
            CPInfo::Class { name_index } => self.utf8(name_index),
            _ => panic!("ClassFormatError"),
        }
    }

    pub fn any_ref(&self, idx: u16) -> (u16, u16) {
        match self.table[idx as usize - 1] {
            CPInfo::Fieldref { class_index, name_and_type_index } => (class_index, name_and_type_index),
            CPInfo::Methodref { class_index, name_and_type_index } => (class_index, name_and_type_index),
            CPInfo::InterfaceMethodref { class_index, name_and_type_index } => (class_index, name_and_type_index),
            _ => unreachable!()
        }
    }

    // Read NameAndType entry
    pub fn nat(&self, idx: u16) -> (String, String) {
        match self.table[idx as usize - 1] {
            CPInfo::NameAndType { name_index, descriptor_index } => (self.utf8(name_index), self.utf8(descriptor_index)),
            _ => unreachable!()
        }
    }
}

#[derive(DekuRead, Debug, PartialEq)]
#[deku(endian = "endian", ctx = "endian: deku::ctx::Endian")]
#[deku(type = "u8")]
pub enum CPInfo {
    #[deku(id = "1")]
    Utf8 {
        length: u16,
        #[deku(count = "length")]
        bytes: Vec<u8>,
    },
    #[deku(id = "3")]
    Integer { bytes: u32 },
    #[deku(id = "4")]
    Float { bytes: u32 },
    #[deku(id = "5")]
    Long { high_bytes: u32, low_bytes: u32 },
    #[deku(id = "6")]
    Double { high_bytes: u32, low_bytes: u32 },
    #[deku(id = "7")]
    Class { name_index: u16 },
    #[deku(id = "8")]
    String { string_index: u16 },
    #[deku(id = "9")]
    Fieldref {
        class_index: u16,
        name_and_type_index: u16,
    },
    #[deku(id = "10")]
    Methodref {
        class_index: u16,
        name_and_type_index: u16,
    },
    #[deku(id = "11")]
    InterfaceMethodref {
        class_index: u16,
        name_and_type_index: u16,
    },
    #[deku(id = "12")]
    NameAndType {
        name_index: u16,
        descriptor_index: u16,
    },
    #[deku(id = "15")]
    MethodHandle {
        reference_kind: u8,
        reference_index: u16,
    },
    #[deku(id = "16")]
    MethodType { descriptor_index: u16 },
    #[deku(id = "17")]
    Dynamic {
        bootstrap_method_attr_index: u16,
        name_and_type_index: u16,
    },
    #[deku(id = "18")]
    InvokeDynamic {
        bootstrap_method_attr_index: u16,
        name_and_type_index: u16,
    },
    #[deku(id = "19")]
    Module { name_index: u16 },
    #[deku(id = "20")]
    Package { name_index: u16 },
    #[deku(id = "0", reader = "CPInfo::unusable_tag()")]
    Unusable,
}

impl CPInfo {
    fn unusable_tag() -> ! {
        panic!("ClassFormatError");
    }
}

#[test]
fn cp_test() {
    #[derive(DekuRead, Debug, PartialEq)]
    #[deku(endian = "big")]
    struct CPTest {
        pub constant_pool_count: u16,
        #[deku(count = "constant_pool_count - 1")]
        pub constant_pool: Vec<CPInfo>,
    }
    let bytes = [0x00, 0x03, 0x07, 0x00, 0x00, 0x09, 0x00, 0x01, 0x00, 0x02];
    let cp = CPTest::from_bytes((&bytes, 0)).unwrap().1;
    assert_eq!(
        cp,
        CPTest {
            constant_pool_count: 3,
            constant_pool: vec![
                CPInfo::Class { name_index: 0 },
                CPInfo::Fieldref {
                    class_index: 1,
                    name_and_type_index: 2,
                }
            ]
        }
    );
}
