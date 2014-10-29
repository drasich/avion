use sync::{RWLock, Arc};
use std::collections::HashMap;
use libc::{c_char, c_void, c_int};
use std::mem;
use std::c_str::CString;
//use std::collections::{DList,Deque};
use std::ptr;

use scene;
use object;
use ui::Window;

#[repr(C)]
pub struct Elm_Object_Item;
#[repr(C)]
pub struct JkTree;

#[link(name = "joker")]
extern {
    pub fn window_tree_new(window : *const Window) -> *const JkTree;
    fn tree_widget_new() -> *const JkTree;
    pub fn tree_register_cb(
        tree : *const JkTree,
        name_get : extern fn(data : *const c_void) -> *const c_char,
        select : extern fn(data : *const c_void) -> (),
        can_expand : extern fn(data : *const c_void) -> bool,
        expand : extern fn(tree: *const JkTree, data : *const c_void, parent: *const Elm_Object_Item) -> (),
        );

    pub fn tree_object_add(
        tree : *const JkTree,
        object : *const c_void,
        parent : *const Elm_Object_Item,
        ) -> *const Elm_Object_Item;
}

pub struct Tree
{
    pub name : String,
    //TODO change the key
    //objects : HashMap<Arc<RWLock<object::Object>>, *const Elm_Object_Item >
    objects : HashMap<String, *const Elm_Object_Item>,
    jk_tree : *const JkTree

}

impl Tree
{
    pub fn new(window : *const Window) -> Tree
    {
        let t = Tree {
            name : String::from_str("caca"),
            objects : HashMap::new(),
            jk_tree : unsafe {window_tree_new(window)}
        };

        unsafe {
            tree_register_cb(
                t.jk_tree,
                name_get,
                select,
                can_expand,
                expand);
        }

        t
    }

    pub fn set_scene(&mut self, scene : &scene::Scene)
    {
        for o in scene.objects.iter() {
            self.add_object(o.clone());
        }
    }

    pub fn add_object(&mut self, object : Arc<RWLock<object::Object>>)
    {
        let eoi = unsafe {
            match object.read().parent {
                Some(ref p) =>  {
                    tree_object_add(
                        self.jk_tree,
                        mem::transmute(box object.clone()),
                        mem::transmute(box p.clone()))
                },
                None => {
                    tree_object_add(
                        self.jk_tree,
                        mem::transmute(box object.clone()),
                        ptr::null())
                }
            }
        };

        self.objects.insert(object.read().name.clone(), eoi);
    }
}

extern fn name_get(data : *const c_void) -> *const c_char {

    let o : &Arc<RWLock<object::Object>> = unsafe {
        mem::transmute(data)
    };

    //println!("name get {:?}", o);

    let cs = o.read().name.to_c_str();

    unsafe {
        cs.unwrap()
    }
}

extern fn select(data : *const c_void) -> () {
    let o : &Arc<RWLock<object::Object>> = unsafe {
        mem::transmute(data)
    };
    println!("select ! {} ", o.read().name);
}

extern fn can_expand(data : *const c_void) -> bool {
    let o : &Arc<RWLock<object::Object>> = unsafe {
        mem::transmute(data)
    };

    println!("can expand :{}", o.read().children.is_empty());
    return !o.read().children.is_empty();
}

extern fn expand(tree: *const JkTree, data : *const c_void, parent : *const Elm_Object_Item) -> () {
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

