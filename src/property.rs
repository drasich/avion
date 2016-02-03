use object;
use vec;
use std::any::{Any};//, AnyRefExt};
use std::f64::consts;
use transform;
use component;
use component::mesh_render;
use resource;
use mesh;
use material;

//log_syntax!()
//trace_macros!(true)

/* //For testing
#[derive(Decodable, Encodable, Clone)]
pub struct Chris
{
    pub x : f64,
    pub y : f64,
    pub z : f64,
    pub position : vec::Vec3,
    pub boxpos : Box<vec::Vec3>
}

impl Chris
{
    pub fn new()-> Chris
    {
        Chris {
            x : 0f64,
            y : 0f64,
            z : 0f64,
            position : vec::Vec3::new(6f64,7f64,8f64),
            boxpos : box vec::Vec3::new(6f64,7f64,8f64)}
    }
}
*/

pub trait PropertyRead
{
  fn get_property(&self) -> Option<Box<Any>>;
  //fn get_some() -> Option<Self>;
}

impl<T:Any+Clone> PropertyRead for T
{
  fn get_property(&self) -> Option<Box<Any>>
  {
      Some(box self.clone())
  }
}

/*
impl<T:Any> PropertyRead for resource::ResTT<T>
{
  fn get_property(&self) -> Option<Box<Any>>
  {
      Some(box self.clone())
  }
}

macro_rules! property_read_impl(
    ($my_type:ty) => (

        impl PropertyRead for $my_type
        {
            fn get_property(&self) -> Option<Box<Any>>
            {
                Some(box self.clone())
            }
        }
));
*/


/*
property_read_impl!(f64);
property_read_impl!(String);
property_read_impl!(vec::Vec3);
property_read_impl!(vec::Quat);
property_read_impl!(transform::Orientation);
*/
//property_read_impl!(mesh_render::MeshRender);


pub trait PropertyGet
{
  fn get_property_hier(&self, name : &str) -> Option<Box<Any>>
  {
      println!("default property does not do anything");
      None
  }
}

impl PropertyGet for f64{}
impl PropertyGet for String{}

/*
impl<T:Any+Clone> PropertyRead for Option<T>
{
  fn get_property(&self) -> Option<Box<Any>>
  {
      match *self {
          Some(ref s) => {
              println!("get option property");
              Some(box s.clone())
          },
          None => {
              println!("get option property : none");
              None
          }
      }
  }
}
*/

pub enum WriteValue
{
    None,
    Some,
    Any(Box<Any>)
}


pub trait PropertyWrite
{
  fn test_set_property(&mut self, value: &Any)
  {
      println!("default set property does nothing");
  }
  fn test_set_property_hier(&mut self, name : &str, value: &Any)
  {
      println!("default set property HIER does nothing");
  }

  fn set_property(&mut self, value: WriteValue)
  {
      match value {
          WriteValue::Any(v) => self.test_set_property(&*v),
          _ => {}
      }
  }
  fn set_property_hier(&mut self, name : &str, value: WriteValue)
  {
      match value {
          WriteValue::Any(v) => self.test_set_property_hier(name, &*v),
          _ => {}
      }
  }

  fn add_item(&mut self, name : &str, index :usize, value : &Any)
  {
      println!("default add item does nothing");
  }

  fn del_item(&mut self, name : &str, index :usize)
  {
      println!("default del does nothing");
  }
}


impl<T:PropertyWrite+Default> PropertyWrite for Vec<T> {

  fn test_set_property_hier(&mut self, name : &str, value: &Any)
  {
      let v : Vec<&str> = name.split('/').collect();

      match v.len() {
          0 => {},
          1 => {
              let index = v[0].parse::<usize>().unwrap();
              self[index].test_set_property(value);
          },
          _ => {
              let yep : String = v[1..].join("/");
              let index = v[0].parse::<usize>().unwrap();
              self[index].test_set_property_hier(yep.as_ref(), value);
          }
      }
  }

  fn add_item(&mut self, name : &str, index : usize, value : &Any)
  {
      let v : Vec<&str> = name.split('/').collect();
      println!("yooooooooo : {}", name);

      match v.len() {
          0 => {
              self.insert(index, Default::default());
              self[index].test_set_property(value);
          },
          1 => {
              //let index = v[0].parse::<usize>().unwrap();
              self.insert(index, Default::default());
              self[index].test_set_property(value);
          },
          _ => {
              let yep : String = v[1..].join("/");
              //let index = v[0].parse::<usize>().unwrap();
              self[index].add_item(yep.as_ref(), index, value);
          }
      }
  }

