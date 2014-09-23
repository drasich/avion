use std::rc::Rc;
use std::cell::RefCell;
use serialize::json;

use vec;
use matrix;
use mesh;
use mesh_render;
use resource;

#[deriving(Decodable, Encodable)]
pub struct Object
{
    pub name : String,
    pub mesh_render : Option<mesh_render::MeshRender>,
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
            mesh_render : None,
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

    pub fn matrix_get(&self) -> matrix::Matrix4
    {
        let mt = matrix::Matrix4::translation(self.position);
        let mq = matrix::Matrix4::rotation(self.orientation);
        let ms = matrix::Matrix4::scale(self.scale);

        mt * mq * ms
    }

}


