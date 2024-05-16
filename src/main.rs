#![feature(lazy_cell, alloc_layout_extra)]

use crate::class_file::descriptors::MethodDescriptor;
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

fn initialize_class(thread: &mut Thread, name: &str) {
    let class = method_area().resolve_class(name);
    thread.ensure_initialized(class);
}

fn main() {
    let mut ma = method_area();
    let system_class = ma.resolve_class("java/lang/System");
    let init_phase_1 =
        ma.resolve_method(system_class, "initPhase1", &MethodDescriptor::read("()V"));
    drop(ma);
    let mut thread = Thread::new(init_phase_1);
    initialize_class(&mut thread, "java/lang/System");
    initialize_class(&mut thread, "java/lang/ref/Finalizer");
    println!("running thread");
    thread.run();

    initialize_class(&mut thread, &CONFIG.main_class);
    let mut ma = method_area();
    let class = ma.resolve_class(CONFIG.main_class);
    let method = ma.resolve_method(
        class,
        "main",
        &MethodDescriptor::read("([Ljava/lang/String;)V"),
    );
    thread.call_method(method);
}
