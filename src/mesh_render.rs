use shader;
use mesh;
use resource;

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

    pub fn new_with_mesh(mesh : resource::ResTT<mesh::Mesh>, material : &str) -> MeshRender
    {
        MeshRender {
            mesh : mesh,
            material : resource::ResTT::new(material)
        }
    }
}


