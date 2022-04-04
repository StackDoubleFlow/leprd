#![feature(once_cell)]
#![feature(slice_group_by)]

use crate::class_loader::method_area;
use crate::jvm::Thread;

mod class;
mod class_file;
mod class_loader;
mod heap;
mod jvm;
mod value;

struct Config {
    classpath: &'static [&'static str],
    main_class: &'static str,
}

static CONFIG: Config = Config {
    classpath: &["./test", "./modules/java.base"],
    main_class: "Test",
};

fn main() {
    let mut ma = method_area();
    let system_class = ma.resolve_class("java/lang/System");
    let init_phase_1 = ma.resolve_method(system_class, "initPhase1", "()V");
    let class = ma.resolve_class(CONFIG.main_class);
    let method = ma.resolve_method(class, "main", "([Ljava/lang/String;)V");
    drop(ma);
    let mut thread = Thread::new(init_phase_1);
    println!("running thread");
    thread.run();
}
