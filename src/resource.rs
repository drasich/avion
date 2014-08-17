use mesh;
use shader;

pub enum Resource {
    Mesh(mesh::Mesh),
    Shader(shader::Shader)
}

pub struct ResourceS
{
    state : int,
    data : Resource
}

pub trait ResourceT  {
    fn init(&mut self);
}

