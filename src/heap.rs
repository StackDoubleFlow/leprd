use id_arena::Id;

use crate::value::Value;

pub type ObjectId = Id<Object>;
pub type ArrayId = Id<Array>;

pub struct Object {
    // TODO
}

pub struct Array {
    contents: Box<[Value]>,
}
