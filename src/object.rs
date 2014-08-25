use mesh;
use std::rc::Rc;
use std::cell::RefCell;
use vec;

pub struct Object
{
    pub name : String,
    pub mesh : Option<Rc<RefCell<mesh::Mesh>>>,
    pub position : vec::Vec3,
    pub orientation : vec::Quat
}

impl Object
{
    pub fn new(name : &str) -> Object
    {
        Object {
            name : String::from_str(name),
            mesh : None,
            position : vec::Vec3::zero(),
            orientation : vec::Quat::identity()
        }
    }


    pub fn empty() -> Object
    {
        Object::new("empty")
    }

    pub fn mesh_set(&mut self, mesh : Rc<RefCell<mesh::Mesh>>)
    {
        self.mesh = Some(mesh);
    }

}

