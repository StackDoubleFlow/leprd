use crate::class::Class;
use crate::class_loader::{method_area, MethodArea};
use crate::jvm::Thread;
use crate::value::Value;

pub fn get_caller_class(thread: &mut Thread) {
    let last_frame = thread.stack_frames.last().unwrap();
    let ma = method_area();
    let method = &ma.methods[last_frame.method];
    let class_id = method.defining_class;
    drop(ma);
    let class_obj = Class::obj(class_id);
    // TODO: ignore frames relating to java.lang.reflect.Method.invoke()
    thread.operand_stack.push(Value::Object(Some(class_obj)));
}
