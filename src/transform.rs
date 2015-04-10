use vec;
use ui;
use std::ptr;
use libc::{c_char};
use std::ffi::CString;
use std::ops::{Mul};


#[link(name = "joker")]
extern {
    fn property_list_enum_add(
        ps : *const ui::JkPropertyList,
        name : *const c_char,
        possible_values : *const c_char,
        value : *const c_char
        ) -> *const ui::PropertyValue;

     fn property_list_enum_update(
        pv : *const ui::PropertyValue,
        value : *const c_char);
}


#[derive(RustcDecodable, RustcEncodable, Clone, Copy)]
pub enum Orientation
{
    AngleXYZ(vec::Vec3),
    Quat(vec::Quat)
}

impl Orientation
{
    pub fn to_angle_xyz(&mut self) {
        match *self {
            Orientation::Quat(q) => *self = Orientation::AngleXYZ(q.to_euler_deg()),
            _ => {}
        }
    }

    pub fn to_quat(&mut self) {
        match *self {
            Orientation::AngleXYZ(a) => *self = Orientation::Quat(vec::Quat::new_angles_deg(&a)),
            _ => {}
        }
    }

    pub fn as_quat(&self) -> vec::Quat
    {
        match *self {
            Orientation::AngleXYZ(a) => vec::Quat::new_angles_deg(&a),
            Orientation::Quat(q) => q
        }
    }

    pub fn new_with_quat(q : &vec::Quat) -> Orientation
    {
        Orientation::Quat(*q)
    }

    pub fn new_with_angle_xyz(v : &vec::Vec3) -> Orientation
    {
        Orientation::AngleXYZ(*v)
    }

    pub fn new_quat() -> Orientation
    {
        Orientation::Quat(vec::Quat::identity())
    }

    pub fn rotate_vec3(&self, v : &vec::Vec3) -> vec::Vec3
    {
        self.as_quat().rotate_vec3(v)
    }

    pub fn inverse(&self) -> Orientation
    {
        match *self {
            Orientation::AngleXYZ(_) => {
                //TODO
                let q = self.as_quat().inverse();
                let mut o = Orientation::new_with_quat(&q);
                o.to_angle_xyz();
                o
            },
            Orientation::Quat(q) => Orientation::Quat(q.inverse())
        }
    }

    pub fn get_angle_xyz(& self) -> vec::Vec3
    {
        match *self {
            Orientation::Quat(q) => q.to_euler_deg(),
            Orientation::AngleXYZ(a) => {a}
        }
    }

    pub fn get_quat(& self) -> vec::Quat
    {
        match *self {
            Orientation::Quat(q) => q,
            Orientation::AngleXYZ(a) => vec::Quat::new_angles_deg(&a),
        }
    }

    pub fn set_and_keep_type(&mut self, ori : Orientation )
    {
        match *self {
            Orientation::AngleXYZ(_) => 
                *self = Orientation::AngleXYZ(ori.get_angle_xyz()),
            Orientation::Quat(_) => 
                *self = Orientation::Quat(ori.get_quat())
        }
    }
}

#[derive(RustcDecodable, RustcEncodable, Clone)]
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
            orientation : Orientation::Quat(vec::Quat::identity())
        }
    }
}

impl ui::PropertyShow for Orientation {

    fn create_widget(&self, property : &mut ui::Property, field : &str, depth : i32)
    {
        println!("...................DEPTH, field  : {}, {}", depth, field);
        if depth == 0 {
            let type_value = match *self {
                Orientation::AngleXYZ(_) => "AngleXYZ",
                Orientation::Quat(_) => "Quat"
            };

            let types = "AngleXYZ/Quat";
            property.add_enum(self, field, types, type_value);
        }

        if depth == 1 {
            match *self {
                Orientation::AngleXYZ(ref v) =>  {
                    v.create_widget(property, field, depth);
                },
                Orientation::Quat(ref q) => {
                    q.create_widget(property, field, depth)
                }
            };
        }
    }

    fn update_widget(&self, pv : *const ui::property::PropertyValue) {
        let type_value = match *self {
            Orientation::AngleXYZ(_) => "AngleXYZ",
            Orientation::Quat(_) => "Quat"
        };

        let v = CString::new(type_value.as_bytes()).unwrap();
        unsafe {
            property_list_enum_update(pv, v.as_ptr());
        }
    }

    fn get_property(&self, field : &str) -> Option<&ui::PropertyShow>
    {
        match *self {
            Orientation::AngleXYZ(ref v) =>  {
                match field {
                    "x" => Some(&v.x as &ui::PropertyShow),
                    "y" => Some(&v.y as &ui::PropertyShow),
                    "z" => Some(&v.z as &ui::PropertyShow),
                    _ => None
                }
            },
            Orientation::Quat(ref q) => {
                match field {
                    "x" => Some(&q.x as &ui::PropertyShow),
                    "y" => Some(&q.y as &ui::PropertyShow),
                    "z" => Some(&q.z as &ui::PropertyShow),
                    "w" => Some(&q.w as &ui::PropertyShow),
                    _ => None
                }
            }
        }
    }

}

impl Mul<Orientation> for Orientation {
    type Output = Orientation;
    fn mul(self, other: Orientation) -> Orientation {
        let p = self.as_quat() * other.as_quat();
        //Orientation::Quat(self.as_quat() * other.as_quat())
        match self {
            Orientation::AngleXYZ(_) => 
                Orientation::AngleXYZ(p.to_euler_deg()),
            Orientation::Quat(_) => 
                Orientation::Quat(p)
        }
    }
}

impl Mul<vec::Quat> for Orientation {
    type Output = Orientation;
    fn mul(self, other: vec::Quat) -> Orientation {
        let p = self.as_quat() * other;
        match self {
            Orientation::AngleXYZ(_) => 
                Orientation::AngleXYZ(p.to_euler_deg()),
            Orientation::Quat(_) => 
                Orientation::Quat(p)
        }
    }
}

