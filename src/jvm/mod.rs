mod exec;
mod natives;

use crate::class_file::attributes::CodeAttribute;
use crate::class_file::descriptors::{BaseType, FieldType};
use crate::class_file::methods;
use crate::class_loader::{method_area, ClassId, MethodId};
use crate::heap::{heap, ObjectRef};
use crate::value::Value;
use std::mem;
use std::sync::Arc;

struct StackFrame {
    method: MethodId,
    return_pc: usize,
    operand_stack: Vec<Value>,
    locals: Vec<Option<Value>>,
}

pub struct Thread {
    method: MethodId,
    code: Arc<CodeAttribute>,
    pc: usize,
    operand_stack: Vec<Value>,
    locals: Vec<Option<Value>>,
    stack_frames: Vec<StackFrame>,
}

impl Thread {
    pub fn new(entry_method: MethodId) -> Thread {
        let method = &method_area().methods[entry_method];
        let code = method.code.clone().unwrap();
        let max_locals = code.max_locals as usize;
        Thread {
            method: entry_method,
            code,
            pc: 0,
            operand_stack: Vec::new(),
            stack_frames: Vec::new(),
            locals: vec![None; max_locals],
        }
    }

    fn read_ins(&mut self) -> u8 {
        let data = self.code.code[self.pc];
        self.pc += 1;
        data
    }

    fn read_u16(&mut self) -> u16 {
        (self.read_ins() as u16) << 8 | self.read_ins() as u16
    }

    fn read_i32(&mut self) -> i32 {
        (self.read_ins() as i32) << 24
            | (self.read_ins() as i32) << 16
            | (self.read_ins() as i32) << 8
            | self.read_ins() as i32
    }

    fn pad_to_int(&mut self) {
        if self.pc & 0b11 != 0 {
            self.pc += 4 - (self.pc & 0b11);
        }
    }

    fn class_id(&self) -> ClassId {
        method_area().methods[self.method].defining_class
    }

    pub fn call_method(&mut self, method_id: MethodId) {
        let ma = method_area();
        let method = &ma.methods[method_id];
        let is_static = method.access_flags & methods::acc::STATIC != 0;
        println!(
            "Calling method: {}.{}",
            ma.classes[method.defining_class].name, method.name
        );
        if method.access_flags & methods::acc::NATIVE != 0 {
            let class_name = ma.classes[method.defining_class].name.clone();
            let method_name = method.name.clone();
            drop(ma);
            natives::run_native(self, class_name, method_name);
            return;
        }

        let mut locals = vec![None; method.code.as_ref().unwrap().max_locals as usize];
        let mut num_params = method.descriptor.0.len();
        if !is_static {
            // objectref
            num_params += 1;
        }
        let mut cur_local = 0;
        for i in 0..num_params {
            let stack_idx = self.operand_stack.len() - num_params + i;
            let val = self.operand_stack[stack_idx];
            locals[cur_local] = Some(val);
            match val {
                Value::Long(_) | Value::Double(_) => cur_local += 2,
                _ => cur_local += 1,
            }
        }
        self.operand_stack
            .truncate(self.operand_stack.len() - num_params);

        let stack_frame = StackFrame {
            method: self.method,
            operand_stack: mem::take(&mut self.operand_stack),
            return_pc: self.pc,
            locals: mem::replace(&mut self.locals, locals),
        };
        self.stack_frames.push(stack_frame);
        self.method = method_id;
        self.code = method.code.clone().unwrap();
        self.pc = 0;

        drop(ma);
        let res = self.run();

        let stack_frame = self.stack_frames.pop().unwrap();
        self.method = stack_frame.method;
        self.code = method_area().methods[self.method].code.clone().unwrap();
        self.pc = stack_frame.return_pc;
        self.operand_stack = stack_frame.operand_stack;
        self.locals = stack_frame.locals;
        if let Some(res) = res {
            self.operand_stack.push(res);
        }
        println!(
            "Returned to method with {} locals, pc:{}",
            self.locals.len(),
            self.pc
        );
    }

