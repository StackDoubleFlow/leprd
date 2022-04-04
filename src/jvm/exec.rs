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
                let str = method_area().classes[class_id]
                    .constant_pool
                    .utf8(string_index);
                Value::Object(Some(self.create_string(&str)))
            }
            CPInfo::Class { .. } => {
                drop(ma);
                let c = Class::class_reference(class_id, cp_idx);
                Value::Object(Some(Class::obj(c)))
            }
            _ => unimplemented!(),
        };
        self.operand_stack.push(val);
    }

    pub fn run(&mut self) -> Option<Value> {
        let cur_pc = self.pc;
        loop {
            let opcode = self.read_ins();
            dbg!(opcode);
            match opcode {
                // nop
                0 => {}
                // aconst_null
                1 => self.operand_stack.push(Value::Object(None)),
                // iconst_<i>
                2 => self.operand_stack.push(Value::Int(-1)),
                3 => self.operand_stack.push(Value::Int(0)),
                4 => self.operand_stack.push(Value::Int(1)),
                5 => self.operand_stack.push(Value::Int(2)),
                6 => self.operand_stack.push(Value::Int(3)),
                7 => self.operand_stack.push(Value::Int(4)),
                8 => self.operand_stack.push(Value::Int(5)),
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
                // float_<n>
                34 => self.operand_stack.push(self.locals[0].unwrap()),
                35 => self.operand_stack.push(self.locals[1].unwrap()),
                36 => self.operand_stack.push(self.locals[2].unwrap()),
                37 => self.operand_stack.push(self.locals[3].unwrap()),
                // iload_<n>
                26 => self.operand_stack.push(self.locals[0].unwrap()),
                27 => self.operand_stack.push(self.locals[1].unwrap()),
                28 => self.operand_stack.push(self.locals[2].unwrap()),
                29 => self.operand_stack.push(self.locals[3].unwrap()),
                // aload_<n>
                42 => self.operand_stack.push(self.locals[0].unwrap()),
                43 => self.operand_stack.push(self.locals[1].unwrap()),
                44 => self.operand_stack.push(self.locals[2].unwrap()),
                45 => self.operand_stack.push(self.locals[3].unwrap()),
                // astore_<n>
                75 => self.locals.insert(0, Some(self.operand_stack.pop().unwrap())),
                76 => self.locals.insert(1, Some(self.operand_stack.pop().unwrap())),
                77 => self.locals.insert(2, Some(self.operand_stack.pop().unwrap())),
                78 => self.locals.insert(3, Some(self.operand_stack.pop().unwrap())),
                // dup
                89 => {
                    let val = *self.operand_stack.last().unwrap();
                    self.operand_stack.push(val);
                }
                // if<cond>
                153..=158 => {
                    let val = match self.pop() {
                        Value::Int(val) => val,
                        _ => unreachable!(),
                    };
                    self.br_if(cur_pc, match opcode {
                        153 => val == 0,
                        154 => val != 0,
                        155 => val < 0,
                        156 => val >= 0,
                        157 => val > 0,
                        158 => val <= 0,
                        _ => unreachable!(),
                    });
                }
                // if_icmp<cond>
                159..=164 => {
                    let rhs = match self.pop() {
                        Value::Int(val) => val,
                        _ => unreachable!(),
                    };
                    let lhs = match self.pop() {
                        Value::Int(val) => val,
                        _ => unreachable!(),
                    };
                    self.br_if(cur_pc, match opcode {
                        159 => lhs == rhs,
                        160 => lhs != rhs,
                        161 => lhs < rhs,
                        162 => lhs >= rhs,
                        163 => lhs > rhs,
                        164 => lhs <= rhs,
                        _ => unreachable!(),
                    });
                }
                // if_acmp<cond>
                165..=166 => {
                    let rhs = match self.pop() {
                        Value::Object(val) => val,
                        _ => unreachable!(),
                    };
                    let lhs = match self.pop() {
                        Value::Object(val) => val,
                        _ => unreachable!(),
                    };
                    self.br_if(cur_pc, match opcode {
                        165 => lhs == rhs,
                        166 => lhs != rhs,
                        _ => unreachable!()
                    });
                }
                // areturn
                176 => return self.operand_stack.pop(),
                // return
                177 => return None,
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
                // getfield
                180 => {
                    let idx = self.read_u16();
                    let class_id = self.class_id();
                    let field = Class::field_reference(class_id, idx);
                    let obj = match self.pop() {
                        Value::Object(Some(obj)) => obj,
                        Value::Object(None) => panic!("NullPointerException"),
                        _ => unreachable!()
                    };
                    self.operand_stack.push(heap().objects[obj].fields[&field]);
                }
                // invokevirtual
                182 => {
                    let idx = self.read_u16();
                    let class_id = self.class_id();
                    let method = Class::method_reference(class_id, idx);
                    self.call_method(method, false);
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
                    let defining_class = method_area().methods[method].defining_class;
                    self.ensure_initialized(defining_class);
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
                // arraylength
                190 => {
                    let arr = match self.operand_stack.pop() {
                        Some(Value::Array(Some(arr))) => arr,
                        Some(Value::Array(None)) => panic!("NullPointerException"),
                        _ => panic!(),
                    };
                    let len = heap().arrays[arr].contents.len() as i32;
                    self.operand_stack.push(Value::Int(len));
                }
                // ifnull, ifnonnull
                198..=199 => {
                    let val = match self.pop() {
                        Value::Object(val) => val,
                        _ => unreachable!(),
                    };
                    self.br_if(cur_pc, match opcode {
                        198 => val.is_none(),
                        199 => val.is_some(),
                        _ => unreachable!(),
                    });
                }
                _ => unimplemented!("opcode: {}", opcode),
            }
        }
    }
}
