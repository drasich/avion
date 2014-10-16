use object;
//use std::collections::{DList,Deque};
use std::collections::{DList};
use std::rc::Rc;
use std::cell::RefCell;
use sync::{RWLock, Arc,RWLockReadGuard};
use std::io::File;
use serialize::{json, Encodable, Encoder, Decoder, Decodable};

pub struct Scene
{
    pub name : String,
    pub objects : DList<Arc<RWLock<object::Object>>>
}

impl Scene
{
    pub fn new(name : &str) -> Scene
    {
        Scene {
            name : String::from_str(name),
            objects : DList::new(),
        }
    }

    pub fn new_from_file(file_path : &str) -> Scene
    {
        let file = File::open(&Path::new(file_path)).read_to_string().unwrap();
        let scene : Scene = json::decode(file.as_slice()).unwrap();

        scene
    }

    pub fn save(&self)
    {
        let mut file = File::create(&Path::new(self.name.as_slice()));
        //let mut stdwriter = stdio::stdout();
        //let mut encoder = json::Encoder::new(&mut stdwriter);
        //let mut encoder = json::PrettyEncoder::new(&mut stdwriter);
        let mut encoder = json::PrettyEncoder::new(&mut file);
        //let mut encoder = json::Encoder::new(&mut file);

        //println!("scene : \n\n {}", json::encode(&scene));
        self.encode(&mut encoder).unwrap();
    }

    pub fn object_find(&self, name : &str) -> Option<Arc<RWLock<object::Object>>>
    {
        for o in self.objects.iter()
        {
            if o.read().name.as_slice() == name {
                return Some(o.clone());
            }
        }

        None
    }

}

impl <S: Encoder<E>, E> Encodable<S, E> for Arc<RWLock<object::Object>> {
  fn encode(&self, encoder: &mut S) -> Result<(), E> {
      self.read().encode(encoder)
  }
}

impl<S: Decoder<E>, E> Decodable<S, E> for Arc<RWLock<object::Object>> {
  fn decode(decoder: &mut S) -> Result<Arc<RWLock<object::Object>>, E> {
      Ok(Arc::new(RWLock::new(try!(Decodable::decode(decoder)))))
  }
}


impl <S: Encoder<E>, E> Encodable<S, E> for Scene {
  fn encode(&self, encoder: &mut S) -> Result<(), E> {
      encoder.emit_struct("Scene", 1, |encoder| {
          try!(encoder.emit_struct_field( "name", 0u, |encoder| self.name.encode(encoder)));
          try!(encoder.emit_struct_field( "objects", 1u, |encoder| self.objects.encode(encoder)));
          Ok(())
      })
  }
}

impl<S: Decoder<E>, E> Decodable<S, E> for Scene {
  fn decode(decoder: &mut S) -> Result<Scene, E> {
    decoder.read_struct("root", 0, |decoder| {
         Ok(Scene{
          name: try!(decoder.read_struct_field("name", 0, |decoder| Decodable::decode(decoder))),
          //objects: DList::new(),
          objects: try!(decoder.read_struct_field("objects", 0, |decoder| Decodable::decode(decoder))),
          //tests: try!(decoder.read_struct_field("objects", 0, |decoder| Decodable::decode(decoder))),
          //tests: DList::new()
        })
    })
  }
}

