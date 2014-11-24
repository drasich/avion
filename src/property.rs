//extern crate libc;
//extern crate serialize;
//use libc::size_t;
//use std::f64;
//use std::io;
//use serialize::{json, Encodable, Decodable};
//use std::str;
use object;
use vec;
use std::any::{Any, AnyRefExt};

//log_syntax!()
//trace_macros!(true)

pub trait PropertyYep
{
    fn transform(&self) -> PropertyType;
}

pub trait TProperty {
  //fn fields(&self) -> ~[u8];
  fn fields(&self) -> Box<[String]>;
  //fn get_type(&self, name: &[u8]) -> u8;
  fn get_property(&self, name: &str) -> PropertyType;
  fn set_property(&mut self, name: &str, value: &PropertyType);
  fn set_easy(&mut self, name: &str, value: &PropertyYep);
}

pub enum ChrisEnum
{
    cInt,
    cFloat,
    cStruct,
    cBox,
    cVec,
    cHashMap,
    cList
}

pub enum ChrisValue
{
    ChrisNone,
    BoxAny(Box<Any>),
    BoxChrisProperty(Box<ChrisProperty+'static>)
}


pub trait ChrisProperty {
  //fn cfields(&self) -> Box<[String]>;
  fn cfields(&self) -> Box<[String]>
  {
      return box [];
  }

