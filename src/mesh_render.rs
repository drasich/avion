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
    //pub material : Rc<RefCell<shader::Material>>
    pub material : resource::ResTT<shader::Material>
}

impl MeshRender
{
    pub fn new(mesh : &str, material : &str) -> MeshRender
    {
        MeshRender {
            mesh : resource::ResTT::new(mesh),
            //material : shader::Material::new_from_file(material)
            material : resource::ResTT::new(material)
        }
    }
}


