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
    PropertyTreeFunc, PropertyConfig, PropertyValue, RefMut, PropertyUser, PropertyShow, PropertyWidget};
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

#[repr(C)]
pub struct JkPropertyBox;

#[link(name = "joker")]
extern {
    fn jk_property_box_new(
        window : *const Window,
        x : c_int,
        y : c_int,
        w : c_int,
        h : c_int
        ) -> *const JkPropertyBox;


    fn property_box_clear(pl : *const JkPropertyBox);

    pub fn jk_property_box_register_cb(
        property : *const JkPropertyBox,
        data : *const PropertyBox,
        panel_move : ui::PanelGeomFunc
        );

    pub fn property_box_cb_get(pl : *const JkPropertyBox) -> *const ui::JkPropertyCb;

    fn property_box_single_item_add(
        ps : *const JkPropertyBox,
        container: *const PropertyValue,
        ) -> *const PropertyValue;

    fn property_box_single_node_add(
        pl : *const JkPropertyBox,
        val : *const PropertyValue,
        ) -> *const PropertyValue;

    fn property_box_enum_update(
        pb : *const JkPropertyBox,
        pv : *const PropertyValue,
        value : *const c_char);

    /*
    pub fn jk_property_box_register_cb(
        property : *const JkPropertyBox,
        data : *const PropertyBox,
        changed_float : ChangedFunc,
        changed_string : ChangedFunc,
        changed_enum : ChangedFunc,
        register_change_string : RegisterChangeFunc,
        register_change_float : RegisterChangeFunc,
        register_change_enum : RegisterChangeFunc,
        register_change_option : RegisterChangeFunc,
        expand : PropertyTreeFunc,
        contract : PropertyTreeFunc,
        panel_move : ui::PanelGeomFunc
        );

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
    pub pv : RefCell<HashMap<String, *const PropertyValue>>,
    visible : Cell<bool>,
    pub id : uuid::Uuid,
    pub config : PropertyConfig,
    pub current : RefCell<Option<RefMut<PropertyUser>>>
}


impl PropertyBox
{
    pub fn new(
        window : *const Window,
        pc : &PropertyConfig
        ) -> PropertyBox
    {
        PropertyBox {
            name : String::from("property_box_name"),
            jk_property : unsafe {jk_property_box_new(
                    window,
                    pc.x, pc.y, pc.w, pc.h)},
            pv : RefCell::new(HashMap::new()),
            visible: Cell::new(true),
            id : uuid::Uuid::new_v4(),
            config : pc.clone(),
            current : RefCell::new(None)
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

        println!("TODO set prop in property box");
        //TODO
        /*
        unsafe {
            property_list_group_add(
                self.jk_property,
                CString::new(title.as_bytes()).unwrap().as_ptr());
        }
        */
        //TODO replace ""
        p.create_widget(self, "", 1, false);
    }

    pub fn update_object_property(&self, object : &PropertyShow, prop : &str)
    {
        println!("TODO update_object_property for box");

        let copy = self.pv.borrow().clone();

        println!("UPDATE OBJECT PROP '{}'", prop);

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
    }

    pub fn update_object(&self, object : &PropertyShow, but : &str)
    {

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

}

impl PropertyWidget for PropertyBox
{
    fn add_simple_item(&self, field : &str, item : *const PropertyValue)
    {
        unsafe {
            property_box_single_item_add(
                self.jk_property,
                item);
        }

        self.pv.borrow_mut().insert(field.to_owned(), item);
    }

    fn add_node_t(&self, field : &str, item : *const PropertyValue)
    {
        unsafe {
            property_box_single_node_add(
                self.jk_property,
                item);
        }

        let vs = ui::make_vec_from_str(field);//&field.to_owned());

        if let Some(ref cur) = *self.current.borrow() {
            match *cur {
                RefMut::Arc(ref a) =>
                {
                    a.read().unwrap().find_and_create(self, vs.clone(), 0);

                },
                RefMut::Cell(ref c) =>
                {
                    c.borrow().find_and_create(self, vs.clone(), 0);
                }
            }
        }
        else {
            println!("box, no current prop....... {}", field);
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

    fn add_vec_item(&self, field : &str, widget_entry : *const PropertyValue, is_node : bool)
    {
        println!("TODO");
    }

    fn update_enum(&self, widget_entry : *const PropertyValue, value : &str)
    {
        println!("TODO   !!!!! update enum BOX ::::::::::: {}", value);
        let v = CString::new(value.as_bytes()).unwrap();
        unsafe {
            property_box_enum_update(self.jk_property, widget_entry, v.as_ptr());
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

