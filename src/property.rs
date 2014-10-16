#![feature(macro_rules)]
#![feature(log_syntax)]
#![feature(trace_macros)]

//extern crate libc;
//extern crate serialize;
//use libc::size_t;
//use std::f64;
//use std::io;
//use serialize::{json, Encodable, Decodable};
use std::str;
use object;

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

pub enum PropertyType {
  Int(i32),
  Float(f64),
  SString(String),
  Struct(Box<TProperty +'static>),
}


macro_rules! new_test(
  ($yo:ident, $member:ident, SString) => (
    $yo.$member.to_owned()
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
    $sself.$member = s.to_owned()
    }
    _ => {
      println!("cant set {:?} to {}, because it is a String", $value, $name);
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
      println!("cant set {:?} to {}, because it is a {}", $value, $name, stringify!($yep));
    }
    }
    )
  )

/*
      &Float(f) => {
        if name == "speed" {
          self.speed = f;
        }
      },
      &String(ref s) => {
          self.name = s.to_owned();
      }
      
      &Struct(ref s) => {
        if name == "position" {
          println!("pos {:?} ", s)
            for p in s.fields().iter() {
              self.position.set_property(p.to_owned(),& s.get_property(p.to_owned()));
            }
            */


pub macro_rules! property_impl(
    //($my_type:ty, [ $($member:ident,$mytype:ty)|+ ]) => ( // invoke it like `(input_5 SpecialE)`
    ($my_type:ty, [ $($member:ident,$mytype:ident)|+ ]) => ( // invoke it like `(input_5 SpecialE)`

      impl TProperty for $my_type
      {
        fn fields(&self) -> Box<[String]>
        {
          //return box ["xaa".to_owned(), "y".to_owned(), "z".to_owned()];
          //*
          return box[
          $(
            stringify!($member).to_owned(),
           )+
          ];
          //*/

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

property_impl!(Vec3, [x,Float|y,Float|z,Float])
property_impl!(Quat, [x,Float|y,Float|z,Float|t,Int|txt,SString|v,Struct])
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



