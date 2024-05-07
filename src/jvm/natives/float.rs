use crate::jvm::Thread;
use crate::value::Value;

pub fn int_bits_to_float(thread: &mut Thread) {
    let val = thread.pop().int();
    let float = f32::from_bits(val as u32);
    thread.operand_stack.push(Value::Float(float));
}

pub fn float_to_int_bits(thread: &mut Thread) {
    let val = thread.pop().float();
    let bits = val.to_bits();
    thread.operand_stack.push(Value::Int(bits as i32));
}

pub fn long_bits_to_double(thread: &mut Thread) {
    let val = thread.pop().long();
    let double = f64::from_bits(val as u64);
    thread.operand_stack.push(Value::Double(double));
}

pub fn double_to_long_bits(thread: &mut Thread) {
    let val = thread.pop().double();
    let bits = val.to_bits();
    thread.operand_stack.push(Value::Long(bits as i64));
}
