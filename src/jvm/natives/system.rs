use crate::class::FieldBacking;
use crate::class_loader::method_area;
use crate::heap::heap;
use crate::jvm::Thread;
use crate::value::Value;

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

fn set_static(thread: &mut Thread, name: &str) {
    let val = thread.pop().object();
    let class_id = thread.class_id();
    let mut ma = method_area();
    let field_id = ma.resolve_field(class_id, name);
    let field = match &mut ma.fields[field_id].backing {
        FieldBacking::StaticValue(val) => val,
        _ => panic!("System.{} is not static?", name),
    };
    *field = Value::Object(val);
}

pub fn set_in(thread: &mut Thread) {
    set_static(thread, "in");
}

pub fn set_out(thread: &mut Thread) {
    set_static(thread, "out");
}

pub fn set_err(thread: &mut Thread) {
    set_static(thread, "err");
}
