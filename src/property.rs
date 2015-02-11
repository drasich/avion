use object;
use vec;
use std::any::{Any};//, AnyRefExt};
use std::f64::consts;
use transform;
use mesh_render;
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
          Some(v) => *self = v.clone(),
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

impl<T:PropertyWrite+'static+Clone> PropertyWrite for Option<T>
{
    fn test_set_property(&mut self, value: &Any)
    {
        match value.downcast_ref::<Option<T>>() {
            Some(v) => {
                *self = v.clone();
                return;
            }
            None => {}
        }

        match value.downcast_ref::<T>() {
            Some(t) => {
                *self = Some(t.clone())
            },
            None => {}
        }
    }

  fn test_set_property_hier(&mut self, name : &str, value: &Any)
  {
      match *self{
          Some(ref mut v) => v.test_set_property_hier(name, value),
          None => {}
      }
  }
}

impl PropertyWrite for transform::Orientation
{
  fn test_set_property(&mut self, value: &Any)
  {
      match value.downcast_ref::<transform::Orientation>() {
          Some(v) => {
              *self = *v;
              return;
          }
          None => {}
      }

      match value.downcast_ref::<String>() {
          Some(s) => {
              match s.as_slice() {
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
      if name == "type" {
          let s = match value.downcast_ref::<String>() {
              Some(v) => v,
              None => return
          };
          match s.as_slice() {
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

/*
impl PropertyWrite for transform::Transform
{
  fn test_set_property_hier(&mut self, name : &str, value: &Any)
  {
      let vs = make_vec_from_string(name);

      match vs.len() {
          0 => {},
          1 => {
              match vs[0].as_slice() {
                  "position" => self.position.test_set_property(value),
                  "orientation" => self.orientation.test_set_property(value),
                  _ => println!("no such member")
              }
          },
          _ => {
              let yep = join_string(&vs.tail().to_vec());
              match vs[0].as_slice() {
                  "position" => 
                      self.position.test_set_property_hier(yep.as_slice(), value),
                  "orientation" =>
                      self.orientation.test_set_property_hier(yep.as_slice(), value),
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
        s.push_str(v.as_slice());
        first = false;
    }

    s
}
*/

pub macro_rules! property_test_impl(
    ($my_type:ty, [ $($member:ident),+ ]) => ( 
        impl PropertyWrite for $my_type
        {
            fn test_set_property(&mut self, value: &Any)
            {
                match value.downcast_ref::<$my_type>() {
                    Some(v) => *self = (*v).clone(),
                    None => {}
                }
            }

            fn test_set_property_hier(&mut self, name : &str, value: &Any)
            {
                let mut v : Vec<&str> = name.split('/').collect();
                //TODO remove this?
                if v[0] == "object" {
                    v = v.tail().to_vec();
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
                        let yep : String = v.tail().connect("/");
                        match v[0] {
                            $(
                                stringify!($member) => self.$member.test_set_property_hier(yep.as_slice(),value),
                                )+
                                _ => println!(">>>> 1 , no such member,hier : {}, {}", v[0], name)
                        }
                    }
                }
            }
        }
)
);

property_test_impl!(vec::Vec3,[x,y,z]);
property_test_impl!(vec::Quat,[x,y,z,w]);
property_test_impl!(mesh_render::MeshRender,[mesh,material]);
property_test_impl!(resource::ResTT<mesh::Mesh>,[name]);
property_test_impl!(resource::ResTT<material::Material>,[name]);
property_test_impl!(object::Object,[name,position,orientation,scale,mesh_render]);