    pub fn ensure_initialized(&mut self, class_id: ClassId) {
        let mut ma = method_area();
        let class = &mut ma.classes[class_id];
        if !class.initialized {
            println!("Initializing class: {}", class.name);
            class.initialized = true;
            let class = &ma.classes[class_id];
            if let Some(method) = class
                .methods
                .iter()
                .cloned()
                .find(|&mid| ma.methods[mid].name == "<clinit>")
            {
                drop(ma);
                self.call_method(method);
            }
        }
    }

    fn br_if(&mut self, cur_pc: usize, cond: bool) {
        let target = cur_pc as isize + self.read_u16() as i16 as isize;
        if cond {
            self.pc = target as usize;
        }
    }

    fn pop(&mut self) -> Value {
        self.operand_stack
            .pop()
            .expect("tried to pop value off of operand stack while stack is empty")
    }

    fn arr_load(&mut self) -> Value {
        let idx = self.pop().int() as usize;
        let Some(arr) = self.pop().array() else {
            panic!("NullPointerException");
        };
        heap().load_arr_elem(arr, idx)
    }

    // Only used for debugging
    #[allow(dead_code)]
    fn debug_value(&mut self, val: Value) {
        println!("{:?}", ValueDebugger(val));
    }

    /// For method selection in invokeinterface and invokevirtual instructions
    fn select_method(&mut self, method_id: MethodId) -> MethodId {
        let ma = method_area();
        let method = &ma.methods[method_id];
        let num_params = method.descriptor.0.len();

        let stack_obj_idx = self.operand_stack.len() - num_params - 1;
        let obj = match self.operand_stack[stack_obj_idx].object() {
            Some(obj) => obj,
            None => panic!("NullPointerException"),
        };
        let obj_class = heap().get_obj_class(obj);

        let mut cur_class = obj_class;
        loop {
            let c = &ma.classes[cur_class];
            let m = c.methods.iter().find(|&m| {
                let m = &ma.methods[*m];
                m.name == method.name && m.descriptor == method.descriptor
            });
            if let Some(&m) = m {
                break m;
            }
            match c.super_class {
                Some(s) => cur_class = s,
                // Could not find an overriding method
                None => break method_id,
            }
        }
    }

    fn print_stacktrace(&self) {
        let ma = method_area();
        println!("stacktrace:");
        for (idx, frame) in self.stack_frames.iter().enumerate() {
            let method = &ma.methods[frame.method];
            let class = &ma.classes[method.defining_class];
            println!("  {}: {}::{}", idx, class.name, method.name);
        }
    }
}

/// This is a different, more detailed, Debug impl for Value
struct ValueDebugger(Value);

impl std::fmt::Debug for ValueDebugger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Value::Object(Some(obj)) => {
                let class_id = heap().get_obj_class(obj);
                let name = method_area().classes[class_id].name.replace('/', ".");
                let mut dbg = f.debug_struct(&name);

                let mut cur_class = class_id;
                loop {
                    let fields = method_area().classes[cur_class].fields.clone();
                    for field in fields {
                        if method_area().fields[field].is_static() {
                            continue;
                        }
                        let name = method_area().fields[field].name.clone();
                        let val = heap().load_field(obj, field).clone();
                        if matches!(val, Value::Object(Some(_))) {
                            // this is to avoid infinite recursion
                            dbg.field(&name, &val);
                        } else {
                            dbg.field(&name, &ValueDebugger(val));
                        }
                    }
                    match method_area().classes[cur_class].super_class {
                        Some(s) => cur_class = s,
                        None => break,
                    }
                }

                dbg.finish()
            }
            Value::Object(None) | Value::Array(None) => write!(f, "null"),
            _ => write!(f, "{:?}", self.0),
        }
    }
}
