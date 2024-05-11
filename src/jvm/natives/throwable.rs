use crate::jvm::Thread;
use crate::value::Value;

pub fn fill_in_stack_trace(thread: &mut Thread) {
    let _dummy = thread.pop().int();
    let _obj = thread.pop().object();
    println!("todo: fill_in_stack_trace");
    thread.operand_stack.push(Value::Object(None))
}
