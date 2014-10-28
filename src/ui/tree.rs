use sync::{RWLock, Arc};
use std::collections::HashMap;

use object;

#[repr(C)]
pub struct Elm_Object_Item;

pub struct Tree
{
    pub name : String,
    //TODO
    //objects : HashMap<Arc<RWLock<object::Object>>, *const Elm_Object_Item >
    objects : HashMap<String, *const Elm_Object_Item >
}

impl Tree
{
    pub fn new() -> Tree
    {
        Tree {
            name : String::from_str("caca"),
            objects : HashMap::new()
        }
    }
}
