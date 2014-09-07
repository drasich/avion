use mesh;
use shader;
use serialize::{json, Encodable, Encoder, Decoder, Decodable};
use std::rc::Rc;
use std::cell::RefCell;
//use std::collections::{DList,Deque};
use std::collections::HashMap;
use sync::{RWLock, Arc};
use std::io::timer::sleep;
//use std::time::duration::Duration;


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

pub enum ResTest<T>
{
    ResData(Arc<RWLock<T>>),
    ResWait(Receiver<Arc<RWLock<T>>>),
    ResNone
}

pub struct ResTT<T>
{
    pub name : String,
    pub resource : ResTest<T>
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
        let v = self.meshestt.find_or_insert_with(String::from_str(name), 
                |key | Arc::new(RWLock::new(mesh::Mesh::new_from_file(name))));

        return v.clone();

        /*
        match self.meshestt.find(&String::from_str(name)) {
            Some(mesh) => return mesh.clone(),
            None => {
                return Arc::new(RWLock::new(mesh::Mesh::new_from_file(name)))
            }
        }
        */
    }

    //pub fn request_use(&mut self, name : &str) -> Receiver<Arc<RWLock<mesh::Mesh>>>
    pub fn request_use(&mut self, name : &str) -> ResTest<mesh::Mesh>
    {
        let v = self.meshestt.find_or_insert_with(String::from_str(name), 
                |key | Arc::new(RWLock::new(mesh::Mesh::new_from_file(name))));

        if v.read().state == 0 {
            v.write().state = 1;
            let (tx, rx) = channel::<Arc<RWLock<mesh::Mesh>>>();
            let vc = v.clone();

            spawn( proc() {
                sleep(::std::time::duration::Duration::seconds(5));
                let mut vv = vc.write();
                vv.init();
                tx.send(vc);
            });

            return ResWait(rx);
        }
        else {
        //    return Some(v.clone());
        }

        return ResNone;


        /*
        let (tx, rx) = channel::<Arc<RWLock<mesh::Mesh>>>();
            let vc = v.clone();

        spawn( proc() {
            sleep(::std::time::duration::Duration::seconds(5));
            let mut vv = vc.write();
            vv.init();
            tx.send(vc);
        });

        return rx;
        */

        /*
        loop {
               match rx.try_recv() {
                   Err(e) => {},//println!("nothing"),
                   Ok(val) =>  { println!("received val {} ", val); }
               }
               println!("yo man");
        }
        */
    }

}


//#[deriving(Decodable, Encodable)]
pub struct ResourceRef
{
    pub name : String,
    pub resource : Resource
}

impl <S: Encoder<E>, E, T> Encodable<S, E> for ResourceRefGen<T> {
  fn encode(&self, encoder: &mut S) -> Result<(), E> {
      encoder.emit_struct("Mesh", 1, |encoder| {
          try!(encoder.emit_struct_field( "name", 0u, |encoder| self.name.encode(encoder)));
          Ok(())
      })
  }
}

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

impl <S: Encoder<E>, E, T> Encodable<S, E> for ResTT<T> {
  fn encode(&self, encoder: &mut S) -> Result<(), E> {
      encoder.emit_struct("Mesh", 1, |encoder| {
          try!(encoder.emit_struct_field( "name", 0u, |encoder| self.name.encode(encoder)));
          Ok(())
      })
  }
}

impl<S: Decoder<E>, E, T> Decodable<S, E> for ResTT<T> {
  fn decode(decoder: &mut S) -> Result<ResTT<T>, E> {
    decoder.read_struct("root", 0, |decoder| {
         Ok(
             ResTT{
                 name : try!(decoder.read_struct_field("name", 0, |decoder| Decodable::decode(decoder))),
                 resource : ResNone,
            }
           )
    })
  }
}


