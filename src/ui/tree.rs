use std::sync::{RwLock, Arc};
use std::collections::HashMap;
use libc::{c_char, c_void, c_int};
use std::mem;
use std::collections::{LinkedList};//,Deque};
use std::ptr;
use std::cell::{RefCell, BorrowState};
use std::rc::Weak;
use std::rc::Rc;
use uuid::Uuid;
use std::ffi::CString;

use dormin::scene;
use dormin::object;
use ui::Window;
use ui::Master;
use ui::{RefMut,PropertyUser};
use ui;

#[repr(C)]
pub struct Elm_Object_Item;
#[repr(C)]
pub struct JkTree;

#[link(name = "joker")]
extern {
    fn window_tree_new(
        window : *const Window,
        x : c_int,
        y : c_int,
        w : c_int,
        h : c_int
        ) -> *const JkTree;
    pub fn tree_register_cb(
        tree : *const JkTree,
        data : *const c_void,
        name_get : extern fn(data : *const c_void) -> *const c_char,
        selected : extern fn(data : *const c_void) -> (),
        can_expand : extern fn(data : *const c_void) -> bool,
        expand : extern fn(tree: *const c_void, data : *const c_void, parent: *const Elm_Object_Item) -> (),
        //sel : extern fn(tree: *const TreeSelectData, data : *const c_void, parent: *const Elm_Object_Item) -> (),
        //unsel : extern fn(tree: *const TreeSelectData, data : *const c_void, parent: *const Elm_Object_Item) -> (),
        sel : extern fn(tree: *const ui::WidgetCbData, data : *const c_void, parent: *const Elm_Object_Item) -> (),
        unsel : extern fn(tree: *const ui::WidgetCbData, data : *const c_void, parent: *const Elm_Object_Item) -> (),
        panel_move : ui::PanelGeomFunc
        );

    fn tree_object_add(
        tree : *const JkTree,
        object : *const c_void,
        parent : *const Elm_Object_Item,
        ) -> *const Elm_Object_Item;

    fn tree_object_remove(
        item : *const Elm_Object_Item);

    fn tree_item_select(item : *const Elm_Object_Item);
    fn tree_item_update(item : *const Elm_Object_Item);
    fn tree_item_expand(item : *const Elm_Object_Item);
    fn tree_deselect_all(item : *const JkTree);
    fn tree_update(tree : *const JkTree);
    fn tree_show(obj : *const JkTree, b : bool);
    fn tree_clear(obj : *const JkTree);
}

pub struct Tree
{
    pub name : String,
    //TODO change the key
    //objects : HashMap<Arc<RwLock<object::Object>>, *const Elm_Object_Item >
    //objects : HashMap<String, *const Elm_Object_Item>,
    objects : HashMap<Uuid, *const Elm_Object_Item>,
    pub jk_tree : *const JkTree,
    pub id : Uuid,
    pub config : ui::WidgetConfig
}

impl Tree
{
    pub fn new(
        window : *const Window,
        config : &ui::WidgetConfig
        ) -> Tree // Box<Tree>
    {
        //let mut t = box Tree {
        let mut t = Tree {
            name : String::from("tree_name"),
            objects : HashMap::new(),
            jk_tree : unsafe {window_tree_new(
                    window, config.x, config.y, config.w, config.h)},
            id : Uuid::new_v4(),
            config : config.clone()
        };

        t.set_visible(config.visible);

        t
    }

    pub fn set_scene(&mut self, scene : &scene::Scene)
    {
        unsafe {tree_clear(self.jk_tree);}
        self.objects.clear();
        for o in &scene.objects {
            self.add_object(o.clone());
        }
    }

    pub fn add_object(&mut self, object : Arc<RwLock<object::Object>>)
    {
        if self.objects.contains_key(&object.read().unwrap().id) {
            return;
        }

        self._add_object(object);
    }

    pub fn add_objects(&mut self, objects : &[Arc<RwLock<object::Object>>])
    {
        for o in objects {
            self.add_object(o.clone());
        }
    }

    fn _add_object(&mut self, object : Arc<RwLock<object::Object>>)
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

    /*
    pub fn add_object_by_id(&mut self, id : Uuid)
    {
        if self.objects.contains_key(&id) {
            return;
        }

        let o = match self.scene {
            Some(s) => {
                match  s.read().unwrap().find_object_by_id(id) {
                    Some(o) => o,
                    None => return
                }
            },
            _ => return
        };

        self._add_object(o);
    }

    pub fn add_objects_by_id(&mut self, ids : LinkedList<Uuid>)
    {
        for id in ids.iter()
        {
            self.add_object_by_id(id);
        }
    }
    */

    pub fn remove_objects_by_id(&mut self, ids : Vec<Uuid>)
    {
        for id in &ids {
            let item = self.objects.remove(id);
            if let Some(i) = item {
                unsafe { tree_object_remove(i);}
            }
        }
    }

    pub fn select(&mut self, id: &Uuid)
    {
        unsafe { tree_deselect_all(self.jk_tree); }
        self._select(id);
    }

    pub fn select_objects(&mut self, ids: Vec<Uuid>)
    {
        unsafe { tree_deselect_all(self.jk_tree); }
        for id in &ids {
            self._select(id);
        }
    }

    fn _select(&mut self, id: &Uuid)
    {
        if let Some(item) = self.objects.get(id) {
            unsafe {tree_item_select(*item);}
        }
    }


    pub fn set_selected(&mut self, ids: LinkedList<Uuid>)
    {
        unsafe { tree_deselect_all(self.jk_tree); }

        for id in &ids {
            if let Some(item) = self.objects.get(id) {
                unsafe {tree_item_select(*item);}
            }
        }
    }

    pub fn update(&self)
    {
        unsafe { tree_update(self.jk_tree); }
    }

