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

#[repr(C)]
pub struct Elm_Object_Item;
#[repr(C)]
pub struct JkTree;

#[link(name = "joker")]
extern {
    pub fn window_tree_new(window : *const Window) -> *const JkTree;
    fn tree_widget_new() -> *const JkTree;
    fn tree_register_cb(
        tree : *const JkTree,
        data : *const Tree,
        name_get : extern fn(data : *const c_void) -> *const c_char,
        selected : extern fn(data : *const c_void) -> (),
        can_expand : extern fn(data : *const c_void) -> bool,
        expand : extern fn(tree: *const Tree, data : *const c_void, parent: *const Elm_Object_Item) -> (),
        sel : extern fn(tree: *const Tree, data : *const c_void, parent: *const Elm_Object_Item) -> (),
        );

    fn tree_object_add(
        tree : *const JkTree,
        object : *const c_void,
        parent : *const Elm_Object_Item,
        ) -> *const Elm_Object_Item;

    pub fn tree_item_select(item : *const Elm_Object_Item);
    fn tree_item_expand(item : *const Elm_Object_Item);
    fn tree_deselect_all(item : *const JkTree);
}

pub struct Tree
{
    pub name : String,
    //TODO change the key
    //objects : HashMap<Arc<RWLock<object::Object>>, *const Elm_Object_Item >
    objects : HashMap<String, *const Elm_Object_Item>,
    jk_tree : *const JkTree,
    master : Weak<RefCell<ui::Master>>,
    dont_forward_signal : bool
}

impl Tree
{
    pub fn new(
        window : *const Window,
        master : Weak<RefCell<ui::Master>>) -> Box<Tree>
    {
        let t = box Tree {
            name : String::from_str("tree_name"),
            objects : HashMap::new(),
            jk_tree : unsafe {window_tree_new(window)},
            master : master,
            dont_forward_signal : false
        };

        unsafe {
            tree_register_cb(
                t.jk_tree,
                &*t,
                name_get,
                item_selected,
                can_expand,
                expand,
                selected);
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
                    match self.objects.find(&p.read().name) {
                        Some(item) => {
                            tree_object_add(
                                self.jk_tree,
                                mem::transmute(box object.clone()),
                                *item)
                        },
                        None => {
                            println!("problem with tree, could not find parent item");
                            ptr::null()
                        }
                    }

                },
                None => {
                    tree_object_add(
                        self.jk_tree,
                        mem::transmute(box object.clone()),
                        ptr::null())
                }
            }
        };

        if eoi != ptr::null() {
            self.objects.insert(object.read().name.clone(), eoi);
        }
    }

    pub fn select(&mut self, name: &String)
    {
        unsafe { tree_deselect_all(self.jk_tree); }

        println!("select from tree");
        self.dont_forward_signal = true;
        match self.objects.find(name) {
            Some(item) => {
                unsafe {tree_item_select(*item);}
            }
            _ => {}
        }

        self.dont_forward_signal = false;
        println!("select from tree end");
    }
}

extern fn name_get(data : *const c_void) -> *const c_char
{
    let o : &Arc<RWLock<object::Object>> = unsafe {
        mem::transmute(data)
    };

    //println!("name get {:?}", o);

    let cs = o.read().name.to_c_str();

    unsafe {
        cs.unwrap()
    }
}

extern fn item_selected(data : *const c_void) -> ()
{
    let o : &Arc<RWLock<object::Object>> = unsafe {
        mem::transmute(data)
    };
    println!("selected ! {} ", o.read().name);
}

extern fn can_expand(data : *const c_void) -> bool
{
    let o : &Arc<RWLock<object::Object>> = unsafe {
        mem::transmute(data)
    };

    println!("can expand :{}", o.read().children.is_empty());
    return !o.read().children.is_empty();
}

extern fn expand(
    tree: *const Tree,
    data : *const c_void,
    parent : *const Elm_Object_Item) -> ()
{
    let o : &Arc<RWLock<object::Object>> = unsafe {
        mem::transmute(data)
    };

    let mut t : &mut Tree = unsafe {mem::transmute(tree)};

    println!("expanding ! {} ", o.read().name);
    println!("expanding ! tree name {} ", t.name);

    for c in o.read().children.iter() {
        println!("expanding ! with child {} ", (*c).read().name);
        unsafe {
            let eoi = tree_object_add(t.jk_tree, mem::transmute(c), parent);
            t.objects.insert(c.read().name.clone(), eoi);
        }
    }
}

extern fn selected(
    tree: *const Tree,
    data : *const c_void,
    parent : *const Elm_Object_Item) -> ()
{
    let o : &Arc<RWLock<object::Object>> = unsafe {
        mem::transmute(data)
    };

    let mut t : &mut Tree = unsafe {mem::transmute(tree)};

    println!("sel ! {} ", o.read().name);
    println!("sel ! tree name {} ", t.name);

    if t.dont_forward_signal {
        return;
    }

    match t.master.upgrade() {
        Some(m) => { 
            match m.try_borrow_mut() {
                Some(ref mut mm) => {
                    mm.select(&o.read().name);
                },
                _ => { println!("already borrowed : mouse_up add_ob ->sel ->add_ob")}
            }
        },
        None => { println!("the master of the tree doesn't exist anymore");}
    }

}

