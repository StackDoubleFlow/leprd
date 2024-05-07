use crate::heap::heap;
use crate::jvm::Thread;

pub fn arraycopy(thread: &mut Thread) {
    let length = thread.pop().int() as usize;
    let dest_pos = thread.pop().int() as usize;
    let Some(dest) = thread.pop().array() else {
        panic!("NullPointerException");
    };
    let src_pos = thread.pop().int() as usize;
    let Some(src) = thread.pop().array() else {
        panic!("NullPointerException");
    };

    heap().array_copy(src, src_pos, dest, dest_pos, length);
}
