use std::sync::{RwLock, Arc};
use std::collections::HashMap;
use libc::{c_char, c_void, c_int};
use std::mem;
//use std::collections::{DList,Deque};
use std::ptr;
use std::cell::RefCell;
use std::rc::Weak;
use std::rc::Rc;
use uuid::Uuid;
use std::ffi::CString;

use scene;
use object;
use ui::Window;
use ui::Master;
use ui;
use control::Control;

#[repr(C)]
pub struct Elm_Object_Item;
#[repr(C)]
pub struct JkTree;

#[link(name = "joker")]
extern {
    fn window_tree_new(window : *const Window) -> *const JkTree;
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

    fn tree_item_select(item : *const Elm_Object_Item);
    fn tree_item_update(item : *const Elm_Object_Item);
    fn tree_item_expand(item : *const Elm_Object_Item);
    fn tree_deselect_all(item : *const JkTree);
    fn tree_update(tree : *const JkTree);
}

pub struct Tree
{
    pub name : String,
    //TODO change the key
    //objects : HashMap<Arc<RwLock<object::Object>>, *const Elm_Object_Item >
    //objects : HashMap<String, *const Elm_Object_Item>,
    objects : HashMap<Uuid, *const Elm_Object_Item>,
    jk_tree : *const JkTree,
    control : Rc<RefCell<Control>>,
    dont_forward_signal : bool
}

impl Tree
{
    pub fn new(
        window : *const Window,
        control : Rc<RefCell<Control>>) -> Box<Tree>
    {
        let t = box Tree {
            name : String::from_str("tree_name"),
            objects : HashMap::new(),
            jk_tree : unsafe {window_tree_new(window)},
            control : control,
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

    pub fn add_object(&mut self, object : Arc<RwLock<object::Object>>)
    {
        let eoi = unsafe {
            match object.read().unwrap().parent {
                Some(ref p) =>  {
                    match self.objects.get(&p.read().unwrap().id) {
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
            self.objects.insert(object.read().unwrap().id.clone(), eoi);
        }
    }

    pub fn select(&mut self, id: &Uuid)
    {
        unsafe { tree_deselect_all(self.jk_tree); }

        println!("select from tree");
        self.dont_forward_signal = true;
        match self.objects.get(id) {
            Some(item) => {
                unsafe {tree_item_select(*item);}
            }
            _ => {}
        }

        self.dont_forward_signal = false;
        println!("select from tree end");
    }

    pub fn update(&self)
    {
        unsafe { tree_update(self.jk_tree); }
    }

    pub fn update_object(&mut self, id: &Uuid)
    {
        match self.objects.get(id) {
            Some(item) => {
                unsafe {tree_item_update(*item);}
            }
            _ => {}
        }
    }
}

extern fn name_get(data : *const c_void) -> *const c_char
{
    let o : &Arc<RwLock<object::Object>> = unsafe {
        mem::transmute(data)
    };

    //println!("name get {:?}", o);

    let cs = CString::from_slice(o.read().unwrap().name.as_bytes());

    unsafe {
        //TODO can use as_ptr?
        cs.as_ptr()
    }
}

extern fn item_selected(data : *const c_void) -> ()
{
    let o : &Arc<RwLock<object::Object>> = unsafe {
        mem::transmute(data)
    };
    println!("selected ! {} ", o.read().unwrap().name);
}

extern fn can_expand(data : *const c_void) -> bool
{
    let o : &Arc<RwLock<object::Object>> = unsafe {
        mem::transmute(data)
    };

    println!("can expand :{}", o.read().unwrap().children.is_empty());
    return !o.read().unwrap().children.is_empty();
}

extern fn expand(
    tree: *const Tree,
    data : *const c_void,
    parent : *const Elm_Object_Item) -> ()
{
    let o : &Arc<RwLock<object::Object>> = unsafe {
        mem::transmute(data)
    };

    let mut t : &mut Tree = unsafe {mem::transmute(tree)};

    println!("expanding ! {} ", o.read().unwrap().name);
    println!("expanding ! tree name {} ", t.name);

    for c in o.read().unwrap().children.iter() {
        println!("expanding ! with child {} ", (*c).read().unwrap().name);
        unsafe {
            let eoi = tree_object_add(t.jk_tree, mem::transmute(c), parent);
            t.objects.insert(c.read().unwrap().id.clone(), eoi);
        }
    }
}

extern fn selected(
    tree: *const Tree,
    data : *const c_void,
    parent : *const Elm_Object_Item) -> ()
{
    let o : &Arc<RwLock<object::Object>> = unsafe {
        mem::transmute(data)
    };

    let t : &Tree = unsafe {mem::transmute(tree)};

    println!("sel ! {} ", o.read().unwrap().name);
    println!("sel ! tree name {} ", t.name);

    if t.dont_forward_signal {
        return;
    }

    match t.control.try_borrow_mut() {
        Some(ref mut c) => {
            c.select(&o.read().unwrap().id);
        },
        _ => { println!("already borrowed : mouse_up add_ob ->sel ->add_ob")}
    }
}

