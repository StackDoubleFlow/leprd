use crate::class::FieldBacking;
use crate::class_file::descriptors::{BaseType, FieldType};
use crate::class_file::fields;
use crate::class_loader::{method_area, ClassId, FieldId, MethodArea};
use crate::value::Value;
use id_arena::{Arena, Id};
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex, MutexGuard};
use std::alloc::{self, alloc_zeroed, Layout};

static HEAP: LazyLock<Mutex<Heap>> = LazyLock::new(Default::default);

/// Retrieve the heap by locking the mutex
pub fn heap() -> MutexGuard<'static, Heap> {
    HEAP.lock().unwrap()
}

fn layout_for_field(field_ty: &FieldType) -> Layout {
    match field_ty {
        FieldType::BaseType(ty) => match ty {
            BaseType::B => Layout::new::<i8>(),
            BaseType::C => Layout::new::<u8>(),
            BaseType::D => Layout::new::<f64>(),
            BaseType::F => Layout::new::<f32>(),
            BaseType::I => Layout::new::<i32>(),
            BaseType::J => Layout::new::<i64>(),
            BaseType::S => Layout::new::<i16>(),
            BaseType::Z => Layout::new::<bool>(),
        }
        FieldType::ObjectType(_) => Layout::new::<*mut Object>(),
        FieldType::ArrayType(_) => Layout::new::<*mut Array>(),
    }
}

/// Gets the layout for an array in memory. Returns the complete layout of the array, the offset to the start of the elements, and the stride.
pub fn arr_layout(elem_ty: &FieldType, len: usize) -> (Layout, usize, usize) {
    let base_layout = Layout::new::<Array>();
    let (arr_layout, stride) = layout_for_field(elem_ty).repeat(len).unwrap();
    let (combined_layout, offset) = base_layout.extend(arr_layout).unwrap();
    (combined_layout, offset, stride)
}

#[derive(Default, Debug)]
pub struct Heap {
    pub objects: Vec<ObjectRef>,
    pub arrays: Vec<ArrayRef>,
}

impl Heap {
    pub fn new_object(&mut self, class_id: ClassId) -> ObjectRef {
        let ma = method_area();
        let class = &ma.classes[class_id];
        let layout = Layout::from_size_align(class.size as usize, class.alignment as usize).unwrap();
        let object = Object {
            class: class_id,
        };
        let object_ptr = unsafe {
            let ptr = alloc_zeroed(layout).cast::<Object>();
            (*ptr) = object;
            ptr
        };

        let obj_ref = ObjectRef(object_ptr);
        self.objects.push(obj_ref);
        obj_ref
    }

    pub fn new_array(&mut self, elem_ty: FieldType, len: usize) -> ArrayRef {
        let (layout, _, _) = arr_layout(&elem_ty, len);

        let array = Array {
            ty: elem_ty,
            len,
        };
        let array_ptr = unsafe {
            let ptr = alloc_zeroed(layout).cast::<Array>();
            (*ptr) = array;
            ptr
        };

        let arr_ref = ArrayRef(array_ptr);
        self.arrays.push(arr_ref);
        arr_ref
    }

    pub fn arr_len(&self, arr: ArrayRef) {
        unsafe { (*arr.0).len }
    }

    pub fn get_obj_class(&self, obj_ref: ObjectRef) -> ClassId {
        unsafe {
            (*obj_ref.0).class
        }
    }

    pub fn load_field(&self, obj_ref: ObjectRef, field_id: FieldId) -> Value {
        let field = &method_area().fields[field_id];
        let ty = &field.descriptor.0;
        let offset = match field.backing {
            FieldBacking::Instance(offset) => offset,
            _ => panic!("tried to load from a static field with instance obj"),
        };

        unsafe {
            let field_ptr = obj_ref.0.byte_offset(offset as isize);
            match ty {
                FieldType::BaseType(ty) => match ty {
                    BaseType::B => Value::Byte(field_ptr.cast::<i8>().read()),
                    BaseType::C => Value::Char(field_ptr.cast::<u8>().read()),
                    BaseType::D => Value::Double(field_ptr.cast::<f64>().read()),
                    BaseType::F => Value::Float(field_ptr.cast::<f32>().read()),
                    BaseType::I => Value::Int(field_ptr.cast::<i32>().read()),
                    BaseType::J => Value::Long(field_ptr.cast::<i64>().read()),
                    BaseType::S => Value::Short(field_ptr.cast::<i16>().read()),
                    BaseType::Z => Value::Boolean(field_ptr.cast::<bool>().read()),
                }
                FieldType::ArrayType(_) => {
                    let arr_ptr = field_ptr.cast::<*mut Array>().read();
                    Value::Array((!arr_ptr.is_null()).then_some(ArrayRef(arr_ptr)))
                }
                FieldType::ObjectType(_) => {
                    let obj_ptr = field_ptr.cast::<*mut Object>().read();
                    Value::Object((!obj_ptr.is_null()).then_some(ObjectRef(obj_ptr)))
                }
            }
        }
    }

