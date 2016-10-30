use std::sync::{RwLock, Arc};
use std::collections::{HashMap};
use libc::{c_char, c_void, c_int, c_float};
use std::{str,mem,ptr,ffi};
use std::rc::{Rc,Weak};
use std::cell::{Cell, RefCell, BorrowState};
use std::any::{Any};
use std::ffi::{CStr,CString};
use uuid;
use uuid::Uuid;

use dormin::scene;
use dormin::camera;
use dormin::object;
use ui::{Window, ButtonCallback};
use ui::{ChangedFunc, RegisterChangeFunc, PropertyTreeFunc, PropertyValue, PropertyConfig, PropertyUser,
PropertyShow, PropertyId, RefMut, Elm_Object_Item, ShouldUpdate, PropertyWidget, PropertyList, JkPropertyList, PropertyChange};
use ui;
use dormin::property;
use operation;
use control::WidgetUpdate;
use dormin::vec;
use dormin::transform;
use dormin::resource;
use dormin::mesh;
use dormin::material;
use dormin::property::PropertyGet;
use dormin::component;
use dormin::component::CompData;
use dormin::armature;
use dormin::transform::Orientation;

#[repr(C)]
pub struct JkPropertyCb;

#[link(name = "png")]

#[link(name = "ecore_evas")]
#[link(name = "ecore_file")]
#[link(name = "elementary")]
#[link(name = "eina")]
#[link(name = "eet")]
#[link(name = "evas")]
#[link(name = "ecore")]
#[link(name = "edje")]
#[link(name = "eo")]
//#[link(name = "GLESv2")]
#[link(name = "joker")]
extern {

    fn property_node_new(
        path : *const c_char
        ) -> *const PropertyValue;

    fn property_float_new(
        name : *const c_char,
        value : c_float
        ) -> *const PropertyValue;

    fn property_string_new(
        name : *const c_char,
        value : *const c_char
        ) -> *const PropertyValue;

    fn property_vec_new(
        path: *const c_char,
        len : c_int
        ) -> *const PropertyValue;

    fn property_enum_new(
        name : *const c_char,
        possible_values : *const c_char,
        value : *const c_char
        ) -> *const PropertyValue;

    fn property_list_option_update(
        pv : *const PropertyValue,
        value : *const c_char);

    fn property_string_update(
        pv : *const PropertyValue,
        value : *const c_char);

    fn property_float_update(
        pv : *const PropertyValue,
        value : c_float);

    pub fn jk_property_cb_register(
        property : *const JkPropertyCb,
        data : *const c_void,
        changed_float : ChangedFunc,
        changed_string : ChangedFunc,
        changed_enum : ChangedFunc,
        register_change_string : RegisterChangeFunc,
        register_change_float : RegisterChangeFunc,
        register_change_enum : RegisterChangeFunc,
        register_change_option : RegisterChangeFunc,
        expand : PropertyTreeFunc,
        contract : PropertyTreeFunc,
        vec_add : RegisterChangeFunc,
        vec_del : RegisterChangeFunc,
        );
}

impl WidgetUpdate for PropertyList
{
    fn update_changed(
        &mut self,
        name : &str,
        new : &Any)
    {
        let pvs = self.pv.borrow();

        let pv = match pvs.get(&name.to_owned()) {
            Some(p) => p,
            None => {
                println!("widget update, could not find {}", name);
                return;
            }
        };

        match new.downcast_ref::<f64>() {
            Some(v) => {
                unsafe {
                    property_float_update(
                        *pv,
                        *v as c_float);
                };
                return;
            },
            None => {
                println!("cannot downcast to f64");
            }
        }

        match new.downcast_ref::<String>() {
            Some(s) => {
                let v = CString::new(s.as_bytes()).unwrap();
                unsafe {
                    property_string_update(
                        *pv,
                        v.as_ptr());
                };
                return;
            },
            None => {
                println!("cannot downcast to string");
            }
        }

    }
}

