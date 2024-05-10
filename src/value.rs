use crate::class_file::descriptors::{BaseType, FieldType};
use crate::heap::{ArrayRef, ObjectRef};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Value {
    Byte(i8),
    Char(u8),
    Double(f64),
    Float(f32),
    Int(i32),
    Long(i64),
    Short(i16),
    Boolean(bool),
    Object(Option<ObjectRef>),
    Array(Option<ArrayRef>),
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

// :(
impl std::ops::Mul for Value {
    type Output = Value;

    fn mul(self, rhs: Value) -> Value {
        match (self, rhs) {
            (Value::Int(lhs), Value::Int(rhs)) => Value::Int(lhs.overflowing_mul(rhs).0),
            (Value::Long(lhs), Value::Long(rhs)) => Value::Long(lhs.overflowing_mul(rhs).0),
            (Value::Float(lhs), Value::Float(rhs)) => Value::Float(lhs * rhs),
            (Value::Double(lhs), Value::Double(rhs)) => Value::Double(lhs * rhs),
            _ => unreachable!(),
        }
    }
}

impl std::ops::Shr for Value {
    type Output = Value;

    fn shr(self, rhs: Value) -> Value {
        match (self, rhs) {
            (Value::Int(lhs), Value::Int(rhs)) => Value::Int(lhs >> rhs),
            (Value::Long(lhs), Value::Int(rhs)) => Value::Long(lhs >> rhs),
            _ => unreachable!(),
        }
    }
}

impl std::ops::Shl for Value {
    type Output = Value;

    fn shl(self, rhs: Value) -> Value {
        match (self, rhs) {
            (Value::Int(lhs), Value::Int(rhs)) => Value::Int(lhs << rhs),
            (Value::Long(lhs), Value::Int(rhs)) => Value::Long(lhs << rhs),
            _ => unreachable!(),
        }
    }
}

impl_val_op_binary!(std::ops::Add => fn add: Int, Long, Float, Double);
impl_val_op_binary!(std::ops::Sub => fn sub: Int, Long, Float, Double);
impl_val_op_binary!(std::ops::Div => fn div: Int, Long, Float, Double);
impl_val_op_binary!(std::ops::Rem => fn rem: Int, Long, Float, Double);
impl_val_op_binary!(std::ops::BitAnd => fn bitand: Int, Long);
impl_val_op_binary!(std::ops::BitOr => fn bitor: Int, Long);
impl_val_op_binary!(std::ops::BitXor => fn bitxor: Int, Long);
impl_val_op_unary!(std::ops::Neg => fn neg: Int, Long, Float, Double);

impl Value {
    pub fn ushr(self, rhs: Value) -> Value {
        match (self, rhs) {
            (Value::Int(lhs), Value::Int(rhs)) => {
                Value::Int((lhs as u32).overflowing_shr(rhs as u32).0 as i32)
            }
            (Value::Long(lhs), Value::Int(rhs)) => Value::Long((lhs as u64 >> rhs) as i64),
            _ => unreachable!(),
        }
    }

    pub fn default_for_ty(ty: &FieldType) -> Value {
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

    pub fn extend_32(self) -> Value {
        match self {
            Value::Byte(x) => Value::Int(x as i32),
            Value::Short(x) => Value::Int(x as i32),
            Value::Boolean(x) => Value::Int(x as i32),
            Value::Char(x) => Value::Int(x as i32),
            x => x,
        }
    }

    pub fn store_ty(self, ty: &FieldType) -> Value {
        if let Value::Int(int) = self {
            match ty {
                FieldType::BaseType(BaseType::I) => Value::Int(int),
                FieldType::BaseType(BaseType::B) => Value::Byte(int as i8),
                FieldType::BaseType(BaseType::S) => Value::Short(int as i16),
                FieldType::BaseType(BaseType::Z) => Value::Boolean((int & 0b1) == 1),
                FieldType::BaseType(BaseType::C) => Value::Char(int as u8),
                _ => unimplemented!(),
            }
        } else {
            self
        }
    }
}

/// Used for type checking loads and stores
pub trait MatchesFieldType {
    fn matches_field_type(field_type: &FieldType) -> bool;
}

macro_rules! impl_matches_field_type {
    ($field_pattern:pat, $ty:ty) => {
        impl MatchesFieldType for $ty {
            fn matches_field_type(field_type: &FieldType) -> bool {
                matches!(field_type, $field_pattern)
            }
        }
    };
}

impl_matches_field_type!(FieldType::BaseType(BaseType::B), i8);
impl_matches_field_type!(FieldType::BaseType(BaseType::C), u8);
impl_matches_field_type!(FieldType::BaseType(BaseType::D), f64);
impl_matches_field_type!(FieldType::BaseType(BaseType::F), f32);
impl_matches_field_type!(FieldType::BaseType(BaseType::I), i32);
impl_matches_field_type!(FieldType::BaseType(BaseType::J), i64);
impl_matches_field_type!(FieldType::ObjectType(_), Option<ObjectRef>);
impl_matches_field_type!(FieldType::ArrayType(_), Option<ArrayRef>);

macro_rules! unwrap_val {
    ($value_type:ident, $expr:expr) => {
        match ($expr) {
            Value::$value_type(x) => x,
            v => panic!("Unwrapped value with incorrect type: {:?}", v),
        }
    };
}

impl Value {
    pub fn byte(self) -> i8 {
        unwrap_val!(Byte, self)
    }

    pub fn char(self) -> u8 {
        unwrap_val!(Char, self)
    }

    pub fn double(self) -> f64 {
        unwrap_val!(Double, self)
    }

    pub fn float(self) -> f32 {
        unwrap_val!(Float, self)
    }

    pub fn int(self) -> i32 {
        unwrap_val!(Int, self)
    }

    pub fn long(self) -> i64 {
        unwrap_val!(Long, self)
    }

    pub fn short(self) -> i16 {
        unwrap_val!(Short, self)
    }

    pub fn boolean(self) -> bool {
        unwrap_val!(Boolean, self)
    }

    pub fn object(self) -> Option<ObjectRef> {
        match self {
            Value::Array(arr_ref) => arr_ref.map(|arr| arr.cast_to_object()),
            _ => unwrap_val!(Object, self),
        }
    }

    pub fn array(self) -> Option<ArrayRef> {
        unwrap_val!(Array, self)
    }
}
