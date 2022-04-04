use super::Thread;
use crate::class::Class;
use crate::class_loader::method_area;
use crate::value::Value;

impl Thread {
    pub fn run(&mut self) {
        loop {
            let opcode = self.read_ins();
            dbg!(opcode);
            match opcode {
                // nop
                0 => {}
                // aconst_null
                1 => self.operand_stack.push(Value::Object(None)),
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
