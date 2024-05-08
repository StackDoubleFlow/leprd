use crate::class_file::descriptors::{FieldType, ObjectType};
use crate::heap::heap;
use crate::jvm::Thread;
use crate::value::Value;

pub fn vm_properties(thread: &mut Thread) {
    let arr = heap().new_array(
        FieldType::ObjectType(ObjectType {
            class_name: "java/lang/String".to_string(),
        }),
        0,
    );

    thread.operand_stack.push(Value::Array(Some(arr)));
}

pub fn platform_properties(thread: &mut Thread) {
    let arr = heap().new_array(
        FieldType::ObjectType(ObjectType {
            class_name: "java/lang/String".to_string(),
        }),
        39,
    );

    thread.operand_stack.push(Value::Array(Some(arr)));
}
