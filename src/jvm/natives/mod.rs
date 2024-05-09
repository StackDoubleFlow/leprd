mod cds;
mod class;
mod float;
mod jdk_unsafe;
mod object;
mod reflection;
mod runtime;
mod string;
mod system;
mod system_props;

use super::Thread;

pub fn run_native(thread: &mut Thread, class: String, method: String) {
    match (class.as_str(), method.as_str()) {
        ("java/lang/System", "registerNatives") => println!("stub: native System.registerNatives"),
        ("java/lang/Class", "registerNatives") => println!("stub: native Class.registerNatives"),
        ("jdk/internal/misc/Unsafe", "registerNatives") => {
            println!("stub: native Unsafe.registerNatives")
        }

        ("java/lang/System", "arraycopy") => system::arraycopy(thread),
        ("java/lang/Runtime", "availableProcessors") => runtime::available_processors(thread),
        ("java/lang/Object", "getClass") => object::get_class(thread),
        ("java/lang/Object", "hashCode") => object::hash_code(thread),
        ("java/lang/Class", "desiredAssertionStatus0") => class::desired_assertion_status(thread),
        ("java/lang/Class", "getPrimitiveClass") => class::get_primitive_class(thread),
        ("java/lang/Class", "isPrimitive") => class::is_primitive(thread),
        ("java/lang/Class", "initClassName") => class::init_class_name(thread),
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
        ("jdk/internal/misc/Unsafe", "arrayIndexScale0") => jdk_unsafe::array_index_scale(thread),
        ("jdk/internal/misc/Unsafe", "arrayBaseOffset0") => jdk_unsafe::array_base_offset(thread),
        ("jdk/internal/misc/Unsafe", "objectFieldOffset1") => {
            jdk_unsafe::object_field_offset(thread)
        }
        ("jdk/internal/reflect/Reflection", "getCallerClass") => {
            reflection::get_caller_class(thread)
        }
        ("jdk/internal/util/SystemProps$Raw", "platformProperties") => {
            system_props::platform_properties(thread)
        }
        ("jdk/internal/util/SystemProps$Raw", "vmProperties") => {
            system_props::vm_properties(thread)
        }

        _ => {
            thread.print_stacktrace();
            unimplemented!("native: {}.{}", &class, &method);
        }
    }
}
