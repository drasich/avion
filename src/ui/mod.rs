//use object::Object;
//use scene::Scene;
use libc::{c_char, c_void, c_int, c_float};
use std::collections::{HashMap,HashSet};
use std::sync::{Arc,RwLock};
use std::cell::RefCell;
use std::rc::{Rc,Weak};
use uuid;

pub use self::def::{
    Master,
    WidgetContainer,
    WidgetCbData,
    AppCbData,
    Widget,
    Event,
    WidgetConfig,
    
    PanelGeomFunc,
    ButtonCallback,
    EntryCallback,

    init_cb,
    exit_cb,
    init_callback_set,
    exit_callback_set,
    elm_simple_window_main,
    Window,
    window_new,
    Evas_Object,
    JkGlview,
    jk_window_new,
    jk_window_request_update,
    jk_glview_new,
    window_callback_set,

    add_empty,
    scene_new,
    scene_list,
    scene_rename,

    ecore_animator_add,
    update_play_cb,

    create_gameview_window,

    evas_object_show,
    evas_object_hide,

    WidgetPanel,
    WidgetPanelConfig
};


pub use self::tree::{Tree};
pub use self::action::{Action,Position};
pub use self::command::{Command};
//pub use self::property::{Property,PropertyConfig,ChangedFunc,RefMut,PropertyUser};
pub use self::property_list::{PropertyList,JkPropertyList};
pub use self::property_box::{PropertyBox,JkPropertyBox};
//pub use self::property::{PropertyShow};
//pub use self::property::{JkPropertyList};
pub use self::property::{make_vec_from_str,find_property_show,JkPropertyCb};

pub use self::view::{View, GameView, gv_close_cb};

mod tree;
mod action;
mod command;
pub mod def;
pub mod property;
pub mod property_box;
pub mod property_list;
pub mod view;
//pub mod dragger;


pub type ChangedFunc = extern fn(
    object : *const c_void,
    property : *const c_void,
    data : *const c_void);

pub type RegisterChangeFunc = extern fn(
    app : *const c_void,
    property : *const c_void,
    old : *const c_void,
    new : *const c_void,
    action_type : c_int
    );

pub type PropertyTreeFunc = extern fn(
    property : *const c_void,
    object : *const c_void,
    parent : *const Elm_Object_Item);

#[repr(C)]
pub struct PropertyValue;

#[derive(RustcDecodable, RustcEncodable, Clone)]
pub struct PropertyConfig
{
    visible : bool,
    x : i32,
    y : i32,
    w : i32,
    h : i32,
    expand : HashSet<String>
}

impl PropertyConfig
{
    pub fn new() -> PropertyConfig
    {
        PropertyConfig {
            visible : true,
            x: 10,
            y: 10,
            w: 100,
            h : 400,
            expand : HashSet::new()
        }
    }
}

use dormin::property::{PropertyWrite, PropertyGet};

pub trait PropertyUser : PropertyWrite + PropertyGet + PropertyShow + PropertyId {
    fn as_show(&self) -> &PropertyShow;
    fn as_write(&self) -> &PropertyWrite;
    fn as_id(&self) -> &PropertyId;
    fn as_get(&self) -> &PropertyGet;
}

impl<T: PropertyWrite + PropertyGet + PropertyShow + PropertyId > PropertyUser for T {

    fn as_show(&self) -> &PropertyShow
    {
        self
    }

    fn as_write(&self) -> &PropertyWrite
    {
        self
    }

    fn as_get(&self) -> &PropertyGet
    {
        self
    }

    fn as_id(&self) -> &PropertyId
    {
        self
    }
}

#[derive( Clone)]
pub enum PropertyChange
{
    Value,
	VecAdd(usize),
	VecDel(usize),
}

pub trait PropertyShow
{
    fn create_widget_itself(&self, field : &str) -> Option<*const PropertyValue>
    {
        println!("           not implemented, field is {}", field);
        None
    }

    fn create_widget_inside(&self, path : &str, widget : &PropertyWidget)//, parent : *const PropertyValue)
    {
        // do nothing by default
        // implement if vec, map etc...
    }

    fn update_widget(&self, pv : *const PropertyValue) {
        //println!("update_widget not implemented for this type");
    }

    fn get_property(&self, field : &str) -> Option<&PropertyShow>
    {
        None
    }

    fn callclosure(&self, f : &Fn(&PropertyShow)) where Self : Sized
    {
        f(self);
    }

