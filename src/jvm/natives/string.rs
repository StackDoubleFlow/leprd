use crate::jvm::Thread;
use crate::value::Value;

pub fn is_big_endian(thread: &mut Thread) {
    let val = cfg!(target_endian = "big");
    // boolean type
    thread.operand_stack.push(Value::Int(val as i32));
}
