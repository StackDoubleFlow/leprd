use crate::class::Class;
use crate::class_file::attributes::CodeAttribute;
use crate::class_loader::{method_area, ClassId, MethodId};
use crate::value::Value;
use std::collections::HashMap;
use std::mem;
use std::sync::Arc;

struct StackFrame {
    method: MethodId,
    return_pc: usize,
    operand_stack: Vec<Value>,
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

    fn call_method(&mut self, method_id: MethodId) {
        println!("Calling method: {}", method_area().methods[method_id].name);
        // TODO: params
        for _ in 0..method_area().methods[method_id].descriptor.0.len() {}
        let stack_frame = StackFrame {
            method: self.method,
            operand_stack: mem::take(&mut self.operand_stack),
            return_pc: self.pc,
        };
        self.stack_frames.push(stack_frame);
        self.method = method_id;
        self.code = method_area().methods[method_id].code.clone().unwrap();
        self.pc = 0;
    }

    fn ensure_initialized(&mut self, class_id: ClassId) {
        let ma = method_area();
        let class = &ma.classes[class_id];
        if !class.initialized {
            println!("Initializing class: {}", class.name);
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
        method_area().classes[class_id].initialized = true;
    }

    pub fn run(&mut self) {
        loop {
            let opcode = self.read_ins();
            match opcode {
                // nop
                0 => {}
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
                // invokestatic
                184 => {
                    let idx = self.read_u16();
                    let class_id = self.class_id();
                    let method = Class::method_reference(class_id, idx);
                    self.call_method(method);
                }
                _ => unimplemented!("opcode: {}", opcode),
            }
        }
    }
}
