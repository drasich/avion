use mesh;
use std::rc::Rc;
use std::cell::RefCell;
use vec;
use matrix;

pub struct Object
{
    pub name : String,
    pub mesh : Option<Rc<RefCell<mesh::Mesh>>>,
    pub position : vec::Vec3,
    pub orientation : vec::Quat,
    pub scale : vec::Vec3
}

impl Object
{
    pub fn new(name : &str) -> Object
    {
        Object {
            name : String::from_str(name),
            mesh : None,
            position : vec::Vec3::zero(),
            orientation : vec::Quat::identity(),
            scale : vec::Vec3::one()
        }
    }

    pub fn newRef(name : &str) -> Rc<RefCell<Object>>
    {
        Rc::new(RefCell::new(Object::new(name)))
    }


    pub fn empty() -> Object
    {
        Object::new("empty")
    }

    pub fn mesh_set(&mut self, mesh : Rc<RefCell<mesh::Mesh>>)
    {
        self.mesh = Some(mesh);
    }

    pub fn matrix_get(&self) -> matrix::Matrix4
    {
        let mt = matrix::Matrix4::translation(self.position);
        let mq = matrix::Matrix4::rotation(self.orientation);
        let ms = matrix::Matrix4::scale(self.scale);

        mt * mq * ms
    }

}


