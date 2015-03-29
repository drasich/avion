use vec;
use matrix;
use mesh_render;
use transform;
use shader;
use resource;
use material;

use std::collections::{LinkedList};
use std::sync::{RwLock, Arc};//,RWLockReadGuard};
use rustc_serialize::{json, Encodable, Encoder, Decoder, Decodable};
use std::collections::hash_map::Entry::{Occupied,Vacant};
use uuid;
use uuid::Uuid;
use core::marker::Sized;

pub trait Component
{
    fn copy(&self) -> Box<Component>;
    fn update(&self, &mut Object, dt : f64) {}
    //fn update(&self, dt : f64) {}
}


//#[derive(Decodable, Encodable)]
//#[derive(Encodable)]
//#[derive(Clone)]
pub struct Object
{
    pub name : String,
    //pub id : u32,
    pub id : uuid::Uuid,
    pub mesh_render : Option<mesh_render::MeshRender>,
    pub position : vec::Vec3,
    //pub orientation : vec::Quat,
    pub orientation : transform::Orientation,
    //pub angles : vec::Vec3,
    pub scale : vec::Vec3,
    pub children : LinkedList<Arc<RwLock<Object>>>,
    pub parent : Option<Arc<RwLock<Object>>>,
    //pub transform : Box<transform::Transform>
    pub components : Vec<Box<Component>>
}

unsafe impl Send for Object {}
unsafe impl Sync for Object {}

impl Clone for Object {

    fn clone(&self) -> Object {
        let mut components = Vec::new();
        for c in self.components.iter() {
            let cc = (*c).copy();
            components.push(cc);
        }
        Object {
            name : self.name.clone(),
            id : self.id.clone(),//?
            mesh_render : self.mesh_render.clone(),
            position : self.position.clone(),
            orientation : self.orientation.clone(),
            scale : self.scale.clone(),
            children : self.children.clone(), //LinkedList::new(),
            parent : self.parent.clone(), //None,
            //transform : box transform::Transform::new()
            components : components
        }
    }
}

impl Object
{
    /*
    pub fn new(name : &str) -> Object
    {
        Object {
            name : String::from_str(name),
            id : 0,
            mesh_render : None,
            position : vec::Vec3::zero(),
            orientation : vec::Quat::identity(),
            //angles : vec::Vec3::zero(),
            scale : vec::Vec3::one(),
            children : LinkedList::new(),
            parent : None
        }
    }
    */

    pub fn get_matrix(&self) -> matrix::Matrix4
    {
        //TODO optim
        let mt = matrix::Matrix4::translation(self.position);
        let mq = matrix::Matrix4::rotation(self.orientation.as_quat());
        let ms = matrix::Matrix4::scale(self.scale);

        &(&mt * &mq) * &ms
    }

    pub fn get_world_matrix(&self) -> matrix::Matrix4
    {
        //TODO optim
        let mt = matrix::Matrix4::translation(self.world_position());
        let mq = matrix::Matrix4::rotation(self.world_orientation());
        let ms = matrix::Matrix4::scale(self.world_scale());

        &(&mt * &mq) * &ms
    }


    /*
    pub fn child_add(&mut self, child : Arc<RwLock<Object>>)
    {
        self.children.push(child);
        child.write().parent = Some(self.clone());
    }
    */


    pub fn world_position(&self) -> vec::Vec3
    {
        match self.parent {
            None => return self.position,
            Some(ref parent) => {
                let wo = parent.read().unwrap().world_orientation();
                let p = wo.rotate_vec3(&self.position);
                return p + parent.read().unwrap().world_position();
            }
        }
    }

    pub fn world_orientation(&self) -> vec::Quat
    {
        match self.parent {
            None => return self.orientation.as_quat(),
            Some(ref p) => {
                return p.read().unwrap().world_orientation() * self.orientation.as_quat();
            }
        }
    }

    pub fn world_scale(&self) -> vec::Vec3
    {
        match self.parent {
            None => return self.scale,
            Some(ref p) => {
                return self.scale * p.read().unwrap().world_scale();
                //return self.scale.mul(p.read().unwrap().world_scale());
            }
        }
    }

    //TODO remove
    pub fn set_uniform_data(&self, name : &str, data : shader::UniformData)
    {
        let render = match self.mesh_render {
            Some(ref r) => r,
            None => return
        };

        match render.material.resource {
            resource::ResTest::ResData(ref d) => {
                d.write().unwrap().set_uniform_data(name, data);
            },
            _ => {}
        }
    }

    pub fn get_material(&self) -> Option<Arc<RwLock<material::Material>>>
    {
        let render = match self.mesh_render {
            Some(ref r) => r,
            None => return None
        };

        match render.material.resource {
            resource::ResTest::ResData(ref d) => {
                Some(d.clone())
            },
            _ => None
        }
    }

