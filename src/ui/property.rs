use sync::{RWLock, Arc};
use std::collections::HashMap;
use libc::{c_char, c_void, c_int, c_float};
use std::mem;
use std::c_str::CString;
use std::collections::{DList,Deque};
use std::ptr;
use std::cell::RefCell;
use std::rc::Weak;
use std::any::{Any, AnyRefExt};

use scene;
use object;
use ui::Window;
use ui::Master;
use ui;
use property;
use property::TProperty;
use property::ChrisProperty;

pub type ChangedFunc = extern fn(
    object : *const c_void,
    name : *const c_char,
    data : *const c_void);

#[repr(C)]
pub struct JkProperty;
#[repr(C)]
pub struct JkPropertySet;


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

    fn jk_property_set_new(window : *const Window) -> *const JkPropertySet;
    fn jk_property_set_data_set(set : *const JkPropertySet, data : *const c_void);

    fn chris_test(changed : ChangedFunc);

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
}

pub struct Property
{
    pub name : String,
    //TODO change the key
    //jk_property : *const JkProperty,
    jk_property_set : *const JkPropertySet,
    master : Weak<RefCell<ui::Master>>,
}


pub struct yep
{
    f : ChangedFunc
}

impl Property
{
    pub fn new(
        window : *const Window,
        master : Weak<RefCell<ui::Master>>) -> Box<Property>
    {
        let p = box Property {
            name : String::from_str("property_name"),
            //jk_property : unsafe {window_property_new(window)},
            jk_property_set : unsafe {jk_property_set_new(window)},
            master : master,
        };

        //TODO
        /*
        unsafe {
            property_register_cb(
                p.jk_property,
                changed,
                name_get
                ); 
        }
        */
        unsafe {
            jk_property_set_register_cb(
                p.jk_property_set,
                &*p, //ptr::null(),
                changed_set_float,
                changed_set_string,
                ); 
        }

        p
    }

    pub fn set_object(&self, o : &object::Object)
    {
        unsafe { property_set_clear(self.jk_property_set); }

        fn get_node_path(path : &Vec<String>) -> String
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

        fn create_node_path(path : &Vec<String>, field : &str) -> String
        {
            let mut s = get_node_path(path);
            if !path.is_empty(){
                s.push('/');
            }
            s.push_str(field);

            s
        }
    

        fn create_entries(property: &Property, o : &ChrisProperty, path : Vec<String>)
        {
            println!("entries!!! {}", path);
            for field in o.cfields().iter()
            {
                match o.cget_property(field.as_slice()) {
                    property::BoxAny(p) => {
                        match p.downcast_ref::<String>() {
                            Some(s) => {
                                let field = create_node_path(&path, field.as_slice());
                                let v = s.to_c_str();
                                let f = field.to_c_str();
                                unsafe {
                                    property_set_string_add(
                                        property.jk_property_set,
                                        f.unwrap(),
                                        v.unwrap());
                                }
                            },
                            None => {}
                        }
                        match p.downcast_ref::<f64>() {
                            Some(v) => {
                                let field = create_node_path(&path, field.as_slice());
                                let f = field.to_c_str();
                                unsafe {
                                    property_set_float_add(
                                        property.jk_property_set,
                                        f.unwrap(),
                                        *v as c_float);
                                }
                            },
                            None => {}
                        }
                        //TODO other type
                    },
                    property::BoxChrisProperty(p) => {
                        let field = create_node_path(&path, field.as_slice());
                        println!("I come here and field is : {}", field);
                        let f = field.to_c_str();
                        unsafe {
                        property_set_node_add(
                            property.jk_property_set,
                            f.unwrap());
                        }
                        let mut yep = path.clone();
                        yep.push(field.clone());
                        create_entries(property, &*p, yep);
                    },
                    property::ChrisNone => {}
                }

            }
        }

        create_entries(self, o, Vec::new());
    }

    pub fn data_set(&self, data : *const c_void)
    {
        //TODO
        //unsafe { property_data_set(self.jk_property, data); }
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

pub extern fn changed_set_float(property : *const c_void, name : *const c_char, data : *const c_void) {

    let f : & f64 = unsafe {mem::transmute(data)};
    changed_set(property, name, f);
}

pub extern fn changed_set_string(property : *const c_void, name : *const c_char, data : *const c_void) {

    let s = unsafe {CString::new(data as *const i8, false) };
    let ss = match s.as_str() {
        Some(sss) => sss.to_string(),
        None => return
    };
    //println!("the string is {}", ss);
    changed_set(property, name, &ss);
}


fn changed_set(property : *const c_void, name : *const c_char, data : &Any) {
    let s = unsafe {CString::new(name as *const i8, false) };
    println!("I changed the value {} ", s);

    let path = match s.as_str() {
        Some(pp) => pp,
        None => return
    };

    let v: Vec<&str> = path.split('/').collect();

    let mut vs = Vec::new();
    for i in v.iter()
    {
        vs.push(i.to_string());
    }


    let p : & Property = unsafe {mem::transmute(property)};

    match p.master.upgrade() {
        Some(m) => { 
            match m.try_borrow() {
                Some(ref mm) => {
                    match mm.render.objects_selected.front() {
                        Some(o) => {
                            o.write().cset_property_hier(vs, data);
                        },
                        None => {}
                    }
                },
                _ => { println!("already borrowed : mouse_up add_ob ->sel ->add_ob")}
            }
        },
        None => { println!("the master of the property doesn't exist anymore");}
    }
}


