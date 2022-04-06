use super::Thread;
use crate::class::Class;
use crate::class_file::constant_pool::CPInfo;
use crate::class_file::descriptors::{BaseType, FieldType, ObjectType};
use crate::class_loader::method_area;
use crate::heap::{heap, Array, Object};
use crate::value::Value;
use std::cmp::Ordering;

macro_rules! binary_op {
    ($self:ident, $op:tt) => {{
        let rhs = $self.pop();
        let lhs = $self.pop();
        $self.operand_stack.push(lhs $op rhs);
    }};
    ($self:ident, ($lhs:ident, $rhs:ident) => $expr:expr) => {{
        let $rhs = $self.pop();
        let $lhs = $self.pop();
        $self.operand_stack.push($expr);
    }};
}

macro_rules! unary_op {
    ($self:ident, $op:tt) => {{
        let input = $self.pop();
        $self.operand_stack.push($op input);
    }};
}

macro_rules! cast {
    ($self:ident, $from:ident, $to:ident, $val:ident -> $expr:expr) => {{
        let val = $self.pop();
        let new = match val {
            Value::$from($val) => Value::$to($expr),
            _ => unreachable!(),
        };
        $self.operand_stack.push(new);
    }};
}

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
        loop {
            let cur_pc = self.pc;
            let opcode = self.read_ins();
            let c = self.class_id();
            let class_name = method_area().classes[c].name.clone();
            let method_name = method_area().methods[self.method].name.clone();
            println!(
                "m: {}.{}, pc: {}, opcode: {}",
                class_name, method_name, cur_pc, opcode
            );
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
                // lconst_<l>
                9 => self.operand_stack.push(Value::Long(0)),
                10 => self.operand_stack.push(Value::Long(1)),
                // fconst_<f>
                11 => self.operand_stack.push(Value::Float(0.0)),
                12 => self.operand_stack.push(Value::Float(1.0)),
                13 => self.operand_stack.push(Value::Float(2.0)),
                // dconst_<d>
                14 => self.operand_stack.push(Value::Double(0.0)),
                15 => self.operand_stack.push(Value::Double(1.0)),
                // bipush
                16 => {
                    let b = self.read_ins() as i8;
                    self.operand_stack.push(Value::Int(b as i32))
                }
                // sipush
                17 => {
                    let s = self.read_u16() as i16;
                    self.operand_stack.push(Value::Int(s as i32))
                }
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
                // iload, lload, fload, dload, aload
                21..=25 => {
                    let idx = self.read_ins() as usize;
                    self.operand_stack.push(self.locals[idx].unwrap());
                }
                // iload_<n>
                26 => self.operand_stack.push(self.locals[0].unwrap()),
                27 => self.operand_stack.push(self.locals[1].unwrap()),
                28 => self.operand_stack.push(self.locals[2].unwrap()),
                29 => self.operand_stack.push(self.locals[3].unwrap()),
                // lload_<n>
                30 => self.operand_stack.push(self.locals[0].unwrap()),
                31 => self.operand_stack.push(self.locals[1].unwrap()),
                32 => self.operand_stack.push(self.locals[2].unwrap()),
                33 => self.operand_stack.push(self.locals[3].unwrap()),
                // fload_<n>
                34 => self.operand_stack.push(self.locals[0].unwrap()),
                35 => self.operand_stack.push(self.locals[1].unwrap()),
                36 => self.operand_stack.push(self.locals[2].unwrap()),
                37 => self.operand_stack.push(self.locals[3].unwrap()),
                // dload_<n>
                38 => self.operand_stack.push(self.locals[0].unwrap()),
                39 => self.operand_stack.push(self.locals[1].unwrap()),
                40 => self.operand_stack.push(self.locals[2].unwrap()),
                41 => self.operand_stack.push(self.locals[3].unwrap()),
                // aload_<n>
                42 => self.operand_stack.push(self.locals[0].unwrap()),
                43 => self.operand_stack.push(self.locals[1].unwrap()),
                44 => self.operand_stack.push(self.locals[2].unwrap()),
                45 => self.operand_stack.push(self.locals[3].unwrap()),
                // iaload, laload, faload, daload, aaload
                46..=50 => {
                    let val = self.arr_load();
                    self.operand_stack.push(val);
                }
                // baload, caload, saload
                51..=53 => {
                    let val = self.arr_load().extend_32();
                    self.operand_stack.push(val);
                }
                // istore, lstore, fstore, dstore, astore
                54..=58 => {
                    let idx = self.read_ins() as usize;
                    self.locals[idx] = Some(self.operand_stack.pop().unwrap());
                }
                // istore_<n>
                59 => self.locals[0] = Some(self.operand_stack.pop().unwrap()),
                60 => self.locals[1] = Some(self.operand_stack.pop().unwrap()),
                61 => self.locals[2] = Some(self.operand_stack.pop().unwrap()),
                62 => self.locals[3] = Some(self.operand_stack.pop().unwrap()),
                // lstore_<n>
                63 => self.locals[0] = Some(self.operand_stack.pop().unwrap()),
                64 => self.locals[1] = Some(self.operand_stack.pop().unwrap()),
                65 => self.locals[2] = Some(self.operand_stack.pop().unwrap()),
                66 => self.locals[3] = Some(self.operand_stack.pop().unwrap()),
                // fstore_<n>
                67 => self.locals[0] = Some(self.operand_stack.pop().unwrap()),
                68 => self.locals[1] = Some(self.operand_stack.pop().unwrap()),
                69 => self.locals[2] = Some(self.operand_stack.pop().unwrap()),
                70 => self.locals[3] = Some(self.operand_stack.pop().unwrap()),
                // dstore_<n>
                71 => self.locals[0] = Some(self.operand_stack.pop().unwrap()),
                72 => self.locals[1] = Some(self.operand_stack.pop().unwrap()),
                73 => self.locals[2] = Some(self.operand_stack.pop().unwrap()),
                74 => self.locals[3] = Some(self.operand_stack.pop().unwrap()),
                // astore_<n>
                75 => self.locals[0] = Some(self.operand_stack.pop().unwrap()),
                76 => self.locals[1] = Some(self.operand_stack.pop().unwrap()),
                77 => self.locals[2] = Some(self.operand_stack.pop().unwrap()),
                78 => self.locals[3] = Some(self.operand_stack.pop().unwrap()),
                // iastore, lastore, fastore, dastore, aastore
                79..=83 => {
                    let val = self.pop();
                    let idx = match self.pop() {
                        Value::Int(idx) => idx,
                        _ => unreachable!(),
                    };
                    let arr = match self.pop() {
                        Value::Array(Some(arr)) => arr,
                        Value::Array(None) => panic!("NullPointerException"),
                        _ => unreachable!(),
                    };
                    heap().arrays[arr].contents[idx as usize] = val;
                }
                // bastore, castore, sastore
                84..=86 => {
                    let val = self.pop();
                    let idx = match self.pop() {
                        Value::Int(idx) => idx,
                        _ => unreachable!(),
                    };
                    let arr = match self.pop() {
                        Value::Array(Some(arr)) => arr,
                        Value::Array(None) => panic!("NullPointerException"),
                        _ => unreachable!(),
                    };
                    let arr = &mut heap().arrays[arr];
                    arr.contents[idx as usize] = val.store_ty(&arr.ty);
                }
                // pop
                87 => {
                    let _ = self.operand_stack.pop();
                }
                // dup
                89 => {
                    let val = *self.operand_stack.last().unwrap();
                    self.operand_stack.push(val);
                }
                // dup_x1
                90 => {
                    let val = *self.operand_stack.last().unwrap();
                    self.operand_stack.insert(self.operand_stack.len() - 2, val);
                }
                // iadd
                96 => binary_op!(self, +),
                // ladd
                97 => binary_op!(self, +),
                // fadd
                98 => binary_op!(self, +),
                // isub
                100 => binary_op!(self, -),
                // imul
                104 => binary_op!(self, *),
                // lmul
                105 => binary_op!(self, *),
                // fdiv
                110 => binary_op!(self, /),
                // irem
                112 => binary_op!(self, %),
                // ineg
                116 => unary_op!(self, -),
                // ishl
                120 => binary_op!(self, <<),
                // lshl
                121 => binary_op!(self, <<),
                // ishr
                122 => binary_op!(self, >>),
                // lshr
                123 => binary_op!(self, >>),
                // iushr
                124 => binary_op!(self, (lhs, rhs) => lhs.ushr(rhs)),
                // lushr
                125 => binary_op!(self, (lhs, rhs) => lhs.ushr(rhs)),
                // iand
                126 => binary_op!(self, &),
                // ior
                128 => binary_op!(self, |),
                // land
                127 => binary_op!(self, &),
                // ixor
                130 => binary_op!(self, ^),
                // iinc
                132 => {
                    let idx = self.read_ins() as usize;
                    let c = self.read_ins() as i8 as i32;
                    match &mut self.locals[idx] {
                        Some(Value::Int(val)) => *val += c,
                        _ => unreachable!(),
                    }
                }
                // i2f
                134 => cast!(self, Int, Float, val -> val as f32),
                // i2l
                133 => cast!(self, Int, Long, val -> val as i64),
                // l2i
                136 => cast!(self, Long, Int, val -> val as i32),
                // f2i
                139 => cast!(self, Float, Int, val -> val as i32),
                // i2c
                146 => cast!(self, Int, Int, val -> val % 0xF),
                // lcmp
                148 => {
                    let value2 = match self.pop() {
                        Value::Long(val) => val,
                        _ => unreachable!(),
                    };
                    let value1 = match self.pop() {
                        Value::Long(val) => val,
                        _ => unreachable!(),
                    };
                    let res = match value1.cmp(&value2) {
                        Ordering::Greater => 1,
                        Ordering::Equal => 0,
                        Ordering::Less => -1,
                    };
                    self.operand_stack.push(Value::Int(res));
                }
                // fcmpg, fcmpl
                149..=150 => {
                    let value2 = match self.pop() {
                        Value::Float(val) => val,
                        _ => unreachable!(),
                    };
                    let value1 = match self.pop() {
                        Value::Float(val) => val,
                        _ => unreachable!(),
                    };
                    let res = match value1.partial_cmp(&value2) {
                        Some(Ordering::Greater) => 1,
                        Some(Ordering::Equal) => 0,
                        Some(Ordering::Less) => -1,
                        _ => match opcode {
                            149 => -1,
                            150 => 1,
                            _ => unreachable!(),
                        }
                    };
                    self.operand_stack.push(Value::Int(res));
                }
                // if<cond>
                153..=158 => {
                    let val = match self.pop() {
                        Value::Int(val) => val,
                        _ => unreachable!(),
                    };
                    self.br_if(
                        cur_pc,
                        match opcode {
                            153 => val == 0,
                            154 => val != 0,
                            155 => val < 0,
                            156 => val >= 0,
                            157 => val > 0,
                            158 => val <= 0,
                            _ => unreachable!(),
                        },
                    );
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
                    self.br_if(
                        cur_pc,
                        match opcode {
                            159 => lhs == rhs,
                            160 => lhs != rhs,
                            161 => lhs < rhs,
                            162 => lhs >= rhs,
                            163 => lhs > rhs,
                            164 => lhs <= rhs,
                            _ => unreachable!(),
                        },
                    );
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
                    self.br_if(
                        cur_pc,
                        match opcode {
                            165 => lhs == rhs,
                            166 => lhs != rhs,
                            _ => unreachable!(),
                        },
                    );
                }
                // goto
                167 => self.br_if(cur_pc, true),
                // ireturn
                172 => return self.operand_stack.pop(),
                // lreturn
                173 => return self.operand_stack.pop(),
                // freturn
                174 => return self.operand_stack.pop(),
                // dreturn
                175 => return self.operand_stack.pop(),
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
                    self.operand_stack.push(
                        method_area().fields[field]
                            .static_value
                            .unwrap()
                            .extend_32(),
                    );
                }
                // putstatic
                179 => {
                    let idx = self.read_u16();
                    let class_id = self.class_id();
                    let field = Class::field_reference(class_id, idx);
                    let defining_class = method_area().fields[field].defining_class;
                    self.ensure_initialized(defining_class);
                    method_area().fields[field].store_static(self.operand_stack.pop().unwrap());
                }
                // getfield
                180 => {
                    let idx = self.read_u16();
                    let class_id = self.class_id();
                    let field = Class::field_reference(class_id, idx);
                    let obj = match self.pop() {
                        Value::Object(Some(obj)) => obj,
                        Value::Object(None) => panic!("NullPointerException"),
                        _ => unreachable!(),
                    };
                    self.operand_stack
                        .push(heap().objects[obj].fields[&field].extend_32());
                }
                // putfield
                181 => {
                    let idx = self.read_u16();
                    let class_id = self.class_id();
                    let field = Class::field_reference(class_id, idx);
                    let val = self.pop();
                    let obj = match self.pop() {
                        Value::Object(Some(obj)) => obj,
                        Value::Object(None) => panic!("NullPointerException"),
                        a => unreachable!("{a:?}"),
                    };
                    heap().objects[obj].store_field(field, val);
                }
                // invokevirtual
                182 => {
                    let idx = self.read_u16();
                    let class_id = self.class_id();
                    let method = Class::method_reference(class_id, idx);
                    let method = self.select_method(method);
                    self.call_method(method);
                }
                // invokespecial
                183 => {
                    let idx = self.read_u16();
                    let class_id = self.class_id();
                    let method = Class::method_reference(class_id, idx);
                    self.call_method(method);
                }
                // invokestatic
                184 => {
                    let idx = self.read_u16();
                    let class_id = self.class_id();
                    let method = Class::method_reference(class_id, idx);
                    let defining_class = method_area().methods[method].defining_class;
                    self.ensure_initialized(defining_class);
                    self.call_method(method);
                }
                // invokeinterface
                185 => {
                    let idx = self.read_u16();
                    let class_id = self.class_id();
                    let method = Class::method_reference(class_id, idx);
                    let method = self.select_method(method);
                    let defining_class = method_area().methods[method].defining_class;
                    self.ensure_initialized(defining_class);
                    self.call_method(method);

                    // here for historical reasons
                    let _count = self.read_ins();
                    let _z = self.read_ins();
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
                    let ty = match atype {
                        4 => FieldType::BaseType(BaseType::Z),
                        5 => FieldType::BaseType(BaseType::C),
                        6 => FieldType::BaseType(BaseType::F),
                        7 => FieldType::BaseType(BaseType::D),
                        8 => FieldType::BaseType(BaseType::B),
                        9 => FieldType::BaseType(BaseType::S),
                        10 => FieldType::BaseType(BaseType::I),
                        11 => FieldType::BaseType(BaseType::J),
                        _ => panic!(),
                    };
                    let val = Value::default_for_ty(&ty);
                    let arr: Box<[Value]> = (0..count).map(|_| val).collect();
                    let id = heap().arrays.alloc(Array { contents: arr, ty });
                    self.operand_stack.push(Value::Array(Some(id)));
                }
                // anewarray
                189 => {
                    let idx = self.read_u16();
                    let class_id = self.class_id();
                    let item_class = Class::class_reference(class_id, idx);
                    let class_name = method_area().classes[item_class].name.clone();
                    let count = match self.operand_stack.pop().unwrap() {
                        Value::Int(val) => val,
                        _ => panic!("array count must be int"),
                    };
                    assert!(count >= 0, "NegativeArraySizeException");

                    let arr: Box<[Value]> =
                        (0..count).map(|_| Value::Object(Option::None)).collect();
                    let id = heap().arrays.alloc(Array {
                        contents: arr,
                        ty: FieldType::ObjectType(ObjectType { class_name }),
                    });
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
                // checkcast
                192 => {
                    let val = self.pop();
                    let obj = match val {
                        Value::Object(Some(obj)) => obj,
                        Value::Object(None) => {
                            self.operand_stack.push(val);
                            continue;
                        }
                        a => unreachable!("{a:?}"),
                    };
                    let obj_class = heap().objects[obj].class;

                    let cp_idx = self.read_u16();
                    let class_id = self.class_id();
                    let ref_class = Class::class_reference(class_id, cp_idx);

                    let instance_of = Class::instance_of(obj_class, ref_class);
                    if !instance_of {
                        panic!("ClassCastException");
                    }
                    self.operand_stack.push(val)
                }
                // instanceof
                193 => {
                    let obj = match self.pop() {
                        Value::Object(Some(obj)) => obj,
                        Value::Object(None) => {
                            self.operand_stack.push(Value::Int(0));
                            continue;
                        }
                        a => unreachable!("{a:?}"),
                    };
                    let obj_class = heap().objects[obj].class;

                    let cp_idx = self.read_u16();
                    let class_id = self.class_id();
                    let ref_class = Class::class_reference(class_id, cp_idx);

                    let instance_of = Class::instance_of(obj_class, ref_class);
                    self.operand_stack.push(Value::Int(instance_of as i32))
                }
                // ifnull, ifnonnull
                198..=199 => {
                    let is_some = match self.pop() {
                        Value::Object(val) => val.is_some(),
                        Value::Array(val) => val.is_some(),
                        a => unreachable!("{:?}", a),
                    };
                    self.br_if(
                        cur_pc,
                        match opcode {
                            198 => !is_some,
                            199 => is_some,
                            _ => unreachable!(),
                        },
                    );
                }
                _ => unimplemented!("opcode: {}", opcode),
            }
        }
    }
}
