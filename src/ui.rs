use libc::{c_char, c_void, c_int};
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
pub struct JkProperty;
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
    fn window_property_new(window : *const Window) -> *const JkProperty;
    fn window_callback_set(
        window : *const Window,
        data: *const c_void,
        mouse_down : extern fn(
            data : *const c_void,
            modifier : *const c_char,
            button : c_int,
            x : c_int, 
            y : c_int,
            timestamp : c_int
            ),
        mouse_up : extern fn(
            data : *const c_void,
            modifier : *const c_char,
            button : c_int,
            x : c_int, 
            y : c_int,
            timestamp : c_int
            ),
        mouse_move : extern fn(
            data : *const c_void,
            modifier : *const c_char,
            button : c_int,
            curx : c_int, 
            cury : c_int,
            prevx : c_int, 
            prevy : c_int,
            timestamp : c_int
            ),
        mouse_wheel : extern fn(
            data : *const c_void,
            modifier : *const c_char,
            direction : c_int,
            z : c_int, 
            x : c_int, 
            y : c_int,
            timestamp : c_int
            ),
        key_down : extern fn(
            data : *const c_void,
            modifier : *const c_char,
            keyname : *mut c_char,
            key : *const c_char,
            timestamp : c_int
            ),
        );

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
        property : *const JkProperty,
        changed : extern fn(object : *const c_void, data : *const c_void),
        get : extern fn(data : *const c_void) -> *const c_char
        );

    fn property_data_set(
        property : *const JkProperty,
        data : *const c_void
        );
}

pub struct Master
{
    //windows : DList<Window>
    pub window : Option<*const Window>,
    pub tree : Option<*const Tree>,
    pub property : Option<*const JkProperty>,
    pub scene : Option<Arc<RWLock<scene::Scene>>>,
    pub render : render::Render

}

impl Master
{
    pub fn new() -> Master
    {
        let mut m = Master {
            window : None,
            tree : None,
            property : None,
            scene : None,
            render : render::Render::new()
        };

        m.scene = Some(m.render.scene.clone());

        m
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

    match s.as_str() {
        Some(ss) => {
            let sss = property::SString(String::from_str(ss));
            o.clone().write().set_property("name", &sss);
        },
        _ => ()
    }

}


pub extern fn init_cb(master: *mut Master) -> () {
    unsafe {
        let w = window_new();
        window_callback_set(
            w,
            mem::transmute(master), //ptr::null(),//TODO
            mouse_down,
            mouse_up,
            mouse_move,
            mouse_wheel,
            key_down
            );

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
                    tree_object_add(t, mem::transmute(box o.clone()), ptr::null());
                }

                let oo = s.read().object_find("yepyoyo");
                match oo {
                    Some(o) => { 
                        property_data_set(p, mem::transmute(box o.clone()));
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

pub extern fn mouse_down(
    data : *const c_void,
    modifier : *const c_char,
    button : c_int,
    x : c_int, 
    y : c_int,
    timestamp : c_int
    )
{
    //println!("rust mouse down button {}, pos: {}, {}", button, x, y);
}

pub extern fn mouse_up(
    data : *const c_void,
    modifier : *const c_char,
    button : c_int,
    x : c_int, 
    y : c_int,
    timestamp : c_int
    )
{
    let m : &Master = unsafe {mem::transmute(data)};
    //println!("rust mouse up button {}, pos: {}, {}", button, x, y);
    let r = m.render.camera.borrow().ray_from_screen(x as f64, y as f64, 1000f64);
    println!("ray : {} ", r);
}

pub extern fn mouse_move(
    data : *const c_void,
    modifier : *const c_char,
    button : c_int,
    curx : c_int, 
    cury : c_int,
    prevx : c_int, 
    prevy : c_int,
    timestamp : c_int
    )
{
    //println!("rust mouse move");
}

pub extern fn mouse_wheel(
    data : *const c_void,
    modifier : *const c_char,
    direction : c_int,
    z : c_int, 
    x : c_int, 
    y : c_int,
    timestamp : c_int
    )
{
    println!("move wheel");
}

pub extern fn key_down(
    data : *const c_void,
    modifier : *const c_char,
    keyname : *mut c_char,
    key : *const c_char,
    timestamp : c_int
    )
{
    println!("rust key_down");
}
