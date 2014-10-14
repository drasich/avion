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

//log_syntax!()
//trace_macros!(true)

macro_rules! new_test(
  ($yo:ident, $member:ident, String) => (
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
  ($member:ident, String, $value:ident) => (
    match $value {
    &String(ref s) => {
    self.$member = s.to_owned()
    }
    _ => {
      println!("cant set {:?} to {}, because it is a String", value, name);
    }
    }
    );
  ($member:ident, Struct, $value:ident) => (
    match $value {
    &Struct(ref s) => {
    for p in s.fields().iter() {
      self.$member.set_property(p.to_owned(),& s.get_property(p.to_owned()));
    }
    }
    _ => {
      println!("cant set {:?} to {}, because it is a Struct", value, name);
    }
    }
    );
  ($member:ident, $yep:ident, $value:ident) => (
    match $value {
    &$yep(f) => {
    self.$member = f;
    }
    _ => {
      println!("cant set {:?} to {}, because it is a {}", value, name, stringify!($yep));
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


macro_rules! property_impl(
    //($my_type:ty, [ $($member:ident,$mytype:ty)|+ ]) => ( // invoke it like `(input_5 SpecialE)`
    ($my_type:ty, [ $($member:ident,$mytype:ident)|+ ]) => ( // invoke it like `(input_5 SpecialE)`

      impl Property for $my_type
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
              new_test_set!($member, $mytype, value)
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
  //txt : ~str
}

property_impl!(Vec3, [x,Float|y,Float|z,Float])
//property_impl!(Quat, [x,Float|y,Float|z,Float|t,Int|txt,String])
property_impl!(Quat, [x,Float|y,Float|z,Float|t,Int|txt,String|v,Struct])
//property_impl!(Quat, [x,Float|y,Float|z,Float|t,Int])
//property_impl!(Quat, [x|y])


pub struct TestStruct {
  id: i32,
  //name : ~str,
  speed : f64,
  position : Box<Vec3>
}

enum PropertyType {
  Int(i32),
  Float(f64),
  String(String),
  Struct(Box<Property>),
}

pub struct Floata<'r>
{
  value : &'r f64,
}

pub struct Inta<'r>
{
  value : &'r i32,
}

pub struct Structa<'r>
{
  value : &'r Box<Property>,
}






trait Property {
  //fn fields(&self) -> ~[u8];
  fn fields(&self) -> Box<[String]>;
  //fn get_type(&self, name: &[u8]) -> u8;
  fn get_property(&self, name: &str) -> PropertyType;
  fn set_property(&mut self, name: &str, value: &PropertyType);
}

/*
impl Property for Vec3
{
  fn fields(&self) -> ~[~str]
  {
    return box ["x".to_owned(), "y".to_owned(), "z".to_owned()];
  }

  fn get_property(&self, name: &str) -> PropertyType
  {
    if name == "x" {
      Float(self.x)
    }
    else if name == "y" {
      Float(self.y)
    }
    else {
      Float(self.z)
    }
  }

  fn set_property(&mut self, name: &str, value: &PropertyType)
  {
    match value {
      &Float(f) => {
        if name == "x" {
          self.x = f;
        }
        else if name == "y" {
          self.y = f;
        }
        else if name == "z" {
          self.z = f;
        }
      }
      _ => {}
    }

  }
  
}
*/

impl Property for TestStruct
{
  fn fields(&self) -> Box<[String]>
  {
    //return ["id", "name", "speed", "position"];
    return box ["id".to_owned(), "name".to_owned(), "speed".to_owned(), "position".to_owned()];
  }

  fn get_property(&self, name: &str) -> PropertyType
  {
    if name == "id" {
      Int(self.id)
    }
    else if name == "name" {
      String(self.name.to_owned())
    }
    else if name == "speed" {
      Float(self.speed)
    }
    else if name == "position" {
      let c = self.position.clone();
      return Struct(c);
    }
    else {
      Float(self.speed)
    }
  }

  fn set_property(&mut self, name: &str, value: &PropertyType)
  {
    match value {
      &Float(f) => {
        if name == "speed" {
          self.speed = f;
        }
      },
      &String(ref s) => {
          self.name = s.to_owned();
      }
      //*
      &Struct(ref s) => {
        if name == "position" {
          println!("pos {:?} ", s)
            for p in s.fields().iter() {
              self.position.set_property(p.to_owned(),& s.get_property(p.to_owned()));
            }
          //self.position.set_property(name, value);
          //println!("pos {:?} ", s as ~Vec3)
          //self.position = s as ~Vec3;
        }
      }
      //  */
      _ => {}
    }

  }
}

//property_impl!(TestStruct, [id,Int|name,String|speed,Float])
//property_impl!(TestStruct, [id,Int|name,String|speed,Float|position,Struct])

fn print_pt(pt : PropertyType)
{
  match pt {
    /*
    Int(i) => println!("my int : {}", i),
    String(s) => println!("my string : {}", s),
    Float(f) => println!("my float : {}", f),
    Struct(ptt) => {
      println!("yo man");
      for f in ptt.fields().iter() {

        //let p = t.get_property("name");
        let p = ptt.get_property(f.to_owned());
        print_pt(p);
      }
    },
    */
    _ => {}
  }
}


/*
  macro_rules! early_return(
    ($inp:expr, [ $($sp:ident)|+ ]) => (
    ($inp:expr, [ $($sp:ident)|+ ]) => (
      match $inp {
        $(
          $sp(x) => { return x; }
         )+
          _ => {}
      }
      );
    )
    */

macro_rules! early_return(
  ($inp:expr, [ $($sp:ident)|+ ]) => ( // invoke it like `(input_5 SpecialE)`
    match $inp {
      //$sp(x) => { x; }
      $(
        $sp(x) => { x; }
       )+
      _ => {}
    }
    );
  )


fn main() {
  //let chris = 5;

  let mut t = TestStruct{ id:5, name:"dance".to_owned(), speed:45.0, position: box Vec3{x:1.2,y:3.4,z:5.6}};
  t.set_property("speed", &Float(999.0));
  t.set_property("position", &Struct(box Vec3{x:-11.0, y:999.0, z:-22.0}));
  println!(" chris {:?} ", t.fields());

  for f in t.fields().iter() {

    //let p = t.get_property("name");
    let p = t.get_property(f.to_owned());
    print_pt(p);

  }

  //let mut q = Quat{x:1.0, y:999.0, z:22.0, w:123.0,t:44, txt:"yepyep".to_owned()};
  let mut q = Quat{x:1.0, y:999.0, z:22.0, w:123.0,t:44,v:box Vec3{x:4.0,y:6.7,z:8.9}, txt:"yepyep".to_owned()};
  //println!(" quat {:?} ", q.fields());
  println!(" quat {:?} ", q.get_property("z"));
  q.set_property("z", &Float(345.0));
  q.set_property("z", &Int(345));
  println!(" after quat {:?} ", q.get_property("z"));
  println!(" striiiiiinggggggg {:?} ", q.get_property("txt"));
  q.set_property("txt", &String("fdfdfdf".to_owned()));
  println!(" striiiiiinggggggg after {:?} ", q.get_property("txt"));

  q.set_property("t", &Int(345));
  println!(" after quat int {:?} ", q.get_property("t"));

  let test = 4.5;

  let floata = Floata{value:& test};
  println!(" int {:?} ", Int);

// ...
//early_return!(input_1, [SpecialA|SpecialC|SpecialD]);
// ...
//early_return!(input_2, [SpecialB]);


  //let p = Float(5.0);
//early_return!(p, [Float]);
  //let myf = early_return!(p, [Float]);

  /*
  let test =
    match p {
      Float(x) => { x; }
      _ => {}
    };
    */


}


