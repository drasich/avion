use mesh;
use texture;
use shader;
use fbo;
use material;
use rustc_serialize::{Encodable, Encoder, Decoder, Decodable};
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::collections::hash_map::Entry::{Occupied,Vacant};
use std::sync::{RwLock, Arc};
use std::sync::mpsc::channel;
//use std::io::timer::sleep;
//use std::time::duration::Duration;
use self::ResTest::{ResData,ResWait,ResNone};
use std::thread::Thread;


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
    ResData(Arc<RwLock<T>>),
    ResWait,
    ResNone
}

pub struct ResTT<T>
{
    pub name : String,
    pub resource : ResTest<T>,
}

impl<T> ResTT<T>
{
    pub fn new(name : &str) -> ResTT<T>
    {
        ResTT {
            name : String::from_str(name),
            resource : ResTest::ResNone
        }
    }

    pub fn new_with_res(name : &str, res : ResTest<T>) -> ResTT<T>
    {
        ResTT {
            name : String::from_str(name),
            resource : res
        }
    }
}

impl<T> Clone for ResTT<T>
{
    fn clone(&self) -> ResTT<T>
    {
        let r = match self.resource {
            ResData(ref d) => ResData(d.clone()),
            ResWait => ResWait,
            ResNone => ResNone
        };

        ResTT {
            name : self.name.clone(),
            resource : r
        }
    }
}

impl <T:'static+Create+Send+Sync> ResTT<T>
{
    pub fn get_resource(&mut self, manager : &mut ResourceManager<T> ) -> Option<Arc<RwLock<T>>>
    {
        match self.resource {
            ResTest::ResData(ref rd) => Some(rd.clone()),
            //ResTest::ResWait => None,
            _ => resource_get(manager, self)
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

impl Create for material::Material
{
    fn create(name : &str) -> material::Material
    {
        material::Material::new(name)
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

impl Create for fbo::Fbo
{
    fn create(name : &str) -> fbo::Fbo
    {
        fbo::Fbo::new(name)
    }

    fn inittt(&mut self)
    {
        //TODO
    }
}




pub struct ResourceManager<T>
{
    resources : Arc<RwLock<HashMap<String, ResTest<T>>>>,
}

impl<T:'static+Create+Sync+Send> ResourceManager<T> {
    pub fn new() -> ResourceManager<T>
    {
        ResourceManager {
            resources : Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn request_use(&mut self, name : &str) -> ResTest<T>
    {
        let ms1 = self.resources.clone();
        let mut ms1w = ms1.write().unwrap();

        let key = String::from_str(name);

        let v : &mut ResTest<T> = match ms1w.entry(key) {
        //let v : &mut ResTest<T> = match ms1w.entry(&s) {
            Entry::Vacant(entry) => entry.insert(ResTest::ResNone),
            Entry::Occupied(entry) => entry.into_mut(),
        };

        let s = String::from_str(name);
        let msc = self.resources.clone();

        match *v 
        {
            ResNone => {
                *v = ResTest::ResWait;

                let ss = s.clone();

                let (tx, rx) = channel::<Arc<RwLock<T>>>();
                let guard = Thread::scoped(move || {
                    //sleep(::std::time::duration::Duration::seconds(5));
                    let mt : T = Create::create(ss.as_slice());
                    let m = Arc::new(RwLock::new(mt));
                    m.write().unwrap().inittt();
                    let result = tx.send(m.clone());
                });

                let result = guard.join();

                Thread::spawn( move || {
                    loop {
                    match rx.try_recv() {
                        Err(_) => {},
                        Ok(value) =>  { 
                            let mut mscwww = msc.write().unwrap();

                            match mscwww.entry(s.clone()) {
                                Entry::Vacant(entry) => entry.insert(ResTest::ResNone),
                                Entry::Occupied(mut entry) => { 
                                    *entry.get_mut() = ResTest::ResData(value.clone());
                                    entry.into_mut()
                                }
                            };

                            break; }
                    }
                    }
                });

                return ResTest::ResWait;
            },
            ResTest::ResData(ref yep) => {
                return ResTest::ResData(yep.clone());
            },
            ResTest::ResWait => {
                return ResTest::ResWait;
            }
        }
    }

    pub fn request_use_no_proc(&mut self, name : &str) -> Arc<RwLock<T>>
    {
        let ms1 = self.resources.clone();
        let mut ms1w = ms1.write().unwrap();

        let key = String::from_str(name);

        let v : &mut ResTest<T> = match ms1w.entry(key) {
            Vacant(entry) => entry.insert(ResNone),
            Occupied(entry) => entry.into_mut(),
        };

        match *v 
        {
            ResNone | ResWait => {
                let mt : T = Create::create(name);
                let m = Arc::new(RwLock::new(mt));
                m.write().unwrap().inittt();

                *v = ResData(m.clone());
                return m.clone();
            },
            ResData(ref yep) => {
                return yep.clone();
            },
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

impl <T> Encodable for ResTT<T> {
    fn encode<S: Encoder>(&self, encoder: &mut S) -> Result<(), S::Error> {
        encoder.emit_struct("NotImportantName", 1, |encoder| {
            try!(encoder.emit_struct_field( "name", 0us, |encoder| self.name.encode(encoder)));
            Ok(())
        })
    }
}

impl<T> Decodable for ResTT<T> {
    fn decode<D : Decoder>(decoder: &mut D) -> Result<ResTT<T>, D::Error> {
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

pub fn resource_get<T:'static+Create+Send+Sync>(
    manager : &mut ResourceManager<T>,
    res: &mut ResTT<T>) 
    -> Option<Arc<RwLock<T>>>
{
    let mut the_res : Option<Arc<RwLock<T>>> = None;
    match res.resource{
        ResNone | ResWait => {
            res.resource = manager.request_use(res.name.as_slice());
            match res.resource {
                ResData(ref data) => {
                    the_res = Some(data.clone());
                }
                _ => {}
            }
        },
        ResData(ref data) => {
            the_res = Some(data.clone());
        },
    }

    the_res
}

