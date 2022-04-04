mod exec;

use crate::class_file::attributes::CodeAttribute;
use crate::class_loader::{method_area, ClassId, MethodId};
use crate::heap::{heap, Array, Object, ObjectId};
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

    // fn pop_u16(&mut self) -> u16 {
    //     (self.operand_stack.pop().unwrap() as u16) << 8 | self.operand_stack.pop().unwrap() as u16
    // }

    fn class_id(&self) -> ClassId {
        method_area().methods[self.method].defining_class
    }

    fn call_method(&mut self, method_id: MethodId, is_static: bool) {
        let ma = method_area();
        let method = &ma.methods[method_id];
        println!(
            "Calling method: {}.{}",
            ma.classes[method.defining_class].name, method.name
        );
        if method.code.is_none() {
            println!("TODO: Native method call");
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
        self.run();

        let stack_frame = self.stack_frames.pop().unwrap();
        self.method = stack_frame.method;
        self.code = method_area().methods[self.method].code.clone().unwrap();
        self.pc = stack_frame.return_pc;
        self.locals = stack_frame.locals;
    }

    fn ensure_initialized(&mut self, class_id: ClassId) {
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
                self.call_method(method, true);
            }
        }
    }

    fn create_string(&mut self, str: &str) -> ObjectId {
        let arr: Box<[Value]> = str.chars().map(Value::Char).collect();
        let arr_id = heap().arrays.alloc(Array { contents: arr });

        let str_class = method_area().resolve_class("java/lang/String");
        let str_obj = Object::new(str_class);
        self.operand_stack.push(Value::Object(Some(str_obj)));
        self.operand_stack.push(Value::Array(Some(arr_id)));
        let str_constructor = method_area().resolve_method(str_class, "<init>", "([C)V");
        self.call_method(str_constructor, false);
        str_obj
    }
}
