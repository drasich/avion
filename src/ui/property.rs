use std::sync::{RwLock, Arc};
use std::collections::{HashMap,HashSet};
use libc::{c_char, c_void, c_int, c_float};
use std::str;
use std::mem;
//use std::collections::{LinkedList,Deque};
use std::collections::{LinkedList};
use std::ptr;
use std::rc::Rc;
use std::cell::{Cell, RefCell, BorrowState};
use std::rc::Weak;
use std::any::{Any};//, AnyRefExt};
use std::ffi::CString;
use std::ffi;
use std::ffi::CStr;
use core::marker;
use uuid;
use uuid::Uuid;

use dormin::scene;
use dormin::camera;
use dormin::object;
use ui::{Window, ButtonCallback};
use ui::{ChangedFunc, RegisterChangeFunc, PropertyTreeFunc, PropertyValue, PropertyConfig, PropertyUser,
PropertyShow, PropertyId, RefMut, Elm_Object_Item, ShouldUpdate, PropertyWidget, PropertyList, JkPropertyList};
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

use util::join_string;

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

    fn property_list_node_add(
        path : *const c_char,
        added_name : *const c_char
        ) -> *const PropertyValue;

    fn property_node_add(
        path : *const c_char
        ) -> *const PropertyValue;

    fn property_list_float_add(
        name : *const c_char,
        value : c_float
        ) -> *const PropertyValue;

    fn property_list_string_add(
        name : *const c_char,
        value : *const c_char
        ) -> *const PropertyValue;

    fn property_list_node_vec_add(
        ps : *const JkPropertyList,
        container: *const PropertyValue,
        ) -> *const PropertyValue;

    fn property_vec_add(
        path: *const c_char,
        len : c_int
        ) -> *const PropertyValue;


    /*
    fn property_list_vec_item_add(
        ps : *const JkPropertyList,
        name : *const c_char,
        value : *const c_char
        ) -> *const PropertyValue;
        */

    fn property_list_enum_add(
        name : *const c_char,
        possible_values : *const c_char,
        value : *const c_char
        ) -> *const PropertyValue;

    fn property_list_option_update(
        pv : *const PropertyValue,
        value : *const c_char);

    fn property_list_string_update(
        pv : *const PropertyValue,
        value : *const c_char);

    fn property_list_float_update(
        pv : *const PropertyValue,
        value : c_float);

    pub fn property_list_enum_update(
        pv : *const ui::PropertyValue,
        value : *const c_char);

    fn property_list_vec_update(
        pv : *const PropertyValue,
        len : c_int);

    pub fn property_expand(
        pv : *const PropertyValue);

    pub fn jk_property_cb_register(
        property : *const JkPropertyCb,
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

    pub fn property_show(obj : *const JkPropertyList, b : bool);
}

pub extern fn name_get(data : *const c_void) -> *const c_char {

    let o : &Arc<RwLock<object::Object>> = unsafe {
        mem::transmute(data)
    };

    let cs = CString::new(o.read().unwrap().name.as_bytes()).unwrap();
    //println!("..........name get {:?}", cs);
    cs.as_ptr()
}


