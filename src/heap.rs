use crate::class::FieldBacking;
use crate::class_file::descriptors::{BaseType, FieldType};
use crate::class_loader::{method_area, ClassId, FieldId};
use crate::value::{MatchesFieldType, Value};
use std::alloc::{alloc_zeroed, Layout};
use std::sync::{LazyLock, Mutex, MutexGuard};

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
        },
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

unsafe fn store_value(ptr: *mut u8, val: Value) {
    match val {
        Value::Byte(val) => ptr.cast::<i8>().write(val),
        Value::Char(val) => ptr.cast::<u8>().write(val),
        Value::Double(val) => ptr.cast::<f64>().write(val),
        Value::Float(val) => ptr.cast::<f32>().write(val),
        Value::Int(val) => ptr.cast::<i32>().write(val),
        Value::Long(val) => ptr.cast::<i64>().write(val),
        Value::Short(val) => ptr.cast::<i16>().write(val),
        Value::Boolean(val) => ptr.cast::<bool>().write(val),
        Value::Array(arr_ref) => {
            let arr_ptr = arr_ref.map_or(std::ptr::null_mut(), |r| r.0);
            let ptr = ptr.cast::<*mut Array>();
            ptr.write(arr_ptr);
        }
        Value::Object(obj_ref) => {
            let obj_ptr = obj_ref.map_or(std::ptr::null_mut(), |r| r.0);
            let ptr = ptr.cast::<*mut Object>();
            ptr.write(obj_ptr);
        }
    }
}

