mod object;

use super::Thread;

pub fn run_native(thread: &mut Thread, class: String, method: String) {
    match (class.as_str(), method.as_str()) {
        ("java/lang/Object", "getClass") => object::get_class(thread),
        _ => println!("Unimplemented native: {}.{}", &class, &method),
    }
}