  fn del_item(&mut self, name : &str, index : usize)
  {
      let v : Vec<&str> = name.split('/').collect();
      println!("yooooooooo del : {}", name);

      match v.len() {
          0 => {
              self.remove(index);
          },
          1 => {
              //let index = v[0].parse::<usize>().unwrap();
              self.remove(index);
          },
          _ => {
              let yep : String = v[1..].join("/");
              //let index = v[0].parse::<usize>().unwrap();
              self[index].del_item(yep.as_ref(), index);
          }
      }
  }
}

impl<T:PropertyGet+PropertyRead> PropertyGet for Vec<T> {

  fn get_property_hier(&self, name : &str) -> Option<Box<Any>>
  {
      let v : Vec<&str> = name.split('/').collect();
      match v.len() {
          0 => {None},
          1 => {
              let index = v[0].parse::<usize>().unwrap();
              self[index].get_property()
          },
          _ => {
              let yep : String = v[1..].join("/");
              let index = v[0].parse::<usize>().unwrap();
              self[index].get_property_hier(yep.as_ref())
          }
      }
  }
}

impl<T:PropertyGet> PropertyGet for Box<T>
{
  fn get_property_hier(&self, name : &str) -> Option<Box<Any>>
  {
      (**self).get_property_hier(name)
  }
}




impl PropertyWrite for f64
{
  fn test_set_property(&mut self, value: &Any)
  {
      match value.downcast_ref::<f64>() {
          Some(v) => *self = *v,
          None => {}
      }
  }
}

impl PropertyWrite for String
{
  fn test_set_property(&mut self, value: &Any)
  {
      match value.downcast_ref::<String>() {
          Some(v) => {
              println!("!!!!! setting string to : {}", v);
              *self = v.clone()
          },
          None => {}
      }
  }
}

impl<T:PropertyWrite> PropertyWrite for Box<T>
{
  fn test_set_property(&mut self, value: &Any)
  {
      (**self).test_set_property(value);
  }

  fn test_set_property_hier(&mut self, name : &str, value: &Any)
  {
      (**self).test_set_property_hier(name, value);
  }
}

//impl<T:PropertyWrite+'static+Clone+resource::Create> PropertyWrite for Option<T>
impl<T:PropertyWrite+Any+Clone+resource::Create> PropertyWrite for Option<T>
{
    fn test_set_property(&mut self, value: &Any)
    {
        match value.downcast_ref::<Option<T>>() {
            Some(v) => {
                println!("it is option T");
                *self = v.clone();
                return;
            }
            None => {
                println!("it is not option T");
            }
        }

        ////////////////////////////

      match value.downcast_ref::<T>() {
          Some(t) => {
                println!("it is T");
              *self = Some(t.clone());
              return;
          },
          None => {
                println!("it is not T");
          }
      }

      match value.downcast_ref::<String>() {
          Some(s) => {
              println!("it is string");
              match s.as_ref() {
                  "Some" => {
                      let some : T = resource::Create::create("nonameyet");
                      *self = Some(some);
                  },
                  "None" => *self = None,
                  _ => println!("no such type")
              }
          },
          None => {
              println!("it is not string");
          }
      }
    }

  fn test_set_property_hier(&mut self, name : &str, value: &Any)
  {
      if name == "type" {
          let s = match value.downcast_ref::<String>() {
              Some(v) => v,
              None => return
          };
          match s.as_ref() {
              "Some" => {
                  let some : T = resource::Create::create("nonameyet");
                  *self = Some(some);
              },
              "None" => *self = None,
              _ => println!("no such type")
          }
      }
      else  {
          match *self{
              Some(ref mut v) => v.test_set_property_hier(name, value),
              None => {}
          }
      }
  }

  fn set_property(&mut self, value: WriteValue)
  {
      match value {
          WriteValue::Any(v) => self.test_set_property(&*v),
          WriteValue::None => *self = None,
          WriteValue::Some => {
              let some : T = resource::Create::create("no_name_yet");
              *self = Some(some);
          },
          //_ => {}
      }
  }

  fn set_property_hier(&mut self, name: &str, value: WriteValue)
  {
      match *self{
          Some(ref mut v) => v.set_property_hier(name, value),
          None => {}
      }
  }
}

impl<T:PropertyGet> PropertyGet for Option<T>
{
  fn get_property_hier(& self, name : &str) -> Option<Box<Any>>
  {
      match *self{
          Some(ref v) => v.get_property_hier(name),
          None => None
      }
  }
}


