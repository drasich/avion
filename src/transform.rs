use vec;
use property;
use ui;
use std::any::{Any, AnyRefExt};

#[deriving(Decodable, Encodable, Clone)]
pub enum Orientation
{
    AngleXYZ(vec::Vec3),
    Quat(vec::Quat)
}



#[deriving(Decodable, Encodable, Clone)]
pub struct Transform {
    pub position : vec::Vec3, 
    pub orientation : Orientation
}

impl Transform
{
    pub fn new() -> Transform
    {
        Transform {
            position : vec::Vec3::zero(),
            orientation : Quat(vec::Quat::identity())
        }
    }
}

impl property::ChrisProperty for Orientation
{
  //TODO set_property return a vec of properties that were changed,
  // execpt the one we sent...
  fn set_property(&mut self, name: &str, value: &Any)
  {
      match *self {
          AngleXYZ(ref mut v) => v.set_property(name, value),
          Quat(ref mut q) => q.set_property(name, value),
          
      };
  }
}

impl ui::PropertyShow for Orientation {

    fn create_widget(&self, property : &mut ui::Property, field : &str)
    {
        //let f = field.to_c_str();
        //TODO add selector of enum

        match *self {
          AngleXYZ(ref v) => v.create_widget(property, field),
          Quat(ref q) => q.create_widget(property, field)
        };
    }
}

