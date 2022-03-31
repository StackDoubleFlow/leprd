#![feature(once_cell)]

mod class;
mod class_file;
mod class_loader;
mod jvm;

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
    dbg!(&class_loader::method_area().classes[class]);
}
