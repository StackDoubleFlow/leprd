use crate::class_file::attributes::CodeAttribute;
use crate::class_loader::{method_area, ClassId};
use std::sync::Arc;

struct StackFrame {
    class: ClassId,
    method: String,
    return_pc: usize,
    operand_stack: Vec<u8>,
}

pub struct Thread {
    class: ClassId,
    method: String,
    code: Arc<CodeAttribute>,
    pc: usize,
    operand_stack: Vec<u8>,
}

impl Thread {
    pub fn new(entry_class: ClassId, entry_method: &str) -> Thread {
        let code = method_area().classes[entry_class].methods[entry_method]
            .code
            .clone()
            .unwrap();
        Thread {
            class: entry_class,
            method: entry_method.to_string(),
            code,
            pc: 0,
            operand_stack: Vec::new(),
        }
    }

    fn read_ins(&mut self) -> u8 {
        let data = self.code.code[self.pc];
        self.pc += 1;
        data
    }

    pub fn run(&mut self) {
        loop {
            let opcode = self.read_ins();
            match opcode {
                0 => {}
                _ => unimplemented!("opcode: {}", opcode),
            }
        }
    }
}
