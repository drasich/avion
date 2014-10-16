use libc::{c_char, c_void};
use render;
use object;
use std::collections::{DList};

use std::mem;
use sync::{RWLock, Arc};
use std::c_str::CString;
use std::ptr;
use scene;
use property::TProperty;
use property;

#[repr(C)]
pub struct Tree;
#[repr(C)]
pub struct Property;
#[repr(C)]
pub struct Window;

#[link(name = "joker")]
extern {
    //fn simple_window_init();
    pub fn elm_simple_window_main();
    fn tree_widget_new() -> *const Tree;
    fn tree_register_cb(
        tree : *const Tree,
        name_get : extern fn(data : *const c_void) -> *const c_char,
        select : extern fn(data : *const c_void) -> (),
        can_expand : extern fn(data : *const c_void) -> bool,
        expand : extern fn(tree: *const Tree, data : *const c_void, parent: *const c_void) -> (),
        );

    fn tree_object_add(
        tree : *const Tree,
        object : *const c_void,
        parent : *const c_void,
        );
    fn window_new() -> *const Window;
    fn window_tree_new(window : *const Window) -> *const Tree;
    fn window_button_new(window : *const Window);
    fn window_property_new(window : *const Window) -> *const Property;

    pub fn init_callback_set(
        cb: extern fn(*mut Master) -> (),
        master: *const Master 
        ) -> ();

    /*
    fn property_object_set(
        Property : *mut Property,
        object : *const c_void
        );

    fn property_object_update(
        Property : *mut Property
        );
        */

    /*
    fn property_set(
        Property : *mut Property,
        name : *const c_char,
        value : *const c_char
        );
        */

    fn property_register_cb(
        Property : *const Property,
        changed : extern fn(object : *const c_void, data : *const c_void),
        get : extern fn(data : *const c_void) -> *const c_char
        );

    fn property_data_set(
        Property : *const Property,
        data : *const c_void
        );
}

pub struct Master
{
    //windows : DList<Window>
    pub window : Option<*const Window>,
    pub tree : Option<*const Tree>,
    pub property : Option<*const Property>,
    pub scene : Option<Arc<RWLock<scene::Scene>>>,
}

impl Master
{
    pub fn new() -> Master
    {
        Master {
            window : None,
            tree : None,
            property : None,
            scene : None
        }
    }
}

pub extern fn name_get(data : *const c_void) -> *const c_char {

    let o : &Arc<RWLock<object::Object>> = unsafe {
        mem::transmute(data)
    };

    let cs = o.read().name.to_c_str();

    unsafe {
        cs.unwrap()
    }
}

pub extern fn select(data : *const c_void) -> () {
    let o : &Arc<RWLock<object::Object>> = unsafe {
        mem::transmute(data)
    };
    println!("select ! {} ", o.read().name);
}

pub extern fn can_expand(data : *const c_void) -> bool {
    let o : &Arc<RWLock<object::Object>> = unsafe {
        mem::transmute(data)
    };

    println!("can expand :{}", o.read().children.is_empty());
    return !o.read().children.is_empty();
}

pub extern fn expand(tree: *const Tree, data : *const c_void, parent : *const c_void) -> () {
    let o : &Arc<RWLock<object::Object>> = unsafe {
        mem::transmute(data)
    };

    println!("expanding ! {} ", o.read().name);

    for c in o.read().children.iter() {
        println!("expanding ! with child {} ", (*c).read().name);
        unsafe {
            tree_object_add(tree, mem::transmute(c), parent);
        }
    }
}

struct PropertySet
{
    name : String
}

trait PropertyTest
{
    fn set(property_set :&PropertySet, field : String, value : Self);
    
}

impl PropertyTest for int
{
    fn set(property_set: &PropertySet, field : String, value : int)
    {
        let test = value;
    }
}

pub extern fn changed(object : *const c_void, data : *const c_void) {
    let o : &Arc<RWLock<object::Object>> = unsafe {
        mem::transmute(object)
    };

    let s = unsafe {CString::new(data as *const i8, false) };
    println!("data changed : {}", s);

    match s.as_str() {
        Some(ss) => {
            let sss = property::SString(String::from_str(ss));
            o.write().set_property("name", &sss);
        },
        _ => ()
    }

}


pub extern fn init_cb(master: *mut Master) -> () {
    unsafe {
        let w = window_new();
        let t = window_tree_new(w);
        tree_register_cb(
            t,
            name_get,
            select,
            can_expand,
            expand);

        let p = window_property_new(w);
        
        property_register_cb(
            p,
            changed,
            name_get
            ); 
    
        match (*master).scene {
            Some(ref s) => {
                for o in s.read().objects.iter() {
                    tree_object_add(t, mem::transmute(o), ptr::null());
                }

                let oo = s.read().object_find("yep");
                match oo {
                    Some(ref o) => { println!("I found the obj in init_cb");
                        property_data_set(p, mem::transmute(o));
                    }
                    None => {}
                };
            }
            None => {}
        };


        //window_button_new(w);

        (*master).window = Some(w);
        (*master).tree = Some(t);
        (*master).property = Some(p);
    }
}

pub struct PropertyWidget {
    name : String,
}

trait PropertyShow
{
    //fn create_widget() -> Widget;
    fn create_widget(window : &Window);
}

impl PropertyShow for String
{
    fn create_widget(window : &Window)
    {
        let c = unsafe { window_property_new(window) };
    }
}

