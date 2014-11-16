use vec;
use matrix;
use mesh_render;

use std::collections::{DList};
use sync::{RWLock, Arc};//,RWLockReadGuard};
use serialize::{json, Encodable, Encoder, Decoder, Decodable};
use uuid;
use uuid::Uuid;

//#[deriving(Decodable, Encodable)]
//#[deriving(Encodable)]
pub struct Object
{
    pub name : String,
    //pub id : u32,
    pub id : uuid::Uuid,
    pub mesh_render : Option<mesh_render::MeshRender>,
    pub position : vec::Vec3,
    pub orientation : vec::Quat,
    //pub angles : vec::Vec3,
    pub scale : vec::Vec3,
    pub children : DList<Arc<RWLock<Object>>>,
    pub parent : Option<Arc<RWLock<Object>>>,
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
            children : DList::new(),
            parent : None
        }
    }
    */

    pub fn matrix_get(&self) -> matrix::Matrix4
    {
        let mt = matrix::Matrix4::translation(self.position);
        let mq = matrix::Matrix4::rotation(self.orientation);
        let ms = matrix::Matrix4::scale(self.scale);

        mt * mq * ms
    }

    /*
    pub fn child_add(&mut self, child : Arc<RWLock<Object>>)
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
                let wo = parent.read().world_orientation();
                let p = wo.rotate_vec3(&self.position);
                return p + parent.read().world_position();
            }
        }
    }

    pub fn world_orientation(&self) -> vec::Quat
    {
        match self.parent {
            None => return self.orientation,
            Some(ref p) => {
                return p.read().world_orientation() * self.orientation;
            }
        }
    }

    pub fn world_scale(&self) -> vec::Vec3
    {
        match self.parent {
            None => return self.scale,
            Some(ref p) => {
                return self.scale.mul(& p.read().world_scale());
            }
        }
    }
}

pub fn child_add(parent : Arc<RWLock<Object>>, child : Arc<RWLock<Object>>)
{
    parent.write().children.push(child.clone());
    child.write().parent = Some(parent.clone());
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

impl<S: Decoder<E>, E> Decodable<S, E> for Object {
  fn decode(decoder: &mut S) -> Result<Object, E> {
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
          parent: None
        })
    })
  }
}


impl <S: Encoder<E>, E> Encodable<S, E> for Object {
  fn encode(&self, encoder: &mut S) -> Result<(), E> {
      encoder.emit_struct("Object", 1, |encoder| {
          try!(encoder.emit_struct_field( "name", 0u, |encoder| self.name.encode(encoder)));
          try!(encoder.emit_struct_field( "id", 1u, |encoder| self.id.encode(encoder)));
          try!(encoder.emit_struct_field( "mesh_render", 2u, |encoder| self.mesh_render.encode(encoder)));
          try!(encoder.emit_struct_field( "position", 3u, |encoder| self.position.encode(encoder)));
          try!(encoder.emit_struct_field( "orientation", 4u, |encoder| self.orientation.encode(encoder)));
          try!(encoder.emit_struct_field( "scale", 5u, |encoder| self.scale.encode(encoder)));
          try!(encoder.emit_struct_field( "children", 6u, |encoder| self.children.encode(encoder)));
          //try!(encoder.emit_struct_field( "parent", 6u, |encoder| self.parent.encode(encoder)));
          Ok(())
      })
  }
}

