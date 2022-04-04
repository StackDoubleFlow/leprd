use std::str::Chars;

pub struct FieldDescriptor(pub FieldType);

impl FieldDescriptor {
    pub fn read(s: &str) -> FieldDescriptor {
        let mut chars = s.chars();
        FieldDescriptor(FieldType::read(&mut chars))
    }
}

pub enum FieldType {
    BaseType(BaseType),
    ObjectType(ObjectType),
    ArrayType(ArrayType),
}

impl FieldType {
    fn read(cs: &mut Chars) -> FieldType {
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
            _ => unreachable!(),
        }
    }
}

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

pub struct ObjectType {
    class_name: String,
}

pub struct ArrayType(Box<ComponentType>);

pub struct ComponentType(FieldType);
