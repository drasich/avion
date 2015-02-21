use object;
use std::collections::{LinkedList};
use std::sync::{RwLock, Arc};
use std::old_io::File;
use rustc_serialize::{json, Encodable, Encoder, Decoder, Decodable};
use uuid::Uuid;

pub struct Scene
{
    pub name : String,
    pub id : Uuid,
    pub objects : LinkedList<Arc<RwLock<object::Object>>>
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
        let file = File::open(&Path::new(file_path)).read_to_string().unwrap();
        let scene : Scene = json::decode(file.as_slice()).unwrap();

        scene.post_read_parent_set();

        scene
    }

    fn post_read_parent_set(&self)
    {
        for o in self.objects.iter()
        {
            post_read_parent_set(o.clone());
        }
    }

    pub fn save(&self)
    {
        println!("save scene todo serialize");
        /*
        let mut file = File::create(&Path::new(self.name.as_slice()));
        //let mut stdwriter = stdio::stdout();
        //let mut encoder = json::Encoder::new(&mut stdwriter);
        //let mut encoder = json::PrettyEncoder::new(&mut stdwriter);
        let mut encoder = json::PrettyEncoder::new(&mut file);
        //let mut encoder = json::Encoder::new(&mut file);

        //println!("scene : \n\n {}", json::encode(&scene));
        self.encode(&mut encoder).unwrap();
        */

        /*
        let mut file = File::create(&Path::new(self.name.as_slice()));
        let encoded = json::encode(self);
        let mut s = String::new();
        {
            let mut encoder = json::PrettyEncoder::new(&mut s);
            let _ = self.encode(&mut encoder);
        }

        let result = file.write_str(s.as_slice());
        */
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
}

//impl <S: Encoder<E>, E> Encodable<S, E> for Arc<RwLock<object::Object>> {
impl Encodable for RwLock<object::Object> {
  fn encode<E : Encoder>(&self, encoder: &mut E) -> Result<(), E::Error> {
      self.read().unwrap().encode(encoder)
  }
}

//impl<S: Decoder<E>, E> Decodable<S, E> for Arc<RwLock<object::Object>> {
//fn decode(decoder: &mut S) -> Result<Arc<RwLock<object::Object>>, E> {
impl Decodable for RwLock<object::Object> {
  fn decode<D : Decoder>(decoder: &mut D) -> Result<RwLock<object::Object>, D::Error> {
      //Ok(Arc::new(RwLock::new(try!(Decodable::decode(decoder)))))
      Ok(RwLock::new(try!(Decodable::decode(decoder))))
  }
}

impl Encodable for Scene {
  fn encode<E : Encoder>(&self, encoder: &mut E) -> Result<(), E::Error> {
      encoder.emit_struct("Scene", 1, |encoder| {
          try!(encoder.emit_struct_field( "name", 0us, |encoder| self.name.encode(encoder)));
          try!(encoder.emit_struct_field( "id", 1us, |encoder| self.id.encode(encoder)));
          try!(encoder.emit_struct_field( "objects", 2us, |encoder| self.objects.encode(encoder)));
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

