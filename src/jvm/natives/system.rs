use crate::heap::heap;
use crate::jvm::Thread;
use crate::value::Value;

pub fn arraycopy(thread: &mut Thread) {
    let length = match thread.pop() {
        Value::Int(val) => val as usize,
        _ => panic!(),
    };
    let dest_pos = match thread.pop() {
        Value::Int(val) => val as usize,
        _ => panic!(),
    };
    let dest = match thread.pop() {
        Value::Array(Some(obj)) => obj,
        _ => panic!(),
    };
    let src_pos = match thread.pop() {
        Value::Int(val) => val as usize,
        _ => panic!(),
    };
    let src = match thread.pop() {
        Value::Array(Some(obj)) => obj,
        _ => panic!(),
    };

    heap().array_copy(src, src_pos, dest, dest_pos, length);
}
