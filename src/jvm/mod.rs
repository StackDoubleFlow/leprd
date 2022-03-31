use crate::class_file::attributes::CodeAttribute;
use crate::class_loader::{method_area, ClassId, MethodId};
use std::sync::Arc;

struct StackFrame {
    method: MethodId,
    return_pc: usize,
    operand_stack: Vec<u8>,
}

pub struct Thread {
    method: MethodId,
    code: Arc<CodeAttribute>,
    pc: usize,
    operand_stack: Vec<u8>,
}

impl Thread {
    pub fn new(entry_method: MethodId) -> Thread {
        let code = method_area().methods[entry_method]
            .code
            .clone()
            .unwrap();
        Thread {
            method: entry_method,
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
