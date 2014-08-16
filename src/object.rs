use mesh;
use std::rc::Rc;
use std::cell::RefCell;

pub struct Object
{
    pub name : String,
    pub mesh : Option<Rc<RefCell<mesh::Mesh>>>
}

