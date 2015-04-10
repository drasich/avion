use std::default::Default;
use std::collections::{LinkedList};
use std::sync::{RwLock, Arc};
use std::rc::Rc;
use std::cell::RefCell;
use uuid;

use object;
use camera;
use scene;
use vec;
use transform;
use mesh;
use resource;
use mesh_render;

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
    fn create_id(&mut self) -> uuid::Uuid
    {
        return uuid::Uuid::new_v4();
        //self.id = self.id + 1;
        //println!("...........create id : {}", self.id);
        //return self.id;
    }

    pub fn create_object(&mut self, name : &str) -> object::Object
    {
        object::Object {
            name : String::from_str(name),
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
            components : Rc::new(RefCell::new(Vec::new()))
        }
    }
    
    pub fn create_camera(&mut self) -> camera::Camera
    {
        let c = camera::Camera {
            data : Default::default(),
            object : Arc::new(RwLock::new(self.create_object("camera"))),
            id : self.create_id(),
            object_id : None
        };

        c.object.write().unwrap().position = vec::Vec3::new(0.1f64, 0f64, 0f64);

        c
    }

    pub fn create_scene(&mut self, name : &str) -> scene::Scene
    {
        scene::Scene {
            name : String::from_str(name),
            id : self.create_id(),
            objects : LinkedList::new(),
            camera : Some(Rc::new(RefCell::new(self.create_camera())))
        }
    }
}

