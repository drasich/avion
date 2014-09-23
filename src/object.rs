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
    //pub mesh : Option<Rc<RefCell<mesh::Mesh>>>,
    //pub mesh : Option<resource::ResourceRefGen<mesh::Mesh>>,
    pub mesh : resource::ResTT<mesh::Mesh>,
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
            mesh : resource::ResTT {name : String::from_str("model/skeletonmesh.mesh"), resource : resource::ResNone},
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

    pub fn mesh_set(&mut self, mesh : Rc<RefCell<mesh::Mesh>>)
    {
        println!("TODO この関数はなにもしてない");
        //self.mesh = Some(mesh);
        /*
        self.mesh = Some(resource::ResourceRefGen {
            name : mesh.borrow().name.clone(),
            resource : Some(mesh),
            resourcett : None
        });
        */
    }

    pub fn matrix_get(&self) -> matrix::Matrix4
    {
        let mt = matrix::Matrix4::translation(self.position);
        let mq = matrix::Matrix4::rotation(self.orientation);
        let ms = matrix::Matrix4::scale(self.scale);

        mt * mq * ms
    }

}


