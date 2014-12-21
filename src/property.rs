use object;
use vec;
use std::any::{Any, AnyRefExt};
use std::f64::consts;
use transform;

//log_syntax!()
//trace_macros!(true)

#[deriving(Decodable, Encodable, Clone)]
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

pub trait ChrisTest
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

impl ChrisTest for f64
{
  fn test_set_property(&mut self, value: &Any)
  {
      match value.downcast_ref::<f64>() {
          Some(v) => *self = *v,
          None => {}
      }
  }
}

impl ChrisTest for String
{
  fn test_set_property(&mut self, value: &Any)
  {
      match value.downcast_ref::<String>() {
          Some(v) => *self = v.clone(),
          None => {}
      }
  }
}

impl<T:ChrisTest> ChrisTest for Box<T>
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

impl ChrisTest for vec::Vec3
{
  fn test_set_property(&mut self, value: &Any)
  {
      match value.downcast_ref::<vec::Vec3>() {
          Some(v) => *self = *v,
          None => {}
      }
  }

  fn test_set_property_hier(&mut self, name : &str, value: &Any)
  {
      match name {
          "x" => self.x.test_set_property(value),
          "y" => self.y.test_set_property(value),
          "z" => self.z.test_set_property(value),
          _ => println!("no such member")
      }
  }
}

impl ChrisTest for vec::Quat
{
  fn test_set_property(&mut self, value: &Any)
  {
      match value.downcast_ref::<vec::Quat>() {
          Some(v) => *self = *v,
          None => {}
      }
  }

  fn test_set_property_hier(&mut self, name : &str, value: &Any)
  {
      match name {
          "x" => self.x.test_set_property(value),
          "y" => self.y.test_set_property(value),
          "z" => self.z.test_set_property(value),
          "w" => self.w.test_set_property(value),
          _ => println!("no such member")
      }
  }
}

impl ChrisTest for transform::Orientation
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
              transform::AngleXYZ(ref mut v) => v.test_set_property_hier(name, value),
              transform::Quat(ref mut v) => v.test_set_property_hier(name, value),
          }
      }
  }
}

impl ChrisTest for transform::Transform
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

impl ChrisTest for object::Object
{
  fn test_set_property_hier(&mut self, name : &str, value: &Any)
  {
      //TODO remove
      println!("REMOVE THIS");
      let mut vs = make_vec_from_string(name);
      if vs.len() > 0 {
          if vs[0].as_slice() == "object" {
              vs = vs.tail().to_vec();
          }
      }

      match vs.len() {
          0 => {},
          1 => {
              match vs[0].as_slice() {
                  "position" => self.position.test_set_property(value),
                  "orientation" => self.orientation.test_set_property(value),
                  "transform" => self.transform.test_set_property(value),
                  _ => println!("no such member : {} ", vs[0])
              }
          },
          _ => {
              let yep = join_string(&vs.tail().to_vec());
              match vs[0].as_slice() {
                  "position" => 
                      self.position.test_set_property_hier(yep.as_slice(), value),
                  "orientation" =>
                      self.orientation.test_set_property_hier(yep.as_slice(), value),
                  "transform" => {
                      println!("yes come here");
                      self.transform.test_set_property_hier(yep.as_slice(), value);
                  }
                  _ => println!("no such member,hier : {}", vs[0])
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