impl PropertyShow for f64 {

    fn create_widget_itself(&self, field : &str) -> Option<*const PropertyValue>
    {
        let f = CString::new(field.as_bytes()).unwrap();
        let pv = unsafe { 
            property_float_new(
                f.as_ptr(),
                *self as c_float)
        };

        Some(pv)
    }

    fn update_widget(&self, pv : *const PropertyValue) {
        unsafe {
            property_float_update(
                pv,
                *self as c_float);
        };
    }
}

impl PropertyShow for String {

    fn create_widget_itself(&self, field : &str) -> Option<*const PropertyValue>
    {
        let f = CString::new(field.as_bytes()).unwrap();
        let v = CString::new(self.as_bytes()).unwrap();

        let pv = unsafe {
            property_string_new(
                f.as_ptr(),
                v.as_ptr())
        };

        Some(pv)
    }

    fn update_widget(&self, pv : *const PropertyValue) {
        let v = CString::new(self.as_bytes()).unwrap();
        unsafe {
            property_string_update(
                pv,
                v.as_ptr());
        };
    }
}

impl<T : PropertyShow> PropertyShow for Box<T> {

    fn create_widget_itself(&self, field : &str) -> Option<*const PropertyValue>
    {
        (**self).create_widget_itself(field)
    }

    fn create_widget_inside(&self, path : &str, widget : &PropertyWidget)
    {
        (**self).create_widget_inside(path, widget);
    }

    fn get_property(&self, field : &str) -> Option<&PropertyShow>
    {
        (**self).get_property(field)
    }

    fn update_property(&self, widget : &PropertyWidget, all_path : &str, path : Vec<String>)
    {
        (**self).update_property(widget, all_path, path);
    }

    fn update_property_new(&self, widget : &PropertyWidget, all_path : &str, path : Vec<String>, change : PropertyChange)
    {
        (**self).update_property_new(widget, all_path, path, change);
    }

    fn is_node(&self) -> bool
    {
        (**self).is_node()
    }

    fn to_update(&self) -> ShouldUpdate
    {
        (**self).to_update()
    }
}

impl<T : PropertyShow> PropertyShow for Rc<RefCell<T>> {

    fn get_property(&self, field : &str) -> Option<&PropertyShow>
    {
        panic!("panic to see if this is called");
        None
    }

    fn update_property(&self, widget : &PropertyWidget, all_path: &str, path : Vec<String>)
    {
        self.borrow().update_property(widget, all_path, path);
    }

    fn update_property_new(&self, widget : &PropertyWidget, all_path: &str, path : Vec<String>, change : PropertyChange)
    {
        self.borrow().update_property_new(widget, all_path, path, change);
    }

    fn is_node(&self) -> bool
    {
        self.borrow().is_node()
    }

    fn to_update(&self) -> ShouldUpdate
    {
        self.borrow().to_update()
    }
}

impl<T : PropertyShow> PropertyShow for Option<T> {

    fn get_property(&self, field : &str) -> Option<&PropertyShow>
    {
        match *self {
            Some(ref s) =>
                s.get_property(field),
            None => None
        }
    }

    fn update_widget(&self, pv : *const PropertyValue) {
        let s = match *self {
            Some(_) => "Some",
            None => "None"
        };
        let v = CString::new(s.as_bytes()).unwrap();
        unsafe {
            property_list_option_update(
                pv,
                v.as_ptr());
        };
    }

    fn to_update(&self) -> ShouldUpdate
    {
        match *self {
            Some(ref s) =>
                s.to_update(),
            None => ShouldUpdate::Nothing
        }
    }
}

impl<T> PropertyShow for resource::ResTT<T>
{
    fn get_property(&self, field : &str) -> Option<&PropertyShow>
    {
        match field {
            "name" => Some(&self.name as &PropertyShow),
            _ => None
        }
    }
}

