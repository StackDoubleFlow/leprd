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

    let mut heap = heap();
    let mut buf = vec![Value::Byte(0); length];
    buf.clone_from_slice(&heap.arrays[src].contents[src_pos..src_pos + length]);

    heap.arrays[dest].contents[dest_pos..dest_pos + length].clone_from_slice(&buf);
}
