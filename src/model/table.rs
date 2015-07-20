use factory;
use object;
use mesh;
//use mesh_render;
use resource;
use std::sync::{Arc,RwLock};

pub struct Table
{
    height : f64,
    width : f64,
    length : f64,
}

pub fn create_table_object(factory : &mut factory::Factory, table : Table) -> object::Object
{
    let o = factory.create_object("table");
    /*
    let m = create_table_mesh(table);
    o.mesh_render = Some(create_mesh_render(m));
    */
    o
}

pub fn create_table_mesh(table : Table) -> mesh::Mesh
{
    let mut m = mesh::Mesh::new();
    create_table(&mut m);
    m
}

/*
pub fn create_mesh_render(mesh : mesh::Mesh) -> mesh_render::MeshRender
{
    let m = Arc::new(RwLock::new(mesh));
    let rs = resource::ResTest::ResData(m);
    let mr = resource::ResTT::new_with_res("table", rs);

    let mesh_render =
        mesh_render::MeshRender::new_with_mesh(mr, "material/line.mat");

    mesh_render
}
*/

fn create_table(m : &mut mesh::Mesh)
{
}


