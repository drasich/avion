use vec;
use property;
use ui;
use std::any::{Any};//, AnyRefExt};
use std::ptr;
use libc::{c_char, c_void, c_int, c_float};
use std::ffi::CString;
//use std::c_str::ToCStr;
use std::ops::{Mul};//, BitXor, Add, Sub, Div};


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
            //let yep = field.to_string() + "/type";
            let f = CString::from_slice(field.as_bytes());
            let type_value = match *self {
                Orientation::AngleXYZ(_) => "AngleXYZ",
                Orientation::Quat(_) => "Quat"
            };

            let v = CString::from_slice(type_value.as_bytes());

            let types = CString::from_slice("AngleXYZ/Quat".as_bytes());

            unsafe {
                let pv = property_list_enum_add(
                    property.jk_property_list,
                    f.as_ptr(),
                    types.as_ptr(),
                    v.as_ptr());

                if pv != ptr::null() {
                    println!("ADDING : {}", field);
                    property.pv.insert(field.to_string(), pv);
                }
            }
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
        println!("TODODODODODO DO YOU CALL ME");
        let type_value = match *self {
            Orientation::AngleXYZ(_) => "AngleXYZ",
            Orientation::Quat(_) => "Quat"
        };

        let v = CString::from_slice(type_value.as_bytes());
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
        Orientation::Quat(self.as_quat() * other.as_quat())
    }
}

