//use component;
use std::rc::Rc;
use std::cell::RefCell;
use rustc_serialize::{json, Encodable, Encoder, Decoder, Decodable};
use std::sync::{RwLock, Arc};

use component::{Component, CompData};
use component::manager::Encode;
//use object::ComponentFunc;
use object::Object;
use transform;
use mesh;
use mesh_render;
use resource;
use material;

pub struct MeshRender
{
    mesh : Arc<RwLock<mesh::Mesh>>,
    material : Arc<RwLock<material::Material>>,

    mesh_instance : Option<Rc<RefCell<mesh::Mesh>>>,
    material_instance : Option<material::Material>,
}

impl Component for MeshRender
{
    fn copy(&self) -> Rc<RefCell<Box<Component>>>
    {
        Rc::new(RefCell::new(
                box MeshRender
                {
                    mesh : self.mesh.clone(),
                    material : self.material.clone(),
                    mesh_instance : None,
                        //match self.mesh_instance {
                        //None => None,
                        //Some(m) => Some(m.clone())
                    //},
                    material_instance : None,
                        //match self.material_instance {
                        //None => None,
                        //Some(m) => Some(m.clone())
                    //},
                }))
    }

    fn update(&mut self, ob : &mut Object, dt : f64)
    {
        println!("update mesh render");
    }

    fn get_name(&self) -> String {
        "mesh_render".to_string()
    }
}

impl MeshRender {
    fn create_mesh_instance(&mut self)
    {
        //self.mesh_instance = 
    }
}

pub fn new(ob : &Object, resource : &resource::ResourceGroup) -> Box<Component>
{
    println!("mesh new---->>>>");
    let mesh_render = {
        match ob.get_comp_data::<mesh_render::MeshRenderData>(){
            Some(m) => m.clone(),
            None => panic!("no armature data")
        }
    };

    let mr = MeshRender {
        mesh : resource.mesh_manager.borrow_mut().request_use_no_proc(mesh_render.mesh.as_ref()),
        material : resource.material_manager.borrow_mut().request_use_no_proc(mesh_render.material.as_ref()),
        mesh_instance : None,
        material_instance : None,
    };

    box mr
}

