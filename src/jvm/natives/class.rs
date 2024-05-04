use std::collections::HashMap;
use std::sync::LazyLock;
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

static PRIMITIVE_CLASSES: LazyLock<Mutex<HashMap<String, ClassId>>> =
    LazyLock::new(Default::default);

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
            "int" => "java/lang/Integer",
            "long" => "java/lang/Long",
            "char" => "java/lang/Character",
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

pub fn init_class_name(thread: &mut Thread) {
    let class_class_obj = match thread.pop() {
        Value::Object(Some(obj)) => obj,
        _ => panic!(),
    };
    let mut ma = method_area();
    let class_class = ma.resolve_class("java/lang/Class");
    let name_field = ma.resolve_field(class_class, "name");

    let class = ma.class_objs[&class_class_obj];
    let name = ma.classes[class].name.clone();
    drop(ma);
    let str_obj = Value::Object(Some(thread.create_string(&name)));
    heap().objects[class_class_obj]
        .fields
        .insert(name_field, str_obj);
    thread.operand_stack.push(str_obj);
}
