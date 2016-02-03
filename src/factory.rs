use std::default::Default;
use std::collections::{LinkedList};
use std::sync::{RwLock, Arc};
use std::rc::Rc;
use std::cell::{Cell, RefCell};
use uuid;

use object;
use camera;
use scene;
use vec;
use transform;
use mesh;
use resource;

//#[derive(RustcDecodable, RustcEncodable)]
pub struct Factory
{
     id : u32,
     pub mesh_manager : Arc<RwLock<resource::ResourceManager<mesh::Mesh>>>,
}

impl Factory {

    pub fn new() -> Factory
    {
        Factory {
            id: 0,
            mesh_manager : Arc::new(RwLock::new(resource::ResourceManager::new())),
        }
    }

    //fn create_id(&mut self) -> u32
    fn create_id(&self) -> uuid::Uuid
    {
        return uuid::Uuid::new_v4();
        //self.id = self.id + 1;
        //println!("...........create id : {}", self.id);
        //return self.id;
    }

    pub fn create_object(&self, name : &str) -> object::Object
    {
        object::Object {
            name : String::from(name),
            id : self.create_id(),
            mesh_render : None,
            position : vec::Vec3::zero(),
            //orientation : vec::Quat::identity(),
            orientation : transform::Orientation::new_quat(),
            //angles : vec::Vec3::zero(),
            scale : vec::Vec3::one(),
            children : LinkedList::new(),
            parent : None,
            //transform : box transform::Transform::new()
            components : Vec::new(),
            comp_data : Vec::new(),
            comp_string : Vec::new(),
            comp_lua : Vec::new(),
        }
    }

    pub fn create_camera(&self) -> camera::Camera
    {
        let c = camera::Camera {
            data : Default::default(),
            object : Arc::new(RwLock::new(self.create_object("camera"))),
            id : self.create_id(),
            object_id : None
        };

        //c.object.write().unwrap().position = vec::Vec3::new(0.1f64, 0f64, 0f64);

        c
    }

    pub fn create_scene(&self, name : &str) -> scene::Scene
    {
        scene::Scene {
            name : String::from(name),
            id : self.create_id(),
            objects : LinkedList::new(),
            camera : Some(Rc::new(RefCell::new(self.create_camera())))
        }
    }

    pub fn copy_object(&self, o : &object::Object) -> object::Object
    {
        let mut children_copy = LinkedList::new();
        for c in &o.children {
            children_copy.push_back(Arc::new(RwLock::new(self.copy_object(&*c.read().unwrap()))));
        }

        //TODO clone children, components, comp_data, comp_string...
        object::Object {
            name : o.name.clone(),
            id : self.create_id(),
            mesh_render : o.mesh_render.clone(),
            position : o.position,
            //orientation : vec::Quat::identity(),
            orientation : o.orientation.clone(),
            //angles : vec::Vec3::zero(),
            scale : o.scale.clone(),
            children : children_copy,
            parent : o.parent.clone(),
            //transform : box transform::Transform::new()
            components : o.components.clone(),
            comp_data : o.comp_data.clone(),
            comp_string : o.comp_string.clone(),
            comp_lua : o.comp_string.clone(),
        }
    }

}