    pub fn update_object(& self, id: &Uuid)
    {
        if let Some(item) = self.objects.get(id) {
            unsafe {tree_item_update(*item);}
        }
    }

    pub fn set_visible(&mut self, b : bool)
    {
        self.config.visible = b;
        unsafe {
            tree_show(self.jk_tree, b);
        }
    }

    pub fn visible(&self) -> bool
    {
        self.config.visible
    }

    pub fn get_config(&self) -> ui::WidgetConfig
    {
        self.config.clone()
    }
}

pub extern fn name_get(data : *const c_void) -> *const c_char
{
    let o : &Arc<RwLock<object::Object>> = unsafe {
        mem::transmute(data)
    };

    //println!("name get {:?}", o);
    let cs = CString::new(o.read().unwrap().name.as_bytes()).unwrap();
    cs.as_ptr()
}

pub extern fn item_selected(data : *const c_void) -> ()
{
    let o : &Arc<RwLock<object::Object>> = unsafe {
        mem::transmute(data)
    };
    println!("item_selected callback ! {}, but this function does nothing for now ", o.read().unwrap().name);
}

pub extern fn can_expand(data : *const c_void) -> bool
{
    let o : &Arc<RwLock<object::Object>> = unsafe {
        mem::transmute(data)
    };

    println!("can expand :{}", o.read().unwrap().children.is_empty());
    return !o.read().unwrap().children.is_empty();
}

pub extern fn expand(
    widget_cb_data: *const c_void,
    data : *const c_void,
    parent : *const Elm_Object_Item) -> ()
{
    let o : &Arc<RwLock<object::Object>> = unsafe {
        mem::transmute(data)
    };

    //let tsd : &TreeSelectData = unsafe {mem::transmute(tsd)};
    //let t : &mut Tree = &mut **tsd.tree.borrow_mut();

    let wcb : & ui::WidgetCbData = unsafe {mem::transmute(widget_cb_data)};
    let t : &mut Tree = unsafe {mem::transmute(wcb.widget)};
    //let container : &mut Box<ui::WidgetContainer> = unsafe {mem::transmute(wcb.container)};


    println!("expanding ! {} ", o.read().unwrap().name);
    println!("expanding ! tree name {} ", t.name);

    for c in &o.read().unwrap().children {
        println!("expanding ! with child {} ", (*c).read().unwrap().name);
        unsafe {
            let eoi = tree_object_add(t.jk_tree, mem::transmute(c), parent);
            t.objects.insert(c.read().unwrap().id.clone(), eoi);
        }
    }
}

pub extern fn selected(
    //tsd: *const TreeSelectData,
    tsd: *const ui::WidgetCbData,
    data : *const c_void,
    parent : *const Elm_Object_Item) -> ()
{
    let wcb : & ui::WidgetCbData = unsafe {mem::transmute(tsd)};
    let tree : &Tree = unsafe {mem::transmute(wcb.widget)};
    let container : &mut Box<ui::WidgetContainer> = unsafe {mem::transmute(wcb.container)};

    let o : &Arc<RwLock<object::Object>> = unsafe {
        mem::transmute(data)
    };

    println!("selected callback, TODO do the following in widget container 'handle' ");
    container.handle_event(ui::Event::SelectObject(o.clone()), tree.id);
}

pub extern fn unselected(
    tsd: *const ui::WidgetCbData,
    data : *const c_void,
    parent : *const Elm_Object_Item) -> ()
{
    let wcb : & ui::WidgetCbData = unsafe {mem::transmute(tsd)};
    let tree : &Tree = unsafe {mem::transmute(wcb.widget)};
    let container : &mut Box<ui::WidgetContainer> = unsafe {mem::transmute(wcb.container)};
    //let tsd : &TreeSelectData = unsafe {mem::transmute(tsd)};

    /*
    if tsd.tree.borrow_state() == BorrowState::Writing {
        return;
    }
    */

    let o : &Arc<RwLock<object::Object>> = unsafe {
        mem::transmute(data)
    };

    println!("TODO,unselect do the following in widget container 'handle'");
    container.handle_event(ui::Event::UnselectObject(o.clone()), tree.id);

    /*
    let o = match tsd.control.borrow_state() {
        BorrowState::Unused => {
            {
                let mut l = LinkedList::new();
                l.push_back(o.read().unwrap().id.clone());
                tsd.control.borrow_mut().unselect(&l);
            }
            tsd.control.borrow().get_selected_object()
        },
        _ => {
            println!("already borrowed : mouse_up add_ob ->sel ->add_ob");
            return;
        }
    };

    match tsd.property.borrow_state() {
        BorrowState::Unused => {
            match o {
                Some(ref o) =>
                    tsd.property.borrow_mut().set_object(&*o.read().unwrap()),
                None =>
                    tsd.property.borrow_mut().set_nothing()
            }
        },
        _ => { println!("property already borrowed : tree unsel ->add_ob"); return;}
    };
    */
}

impl ui::Widget for Tree
{
    fn get_id(&self) -> Uuid
    {
        self.id
    }

    fn handle_change_prop(&self, prop_user : &PropertyUser, name : &str)
    {
        if name == "name" {
            self.update_object(&prop_user.get_id());
        }
    }
}

pub extern fn panel_move(
    widget_cb_data : *const c_void,
    x : c_int, y : c_int, w : c_int, h : c_int)
{
    println!("panel geom !!!!!!!!! {}, {}, {}, {}", x, y, w, h);
    let wcb : & ui::WidgetCbData = unsafe {mem::transmute(widget_cb_data)};
    let mut t : &mut Tree = unsafe {mem::transmute(wcb.widget)};

    t.config.x = x;
    t.config.y = y;
    t.config.w = w;
    t.config.h = h;
}

