use std::sync::{RwLock, Arc};
use std::collections::{HashMap,HashSet};
use libc::{c_char, c_void, c_int, c_float};
use std::str;
use std::mem;
use std::ptr;
use std::rc::{Rc,Weak};
use std::cell::{Cell, RefCell, BorrowState};
use std::ffi;
use std::ffi::{CStr,CString};
use uuid;
use uuid::Uuid;

use ui::{Window, ButtonCallback, ChangedFunc, RegisterChangeFunc, 
    PropertyTreeFunc, PropertyConfig, PropertyValue, RefMut, PropertyUser, PropertyShow, PropertyWidget,PropertyChange, NodeChildren, PropertyNode};
use ui;
use dormin::property;
use operation;
use control::WidgetUpdate;
use util;

#[repr(C)]
pub struct JkPropertyBox;

#[link(name = "joker")]
extern {
    fn jk_property_box_new(
        eo : *const ui::Evas_Object)
        -> *const JkPropertyBox;

    fn property_box_clear(pl : *const JkPropertyBox);

    pub fn property_box_cb_get(
        pl : *const JkPropertyBox
        ) -> *const ui::JkPropertyCb;

    fn property_box_single_item_add(
        ps : *const JkPropertyBox,
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

    fn property_box_enum_update(
        pb : *const JkPropertyBox,
        pv : *const PropertyValue,
        value : *const c_char);

    fn property_box_vec_update(
        pb : *const JkPropertyBox,
        pv : *const PropertyValue,
        len : c_int);
}

pub struct PropertyBox
{
    pub name : String,
    pub jk_property : *const JkPropertyBox,
    pub id : uuid::Uuid,
    current : RefCell<Option<RefMut<PropertyUser>>>,
    nodes : RefCell<NodeChildren>,
}


impl PropertyBox
{
    pub fn new(
        panel : &ui::WidgetPanel,
        ) -> PropertyBox
    {
        PropertyBox {
            name : String::from("property_box_name"),
            jk_property : unsafe {jk_property_box_new(
                    panel.eo,
                    )},
            id : uuid::Uuid::new_v4(),
            current : RefCell::new(None),
            nodes : RefCell::new(NodeChildren::None)
        }
    }

    pub fn set_prop_cell(&self, p : Rc<RefCell<PropertyUser>>, title : &str)
    {
        *self.current.borrow_mut() = Some(RefMut::Cell(p.clone()));
        self._set_prop(&*p.borrow().as_show(), title);
    }

    pub fn set_prop_arc(&self, p : Arc<RwLock<PropertyUser>>, title : &str)
    {
        *self.current.borrow_mut() = Some(RefMut::Arc(p.clone()));
        self._set_prop(&*p.read().unwrap().as_show(), title);
    }

    fn _set_prop(&self, p : &PropertyShow, title : &str)
    {
        unsafe { property_box_clear(self.jk_property); }
        *self.nodes.borrow_mut() = NodeChildren::None;
        p.create_widget_inside("", self);
    }

    pub fn update_object_property(&self, object : &PropertyShow, prop : &str)
    {
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
        unsafe { property_box_clear(self.jk_property); }
        *self.nodes.borrow_mut() = NodeChildren::None;
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
            panic!("property box add : path is empty.");
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
            panic!("property box del item : path is empty");
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
        panic!("TODO, add option");
        ptr::null()
    }

    fn add_vec(&self, name : &str, len : usize)
    {
        panic!("TODO, add vec");
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
        *self.current.borrow_mut() = Some(p.clone());

        match p {
            RefMut::Arc(ref a) => 
                self._set_prop(&*a.read().unwrap().as_show(), title),
            RefMut::Cell(ref c) => 
                self._set_prop(&*c.borrow().as_show(), title),
        }
    }

    fn get_property(&self, path : &str) -> Option<*const PropertyValue> 
    {
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

