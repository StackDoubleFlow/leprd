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

macro_rules! impl_val_op_unary {
    ($trait:ty => fn $fn_name:ident: $($variant:ident),+) => {
        impl $trait for Value {
            type Output = Value;

            fn $fn_name(self) -> Value {
                match self {
                    $(
                        Value::$variant(val) => Value::$variant(val.$fn_name()),
                    )+
                    _ => unreachable!(),
                }
            }
        }
    };
}

macro_rules! impl_val_op_binary {
    ($trait:ty => fn $fn_name:ident: $($variant:ident),+) => {
        impl $trait for Value {
            type Output = Value;

            fn $fn_name(self, rhs: Value) -> Value {
                match (self, rhs) {
                    $(
                        (Value::$variant(lhs), Value::$variant(rhs)) => Value::$variant(lhs.$fn_name(rhs)),
                    )+
                    _ => unreachable!(),
                }
            }
        }
    };
}

impl_val_op_binary!(std::ops::Add => fn add: Int, Long, Float, Double);
impl_val_op_binary!(std::ops::Sub => fn sub: Int, Long, Float, Double);
impl_val_op_binary!(std::ops::Mul => fn mul: Int, Long, Float, Double);
impl_val_op_binary!(std::ops::Div => fn div: Int, Long, Float, Double);
impl_val_op_binary!(std::ops::Rem => fn rem: Int, Long, Float, Double);
impl_val_op_binary!(std::ops::BitAnd => fn bitand: Int, Long);
impl_val_op_binary!(std::ops::BitOr => fn bitor: Int, Long);
impl_val_op_binary!(std::ops::BitXor => fn bitxor: Int, Long);
impl_val_op_unary!(std::ops::Neg => fn neg: Int, Long, Float, Double);

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
