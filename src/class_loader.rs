use id_arena::{Arena, Id};
use std::collections::HashMap;
use std::lazy::SyncLazy;
use std::path::PathBuf;
use std::sync::Mutex;
use std::{fs, str};

static METHOD_AREA: SyncLazy<Mutex<MethodArea>> = SyncLazy::new(Default::default);

use deku::DekuContainerRead;

use crate::class_file::ClassFile;
use crate::CONFIG;

type ClassId = Id<Class>;
// type MethodId = Id<Method>;
// type FieldId = Id<Field>;

enum ClassLoader {
    Bootstrap,
    UserDefined(/* TODO */),
}

#[derive(Default)]
struct MethodArea {
    classes: Arena<Class>,
    class_map: HashMap<String, ClassId>,
}

pub struct Class {
    defining_loader: ClassLoader,
    name: String,
    super_class: Option<ClassId>,
    interfaces: Vec<ClassId>,
    access_flags: u16,
}


pub fn resolve_class(name: &str) -> ClassId {
    let method_area = METHOD_AREA.lock().unwrap();
    let id = method_area.class_map.get(name).cloned();
    drop(method_area);
    match id {
        Some(id) => id,
        None => load_class_bootstrap(name),
    }
}

pub fn load_class_bootstrap(name: &str) -> ClassId {
    // let mut method_area = METHOD_AREA.lock().unwrap();

    // if method_area.class_map.contains_key(name) {
    //     panic!("LinkageError");
    // }

    let data = CONFIG.classpath.iter().find_map(|classpath| {
        let mut path = PathBuf::from(classpath);
        path.push(name);
        path.set_extension("class");
        if path.exists() {
            dbg!(&path);
            Some(fs::read(path).unwrap())
        } else {
            None
        }
    }).expect("ClassNotFoundException");
    let class_file = ClassFile::from_bytes((&data, 0)).expect("ClassFormatError").1;

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

    let class = Class {
        defining_loader: ClassLoader::Bootstrap,
        name: name.clone(),
        super_class,
        interfaces,
        access_flags: class_file.access_flags,
    };

    let mut method_area = METHOD_AREA.lock().unwrap();
    let id = method_area.classes.alloc(class);
    method_area.class_map.insert(name, id);

    let class = &method_area.classes[id];
    for &interface in &class.interfaces {
        if interface == id {
            panic!("ClassCircularityError");
        }
    }

    id
}
