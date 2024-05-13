use crate::heap::heap;
use crate::jvm::Thread;
use crate::value::Value;
use nix::libc::{SIGHUP, SIGINT, SIGTERM};

pub fn find_signal(thread: &mut Thread) {
    let Some(name_obj) = thread.pop().object() else {
        panic!("NullPointerException");
    };
    let str = heap().read_string(name_obj);
    // TODO: Signals on windows
    let sig = match str.as_str() {
        "INT" => SIGINT,
        "HUP" => SIGHUP,
        "TERM" => SIGTERM,
        _ => -1,
    };
    thread.operand_stack.push(Value::Int(sig));
}

pub fn handle(thread: &mut Thread) {
    let _native_handler = thread.pop().long();
    let _sig = thread.pop().int();
    // TODO: set up signal handlers
    thread.operand_stack.push(Value::Long(0));
}