impl<T:PropertyShow> PropertyShow for Vec<T>
{
    fn create_widget_itself(&self, field : &str) -> Option<*const PropertyValue>
    {
        let f = CString::new(field.as_bytes()).unwrap();
        unsafe {
            Some(property_vec_new(f.as_ptr(), self.len() as c_int))
        }
    }

    fn create_widget_inside(&self, path : &str, widget : &PropertyWidget)
    {
        if self.is_empty() {
            //TODO? add "no item" item
        }

        for (n,i) in self.iter().enumerate() {
            let mut nf = String::from(path);
            nf.push_str("/");
            nf.push_str(n.to_string().as_str());
            if let Some(pv) = i.create_widget_itself(nf.as_str()) {
                widget.add_vec_item(nf.as_str(), pv, n);
                i.create_widget_inside(nf.as_str(), widget);
            }
        }
    }

    fn get_property(&self, field : &str) -> Option<&PropertyShow>
    {
        match field.parse::<usize>() {
            Ok(index) => {
                if self.is_empty() || index > self.len() -1 {
                    None
                }
                else {
                    Some(&self[index] as &PropertyShow)
                }
            }
            _ => {
                None
            }
        }
    }

    fn update_widget(&self, pv : *const PropertyValue)
    {
        panic!("This does not do anything right now)");
    }

    fn update_property(&self, widget : &PropertyWidget, all_path: &str, path : Vec<String>)
    {
        if path.is_empty() {
            if let Some(pv) = widget.get_property(all_path) {
                widget.update_vec(pv, self.len());
            }
            return;
        }

        match path[0].parse::<usize>() {
            Ok(index) => {
                self[index].update_property(widget, all_path, path[1..].to_vec());
            }
            _ => {
            }
        }

    }

    fn update_property_new(&self, widget : &PropertyWidget, all_path : &str, local_path : Vec<String>, change : PropertyChange)
    {
        if local_path.is_empty() {
            match change {
                PropertyChange::Value => {
                    panic!("error cannot change vec value");
                },
                PropertyChange::VecAdd(index) => {
                    if !local_path.is_empty() {
                        return;
                    }

                    let mut nf = String::from(all_path);
                    nf.push_str("/");
                    nf.push_str(index.to_string().as_str());

                    if let Some(pv) = self[index].create_widget_itself(nf.as_str()) {
                        widget.add_vec_item(nf.as_str(), pv, index);
                        self[index].create_widget_inside(nf.as_str(), widget);
                    }
                },
                PropertyChange::VecDel(index) => {
                    if !local_path.is_empty() {
                        return;
                    }

                    let mut nf = String::from(all_path);
                    nf.push_str("/");
                    nf.push_str(index.to_string().as_str());
                    widget.del_vec_item(nf.as_str(), index);
                }
            }
        }
        else {
            match local_path[0].parse::<usize>() {
                Ok(index) => {
                    self[index].update_property_new(widget, all_path, local_path[1..].to_vec(), change);
                }
                _ => {
                    panic!("not an index");
                }
            }
        }
    }
}

impl PropertyShow for CompData
{
    fn create_widget_itself(&self, field : &str) -> Option<*const PropertyValue>
    {
        let type_value = self.get_kind_string();
        let types = CompData::get_all_kind();
        Some(add_enum(field, types.as_str(), type_value.as_str()))
    }

    fn create_widget_inside(&self, path : &str, widget : &PropertyWidget)
    {

        let ps : &PropertyShow = match *self {
            CompData::Player(ref p) => {
                p
            },
            CompData::ArmaturePath(ref p) => {
                p
            },
            CompData::MeshRender(ref p) => {
                p
            },
            _ => { println!("not yet implemented"); return; }
        };

        ps.create_widget_inside(path, widget);
    }