    pub fn update(&mut self, dt : f64)
    {
        let mut comps = self.components.iter();

        //comps.update(self, dt);
        /*
        for c in comps
        {
            c.update(self,dt);
        }
        */

        loop {
            match comps.next() { 
                None => break,
                Some(i) => {
                    //i.update(self,dt);
                }
            }
        }
    }


}

pub fn child_add(parent : Arc<RwLock<Object>>, child : Arc<RwLock<Object>>)
{
    parent.write().unwrap().children.push_back(child.clone());
    child.write().unwrap().parent = Some(parent.clone());
}


/*
impl PropertyShow for Object
{
    fn create_widget(window : &Window)
    {
        let c = unsafe { window_property_new(window) };
    }
}
*/

impl Decodable for Object {
  fn decode<D : Decoder>(decoder: &mut D) -> Result<Object, D::Error> {
    decoder.read_struct("root", 0, |decoder| {
         Ok(Object{
          name: try!(decoder.read_struct_field("name", 0, |decoder| Decodable::decode(decoder))),
          id: try!(decoder.read_struct_field("id", 0, |decoder| Decodable::decode(decoder))),
          mesh_render: try!(decoder.read_struct_field("mesh_render", 0, |decoder| Decodable::decode(decoder))),
          position: try!(decoder.read_struct_field("position", 0, |decoder| Decodable::decode(decoder))),
          orientation: try!(decoder.read_struct_field("orientation", 0, |decoder| Decodable::decode(decoder))),
          scale: try!(decoder.read_struct_field("scale", 0, |decoder| Decodable::decode(decoder))),
          children: try!(decoder.read_struct_field("children", 0, |decoder| Decodable::decode(decoder))),
          //parent: try!(decoder.read_struct_field("children", 0, |decoder| Decodable::decode(decoder))),
          parent: None,
          //transform : box transform::Transform::new()
          components : Vec::new()
        })
    })
  }
}


impl Encodable  for Object {
  fn encode<E : Encoder>(&self, encoder: &mut E) -> Result<(), E::Error> {
      encoder.emit_struct("Object", 1, |encoder| {
          try!(encoder.emit_struct_field( "name", 0usize, |encoder| self.name.encode(encoder)));
          try!(encoder.emit_struct_field( "id", 1usize, |encoder| self.id.encode(encoder)));
          try!(encoder.emit_struct_field( "mesh_render", 2usize, |encoder| self.mesh_render.encode(encoder)));
          try!(encoder.emit_struct_field( "position", 3usize, |encoder| self.position.encode(encoder)));
          try!(encoder.emit_struct_field( "orientation", 4usize, |encoder| self.orientation.encode(encoder)));
          try!(encoder.emit_struct_field( "scale", 5usize, |encoder| self.scale.encode(encoder)));
          try!(encoder.emit_struct_field( "children", 6usize, |encoder| self.children.encode(encoder)));
          //try!(encoder.emit_struct_field( "transform", 7u, |encoder| self.transform.encode(encoder)));
          //try!(encoder.emit_struct_field( "parent", 6u, |encoder| self.parent.encode(encoder)));
          Ok(())
      })
  }
}

/*
impl Clone for Object
{
    fn clone(&self) -> Object
    {
        Object {
            name : self.name.clone(),
            id : uuid.clone(),
            mesh_render : None,
            position : vec::Vec3::zero(),
            orientation : vec::Quat::identity(),
            //angles : vec::Vec3::zero(),
            scale : vec::Vec3::one(),
            children : LinkedList::new(),
            parent : None
        }

    }
}
*/

pub struct ObjectRef
{
    pub id : uuid::Uuid,
    pub object : Option<Arc<RwLock<Object>>>,
}

impl ObjectRef
{
    pub fn new_with_id(id : uuid::Uuid) -> ObjectRef
    {
        ObjectRef {
            id : id,
            object : None
        }
    }

    pub fn new_with_object(o : Arc<RwLock<Object>>) -> ObjectRef
    {
        let id = o.read().unwrap().id.clone();
        ObjectRef {
            id : id,
            object : Some(o)
        }
    }
}

impl Decodable for ObjectRef {
  fn decode<D : Decoder>(decoder: &mut D) -> Result<ObjectRef, D::Error> {
    decoder.read_struct("root", 0, |decoder| {
         Ok(ObjectRef{
          id: try!(decoder.read_struct_field("id", 0, |decoder| Decodable::decode(decoder))),
          object: None,
        })
    })
  }
}


impl Encodable  for ObjectRef {
  fn encode<E : Encoder>(&self, encoder: &mut E) -> Result<(), E::Error> {
      encoder.emit_struct("ObjectRef", 1, |encoder| {
          try!(encoder.emit_struct_field( "id", 0usize, |encoder| self.id.encode(encoder)));
          Ok(())
      })
  }
}

