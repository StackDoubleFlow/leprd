mod cds;
mod class;
mod float;
mod object;
mod string;

use super::Thread;

pub fn run_native(thread: &mut Thread, class: String, method: String) {
    match (class.as_str(), method.as_str()) {
        ("java/lang/Object", "getClass") => object::get_class(thread),
        ("java/lang/Class", "desiredAssertionStatus0") => class::desired_assertion_status(thread),
        ("java/lang/Class", "getPrimitiveClass") => class::get_primitive_class(thread),
        ("java/lang/StringUTF16", "isBigEndian") => string::is_big_endian(thread),
        ("java/lang/Float", "intBitsToFloat") => float::int_bits_to_float(thread),
        ("java/lang/Float", "floatToRawIntBits") => float::float_to_int_bits(thread),
        ("java/lang/Double", "longBitsToDouble") => float::long_bits_to_double(thread),
        ("java/lang/Double", "doubleToRawLongBits") => float::double_to_long_bits(thread),
        ("jdk/internal/misc/CDS", "isDumpingClassList0") => cds::is_dumping_class_list(thread),
        ("jdk/internal/misc/CDS", "isDumpingArchive0") => cds::is_dumping_archive(thread),
        ("jdk/internal/misc/CDS", "isSharingEnabled0") => cds::is_sharing_enabled(thread),
        ("jdk/internal/misc/CDS", "getRandomSeedForDumping") => {
            cds::get_random_seed_for_dumping(thread)
        }
        ("jdk/internal/misc/CDS", "initializeFromArchive") => cds::intialize_from_archive(thread),
        _ => println!("Unimplemented native: {}.{}", &class, &method),
    }
}
