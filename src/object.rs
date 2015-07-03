use vec;
use matrix;
use transform;
use shader;
use resource;
use material;
use component;
use component::{Component,CompData, Components};
use component::mesh_render;
use mesh;

use std::collections::{LinkedList};
use std::sync::{RwLock, Arc};//,RWLockReadGuard};
use rustc_serialize::{json, Encodable, Encoder, Decoder, Decodable};
use std::collections::hash_map::Entry::{Occupied,Vacant};
use uuid;
use uuid::Uuid;
use core::marker::Sized;
use std::rc::Rc;
use std::cell::RefCell;
use std::any::Any;

use std::sync::mpsc::channel;

pub struct ThreadObject(Arc<RwLock<Object>>);

//#[derive(Decodable, Encodable)]
//#[derive(Encodable)]
//#[derive(Clone)]
pub struct Object
{
    pub name : String,
    pub id : uuid::Uuid,
    pub mesh_render : Option<mesh_render::MeshRenderer>,
    pub position : vec::Vec3,
    pub orientation : transform::Orientation,
    pub scale : vec::Vec3,
    pub children : LinkedList<Arc<RwLock<Object>>>,
    //pub children : LinkedList<ThreadObject>,
    pub parent : Option<Arc<RwLock<Object>>>,
    //pub transform : Box<transform::Transform>
    pub components : Vec<Box<Components>>,
    pub comp_data : Vec<Box<CompData>>,
    pub comp_string : Vec<String>,
}

//Real object that the user use (like object state)
// TODO rename to object
pub struct ObjectRom
{
    pub name : String,
    pub id : uuid::Uuid,
    //pub mesh_render : Option<mesh_render::MeshRender>,
    pub mesh_render : Option<(String, String)>,
    pub position : vec::Vec3,
    pub orientation : transform::Orientation,
    pub scale : vec::Vec3,
    pub children : Vec<ObjectRom>,
    pub parent : Option<uuid::Uuid>,
    //pub transform : Box<transform::Transform>
    pub components : Vec<String>,
    pub comp_data : Vec<Box<CompData>>,
    
    pub instance : Option<ObjectInstance>
}

// only used by engine, not editable by yser, not encodable
pub struct ObjectInstance
{
    pub mesh_render : Option<mesh_render::MeshRenderer>,
    //pub children : LinkedList<Arc<RwLock<ObjectInstance>>>,
    pub parent : Option<Arc<RwLock<ObjectRom>>>,
    pub components : Rc<RefCell<Vec<Rc<RefCell<Box<Component>>>>>>,
}


unsafe impl Send for Object {}
unsafe impl Sync for Object {}

impl Clone for Object {

