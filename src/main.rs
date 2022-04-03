#![feature(once_cell)]

use crate::class_loader::{resolve_class, resolve_method};
use crate::jvm::Thread;

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
    let class = resolve_class(CONFIG.main_class);
    let method = resolve_method(class, "main", "([Ljava/lang/String;)V");
    let mut thread = Thread::new(method);
    println!("running thread");
    thread.run();
}
