use vec;
use property;
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
    /*
  fn fields(&self) -> Box<[String]> 
  {
      return box[
          String::from_str("AnglesXYZ"),
          String::from_str("Quat"),
      ];
  }
  */

  /*
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
  */

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
