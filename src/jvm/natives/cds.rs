//! Class Data Sharing
//! https://docs.oracle.com/en/java/javase/18/vm/class-data-sharing.html

use crate::jvm::Thread;
use crate::value::Value;

pub fn is_dumping_class_list(thread: &mut Thread) {
    // boolean type
    thread.operand_stack.push(Value::Int(0));
}

pub fn is_dumping_archive(thread: &mut Thread) {
    // boolean type
    thread.operand_stack.push(Value::Int(0));
}

pub fn is_sharing_enabled(thread: &mut Thread) {
    // boolean type
    thread.operand_stack.push(Value::Int(0));
}
