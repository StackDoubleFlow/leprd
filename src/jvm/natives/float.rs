use crate::jvm::Thread;
use crate::value::Value;

pub fn int_bits_to_float(thread: &mut Thread) {
    let val = match thread.pop() {
        Value::Int(val) => val,
        _ => unreachable!(),
    };
    let float = f32::from_bits(val as u32);
    thread.operand_stack.push(Value::Float(float));
}

pub fn float_to_int_bits(thread: &mut Thread) {
    let val = match thread.pop() {
        Value::Float(val) => val,
        _ => unreachable!(),
    };
    let bits = val.to_bits();
    thread.operand_stack.push(Value::Int(bits as i32));
}

pub fn long_bits_to_double(thread: &mut Thread) {
    let val = match thread.pop() {
        Value::Long(val) => val,
        _ => unreachable!(),
    };
    let double = f64::from_bits(val as u64);
    thread.operand_stack.push(Value::Double(double));
}

pub fn double_to_long_bits(thread: &mut Thread) {
    let val = match thread.pop() {
        Value::Double(val) => val,
        _ => unreachable!(),
    };
    let bits = val.to_bits();
    thread.operand_stack.push(Value::Long(bits as i64));
}
