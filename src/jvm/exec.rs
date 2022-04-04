use super::Thread;
use crate::class::Class;
use crate::class_file::constant_pool::CPInfo;
use crate::class_file::descriptors::{BaseType, FieldType};
use crate::class_loader::method_area;
use crate::heap::{heap, Array, Object};
use crate::value::Value;

impl Thread {
    fn ldc(&mut self, cp_idx: u16) {
        let class_id = self.class_id();
        let ma = method_area();
        let cp_info = &ma.classes[class_id].constant_pool.table[cp_idx as usize - 1];
        let val = match *cp_info {
            CPInfo::Integer { val } => Value::Int(val),
            CPInfo::Float { val } => Value::Float(val),
            CPInfo::Long { val } => Value::Long(val),
            CPInfo::Double { val } => Value::Double(val),
            CPInfo::String { string_index } => {
                drop(ma);
                let str = method_area().classes[class_id].constant_pool.utf8(string_index);
                Value::Object(Some(self.create_string(&str)))
            }
            _ => unimplemented!(),
        };
        self.operand_stack.push(val);
    }

    pub fn run(&mut self) {
        loop {
            let opcode = self.read_ins();
            dbg!(opcode);
            match opcode {
                // nop
                0 => {}
                // aconst_null
                1 => self.operand_stack.push(Value::Object(None)),
                // ldc
                18 => {
                    let cp_idx = self.read_ins() as u16;
                    self.ldc(cp_idx)
                }
                // ldc_w
                19 => {
                    let cp_idx = self.read_u16();
                    self.ldc(cp_idx)
                }
                // ldc2_w
                20 => {
                    let cp_idx = self.read_u16();
                    self.ldc(cp_idx)
                }
                // return
                177 => return,
                // getstatic
                178 => {
                    let idx = self.read_u16();
                    let class_id = self.class_id();
                    let field = Class::field_reference(class_id, idx);
                    let defining_class = method_area().fields[field].defining_class;
                    self.ensure_initialized(defining_class);
                    self.operand_stack
                        .push(method_area().fields[field].static_value.unwrap());
                }
                // putstatic
                179 => {
                    let idx = self.read_u16();
                    let class_id = self.class_id();
                    let field = Class::field_reference(class_id, idx);
                    let defining_class = method_area().fields[field].defining_class;
                    self.ensure_initialized(defining_class);
                    method_area().fields[field].static_value = self.operand_stack.pop();
                }
                // invokespecial
                183 => {
                    let idx = self.read_u16();
                    let class_id = self.class_id();
                    let method = Class::method_reference(class_id, idx);
                    self.call_method(method, false);
                }
                // invokestatic
                184 => {
                    let idx = self.read_u16();
                    let class_id = self.class_id();
                    let method = Class::method_reference(class_id, idx);
                    self.call_method(method, true);
                }
                // new
                187 => {
                    let idx = self.read_u16();
                    let class_id = self.class_id();
                    let obj_class = Class::class_reference(class_id, idx);
                    self.ensure_initialized(obj_class);
                    let obj_id = Object::new(obj_class);
                    self.operand_stack.push(Value::Object(Some(obj_id)))
                }
                // newarray
                188 => {
                    let atype = self.read_ins();
                    let count = match self.operand_stack.pop().unwrap() {
                        Value::Int(val) => val,
                        _ => panic!("array count must be int"),
                    };
                    assert!(count >= 0, "NegativeArraySizeException");

                    let val = Value::default_for_ty(&match atype {
                        4 => FieldType::BaseType(BaseType::Z),
                        5 => FieldType::BaseType(BaseType::C),
                        6 => FieldType::BaseType(BaseType::F),
                        7 => FieldType::BaseType(BaseType::D),
                        8 => FieldType::BaseType(BaseType::B),
                        9 => FieldType::BaseType(BaseType::S),
                        10 => FieldType::BaseType(BaseType::I),
                        11 => FieldType::BaseType(BaseType::J),
                        _ => panic!(),
                    });
                    let arr: Box<[Value]> = (0..count).map(|_| val).collect();
                    let id = heap().arrays.alloc(Array { contents: arr });
                    self.operand_stack.push(Value::Array(Some(id)));
                }
                _ => unimplemented!("opcode: {}", opcode),
            }
        }
    }
}
