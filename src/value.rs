use crate::class_file::descriptors::{BaseType, FieldType};
use crate::heap::{ArrayId, ObjectId};

#[derive(Debug, Clone, Copy)]
pub enum Value {
    Byte(i8),
    Char(char),
    Double(f64),
    Float(f32),
    Int(i32),
    Long(i64),
    Short(i16),
    Boolean(bool),
    Object(Option<ObjectId>),
    Array(Option<ArrayId>),
}

impl Value {
    pub fn default_for_ty(ty: FieldType) -> Value {
        match ty {
            FieldType::BaseType(BaseType::B) => Value::Byte(Default::default()),
            FieldType::BaseType(BaseType::C) => Value::Char(Default::default()),
            FieldType::BaseType(BaseType::D) => Value::Double(Default::default()),
            FieldType::BaseType(BaseType::F) => Value::Float(Default::default()),
            FieldType::BaseType(BaseType::I) => Value::Int(Default::default()),
            FieldType::BaseType(BaseType::J) => Value::Long(Default::default()),
            FieldType::BaseType(BaseType::S) => Value::Short(Default::default()),
            FieldType::BaseType(BaseType::Z) => Value::Boolean(Default::default()),
            FieldType::ObjectType(_) => Value::Object(Default::default()),
            FieldType::ArrayType(_) => Value::Array(Default::default()),
        }
    }
}
