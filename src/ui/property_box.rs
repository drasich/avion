use std::sync::{RwLock, Arc};
use std::collections::{HashMap,HashSet};
use libc::{c_char, c_void, c_int, c_float};
use std::str;
use std::mem;
//use std::collections::{LinkedList,Deque};
use std::collections::{LinkedList};
use std::ptr;
use std::rc::Rc;
use std::cell::{Cell, RefCell, BorrowState};
use std::rc::Weak;
use std::any::{Any};//, AnyRefExt};
use std::ffi::CString;
use std::ffi;
use std::ffi::CStr;
use core::marker;
use uuid;
use uuid::Uuid;

use dormin::scene;
use dormin::camera;
use dormin::object;
use ui::{Window, ButtonCallback, ChangedFunc, RegisterChangeFunc, 
    PropertyTreeFunc, PropertyConfig, PropertyValue, RefMut, PropertyUser, PropertyShow, PropertyWidget,PropertyChange, NodeChildren, PropertyNode};
use ui;
use dormin::property;
use operation;
use control::WidgetUpdate;
use dormin::vec;
use dormin::transform;
use dormin::resource;
use dormin::mesh;
use dormin::material;
use dormin::property::PropertyGet;
use dormin::component;
use dormin::component::CompData;
use dormin::armature;
use dormin::transform::Orientation;
use util;

#[repr(C)]
pub struct JkPropertyBox;

#[link(name = "joker")]
extern {
    fn jk_property_box_new(eo : *const ui::Evas_Object) -> *const JkPropertyBox;

    fn property_box_clear(pl : *const JkPropertyBox);

    pub fn property_box_cb_get(pl : *const JkPropertyBox) -> *const ui::JkPropertyCb;

    fn property_box_single_item_add(
        ps : *const JkPropertyBox,
        //cb_data : *const Weak<RefCell<PropertyNode>>,
        cb_data : *const c_void,
        pv: *const PropertyValue,
        parent: *const PropertyValue,
        ) -> *const PropertyValue;

    fn property_box_vec_item_add(
        ps : *const JkPropertyBox,
        cb_data : *const c_void,
        pv: *const PropertyValue,
        parent: *const PropertyValue,
        index : c_int,
        ) -> *const PropertyValue;

    fn property_box_vec_item_del(
        ps : *const JkPropertyBox,
        parent: *const PropertyValue,
        index : c_int);

    fn property_box_single_node_add(
        pl : *const JkPropertyBox,
        val : *const PropertyValue,
        ) -> *const PropertyValue;

    fn property_box_enum_update(
        pb : *const JkPropertyBox,
        pv : *const PropertyValue,
        value : *const c_char);

    fn property_box_vec_update(
        pb : *const JkPropertyBox,
        pv : *const PropertyValue,
        len : c_int);

    fn property_box_remove(
        pb : *const JkPropertyBox,
        value : *const PropertyValue);


    /*
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

    fn jk_property_set_data_set(set : *const JkPropertyBox, data : *const c_void);

    fn jk_property_set_register_cb(
        property : *const JkPropertyBox,
        data : *const Property,
        changed_float : ChangedFunc,
        changed_string : ChangedFunc,
        );

    fn property_set_string_add(
        ps : *const JkPropertyBox,
        name : *const c_char,
        value : *const c_char
        );

    fn property_set_float_add(
        ps : *const JkPropertyBox,
        name : *const c_char,
        value : c_float
        );

    fn property_set_node_add(
        ps : *const JkPropertyBox,
        name : *const c_char
        );

    fn property_set_clear(
        ps : *const JkPropertyBox);
        */
}

pub struct PropertyBox
{
    pub name : String,
    pub jk_property : *const JkPropertyBox,
    visible : Cell<bool>,
    pub id : uuid::Uuid,
    //pub config : PropertyConfig,
    current : RefCell<Option<RefMut<PropertyUser>>>,
    nodes : RefCell<NodeChildren>
}


impl PropertyBox
{
    pub fn new(
        panel : &ui::WidgetPanel,
        //pc : &PropertyConfig
        ) -> PropertyBox
    {
        PropertyBox {
            name : String::from("property_box_name"),
            jk_property : unsafe {jk_property_box_new(
                    panel.eo,
                    //pc.x, pc.y, pc.w, pc.h
                    )},
            visible: Cell::new(true),
            id : uuid::Uuid::new_v4(),
            //config : pc.clone(),
            current : RefCell::new(None),
            nodes : RefCell::new(NodeChildren::None)
        }
    }

