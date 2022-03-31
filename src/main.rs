#![feature(once_cell)]

mod class_file;
mod class_loader;
mod jvm;

use class_file::ClassFile;
use deku::DekuContainerRead;
use std::fs::read;

struct Config {
    classpath: &'static [&'static str],
    main_class: &'static str,
}

static CONFIG: Config = Config {
    classpath: &["./test", "./modules/java.base"],
    main_class: "Test",
};

fn main() {
    // class_loader::load_class_bootstrap("module-info");
    let class = class_loader::load_class_bootstrap(CONFIG.main_class);
    dbg!(class);
}
