use std::sync::atomic::{fence, AtomicI32, AtomicI64, AtomicPtr, Ordering};
use crate::class::FieldBacking;
use crate::class_loader::method_area;
use crate::heap::{arr_layout, heap, Object, ObjectRef};
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

pub fn full_fence(_thread: &mut Thread) {
    fence(Ordering::SeqCst);
}

macro_rules! cas {
    ($exchange_name:ident, $set_name:ident, $val_fn:ident, $ty:ty, $val_ty:ident, $atomic_ty:ty) => {
        pub fn $exchange_name(thread: &mut Thread) {
            let x = thread.pop().$val_fn();
            let expected = thread.pop().$val_fn();
            let offset = thread.pop().long() as isize;
            let Some(obj_ref) = thread.pop().object() else {
                panic!("NullPointerException");
            };

            let atomic = unsafe {
                let ptr = obj_ref.inner_ptr().cast::<$ty>().byte_offset(offset);
                <$atomic_ty>::from_ptr(ptr)
            };
            let res = atomic.compare_exchange(expected, x, Ordering::SeqCst, Ordering::SeqCst);
            let val = match res {
                Result::Ok(val) => val,
                Result::Err(val) => val,
            };
            thread.operand_stack.push(Value::$val_ty(val));
        }

        pub fn $set_name(thread: &mut Thread) {
            let x = thread.pop().$val_fn();
            let expected = thread.pop().$val_fn();
            let offset = thread.pop().long() as isize;
            let Some(obj_ref) = thread.pop().object() else {
                panic!("NullPointerException");
            };

            let atomic = unsafe {
                let ptr = obj_ref.inner_ptr().cast::<$ty>().byte_offset(offset);
                <$atomic_ty>::from_ptr(ptr)
            };
            let res = atomic.compare_exchange(expected, x, Ordering::SeqCst, Ordering::SeqCst);
            thread.operand_stack.push(Value::Int(res.is_ok() as i32));
        }
    };
}

cas!(compare_and_exchange_int, compare_and_set_int, int, i32, Int, AtomicI32);
cas!(compare_and_exchange_long, compare_and_set_long, long, i64, Long, AtomicI64);

fn object_ptr(op: Option<ObjectRef>) -> *mut Object {
    match op {
        Some(x_ref) => x_ref.inner_ptr(),
        None => std::ptr::null_mut(),
    }
}

pub fn compare_and_set_reference(thread: &mut Thread) {
    let x = object_ptr(thread.pop().object());
    let expected = object_ptr(thread.pop().object());
    let offset = thread.pop().long() as isize;
    let Some(obj_ref) = thread.pop().object() else {
        panic!("NullPointerException");
    };

    let atomic = unsafe {
        let ptr = obj_ref.inner_ptr().cast::<*mut Object>().byte_offset(offset);
        AtomicPtr::from_ptr(ptr)
    };
    let res = atomic.compare_exchange(expected, x, Ordering::SeqCst, Ordering::SeqCst);
    thread.operand_stack.push(Value::Int(res.is_ok() as i32));
}

pub fn get_reference_volatile(thread: &mut Thread) {
    let offset = thread.pop().long() as isize;
    let Some(obj_ref) = thread.pop().object() else {
        panic!("NullPointerException");
    };

    let val = unsafe {
        let ptr = obj_ref.inner_ptr().cast::<*mut Object>().byte_offset(offset);
        ObjectRef::from_ptr(ptr.read_volatile())
    };

    thread.operand_stack.push(Value::Object(val))
}

pub fn put_reference_volatile(thread: &mut Thread) {
    let x = thread.pop().object();
    let offset = thread.pop().long() as isize;
    let Some(obj_ref) = thread.pop().object() else {
        panic!("NullPointerException");
    };

    unsafe {
        let val = object_ptr(x);
        let ptr = obj_ref.inner_ptr().cast::<*mut Object>().byte_offset(offset);
        ptr.write_volatile(val);
    }
}