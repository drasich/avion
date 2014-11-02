use sync::{RWLock, Arc};
use std::collections::HashMap;
use libc::{c_char, c_void, c_int};
use std::mem;
use std::c_str::CString;
//use std::collections::{DList,Deque};
use std::ptr;
use std::cell::RefCell;
use std::rc::Weak;

use scene;
use object;
use ui::Window;
use ui::Master;
use ui;
use property;
use property::TProperty;

#[repr(C)]
pub struct JkProperty;

#[link(name = "joker")]
extern {
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
}

pub struct Property
{
    pub name : String,
    //TODO change the key
    jk_property : *const JkProperty,
    master : Weak<RefCell<ui::Master>>,
    dont_forward_signal : bool
}

impl Property
{
    pub fn new(
        window : *const Window,
        master : Weak<RefCell<ui::Master>>) -> Box<Property>
    {
        let p = box Property {
            name : String::from_str("property_name"),
            jk_property : unsafe {window_property_new(window)},
            master : master,
            dont_forward_signal : false
        };

        unsafe {
            property_register_cb(
                p.jk_property,
                changed,
                name_get
                ); 
        }

        p
    }

    pub fn data_set(&self, data : *const c_void)
    {
        unsafe { property_data_set(self.jk_property, data); }
    }

}

pub extern fn name_get(data : *const c_void) -> *const c_char {

    let o : &Arc<RWLock<object::Object>> = unsafe {
        mem::transmute(data)
    };

    //println!("name get {:?}", o);

    let cs = o.read().name.to_c_str();

    unsafe {
        cs.unwrap()
    }
}


pub extern fn changed(object : *const c_void, data : *const c_void) {
    let o : &Arc<RWLock<object::Object>> = unsafe {
        mem::transmute(object)
    };

    let s = unsafe {CString::new(data as *const i8, false) };

    match s.as_str() {
        Some(ss) => {
            let sss = property::SString(String::from_str(ss));
            o.clone().write().set_property("name", &sss);
        },
        _ => ()
    }

}

