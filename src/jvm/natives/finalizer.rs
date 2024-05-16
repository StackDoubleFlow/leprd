use crate::jvm::Thread;
use crate::value::Value;

pub fn is_finalization_enabled(thread: &mut Thread) {
    // boolean value
    thread.operand_stack.push(Value::Int(0));
}
