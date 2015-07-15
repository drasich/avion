use mesh;
use texture;
use shader;
use fbo;
use material;
use armature;

use rustc_serialize::{Encodable, Encoder, Decoder, Decodable};
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::collections::hash_map::Entry::{Occupied,Vacant};
use std::sync::{RwLock, Arc};
use std::sync::mpsc::channel;
//use std::time::Duration;
use self::ResTest::{ResData,ResWait,ResNone};
use std::thread;

use std::rc::Rc;
use std::cell::RefCell;


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

impl<T:Create+Send+Sync+'static> ResTT<T>
{
    pub fn new(name : &str) -> ResTT<T>
    {
        ResTT {
            name : String::from(name),
            resource : ResTest::ResNone
        }
    }

    pub fn new_instant(name : &str, rm : &mut ResourceManager<T>) -> ResTT<T>
    {
        let mut r = ResTT::new(name);
        r.load_instant(rm);

        r
    }

    pub fn new_with_res(name : &str, res : ResTest<T>) -> ResTT<T>
    {
        ResTT {
            name : String::from(name),
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

    pub fn load_instant(&mut self, manager : &mut ResourceManager<T> )
    {
        match self.resource {
            ResNone | ResWait => {
                let data = manager.request_use_no_proc(self.name.as_ref());
                self.resource = ResTest::ResData(data);
            },
            _ => {}
        }
    }

    pub fn get_resource_instant(&mut self, manager : &mut ResourceManager<T> ) -> Arc<RwLock<T>>
    {
        match self.resource {
            ResTest::ResData(ref rd) => rd.clone(),
            _ => {
                let data = manager.request_use_no_proc(self.name.as_ref());
                self.resource = ResTest::ResData(data.clone());
                data
            }
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

impl Create for armature::Armature
{
    fn create(name : &str) -> armature::Armature
    {
        armature::Armature::new(name)
    }

    fn inittt(&mut self)
    {
        if self.state == 0 {
            self.file_read();
        }
    }
}





pub struct ResourceManager<T>
{
    resources : HashMap<String, Arc<RwLock<ResTest<T>>>>,
}

unsafe impl<T:Send> Send for ResourceManager<T> {}
unsafe impl<T:Sync> Sync for ResourceManager<T> {}

type ReceiveResource<T> = fn(ResTest<T>);

impl<T:'static+Create+Sync+Send> ResourceManager<T> {
    pub fn new() -> ResourceManager<T>
    {
        ResourceManager {
            resources : HashMap::new(),
        }
    }

    pub fn request_use(&mut self, name : &str) -> ResTest<T>
    {
        let key = String::from(name);

        let va : Arc<RwLock<ResTest<T>>> = match self.resources.entry(key) {
            Entry::Vacant(entry) => entry.insert(Arc::new(RwLock::new(ResTest::ResNone))).clone(),
            Entry::Occupied(entry) => entry.into_mut().clone(),
        };

        {
            let v : &mut ResTest<T> = &mut *va.write().unwrap();

            match *v {
                ResTest::ResData(ref yep) => {
                    return ResTest::ResData(yep.clone());
                },
                ResTest::ResWait => {
                    return ResTest::ResWait;
                },
                ResTest::ResNone => {
                    *v = ResTest::ResWait;
                },
            }
        }

        let s = String::from(name);

        let (tx, rx) = channel::<Arc<RwLock<T>>>();
        let guard = thread::spawn(move || {
            //thread::sleep(::std::time::Duration::seconds(5));
            //thread::sleep_ms(5000);
            let mt : T = Create::create(s.as_ref());
            let m = Arc::new(RwLock::new(mt));
            m.write().unwrap().inittt();
            let result = tx.send(m.clone());
        });

        //let result = guard.join();

        thread::spawn( move || {
            loop {
                match rx.try_recv() {
                    Err(_) => {},
                    Ok(value) =>  { 
                        let entry = &mut *va.write().unwrap();
                        *entry = ResTest::ResData(value.clone());
                        break; }
                }
            }
        });

        return ResTest::ResWait;


    }

        //TODO wip
        /*
    pub fn request_use_and_call<F>(&mut self, name : &str, f : F) 
        -> ResTest<T> where F : Fn(ResTest<T>), F:Send +'static
    {
        let ms1 = self.resources.clone();
        let mut ms1w = ms1.write().unwrap();

        let key = String::from(name);

        let v : &mut ResTest<T> = match ms1w.entry(key) {
        //let v : &mut ResTest<T> = match ms1w.entry(&s) {
            Entry::Vacant(entry) => entry.insert(ResTest::ResNone),
            Entry::Occupied(entry) => entry.into_mut(),
        };

        let s = String::from(name);
        let msc = self.resources.clone();

        match *v 
        {
            ResNone => {
                *v = ResTest::ResWait;

                let ss = s.clone();

                let (tx, rx) = channel::<Arc<RwLock<T>>>();
                let guard = thread::scoped(move || {
                    //sleep(::std::time::duration::Duration::seconds(5));
                    let mt : T = Create::create(ss.as_ref());
                    let m = Arc::new(RwLock::new(mt));
                    m.write().unwrap().inittt();
                    let result = tx.send(m.clone());
                });

                let result = guard.join();

                thread::spawn( move || {
                    loop {
                    match rx.try_recv() {
                        Err(_) => {},
                        Ok(value) =>  { 
                            let mut mscwww = msc.write().unwrap();
                            let rd = ResTest::ResData(value.clone());
                            f(rd);

                            match mscwww.entry(s.clone()) {
                                //Entry::Vacant(entry) => entry.insert(ResTest::ResNone),
                                Entry::Vacant(entry) => entry.insert(ResTest::ResData(value.clone())),
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
    */


    pub fn request_use_no_proc(&mut self, name : &str) -> Arc<RwLock<T>>
    {
        let key = String::from(name);

        let va : Arc<RwLock<ResTest<T>>> = match self.resources.entry(key) {
            Vacant(entry) => entry.insert(Arc::new(RwLock::new(ResNone))).clone(),
            Occupied(entry) => entry.into_mut().clone(),
        };

        let v : &mut ResTest<T> = &mut *va.write().unwrap();

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
            try!(encoder.emit_struct_field( "name", 0usize, |encoder| self.name.encode(encoder)));
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
            res.resource = manager.request_use(res.name.as_ref());
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

pub struct ResourceGroup
{
    pub mesh_manager : RefCell<ResourceManager<mesh::Mesh>>,
    pub shader_manager : RefCell<ResourceManager<shader::Shader>>,
    pub texture_manager : RefCell<ResourceManager<texture::Texture>>,
    pub material_manager : RefCell<ResourceManager<material::Material>>,
    pub fbo_manager : RefCell<ResourceManager<fbo::Fbo>>,
    pub armature_manager : RefCell<ResourceManager<armature::Armature>>,
}

impl ResourceGroup
{
    pub fn new() -> ResourceGroup
    {
        //let fbo_all = fbo_manager.request_use_no_proc("fbo_all");
        //let fbo_selected = fbo_manager.request_use_no_proc("fbo_selected");

        ResourceGroup {
            mesh_manager : RefCell::new(ResourceManager::new()),
            shader_manager : RefCell::new(ResourceManager::new()),
            texture_manager : RefCell::new(ResourceManager::new()),
            material_manager : RefCell::new(ResourceManager::new()),
            fbo_manager : RefCell::new(ResourceManager::new()),
            armature_manager : RefCell::new(ResourceManager::new()),
        }
    }
}
