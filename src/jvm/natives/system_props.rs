use crate::class_file::descriptors::{FieldType, ObjectType};
use crate::class_loader::method_area;
use crate::heap::{heap, ArrayRef};
use crate::jvm::Thread;
use crate::value::Value;

pub fn vm_properties(thread: &mut Thread) {
    let mut props = Vec::new();

    // Simple macro to avoid to_string repetition everywhere
    macro_rules! prop {
        ($key:expr, $val:expr) => {
            props.push($key.to_string());
            props.push($val.to_string());
        };
    }

    // See: https://docs.oracle.com/en/java/javase/22/docs/api/java.base/java/lang/System.html#getProperties()
    prop!("java.version", "22");
    prop!("java.vendor", "StackDoubleFlow");
    prop!("java.vendor.url", "https://stackdoubleflow.net/");
    prop!("java.home", env!("CARGO_MANIFEST_DIR"));
    prop!("java.vm.specification.version", "22");
    prop!("java.vm.specification.vendor", "Oracle");
    prop!(
        "java.vm.specification.name",
        "Java Virtual Machine Specification"
    );
    prop!("java.vm.version", env!("CARGO_PKG_VERSION"));
    prop!("java.vm.vendor", env!("CARGO_PKG_AUTHORS"));
    prop!("java.vm.name", env!("CARGO_PKG_NAME"));
    prop!("java.specification.version", "22");
    prop!("java.specification.vendor", "Oracle");
    prop!("java.specification.name", "Java Platform API Specification");
    prop!("java.io.tmpdir", std::env::temp_dir().to_string_lossy());
    prop!("os.name", whoami::platform());
    prop!("os.arch", whoami::arch());
    prop!("os.version", whoami::distro());
    prop!("user.name", whoami::username());
    prop!("user.home", dirs::home_dir().unwrap().to_string_lossy());
    prop!(
        "user.dir",
        std::env::current_dir().unwrap().to_string_lossy()
    );
    prop!("native.encoding", "UTF-8");
    prop!("stdout.encoding", "UTF-8");
    prop!("stderr.encoding", "UTF-8");

    let props: Vec<_> = props
        .into_iter()
        .map(|prop| heap().create_string(&mut method_area(), &prop))
        .collect();

    let mut heap = heap();
    let arr = heap.new_array(
        &mut method_area(),
        FieldType::ObjectType(ObjectType {
            class_name: "java/lang/String".to_string(),
        }),
        props.len(),
    );
    for (idx, prop) in props.into_iter().enumerate() {
        heap.store_arr_elem(arr, idx, Value::Object(Some(prop)));
    }

    thread.operand_stack.push(Value::Array(Some(arr)));
}

pub fn platform_properties(thread: &mut Thread) {
    let arr = heap().new_array(
        &mut method_area(),
        FieldType::ObjectType(ObjectType {
            class_name: "java/lang/String".to_string(),
        }),
        39,
    );

    thread.operand_stack.push(Value::Array(Some(arr)));
}
