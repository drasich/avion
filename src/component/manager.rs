use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::rc::Rc;
use std::cell::RefCell;
use object::Object;
use rustc_serialize::{json, Encodable, Encoder, Decoder, Decodable};
use std::any::Any;
//use std::thread;
///use std::sync::mpsc::channel;
use component::player::{Player, Enemy, Collider};
use armature::ArmaturePath;
use component::mesh_render::{MeshRender, MeshRenderer};
use component::armature_animation::ArmatureAnimation;
use component::player::PlayerBehavior;
use resource;

use property::{PropertyGet, PropertyWrite, WriteValue};

pub trait Component : Any
{
    //fn new(&self) -> Rc<RefCell<Box<Component>>>;
    //fn copy(&self) -> Rc<RefCell<Box<Component>>>;
    /*
    {
        let comp_mgr = COMP_MGR.lock().unwrap();
        let comp = comp_mgr.create_component(self.get_name().as_ref()).unwrap();
        Rc::new(RefCell::new(comp))
    }
    */
    fn load(&mut self) {}
    fn update(&mut self, ob : &mut Object, dt : f64) {}

    fn get_name(&self) -> String;
    /*
    fn get_dependencies(&self) -> Vec<String>
    {
        return Vec::new()
    }
    */
    //fn new(ob : &Object) -> Self where Self : Sized;
    /*
    fn new(ob : &Object) -> Box<Component>
    {
        let comp_mgr = COMP_MGR.lock().unwrap();
        let comp = comp_mgr.create_component("remove").unwrap();
        comp
    }
    */

}

#[derive(Clone)]
pub enum Components
{
    Empty,
    MeshRender(MeshRenderer),
    ArmatureAnimation(ArmatureAnimation),
    PlayerBehavior(PlayerBehavior)
}

impl Components {
    pub fn get_comp<T:Any>(&self) -> Option<&T>
    {
        match *self {
            Components::Empty => {
                None
            }
            Components::MeshRender(ref p) => {
                let anyp = p as &Any;
                anyp.downcast_ref::<T>()
            },
            Components::ArmatureAnimation(ref p) => {
                let anyp = p as &Any;
                anyp.downcast_ref::<T>()
            },
            Components::PlayerBehavior(ref p) => {
                let anyp = p as &Any;
                anyp.downcast_ref::<T>()
            },
            //_ => None
        }
    }

}

impl Component for Components
{
    /*
    fn copy(&self) -> Rc<RefCell<Box<Component>>>
    {
        //TODO
        Rc::new(RefCell::new(box PlayerBehavior))
    }
    */

    fn get_name(&self) -> String
    {
        match *self {
            Components::Empty => {
                String::from("empty")
            },
            Components::MeshRender(ref p) => {
                p.get_name()
            },
            Components::ArmatureAnimation(ref p) => {
                p.get_name()
            },
            Components::PlayerBehavior(ref p) => {
                p.get_name()
            },
            //_ => String::from_str("no_name_implemented")
        }

    }

    fn update(&mut self, ob : &mut Object, dt : f64)
    {
        match *self {
            Components::Empty => {},
            Components::MeshRender(ref mut p) => {
                p.update(ob, dt);
            },
            Components::ArmatureAnimation(ref mut p) => {
                p.update(ob, dt);
            },
            Components::PlayerBehavior(ref mut p) => {
                p.update(ob, dt);
            },
            //_ => String::from_str("no_name_implemented")
        }

    }


}


pub trait Encode
{
  fn encode_this<E: Encoder>(&self, encoder: &mut E);// -> Result<(), &str>;
}


#[derive(Clone, RustcEncodable, RustcDecodable)]
pub enum CompData
{
    None,
    Player(Player),
    Enemy(Enemy),
    Collider(Collider),
    Armature(ArmaturePath),
    MeshRender(MeshRender)
}

impl Default for CompData
{
    fn default() -> CompData 
    { 
        CompData::None
    }
}

impl CompData
{
    pub fn get_comp<T:Any>(&self) -> Option<&T>
    {
        match *self {
            CompData::Player(ref p) => {
                let anyp = p as &Any;
                anyp.downcast_ref::<T>()
            },
            CompData::Armature(ref p) => {
                let anyp = p as &Any;
                anyp.downcast_ref::<T>()
            },
            CompData::MeshRender(ref c) => {
                let anyp = c as &Any;
                anyp.downcast_ref::<T>()
            },
            _ => None
        }
    }

    pub fn get_mut_comp<T:Any>(&mut self) -> Option<&mut T>
    {
        match *self {
            CompData::Player(ref mut p) => {
                let anyp = p as &mut Any;
                anyp.downcast_mut::<T>()
            },
            CompData::Armature(ref mut p) => {
                let anyp = p as &mut Any;
                anyp.downcast_mut::<T>()
            },
            CompData::MeshRender(ref mut p) => {
                let anyp = p as &mut Any;
                anyp.downcast_mut::<T>()
            },
            _ => None
        }
    }

    pub fn get_kind_string(&self) -> String
    {
        match *self {
            CompData::Player(_) => {
                String::from("Player")
            },
            CompData::Armature(_) => {
                String::from("Armature")
            },
            CompData::MeshRender(_) => {
                String::from("MeshRender")
            },
            _ => {
                String::from("NotImplemented")
            }
        }

    }

}

