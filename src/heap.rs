use crate::class_file::fields;
use crate::class_loader::{method_area, ClassId, FieldId};
use crate::value::Value;
use id_arena::{Arena, Id};
use std::collections::HashMap;
use std::lazy::SyncLazy;
use std::sync::{Mutex, MutexGuard};

static HEAP: SyncLazy<Mutex<Heap>> = SyncLazy::new(Default::default);

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
}

#[derive(Debug)]
pub struct Array {
    pub contents: Box<[Value]>,
}
