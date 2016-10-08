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
        pv: *const PropertyValue,
        parent: *const PropertyValue,
        ) -> *const PropertyValue;

    fn property_box_vec_item_add(
        ps : *const JkPropertyBox,
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
    pv : RefCell<HashMap<String, *const PropertyValue>>,
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
            pv : RefCell::new(HashMap::new()),
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
        self.pv.borrow_mut().clear();
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

        //let copy = self.pv.borrow().clone();

        println!("boxxxxx UPDATE OBJECT PROP '{}'", prop);

        /*
        for (f,pv) in &copy {
            match self.pv.borrow().get(f) {
                Some(p) => if *p != *pv {
                    panic!("different pointer???");
                    continue
                },
                None => continue
            }

            if f.starts_with(prop) {
                let yep = ui::make_vec_from_str(f);
                //if let Some(ppp) = find_property_show(object, yep.clone()) {
                    //ppp.update_widget(*pv);
                //}
                //let test = |ps| {};
                object.update_property(self,yep, *pv);
                //object.callclosure(&test);
            }
        }

        match self.pv.borrow().get(prop) {
            Some(p) => {
                let yep = ui::make_vec_from_str(prop);
                println!("boxxxxx UPDATE OBJECT PROP 2222222222 '{:?}'", yep);
                object.update_property(self, yep, *p);
                println!("boxxxxx UPDATE OBJECT PROP 33333333333333333333333333 end ");
            },
            None => {}
        }
        */

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
        for (f,pv) in self.pv.borrow().iter() {
            let fstr : &str = f.as_ref();
            if fstr == but {
                println!("buuuuuuuuuuuuuuuuuuuuuuuuuut: {} ", f);
                continue;
            }
            let yep = ui::make_vec_from_str(f);
            match ui::find_property_show(object, yep.clone()) {
                Some(ppp) => {
                    ppp.update_widget(*pv);
                },
                None => {
                    println!("could not find prop : {:?}", yep);
                }
            }
        }

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

    fn find_parent_of(&self, path : &str) -> Option<*const PropertyValue>
    {
        let mut v : Vec<&str> = path.split("/").collect();
        if v.len() > 1 {
            v.pop();
            let s = util::join_str(&v);
            self.pv.borrow().get(&s).map(|o| *o)
        }
        else {
            None
        }
    }

    fn get_node(&self, path : &str) -> Option<Weak<PropertyNode>>
    {
        self.nodes.borrow().get_node(path)
    }


}


impl PropertyWidget for PropertyBox
{
    fn add_simple_item(&self, field : &str, item : *const PropertyValue)
    {
        let parent = if let Some(pv) = self.find_parent_of(field)
        {
            println!("FOUND THE FATHER");
            pv
        }
        else {
            ptr::null()
        };

        unsafe {
            property_box_single_item_add(
                self.jk_property,
                item,
                parent);
        }

        self.pv.borrow_mut().insert(field.to_owned(), item);
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

    fn add_vec_item(&self, field : &str, item : *const PropertyValue, index : usize)
    {
        //println!("TODO");

        let parent = if let Some(pv) = self.find_parent_of(field)
        {
            println!("FOUND THE FATHER");
            pv
        }
        else {
            ptr::null()
        };

        unsafe {
            property_box_vec_item_add(
                self.jk_property,
                item,
                parent,
                index as c_int);
        }

        self.pv.borrow_mut().insert(field.to_owned(), item);
    }

    fn del_vec_item(&self, field : &str, index : usize)
    {
        //println!("TODO");

        let parent = if let Some(pv) = self.find_parent_of(field)
        {
            println!("FOUND THE FATHER");
            pv
        }
        else {
            ptr::null()
        };

        unsafe {
            property_box_vec_item_del(
                self.jk_property,
                parent,
                index as c_int);
        }

        self.pv.borrow_mut().remove(field);
    }


    fn update_enum(&self, path : &str, widget_entry : *const PropertyValue, value : &str)
    {
        println!("TODO   !!!!! [{}] update enum BOX ::::::::::: {}", path, value);
        let v = CString::new(value.as_bytes()).unwrap();
        unsafe {
            property_box_enum_update(self.jk_property, widget_entry, v.as_ptr());

        }

        let copy = self.pv.borrow().clone();

        //println!("UPDATE OBJECT PROP '{}'", prop);

        for (f,pv) in &copy {
            /*
            match self.pv.borrow().get(f) {
                Some(p) => if *p != *pv {
                    panic!("different pointer???");
                    continue
                },
                None => continue
            }
            */

            /*
            println!("check this value '{}' with '{}'", f, path);

            if f != path && f.starts_with(path) {
                unsafe { property_box_remove(self.jk_property, *pv); }
            }
            */
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
        self.pv.borrow().get(path).map( |o| *o)
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

