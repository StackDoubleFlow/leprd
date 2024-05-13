use super::Thread;
use crate::class::Class;
use crate::class_file::constant_pool::CPInfo;
use crate::class_file::descriptors::{BaseType, FieldType, ObjectType};
use crate::class_loader::method_area;
use crate::heap::heap;
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
        'inst: loop {
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
                    self.locals[idx] = Some(self.pop());
                }
                // istore_<n>
                59 => self.locals[0] = Some(self.pop()),
                60 => self.locals[1] = Some(self.pop()),
                61 => self.locals[2] = Some(self.pop()),
                62 => self.locals[3] = Some(self.pop()),
                // lstore_<n>
                63 => self.locals[0] = Some(self.pop()),
                64 => self.locals[1] = Some(self.pop()),
                65 => self.locals[2] = Some(self.pop()),
                66 => self.locals[3] = Some(self.pop()),
                // fstore_<n>
                67 => self.locals[0] = Some(self.pop()),
                68 => self.locals[1] = Some(self.pop()),
                69 => self.locals[2] = Some(self.pop()),
                70 => self.locals[3] = Some(self.pop()),
                // dstore_<n>
                71 => self.locals[0] = Some(self.pop()),
                72 => self.locals[1] = Some(self.pop()),
                73 => self.locals[2] = Some(self.pop()),
                74 => self.locals[3] = Some(self.pop()),
                // astore_<n>
                75 => self.locals[0] = Some(self.pop()),
                76 => self.locals[1] = Some(self.pop()),
                77 => self.locals[2] = Some(self.pop()),
                78 => self.locals[3] = Some(self.pop()),
                // iastore, lastore, fastore, dastore, aastore
                79..=83 => {
                    let val = self.pop();
                    let idx = self.pop().int();
                    let Some(arr) = self.pop().array() else {
                        panic!("NullPointerException");
                    };
                    heap().store_arr_elem(arr, idx as usize, val);
                }
                // bastore, castore, sastore
                84..=86 => {
                    let val = self.pop();
                    let idx = self.pop().int();
                    let Some(arr) = self.pop().array() else {
                        panic!("NullPointerException");
                    };
                    let heap = heap();
                    let store_val = val.store_ty(heap.arr_ty(arr));
                    heap.store_arr_elem(arr, idx as usize, store_val);
                }
                // pop
                87 => {
                    let _ = self.pop();
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
                // dup_x2
                91 => {
                    let val = *self.operand_stack.last().unwrap();
                    if (self.operand_stack[self.operand_stack.len() - 2]).is_cat_2() {
                        self.operand_stack.insert(self.operand_stack.len() - 2, val);
                    } else {
                        self.operand_stack.insert(self.operand_stack.len() - 3, val);
                    }
                }
                // dup2
                92 => {
                    let val1 = *self.operand_stack.last().unwrap();
                    if val1.is_cat_2() {
                        self.operand_stack.push(val1);
                    } else {
                        self.operand_stack
                            .push(self.operand_stack[self.operand_stack.len() - 2]);
                        self.operand_stack.push(val1);
                    }
                }
                // dup2_x1
                93 => {
                    let val1 = *self.operand_stack.last().unwrap();
                    let val2 = self.operand_stack[self.operand_stack.len() - 2];
                    assert!(!val2.is_cat_2());
                    if val1.is_cat_2() {
                        self.operand_stack
                            .insert(self.operand_stack.len() - 2, val1);
                    } else {
                        let dest = self.operand_stack.len() - 3;
                        self.operand_stack.splice(dest..dest, [val2, val1]);
                    }
                }
                // dup2_x2
                94 => {
                    let val1 = *self.operand_stack.last().unwrap();
                    let val2 = self.operand_stack[self.operand_stack.len() - 2];
                    let val3 = self.operand_stack[self.operand_stack.len() - 3];
                    let dest = if val1.is_cat_2() && val2.is_cat_2() {
                        // take one, insert 2 values down
                        self.operand_stack.len() - 2
                    } else if val1.is_cat_2() && !val2.is_cat_2() && !val3.is_cat_2() {
                        // take one, insert 4 values down
                        self.operand_stack.len() - 4
                    } else if !val1.is_cat_2() && !val2.is_cat_2() && val3.is_cat_2() {
                        // take two, insert 3 values down
                        self.operand_stack.len() - 3
                    } else if !val1.is_cat_2() && !val2.is_cat_2() && !val3.is_cat_2() {
                        // take two, insert 4 values down
                        self.operand_stack.len() - 3
                    } else {
                        panic!("invalid dup2_x2");
                    };
                    if val1.is_cat_2() {
                        // take one
                        self.operand_stack.insert(dest, val1);
                    } else {
                        // take two
                        self.operand_stack.splice(dest..dest, [val2, val1]);
                    }
                }
                // swap
                95 => {
                    let val = self.pop();
                    self.operand_stack.insert(self.operand_stack.len() - 1, val);
                }
                // iadd
                96 => binary_op!(self, +),
                // ladd
                97 => binary_op!(self, +),
                // fadd
                98 => binary_op!(self, +),
                // dadd
                99 => binary_op!(self, +),
                // isub
                100 => binary_op!(self, -),
                // lsub
                101 => binary_op!(self, -),
                // fsub
                102 => binary_op!(self, -),
                // dsub
                103 => binary_op!(self, -),
                // imul
                104 => binary_op!(self, *),
                // lmul
                105 => binary_op!(self, *),
                // fmul
                106 => binary_op!(self, *),
                // dmul
                107 => binary_op!(self, *),
                // idiv
                108 => binary_op!(self, /),
                // ldiv
                109 => binary_op!(self, /),
                // fdiv
                110 => binary_op!(self, /),
                // ddiv
                111 => binary_op!(self, /),
                // irem
                112 => binary_op!(self, %),
                // lrem
                113 => binary_op!(self, %),
                // frem
                114 => binary_op!(self, %),
                // drem
                115 => binary_op!(self, %),
                // ineg
                116 => unary_op!(self, -),
                // lneg
                117 => unary_op!(self, -),
                // fneg
                118 => unary_op!(self, -),
                // dneg
                119 => unary_op!(self, -),
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
                // land
                127 => binary_op!(self, &),
                // ior
                128 => binary_op!(self, |),
                // lor
                129 => binary_op!(self, |),
                // ixor
                130 => binary_op!(self, ^),
                // lxor
                131 => binary_op!(self, ^),
                // iinc
                132 => {
                    let idx = self.read_ins() as usize;
                    let c = self.read_ins() as i8 as i32;
                    match &mut self.locals[idx] {
                        Some(Value::Int(val)) => *val += c,
                        _ => unreachable!(),
                    }
                }
                // i2l
                133 => cast!(self, Int, Long, val -> val as i64),
                // i2f
                134 => cast!(self, Int, Float, val -> val as f32),
                // i2d
                135 => cast!(self, Int, Double, val -> val as f64),
                // l2i
                136 => cast!(self, Long, Int, val -> val as i32),
                // l2f
                137 => cast!(self, Long, Float, val -> val as f32),
                // l2d
                138 => cast!(self, Long, Double, val -> val as f64),
                // f2i
                139 => cast!(self, Float, Int, val -> val as i32),
                // f2l
                140 => cast!(self, Float, Long, val -> val as i64),
                // f2d
                141 => cast!(self, Float, Double, val -> val as f64),
                // d2i
                142 => cast!(self, Double, Int, val -> val as i32),
                // d2l
                143 => cast!(self, Double, Long, val -> val as i64),
                // d2f
                144 => cast!(self, Double, Float, val -> val as f32),
                // i2b
                145 => cast!(self, Int, Int, val -> (val as i8) as i32),
                // i2c
                146 => cast!(self, Int, Int, val -> val & 0xFF),
                // i2s
                147 => cast!(self, Int, Int, val -> val & 0xFFFF),
                // lcmp
                148 => {
                    let value2 = self.pop().long();
                    let value1 = self.pop().long();
                    let res = match value1.cmp(&value2) {
                        Ordering::Greater => 1,
                        Ordering::Equal => 0,
                        Ordering::Less => -1,
                    };
                    self.operand_stack.push(Value::Int(res));
                }
                // fcmpg, fcmpl
                149..=150 => {
                    let value2 = self.pop().float();
                    let value1 = self.pop().float();
                    let res = match value1.partial_cmp(&value2) {
                        Some(Ordering::Greater) => 1,
                        Some(Ordering::Equal) => 0,
                        Some(Ordering::Less) => -1,
                        _ => match opcode {
                            149 => -1,
                            150 => 1,
                            _ => unreachable!(),
                        },
                    };
                    self.operand_stack.push(Value::Int(res));
                }
                // if<cond>
                153..=158 => {
                    let val = self.pop().int();
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
                    let rhs = self.pop().int();
                    let lhs = self.pop().int();
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
                    let rhs = self.pop().object();
                    let lhs = self.pop().object();
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
                // tableswitch
                170 => {
                    let base_pc = self.pc - 1;
                    let idx = self.pop().int();
                    self.pad_to_int();
                    let default = self.read_i32();
                    let low = self.read_i32();
                    let high = self.read_i32();

                    if idx < low || idx > high {
                        self.pc = base_pc.saturating_add_signed(default as isize);
                        continue;
                    }

                    self.pc += 4 * (idx - low) as usize;
                    let offset = self.read_i32();
                    self.pc = base_pc.saturating_add_signed(offset as isize);
                }
                // lookupswitch
                171 => {
                    let base_pc = self.pc - 1;
                    let key = self.pop().int();
                    self.pad_to_int();
                    let default = self.read_i32();
                    let num_pairs = self.read_i32();
                    for _ in 0..num_pairs {
                        let m = self.read_i32();
                        let offset = self.read_i32();
                        if key == m {
                            self.pc = base_pc.saturating_add_signed(offset as isize);
                            continue 'inst;
                        }
                    }
                    self.pc = base_pc.saturating_add_signed(default as isize);
                }
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
                    self.operand_stack
                        .push(method_area().fields[field].load_static());
                }
                // putstatic
                179 => {
                    let idx = self.read_u16();
                    let class_id = self.class_id();
                    let field = Class::field_reference(class_id, idx);
                    let defining_class = method_area().fields[field].defining_class;
                    self.ensure_initialized(defining_class);
                    method_area().fields[field].store_static(self.pop());
                }
                // getfield
                180 => {
                    let idx = self.read_u16();
                    let class_id = self.class_id();
                    let field = Class::field_reference(class_id, idx);
                    let Some(obj) = self.pop().object() else {
                        panic!("NullPointerException");
                    };
                    self.operand_stack
                        .push(heap().load_field(obj, field).extend_32());
                }
                // putfield
                181 => {
                    let idx = self.read_u16();
                    let class_id = self.class_id();
                    let field = Class::field_reference(class_id, idx);
                    let val = self.pop();
                    let Some(obj) = self.pop().object() else {
                        panic!("NullPointerException");
                    };
                    let ma = method_area();
                    let ty = &ma.fields[field].descriptor.0;
                    let store_val = val.store_ty(&ty);
                    drop(ma);
                    heap().store_field(obj, field, store_val);
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
                    let obj_ref = heap().new_object(obj_class);
                    self.operand_stack.push(Value::Object(Some(obj_ref)))
                }
                // newarray
                188 => {
                    let atype = self.read_ins();
                    let count = self.pop().int();
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
                    let arr = heap().new_array(ty, count as usize);
                    self.operand_stack.push(Value::Array(Some(arr)));
                }
                // anewarray
                189 => {
                    let idx = self.read_u16();
                    let class_id = self.class_id();
                    let item_class = Class::class_reference(class_id, idx);
                    let class_name = method_area().classes[item_class].name.clone();
                    let count = self.pop().int();
                    assert!(count >= 0, "NegativeArraySizeException");

                    let ty = FieldType::ObjectType(ObjectType { class_name });
                    let arr = heap().new_array(ty, count as usize);
                    self.operand_stack.push(Value::Array(Some(arr)));
                }
                // arraylength
                190 => {
                    let arr = match self.pop().array() {
                        Some(arr) => arr,
                        None => panic!("NullPointerException"),
                    };
                    let len = heap().arr_len(arr) as i32;
                    self.operand_stack.push(Value::Int(len));
                }
                // athrow
                191 => {
                    let Some(throwable_obj) = self.pop().object() else {
                        panic!("NullPointerException");
                    };
                    let class_id = heap().get_obj_class(throwable_obj);
                    println!(
                        "An exception was thrown: {}",
                        method_area().classes[class_id].name
                    );

                    let mut ma = method_area();
                    let throwable_class = ma.resolve_class("java/lang/Throwable");
                    let details_field = ma.resolve_field(throwable_class, "detailMessage");
                    drop(ma);
                    let details = heap().load_field(throwable_obj, details_field).object();
                    if let Some(details) = details {
                        let details = heap().read_string(details);
                        println!("The following details were given: {}", details);
                    }

                    panic!("todo: exceptions");
                }
                // checkcast
                192 => {
                    let val = self.pop();
                    let cp_idx = self.read_u16();
                    if matches!(val, Value::Object(None) | Value::Array(None)) {
                        self.operand_stack.push(val);
                        continue;
                    }

                    let class_id = self.class_id();
                    let ref_class = Class::class_reference(class_id, cp_idx);

                    let obj_class = match val {
                        Value::Object(Some(obj)) => heap().get_obj_class(obj),
                        Value::Array(Some(arr)) => {
                            let heap = heap();
                            let elem_ty = heap.arr_ty(arr);
                            method_area().resolve_arr_class(elem_ty)
                        }
                        a => unreachable!("{a:?}"),
                    };

                    let instance_of = Class::instance_of(obj_class, ref_class);
                    if !instance_of {
                        panic!("ClassCastException");
                    }
                    self.operand_stack.push(val)
                }
                // instanceof
                193 => {
                    let val = self.pop();
                    let cp_idx = self.read_u16();

                    if matches!(val, Value::Object(None) | Value::Array(None)) {
                        self.operand_stack.push(Value::Int(0));
                        continue;
                    }

                    let obj_class = match val {
                        Value::Object(Some(obj)) => heap().get_obj_class(obj),
                        Value::Array(Some(arr)) => {
                            let heap = heap();
                            let elem_ty = heap.arr_ty(arr);
                            method_area().resolve_arr_class(elem_ty)
                        }
                        a => unreachable!("{a:?}"),
                    };

                    let class_id = self.class_id();
                    let ref_class = Class::class_reference(class_id, cp_idx);

                    let instance_of = Class::instance_of(obj_class, ref_class);
                    self.operand_stack.push(Value::Int(instance_of as i32))
                }
                // monitorenter
                194 => {
                    let _obj = self.pop().object();
                    println!("todo inst: monitorenter");
                }
                // monitorexit
                195 => {
                    let _obj = self.pop().object();
                    println!("todo inst: monitorexit");
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
