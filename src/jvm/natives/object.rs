use crate::class::Class;
use crate::heap::heap;
use crate::jvm::Thread;
use crate::value::Value;

pub fn get_class(thread: &mut Thread) {
    let obj = match thread.pop() {
        Value::Object(Some(obj)) => obj,
        _ => panic!(),
    };
    let class = heap().get_obj_class(obj);
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
    // FIXME: According to the spec, this is technically valid,
    // but will kill the performance of hashtables.
    thread.operand_stack.push(Value::Int(69));
}
