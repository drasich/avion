//use object::Object;
//use scene::Scene;
use libc::{c_char, c_void, c_int, c_float};
use std::collections::{HashMap,HashSet};
use std::sync::{Arc,RwLock};
use std::cell::RefCell;
use std::rc::Rc;
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
    name : *const c_char,
    data : *const c_void);

pub type RegisterChangeFunc = extern fn(
    object : *const c_void,
    name : *const c_char,
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


pub trait PropertyShow
{
    fn create_widget(
        &self,
        property : &PropertyWidget,
        field : &str,
        depth : i32,
        has_container : bool ) -> Option<*const PropertyValue>;

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

    fn update_property(&self, path : Vec<String>, pv :*const PropertyValue)
    {
        println!("default update property : {:?}", path);
        if path.is_empty() {
            self.update_widget(pv);
        }
    }

    fn find_and_create(&self, property : &PropertyWidget, path : Vec<String>, start : usize)
    {
        println!("default create property : {:?}", path);
        if path.is_empty() {
            self.create_widget(property, "" , 1, false);
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

pub trait PropertyWidget {

    fn add_simple_item(&self, field : &str, item : *const PropertyValue);
    fn add_node_t(&self, field : &str, item : *const PropertyValue);
    fn add_option(&self, field : &str, is_some : bool) -> *const PropertyValue;
    fn add_vec(&self, field : &str, len : usize);
    fn add_vec_item(&self, field : &str, widget_entry : *const PropertyValue, is_node : bool);

    //fn update_option(&mut self, widget_entry : *const PropertyValue, is_some : bool);

    //fn update_vec(&mut self, widget_entry : *const PropertyValue, len : usize);
    fn update_enum(&mut self, widget_entry : *const PropertyValue, value : &str);

    fn get_current(&self) -> Option<RefMut<PropertyUser>>;
    fn set_current(&self, p : RefMut<PropertyUser>, title : &str);
}

