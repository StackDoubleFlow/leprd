
use std::os::fd::RawFd;
use crate::jvm::Thread;
use crate::value::Value;
use nix::fcntl::{fcntl, FcntlArg};
use nix::libc::O_APPEND;

pub fn get_handle(thread: &mut Thread) {
    let _fd = thread.pop().int();
    println!("todo: native java.io.FileDescriptor.getHandle");
    thread.operand_stack.push(Value::Long(0));
}

pub fn get_append(thread: &mut Thread) {
    let fd = thread.pop().int();
    let flags = fcntl(fd as RawFd, FcntlArg::F_GETFL).unwrap();
    let append_set = flags & O_APPEND != 0;
    // boolean return
    thread.operand_stack.push(Value::Int(append_set as i32));
}