  fn cget_property(&self, name: &str) -> ChrisValue
  //fn cget_property(&self, name: &str) -> Option<(Box<Any>, ChrisEnum)>
  {
      return ChrisNone;
  }
  fn cget_property_hier(&self, name: Vec<String>) -> ChrisValue
  {
      match name.len() {
          0 => ChrisNone,
          _ => self.cget_property(name[0].as_slice())
      }
  }
  fn cset_property(&mut self, name: &str, value: &Any)
  {
  }
  fn cset_property_hier(&mut self, name: Vec<String>, value: &Any)
  {
      match name.len() {
          0 => {},
          _ => self.cset_property(name[0].as_slice(), value)
      };
  }
}

/*
impl<T: 'static> ChrisProperty for T {
  fn cfields(&self) -> Box<[String]>
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


impl ChrisProperty for vec::Vec3
{
  fn cfields(&self) -> Box<[String]> 
  {
      return box[
          String::from_str("x"),
          String::from_str("y"),
          String::from_str("z"),
      ];
  }

  fn cget_property(&self, name: &str) -> ChrisValue
  {
      let v = match name {
          "x" => box self.x,// as Box<Any>,
          "y" => box self.y,// as Box<Any>,
          "z" => box self.z,// as Box<Any>,
          _ => return ChrisNone
      };

      BoxAny(v as Box<Any>)
  }

  fn cset_property(&mut self, name: &str, value: &Any)
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

/*
impl ChrisProperty for Chris
{
  fn cfields(&self) -> Box<[String]> 
  {
      return box[
          String::from_str("x"),
          String::from_str("y"),
          String::from_str("z"),
          String::from_str("position"),
      ];
  }

  fn cget_property(&self, name: &str) -> Option<Box<Any>>
  {
      let v : Box<Any> = match name {
          "x" => box self.x as Box<Any>,
          "y" => box self.y as Box<Any>,
          "z" => box self.z as Box<Any>,
          "position" => box self.position as Box<Any>,
          "boxpos" => box self.boxpos.clone() as Box<Any>,
          _ => return None
      };

      Some(v)
  }

  fn cget_property_hier(&self, names: Vec<String>) -> Option<Box<Any>>
  {
      match names.len() {
          0 => return None,
          1 => return self.cget_property(names[0].as_slice()),
          _ => {
              let yep = names.tail().to_vec();
              match names[0].as_slice() {
                  "position" => { return self.position.cget_property_hier(yep);},
                  "boxpos" => { return self.boxpos.cget_property_hier(yep);},
                  _ => {}
              }
          }
      }

      return None;
  }

  fn cset_property(&mut self, name: &str, value: &Any)
  {
      match name {
          "x" => {
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
          },
          "position" => {
              match value.downcast_ref::<vec::Vec3>() {
                  Some(v) => self.position = *v,
                  None => {}
              }
          },
          "boxpos" => {
              match value.downcast_ref::<vec::Vec3>() {
                  Some(v) => *self.boxpos = *v,
                  None => {}
              }
          }
          _ => {}
      }

  }

  fn cset_property_hier(&mut self, names: Vec<String>, value: &Any)
  {
      match names.len() {
          0 => return,
          1 => self.cset_property(names[0].as_slice(),value),
          _ => {
              let yep = names.tail().to_vec();
              match names[0].as_slice() {
                  "position" => { self.position.cset_property_hier(yep, value);},
                  "boxpos" => { self.boxpos.cset_property_hier(yep, value);},
                  _ => {}
              }
          }
      }
    }
}
*/


pub enum PropertyType {
  Int(i32),
  Float(f64),
  SString(String),
  Struct(Box<TProperty +'static>),
  //AnyStruct(Any),
  //NormalStruct(&'static TProperty+'static),
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



pub macro_rules! property_impl(
    //($my_type:ty, [ $($member:ident,$mytype:ty)|+ ]) => ( // invoke it like `(input_5 SpecialE)`
    ($my_type:ty, [ $($member:ident,$mytype:ident)|+ ]) => ( // invoke it like `(input_5 SpecialE)`

      impl TProperty for $my_type
      {
        fn fields(&self) -> Box<[String]>
        {
          return box[
          $(
            stringify!($member).to_string(),
           )+
          ];

        }

        fn get_property(&self, name: &str) -> PropertyType
        {
          $(
            if name == stringify!($member)
            {
              //return $mytype(self.$member)
              return $mytype(new_test!(self, $member, $mytype))
            }
           )+

          else {
            Float(0.0)
          }
        }

        fn set_property(&mut self, name: &str, value: &PropertyType)
        {
          //*
          $(
            if name == stringify!($member)
            {
              new_test_set!(self, $member, $mytype, value, name)
            /*
              match value {
                //&Float(v) => {
                &$mytype(v) => {
                  self.$member = v;
                }
                _ => {
                  println!("cant set {:?} to {}, because it is a {}", value, name, stringify!($mytype));
                }
              }
                */

              return;
            }
           )+
           //*/
        }
        fn set_easy(&mut self, name : &str, value: &PropertyYep)
        {
            self.set_property(name, &value.transform());
        }
     }
  );
)


    /*
#[deriving(Clone)]
pub struct Vec3
{
  x : f64,
  y : f64,
  z : f64
}

#[deriving(Clone)]
pub struct Quat
{
  x : f64,
  y : f64,
  z : f64,
  w : f64,
  t : i32,
  v : Box<Vec3>,
  txt : String
}
*/

//property_impl!(Vec3, [x,Float|y,Float|z,Float])
property_impl!(vec::Vec3, [x,Float|y,Float|z,Float])
//property_impl!(Quat, [x,Float|y,Float|z,Float|t,Int|txt,SString|v,Struct])
//property_impl!(object::Object, [name,SString|position,NormalStruct])
property_impl!(object::Object, [name,SString])

pub fn print_pt(pt : PropertyType)
{
  match pt {
    Int(i) => println!("my int : {}", i),
    SString(s) => println!("my string : {}", s),
    Float(f) => println!("my float : {}", f),
    Struct(ptt) => {
      println!("yo man");
      for f in ptt.fields().iter() {

        //let p = t.get_property("name");
        let p = ptt.get_property(f.as_slice());
        print_pt(p);
      }
    },
    //_ => {}
  }
}

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
      return $yo.$member.cget_property_hier($yep)
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
      $yo.$member.cset_property_hier($yep, $value)
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
        fn cfields(&self) -> Box<[String]>
        {
          return box[
          $(
            stringify!($member).to_string(),
           )+
          ];

        }

        fn cget_property(&self, name: &str) -> ChrisValue
        {
            let v : Box<Any> = match name {
          $(
            stringify!($member) => match_get!(self, $member, $mytype, $alloctype),
           )+
              _ => return ChrisNone
            };
        }

        fn cset_property(&mut self, name: &str, value: &Any)
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

        fn cget_property_hier(&self, names: Vec<String>) -> ChrisValue
        {
            match names.len() {
                0 => return ChrisNone,
                1 => return self.cget_property(names[0].as_slice()),
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

        fn cset_property_hier(&mut self, names: Vec<String>, value: &Any)
        {
            match names.len() {
                0 => return,
                1 => self.cset_property(names[0].as_slice(),value),
                _ => {
                    let yep = names.tail().to_vec();
                    match names[0].as_slice() {
                        //"position" => { self.position.cset_property_hier(yep, value);},
                        //"boxpos" => { self.boxpos.cset_property_hier(yep, value);},
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

chris_property_impl!(vec::Quat,
                     [x,f64,Plain|
                     y,f64,Plain|
                     z,f64,Plain|
                     w,f64,Plain])

chris_property_impl!(object::Object,
                     [name,String,PlainString
                     |position,vec::Vec3,PlainStruct
                     |orientation,vec::Quat,PlainStruct
                     |scale,vec::Vec3,PlainStruct
                     ])

pub fn find_property(p : &ChrisProperty, path : Vec<String>) -> 
Option<Box<ChrisProperty>>
{
    match p.cget_property_hier(path) {
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

