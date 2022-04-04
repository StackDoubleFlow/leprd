mod class;
mod object;
mod string;
mod cds;

use super::Thread;

pub fn run_native(thread: &mut Thread, class: String, method: String) {
    match (class.as_str(), method.as_str()) {
        ("java/lang/Object", "getClass") => object::get_class(thread),
        ("java/lang/Class", "desiredAssertionStatus0") => class::desired_assertion_status(thread),
        ("java/lang/StringUTF16", "isBigEndian") => string::is_big_endian(thread),
        ("jdk/internal/misc/CDS", "isDumpingClassList0") => cds::is_dumping_class_list(thread),
        ("jdk/internal/misc/CDS", "isDumpingArchive0") => cds::is_dumping_archive(thread),
        ("jdk/internal/misc/CDS", "isSharingEnabled0") => cds::is_sharing_enabled(thread),
        _ => println!("Unimplemented native: {}.{}", &class, &method),
    }
}
