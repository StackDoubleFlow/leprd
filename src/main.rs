mod class_file;

use class_file::ClassFile;
use deku::DekuContainerRead;
use std::fs::read;

fn main() {
    let data = read("./test/Test.class").unwrap();
    dbg!(ClassFile::from_bytes((&data, 0)).unwrap());
}