    fn clone(&self) -> Object {
        let mut components = Vec::new();
        for c in self.components.iter() {
            let cc = c.clone();
            components.push(cc);
        }
        let comp_data = self.comp_data.clone();
        Object {
            name : self.name.clone(),
            id : self.id.clone(),//?
            mesh_render : if let Some(ref mr) = self.mesh_render {
                Some(mr.clone())
            } else {None},
            position : self.position.clone(),
            orientation : self.orientation.clone(),
            scale : self.scale.clone(),
            children : self.children.clone(), //LinkedList::new(),
            parent : self.parent.clone(), //None,
            //transform : box transform::Transform::new()
            components : components,
            comp_data : comp_data,
            comp_string : self.comp_string.clone()
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

        render.material.write().unwrap().set_uniform_data(name, data);
    }

    pub fn get_material(&self) -> Option<Arc<RwLock<material::Material>>>
    {
        let render = match self.mesh_render {
            Some(ref r) => r,
            None => return None
        };

        Some(render.material.clone())
    }

    pub fn update(&mut self, dt : f64)
    {
        let len = self.components.len();

        let mut index = 0;
        loop {
            if index >= self.components.len() {
                break;
            }

            self.components.push(box Components::Empty);
            let mut c = self.components.swap_remove(index);
            c.update(self, dt);
            self.components[index] = c;
            index = index +1;
        }
    }

    pub fn add_component(&mut self, c : Box<Components>)
    {
        self.components.push(c);
    }

    pub fn add_comp_string(&mut self, c : &str) 
    {
        self.comp_string.push(c.to_string());

    }

    pub fn add_comp_data(&mut self, c : Box<CompData>)
    {
        self.comp_data.push(c);
        //let (tx, rx) = channel();
    }

    pub fn get_comp_data<T:Any>(&self) -> Option<&T>
    {
        for c in self.comp_data.iter()
        {
             if let Some(s) = c.get_comp::<T>() {
                 return Some(s);
             }
             //if cd.unwrap().is::<T>() {
             //}
        }
        None
    }

    pub fn get_mut_comp_data<T:Any>(&mut self) -> Option<&mut T>
    {
        for c in self.comp_data.iter_mut()
        {
             if let Some(s) = c.get_mut_comp::<T>() {
                 return Some(s);
             }
             //if cd.unwrap().is::<T>() {
             //}
        }
        None
    }

    pub fn init_components(&mut self, comp_mgr : &component::Manager, resource : &resource::ResourceGroup)
    {
        let mut comps = Vec::new();

        for c in self.comp_string.iter() {
            let f = comp_mgr.get_component_create_fn(c.as_ref()).unwrap();
            let pc = f(self, resource);
            comps.push(pc);
        }

        self.components = comps;

        for child in self.children.iter()
        {
            child.write().unwrap().init_components(comp_mgr, resource);
        }
    }

    pub fn get_component<T:Any>(& self) -> Option<& T>
    {
        for c in self.components.iter()
        {
            if let Some(s) = c.get_comp::<T>() {
                return Some(s);
            }

        }
        None
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
          mesh_render: None, //try!(decoder.read_struct_field("mesh_render", 0, |decoder| Decodable::decode(decoder))),
          position: try!(decoder.read_struct_field("position", 0, |decoder| Decodable::decode(decoder))),
          orientation: try!(decoder.read_struct_field("orientation", 0, |decoder| Decodable::decode(decoder))),
          scale: try!(decoder.read_struct_field("scale", 0, |decoder| Decodable::decode(decoder))),
          children: try!(decoder.read_struct_field("children", 0, |decoder| Decodable::decode(decoder))),
          //children : LinkedList::new(),
          //parent: try!(decoder.read_struct_field("children", 0, |decoder| Decodable::decode(decoder))),
          parent: None,
          //transform : box transform::Transform::new()
          components : Vec::new(),
          //comp_data : Rc::new(RefCell::new(Vec::new()))
          //components: try!(decoder.read_struct_field("components", 0, |decoder| Decodable::decode(decoder))),
          comp_data: try!(decoder.read_struct_field("comp_data", 0, |decoder| Decodable::decode(decoder))),
          comp_string: try!(decoder.read_struct_field("components", 0, |decoder| Decodable::decode(decoder))),
        })
    })
  }
}


impl Encodable  for Object {
  fn encode<E : Encoder>(&self, encoder: &mut E) -> Result<(), E::Error> {
      encoder.emit_struct("Object", 1, |encoder| {
          try!(encoder.emit_struct_field( "name", 0usize, |encoder| self.name.encode(encoder)));
          try!(encoder.emit_struct_field( "id", 1usize, |encoder| self.id.encode(encoder)));
          //try!(encoder.emit_struct_field( "mesh_render", 2usize, |encoder| self.mesh_render.encode(encoder)));
          try!(encoder.emit_struct_field( "position", 3usize, |encoder| self.position.encode(encoder)));
          try!(encoder.emit_struct_field( "orientation", 4usize, |encoder| self.orientation.encode(encoder)));
          try!(encoder.emit_struct_field( "scale", 5usize, |encoder| self.scale.encode(encoder)));
          try!(encoder.emit_struct_field( "children", 6usize, |encoder| self.children.encode(encoder)));
          //try!(encoder.emit_struct_field( "transform", 7u, |encoder| self.transform.encode(encoder)));
          //try!(encoder.emit_struct_field( "parent", 6u, |encoder| self.parent.encode(encoder)));
          //try!(encoder.emit_struct_field( "components", 7usize, |encoder| self.components.encode(encoder)));
          try!(encoder.emit_struct_field( "components", 7usize, |encoder| self.comp_string.encode(encoder)));
          try!(encoder.emit_struct_field( "comp_data", 8usize, |encoder| self.comp_data.encode(encoder)));
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