    pub fn store_field(&mut self, obj_ref: ObjectRef, field_id: FieldId, val: Value) {
        let field = &method_area().fields[field_id];
        let ty = &field.descriptor.0;
        let offset = match field.backing {
            FieldBacking::Instance(offset) => offset,
            _ => panic!("tried to store into a static field with instance obj"),
        };

        unsafe {
            let field_ptr = obj_ref.0.byte_offset(offset as isize);
            match val.store_ty(ty) {
                Value::Byte(val) => field_ptr.cast::<i8>().write(val),
                Value::Char(val) => field_ptr.cast::<u8>().write(val),
                Value::Double(val) => field_ptr.cast::<f64>().write(val),
                Value::Float(val) => field_ptr.cast::<f32>().write(val),
                Value::Int(val) => field_ptr.cast::<i32>().write(val),
                Value::Long(val) => field_ptr.cast::<i64>().write(val),
                Value::Short(val) => field_ptr.cast::<i16>().write(val),
                Value::Boolean(val) => field_ptr.cast::<bool>().write(val),
                Value::Array(arr_ref) => {
                    let arr_ptr = arr_ref.map_or(std::ptr::null_mut(), |r| r.0);
                    let field_ptr = field_ptr.cast::<*mut Array>();
                    field_ptr.write(arr_ptr);
                }
                Value::Object(obj_ref) => {
                    let obj_ptr = obj_ref.map_or(std::ptr::null_mut(), |r| r.0);
                    let field_ptr = field_ptr.cast::<*mut Object>();
                    field_ptr.write(obj_ptr);
                }
            }
        }
    }
}

/// We CANNOT keep these around between GC runs unless it is somewhere the GC can see,
/// i.e. in an object or array.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObjectRef(*mut Object);

unsafe impl Send for ObjectRef {}
unsafe impl Sync for ObjectRef {}

/// We CANNOT keep these around between GC runs unless it is somewhere the GC can see,
/// i.e. in an object or array.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct ArrayRef(*mut Array);

unsafe impl Send for ArrayRef {}
unsafe impl Sync for ArrayRef {}

pub type ObjectId = Id<Object>;
pub type ArrayId = Id<Array>;

#[derive(Debug)]
pub struct Object {
    pub class: ClassId,
}

impl Object {
    pub fn read_string(str_obj: ObjectRef) -> String {
        let str_class = heap().get_obj_class(str_obj);

        let ma = method_area();
        let value_field = ma.resolve_field(str_class, "value");
        let coder_field = ma.resolve_field(str_class, "coder");
        drop(ma);

        let mut heap = heap();
        let value = match heap.load_field(str_obj, value_field) {
            Value::Array(Some(arr)) => arr,
            _ => unreachable!(),
        };
        let coder = match heap.load_field(str_obj, coder_field) {
            Value::Byte(val) => val,
            _ => unreachable!(),
        };
        if coder == 0 {
            let utf8 = heap.arrays[value]
                .contents
                .iter()
                .map(|x| match x {
                    Value::Byte(x) => *x as u8,
                    _ => unreachable!(),
                })
                .collect();
            String::from_utf8(utf8).unwrap()
        } else {
            let utf16 = heap.arrays[value]
                .contents
                .chunks_exact(2)
                .map(|c| match c {
                    &[Value::Byte(a), Value::Byte(b)] => u16::from_ne_bytes([a as u8, b as u8]),
                    _ => unreachable!(),
                })
                .collect::<Vec<_>>();
            String::from_utf16(&utf16).unwrap()
        }
    }
}

#[derive(Debug)]
pub struct Array {
    pub ty: FieldType,
    pub len: usize,
}