impl PropertyWrite for CompData {
  fn test_set_property_hier(&mut self, name : &str, value: &Any)
  {
      println!("compdata TEST set property hier: {}", name);
      let v : Vec<&str> = name.split('/').collect();

      match v.len() {
          0 => {},
          1 => {
              match *self {
                  CompData::Player(ref mut p) => {
                      p.test_set_property_hier(v[0], value);
                  },
                  CompData::Armature(ref mut p) => {
                      p.test_set_property_hier(v[0], value);
                  },
                  CompData::MeshRender(ref mut p) => {
                      p.test_set_property_hier(v[0], value);
                  },
                  _ => {println!("not yet implemented");}
              }
          },
          _ => {
              let yep : String = v[1..].join("/");
              if v[0] == self.get_kind_string() {
                  match *self {
                      CompData::Player(ref mut p) => {
                          p.test_set_property_hier(yep.as_ref(), value);
                      },
                      CompData::Armature(ref mut p) => {
                          p.test_set_property_hier(yep.as_ref(), value);
                      },
                      CompData::MeshRender(ref mut p) => {
                          p.test_set_property_hier(yep.as_ref(), value);
                      },
                      _ => {println!("not yet implemented");}
                  }
              }
          }
      }
  }

  fn set_property_hier(&mut self, name : &str, value: WriteValue)
  {
      println!("compdata set property hier: {}", name);
      let v : Vec<&str> = name.split('/').collect();

      match v.len() {
          0 => {},
          1 => {},
          _ => {
              let yep : String = v[1..].join("/");
              if v[0] == self.get_kind_string() {
                  match *self {
                      CompData::Player(ref mut p) => {
                          p.set_property_hier(yep.as_ref(), value);
                      },
                      CompData::Armature(ref mut p) => {
                          p.set_property_hier(yep.as_ref(), value);
                      },
                      CompData::MeshRender(ref mut p) => {
                          p.set_property_hier(yep.as_ref(), value);
                      },
                      _ => {println!("not yet implemented");}
                  }
              }
          }
      }
  }
}

impl PropertyGet for CompData
{
  fn get_property_hier(&self, name : &str) -> Option<Box<Any>>
  {
      let v : Vec<&str> = name.split('/').collect();

      match v.len() {
          0 => None,
          1 => None,
          _ => {
              let yep : String = v[1..].join("/");
              if v[0] == self.get_kind_string() {
                  match *self {
                      CompData::Player(ref p) => {
                          p.get_property_hier(yep.as_ref())
                      },
                      CompData::Armature(ref p) => {
                          p.get_property_hier(yep.as_ref())
                      },
                      CompData::MeshRender(ref p) => {
                          p.get_property_hier(yep.as_ref())
                      },
                      _ => {
                          println!("not yet implemented");
                          None
                      }
                  }
              }
              else {
                  None
              }
          }
      }
  }
}


//type ComponentCreationFn = fn() -> Box<Component>;
type ComponentCreationFn = fn(&Object, &resource::ResourceGroup) -> Box<Components>;

pub struct Manager {
    name : String,
    components : HashMap<String, ComponentCreationFn>
}

impl Manager {
    pub fn new() -> Self
    {
        Manager {
            name : "test".to_string(),
            components: HashMap::new()
        }
    }
    pub fn register_component(&mut self, name : &str,
                              f : ComponentCreationFn)
                              //f : fn() -> Box<Component>)
                              //f : fn() -> Component)
    {
        self.components.insert(name.to_string(), f);
    }

    pub fn get_component_create_fn(&self, name : &str) -> Option<&ComponentCreationFn>
    {
        //self.components.get(&name.to_string())
        match self.components.get(&name.to_string()) {
            Some(f) => Some(f),
            None => None
        }
    }

}


lazy_static! {
    pub static ref HASHMAP: Mutex<HashMap<u32, &'static str>> = {
        let mut m = HashMap::new();
        m.insert(0, "foo");
        m.insert(1, "bar");
        m.insert(2, "baz");
        println!("test chris");
        Mutex::new(m)
    };
    pub static ref COUNT: usize = {
        println!("size !!!!!!");
        let hash = &mut HASHMAP.lock().unwrap();
        hash.len()
    };
    static ref NUMBER: u32 = times_two(21);
    pub static ref COMP_MGR : Mutex<Manager> = Mutex::new(Manager::new());
}

fn times_two(n: u32) -> u32 { n * 2 }

/*
impl Encodable for Component {
  fn encode<E : Encoder>(&self, encoder: &mut E) -> Result<(), E::Error> {
      encoder.emit_struct("Component", 1, |encoder| {
          try!(encoder.emit_struct_field( "name", 0usize, |encoder| self.get_name().encode(encoder)));
          Ok(())
      })
  }
}

impl Decodable for Box<Component> {
  fn decode<D : Decoder>(decoder: &mut D) -> Result<Box<Component>, D::Error> {
    decoder.read_struct("root", 0, |decoder| {
          let name : String = try!(decoder.read_struct_field("name", 0, |decoder| Decodable::decode(decoder)));
          let comp_mgr = COMP_MGR.lock().unwrap();
          Ok(
              comp_mgr.create_component(name.as_ref()).unwrap()
            )
    })
  }
}
*/

