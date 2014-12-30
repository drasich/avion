use std::default::Default;
use std::collections::{DList};
use std::sync::{RWLock, Arc};
use uuid;

use object;
use camera;
use scene;
use vec;
use transform;

#[deriving(RustcDecodable, RustcEncodable)]
pub struct Factory
{
     id : u32
}

impl Factory {

    pub fn new() -> Factory
    {
        Factory {
            id: 0,
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
            children : DList::new(),
            parent : None,
            //transform : box transform::Transform::new()
        }
    }

    pub fn create_camera(&mut self) -> camera::Camera
    {
        let c = camera::Camera {
            data : Default::default(),
            object : Arc::new(RWLock::new(self.create_object("camera")))
        };

        c.object.write().position = vec::Vec3::new(0.1f64, 0f64, 0f64);

        c
    }

    pub fn create_scene(&mut self, name : &str) -> scene::Scene
    {
        scene::Scene {
            name : String::from_str(name),
            id : self.create_id(),
            objects : DList::new(),
        }
    }
}

