use crate::class_file::descriptors::{BaseType, FieldType};
use crate::class_file::fields;
use crate::class_loader::{method_area, ClassId, FieldId};
use crate::value::Value;
use id_arena::{Arena, Id};
use std::collections::HashMap;
use std::sync::LazyLock;
use std::sync::{Mutex, MutexGuard};

static HEAP: LazyLock<Mutex<Heap>> = LazyLock::new(Default::default);

/// Retrieve the heap by locking the mutex
pub fn heap() -> MutexGuard<'static, Heap> {
    HEAP.lock().unwrap()
}

#[derive(Default, Debug)]
pub struct Heap {
    pub objects: Arena<Object>,
    pub arrays: Arena<Array>,
}

pub type ObjectId = Id<Object>;
pub type ArrayId = Id<Array>;

#[derive(Debug)]
pub struct Object {
    pub class: ClassId,
    pub fields: HashMap<FieldId, Value>,
}

impl Object {
    pub fn new(class: ClassId) -> ObjectId {
        let mut fields = HashMap::new();

        let ma = method_area();
        let mut cur_class = class;
        loop {
            let class = &ma.classes[cur_class];
            for &field_id in &class.fields {
                let field = &ma.fields[field_id];
                if field.access_flags & fields::acc::STATIC == 0 {
                    fields.insert(field_id, Value::default_for_ty(&field.descriptor.0));
                }
            }
            if let Some(parent) = class.super_class {
                cur_class = parent;
            } else {
                break;
            }
        }

        let object = Object { class, fields };
        heap().objects.alloc(object)
    }

    pub fn store_field(&mut self, field: FieldId, val: Value) {
        let ty = &method_area().fields[field].descriptor.0;
        self.fields.insert(field, val.store_ty(ty));
    }

    pub fn read_string(str_obj: ObjectId) -> String {
        let str_class = heap().objects[str_obj].class;

        let ma = method_area();
        let value_field = ma.resolve_field(str_class, "value");
        let coder_field = ma.resolve_field(str_class, "coder");
        drop(ma);

        let mut heap = heap();
        let str = &mut heap.objects[str_obj];
        let value = match str.fields[&value_field] {
            Value::Array(Some(arr)) => arr,
            _ => unreachable!(),
        };
        let coder = match str.fields[&coder_field] {
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
    pub contents: Box<[Value]>,
}
