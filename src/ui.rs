use libc::{c_char, c_void};
use render;
use object;

use std::mem;
use sync::{RWLock, Arc};
use std::c_str::CString;
use std::ptr;

#[repr(C)]
pub struct Tree;
#[repr(C)]
pub struct Property;
#[repr(C)]
pub struct Creator;

#[link(name = "joker")]
extern {
    //fn simple_window_init();
    pub fn elm_simple_window_main();
    fn tree_widget_new() -> *const Tree;
    fn tree_register_cb(
        tree : *mut Tree,
        name_get : extern fn(data : *const c_void) -> *const c_char,
        select : extern fn(data : *const c_void) -> (),
        can_expand : extern fn(data : *const c_void) -> bool,
        expand : extern fn(tree: *mut Tree, data : *const c_void, parent: *const c_void) -> (),
        );

    fn tree_object_add(
        tree : *mut Tree,
        object : *const c_void,
        parent : *const c_void,
        );
    fn creator_new() -> *const Creator;
    fn creator_tree_new(creator : *const Creator) -> *mut Tree;
    fn creator_button_new(creator : *const Creator);
    fn creator_property_new(creator : *const Creator) -> *mut Property;

    pub fn init_callback_set(
        cb: extern fn(*mut render::Render) -> (),
        render: *const render::Render
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
        Property : *mut Property,
        changed : extern fn(object : *const c_void, data : *const c_void)
        );
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

pub extern fn expand(tree: *mut Tree, data : *const c_void, parent : *const c_void) -> () {
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

pub extern fn changed(object : *const c_void, data : *const c_void) {
    let o : &Arc<RWLock<object::Object>> = unsafe {
        mem::transmute(object)
    };

    let s = unsafe {CString::new(data as *const i8, false) };
    println!("data changed : {}", s);
}


pub extern fn init_cb(render: *mut render::Render) -> () {
    unsafe {
        let c = creator_new();
        let t = creator_tree_new(c);
        tree_register_cb(
            t,
            name_get,
            select,
            can_expand,
            expand);

        for o in (*render).scene.objects.iter() {
            tree_object_add(t, mem::transmute(o), ptr::null());
        }

        let p = creator_property_new(c);
        property_register_cb(
            p,
            changed);

        //creator_button_new(c);
    }
}

