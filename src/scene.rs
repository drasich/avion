use std::collections::{LinkedList};
use std::sync::{RwLock, Arc};
use std::rc::Rc;
use std::cell::RefCell;
use std::fs::File;
use std::io::{Read,Write};
use rustc_serialize::{json, Encodable, Encoder, Decoder, Decodable};
use uuid::Uuid;
use std::path::Path;
use toml;

use object;
use camera;
use component;

pub struct Scene
{
    pub name : String,
    pub id : Uuid,
    pub objects : LinkedList<Arc<RwLock<object::Object>>>,
    pub camera : Option<Rc<RefCell<camera::Camera>>>
}

impl Scene
{
    /*
    pub fn new(name : &str) -> Scene
    {
        Scene {
            name : String::from_str(name),
            objects : LinkedList::new(),
        }
    }
    */

    pub fn new_from_file(file_path : &str) -> Scene
    {
        let mut file = String::new();
        File::open(&Path::new(file_path)).ok().unwrap().read_to_string(&mut file);
        let scene : Scene = json::decode(file.as_slice()).unwrap();

        scene.post_read();

        scene
    }

    fn post_read(&self)
    {
        for o in self.objects.iter()
        {
            post_read_parent_set(o.clone());

            if let Some(ref c) = self.camera {
                let mut cam = c.borrow_mut();
                let id = match cam.object_id {
                    Some(ref id) => id.clone(),
                    None => {
                        println!("camera has no id");
                        continue;
                    }
                };

                let (ob_id, name) = {
                    let ob = o.read().unwrap();
                    (ob.id.clone(), ob.name.clone())
                };

                if ob_id == id {
                    println!("fiiiiiiiiiiiiiiiiiiiiind");
                    cam.object = o.clone();
                }
                else if name == "robot"{
                /*
                    println!("it is not {}", o.read().unwrap().name);
                    let comp_mgr = component::manager::COMP_MGR.lock().unwrap();
                    let pc = comp_mgr.create_component("player_behavior").unwrap();
                    o.write().unwrap().add_component(
                        //Rc::new(RefCell::new(Box::new(component::player::Player::new()))));
                        Rc::new(RefCell::new(pc)));
                */
                }
            }
            else {
                println!("nooooooooo camera");
            }
        }
    }

    pub fn save(&self)
    {
        println!("save scene todo serialize");

        let mut file = File::create(&Path::new(self.name.as_slice())).ok().unwrap();
        let mut s = String::new();
        {
            let mut encoder = json::Encoder::new_pretty(&mut s);
            let _ = self.encode(&mut encoder);
        }

        let result = file.write(s.as_slice().as_bytes());
    }

    pub fn object_find(&self, name : &str) -> Option<Arc<RwLock<object::Object>>>
    {
        for o in self.objects.iter()
        {
            if o.read().unwrap().name.as_slice() == name {
                return Some(o.clone());
            }
        }

        None
    }

    pub fn find_object_by_id(&self, id : &Uuid) -> Option<Arc<RwLock<object::Object>>>
    {
        fn find(list : &LinkedList<Arc<RwLock<object::Object>>>, id : &Uuid) ->
            Option<Arc<RwLock<object::Object>>>
            {
                for o in list.iter()
                {
                    if o.read().unwrap().id == *id {
                        return Some(o.clone());
                    }
                    else {
                        if let Some(aro) = find(&o.read().unwrap().children, id) {
                            return Some(aro);
                        }
                    }
                }
                None
            }

        find(&self.objects, id)
    }

