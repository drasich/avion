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
use ui::{Window, ButtonCallback};
use ui::{ChangedFunc, RegisterChangeFunc, PropertyTreeFunc, PropertyValue, PropertyConfig, PropertyUser,
PropertyShow, PropertyId, RefMut, Elm_Object_Item, ShouldUpdate, PropertyWidget};
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
pub struct JkPropertyList;



pub struct PropertyList
{
    pub name : String,
    pub jk_property_list : *const JkPropertyList,
    pub pv : RefCell<HashMap<String, *const PropertyValue>>,
    visible : Cell<bool>,
    pub id : uuid::Uuid,
    pub config : PropertyConfig,
    pub current : RefCell<Option<RefMut<PropertyUser>>>
}

impl PropertyList
{
    pub fn new(
        window : *const Window,
        pc : &PropertyConfig
        ) -> PropertyList
    {
        PropertyList {
            name : String::from("property_name"),
            jk_property_list : unsafe {jk_property_list_new(
                    window,
                    pc.x, pc.y, pc.w, pc.h)},
            pv : RefCell::new(HashMap::new()),
            visible: Cell::new(true),
            id : uuid::Uuid::new_v4(),
            config : pc.clone(),
            current : RefCell::new(None)
        }
    }

    /*
    pub fn set_object(&mut self, o : &object::Object)
    {
        unsafe { property_list_clear(self.jk_property_list); }
        self.pv.clear();

        unsafe {
            property_list_group_add(
                self.jk_property_list,
                CString::new("object".as_bytes()).unwrap().as_ptr());
        }
        //let mut v = Vec::new();
        //v.push("object".to_owned());
        o.create_widget(self, "object", 1, false);

        self.add_tools();
    }
    */

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
        unsafe { property_list_clear(self.jk_property_list); }
        self.pv.borrow_mut().clear();

        unsafe {
            property_list_group_add(
                self.jk_property_list,
                CString::new(title.as_bytes()).unwrap().as_ptr());
        }
        //TODO replace ""
        p.create_widget(self, "", 1, false);

        self.add_tools();
    }


    /*
    pub fn set_scene(&mut self, s : &scene::Scene)
    {
        unsafe { property_list_clear(self.jk_property_list); }
        self.pv.clear();

        unsafe {
            property_list_group_add(
                self.jk_property_list,
                CString::new("scene".as_bytes()).unwrap().as_ptr());
        }
        //let mut v = Vec::new();
        //v.push("object".to_owned());
        s.create_widget(self, "scene", 1, false);
    }
    */


    fn add_tools(&self)
    {
        //add component
        // add as prefab
        // if linked to prefab :
        // State : linked, inherit
        // operation : change state : if linked, remove link(set independant)
        //TODO
        unsafe {
            property_list_group_add(
                self.jk_property_list,
                CString::new("tools").unwrap().as_ptr());
        }
    }


    pub fn set_nothing(&self)
    {
        unsafe { property_list_clear(self.jk_property_list); }
        self.pv.borrow_mut().clear();

        //self.current = None;
        *(self.current.borrow_mut()) = None;
    }

    pub fn data_set(&self, data : *const c_void)
    {
        //TODO
        //unsafe { property_data_set(self.jk_property, data); }
    }

    pub fn update_object_property(&self, object : &PropertyShow, prop : &str)
    {
        // update_widget might add/remove/update self.pv so we have to copy it
        // and check
        let copy = self.pv.borrow().clone();

        println!("UPDATE OBJECT PROP '{}'", prop);

        for (f,pv) in &copy {
            match self.pv.borrow().get(f) {
                Some(p) => if *p != *pv {
                    continue
                },
                None => continue
            }

            if f.starts_with(prop) {
                let yep = make_vec_from_str(f);
                //if let Some(ppp) = find_property_show(object, yep.clone()) {
                    //ppp.update_widget(*pv);
                //}
                //let test = |ps| {};
                object.update_property(yep, *pv);
                //object.callclosure(&test);
            }
        }
    }

    pub fn update_object(&self, object : &PropertyShow, but : &str)
    {
        // update_widget might add/remove/update self.pv so we have to copy it
        // and check
        let copy = self.pv.borrow().clone();
        for (f,pv) in &copy {
            match self.pv.borrow().get(f) {
                Some(p) => if *p != *pv {
                    continue
                },
                None => continue
            }
            let fstr : &str = f.as_ref();
            //if f.as_ref() as &str == but {
            if fstr == but {
                println!("buuuuuuuuuuuuuuuuuuuuuuuuuut: {} ", f);
                continue;
            }
            let yep = make_vec_from_str(f);
            match find_property_show(object, yep.clone()) {
                Some(ppp) => {
                    ppp.update_widget(*pv);
                },
                None => {
                    println!("could not find prop : {:?}", yep);
                }
            }
        }
    }

    pub fn add_node(
        &self,
        ps : &PropertyShow,
        name : &str,
        has_container : bool,
        added_name : Option<&str>,
        ) -> *const PropertyValue
    {
        let f = CString::new(name.as_bytes()).unwrap();
        let mut pv = unsafe {
            let test = if has_container {
                if let Some(n) = added_name {
                    CString::new(n).unwrap().as_ptr()
                }
                else {
                    ptr::null()
                }
            }
            else {
                ptr::null()
            };
            println!("adding node : {}", name);
            property_list_node_add(
                f.as_ptr(),
                test
                )
        };

        if !has_container {
            println!(".......with single node : {}", name);
            self.add_node_t(name, pv);
        }

        return pv;
    }

    pub fn add_enum(
        &self,
        path : &str,
        types : &str,
        value : &str,
        is_node : bool,
        has_container : bool
        ) -> *const PropertyValue
    {
        let f = CString::new(path.as_bytes()).unwrap();
        let types = CString::new(types.as_bytes()).unwrap();
        let v = CString::new(value.as_bytes()).unwrap();

        let pv = unsafe {
            property_list_enum_add(
                f.as_ptr(),
                types.as_ptr(),
                v.as_ptr())

        };

        if !has_container {
            if is_node {
                self.add_node_t(path, pv);
            }
            else {
                self.add_simple_item(path, pv);
            }
        }

        pv
    }

    pub fn set_visible(&self, b : bool)
    {
        self.visible.set(b);
        unsafe {
            property_show(self.jk_property_list, b);
        }
    }

    pub fn visible(&self) -> bool
    {
        self.visible.get()
    }

}

impl ui::Widget for PropertyList
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


