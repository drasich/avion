use object;
use std::collections::{DList,Deque};
use std::rc::Rc;
use std::cell::RefCell;


#[deriving(Decodable, Encodable)]
pub struct Scene
{
    pub name : String,
    pub objects : DList<Rc<RefCell<object::Object>>>
}

impl Scene
{
    pub fn new(name : &str) -> Scene
    {
        Scene {
            name : String::from_str(name),
            objects : DList::new()
        }
    }
}
