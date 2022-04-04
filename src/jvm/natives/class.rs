use std::collections::HashMap;
use std::lazy::SyncLazy;
use std::sync::Mutex;

use crate::class::Class;
use crate::class_loader::{method_area, ClassId};
use crate::heap::{heap, Object};
use crate::jvm::Thread;
use crate::value::Value;

pub fn desired_assertion_status(thread: &mut Thread) {
    let _class = thread.pop();
    // boolean type
    thread.operand_stack.push(Value::Int(0));
}

static PRIMITIVE_CLASSES: SyncLazy<Mutex<HashMap<String, ClassId>>> =
    SyncLazy::new(Default::default);

pub fn get_primitive_class(thread: &mut Thread) {
    let str_obj = match thread.pop() {
        Value::Object(Some(obj)) => obj,
        Value::Object(None) => panic!("NullPointerException"),
        _ => unreachable!(),
    };
    let str = Object::read_string(str_obj);
    let mut primitive_classes = PRIMITIVE_CLASSES.lock().unwrap();
    let class = primitive_classes.entry(str.clone()).or_insert_with(|| {
        let class_name = match str.as_str() {
            "byte" => "java/lang/Byte",
            "short" => "java/lang/Short",
            "int" => "java/lang/Int",
            "long" => "java/lang/Long",
            "char" => "java/lang/Char",
            "float" => "java/lang/Float",
            "double" => "java/lang/Double",
            "boolean" => "java/lang/Boolean",
            "void" => "java/lang/Void",
            _ => unreachable!(),
        };
        method_area().resolve_class(class_name)
    });
    thread
        .operand_stack
        .push(Value::Object(Some(Class::obj(*class))));
}
