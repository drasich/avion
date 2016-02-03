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
use resource;
use material;

use property::{PropertyRead, PropertyGet, PropertyWrite, WriteValue};
use std::any::Any;

#[derive(RustcDecodable, RustcEncodable, Clone)]
pub struct MeshRender
{
    pub mesh : String,
    pub material : String,
}

impl MeshRender
{
    pub fn new(mesh : &str, material : &str) -> MeshRender
    {
        MeshRender {
            mesh : mesh.to_owned(),
            material : material.to_owned(),
        }
    }
}

#[derive(Clone)]
pub struct MeshRenderer
{
    pub mesh : Arc<RwLock<mesh::Mesh>>,
    pub material : Arc<RwLock<material::Material>>,

    //pub mesh_instance : Option<Rc<RefCell<mesh::Mesh>>>,
    //pub mesh_instance : Option<Arc<mesh::Mesh>>,
    pub mesh_instance : Option<Box<mesh::Mesh>>,
    //pub mesh_instance : Option<Rc<mesh::Mesh>>,
    pub material_instance : Option<Box<material::Material>>,
}

impl Component for MeshRenderer
{
    /*
    fn copy(&self) -> Rc<RefCell<Box<Component>>>
    {
        Rc::new(RefCell::new(
                box MeshRenderer
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
    */

    fn update(&mut self, ob : &mut Object, dt : f64)
    {
        println!("update mesh render");
    }

    fn get_name(&self) -> String {
        "mesh_render".to_owned()
    }
}

impl MeshRenderer{
    fn create_mesh_instance(&mut self) 
    {
        self.mesh_instance = Some(box self.mesh.read().unwrap().clone())
    }

    //TODO
    //pub fn get_or_create_mesh_instance<'a>(&'a mut self) -> &'a mut Box<mesh::Mesh>
    //pub fn get_or_create_mesh_instance<'a>(&'a mut self) -> &'a mut mesh::Mesh
    pub fn get_or_create_mesh_instance(& mut self) -> & mut mesh::Mesh
    {
        if self.mesh_instance.is_none() {
            self.mesh_instance = Some(box self.mesh.read().unwrap().clone())
        }

        //let yo = self.mesh_instance.unwrap();

        match self.mesh_instance {
            Some(ref mut mi) => &mut *mi,
            None => panic!("impossible")
        }
    }

    pub fn get_mesh(&self) -> Arc<RwLock<mesh::Mesh>>
    {
        self.mesh.clone()
    }

    pub fn new(ob : &Object, resource : &resource::ResourceGroup) -> MeshRenderer
    {
        let mesh_render = {
            match ob.get_comp_data::<MeshRender>(){
                Some(m) => m.clone(),
                None => panic!("no mesh data")
            }
        };

        MeshRenderer {
            mesh : resource.mesh_manager.borrow_mut().request_use_no_proc(mesh_render.mesh.as_ref()),
            material : resource.material_manager.borrow_mut().request_use_no_proc(mesh_render.material.as_ref()),
            mesh_instance : None,
            material_instance : None,
        }
    }

    pub fn with_names(mesh : &str, material : &str, resource : &resource::ResourceGroup) -> MeshRenderer
    {
        MeshRenderer {
            mesh : resource.mesh_manager.borrow_mut().request_use_no_proc(mesh),
            material : resource.material_manager.borrow_mut().request_use_no_proc(material),
            mesh_instance : None,
            material_instance : None,
        }
    }

    pub fn with_mesh_render(mesh_render : &MeshRender, resource : &resource::ResourceGroup) -> MeshRenderer
    {
        MeshRenderer {
            mesh : resource.mesh_manager.borrow_mut().request_use_no_proc(mesh_render.mesh.as_ref()),
            material : resource.material_manager.borrow_mut().request_use_no_proc(mesh_render.material.as_ref()),
            mesh_instance : None,
            material_instance : None,
        }
    }

    pub fn new_with_mesh(
        mesh : Arc<RwLock<mesh::Mesh>>,
        material : &str,
        resource : &resource::ResourceGroup) -> MeshRenderer
    {
        MeshRenderer {
            mesh : mesh,
            material : resource.material_manager.borrow_mut().request_use_no_proc(material.as_ref()),
            mesh_instance : None,
            material_instance : None,
        }
    }

    pub fn new_with_mat(
        mesh : &str, 
        material : Arc<RwLock<material::Material>>,
        resource : &resource::ResourceGroup) -> MeshRenderer
    {
        MeshRenderer {
            mesh : resource.mesh_manager.borrow_mut().request_use_no_proc(mesh.as_ref()),
            material : material,
            mesh_instance : None,
            material_instance : None,
        }
    }


    pub fn new_with_mesh_and_mat(
        mesh : Arc<RwLock<mesh::Mesh>>,
        material : Arc<RwLock<material::Material>>) -> MeshRenderer
    {
        MeshRenderer {
            mesh : mesh,
            material : material,
            mesh_instance : None,
            material_instance : None,
        }
    }

    pub fn get_mesh_mat_instance(&self) -> 
        Option<(&material::Material, &mesh::Mesh)>
        //Option<&'a mesh::Mesh>
        {
            let me =
            match self.mesh_instance {
                Some(ref m) => &*m,
                None => return None
            };

            let ma =
            match self.material_instance {
                Some(ref m) => &*m,
                None => return None
            };

            Some((ma,me))

            /*
            match (self.material_instance,self.mesh_instance) {
                (Some(ref mat),  Some(ref mesh)) => Some((&*mat, &*mesh)),
                (_,_) => None
            }
            */


            /*
            let mat = self.material.read();
            let mesh = self.mesh.read();
            (&*mat.unwrap(),
            &*mesh.unwrap())
            */
        }

    pub fn get_mat_instance(&mut self) -> Option<&mut material::Material>
    {
        match self.material_instance {
            Some(ref mut m) => Some(&mut *m),
            None => return None
        }
    }

    pub fn get_mesh_instance(&mut self) -> Option<&mut mesh::Mesh>
    {
        match self.mesh_instance {
            Some(ref mut m) => Some(&mut *m),
            None => return None
        }
    }

}

pub fn new(ob : &Object, resource : &resource::ResourceGroup) -> Box<Component>
{
    box MeshRenderer::new(ob, resource)
}

property_set_impl!(MeshRender,[mesh,material]);
property_get_impl!(MeshRender,[mesh,material]);

