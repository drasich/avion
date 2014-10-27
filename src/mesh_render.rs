use shader;
use mesh;
use resource;
use material;

#[deriving(Decodable, Encodable)]
pub struct MeshRender
{
    pub mesh : resource::ResTT<mesh::Mesh>,
    pub material : resource::ResTT<material::Material>
}

impl MeshRender
{
    pub fn new(mesh : &str, material : &str) -> MeshRender
    {
        MeshRender {
            mesh : resource::ResTT::new(mesh),
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

    pub fn new_with_mesh_and_mat(mesh : resource::ResTT<mesh::Mesh>, material : resource::ResTT<material::Material>) -> MeshRender
    {
        MeshRender {
            mesh : mesh,
            material : material
        }
    }
}