impl WidgetUpdate for PropertyList
{
    fn update_changed(
        &mut self,
        name : &str,
        new : &Any)
    {

        //println!("property update changed {}", name);
        //
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
                    property_list_float_update(
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
                    property_list_string_update(
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

/*
impl PropertyShow for vec::Quat {

    fn create_entries(
        &mut self,
        property:&mut Property,
        path : Vec<String>)
    {
        println!("quuuuuuuuuuuuuuuuaaaaaaaaaaaaaaaaaaaaaaat {} ", path);
    }
}
*/

impl PropertyShow for f64 {

    fn create_widget(
        &self,
        property : &PropertyWidget,
        field : &str,
        depth : i32,
        has_container : bool ) -> Option<*const PropertyValue>
    {
        let f = CString::new(field.as_bytes()).unwrap();
        println!("create f64 for : {}", field);
        let pv = unsafe { 
            property_list_float_add(
                f.as_ptr(),
                *self as c_float)
        };

        if !has_container {
            property.add_simple_item(field, pv);
            None
        }
        else {
            Some(pv)
        }
    }

    fn create_widget_itself(&self, field : &str) -> Option<*const PropertyValue>
    {
        let f = CString::new(field.as_bytes()).unwrap();
        println!("create f64 for : {}", field);
        let pv = unsafe { 
            property_list_float_add(
                f.as_ptr(),
                *self as c_float)
        };

        Some(pv)
    }

    fn update_widget(&self, pv : *const PropertyValue) {
        unsafe {
            property_list_float_update(
                pv,
                *self as c_float);
        };
    }
}

impl PropertyShow for String {

    fn create_widget(
        &self,
        property : &PropertyWidget,
        field : &str,
        depth : i32,
        has_container : bool ) -> Option<*const PropertyValue>
    {
        let f = CString::new(field.as_bytes()).unwrap();
        let v = CString::new(self.as_bytes()).unwrap();

        let pv = unsafe {
            property_list_string_add(
                f.as_ptr(),
                v.as_ptr())
        };

        if !has_container {
            property.add_simple_item(field, pv);
            None
        }
        else {
            Some(pv)
        }
    }

    fn create_widget_itself(&self, field : &str) -> Option<*const PropertyValue>
    {
        let f = CString::new(field.as_bytes()).unwrap();
        let v = CString::new(self.as_bytes()).unwrap();

        let pv = unsafe {
            property_list_string_add(
                f.as_ptr(),
                v.as_ptr())
        };

        Some(pv)
    }

    fn update_widget(&self, pv : *const PropertyValue) {
        let v = CString::new(self.as_bytes()).unwrap();
        println!("update string value with : {}", self);
        unsafe {
            property_list_string_update(
                pv,
                v.as_ptr());
        };
    }
}

impl<T : PropertyShow> PropertyShow for Box<T> {

    fn create_widget(
        &self,
        property : &PropertyWidget,
        field : &str,
        depth : i32,
        has_container : bool ) -> Option<*const PropertyValue>
    {
        (**self).create_widget(property ,field, depth, has_container)
    }

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

    fn find_and_create(&self, property : &PropertyWidget, path : Vec<String>, start : usize)
    {
        (**self).find_and_create(property, path, start);
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

    fn create_widget(
        &self,
        property : &PropertyWidget,
        field : &str,
        depth : i32,
        has_container : bool ) -> Option<*const PropertyValue>
    {
        self.borrow().create_widget(property ,field, depth, has_container)
    }

    fn get_property(&self, field : &str) -> Option<&PropertyShow>
    {
        //(**self).get_property(field)
        //(**self).get_property(field)
        None
    }

    fn update_property(&self, widget : &PropertyWidget, all_path: &str, path : Vec<String>)
    {
        //(**self).update_property(path, pv);
        self.borrow().update_property(widget, all_path, path);
    }

    fn find_and_create(&self, property : &PropertyWidget, path : Vec<String>, start : usize)
    {
        self.borrow().find_and_create(property, path, start);
    }

    fn is_node(&self) -> bool
    {
        //(**self).is_node()
        self.borrow().is_node()
    }

    fn to_update(&self) -> ShouldUpdate
    {
        self.borrow().to_update()
    }
}

impl<T : PropertyShow> PropertyShow for Option<T> {

    fn create_widget(
        &self,
        property : &PropertyWidget,
        field : &str,
        depth : i32,
        has_container : bool ) -> Option<*const PropertyValue>
    {
        if depth == 0 {
            unsafe {
                property.add_option(field, self.is_some());
                return None;
            }
        }

        if depth == 1 {
            if let Some(ref s) = *self {
                return s.create_widget(property, field, depth, has_container);
            };
        }

        None
    }

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

    fn find_and_create(&self, property : &PropertyWidget, path : Vec<String>, start : usize)
    {
        if let Some(ref s) = *self {
                s.find_and_create(property, path, start);
        }
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
    fn create_widget(
        &self,
        property : &PropertyWidget,
        field : &str,
        depth : i32,
        has_container : bool ) -> Option<*const PropertyValue>
    {
        if depth < 0 {
            return None;
        }

        if depth == 0 && field != ""
        {
            add_node(property, self, field, has_container, None);
        }

        if depth > 0 {
            let s = field.to_owned() + "/name";
            return self.name.create_widget(property, s.as_ref(), depth-1, has_container);
        }

        None
    }

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
    fn create_widget(
        &self,
        property : &PropertyWidget,
        field : &str,
        depth : i32,
        has_container : bool ) -> Option<*const PropertyValue>
    {
        if depth < 0 {
            return None;
        }

        if depth == 0 && field != ""
        {
            //TODO
            //TODO
            property.add_vec(field, self.len());
        }

        if depth > 0 {
            if self.is_empty() {
                //add "no item" item
            }

            for (n,i) in self.iter().enumerate() {
                let mut nf = String::from(field);
                nf.push_str("/");
                nf.push_str(n.to_string().as_str());
                if let Some(ref mut pv) = i.create_widget(property, nf.as_str(), depth -1, true) {
                    unsafe {
                        property.add_vec_item(nf.as_str(), *pv, i.is_node());
                    }
                }
                else {
                    println!("___ Vec : failed" );
                }
            }
        }

        None
    }

    fn create_widget_itself(&self, field : &str) -> Option<*const PropertyValue>
    {
        println!("create widget itself vec");
        let f = CString::new(field.as_bytes()).unwrap();
        unsafe {
            Some(property_vec_add(f.as_ptr(), self.len() as c_int))
        }
    }

    fn create_widget_inside(&self, path : &str, widget : &PropertyWidget)//, parent : *const PropertyValue)
    {
        println!("TODO inside vec");
        if self.is_empty() {
            //add "no item" item
        }

        for (n,i) in self.iter().enumerate() {
            let mut nf = String::from(path);
            nf.push_str("/");
            nf.push_str(n.to_string().as_str());
            //chris
            /*
            if let Some(ref mut pv) = i.create_widget(property, nf.as_str(), depth -1, true) {
                unsafe {
                    property.add_vec_item(nf.as_str(), *pv, i.is_node());
                    }
            }
            else {
                println!("___ Vec : failed" );
            }
            */

            if let Some(pv) = i.create_widget_itself(nf.as_str()) {
                widget.add_simple_item(nf.as_str(), pv);

                i.create_widget_inside(nf.as_str(), widget);
            }
        }
    }


    fn get_property(&self, field : &str) -> Option<&PropertyShow>
    {
        match field.parse::<usize>() {
            Ok(index) => {
                if self.is_empty() || index > self.len() -1 {
                    println!("5555555555555555555get property of vec :: index is too big, or list is empty : {}, {}", index, self.len());
                    None
                }
                else {
                    Some(&self[index] as &PropertyShow)
                }
            }
            _ => {
                println!("$$$$$$$$$$$$$$$ Vec return none for field {}", field);
                None
            }
        }
    }

    fn update_widget(&self, pv : *const PropertyValue) {
        
        println!("TODO TODO call update_vec from property");
        unsafe { property_list_vec_update(pv, self.len() as c_int); }
        unsafe { property_expand(pv); }
    }

    fn find_and_create(&self, property : &PropertyWidget, path : Vec<String>, start : usize)
    {
        if path.is_empty() {
            self.create_widget(property, "" , 0, false);
            return;
        }
        else if start == path.len() -1 {
            match path[start].parse::<usize>() {
                Ok(index) => {
                    self[index].create_widget(property, join_string(&path).as_str(),1, false);
                },
                _ => return
            }
        }
        else {
            match path[start].parse::<usize>() {
                Ok(index) => {
                    self[index].find_and_create(property, path, start + 1);
                },
                _ => return
            }
        }

    }

}

impl PropertyShow for CompData
{
    fn create_widget(
        &self,
        property : &PropertyWidget,
        field : &str,
        depth : i32,
        has_container : bool ) -> Option<*const PropertyValue>
    {
        let kind : String = self.get_kind_string();
        let kindr : &str = kind.as_ref();
        let ss = field.to_owned() + "/" + kindr;
        //let ss = field.to_owned() + ":" + kindr;
        let s : &str = ss.as_ref();

        /*
        let mut v : Vec<&str> = field.split('/').collect();

        if v.len() >= 1 {
            v.pop();
        }

        v.push(kindr);

        //println!("--before compdata property show for : {}, {}, {}", field, depth, kind);

        let yo : String = v.join("/");
        let field = yo.as_ref();
        */


        if depth < 0 {
            return None;
        }

        if depth == 0 && field != ""
        {
            /*
            println!("00--> compdata property show for : {}, {}, {}", s, depth, kind );
            //let pv = property.add_node(self, s, has_container);
            let pv = property.add_node(self, field, has_container, Some(kindr));
            return Some(pv);
            */

            let type_value = self.get_kind_string();

            let types = CompData::get_all_kind();
            let pv = add_enum(property, field, types.as_str(), type_value.as_str(), true, has_container);
            return Some(pv);
        }

        if depth > 0
        {
            match *self {
                CompData::Player(ref p) => {
                    return p.create_widget(property, field, depth, has_container);
                },
                CompData::ArmaturePath(ref p) => {
                    return p.create_widget(property, field, depth, has_container);
                },
                CompData::MeshRender(ref p) => {
                    return p.create_widget(property, field, depth, has_container);
                },
                _ => {println!("not yet implemented"); }
            }
        }
        None
    }

    fn create_widget_itself(&self, field : &str) -> Option<*const PropertyValue>
    {
        println!("create widget itself, compdata");
        let type_value = self.get_kind_string();

        let types = CompData::get_all_kind();
        Some(add_enum2(field, types.as_str(), type_value.as_str()))
    }

    fn create_widget_inside(&self, path : &str, widget : &PropertyWidget)//, parent : *const PropertyValue)
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

    fn create_widget(
        &self,
        property : &PropertyWidget,
        field : &str,
        depth : i32,
        has_container : bool ) -> Option<*const ui::PropertyValue>
    {
        if depth == 0 {
            let type_value = match *self {
                Orientation::AngleXYZ(_) => "AngleXYZ",
                Orientation::Quat(_) => "Quat"
            };

            let types = "AngleXYZ/Quat";
            add_enum(property, field, types, type_value, true, has_container);
        }

        if depth == 1 {
            match *self {
                Orientation::AngleXYZ(ref v) =>  {
                    return v.create_widget(property, field, depth, has_container);
                },
                Orientation::Quat(ref q) => {
                    return q.create_widget(property, field, depth, has_container)
                }
            };
        }

        None
    }

    fn create_widget_itself(&self, field : &str) -> Option<*const PropertyValue>
    {
        let type_value = match *self {
            Orientation::AngleXYZ(_) => "AngleXYZ",
            Orientation::Quat(_) => "Quat"
        };

        let types = "AngleXYZ/Quat";
        Some(add_enum2(field, types, type_value))
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
        println!("update property orientation with path : {:?} ", path);
        if path.is_empty() {
            if let Some(pv) = widget.get_property(all_path) {
                self.update_widget(pv);

                let type_value = match *self {
                    Orientation::AngleXYZ(_) => "AngleXYZ",
                    Orientation::Quat(_) => "Quat"
                };

                //widget.update_enum(join_string(&path).as_str(),pv, type_value);
                widget.update_enum(all_path, pv, type_value);

                /*
                match *self {
                    Orientation::AngleXYZ(ref v) =>  {
                        v.create_widget(widget, all_path, 1, false);
                    },
                    Orientation::Quat(ref q) => {
                        q.create_widget(widget, all_path, 1, false);
                    }
                };
                */
                self.create_widget_inside(all_path, widget);
            }
            return;
        }

        match *self {
            Orientation::AngleXYZ(ref v) =>  {
                match path[0].as_str() {
                    "x" => v.x.update_property(widget, all_path, path[1..].to_vec()),
                    "y" => v.y.update_property(widget, all_path, path[1..].to_vec()),
                    "z" => v.z.update_property(widget, all_path, path[1..].to_vec()),
                    _ => {}
                }
            },
            Orientation::Quat(ref q) => {
                match path[0].as_str() {
                    "x" => q.x.update_property(widget, all_path, path[1..].to_vec()),
                    "y" => q.y.update_property(widget, all_path, path[1..].to_vec()),
                    "z" => q.z.update_property(widget, all_path, path[1..].to_vec()),
                    "w" => q.w.update_property(widget, all_path, path[1..].to_vec()),
                    _ => {}
                }
            }
        }
    }


    fn update_widget(&self, pv : *const PropertyValue) {
        let type_value = match *self {
            Orientation::AngleXYZ(_) => "AngleXYZ",
            Orientation::Quat(_) => "Quat"
        };

        //widget.update_enum(pv, type_value);
        /*
        let v = CString::new(type_value.as_bytes()).unwrap();
        unsafe {
            property_list_enum_update(pv, v.as_ptr());
        }
        */
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

            fn create_widget(
                &self,
                property : &PropertyWidget,
                field : &str,
                depth : i32,
                has_container : bool ) -> Option<*const PropertyValue>
            {
                if depth < 0 {
                    return None;
                }

                println!("macro create widget : {}, {}", field, depth);


                if depth == 0 && field != ""
                {
                    add_node(property, self, field, has_container, None);
                }

                if depth > 0 {
                    self.create_widget_inside(field, property);
                }

                None
            }

            fn create_widget_itself(&self, field : &str) -> Option<*const PropertyValue>
            {
                println!("property show methods **{}**, create itself", field);
                Some(add_node2(field))
            }

            fn create_widget_inside(&self, path : &str, widget : &PropertyWidget)//, parent : *const PropertyValue)
            {
                $(
                    let s = if path != "" {
                        path.to_owned()
                            + "/"
                            + stringify!($member)
                    }else {
                        stringify!($member).to_owned()
                    };
                    println!("**************** GOING to create THISSSSSSSSSS : {}, {}", s, stringify!($member));
                    if let Some(pv) = self.$member.create_widget_itself(s.as_str()) {
                        println!("**************** looks ok : {}, {}", s, stringify!($member));
                        widget.add_simple_item(s.as_str(), pv);

                        self.$member.create_widget_inside(s.as_str(), widget);
                    }
                    else {
                        println!("**************** cannot add pv : {}, {}", s, stringify!($member));
                    }
                 )+

            }

            /*
            fn get_children(&self) -> Option<LinkedList<&PropertyShow>>
            {
                let mut list = LinkedList::new();
                $(
                    list.push_back(&self.$member as &PropertyShow);
                 )+

                if list.len() > 0 {
                    Some(list)
                }
                else {
                    None
                }
            }
            */

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
                println!("macro... {}, {:?}", all_path, path);

                if path.is_empty() {
                    //update all
                    $(
                        let s = String::from(all_path) + "/" + stringify!($member);
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

            fn find_and_create(&self, property : &PropertyWidget, path : Vec<String>, start : usize)
            {
                if path.is_empty() {
                    println!("macro create property 000000 : empty path");
                    self.create_widget(property, "" , 0, false);
                    return;
                }
                else if start == path.len() -1 {
                    match path[start].as_str() {
                        $(
                            stringify!($member) => {
                                self.$member.create_widget(property, join_string(&path).as_str(),1, false);
                            },
                            )+
                            _ => {}
                    }
                    return;
                }

                match path[start].as_str() {
                $(
                    stringify!($member) => self.$member.find_and_create(property, path, start + 1),
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
//property_show_impl!(mesh_render::MeshRender,[mesh,material]);
property_show_impl!(object::Object,
                     //[name,position,orientation,scale]);
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
    //let vs = make_vec_from_string(&field.to_owned());

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
    name : *const c_char,
    old : *const c_void,
    new : *const c_void,
    action : c_int)
{
    let s = unsafe {CStr::from_ptr(name).to_bytes()};

    let path = match str::from_utf8(s) {
        Ok(pp) => pp,
        _ => {
            println!("problem with the path");
            return;}
    };

    let wcb : & ui::WidgetCbData = unsafe {mem::transmute(data)};
    //let p : &Property = unsafe {mem::transmute(wcb.widget)};
    let container : &mut Box<ui::WidgetContainer> = unsafe {mem::transmute(wcb.container)};

    let change = container.request_operation_vec_add(path);
    container.handle_change(&change, uuid::Uuid::nil());//p.id);
    //ui::add_empty(container, action.view_id);
}

pub extern fn vec_del(
    data : *const c_void,
    name : *const c_char,
    old : *const c_void,
    new : *const c_void,
    action : c_int)
{
    println!("TODO vec del");

    let s = unsafe {CStr::from_ptr(name).to_bytes()};

    let path = match str::from_utf8(s) {
        Ok(pp) => pp,
        _ => {
            println!("problem with the path");
            return;}
    };

    let wcb : & ui::WidgetCbData = unsafe {mem::transmute(data)};
    //let p : & Property = unsafe {mem::transmute(wcb.widget)};
    let container : &mut Box<ui::WidgetContainer> = unsafe {mem::transmute(wcb.container)};

    let change = container.request_operation_vec_del(path);
    container.handle_change(&change, uuid::Uuid::nil());//p.id);
    //ui::add_empty(container, action.view_id);
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
    property : &PropertyWidget,
    ps : &PropertyShow,
    name : &str,
    has_container : bool,
    added_name : Option<&str>,
    ) -> *const PropertyValue
{
    let f = CString::new(name.as_bytes()).unwrap();
    let mut pv = unsafe {
        let test = if has_container {
            if let Some(n) = added_name {
                CString::new(n).unwrap().as_ptr()
            }
            else {
                ptr::null()
            }
        }
        else {
            ptr::null()
        };
        println!("adding node : {}", name);
        property_list_node_add(
            f.as_ptr(),
            test
            )
    };

    if !has_container {
        println!(".......with single node : {}", name);
        property.add_node_t(name, pv);
    }

    return pv;
}

pub fn add_node2(
    name : &str
    ) -> *const PropertyValue
{
    let f = CString::new(name.as_bytes()).unwrap();
    unsafe {
        property_node_add(f.as_ptr())
    }
}


pub fn add_enum(
    property : &PropertyWidget,
    path : &str,
    types : &str,
    value : &str,
    is_node : bool,
    has_container : bool
    ) -> *const PropertyValue
{
    let f = CString::new(path.as_bytes()).unwrap();
    let types = CString::new(types.as_bytes()).unwrap();
    let v = CString::new(value.as_bytes()).unwrap();

    let pv = unsafe {
        property_list_enum_add(
            f.as_ptr(),
            types.as_ptr(),
            v.as_ptr())

    };

    if !has_container {
        if is_node {
            property.add_node_t(path, pv);
        }
        else {
            property.add_simple_item(path, pv);
        }
    }

    pv
}

pub fn add_enum2(
    path : &str,
    types : &str,
    value : &str
    ) -> *const PropertyValue
{
    let f = CString::new(path.as_bytes()).unwrap();
    let types = CString::new(types.as_bytes()).unwrap();
    let v = CString::new(value.as_bytes()).unwrap();

    unsafe {
        property_list_enum_add(
            f.as_ptr(),
            types.as_ptr(),
            v.as_ptr())
    }
}


/*
fn create_the_widget(
        widget : &PropertyWidget,
        property_show : &PropertyShow,
        all_path : &str,
        field : &str,
        has_container : bool ) -> Option<*const PropertyValue>
{
    if let Some(pv) = property_show.create_widget_itself(field) {

        if !has_container {
            widget.add_simple_item(all_path + field, pv);
            property_show.create_widget_inside(pv, all_path + field);

            None
        }
        else {
            Some(pv)
        }
    }
    else {
        None
    }
}
*/

