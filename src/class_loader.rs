use crate::class::{Class, Field, Method, Reference};
use crate::class_file::constant_pool::CPInfo;
use crate::class_file::descriptors::{FieldDescriptor, MethodDescriptor};
use crate::class_file::{fields, ClassFile};
use crate::heap::ObjectId;
use crate::value::Value;
use crate::CONFIG;
use deku::DekuContainerRead;
use id_arena::{Arena, Id};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::LazyLock;
use std::sync::{Arc, Mutex, MutexGuard};
use std::{fs, str};

static METHOD_AREA: LazyLock<Mutex<MethodArea>> = LazyLock::new(Default::default);

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
    pub class_objs: HashMap<ObjectId, ClassId>,
    pub methods: Arena<Method>,
    pub fields: Arena<Field>,
}

impl MethodArea {
    fn resolve_arr_class(&mut self, desc: FieldDescriptor) -> Id<Class> {
        let name = format!("{}", desc.0);

        let id = self.class_map.get(&name).cloned();
        match id {
            Some(id) => id,
            None => load_arr_class_bootstrap(self, &name),
        }
    }

    pub fn resolve_class(&mut self, name: &str) -> ClassId {
        if name.starts_with('[') {
            let desc = FieldDescriptor::read(name);
            return self.resolve_arr_class(desc);
        }

        let id = self.class_map.get(name).cloned();
        match id {
            Some(id) => id,
            None => load_class_bootstrap(self, name),
        }
    }

    pub fn resolve_method(
        &self,
        class: ClassId,
        name: &str,
        descriptor: &MethodDescriptor,
    ) -> MethodId {
        // 5.4.3.3. Method Resolution
        let class = &self.classes[class];
        let method = class
            .methods
            .iter()
            .copied()
            .find(|&id| {
                let method = &self.methods[id];
                method.name == name && method.descriptor == *descriptor
            });

        match method {
            Some(method) => method,
            None => {
                if let Some(super_class) = class.super_class {
                    self.resolve_method(super_class, name, descriptor)
                } else {
                    panic!("NoSuchMethodError");
                }
            }
        }
    }

    pub fn resolve_field(&self, class: ClassId, name: &str) -> FieldId {
        // 5.4.3.2. Field Resolution
        let class = &self.classes[class];
        let field = class
            .fields
            .iter()
            .copied()
            .find(|&id| {
                let field = &self.fields[id];
                field.name == name
            });
        
        match field {
            Some(field) => field,
            None => {
                if let Some(super_class) = class.super_class {
                    self.resolve_field(super_class, name)
                } else {
                    panic!("NoSuchFieldError");
                }
            }
        }
    }
}

pub fn load_class_bootstrap(ma: &mut MethodArea, name: &str) -> ClassId {
    if ma.class_map.contains_key(name) {
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
                println!("Loading class at {}", path.display());
                Some(fs::read(path).unwrap())
            } else {
                None
            }
        })
        .unwrap_or_else(|| panic!("ClassNotFoundException: {}", name));
    let class_file = ClassFile::from_bytes((&data, 0))
        .expect("ClassFormatError")
        .1;

    assert!(class_file.magic == 0xCAFEBABE);
    assert!((45..62).contains(&class_file.major_version));

    let super_class = if class_file.super_class > 0 {
        let super_name = class_file.constant_pool.class_name(class_file.super_class);
        Some(ma.resolve_class(&super_name))
    } else {
        // Super class should only be None for Object
        None
    };
    let name = class_file.constant_pool.class_name(class_file.this_class);

    let mut interfaces = Vec::new();
    for interface in class_file.interfaces {
        let interface_name = class_file.constant_pool.class_name(interface);
        interfaces.push(ma.resolve_class(&interface_name));
    }

    // No new classes can be loaded after this point
    let class_id = ma.classes.next_id();

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
        let id = ma.methods.alloc(Method {
            defining_class: class_id,
            name,
            descriptor: MethodDescriptor::read(&descriptor),
            access_flags: method.access_flags,
            code,
        });
        methods.push(id);
    }

    let mut fields = Vec::new();
    for field in class_file.fields {
        let name = class_file.constant_pool.utf8(field.name_index);
        let descriptor = class_file.constant_pool.utf8(field.descriptor_index);
        let descriptor = FieldDescriptor::read(&descriptor);

        let static_value = if field.access_flags & fields::acc::STATIC != 0 {
            Some(Value::default_for_ty(&descriptor.0))
        } else {
            None
        };

        let id = ma.fields.alloc(Field {
            name,
            defining_class: class_id,
            access_flags: field.access_flags,
            static_value,
            descriptor,
        });
        fields.push(id);
    }

    let mut references = HashMap::new();
    for (i, entry) in class_file.constant_pool.table.iter().enumerate() {
        match entry {
            CPInfo::Fieldref { .. }
            | CPInfo::Methodref { .. }
            | CPInfo::InterfaceMethodref { .. }
            | CPInfo::Class { .. } => {
                references.insert(i as u16 + 1, Reference::Unresolved);
            }
            _ => {}
        }
    }

    let class = Class {
        initialized: false,
        defining_loader: ClassLoader::Bootstrap,
        references,
        class_obj: None,
        name: name.clone(),
        super_class,
        interfaces,
        methods,
        fields,
        access_flags: class_file.access_flags,
        constant_pool: class_file.constant_pool,
    };

    let id = ma.classes.alloc(class);
    ma.class_map.insert(name, id);

    // See if this class is an interface of itself
    let class = &ma.classes[id];
    for &interface in &class.interfaces {
        if interface == id {
            panic!("ClassCircularityError");
        }
    }

    id
}

pub fn load_arr_class_bootstrap(ma: &mut MethodArea, name: &str) -> ClassId {
    let name = name.to_string();
    let super_class = Some(ma.resolve_class("java/lang/Object"));
    let class = Class {
        initialized: false,
        defining_loader: ClassLoader::Bootstrap,
        references: HashMap::new(),
        class_obj: None,
        name: name.clone(),
        super_class,
        interfaces: vec![],
        methods: vec![], // FIXME: this should have clone
        fields: vec![],  // FIXME: this should have length
        access_flags: Default::default(),
        constant_pool: Default::default(),
    };

    let id = ma.classes.alloc(class);
    ma.class_map.insert(name.clone(), id);

    println!("Created array class: {}", name);

    id
}
