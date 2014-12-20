use vec;
use property;
use ui;
use std::any::{Any, AnyRefExt};
use std::ptr;
use libc::{c_char, c_void, c_int, c_float};


#[link(name = "joker")]
extern {
    fn property_list_enum_add(
        ps : *const ui::JkPropertyList,
        name : *const c_char,
        possible_values : *const c_char,
        value : *const c_char
        ) -> *const ui::PropertyValue;
}


#[deriving(Decodable, Encodable, Clone)]
pub enum Orientation
{
    AngleXYZ(vec::Vec3),
    Quat(vec::Quat)
}

impl Orientation
{
    pub fn to_angle_xyz(&mut self) {
        match *self {
            Quat(q) => *self = AngleXYZ(q.to_euler_deg()),
            _ => {}
        }
        println!("todo : to angle xyz");
    }

    pub fn to_quat(&mut self) {
        println!("todo : to quat");
        match *self {
            AngleXYZ(a) => *self = Quat(vec::Quat::new_angles_deg(&a)),
            _ => {}
        }
    }
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

    fn create_widget(&self, property : &mut ui::Property, field : &str, depth : i32)
    {
        //let f = field.to_c_str();
        //TODO add selector of enum
        /*
        unsafe {
            let pv= property_list_enum_add(
                property.jk_property_list,
                field + "/type" .unwrap(),
                "AngleXYZ"); // "Quat"
                    
            if pv != ptr::null() {
                self.pv.insert(field, pv);
            }
        }
        */

        //let yep = field.to_string() + "/type";
        let f = field.to_c_str();
        let type_value = match *self {
            AngleXYZ(_) => "AngleXYZ",
            Quat(_) => "Quat"
        };
        let v = type_value.to_c_str();

        let types = "AngleXYZ/Quat".to_c_str();

        unsafe {
            let pv = property_list_enum_add(
                property.jk_property_list,
                f.unwrap(),
                types.unwrap(),
                v.unwrap());

            if pv != ptr::null() {
                property.pv.insert(field.to_string(), pv);
            }
        }

        return;

        match *self {
          AngleXYZ(ref v) =>  {
              //v.create_widget(property, field, depth);
              let yep = field.to_string() + "/AngleXYZ";
              v.create_widget(property, yep.as_slice(), depth);
          },
          Quat(ref q) => {
              //q.create_widget(property, field, depth)
              let yep = field.to_string() + "/Quat";
              q.create_widget(property, yep.as_slice(), depth);
          }
        };
    }
}


