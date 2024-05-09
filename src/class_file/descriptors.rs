use std::alloc::Layout;

#[test]
fn method_desc_test() {
    let desc = MethodDescriptor::read("([CII)V");
    assert_eq!(
        desc,
        MethodDescriptor(
            vec![
                ParameterDescriptor(FieldType::ArrayType(ArrayType(Box::new(ComponentType(
                    FieldType::BaseType(BaseType::C)
                ))))),
                ParameterDescriptor(FieldType::BaseType(BaseType::I)),
                ParameterDescriptor(FieldType::BaseType(BaseType::I))
            ],
            ReturnDescriptor::Void
        )
    )
}

#[derive(Debug, PartialEq)]
pub struct MethodDescriptor(pub Vec<ParameterDescriptor>, pub ReturnDescriptor);

impl MethodDescriptor {
    pub fn read(s: &str) -> MethodDescriptor {
        let mut chars = s.chars().skip(1).peekable();
        let mut params = Vec::new();
        loop {
            match chars.peek() {
                Some(')') => break,
                Some(_) => params.push(ParameterDescriptor(FieldType::read(&mut chars))),
                None => unreachable!(),
            }
        }
        chars.next();
        let return_des = match chars.peek() {
            Some('V') => ReturnDescriptor::Void,
            Some(_) => ReturnDescriptor::FieldType(FieldType::read(&mut chars)),
            None => unreachable!(),
        };

        MethodDescriptor(params, return_des)
    }
}

#[derive(Debug, PartialEq)]
pub struct ParameterDescriptor(FieldType);

#[derive(Debug, PartialEq)]
pub enum ReturnDescriptor {
    FieldType(FieldType),
    Void,
}

#[derive(Debug, PartialEq)]
pub struct FieldDescriptor(pub FieldType);

impl FieldDescriptor {
    pub fn read(s: &str) -> FieldDescriptor {
        let mut chars = s.chars();
        FieldDescriptor(FieldType::read(&mut chars))
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum FieldType {
    BaseType(BaseType),
    ObjectType(ObjectType),
    ArrayType(ArrayType),
}

impl FieldType {
    fn read(cs: &mut impl Iterator<Item = char>) -> FieldType {
        match cs.next().unwrap() {
            'B' => FieldType::BaseType(BaseType::B),
            'C' => FieldType::BaseType(BaseType::C),
            'D' => FieldType::BaseType(BaseType::D),
            'F' => FieldType::BaseType(BaseType::F),
            'I' => FieldType::BaseType(BaseType::I),
            'J' => FieldType::BaseType(BaseType::J),
            'L' => {
                let mut class_name = String::new();
                loop {
                    match cs.next() {
                        Some(';') => break FieldType::ObjectType(ObjectType { class_name }),
                        Some(c) => class_name.push(c),
                        None => unreachable!(),
                    }
                }
            }
            'S' => FieldType::BaseType(BaseType::S),
            'Z' => FieldType::BaseType(BaseType::Z),
            '[' => FieldType::ArrayType(ArrayType(Box::new(ComponentType(FieldType::read(cs))))),
            c => unreachable!("field type {}", c),
        }
    }
}

impl std::fmt::Display for FieldType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FieldType::BaseType(base_type) => write!(f, "{}", base_type),
            FieldType::ArrayType(arr_type) => write!(f, "{}[]", arr_type.0 .0),
            FieldType::ObjectType(obj_type) => write!(f, "{}", obj_type.class_name),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum BaseType {
    /// byte
    B,
    /// char
    C,
    /// double
    D,
    /// float
    F,
    /// int
    I,
    /// long
    J,
    /// short
    S,
    /// boolean
    Z,
}

impl std::fmt::Display for BaseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                BaseType::B => "byte",
                BaseType::C => "char",
                BaseType::D => "double",
                BaseType::F => "float",
                BaseType::I => "int",
                BaseType::J => "long",
                BaseType::S => "short",
                BaseType::Z => "boolean",
            }
        )
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct ObjectType {
    pub class_name: String,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct ArrayType(pub Box<ComponentType>);

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct ComponentType(pub FieldType);