    fn update_widget(&self, pv : *const PropertyValue) {
        match *self {
            CompData::Player(ref p) => {
                p.update_widget(pv);
            },
            CompData::ArmaturePath(ref p) => {
                p.update_widget(pv);
            },
            CompData::MeshRender(ref p) => {
                p.update_widget(pv);
            },
            _ => {println!("not yet implemented");}
        }
    }

    fn update_property(&self, widget : &PropertyWidget, all_path: &str, path : Vec<String>)
    {
        if path.is_empty() {
            if let Some(pv) = widget.get_property(all_path) {
                self.update_widget(pv);

                let type_value = self.get_kind_string();
                widget.update_enum(all_path, pv, type_value.as_str());
                self.create_widget_inside(all_path, widget);
            }
            return;
        }

        let ppp : &PropertyShow = match *self {
            CompData::Player(ref p) => {
                p
            },
            CompData::ArmaturePath(ref p) => {
                p
            },
            CompData::MeshRender(ref p) => {
                p
            },
            _ => {println!("not yet implemented"); return;}
        };

        ppp.update_property(widget, all_path, path);
    }

    fn update_property_new(&self, widget : &PropertyWidget, all_path: &str, path : Vec<String>, change : PropertyChange)
    {
        if path.is_empty() {
            if let Some(pv) = widget.get_property(all_path) {
                self.update_widget(pv);

                let type_value = self.get_kind_string();
                widget.update_enum(all_path, pv, type_value.as_str());
                self.create_widget_inside(all_path, widget);
            }
            return;
        }

        let ppp : &PropertyShow = match *self {
            CompData::Player(ref p) => {
                p
            },
            CompData::ArmaturePath(ref p) => {
                p
            },
            CompData::MeshRender(ref p) => {
                p
            },
            _ => {println!("not yet implemented"); return;}
        };

        ppp.update_property_new(widget, all_path, path, change);
    }



    fn get_property(&self, field : &str) -> Option<&PropertyShow>
    {
        if field != self.get_kind_string() {
            return None;
        }

        match *self {
            CompData::Player(ref p) => {
                Some(p)
            },
            CompData::ArmaturePath(ref p) => {
                Some(p)
            },
            CompData::MeshRender(ref p) => {
                Some(p)
            },
            _ => {
                println!("not yet implemented");
                None
            }
        }
    }

    fn is_node(&self) -> bool
    {
        true
    }

    fn to_update(&self) -> ShouldUpdate
    {
        match *self {
            CompData::Player(ref p) => {
                p.to_update()
            },
            CompData::ArmaturePath(ref p) => {
                p.to_update()
            },
            CompData::MeshRender(ref p) => {
                p.to_update()
            },
            _ => {
                println!("not yet implemented");
                ShouldUpdate::Nothing
            }
        }
    }

}

impl ui::PropertyShow for Orientation {

    fn create_widget_itself(&self, field : &str) -> Option<*const PropertyValue>
    {
        let type_value = match *self {
            Orientation::AngleXYZ(_) => "AngleXYZ",
            Orientation::Quat(_) => "Quat"
        };

        let types = "AngleXYZ/Quat";
        Some(add_enum(field, types, type_value))
    }

    fn create_widget_inside(&self, path : &str, widget : &PropertyWidget)//, parent : *const PropertyValue)
    {
        let ps : &PropertyShow = match *self {
            Orientation::AngleXYZ(ref v) =>  {
                v
            },
            Orientation::Quat(ref q) => {
                q
            }
        };

        ps.create_widget_inside(path, widget);
    }

    fn update_property(&self, widget : &PropertyWidget, all_path: &str, path : Vec<String>)
    {
        if path.is_empty() {
            if let Some(pv) = widget.get_property(all_path) {
                self.update_widget(pv);

                let type_value = match *self {
                    Orientation::AngleXYZ(_) => "AngleXYZ",
                    Orientation::Quat(_) => "Quat"
                };

                widget.update_enum(all_path, pv, type_value);

                self.create_widget_inside(all_path, widget);
            }
            return;
        }

        let ppp : &PropertyShow = match *self {
            Orientation::AngleXYZ(ref v) => v,
            Orientation::Quat(ref q) => q,
        };

        ppp.update_property(widget, all_path, path);
    }

