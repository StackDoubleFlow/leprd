use crate::class::{Class, Field, Method};
use crate::class_file::ClassFile;
use crate::CONFIG;
use deku::DekuContainerRead;
use id_arena::{Arena, Id};
use std::collections::HashMap;
use std::lazy::SyncLazy;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};
use std::{fs, str};

static METHOD_AREA: SyncLazy<Mutex<MethodArea>> = SyncLazy::new(Default::default);

/// Retrieve the method area by locking the mutex
pub fn method_area() -> MutexGuard<'static, MethodArea> {
    METHOD_AREA.lock().unwrap()
}

pub type ClassId = Id<Class>;
// type MethodId = Id<Method>;
// type FieldId = Id<Field>;

#[derive(Debug)]
pub enum ClassLoader {
    Bootstrap,
    UserDefined(/* TODO */),
}

#[derive(Default)]
pub struct MethodArea {
    pub classes: Arena<Class>,
    pub class_map: HashMap<String, ClassId>,
}

pub fn resolve_class(name: &str) -> ClassId {
    let id = method_area().class_map.get(name).cloned();
    match id {
        Some(id) => id,
        None => load_class_bootstrap(name),
    }
}

pub fn load_class_bootstrap(name: &str) -> ClassId {
    if method_area().class_map.contains_key(name) {
        panic!("LinkageError");
    }

    let data = CONFIG
        .classpath
        .iter()
        .find_map(|classpath| {
            let mut path = PathBuf::from(classpath);
            path.push(name);
            path.set_extension("class");
            if path.exists() {
                dbg!(&path);
                Some(fs::read(path).unwrap())
            } else {
                None
            }
        })
        .expect("ClassNotFoundException");
    let class_file = ClassFile::from_bytes((&data, 0))
        .expect("ClassFormatError")
        .1;

    assert!(class_file.magic == 0xCAFEBABE);
    assert!((45..62).contains(&class_file.major_version));

    let super_class = if class_file.super_class > 0 {
        let super_name = class_file.constant_pool.class_name(class_file.super_class);
        Some(resolve_class(&super_name))
    } else {
        // Super class should only be None for Object
        None
    };
    let name = class_file.constant_pool.class_name(class_file.this_class);

    let mut interfaces = Vec::new();
    for interface in class_file.interfaces {
        let interface_name = class_file.constant_pool.class_name(interface);
        interfaces.push(resolve_class(&interface_name));
    }

    let mut methods = HashMap::new();
    for method in class_file.methods {
        let name = class_file.constant_pool.utf8(method.name_index);
        let mut code = None;
        for attr in method.attributes {
            let attr_name = class_file.constant_pool.utf8(attr.attribute_name_index);
            if attr_name == "Code" {
                code = Some(attr.info.into());
            }
        }

        methods.insert(
            name,
            Method {
                access_flags: method.access_flags,
                code,
            },
        );
    }

    let mut fields = HashMap::new();
    for field in class_file.fields {
        let name = class_file.constant_pool.utf8(field.name_index);
        fields.insert(
            name,
            Field {
                access_flags: field.access_flags,
            },
        );
    }

    let class = Class {
        defining_loader: ClassLoader::Bootstrap,
        name: name.clone(),
        super_class,
        interfaces,
        methods,
        access_flags: class_file.access_flags,
        constant_pool: class_file.constant_pool,
    };

    let mut method_area = method_area();
    let id = method_area.classes.alloc(class);
    method_area.class_map.insert(name, id);

    // See if this class is an interface of itself
    let class = &method_area.classes[id];
    for &interface in &class.interfaces {
        if interface == id {
            panic!("ClassCircularityError");
        }
    }

    id
}