    fn update_property(&self, widget : &PropertyWidget, all_path : &str, local_path : Vec<String>)
    {
        println!("default update property : {}, {:?}", all_path, local_path);
        if local_path.is_empty() {
            println!("path is empty");
            if let Some(pv) = widget.get_property(all_path) {
                println!("found property");
                self.update_widget(pv);
            }
        }
    }

    fn update_property_new(&self, widget : &PropertyWidget, all_path : &str, local_path : Vec<String>, change : PropertyChange)
    {
        if let PropertyChange::Value = change {
            self.update_property(widget, all_path, local_path);
        }
    }

    /*
    fn find_and_update_property(&self, field : &str, pv : *const PropertyValue)
    {
        let path = make_vec_from_str(field);

        match path.len() {
            0 =>  self.update_widget(pv),
            _ => {
                macro with field
                match p.get_property(path[0].as_ref()) {
                    Some(ppp) => {
                        find_and_update_property_show(ppp, path[1..].to_vec())
                    },
                    None => {
                        None
                    }
                }
            }
        }
    }
    */

    fn is_node(&self) -> bool
    {
        false
    }

    fn to_update(&self) -> ShouldUpdate
    {
        ShouldUpdate::Nothing
    }
}

pub trait PropertyId
{
    fn get_id(&self) -> uuid::Uuid;
}

pub enum RefMut<T:?Sized> {
    Arc(Arc<RwLock<T>>),
    Cell(Rc<RefCell<T>>),
}

impl<T:?Sized> Clone for RefMut<T>
{
    fn clone(&self) -> RefMut<T>
    {
        match *self {
            RefMut::Arc(ref a) => RefMut::Arc(a.clone()),
            RefMut::Cell(ref c) => RefMut::Cell(c.clone()),
        }
    }
}

#[repr(C)]
pub struct Elm_Object_Item;

#[derive(Debug)]
pub enum ShouldUpdate
{
    Nothing,
    Mesh
}

pub trait PropertyWidget : Widget {

    fn add_simple_item(&self, field : &str, item : *const PropertyValue);
    fn add_option(&self, field : &str, is_some : bool) -> *const PropertyValue;
    fn add_vec(&self, field : &str, len : usize);
    fn add_vec_item(&self, field : &str, widget_entry : *const PropertyValue, index : usize);
    fn del_vec_item(&self, field : &str, index : usize);

    //fn update_option(&mut self, widget_entry : *const PropertyValue, is_some : bool);

    fn update_vec(&self, widget_entry : *const PropertyValue, len : usize);
    fn update_enum(&self, path : &str, widget_entry : *const PropertyValue, value : &str);

    fn get_current(&self) -> Option<RefMut<PropertyUser>>;
    fn set_current(&self, p : RefMut<PropertyUser>, title : &str);

    fn get_property(&self, path : &str) -> Option<*const PropertyValue> 
    {
        println!("PropertyWidget, 'get_property' not implemented no return None");
        None
    }
}

pub enum NodeChildren
{
    None,
    Struct(HashMap<String,Rc<RefCell<PropertyNode>>>),
    Vec(Vec<Rc<RefCell<PropertyNode>>>),
}

pub struct PropertyNode
{
    value : *const PropertyValue,
    children : NodeChildren,
    parent : Option<Weak<RefCell<PropertyNode>>>,
    name : String
}

impl PropertyNode
{
    fn new(
        name : &str, 
        value : *const PropertyValue,
        //parent : Option<Weak<RefCell<PropertyNode>>>
        )
        -> PropertyNode
    {
        PropertyNode {
            value : value,
            children : NodeChildren::None,
            parent : None,
            name : String::from(name)
        }
    }

    fn get_node(&self, path : &str) -> Option<Weak<RefCell<PropertyNode>>>
    {
        self.children.get_node(path)
    }

    fn del_child(&mut self, field : &str)
    {
        self.children.del_node(field)
    }

    fn get_path(&self) -> String
    {
        let mut path = String::new();
        if let Some(ref p) = self.parent {
            if let Some(p) = p.upgrade() {
                path.push_str(&p.borrow().get_path());
            }

        }

        if !path.is_empty() {
            path.push('/');
        }

        path.push_str(&self.name);

        return path;
    }

    fn print_stuff(&self)
    {
        println!(">>>my name is ::: {} \n\n", self.name);
        self.children.print_stuff();
    }

    fn get_child_count(&self) -> usize {
        self.children.get_count()
    }
}

