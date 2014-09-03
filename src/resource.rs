use mesh;
use shader;
use serialize::{json, Encodable, Encoder, Decoder, Decodable};
use std::rc::Rc;
use std::cell::RefCell;
//use std::collections::{DList,Deque};
use std::collections::HashMap;
use sync::{RWLock, Arc};


//#[deriving(Decodable, Encodable)]
pub enum Resource {
    Mesh(mesh::Mesh),
    //Shader(shader::Material)
}

pub struct ResourceS
{
    state : int,
    data : Resource
}

pub trait ResourceT  {
    fn init(&mut self);
}

pub struct ResourceRefGen<T>
{
    pub name : String,
    //pub resource : Option<T>
    pub resource : Option<Rc<RefCell<T>>>,
    pub resourcett : Option<Arc<RWLock<T>>>,
}

impl<T> ResourceRefGen<T>
{
    pub fn new(name : &str) -> ResourceRefGen<T>
    {
        ResourceRefGen { 
            name : String::from_str(name), 
                 resource : None,
                 resourcett : None }
    }
}

pub struct ResourceManager
{
    pub meshes : HashMap<String, Rc<RefCell<mesh::Mesh>>>,
    pub meshestt : HashMap<String, Arc<RWLock<mesh::Mesh>>>
}

impl ResourceManager {
    pub fn new() -> ResourceManager
    {
        ResourceManager {
            meshes : HashMap::new(),
            meshestt : HashMap::new(),
        }
    }

    pub fn get_or_create(&mut self, name : &str) -> Rc<RefCell<mesh::Mesh>>
    {
        match self.meshes.find(&String::from_str(name)) {
            Some(mesh) => return mesh.clone(),
            None => return Rc::new(RefCell::new(mesh::Mesh::new_from_file(name)))
        }
    }

    pub fn get_or_creatett(&mut self, name : &str) -> Arc<RWLock<mesh::Mesh>>
    {
        match self.meshestt.find(&String::from_str(name)) {
            Some(mesh) => return mesh.clone(),
            None => return Arc::new(RWLock::new(mesh::Mesh::new_from_file(name)))
        }

    }

}


//#[deriving(Decodable, Encodable)]
pub struct ResourceRef
{
    pub name : String,
    pub resource : Resource
}

/*
impl <S: Encoder<E>, E> Encodable<S, E> for ResourceRefGen<mesh::Mesh> {
  fn encode(&self, encoder: &mut S) -> Result<(), E> {
      encoder.emit_struct("Mesh", 1, |encoder| {
          try!(encoder.emit_struct_field( "name", 0u, |encoder| self.name.encode(encoder)));
          Ok(())
      })
  }
}
*/

impl <S: Encoder<E>, E, T> Encodable<S, E> for ResourceRefGen<T> {
  fn encode(&self, encoder: &mut S) -> Result<(), E> {
      encoder.emit_struct("Mesh", 1, |encoder| {
          try!(encoder.emit_struct_field( "name", 0u, |encoder| self.name.encode(encoder)));
          Ok(())
      })
  }
}

/*
impl<S: Decoder<E>, E> Decodable<S, E> for ResourceRefGen<mesh::Mesh> {
  fn decode(decoder: &mut S) -> Result<Mesh, E> {
    decoder.read_struct("root", 0, |decoder| {
         Ok(
             ResourceRefGen<mesh::Mesh>
            {
                name : try!(decoder.read_struct_field("name", 0, |decoder| Decodable::decode(decoder))),
                resource : None
            }
           )
    })
  }
}
*/

impl<S: Decoder<E>, E, T> Decodable<S, E> for ResourceRefGen<T> {
  fn decode(decoder: &mut S) -> Result<ResourceRefGen<T>, E> {
    decoder.read_struct("root", 0, |decoder| {
         Ok(
             ResourceRefGen{
                 name : try!(decoder.read_struct_field("name", 0, |decoder| Decodable::decode(decoder))),
                 resource : None,
                 resourcett : None
            }
           )
    })
  }
}

