use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use crate::class::Class;
use crate::class_loader::{method_area, ClassId};
use crate::heap::heap;
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
    let Some(str_obj) = thread.pop().object() else {
        panic!("NullPointerException");
    };
    let str = heap().read_string(str_obj);
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

pub fn is_primitive(thread: &mut Thread) {
    let Some(class_obj) = thread.pop().object() else {
        panic!("NullPointerException");
    };
    let class = method_area().class_objs[&class_obj];

    let primitive_classes = PRIMITIVE_CLASSES.lock().unwrap();
    let is_primitive = primitive_classes
        .values()
        .find(|&&primitive_class| primitive_class == class)
        .is_some();

    // boolean type
    thread.operand_stack.push(Value::Int(is_primitive as i32));
}

pub fn init_class_name(thread: &mut Thread) {
    let Some(class_class_obj) = thread.pop().object() else {
        panic!("NullPointerException");
    };
    let mut ma = method_area();
    let class_class = ma.resolve_class("java/lang/Class");
    let name_field = ma.resolve_field(class_class, "name");

    let class = ma.class_objs[&class_class_obj];
    let name = ma.classes[class].name.clone();
    let str_obj = Value::Object(Some(heap().create_string(&mut ma, &name)));
    heap().store_field(&ma, class_class_obj, name_field, str_obj);
    thread.operand_stack.push(str_obj);
}
