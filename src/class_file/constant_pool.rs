use deku::prelude::*;

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
    MethodType { tag: u8, descriptor_index: u16 },
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
