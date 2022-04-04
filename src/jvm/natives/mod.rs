mod class;
mod object;
mod string;

use super::Thread;

pub fn run_native(thread: &mut Thread, class: String, method: String) {
    match (class.as_str(), method.as_str()) {
        ("java/lang/Object", "getClass") => object::get_class(thread),
        ("java/lang/Class", "desiredAssertionStatus0") => class::desired_assertion_status(thread),
        ("java/lang/StringUTF16", "isBigEndian") => string::is_big_endian(thread),
        _ => println!("Unimplemented native: {}.{}", &class, &method),
    }
}
