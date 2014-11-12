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

#[repr(C)]
pub struct Elm_Object_Item;

pub type ChangedFunc = extern fn(
    object : *const c_void,
    name : *const c_char,
    data : *const c_void);

pub type PropertyTreeExpandFunc = extern fn(
    property : *const Property,
    object : *const c_void,
    parent : *const Elm_Object_Item);

#[repr(C)]
pub struct JkProperty;
#[repr(C)]
pub struct JkPropertySet;
#[repr(C)]
pub struct JkPropertyList;


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

    fn jk_property_list_new(window : *const Window) -> *const JkPropertyList;
    fn property_list_clear(pl : *const JkPropertyList);
    fn jk_property_list_register_cb(
        property : *const JkPropertyList,
        data : *const Property,
        changed_float : ChangedFunc,
        changed_string : ChangedFunc,
        expand : PropertyTreeExpandFunc
        );

    fn property_list_node_add(
        pl : *const JkPropertyList,
        name : *const c_char
        );

    fn property_list_group_add(
        pl : *const JkPropertyList,
        name : *const c_char
        );

    fn property_list_float_add(
        ps : *const JkPropertyList,
        name : *const c_char,
        value : c_float
        );
}

pub struct Property
{
    pub name : String,
    //TODO change the key
    //jk_property : *const JkProperty,
    jk_property_set : *const JkPropertySet,
    jk_property_list : *const JkPropertyList,
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
            jk_property_list : unsafe {jk_property_list_new(window)},
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

        unsafe {
            jk_property_list_register_cb(
                p.jk_property_list,
                &*p, //ptr::null(),
                changed_set_float,
                changed_set_string,
                expand
                ); 
        }

        p
    }

    pub fn set_object(&self, o : &object::Object)
    {
        unsafe { property_set_clear(self.jk_property_set); }
        unsafe { property_list_clear(self.jk_property_list); }

        unsafe {
            property_list_group_add(
                self.jk_property_list,
                "object".to_c_str().unwrap());
        }
        let mut v = Vec::new();
        v.push("object".to_string());
        self.create_entries(o, v);//Vec::new());
    }

    //*
    //fn find_property<'s>(p : &'s ChrisProperty, path : Vec<String>) 
    fn find_property(p : &ChrisProperty, path : Vec<String>) 
        -> Option<Box<ChrisProperty>>
        //-> Option<Box<ChrisProperty+'static>>
    //fn find_property(p : &ChrisProperty, path : Vec<String>)
    {
        /*
        if path.is_empty() {
            return Some(p);
            //return;
        }
        */
        
        /*
        for field in p.cfields().iter() {
            match p.cget_property(field.as_slice()) {
                property::ChrisNone => {return None;},
                property::BoxChrisProperty(bp) => 
                {
                    return
                    Property::find_property(&*bp, path.tail().to_vec());
                },
                _ => {}
            }

        }
        */

        match p.cget_property_hier(path) {
            property::ChrisNone => {return None;},
            property::BoxChrisProperty(bp) => 
            {
                return Some(bp);
            },
            _ => {
                println!("find prop, must be box any...");
            }
        }

        return None;
    }
    //*/

    fn create_entries(
            &self,
            o : &ChrisProperty,
            path : Vec<String>)
        {

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

        //TODO this function for jk_property_list
            println!("entries!!! {}", path);
            for field in o.cfields().iter()
            {
                match o.cget_property(field.as_slice()) {
                    property::BoxAny(p) => {
                        match p.downcast_ref::<String>() {
                            Some(s) => {
                                let field = create_node_path(
                                    &path,
                                    field.as_slice());
                                let v = s.to_c_str();
                                let f = field.to_c_str();
                                unsafe {
                                    property_set_string_add(
                                        self.jk_property_set,
                                        f.unwrap(),
                                        v.unwrap());
                                }
                            },
                            None => {}
                        }
                        match p.downcast_ref::<f64>() {
                            Some(v) => {
                                let field = create_node_path(
                                    &path,
                                    field.as_slice());
                                let f = field.to_c_str();
                                /*
                                unsafe {
                                    property_set_float_add(
                                        self.jk_property_set,
                                        f.unwrap(),
                                        *v as c_float);
                                }
                                */
                                unsafe {
                                    property_list_float_add(
                                        self.jk_property_list,
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
                        /*
                        unsafe {
                        property_set_node_add(
                            self.jk_property_set,
                            f.unwrap());
                        }
                        */
                        unsafe {
                        property_list_node_add(
                            self.jk_property_list,
                            f.unwrap());
                        }
                        /*
                        let mut yep = path.clone();
                        yep.push(field.clone());
                        self.create_entries(&*p, yep);
                        */
                    },
                    property::ChrisNone => {}
                }

            }
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
        None => {
            println!("problem with the path");
            return;}
    };

    let v: Vec<&str> = path.split('/').collect();

    let mut vs = Vec::new();
    for i in v.iter()
    {
        vs.push(i.to_string());
    }

    let vs = vs.tail().to_vec();

    let p : & Property = unsafe {mem::transmute(property)};

    match p.master.upgrade() {
        Some(m) => { 
            match m.try_borrow() {
                Some(ref mm) => {
                    match mm.render.objects_selected.front() {
                        Some(o) => {
                            o.write().cset_property_hier(vs, data);
                        },
                        None => {
                            println!("no objetcs selected");
                        }
                    }
                },
                _ => { println!("already borrowed : mouse_up add_ob ->sel ->add_ob")}
            }
        },
        None => { println!("the master of the property doesn't exist anymore");}
    }
}


extern fn expand(
    property: *const Property,
    data : *const c_void,
    parent : *const Elm_Object_Item) -> ()
{
    let s = unsafe {CString::new(data as *const i8, false) };
    let mut p : &mut Property = unsafe {mem::transmute(property)};

    println!("expanding ! property name {} ", p.name);

    let s = unsafe {CString::new(data as *const i8, false) };
    println!("I changed the value {} ", s);

    let path = match s.as_str() {
        Some(pp) => pp,
        None => {
            println!("problem with the path");
            return;}
    };

    let v: Vec<&str> = path.split('/').collect();

    let mut vs = Vec::new();
    for i in v.iter()
    {
        vs.push(i.to_string());
        println!("pushing {}", i);
    }

    let yep = vs.tail().to_vec();

    match p.master.upgrade() {
        Some(m) => { 
            match m.try_borrow() {
                Some(ref mm) => {
                    match mm.render.objects_selected.front() {
                        Some(o) => {
                            //TODO
                            //match Property::find_property(&*o.read(), vs.clone()) {
                            match Property::find_property(&*o.read(), yep.clone()) {
                                Some(ppp) => {
                                    //p.create_entries(&*o.read(), vs);
                                    p.create_entries(&*ppp, vs.clone());
                                },
                                None => {
                                    println!("could not find property {} ", vs);
                                }
                            }
                        },
                        None => {
                            println!("no objetcs selected");
                        }
                    }
                },
                _ => { println!("already borrowed : mouse_up add_ob ->sel ->add_ob")}
            }
        },
        None => { println!("the master of the property doesn't exist anymore");}
    }
}

