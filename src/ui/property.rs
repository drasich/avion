use std::sync::{RwLock, Arc};
use std::collections::HashMap;
use libc::{c_char, c_void, c_int, c_float};
use std::str;
use std::mem;
//use std::collections::{LinkedList,Deque};
use std::collections::{LinkedList};
use std::ptr;
use std::rc::Rc;
use std::cell::{RefCell, BorrowState};
use std::rc::Weak;
use std::any::{Any};//, AnyRefExt};
use std::ffi::CString;
use std::ffi;
use std::ffi::CStr;
use core::marker;

use scene;
use object;
use ui::Window;
use ui;
use property;
use operation;
use control::Control;
use control::WidgetUpdate;
use vec;
use transform;
use resource;
use mesh;
use mesh_render;
use material;
use property::PropertyGet;

#[repr(C)]
pub struct Elm_Object_Item;

pub type ChangedFunc = extern fn(
    object : *const c_void,
    name : *const c_char,
    data : *const c_void);

pub type RegisterChangeFunc = extern fn(
    object : *const c_void,
    name : *const c_char,
    old : *const c_void,
    new : *const c_void,
    action_type : c_int
    );

pub type PropertyTreeFunc = extern fn(
    property : *const Property,
    object : *const c_void,
    parent : *const Elm_Object_Item);

#[repr(C)]
pub struct JkProperty;
#[repr(C)]
pub struct JkPropertySet;
#[repr(C)]
pub struct JkPropertyList;
#[repr(C)]
pub struct PropertyValue;

#[link(name = "png")]