impl<T> PropertyWrite for resource::ResTT<T> where T:Any
{
    fn test_set_property(&mut self, value: &Any)
    {
        match value.downcast_ref::<resource::ResTT<T>>() {
            Some(v) => {
                *self = v.clone();
                return;
            }
            None => {}
        }
    }

  fn test_set_property_hier(&mut self, name : &str, value: &Any)
  {
      println!("property write restt : {}", name);
      if name == "name" {
        match value.downcast_ref::<String>() {
            Some(v) => {
                self.name = v.clone();
                self.resource = resource::ResTest::ResNone;
            }
            None => {}
        }
      }
  }
}


impl PropertyWrite for transform::Orientation
{
  fn test_set_property(&mut self, value: &Any)
  {
      println!("%%%%%%% test set property");
      match value.downcast_ref::<transform::Orientation>() {
          Some(v) => {
              *self = *v;
              return;
          }
          None => {}
      }

      match value.downcast_ref::<String>() {
          Some(s) => {
              match s.as_ref() {
                  "AngleXYZ" => self.to_angle_xyz(),
                  "Quat" => self.to_quat(),
                  _ => println!("no such type")
              }
          },
          None => {}
      }
  }

  fn test_set_property_hier(&mut self, name : &str, value: &Any)
  {
      println!("#######ori ######### name {} ", name);

      if name == "type" {
          let s = match value.downcast_ref::<String>() {
              Some(v) => v,
              None => return
          };
          match s.as_ref() {
              "AngleXYZ" => self.to_angle_xyz(),
              "Quat" => self.to_quat(),
              _ => println!("no such type")
          }
      }
      else  {
          match *self {
              transform::Orientation::AngleXYZ(ref mut v) => v.test_set_property_hier(name, value),
              transform::Orientation::Quat(ref mut v) => v.test_set_property_hier(name, value),
          }
      }
  }
}

impl PropertyGet for transform::Orientation
{
  fn get_property_hier(&self, name : &str) -> Option<Box<Any>>
  {
      match *self {
          transform::Orientation::AngleXYZ(ref v) => v.get_property_hier(name),
          transform::Orientation::Quat(ref v) => v.get_property_hier(name),
      }
  }
}


/*
impl PropertyWrite for transform::Transform
{
  fn test_set_property_hier(&mut self, name : &str, value: &Any)
  {
      let vs = make_vec_from_string(name);

      match vs.len() {
          0 => {},
          1 => {
              match vs[0].as_ref() {
                  "position" => self.position.test_set_property(value),
                  "orientation" => self.orientation.test_set_property(value),
                  _ => println!("no such member")
              }
          },
          _ => {
              let yep = join_string(&vs[1..].to_vec());
              match vs[0].as_ref() {
                  "position" =>
                      self.position.test_set_property_hier(yep.as_ref(), value),
                  "orientation" =>
                      self.orientation.test_set_property_hier(yep.as_ref(), value),
                  _ => println!("no such member")
              }
          }
      }
  }
}

fn make_vec_from_string(s : &str) -> Vec<String>
{
    let v: Vec<&str> = s.split('/').collect();

    let mut vs = Vec::new();
    for i in v.iter()
    {
        vs.push(i.to_string());
    }

    vs
}

fn join_string(path : &Vec<String>) -> String
{
    let mut s = String::new();
    let mut first = true;
    for v in path.iter() {
        if !first {
            s.push('/');
        }
        s.push_str(v.as_ref());
        first = false;
    }

    s
}
*/

