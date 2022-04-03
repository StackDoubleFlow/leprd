use byteorder::{ReadBytesExt, BE};

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
        let code = method_area().methods[entry_method].code.clone().unwrap();
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

    fn read_u16(&mut self) -> u16 {
        (self.read_ins() as u16) << 8 | self.read_ins() as u16
    }

    fn pop_u16(&mut self) -> u16 {
        (self.operand_stack.pop().unwrap() as u16) << 8 | self.operand_stack.pop().unwrap() as u16
    }

    fn class_id(&self) -> ClassId {
        method_area().methods[self.method].defining_class
    }

    pub fn run(&mut self) {
        loop {
            let opcode = self.read_ins();
            match opcode {
                0 => {}
                178 => {
                    let idx = self.read_u16();
                    let class_id = self.class_id();
                    let field = method_area().classes[class_id].field_reference(idx);
                }
                _ => unimplemented!("opcode: {}", opcode),
            }
        }
    }
}
