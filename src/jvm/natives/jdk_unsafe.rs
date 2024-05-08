use crate::class::FieldBacking;
use crate::class_loader::method_area;
use crate::heap::{arr_layout, heap};
use crate::jvm::Thread;
use crate::value::Value;

fn arr_class_layout(thread: &mut Thread) -> (usize, usize) {
    let Some(arr_class_obj) = thread.pop().object() else {
        panic!("NullPointerException");
    };
    let ma = method_area();
    let arr_class_id = ma.class_objs[&arr_class_obj];
    let arr_class = &ma.classes[arr_class_id];
    let elem_ty = arr_class
        .elem_ty
        .as_ref()
        .expect("class was not array class");
    let (_, base, stride) = arr_layout(elem_ty, 128);
    (base, stride)
}

pub fn array_index_scale(thread: &mut Thread) {
    let (_, stride) = arr_class_layout(thread);
    thread.operand_stack.push(Value::Int(stride as i32));
}

pub fn array_base_offset(thread: &mut Thread) {
    let (offset, _) = arr_class_layout(thread);
    thread.operand_stack.push(Value::Int(offset as i32));
}

pub fn object_field_offset(thread: &mut Thread) {
    let Some(name_str) = thread.pop().object() else {
        panic!("NullPointerException");
    };
    let Some(class_obj) = thread.pop().object() else {
        panic!("NullPointerException");
    };

    let name = heap().read_string(name_str);
    let ma = method_area();
    let class_id = ma.class_objs[&class_obj];
    let field_id = ma.resolve_field(class_id, &name);
    let FieldBacking::Instance(offset) = ma.fields[field_id].backing else {
        panic!("tried to get offset of static field");
    };

    thread.operand_stack.push(Value::Long(offset as i64));
}
