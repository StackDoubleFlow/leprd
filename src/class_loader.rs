use crate::class::{Class, Field, Method, Reference};
use crate::class_file::constant_pool::CPInfo;
use crate::class_file::ClassFile;
use crate::CONFIG;
use deku::DekuContainerRead;
use id_arena::{Arena, Id};
use std::collections::HashMap;
use std::lazy::SyncLazy;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard};
use std::{fs, str};

static METHOD_AREA: SyncLazy<Mutex<MethodArea>> = SyncLazy::new(Default::default);

/// Retrieve the method area by locking the mutex
pub fn method_area() -> MutexGuard<'static, MethodArea> {
    METHOD_AREA.lock().unwrap()
}

pub type ClassId = Id<Class>;
pub type MethodId = Id<Method>;
pub type FieldId = Id<Field>;

#[derive(Debug)]
pub enum ClassLoader {
    Bootstrap,
    UserDefined(/* TODO */),
}

#[derive(Default, Debug)]
pub struct MethodArea {
    pub classes: Arena<Class>,
    pub class_map: HashMap<String, ClassId>,
    pub methods: Arena<Method>,
    pub fields: Arena<Field>,
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

    // No new classes can be loaded after this point
    let class_id = method_area().classes.next_id();

    let mut methods = Vec::new();
    for method in class_file.methods {
        let name = class_file.constant_pool.utf8(method.name_index);
        let descriptor = class_file.constant_pool.utf8(method.descriptor_index);
        let mut code = None;
        for attr in method.attributes {
            let attr_name = class_file.constant_pool.utf8(attr.attribute_name_index);
            if attr_name == "Code" {
                code = Some(Arc::new(attr.code()));
            }
        }
        let id = method_area().methods.alloc(Method {
            defining_class: class_id,
            name,
            descriptor,
            access_flags: method.access_flags,
            code,
        });
        methods.push(id);
    }

    let mut fields = Vec::new();
    for field in class_file.fields {
        let name = class_file.constant_pool.utf8(field.name_index);
        let id = method_area().fields.alloc(Field {
            name,
            defining_class: class_id,
            access_flags: field.access_flags,
        });
        fields.push(id);
    }

    let mut references = HashMap::new();
    for (i, entry) in class_file.constant_pool.table.iter().enumerate() {
        match entry {
            CPInfo::Fieldref { .. } | CPInfo::Methodref { .. } | CPInfo::Class { .. } => {
                references.insert(i as u16, Reference::Unresolved);
            }
            _ => {}
        }
    }

    let class = Class {
        defining_loader: ClassLoader::Bootstrap,
        references,
        name: name.clone(),
        super_class,
        interfaces,
        methods,
        fields,
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
