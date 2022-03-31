use crate::class_file::CPInfo;
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
    super_class: ClassId,
}

pub fn resolve_class(name: &str) -> ClassId {
    let method_area = METHOD_AREA.lock().unwrap();
    if let Some(&id) = method_area.class_map.get(name) {
        id
    } else {
        // TODO: Load using correct class loader
        load_class_bootstrap(name)
    }
}

fn cp_utf8(cp: &[CPInfo], idx: u16) -> String {
    match &cp[idx as usize - 1] {
        CPInfo::Utf8 { bytes, .. } => String::from_utf8(bytes.clone()).unwrap(),
        _ => panic!("ClassFormatError"),
    }
}

fn cp_class_name(cp: &[CPInfo], idx: u16) -> String {
    match cp[idx as usize - 1] {
        CPInfo::Class { name_index } => cp_utf8(cp, name_index),
        _ => panic!("ClassFormatError"),
    }
}

pub fn load_class_bootstrap(name: &str) -> ClassId {
    let classpath = CONFIG.classpath;
    // let mut method_area = METHOD_AREA.lock().unwrap();

    // if method_area.class_map.contains_key(name) {
    //     panic!("LinkageError");
    // }

    let mut path = PathBuf::from(classpath);
    path.push(name);
    path.set_extension("class");
    dbg!(&path);

    let data = fs::read(path).expect("ClassNotFoundException");
    let class_file = ClassFile::from_bytes((&data, 0)).unwrap().1;

    assert!(class_file.magic == 0xCAFEBABE);
    assert!((45..62).contains(&class_file.major_version));

    let super_name = cp_class_name(&class_file.constant_pool, class_file.super_class);
    let super_class = resolve_class(&super_name);

    let name = cp_class_name(&class_file.constant_pool, class_file.this_class);

    dbg!(class_file);

    let class = Class {
        defining_loader: ClassLoader::Bootstrap,
        name: name.clone(),
        super_class
    };

    let mut method_area = METHOD_AREA.lock().unwrap();
    let id = method_area.classes.alloc(class);
    method_area.class_map.insert(name, id);
    id
}
