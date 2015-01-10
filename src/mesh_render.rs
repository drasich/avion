use shader;
use mesh;
use resource;
use material;

#[derive(RustcDecodable, RustcEncodable)]
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

    /*
    pub fn new_direct(
        mat_manager : &mut resource::ResourceManager<material::Material>,
        mesh : &str, material : &str) -> MeshRender
    {
        let mut mr =  MeshRender {
            mesh : resource::ResTT::new(mesh),
            material : resource::ResTT::new(material)
        };

        mr.material.resource = resource::ResTest::ResData(mat_manager.request_use_no_proc(material.as_slice()));

        mr
    }
    */

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

    pub fn new_with_mat(
        mesh : &str, 
        material : resource::ResTT<material::Material>) -> MeshRender
    {
        MeshRender {
            mesh : resource::ResTT::new(mesh),
            material : material
        }
    }

}


