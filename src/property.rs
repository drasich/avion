use object;
use vec;
use std::any::{Any, AnyRefExt};
use std::f64::consts;

//log_syntax!()
//trace_macros!(true)

pub enum ChrisValue
{
    ChrisNone,
    BoxAny(Box<Any>),
    BoxChrisProperty(Box<ChrisProperty+'static>)
}

/*
pub enum teststtst
{
    Yep,
    Yop(f64, i32),
    BoxAnytest(Box<Any>),
    BoxChrisPropertytest(Box<ChrisProperty+'static>),
    BoxCouple(Box<Any>, Box<ChrisProperty+'static>)
}
*/


pub trait ChrisProperty {
  fn fields(&self) -> Box<[String]>
  {
      return box [];
  }

  fn get_property(&self, name: &str) -> ChrisValue
  {
      return ChrisNone;
  }
  fn get_property_hier(&self, name: Vec<String>) -> ChrisValue
  {
      match name.len() {
          0 => ChrisNone,
          _ => self.get_property(name[0].as_slice())
      }
  }
  fn set_property(&mut self, name: &str, value: &Any)
  {
  }
  fn set_property_hier(&mut self, name: Vec<String>, value: &Any)
  {
      match name.len() {
          0 => {},
          _ => self.set_property(name[0].as_slice(), value)
      };
  }
}

/*
impl<T: 'static> ChrisProperty for T {
  fn fields(&self) -> Box<[String]>
  {
      return box [];
  }
}
*/
impl ChrisProperty for f64 {
}

impl ChrisProperty for String {
}

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


/*
impl ChrisProperty for vec::Vec3
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
      let v = match name {
          "x" => box self.x,// as Box<Any>,
          "y" => box self.y,// as Box<Any>,
          "z" => box self.z,// as Box<Any>,
          _ => return ChrisNone
      };

      BoxAny(v as Box<Any>)
  }

  fn set_property(&mut self, name: &str, value: &Any)
  {
      match name {
          "x" => {
              //if self.x.get_type_id() == value.get_type_id {
              //}
              match value.downcast_ref::<f64>() {
                  Some(v) => self.x = *v,
                  None => {}

              }
          },
          "y" => {
              match value.downcast_ref::<f64>() {
                  Some(v) => self.y = *v,
                  None => {}
              }
          },
          "z" => {
              match value.downcast_ref::<f64>() {
                  Some(v) => self.z = *v,
                  None => {}
              }
          }
          _ => {}
      }
  }
}
*/

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

macro_rules! new_test_set(
  ($sself:ident, $member:ident, SString, $value:ident, $name:ident) => (
    match $value {
    &SString(ref s) => {
    $sself.$member = s.to_string()
    }
    _ => {
      //println!("cant set {} to {}, because it is a String", $value, $name);
      println!("cant set property(TODO) to {}, because it is a String", $name);
    }
    }
    );
  ($sself:ident, $member:ident, Struct, $value:ident, $name:ident) => (
    match $value {
    &Struct(ref s) => {
    for p in s.fields().iter() {
      $sself.$member.set_property(p.as_slice(),& s.get_property(p.as_slice()));
    }
    }
    _ => {
      println!("cant set {:?} to {}, because it is a Struct", $value, $name);
    }
    }
    );
  ($sself:ident, $member:ident, $yep:ident, $value:ident, $name:ident) => (
    match $value {
    &$yep(f) => {
    $sself.$member = f;
    }
    _ => {
      //println!("cant set {} to {}, because it is a {}", $value, $name, stringify!($yep));
      println!("cant set property(TODO) to {}, because it is a {}", $name, stringify!($yep));
    }
    }
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

pub macro_rules! chris_property_impl(
    ($my_type:ty, [ $($member:ident,$mytype:ty,$alloctype:ident)|+ ]) => ( 

      impl ChrisProperty for $my_type
      {
        fn fields(&self) -> Box<[String]>
        {
          return box[
          $(
            stringify!($member).to_string(),
           )+
          ];

        }

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

chris_property_impl!(Chris,
                     [x,f64,Plain|
                     y,f64,Plain|
                     z,f64,Plain|
                     position,vec::Vec3,Plain|
                     boxpos,vec::Vec3,Boxed])
//chris_property_impl!(Chris, [x,f64,Plain|y,f64,Plain|z,f64,Plain|position,vec::Vec3,Plain])
//chris_property_impl!(Chris, [x,f64|y,f64|z,f64])

/*
chris_property_impl!(vec::Quat,
                     [x,f64,Plain|
                     y,f64,Plain|
                     z,f64,Plain|
                     w,f64,Plain])
                     */

chris_property_impl!(vec::Vec3,
                     [x,f64,Plain|
                     y,f64,Plain|
                     z,f64,Plain])

chris_property_impl!(object::Object,
                     [name,String,PlainString
                     |position,vec::Vec3,PlainStruct
                     |orientation,vec::Quat,PlainStruct
                     |scale,vec::Vec3,PlainStruct
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

      //*self = vec::Quat::new_angles_deg(&deg);
      *self = *self * q;

      let mut deg = self.to_euler_deg();
      println!("deg end : {}", deg);
  }
}
