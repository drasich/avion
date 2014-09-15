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
    ResWait,
    ResNone
}

pub struct ResTT<T>
{
    pub name : String,
    pub resource : ResTest<T>
}

pub struct ResourceManager
{
    meshes : Arc<RWLock<HashMap<String, ResTest<mesh::Mesh>>>>
}

impl ResourceManager {
    pub fn new() -> ResourceManager
    {
        ResourceManager {
            meshes : Arc::new(RWLock::new(HashMap::new())),
        }
    }

    pub fn request_use(&mut self, name : &str) -> ResTest<mesh::Mesh>
    {
        let ms1 = self.meshes.clone();
        let mut ms1w = ms1.write();

        let v : &mut ResTest<mesh::Mesh>  = ms1w.find_or_insert_with(String::from_str(name), 
                |key | ResNone);

        let s = String::from_str(name);

        let msc = self.meshes.clone();

        match *v 
        {
            ResNone => {
                *v = ResWait;

                let ss = s.clone();

                let (tx, rx) = channel::<Arc<RWLock<mesh::Mesh>>>();
                spawn( proc() {
                    //sleep(::std::time::duration::Duration::seconds(5));
                    let m = Arc::new(RWLock::new(mesh::Mesh::new_from_file(ss.as_slice())));
                    m.write().inittt();
                    tx.send(m.clone());
                });

                spawn( proc() {
                    loop {
                    match rx.try_recv() {
                        Err(e) => {},//println!("nothing"),
                        Ok(value) =>  { 
                            println!("received val {} ", value.read().name);

                            let mut mscwww = msc.write();

                            let newval = mscwww.insert_or_update_with(
                                s.clone(),
                                ResNone,
                                |_key, val| *val = ResData(value.clone()));

                            break; }
                    }
                    }
                });

                println!("request : it was none, now it is wait");
                return ResWait;
            },
            ResData(ref yep) => {
                println!("request : yes! returning data");
                return ResData(yep.clone());
            },
            ResWait => {
                println!("request not yet, please wait");
                return ResWait;
            }

        }

        return ResNone;
    }

    pub fn start(&self)
    {
        //to be spawn
        loop{
            //TODO for all meshes state that are receiver, try to receive
        }
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