    pub fn find_objects_by_id(&self, ids : &mut Vec<Uuid>) -> LinkedList<Arc<RwLock<object::Object>>>
    {
        let mut return_list = LinkedList::new();
        fn find(
            list : &LinkedList<Arc<RwLock<object::Object>>>,
            ids : &mut Vec<Uuid>,
            return_list : &mut LinkedList<Arc<RwLock<object::Object>>>
            )
            {
                for o in list.iter()
                {
                    let mut found = false;
                    for i in 0..ids.len() {
                        if o.read().unwrap().id == ids[i] {
                            ids.remove(i);
                            return_list.push_back(o.clone());
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        find(&o.read().unwrap().children, ids, return_list);
                    }
                }
            }

        find(&self.objects, ids, &mut return_list);
        return_list
    }

    pub fn add_objects(&mut self, obs : &LinkedList<Arc<RwLock<object::Object>>>)
    {
        self.objects.append(&mut obs.clone());
    }

    pub fn add_objects_by_vec(&mut self, obs : Vec<Arc<RwLock<object::Object>>>)
    {
        //TODO vec to list
        for o in obs.iter() {
            self.objects.push_back(o.clone());
        }
    }

    pub fn remove_objects(&mut self, obs : &LinkedList<Arc<RwLock<object::Object>>>)
    {
        let mut list = LinkedList::new();
        for o in self.objects.iter() {
            let mut not_found = true;
            for r in obs.iter() {
                if o.read().unwrap().id == r.read().unwrap().id {
                    println!("found the id, break {}", o.read().unwrap().name);
                    not_found = false;
                    break;
                }
            }
            if not_found {
            println!("dit not found the id, adding {}", o.read().unwrap().name);
            list.push_back(o.clone());
            }
        }

        self.objects = list;
    }

    pub fn remove_objects_by_vec(&mut self, obs : Vec<Arc<RwLock<object::Object>>>)
    {
        let mut list = LinkedList::new();
        for o in self.objects.iter() {
            let mut not_found = true;
            for r in obs.iter() {
                if o.read().unwrap().id == r.read().unwrap().id {
                    println!("found the id, break {}", o.read().unwrap().name);
                    not_found = false;
                    break;
                }
            }
            if not_found {
            println!("dit not found the id, adding {}", o.read().unwrap().name);
            list.push_back(o.clone());
            }
        }

        self.objects = list;
    }

    pub fn savetoml(&self)
    {
        let s = toml::encode_str(self);
        println!("encoder toml : {} ", s );
    }

    /*
    pub fn new_toml(s : &str) -> Material
    {
        let mat : Material = toml::decode_str(s).unwrap();
        mat
    }
    */

    pub fn update(&mut self, dt : f64)
    {
        for o in self.objects.iter() {
            o.write().unwrap().update(dt);
        }
    }
}

impl Encodable for Scene {
  fn encode<E : Encoder>(&self, encoder: &mut E) -> Result<(), E::Error> {
      encoder.emit_struct("Scene", 1, |encoder| {
          try!(encoder.emit_struct_field( "name", 0usize, |encoder| self.name.encode(encoder)));
          try!(encoder.emit_struct_field( "id", 1usize, |encoder| self.id.encode(encoder)));
          try!(encoder.emit_struct_field( "objects", 2usize, |encoder| self.objects.encode(encoder)));
          try!(encoder.emit_struct_field( "camera", 3usize, |encoder| self.camera.encode(encoder)));
          Ok(())
      })
  }
}

impl Decodable for Scene {
  fn decode<D : Decoder>(decoder: &mut D) -> Result<Scene, D::Error> {
      decoder.read_struct("root", 0, |decoder| {
         Ok(Scene{
          name: try!(decoder.read_struct_field("name", 0, |decoder| Decodable::decode(decoder))),
          id: try!(decoder.read_struct_field("id", 0, |decoder| Decodable::decode(decoder))),
         //id : Uuid::new_v4(),
          //objects: LinkedList::new(),
          objects: try!(decoder.read_struct_field("objects", 0, |decoder| Decodable::decode(decoder))),
          //tests: try!(decoder.read_struct_field("objects", 0, |decoder| Decodable::decode(decoder))),
          //tests: LinkedList::new()
          //camera : None //try!(decoder.read_struct_field("camera", 0, |decoder| Decodable::decode(decoder)))
          camera : try!(decoder.read_struct_field("camera", 0, |decoder| Decodable::decode(decoder)))
          //camera : None
        })
    })
  }
}

pub fn post_read_parent_set(o : Arc<RwLock<object::Object>>)
{
    for c in o.read().unwrap().children.iter()
    {
        c.write().unwrap().parent = Some(o.clone());
        post_read_parent_set(c.clone());
    }
}

