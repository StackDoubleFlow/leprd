use crate::class::Class;
use crate::heap::heap;
use crate::jvm::Thread;
use crate::value::Value;

pub fn get_class(thread: &mut Thread) {
    let Some(obj) = thread.pop().object() else {
        panic!("NullPointerException");
    };
    let class = heap().get_obj_class(obj);
    thread
        .operand_stack
        .push(Value::Object(Some(Class::obj(class))));
}

pub fn hash_code(thread: &mut Thread) {
    let Some(_obj) = thread.pop().object() else {
        panic!("NullPointerException");
    };
    // lmaoxd
    // FIXME: According to the spec, this is technically valid,
    // but will kill the performance of hashtables.
    thread.operand_stack.push(Value::Int(69));
}

pub fn clone(thread: &mut Thread) {
    let Some(obj) = thread.pop().object() else {
        panic!("NullPointerException");
    };
    let new_obj = heap().clone_object(obj);
    thread.operand_stack.push(Value::Object(Some(new_obj)));
}