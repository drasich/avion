use shader;
use mesh;
use resource;

use std::rc::Rc;
use std::cell::RefCell;

#[deriving(Decodable, Encodable)]
pub struct MeshRender
{
    pub mesh : resource::ResTT<mesh::Mesh>,
    //pub material : shader::Material
    pub material : Rc<RefCell<shader::Material>>
}

/*
impl MeshRender
{
    pub fn new(mesh : &str, material : &str) -> MeshRender
    {
        MeshRender {
            mesh : mesh::
            material : ,
        }
    }
}
*/


