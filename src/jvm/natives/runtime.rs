use crate::jvm::Thread;
use crate::value::Value;

pub fn available_processors(thread: &mut Thread) {
    // TODO: change if necessary
    thread.operand_stack.push(Value::Int(1));
}

pub fn max_memory(thread: &mut Thread) {
    thread.operand_stack.push(Value::Long(i64::MAX));
}