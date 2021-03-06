use crate::class::Class;
use crate::heap::heap;
use crate::jvm::Thread;
use crate::value::Value;

pub fn get_class(thread: &mut Thread) {
    let obj = match thread.pop() {
        Value::Object(Some(obj)) => obj,
        _ => panic!(),
    };
    let class = heap().objects[obj].class;
    thread
        .operand_stack
        .push(Value::Object(Some(Class::obj(class))));
}

pub fn hash_code(thread: &mut Thread) {
    let _obj = match thread.pop() {
        Value::Object(Some(obj)) => obj,
        _ => panic!(),
    };
    // lmaoxd
    thread.operand_stack.push(Value::Int(69));
}