#[link(name = "joker")]
extern {
    /*
    fn window_property_new(window : *const Window) -> *const JkProperty;
    fn property_register_cb(
        property : *const JkProperty,
        changed : extern fn(object : *const c_void, data : *const c_void),
        get : extern fn(data : *const c_void) -> *const c_char
        );

    fn property_data_set(
        property : *const JkProperty,
        data : *const c_void
        );

    fn jk_property_set_new(window : *const Window) -> *const JkPropertySet;
    fn jk_property_set_data_set(set : *const JkPropertySet, data : *const c_void);

    fn jk_property_set_register_cb(
        property : *const JkPropertySet,
        data : *const Property,
        changed_float : ChangedFunc,
        changed_string : ChangedFunc,
        );

    fn property_set_string_add(
        ps : *const JkPropertySet,
        name : *const c_char,
        value : *const c_char
        );

    fn property_set_float_add(
        ps : *const JkPropertySet,
        name : *const c_char,
        value : c_float
        );

    fn property_set_node_add(
        ps : *const JkPropertySet,
        name : *const c_char
        );

    fn property_set_clear(
        ps : *const JkPropertySet);
        */

    fn jk_property_list_new(window : *const Window) -> *const JkPropertyList;

    fn property_list_clear(pl : *const JkPropertyList);

    fn jk_property_list_register_cb(
        property : *const JkPropertyList,
        data : *const Property,
        changed_float : ChangedFunc,
        changed_string : ChangedFunc,
        changed_enum : ChangedFunc,
        register_change_string : RegisterChangeFunc,
        register_change_float : RegisterChangeFunc,
        register_change_enum : RegisterChangeFunc,
        register_change_option : RegisterChangeFunc,
        expand : PropertyTreeFunc,
        contract : PropertyTreeFunc
        );

    fn property_list_group_add(
        pl : *const JkPropertyList,
        name : *const c_char
        );

    fn property_list_node_add(
        pl : *const JkPropertyList,
        name : *const c_char
        ) -> *const PropertyValue;

    fn property_list_nodes_remove(
        pl : *const JkPropertyList,
        name : *const c_char
        );

    fn property_list_float_add(
        ps : *const JkPropertyList,
        name : *const c_char,
        value : c_float
        ) -> *const PropertyValue;

    fn property_list_string_add(
        ps : *const JkPropertyList,
        name : *const c_char,
        value : *const c_char
        ) -> *const PropertyValue;

    fn property_list_enum_add(
        ps : *const JkPropertyList,
        name : *const c_char,
        possible_values : *const c_char,
        value : *const c_char
        ) -> *const PropertyValue;

    fn property_list_option_add(
        ps : *const JkPropertyList,
        name : *const c_char,
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

    fn property_expand(
        pv : *const PropertyValue);

}

pub struct Property
{
    pub name : String,
    pub jk_property_list : *const JkPropertyList,
    pub pv : HashMap<String, *const PropertyValue>,
    control : Rc<RefCell<Control>>,
    expand_state : HashMap<String, bool>,
}

impl Property
{
    pub fn new(
        window : *const Window,
        control : Rc<RefCell<Control>>
        ) -> Box<Property>
    {
        let p = box Property {
            name : String::from_str("property_name"),
            jk_property_list : unsafe {jk_property_list_new(window)},
            pv : HashMap::new(),
            control : control,
            expand_state : HashMap::new()
        };

        unsafe {
            jk_property_list_register_cb(
                p.jk_property_list,
                //mem::transmute(box p.control.clone()),
                &*p, //ptr::null(),
                changed_set_float,
                changed_set_string,
                changed_set_enum,
                register_change_string,
                register_change_float,
                register_change_enum,
                register_change_option,
                expand,
                contract
                ); 
        }

        p
    }

    pub fn set_object(&mut self, o : &object::Object)
    {
        unsafe { property_list_clear(self.jk_property_list); }
        self.pv.clear();

        unsafe {
            property_list_group_add(
                self.jk_property_list,
                CString::new("object".as_bytes()).unwrap().as_ptr());
        }
        let mut v = Vec::new();
        v.push("object".to_string());
        o.create_widget(self, "object", 1);
    }

    pub fn set_nothing(&mut self)
    {
        unsafe { property_list_clear(self.jk_property_list); }
        self.pv.clear();
    }

    pub fn data_set(&self, data : *const c_void)
    {
        //TODO
        //unsafe { property_data_set(self.jk_property, data); }
    }

    pub fn update_object_property(&self, object : &PropertyShow, prop : &str)
    {
        for (f,pv) in self.pv.iter() {
            if f.as_slice().starts_with(prop) {
                let yep = make_vec_from_string(f).tail().to_vec();
                match find_property_show(object, yep.clone()) {
                    Some(ppp) => {
                        ppp.update_widget(*pv);
                    },
                    None => {}
                }
            }
        }
    }

    pub fn update_object(&self, object : &PropertyShow, but : &str)
    {
        for (f,pv) in self.pv.iter() {
            println!("UPDATEOBJECt contains property : Val : {}", f);
        }

        for (f,pv) in self.pv.iter() {
            if f.as_slice() == but {
                println!("buuuuuuuuuuuuuuuuuuuuuuuuuut: {} ", f);
                continue;
            }
            let yep = make_vec_from_string(f).tail().to_vec();
            match find_property_show(object, yep.clone()) {
                Some(ppp) => {
                    println!("I find the property : {:?}", yep);
                    ppp.update_widget(*pv);
                },
                None => {
                    println!("could not find prop : {:?}", yep);
                }
            }
        }
    }

    pub fn add_node(&mut self, ps : &PropertyShow, name : &str) {
        let f = CString::new(name.as_bytes()).unwrap();
        let pv = unsafe {
            property_list_node_add(
                self.jk_property_list,
                f.as_ptr()
                )
        };
        
        let es = match self.expand_state.get(&name.to_string()) {
            Some(b) => *b,
            None => return,
        };

        if es {
            unsafe {
                property_expand(pv);
            }
        }
    }

    pub fn add_enum(
        &mut self,
        ps : &PropertyShow,
        path : &str,
        types : &str,
        value : &str
        )
    {
        let f = CString::new(path.as_bytes()).unwrap();
        let types = CString::new(types.as_bytes()).unwrap();
        let v = CString::new(value.as_bytes()).unwrap();

        let pv = unsafe {
            property_list_enum_add(
                self.jk_property_list,
                f.as_ptr(),
                types.as_ptr(),
                v.as_ptr())

        };

        if pv != ptr::null() {
            self.pv.insert(path.to_string(), pv);
        }

        let es = match self.expand_state.get(&path.to_string()) {
            Some(b) => *b,
            None => return,
        };

        if es {
            unsafe {
                property_expand(pv);
            }
        }
    }
}

pub extern fn name_get(data : *const c_void) -> *const c_char {

    let o : &Arc<RwLock<object::Object>> = unsafe {
        mem::transmute(data)
    };

    let cs = CString::new(o.read().unwrap().name.as_bytes()).unwrap();
    //println!("..........name get {:?}", cs);
    cs.as_ptr()
}

pub extern fn changed_set_float(
    property : *const c_void,
    name : *const c_char,
    data : *const c_void) {

    let f : & f64 = unsafe {mem::transmute(data)};
    changed_set(property, name, None, f, 0);
}

pub extern fn changed_set_string(
    property : *const c_void,
    name : *const c_char,
    data : *const c_void) {

    let datachar = data as *const i8;
    let s = unsafe {CStr::from_ptr(datachar).to_bytes()};
    let ss = match str::from_utf8(s) {
        Ok(sss) => sss.to_string(),
        _ => {
            return;
        }
    };
    //println!("the string is {}", ss);
    changed_set(property, name, None, &ss, 0);
}

pub extern fn changed_set_enum(
    property : *const c_void,
    name : *const c_char,
    data : *const c_void) {
    println!("DOES NOT NO ANYTHING");
}


pub extern fn register_change_string(
    property : *const c_void,
    name : *const c_char,
    old : *const c_void,
    new : *const c_void,
    action : c_int
    ) {

    let newchar = new as *const i8;
    let s = unsafe {CStr::from_ptr(newchar).to_bytes()};
    let ss = match str::from_utf8(s) {
        Ok(sss) => sss.to_string(),
        _ => {
            println!("error");
            return;
        }
    };

    //println!("the string is {}", ss);
    if action == 1 && old != ptr::null() {
        let oldchar = old as *const i8;
        let so = unsafe {CStr::from_ptr(oldchar).to_bytes()};
        let sso = match str::from_utf8(so) {
            Ok(ssso) => ssso.to_string(),
            _ => {
                println!("error");
                return;
            }
        };

        changed_set(property, name, Some(&sso), &ss, action);
    }
    else {
        changed_set(property, name, None, &ss, action);
    }
}

pub extern fn register_change_float(
    property : *const c_void,
    name : *const c_char,
    old : *const c_void,
    new : *const c_void,
    action : c_int
    ) {

    let fnew : & f64 = unsafe {mem::transmute(new)};

    if action == 1 && old != ptr::null() {
        let fold : & f64 = unsafe {mem::transmute(old)};
        changed_set(property, name, Some(fold), fnew, action);
    }
    else {
        changed_set(property, name, None, fnew, action);
    }
}

pub extern fn register_change_enum(
    property : *const c_void,
    name : *const c_char,
    old : *const c_void,
    new : *const c_void,
    action : c_int
    ) {

    let newchar = new as *const i8;
    let s = unsafe {CStr::from_ptr(newchar).to_bytes()};
    let ss = match str::from_utf8(s) {
        Ok(sss) => sss.to_string(),
        _ => {
            println!("error");
            return
        }
    };

    //println!("the string is {}", ss);
    if action == 1 && old != ptr::null() {
        let oldchar = old as *const i8;
        let so = unsafe {CStr::from_ptr(oldchar).to_bytes()};
        let sso = match str::from_utf8(so) {
            Ok(ssso) => ssso.to_string(),
            _ => {
                println!("error");
                return
            }
        };
        changed_set(property, name, Some(&sso), &ss, action);
    }
    else {
        changed_set(property, name, None, &ss, action);
    }
}

pub extern fn register_change_option(
    property : *const c_void,
    name : *const c_char,
    old : *const c_void,
    new : *const c_void,
    action : c_int
    ) {

    let newchar = new as *const i8;
    let s = unsafe {CStr::from_ptr(newchar).to_bytes()};
    let ss = match str::from_utf8(s) {
        Ok(sss) => sss.to_string(),
        _ => {
            println!("error");
            return
        }
    };

    //println!("the string is {}", ss);
    if old == ptr::null() {
        println!("old is null, return");
        return;
    }

    let oldchar = old as *const i8;
    let so = unsafe {CStr::from_ptr(oldchar).to_bytes()};
    let sso = match str::from_utf8(so) {
        Ok(ssso) => ssso.to_string(),
        _ => {
            println!("error");
            return
        }
    };

    changed_option(property, name, &sso, &ss);
}


//fn changed_set(property : *const c_void, name : *const c_char, data : &Any) {
fn changed_set<T : Any+Clone+PartialEq>(
    property : *const c_void,
    name : *const c_char,
    old : Option<&T>,
    new : &T,
    action : c_int
    ) {
    let s = unsafe {CStr::from_ptr(name).to_bytes()};

    let path = match str::from_utf8(s) {
        Ok(pp) => pp,
        _ => {
            println!("problem with the path");
            return;}
    };

    println!("I changed the value {} ", path);

    let v: Vec<&str> = path.split('/').collect();

    let mut vs = Vec::new();
    for i in v.iter()
    {
        vs.push(i.to_string());
    }

    //let vs = vs.tail().to_vec();

    let p : & Property = unsafe {mem::transmute(property)};

    let mut control = match p.control.borrow_state() {
        BorrowState::Unused => p.control.borrow_mut(),
        _ => { println!("cannot borrow control"); return; }
    };

    let change = match (old, action) {
        (Some(oldd), 1) => {
            println!(".....adding operation!!");
            control.request_operation_old_new(
                vs,
                box oldd.clone(),
                box new.clone())
        },
        _ => {
            control.request_direct_change(vs, new)
        }
    };

    let id = if let Some(o) = control.get_selected_object(){
        o.read().unwrap().id.clone()
    }
    else { 
        return;
    };

    match change {
        operation::Change::DirectChange(s) |
        operation::Change::Objects(s, _) => {
            if s == "object/name" {
                match control.tree {
                    Some(ref t) =>
                        match t.borrow_state() {
                            BorrowState::Writing => {}
                            _ => {
                                t.borrow().update_object(&id);
                            },
                        },
                        None => {}
                };
            }
        },
        _ => {}
    }
            
}

fn changed_option(
//fn changed_option<T : Any+Clone+PartialEq>(
    property : *const c_void,
    name : *const c_char,
    old : &str,
    new : &str
    ) {
    let s = unsafe {CStr::from_ptr(name).to_bytes()};

    let path = match str::from_utf8(s) {
        Ok(pp) => pp,
        _ => {
            println!("problem with the path");
            return;}
    };

    println!("OPTIONNNNNNNNNN I changed the value {} ", path);

    let v: Vec<&str> = path.split('/').collect();

    let mut vs = Vec::new();
    for i in v.iter()
    {
        vs.push(i.to_string());
    }

    //let vs = vs.tail().to_vec();

    let p : & Property = unsafe {mem::transmute(property)};

    let mut control = match p.control.borrow_state() {
        BorrowState::Unused => p.control.borrow_mut(),
        _ => { println!("cannot borrow control"); return; }
    };

    /*
    let (id, prop) = if let Some(o) = control.get_selected_object(){
        let p : Option<Box<Any>> = o.read().unwrap().get_property_hier(path);
        (o.read().unwrap().id.clone(),p)
    }
    else { 
        return;
    };
    */

    let change = if new == "Some" {
        control.request_operation_option_to_some(vs)
    }
    else {
        control.request_operation_option_to_none(path)
        /*
        if let Some(propget) = prop {
                let test = *propget;
            control.request_operation_option_to_none(
                vs,
                box test.clone())
        }
        else {
            return
        }
        */
    };

    /*
    match change {
        operation::Change::DirectChange(s) |
        operation::Change::Objects(s, _) => {
            if s == "object/name" {
                match control.tree {
                    Some(ref t) =>
                        match t.borrow_state() {
                            BorrowState::Writing => {}
                            _ => {
                                t.borrow().update_object(&id);
                            },
                        },
                        None => {}
                };
            }
        },
        _ => {}
    }
    */
            
}



extern fn expand(
    property: *const Property,
    data : *const c_void,
    parent : *const Elm_Object_Item) -> ()
{
    let datachar = data as *const i8;
    let s = unsafe {CStr::from_ptr(datachar).to_bytes()};
    let mut p : &mut Property = unsafe {mem::transmute(property)};

    //println!("expanding ! property name {} ", p.name);

    let path = match str::from_utf8(s) {
        Ok(pp) => pp,
        _ => {
            println!("problem with the path");
            return;}
    };

    println!("I expand the value {} ", path);

    let vs = make_vec_from_string(&path.to_string());

    let yep = vs.tail().to_vec();
    println!("expand : {:?}", vs);

    match p.control.clone().borrow_state() {
        BorrowState::Writing => {
            println!("cannot borrow control, might be already borrowed...");
            return;
        },
        _ => {
            let o = match p.control.borrow().get_selected_object() {
                Some(ob) => ob,
                None => {
                    println!("no selected objectttttttttttttt");
                    return;
                }
            };

            let or  = o.read().unwrap();
            match find_property_show(&*or, yep.clone()) {
                Some(ppp) => {
                    //p.create_entries(&*ppp, vs.clone());
                    println!("I found and create {:?} ", vs);
                    ppp.create_widget(p, path , 1);
                    p.expand_state.insert(path.to_string(), true);
                },
                None => {
                    println!("could not find property {:?} ", vs);
                }
            }
        },
    };

}

extern fn contract(
    property: *const Property,
    data : *const c_void,
    parent : *const Elm_Object_Item) -> ()
{
    let mut p : &mut Property = unsafe {mem::transmute(property)};

    unsafe {
        property_list_nodes_remove(
            p.jk_property_list,
            data as *const c_char
            //s.unwrap(),
            );
    };

    let datachar = data as *const i8;
    let s = unsafe {CStr::from_ptr(datachar).to_bytes()};
    let path = match str::from_utf8(s) {
        Ok(pp) => pp,
        _ => {
            println!("problem with the path");
            return;}
    };

    println!("I contract the path {} ", path);

    p.expand_state.insert(path.to_string(), false);

    let vs = make_vec_from_string(&path.to_string());

    let yep = vs.tail().to_vec();
    println!("contract : {:?}", vs);

    let clone = p.pv.clone();

    for (key,pv) in clone.iter() {
        println!("cccccccccccccccccontract  start with key '{}' ", key);
        if key.as_slice().starts_with(path) && key.as_slice() != path  {
            println!("yes, '{}' starts with '{}'", key, path);
            match p.pv.remove(key) {
                Some(_) => println!("yes I removed {}", key),
                None => println!("could not find {}", key)
            }
        }
    }
}

impl WidgetUpdate for Property
{
    fn update_changed(
        &mut self,
        name : &str,
        new : &Any)
    {

        //println!("property update changed {}", name);

        let pv = match self.pv.get(&name.to_string()) {
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

pub trait PropertyShow
{
    fn create_widget(
        &self,
        property : &mut Property,
        field : &str,
        depth : i32
        );

    fn update_widget(&self, pv : *const PropertyValue) {
        println!("update_widget not implemented for this type");
    }

    fn get_property(&self, field : &str) -> Option<&PropertyShow>
    {
        return None;
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

    fn create_widget(&self, property : &mut Property, field : &str, depth : i32)
    {
        println!("adding field : {}", field);
        let f = CString::new(field.as_bytes()).unwrap();
        unsafe {
            let pv = property_list_float_add(
                property.jk_property_list,
                f.as_ptr(),
                *self as c_float);
            if pv != ptr::null() {
                property.pv.insert(field.to_string(), pv);
            }
        }
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

    fn create_widget(&self, property : &mut Property, field : &str, depth : i32)
    {
        let f = CString::new(field.as_bytes()).unwrap();
        let v = CString::new(self.as_bytes()).unwrap();

        unsafe {
            let pv = property_list_string_add(
                property.jk_property_list,
                f.as_ptr(),
                v.as_ptr());
            if pv != ptr::null() {
                property.pv.insert(field.to_string(), pv);
            }
        }
    }

    fn update_widget(&self, pv : *const PropertyValue) {
        let v = CString::new(self.as_bytes()).unwrap();
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
        property : &mut Property,
        field : &str,
        depth : i32)
    {
        (**self).create_widget(property ,field, depth);
    }

    fn get_property(&self, field : &str) -> Option<&PropertyShow>
    {
        (**self).get_property(field)
    }
}

impl<T : PropertyShow> PropertyShow for Option<T> {

    fn create_widget(
        &self,
        property : &mut Property,
        field : &str,
        depth : i32)
    {
        if depth == 0 {
            let f = CString::new(field.as_bytes()).unwrap();
            let type_value = match *self {
                Some(_) => "Some",
                None => "None"
            };

            println!(".......type value : {}", type_value);
            let v = CString::new(type_value.as_bytes()).unwrap();

            unsafe {
                let pv = property_list_option_add(
                    property.jk_property_list,
                    f.as_ptr(),
                    v.as_ptr());

                if pv != ptr::null() {
                    println!("ADDING : {}", field);
                    property.pv.insert(field.to_string(), pv);
                }
            }
        }

        if depth == 1 {
            match *self {
                Some(ref s) =>  {
                    s.create_widget(property, field, depth);
                },
                None => {}
            };
        }
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
}

impl<T> PropertyShow for resource::ResTT<T>
{
    fn create_widget(
        &self,
        property : &mut Property,
        field : &str,
        depth : i32)
    {
        if depth < 0 {
            return;
        }

        if depth == 0 && field != ""
        {
            property.add_node(self, field);
        }

        if depth > 0 {
            let s = field.to_string() + "/name";
            self.name.create_widget(property, s.as_slice(), depth-1);
        }
    }

    fn get_property(&self, field : &str) -> Option<&PropertyShow>
    {
        match field {
            "name" => Some(&self.name as &PropertyShow),
            _ => None
        }
    }
}

macro_rules! property_show_impl(
    ($my_type:ty, [ $($member:ident),+ ]) => ( 

        impl PropertyShow for $my_type
        {
            fn create_widget(
                &self,
                property : &mut Property,
                field : &str,
                depth : i32)
            {
                if depth < 0 {
                    return;
                }

                if depth == 0 && field != ""
                {
                    property.add_node(self, field);
                }

                if depth > 0 {
                $(
                    let s = field.to_string()
                    + "/"//.to_string()
                    + stringify!($member);//.to_string();
                    self.$member.create_widget(property, s.as_slice(), depth-1);
                 )+
                }
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
        }
    )
);

property_show_impl!(vec::Vec3,[x,y,z]);
property_show_impl!(vec::Quat,[x,y,z,w]);
property_show_impl!(transform::Transform,[position,orientation]);
property_show_impl!(mesh_render::MeshRender,[mesh,material]);
property_show_impl!(object::Object,
                     [name,position,orientation,scale,mesh_render]);

fn join_string(path : &Vec<String>) -> String
{
    let mut s = String::new();
    let mut first = true;
    for v in path.iter() {
        if !first {
            s.push('/');
        }
        s.push_str(v.as_slice());
        first = false;
    }

    s
}

fn make_vec_from_string(s : &String) -> Vec<String>
{
    let v: Vec<&str> = s.split('/').collect();

    let mut vs = Vec::new();
    for i in v.iter()
    {
        vs.push(i.to_string());
    }

    vs
}

pub fn find_property_show(p : &PropertyShow, path : Vec<String>) -> 
Option<&PropertyShow>
{
    //let vs = make_vec_from_string(&field.to_string());

    match path.len() {
        0 =>  None,
        1 => p.get_property(path[0].as_slice()),
        _ => { 
             match p.get_property(path[0].as_slice()) {
                 Some(ppp) => {
                     find_property_show(ppp, path.tail().to_vec())
                 },
                 None => {
                     None
                 }
             }
        }
    }
}