    fn update_property_new(&self, widget : &PropertyWidget, all_path: &str, path : Vec<String>, change : PropertyChange)
    {
        if path.is_empty() {
            if let Some(pv) = widget.get_property(all_path) {
                self.update_widget(pv);

                let type_value = match *self {
                    Orientation::AngleXYZ(_) => "AngleXYZ",
                    Orientation::Quat(_) => "Quat"
                };

                widget.update_enum(all_path, pv, type_value);

                self.create_widget_inside(all_path, widget);
            }
            return;
        }

        let ppp : &PropertyShow = match *self {
            Orientation::AngleXYZ(ref v) => v,
            Orientation::Quat(ref q) => q,
        };

        ppp.update_property_new(widget, all_path, path, change);
    }


    fn update_widget(&self, pv : *const PropertyValue) {
        let type_value = match *self {
            Orientation::AngleXYZ(_) => "AngleXYZ",
            Orientation::Quat(_) => "Quat"
        };
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


macro_rules! property_show_methods(
    ($my_type:ty, [ $($member:ident),+ ]) => (

            fn create_widget_itself(&self, field : &str) -> Option<*const PropertyValue>
            {
                Some(add_node(field))
            }

            fn create_widget_inside(&self, path : &str, widget : &PropertyWidget)
            {
                $(
                    let s = if path != "" {
                        path.to_owned()
                            + "/"
                            + stringify!($member)
                    }else {
                        stringify!($member).to_owned()
                    };
                    if let Some(pv) = self.$member.create_widget_itself(s.as_str()) {
                        widget.add_simple_item(s.as_str(), pv);
                        self.$member.create_widget_inside(s.as_str(), widget);
                    }
                 )+
            }

            fn get_property(&self, field : &str) -> Option<&PropertyShow>
            {
                match field {
                $(
                    stringify!($member) => Some(&self.$member as &PropertyShow),
                 )+
                    _ => None
                }
            }

            fn update_property(&self, widget : &PropertyWidget, all_path: &str, path : Vec<String>)
            {
                let mut pp = String::from(all_path);
                if !path.is_empty() && path[0] == "*" {
                    pp = pp.replace("/*", "");
                }

                if path.is_empty() || path[0] == "*" {
                    //update all
                    $(
                        let s = pp.clone() + "/" + stringify!($member);
                        self.$member.update_property(widget, s.as_str(), Vec::new());
                    )+
                    return;
                }

                match path[0].as_str() {
                $(
                    stringify!($member) => self.$member.update_property(widget, all_path, path[1..].to_vec()),
                 )+
                    _ => {}
                }
            }

            fn update_property_new(&self, widget : &PropertyWidget, all_path: &str, path : Vec<String>, change : PropertyChange)
            {
                let mut pp = String::from(all_path);
                if !path.is_empty() && path[0] == "*" {
                    pp = pp.replace("/*", "");
                }

                if path.is_empty() || path[0] == "*" {
                    //update all
                    $(
                        let s = pp.clone() + "/" + stringify!($member);
                        self.$member.update_property_new(widget, s.as_str(), Vec::new(),change.clone());
                    )+
                    return;
                }

                match path[0].as_str() {
                $(
                    stringify!($member) => self.$member.update_property_new(widget, all_path, path[1..].to_vec(), change),
                 )+
                    _ => {}
                }
            }

            fn is_node(&self) -> bool
            {
                true
            }
    )
);

macro_rules! property_show_impl(
    ($my_type:ty, $e:tt) => (
        impl PropertyShow for $my_type {
            property_show_methods!($my_type, $e);
        });
    ($my_type:ty, $e:tt, $up:expr) => (
        impl PropertyShow for $my_type {
            property_show_methods!($my_type, $e);

            fn to_update(&self) -> ShouldUpdate
            {
                $up
            }
        }
        )
    );

property_show_impl!(vec::Vec3,[x,y,z]);
property_show_impl!(vec::Quat,[x,y,z,w]);
property_show_impl!(transform::Transform,[position,orientation]);
property_show_impl!(object::Object,
                     [name,position,orientation,scale,comp_data,comp_lua]);

property_show_impl!(component::mesh_render::MeshRender,[mesh,material], ShouldUpdate::Mesh);
property_show_impl!(component::player::Player,[speed]);
property_show_impl!(component::player::Enemy,[name]);
property_show_impl!(component::player::Collider,[name]);
property_show_impl!(armature::ArmaturePath,[name]);

property_show_impl!(scene::Scene,[name,camera]);
property_show_impl!(camera::Camera,[data]);
property_show_impl!(camera::CameraData,[far,near]);

pub fn make_vec_from_str(s : &str) -> Vec<String>
{
    let v: Vec<&str> = s.split('/').collect();

    let mut vs = Vec::new();
    for i in &v
    {
        vs.push(i.to_string());
    }

    vs
}

pub fn find_property_show(p : &PropertyShow, path : Vec<String>) ->
Option<&PropertyShow>
{
    match path.len() {
        0 =>  None,
        1 => p.get_property(path[0].as_ref()),
        _ => {
             match p.get_property(path[0].as_ref()) {
                 Some(ppp) => {
                     find_property_show(ppp, path[1..].to_vec())
                 },
                 None => {
                     None
                 }
             }
        }
    }
}

pub extern fn vec_add(
    data : *const c_void,
    property : *const c_void,
    old : *const c_void,
    new : *const c_void,
    action : c_int)
{
    let node : &Weak<RefCell<ui::PropertyNode>> = unsafe {mem::transmute(property)};
    let node = if let Some(n) = node.upgrade() {
        n
    }
    else {
        panic!("VEC ADD :: cannot upgrade rc, node");
    };

    let wcb : & ui::WidgetCbData = unsafe {mem::transmute(data)};
    let container : &mut Box<ui::WidgetContainer> = unsafe {mem::transmute(wcb.container)};

    let change = container.request_operation_vec_add(node.clone());
    container.handle_change(&change, uuid::Uuid::nil());//p.id);
}

pub extern fn vec_del(
    data : *const c_void,
    property : *const c_void,
    old : *const c_void,
    new : *const c_void,
    action : c_int)
{
    let node : &Weak<RefCell<ui::PropertyNode>> = unsafe {mem::transmute(property)};
    let node = if let Some(n) = node.upgrade() {
        n
    }
    else {
        panic!("problem with node");
    };

    let wcb : & ui::WidgetCbData = unsafe {mem::transmute(data)};
    let container : &mut Box<ui::WidgetContainer> = unsafe {mem::transmute(wcb.container)};

    let change = container.request_operation_vec_del(node);
    container.handle_change(&change, uuid::Uuid::nil());
}

impl PropertyId for object::Object
{
    fn get_id(&self) -> uuid::Uuid
    {
        return self.id
    }
}

impl PropertyId for scene::Scene
{
    fn get_id(&self) -> uuid::Uuid
    {
        return self.id
    }
}

pub fn add_node(
    name : &str
    ) -> *const PropertyValue
{
    let f = CString::new(name.as_bytes()).unwrap();
    unsafe {
        property_node_new(f.as_ptr())
    }
}

pub fn add_enum(
    path : &str,
    types : &str,
    value : &str
    ) -> *const PropertyValue
{
    let f = CString::new(path.as_bytes()).unwrap();
    let types = CString::new(types.as_bytes()).unwrap();
    let v = CString::new(value.as_bytes()).unwrap();

    unsafe {
        property_enum_new(
            f.as_ptr(),
            types.as_ptr(),
            v.as_ptr())
    }
}

