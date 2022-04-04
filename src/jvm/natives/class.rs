use crate::jvm::Thread;
use crate::value::Value;

pub fn desired_assertion_status(thread: &mut Thread) {
    let _class = thread.pop();
    // boolean type
    thread.operand_stack.push(Value::Int(0));
}