pub fn node_add_child(field : &str, parent : Rc<RefCell<PropertyNode>>, child : Rc<RefCell<PropertyNode>>)
{
    child.borrow_mut().parent = Some(Rc::downgrade(&parent));
    parent.borrow_mut().children.add_node(field, child);
}


impl NodeChildren {
    pub fn update(&self, ps : &PropertyShow, but : &str)
    {
        match *self {
            NodeChildren::None => {},
            NodeChildren::Struct(ref m) => {
                for (f,pn) in m.iter() {
                    let fstr : &str = f.as_ref();
                    if fstr == but {
                        println!("buuuuuuuuuuuuuuuuuuuuuuuuuut: {} ", fstr);
                        continue;
                    }
                    let but = if but.starts_with(fstr) {
                        let (left, right) = but.split_at(fstr.len()+1);
                        right
                    }
                    else {
                        ""
                    };
                    match ps.get_property(f) {
                        Some(ppp) => {
                            ppp.update_widget(pn.borrow().value);
                            pn.borrow().children.update(ppp, but);
                        },
                        None => {
                            println!("could not find prop : {:?}", f);
                        }
                    }
                }


            },
            _ => {}
        }
    }

    fn get_node(&self, path : &str) -> Option<Weak<RefCell<PropertyNode>>>
    {
        let mut v : Vec<&str> = path.splitn(2,"/").collect();

        if v.is_empty() {
            return None;
        }

        match *self {
            NodeChildren::Vec(ref vec) => {
                if let Ok(index) = v[0].parse::<usize>() {
                    let n = &vec[index];
                    if v.len() == 1 {
                        Some(Rc::downgrade(n))
                    }
                    else {
                        n.borrow().get_node(v[1])
                    }
                }
                else {
                    panic!("not an index : {}", v[0]);
                    None
                }
            },
            NodeChildren::Struct(ref m) => {
                match m.get(v[0]) { //.map(|o| *o) {
                    Some(node) => {
                        if v.len() == 1 {
                            Some(Rc::downgrade(node))
                        }
                        else {
                            node.borrow().get_node(v[1])
                        }
                    },
                    None => None
                }
            },
            _ => None
        }
    }

    fn add_node(&mut self, field : &str, node : Rc<RefCell<PropertyNode>>)
    {
        match *self {
            NodeChildren::Vec(ref mut vec) => {
                if let Ok(index) = field.parse::<usize>() {
                    vec.insert(index, node);
                    for i in (index+1)..vec.len() {
                        vec[i].borrow_mut().name = i.to_string();
                    }
                }
                else {
                    panic!("cannot add to vec");
                }
            },
            NodeChildren::Struct(ref mut map) => {
                map.insert(field.to_owned(), node);
            },
            NodeChildren::None => {
                //TODO temporary
                // children type should be set when creating the node
                if let Ok(index) = field.parse::<usize>() {
                    let mut vec = Vec::new();
                    vec.push(node);
                    *self = NodeChildren::Vec(vec)
                }
                else {
                    let mut map = HashMap::new();
                    map.insert(field.to_owned(), node);
                    *self = NodeChildren::Struct(map)
                }
            }
        }
    }

    fn del_node(&mut self, field : &str)
    {
        match *self {
            NodeChildren::Vec(ref mut vec) => {
                if let Ok(index) = field.parse::<usize>() {
                    vec.remove(index);
                    for i in index..vec.len() {
                        vec[i].borrow_mut().name = i.to_string();
                    }
                }
                else {
                    panic!("cannot remove from vec");
                }
            },
            NodeChildren::Struct(ref mut map) => {
                map.remove(field);
            },
            NodeChildren::None => {
                    panic!("there was nothing");
            }
        }
    }

    fn print_stuff(&self)
    {
        match *self {
            NodeChildren::Vec(ref vec) => {
                    println!("it's a vec : {}", vec.len());
                for i in 0..vec.len() {
                    let name = &vec[i].borrow().name;
                    println!("{}, {} : {}, {}", i, name, Rc::strong_count(&vec[i]), Rc::weak_count(&vec[i]));
                }
            },
            NodeChildren::Struct(ref map) => {
                    println!("it's a struct");
            },
            NodeChildren::None => {
                    println!("there was nothing");
            }
        }
    }

    fn get_count(&self) -> usize
    {
        match *self {
            NodeChildren::Vec(ref vec) => {
                vec.len()
            },
            NodeChildren::Struct(ref map) => {
                map.len()
            },
            NodeChildren::None => {
                0
            }
        }

    }
}


