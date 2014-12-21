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

macro_rules! new_test(
  ($yo:ident, $member:ident, SString) => (
    $yo.$member.to_string()
    );
  ($yo:ident, $member:ident, Struct) => (
    $yo.$member.clone()
    );
  ($yo:ident, $member:ident, $yep:expr) => (
    $yo.$member
    )
  )

macro_rules! match_get(
  ($yo:ident, $member:ident, $yep:ty, PlainString) => (
      return BoxAny(box $yo.$member.clone() as Box<Any>)
    );
  ($yo:ident, $member:ident, $yep:ty, PlainStruct) => (
      return BoxChrisProperty(box $yo.$member.clone())
    );
  ($yo:ident, $member:ident, $yep:ty, Plain) => (
      return BoxAny(box $yo.$member as Box<Any>)
    );
  ($yo:ident, $member:ident, $yep:ty, Boxed) => (
      return BoxAny(box $yo.$member.clone() as Box<Any>)
      );
  )

macro_rules! match_hier_get(
  ($yo:ident, $member:ident, $yep:ident) => (
      return $yo.$member.get_property_hier($yep)
    )
  )

macro_rules! match_set(
  ($yo:ident, $member:ident, $my_type:ty, $value:ident, Boxed) => (
      match $value.downcast_ref::<$my_type>() {
          Some(v) => *$yo.$member = *v,
          None => {}
      }
      );
  ($yo:ident, $member:ident, $my_type:ty, $value:ident, Plain) => (
      match $value.downcast_ref::<$my_type>() {
          Some(v) => $yo.$member = *v,
          None => {}
      }
      );
  ($yo:ident, $member:ident, $my_type:ty, $value:ident, PlainStruct) => (
      match $value.downcast_ref::<$my_type>() {
          Some(v) => $yo.$member = *v,
          None => {}
      }
      );
  ($yo:ident, $member:ident, $my_type:ty, $value:ident, PlainString) => (
      match $value.downcast_ref::<$my_type>() {
          Some(v) => $yo.$member = v.clone(),
          None => {}
      }
      );
  )

macro_rules! match_hier_set(
  ($yo:ident, $member:ident, $yep:ident, $value:ident) => (
      $yo.$member.set_property_hier($yep, $value)
    )
  )

pub enum AllocStyle
{
    Plain,
    Boxed,
    PlainString,
    PlainStruct
}

/*
pub macro_rules! chris_property_impl(
    ($my_type:ty, [ $($member:ident,$mytype:ty,$alloctype:ident)|+ ]) => ( 

      impl ChrisProperty for $my_type
      {
          /*
        fn fields(&self) -> Box<[String]>
        {
          return box[
          $(
            stringify!($member).to_string(),
           )+
          ];

        }
        */

        fn get_property(&self, name: &str) -> ChrisValue
        {
            match name {
          $(
            stringify!($member) => match_get!(self, $member, $mytype, $alloctype),
           )+
              _ => return ChrisNone
            };
        }

        fn set_property(&mut self, name: &str, value: &Any)
        {
            match name {
          $(
              //TODO
            stringify!($member) => {
              match_set!(self, $member, $mytype, value, $alloctype)
            },

           )+
            _ => {}
            }
        }

        fn get_property_hier(&self, names: Vec<String>) -> ChrisValue
        {
            match names.len() {
                0 => return ChrisNone,
                1 => return self.get_property(names[0].as_slice()),
                _ => {
                    let yep = names.tail().to_vec();
                    match names[0].as_slice() {
                        $(
                            stringify!($member) => match_hier_get!(self, $member, yep),
                        )+
                        _ => {}
                    }
                }
            }

            return ChrisNone;
        }

        fn set_property_hier(&mut self, names: Vec<String>, value: &Any)
        {
            match names.len() {
                0 => return,
                1 => self.set_property(names[0].as_slice(),value),
                _ => {
                    let yep = names.tail().to_vec();
                    match names[0].as_slice() {
                        //"position" => { self.position.set_property_hier(yep, value);},
                        //"boxpos" => { self.boxpos.set_property_hier(yep, value);},
                        $(
                            stringify!($member) => match_hier_set!(self, $member, yep, value),
                        )+
                        _ => {}
                    }
                }
            }
        }

     }
  );
)
*/

/*
chris_property_impl!(Chris,
                     [x,f64,Plain|
                     y,f64,Plain|
                     z,f64,Plain|
                     position,vec::Vec3,Plain|
                     boxpos,vec::Vec3,Boxed])
//chris_property_impl!(Chris, [x,f64,Plain|y,f64,Plain|z,f64,Plain|position,vec::Vec3,Plain])
//chris_property_impl!(Chris, [x,f64|y,f64|z,f64])

chris_property_impl!(vec::Quat,
                     [x,f64,Plain|
                     y,f64,Plain|
                     z,f64,Plain|
                     w,f64,Plain])

chris_property_impl!(vec::Vec3,
                     [x,f64,Plain|
                     y,f64,Plain|
                     z,f64,Plain])

chris_property_impl!(transform::Transform,
                     [position,vec::Vec3,PlainStruct])
                     //[position,vec::Vec3,PlainStruct|
                     //orientation,transform::Orientation,PlainStruct])

chris_property_impl!(object::Object,
                     [name,String,PlainString
                     |position,vec::Vec3,PlainStruct
                     |orientation,vec::Quat,PlainStruct
                     |scale,vec::Vec3,PlainStruct
                     |transform,transform::Transform,Boxed
                     ])

pub fn find_property(p : &ChrisProperty, path : Vec<String>) -> 
Option<Box<ChrisProperty>>
{
    match p.get_property_hier(path) {
        ChrisNone => {return None;},
        BoxChrisProperty(bp) => 
        {
            return Some(bp);
        },
        _ => {
            println!("problem with find property : must be box any...");
        }
    }

    return None;
}
*/


/*
impl ChrisProperty for vec::Quat
{
  fn fields(&self) -> Box<[String]> 
  {
      return box[
          String::from_str("x"),
          String::from_str("y"),
          String::from_str("z"),
      ];
  }

  fn get_property(&self, name: &str) -> ChrisValue
  {
      let deg = self.to_euler_deg();
      let v = match name {
          "x" => box deg.x,// as Box<Any>,
          "y" => box deg.y,// as Box<Any>,
          "z" => box deg.z,// as Box<Any>,
          _ => return ChrisNone
      };

      BoxAny(v as Box<Any>)
  }

  //TODO set_property return a vec of properties that were changed,
  // execpt the one we sent...
  fn set_property(&mut self, name: &str, value: &Any)
  {
      let mut deg = self.to_euler_deg();
      println!("deg start : {}", deg);

      let f = match value.downcast_ref::<f64>() {
          Some(v) => v,
          None => return
      };


      let mut q = match name {
          "x" => {
              //deg.x = *f;
              let diff = *f - deg.x;
              //println!("{} - {} = diff : {}", *f, deg.x, diff);
              vec::Quat::new_angles_deg(&vec::Vec3::new(diff,0f64,0f64))
          },
          "y" => {
              //deg.y = *f;
              let diff = *f - deg.y;
              vec::Quat::new_angles_deg(&vec::Vec3::new(0f64,diff,0f64))
          },
          "z" => {
              //deg.z = *f;
              let diff = *f - deg.z;
              vec::Quat::new_angles_deg(&vec::Vec3::new(0f64,0f64,diff))
          }
          _ => return
      };

      // *self = vec::Quat::new_angles_deg(&deg);
      *self = *self * q;

      let mut deg = self.to_euler_deg();
      println!("deg end : {}", deg);
  }
}
*/

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