    pub fn set_prop_cell(&self, p : Rc<RefCell<PropertyUser>>, title : &str)
    {
        // the {} are for ending the borrow
        {
        let mut cur = self.current.borrow_mut();// = Some(RefMut::Cell(p));
        *cur = Some(RefMut::Cell(p.clone()));
        }
        self._set_prop(&*p.borrow().as_show(), title);
    }

    pub fn set_prop_arc(&self, p : Arc<RwLock<PropertyUser>>, title : &str)
    {
        // the {} are for ending the borrow
        {
        let mut cur = self.current.borrow_mut();// = Some(RefMut::Cell(p));
        *cur = Some(RefMut::Arc(p.clone()));
        }
        self._set_prop(&*p.read().unwrap().as_show(), title);
    }

    fn _set_prop(&self, p : &PropertyShow, title : &str)
    {
        unsafe { property_box_clear(self.jk_property); }
        *self.nodes.borrow_mut() = NodeChildren::None;

        println!("TODO set prop in property box>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>.");
        //TODO
        /*
        unsafe {
            property_list_group_add(
                self.jk_property,
                CString::new(title.as_bytes()).unwrap().as_ptr());
        }
        */
        //TODO replace ""
        //p.create_widget(self, "", 1, false);
        p.create_widget_inside("", self);
    }

    pub fn update_object_property(&self, object : &PropertyShow, prop : &str)
    {
        println!("TODO update_object_property for box");
        println!("boxxxxx UPDATE OBJECT PROP '{}'", prop);

        let yep = ui::make_vec_from_str(prop);
        object.update_property(self, prop, yep);

    }

    pub fn vec_add(&self, object : &PropertyShow, prop : &str, index : usize)
    {
        let yep = ui::make_vec_from_str(prop);
        object.update_property_new(self, prop, yep, PropertyChange::VecAdd(index));
    }

    pub fn vec_del(&self, object : &PropertyShow, prop : &str, index : usize)
    {
        let yep = ui::make_vec_from_str(prop);
        object.update_property_new(self, prop, yep, PropertyChange::VecDel(index));
    }

    pub fn update_object(&self, object : &PropertyShow, but : &str)
    {
        self.nodes.borrow().update(object, but);
    }

    pub fn set_nothing(&self)
    {
        println!("TODO");
    }

    pub fn set_visible(&self, b : bool)
    {
        self.visible.set(b);
        unsafe {
            println!("TODO visible");
            //ui::property::property_show(self.jk_property, b);
        }
    }

    pub fn visible(&self) -> bool
    {
        self.visible.get()
    }

    fn get_node(&self, path : &str) -> Option<Weak<RefCell<PropertyNode>>>
    {
        self.nodes.borrow().get_node(path)
    }

    fn add_common(&self, path : &str, item : *const PropertyValue) ->
        (Option<Rc<RefCell<PropertyNode>>>, Rc<RefCell<PropertyNode>>)
    {
        let v : Vec<&str> = path.rsplitn(2,"/").collect();

        let (parent_path, field_name) = if v.len() == 2 {
            (v[1],v[0])
        }
        else if v.len() == 1 {
            ("", v[0])
        }
        else {
            panic!("property pb");
        };

        let parent_node = if let Some(pv) = self.get_node(parent_path)
        {
            if let Some(rcv) = pv.upgrade()
            {
                Some(rcv)
            }
            else {
                panic!("cannot updgrade the value");
            }
        }
        else {
            println!("could not find parent : {}", parent_path );
            None
        };

        let new_node = Rc::new(RefCell::new(PropertyNode::new(field_name, item)));

        if let Some(ref n) = parent_node {
            ui::node_add_child(
                field_name,
                n.clone(),
                new_node.clone());
        }
        else {
            self.nodes.borrow_mut().add_node(
                field_name,
                new_node.clone());
        };

        (parent_node, new_node)
    }