macro_rules! property_set_impl(
    ($my_type:ty, [ $($member:ident),+ ]) => (
        impl PropertyWrite for $my_type
        {
            fn test_set_property(&mut self, value: &Any)
            {
                if let Some(v) = value.downcast_ref::<$my_type>() {
                    *self = (*v).clone();
                }
            }

            fn test_set_property_hier(&mut self, name : &str, value: &Any)
            {
                let mut v : Vec<&str> = name.split('/').collect();
                //TODO remove this?
                if v[0] == "object" {
                    v = v[1..].to_vec();
                }

                match v.len() {
                    0 => {},
                    1 => {
                        match v[0] {
                            $(
                                stringify!($member) => self.$member.test_set_property(value),
                                )+
                                _ => println!("1111 no such member, name : {}", v[0])
                        }
                    },
                    _ => {
                        let yep : String = v[1..].join("/");
                        match v[0] {
                            $(
                                stringify!($member) => self.$member.test_set_property_hier(yep.as_ref(),value),
                                )+
                                _ => println!(">>>> 1 , no such member,hier : {}, {}", v[0], name)
                        }
                    }
                }
            }

            fn set_property_hier(&mut self, name : &str, value: WriteValue)
            {
                let mut v : Vec<&str> = name.split('/').collect();
                //TODO remove this?
                if v[0] == "object" {
                    v = v[1..].to_vec();
                }

                match v.len() {
                    0 => {},
                    1 => {
                        match v[0] {
                            $(
                                stringify!($member) => self.$member.set_property(value),
                                )+
                                _ => println!("1111 no such member, name : {}", v[0])
                        }
                    },
                    _ => {
                        let yep : String = v[1..].join("/");
                        match v[0] {
                            $(
                                stringify!($member) => self.$member.set_property_hier(yep.as_ref(),value),
                                )+
                                _ => println!(">>>> 1 , no such member,hier : {}, {}", v[0], name)
                        }
                    }
                }
            }

            fn add_item(&mut self, name : &str, index :usize, value : &Any)
            {
                let mut v : Vec<&str> = name.split('/').collect();
                println!("yooooooooo frommacro : {}", name);

                //TODO remove this?
                if v[0] == "object" {
                    v = v[1..].to_vec();
                }

                match v.len() {
                    0 => {},
                    //1 => {
                    //},
                    _ => {
                        let yep : String = v[1..].join("/");
                        match v[0] {
                            $(
                                stringify!($member) => self.$member.add_item(yep.as_ref(), index, value),
                                )+
                                _ => println!(">>>> 1 , no such member, add_item : {}, {}", v[0], name)
                        }
                    }
                }
            }

            fn del_item(&mut self, name : &str, index :usize)
            {
                let mut v : Vec<&str> = name.split('/').collect();
                println!("yooooooooo frommacro : {}", name);

                //TODO remove this?
                if v[0] == "object" {
                    v = v[1..].to_vec();
                }

                match v.len() {
                    0 => {},
                    //1 => {
                    //},
                    _ => {
                        let yep : String = v[1..].join("/");
                        match v[0] {
                            $(
                                stringify!($member) => self.$member.del_item(yep.as_ref(), index),
                                )+
                                _ => println!(">>>> 1 , no such member, del_item : {}, {}", v[0], name)
                        }
                    }
                }
            }
        }

)
);

property_set_impl!(vec::Vec3,[x,y,z]);
property_set_impl!(vec::Quat,[x,y,z,w]);
//property_set_impl!(mesh_render::MeshRender,[mesh,material]);
//property_set_impl!(armature::MeshRender,[mesh,material]);
property_set_impl!(object::Object,[name,position,orientation,scale,comp_data,comp_lua]);
//property_set_impl!(object::Object,[name,position,orientation,scale]);

macro_rules! property_get_impl(
    ($my_type:ty, [ $($member:ident),+ ]) => (
        impl PropertyGet for $my_type
        {
            fn get_property_hier(&self, name : &str) -> Option<Box<Any>>
            {
                let mut v : Vec<&str> = name.split('/').collect();
                //TODO remove this?
                if v[0] == "object" {
                    v = v[1..].to_vec();
                }

                match v.len() {
                    0 => {None},
                    1 => {
                        match v[0] {
                            $(
                                stringify!($member) => self.$member.get_property(),
                                )+
                                _ => {
                                    println!("1111 no such member, name : {}", v[0]);
                                    None
                                }
                        }
                    },
                    _ => {
                        let yep : String = v[1..].join("/");
                        match v[0] {
                            $(
                                stringify!($member) => self.$member.get_property_hier(yep.as_ref()),
                                )+
                                _ => {
                                    println!("GET >>>> 1 , no such member,hier : {}, {}", v[0], name);
                                    None
                                }
                        }
                    }
                }
            }
        }
)
);

property_get_impl!(vec::Vec3,[x,y,z]);
property_get_impl!(vec::Quat,[x,y,z,w]);
property_get_impl!(resource::ResTT<mesh::Mesh>,[name]);
property_get_impl!(resource::ResTT<material::Material>,[name]);
//property_get_impl!(mesh_render::MeshRender,[mesh,material]);
//property_get_impl!(object::Object,[name,position,orientation,scale]);
property_get_impl!(object::Object,[name,position,orientation,scale,comp_data,comp_lua]);