unsafe fn load_value(ptr: *const u8, ty: &FieldType) -> Value {
    match ty {
        FieldType::BaseType(ty) => match ty {
            BaseType::B => Value::Byte(ptr.cast::<i8>().read()),
            BaseType::C => Value::Char(ptr.cast::<u8>().read()),
            BaseType::D => Value::Double(ptr.cast::<f64>().read()),
            BaseType::F => Value::Float(ptr.cast::<f32>().read()),
            BaseType::I => Value::Int(ptr.cast::<i32>().read()),
            BaseType::J => Value::Long(ptr.cast::<i64>().read()),
            BaseType::S => Value::Short(ptr.cast::<i16>().read()),
            BaseType::Z => Value::Boolean(ptr.cast::<bool>().read()),
        },
        FieldType::ArrayType(_) => {
            let arr_ptr = ptr.cast::<*mut Array>().read();
            Value::Array(ArrayRef::from_ptr(arr_ptr))
        }
        FieldType::ObjectType(_) => {
            let obj_ptr = ptr.cast::<*mut Object>().read();
            Value::Object(ObjectRef::from_ptr(obj_ptr))
        }
    }
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
        let layout =
            Layout::from_size_align(class.size as usize, class.alignment as usize).unwrap();
        let object = Object { class: class_id };
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
        let (layout, offset, _) = arr_layout(&elem_ty, len);

        let array = Array {
            obj: Object {
                class: method_area().resolve_arr_class(&elem_ty),
            },
            ty: elem_ty,
            len,
            offset,
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

    pub fn arr_len(&self, arr: ArrayRef) -> usize {
        unsafe { (*arr.0).len }
    }

    pub fn arr_ty(&self, arr: ArrayRef) -> &FieldType {
        unsafe { &(*arr.0).ty }
    }

    pub fn get_obj_class(&self, obj_ref: ObjectRef) -> ClassId {
        unsafe { (*obj_ref.0).class }
    }

    pub fn array_copy(
        &mut self,
        src_ref: ArrayRef,
        src_idx: usize,
        dst_ref: ArrayRef,
        dst_idx: usize,
        len: usize,
    ) {
        unsafe {
            let src_arr = &*src_ref.0;
            let dst_arr = &*dst_ref.0;
            assert_eq!(src_arr.ty, dst_arr.ty);
            assert!(src_idx + len <= src_arr.len);
            assert!(dst_idx + len <= dst_arr.len);

            let elem_layout = layout_for_field(&src_arr.ty);
            let (span_layout, stride) = elem_layout.repeat(len).unwrap();

            let src_ptr = src_ref
                .0
                .cast::<u8>()
                .offset((src_arr.offset + src_idx * stride) as isize);
            let dst_ptr = dst_ref
                .0
                .cast::<u8>()
                .offset((dst_arr.offset + dst_idx * stride) as isize);
            std::ptr::copy(src_ptr, dst_ptr, span_layout.size());
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
            load_value(field_ptr.cast::<u8>(), ty)
        }
    }

    pub fn store_field(&mut self, obj_ref: ObjectRef, field_id: FieldId, val: Value) {
        let field = &method_area().fields[field_id];
        let offset = match field.backing {
            FieldBacking::Instance(offset) => offset,
            _ => panic!("tried to store into a static field with instance obj"),
        };

        unsafe {
            let field_ptr = obj_ref.0.byte_offset(offset as isize);
            store_value(field_ptr.cast::<u8>(), val);
        }
    }

    unsafe fn arr_elem_ptr(arr: &mut Array, idx: usize) -> *mut u8 {
        let (_, offset, stride) = arr_layout(&arr.ty, arr.len);
        (arr as *mut Array)
            .cast::<u8>()
            .offset((offset + idx * stride) as isize)
    }

    pub fn load_arr_elem(&self, arr_ref: ArrayRef, idx: usize) -> Value {
        unsafe {
            let arr = &mut *arr_ref.0;
            let elem_ptr = Self::arr_elem_ptr(arr, idx);
            load_value(elem_ptr, &arr.ty)
        }
    }

    pub fn store_arr_elem(&self, arr_ref: ArrayRef, idx: usize, val: Value) {
        unsafe {
            let arr = &mut *arr_ref.0;
            let elem_ptr = Self::arr_elem_ptr(arr, idx);
            store_value(elem_ptr, val);
        }
    }

    pub unsafe fn array_contents_unchecked<T>(&mut self, arr_ref: ArrayRef) -> &mut [T] {
        let arr = &*arr_ref.0;
        let data_ptr = arr_ref.0.byte_offset(arr.offset as isize);
        std::slice::from_raw_parts_mut(data_ptr.cast::<T>(), arr.len)
    }

    pub fn array_contents<T: MatchesFieldType>(&mut self, arr_ref: ArrayRef) -> &mut [T] {
        let elem_ty = &unsafe { &*arr_ref.0 }.ty;
        T::matches_field_type(&elem_ty);
        unsafe { self.array_contents_unchecked(arr_ref) }
    }

    pub fn read_string(&mut self, str_obj: ObjectRef) -> String {
        let str_class = self.get_obj_class(str_obj);

        let ma = method_area();
        let value_field = ma.resolve_field(str_class, "value");
        let coder_field = ma.resolve_field(str_class, "coder");
        drop(ma);

        let value = match self.load_field(str_obj, value_field) {
            Value::Array(Some(arr)) => arr,
            _ => unreachable!(),
        };
        let coder = match self.load_field(str_obj, coder_field) {
            Value::Byte(val) => val,
            _ => unreachable!(),
        };
        if coder == 0 {
            let utf8 = self
                .array_contents(value)
                .iter()
                .map(|c: &i8| *c as u8)
                .collect();
            String::from_utf8(utf8).unwrap()
        } else {
            let utf16 = self
                .array_contents(value)
                .chunks_exact(2)
                .map(|c| match c {
                    &[a, b] => u16::from_ne_bytes([a, b]),
                    _ => unreachable!(),
                })
                .collect::<Vec<_>>();
            String::from_utf16(&utf16).unwrap()
        }
    }
}

/// We CANNOT keep these around between GC runs unless it is somewhere the GC can see,
/// i.e. in an object or array.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ObjectRef(*mut Object);

impl ObjectRef {
    pub fn inner_ptr(self) -> *mut Object {
        self.0
    }

    pub unsafe fn from_ptr(ptr: *mut Object) -> Option<ObjectRef> {
        (!ptr.is_null()).then_some(ObjectRef(ptr))
    }
}

unsafe impl Send for ObjectRef {}
unsafe impl Sync for ObjectRef {}

/// We CANNOT keep these around between GC runs unless it is somewhere the GC can see,
/// i.e. in an object or array.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct ArrayRef(*mut Array);

impl ArrayRef {
    pub unsafe fn inner_ptr(self) -> *mut Array {
        self.0
    }

    pub unsafe fn from_ptr(ptr: *mut Array) -> Option<ArrayRef> {
        (!ptr.is_null()).then_some(ArrayRef(ptr))
    }

    pub fn cast_to_object(self) -> ObjectRef {
        ObjectRef(self.0.cast::<Object>())
    }
}

unsafe impl Send for ArrayRef {}
unsafe impl Sync for ArrayRef {}

#[derive(Debug)]
pub struct Object {
    class: ClassId,
}

#[derive(Debug)]
pub struct Array {
    // Having Object as the first field allows us to cast array pointers to object pointers
    obj: Object,
    ty: FieldType,
    len: usize,
    offset: usize,
}
