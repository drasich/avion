use mesh;
use texture;
use shader;
use serialize::{Encodable, Encoder, Decoder, Decodable};
use std::collections::HashMap;
use std::collections::hashmap::{Occupied,Vacant};
use sync::{RWLock, Arc};
//use std::io::timer::sleep;
//use std::time::duration::Duration;


/*
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
*/

pub trait ResourceT  {
    fn init(&mut self);
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

impl<T> ResTT<T>
{
    pub fn new(name : &str) -> ResTT<T>
    {
        ResTT {
            name : String::from_str(name),
            resource : ResNone
        }
    }
}

pub trait Create
{
    fn create(name : &str) -> Self;
    fn inittt(&mut self);
}

impl Create for mesh::Mesh
{
    fn create(name : &str) -> mesh::Mesh
    {
        mesh::Mesh::new_from_file(name)
    }

    fn inittt(&mut self)
    {
        if self.state == 0 {
            //TODO can be read anywhere
            self.file_read();
        }
    }
}

impl Create for shader::Material
{
    fn create(name : &str) -> shader::Material
    {
        shader::Material::new(name)
    }

    fn inittt(&mut self)
    {
        //TODO
        self.read();
    }
}

impl Create for shader::Shader
{
    fn create(name : &str) -> shader::Shader
    {
        shader::Shader::new(name)
    }

    fn inittt(&mut self)
    {
        //TODO
        //self.read();
    }
}

impl Create for texture::Texture
{
    fn create(name : &str) -> texture::Texture
    {
        texture::Texture::new(name)
    }

    fn inittt(&mut self)
    {
        //TODO
        self.load();
    }
}




pub struct ResourceManager<T>
{
    resources : Arc<RWLock<HashMap<String, ResTest<T>>>>,
}

impl<T:'static+Create+Sync+Send> ResourceManager<T> {
    pub fn new() -> ResourceManager<T>
    {
        ResourceManager {
            resources : Arc::new(RWLock::new(HashMap::new())),
        }
    }

    pub fn request_use(&mut self, name : &str) -> ResTest<T>
    {
        let ms1 = self.resources.clone();
        let mut ms1w = ms1.write();

        let v : &mut ResTest<T> = match ms1w.entry(String::from_str(name)) {
            Vacant(entry) => entry.set(ResNone),
            Occupied(entry) => entry.into_mut(),
        };

        let s = String::from_str(name);
        let msc = self.resources.clone();

        match *v 
        {
            ResNone => {
                *v = ResWait;

                let ss = s.clone();

                let (tx, rx) = channel::<Arc<RWLock<T>>>();
                spawn( proc() {
                    //sleep(::std::time::duration::Duration::seconds(5));
                    let mt : T = Create::create(ss.as_slice());
                    let m = Arc::new(RWLock::new(mt));
                    m.write().inittt();
                    tx.send(m.clone());
                });

                spawn( proc() {
                    loop {
                    match rx.try_recv() {
                        Err(_) => {},//println!("nothing"),
                        Ok(value) =>  { 
                            //println!("received val {} ", value.read().name);

                            let mut mscwww = msc.write();

                            match mscwww.entry(s.clone()) {
                                Vacant(entry) => entry.set(ResNone),
                                Occupied(mut entry) => { 
                                    *entry.get_mut() = ResData(value.clone());
                                    entry.into_mut()
                                }
                            };

                            break; }
                    }
                    }
                });

                return ResWait;
            },
            ResData(ref yep) => {
                return ResData(yep.clone());
            },
            ResWait => {
                return ResWait;
            }
        }
    }
}


//#[deriving(Decodable, Encodable)]
/*
pub struct ResourceRef
{
    pub name : String,
    pub resource : Resource
}
*/

impl <S: Encoder<E>, E, T> Encodable<S, E> for ResTT<T> {
  fn encode(&self, encoder: &mut S) -> Result<(), E> {
      encoder.emit_struct("NotImportantName", 1, |encoder| {
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