    fn del_common(&self, path : &str) ->
        Option<Rc<RefCell<PropertyNode>>>
    {
        let v : Vec<&str> = path.rsplitn(2,"/").collect();

        let (parent_path, field_name) = if v.len() == 2 {
            (v[1],v[0])
        }
        else if v.len() == 1 {
            ("", v[0])
        }
        else {
            panic!("property pb");
        };

        let parent_node = if let Some(pv) = self.get_node(parent_path)
        {
            if let Some(rcv) = pv.upgrade()
            {
                Some(rcv)
            }
            else {
                panic!("cannot updgrade the value");
            }
        }
        else {
            None
        };

        if let Some(ref n) = parent_node {
            n.borrow_mut().del_child(field_name);
        }
        else {
            self.nodes.borrow_mut().del_node(field_name);
        };

        parent_node
    }


}


impl PropertyWidget for PropertyBox
{
    fn add_simple_item(&self, path : &str, item : *const PropertyValue)
    {
        let (parent, node) = self.add_common(path, item);

        let parent_value = if let Some(ref n) = parent {
            n.borrow().value
        }
        else {
            ptr::null()
        };

        unsafe {
            property_box_single_item_add(
                self.jk_property,
                mem::transmute(box Rc::downgrade(&node)),
                item,
                parent_value);
        }
    }

    fn add_option(&self, field : &str, is_some : bool) -> *const PropertyValue
    {
        println!("TODO");
        ptr::null()
    }

    fn add_vec(&self, name : &str, len : usize)
    {
        println!("TODO");
    }

    fn add_vec_item(&self, path : &str, item : *const PropertyValue, index : usize)
    {
        let (parent, node) = self.add_common(path, item);

        let parent_value = if let Some(ref n) = parent {
            n.borrow().value
        }
        else {
            ptr::null()
        };

        unsafe {
            property_box_vec_item_add(
                self.jk_property,
                mem::transmute(box Rc::downgrade(&node)),
                item,
                parent_value,
                index as c_int);
        }

        if let Some(ref p) = parent {
            self.update_vec(parent_value, p.borrow().get_child_count());
        }
    }

    fn del_vec_item(&self, path : &str, index : usize)
    {
        let parent_node = self.del_common(path);

        let parent_value = if let Some(ref p) = parent_node {
            let p = p.borrow();
            self.update_vec(p.value, p.get_child_count());
            p.value
        }
        else {
            ptr::null()
        };
        
        unsafe {
            property_box_vec_item_del(
                self.jk_property,
                parent_value,
                index as c_int);
        }
    }


    fn update_enum(&self, path : &str, widget_entry : *const PropertyValue, value : &str)
    {
        println!("TODO   !!!!! [{}] update enum BOX ::::::::::: {}", path, value);
        let v = CString::new(value.as_bytes()).unwrap();
        unsafe {
            property_box_enum_update(self.jk_property, widget_entry, v.as_ptr());

        }
    }

    fn update_vec(&self, widget_entry : *const PropertyValue, len : usize)
    {
        unsafe {
            property_box_vec_update(self.jk_property, widget_entry, len as c_int);

        }

    }


    fn get_current(&self) -> Option<RefMut<PropertyUser>>
    {
        if let Some(ref cur) = *self.current.borrow() {
            Some(cur.clone())
        }
        else {
            None
        }
    }

    fn set_current(&self, p : RefMut<PropertyUser>, title : &str)
    {
        let mut cur = self.current.borrow_mut();// = Some(RefMut::Cell(p));
        *cur = Some(p.clone());
        //self._set_prop(&*p.borrow().as_show(), title);

        match p {
            RefMut::Arc(ref a) => 
                self._set_prop(&*a.read().unwrap().as_show(), title),
            RefMut::Cell(ref c) => 
                self._set_prop(&*c.borrow().as_show(), title),
        }
    }

    fn get_property(&self, path : &str) -> Option<*const PropertyValue> 
    {
        //self.pv.borrow().get(path).map( |o| *o)
        if let Some(n) = self.get_node(path) {
            n.upgrade().map(|o| o.borrow().value)
        }
        else {
            None
        }
    }

}

impl ui::Widget for PropertyBox
{
    fn get_id(&self) -> Uuid
    {
        self.id
    }

    fn handle_change_prop(&self, prop_user : &PropertyUser, name : &str)
    {
        self.update_object_property(prop_user.as_show(), name);
    }
}

