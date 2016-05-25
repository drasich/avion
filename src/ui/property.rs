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
PropertyShow, PropertyId, RefMut, Elm_Object_Item, ShouldUpdate};
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
pub struct JkProperty;
#[repr(C)]
pub struct JkPropertySet;
#[repr(C)]
pub struct JkPropertyList;

#[link(name = "png")]

#[link(name = "ecore_evas")]
#[link(name = "ecore_file")]
#[link(name = "elementary")]
#[link(name = "eina")]
#[link(name = "eet")]
#[link(name = "evas")]
#[link(name = "ecore")]
#[link(name = "edje")]
#[link(name = "eo")]
//#[link(name = "GLESv2")]
#[link(name = "joker")]
extern {

    fn jk_property_list_new(
        window : *const Window,
        x : c_int,
        y : c_int,
        w : c_int,
        h : c_int
        ) -> *const JkPropertyList;

    fn property_list_clear(pl : *const JkPropertyList);

    pub fn jk_property_list_register_cb(
        property : *const JkPropertyList,
        data : *const Property,
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

    pub fn jk_property_list_register_vec_cb(
        property : *const JkPropertyList,
        vec_add : RegisterChangeFunc,
        vec_del : RegisterChangeFunc);

    fn property_list_group_add(
        pl : *const JkPropertyList,
        name : *const c_char
        );

    fn property_list_node_add(
        pl : *const JkPropertyList,
        path : *const c_char,
        added_name : *const c_char
        ) -> *const PropertyValue;

    fn property_list_single_node_add(
        pl : *const JkPropertyList,
        val : *const PropertyValue,
        ) -> *const PropertyValue;

    fn property_list_vec_add(
        pl : *const JkPropertyList,
        name : *const c_char,
        len : c_int
        ) -> *const PropertyValue;

    fn property_list_nodes_remove(
        pl : *const JkPropertyList,
        name : *const c_char
        );

    fn property_list_float_add(
        ps : *const JkPropertyList,
        name : *const c_char,
        value : c_float
        ) -> *const PropertyValue;

    fn property_list_string_add(
        ps : *const JkPropertyList,
        name : *const c_char,
        value : *const c_char
        ) -> *const PropertyValue;

    fn property_list_single_item_add(
        ps : *const JkPropertyList,
        container: *const PropertyValue,
        ) -> *const PropertyValue;

    fn property_list_single_vec_add(
        ps : *const JkPropertyList,
        container: *const PropertyValue,
        is_node : bool
        ) -> *const PropertyValue;

    fn property_list_node_vec_add(
        ps : *const JkPropertyList,
        container: *const PropertyValue,
        ) -> *const PropertyValue;

    fn property_list_vec_update(
        pv : *const PropertyValue,
        len : c_int);

    /*
    fn property_list_vec_item_add(
        ps : *const JkPropertyList,
        name : *const c_char,
        value : *const c_char
        ) -> *const PropertyValue;
        */

    fn property_list_enum_add(
        ps : *const JkPropertyList,
        name : *const c_char,
        possible_values : *const c_char,
        value : *const c_char
        ) -> *const PropertyValue;

    fn property_list_option_add(
        ps : *const JkPropertyList,
        name : *const c_char,
        value : *const c_char
        ) -> *const PropertyValue;

    fn property_list_option_update(
        pv : *const PropertyValue,
        value : *const c_char);

    fn property_list_string_update(
        pv : *const PropertyValue,
        value : *const c_char);

    fn property_list_float_update(
        pv : *const PropertyValue,
        value : c_float);

    fn property_expand(
        pv : *const PropertyValue);

    fn property_show(obj : *const JkPropertyList, b : bool);

    fn property_list_enum_update(
        pv : *const ui::PropertyValue,
        value : *const c_char);
}

pub struct Property
{
    pub name : String,
    pub jk_property_list : *const JkPropertyList,
    pub pv : RefCell<HashMap<String, *const PropertyValue>>,
    visible : Cell<bool>,
    pub id : uuid::Uuid,
    pub config : PropertyConfig,
    pub current : RefCell<Option<RefMut<PropertyUser>>>
}

impl Property
{
    pub fn new(
        window : *const Window,
        pc : &PropertyConfig
        ) -> Property
    {
        Property {
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
                self.jk_property_list,
                f.as_ptr(),
                test
                )
        };

        if !has_container {
            println!(".......with single node : {}", name);
            unsafe {
            pv = property_list_single_node_add(
                self.jk_property_list,
                pv);
            }
        }

        if pv != ptr::null() {
            self.pv.borrow_mut().insert(name.to_owned(), pv);
        }

        if self.config.expand.contains(name) {
            unsafe {
                property_expand(pv);
            }
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
                self.jk_property_list,
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

impl ui::Widget for Property
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


pub extern fn name_get(data : *const c_void) -> *const c_char {

    let o : &Arc<RwLock<object::Object>> = unsafe {
        mem::transmute(data)
    };

    let cs = CString::new(o.read().unwrap().name.as_bytes()).unwrap();
    //println!("..........name get {:?}", cs);
    cs.as_ptr()
}

pub extern fn changed_set_float(
    property : *const c_void,
    name : *const c_char,
    data : *const c_void) {

    let f : & f64 = unsafe {mem::transmute(data)};
    changed_set(property, name, None, f, 0);
}

pub extern fn changed_set_string(
    property : *const c_void,
    name : *const c_char,
    data : *const c_void) {

    let datachar = data as *const i8;
    let s = unsafe {CStr::from_ptr(datachar).to_bytes()};
    let ss = match str::from_utf8(s) {
        Ok(sss) => sss.to_owned(),
        _ => {
            return;
        }
    };
    changed_set(property, name, None, &ss, 0);
}

pub extern fn changed_set_enum(
    property : *const c_void,
    name : *const c_char,
    data : *const c_void) {
    println!("DOES NOT NO ANYTHING");
}


pub extern fn register_change_string(
    property : *const c_void,
    name : *const c_char,
    old : *const c_void,
    new : *const c_void,
    action : c_int
    ) {

    let newchar = new as *const i8;
    let s = unsafe {CStr::from_ptr(newchar).to_bytes()};
    let ss = match str::from_utf8(s) {
        Ok(sss) => sss.to_owned(),
        _ => {
            println!("error");
            return;
        }
    };

    if action == 1 && old != ptr::null() {
        let oldchar = old as *const i8;
        let so = unsafe {CStr::from_ptr(oldchar).to_bytes()};
        let sso = match str::from_utf8(so) {
            Ok(ssso) => ssso.to_owned(),
            _ => {
                println!("error");
                return;
            }
        };

        changed_set(property, name, Some(&sso), &ss, action);
    }
    else {
        changed_set(property, name, None, &ss, action);
    }
}

pub extern fn register_change_float(
    property : *const c_void,
    name : *const c_char,
    old : *const c_void,
    new : *const c_void,
    action : c_int
    ) {

    let fnew : & f64 = unsafe {mem::transmute(new)};

    if action == 1 && old != ptr::null() {
        let fold : & f64 = unsafe {mem::transmute(old)};
        changed_set(property, name, Some(fold), fnew, action);
    }
    else {
        changed_set(property, name, None, fnew, action);
    }
}

pub extern fn register_change_enum(
    widget_cb_data : *const c_void,
    name : *const c_char,
    old : *const c_void,
    new : *const c_void,
    action : c_int
    ) {

    let newchar = new as *const i8;
    let s = unsafe {CStr::from_ptr(newchar).to_bytes()};
    let ss = match str::from_utf8(s) {
        Ok(sss) => sss.to_owned(),
        _ => {
            println!("error");
            return
        }
    };

    //println!("the string is {}", ss);
    if action == 1 && old != ptr::null() {
        let oldchar = old as *const i8;
        let so = unsafe {CStr::from_ptr(oldchar).to_bytes()};
        let sso = match str::from_utf8(so) {
            Ok(ssso) => ssso.to_owned(),
            _ => {
                println!("error");
                return
            }
        };
        changed_enum(widget_cb_data, name, &ss);
    }
    else {
        changed_enum(widget_cb_data, name, &ss);
    }
}

pub extern fn register_change_option(
    widget_cb_data : *const c_void,
    name : *const c_char,
    old : *const c_void,
    new : *const c_void,
    action : c_int
    ) {

    let newchar = new as *const i8;
    let s = unsafe {CStr::from_ptr(newchar).to_bytes()};
    let ss = match str::from_utf8(s) {
        Ok(sss) => sss.to_owned(),
        _ => {
            println!("error");
            return
        }
    };

    //println!("the string is {}", ss);
    if old == ptr::null() {
        println!("option : old is null, return");
        return;
    }

    let oldchar = old as *const i8;
    let so = unsafe {CStr::from_ptr(oldchar).to_bytes()};
    let sso = match str::from_utf8(so) {
        Ok(ssso) => ssso.to_owned(),
        _ => {
            println!("error");
            return
        }
    };

    changed_option(widget_cb_data, name, &sso, &ss);
}

fn get_str<'a>(cstr : *const c_char) -> Option<&'a str>
{
    let s = unsafe {CStr::from_ptr(cstr).to_bytes()};
    str::from_utf8(s).ok()
}

fn get_widget_data<'a>(widget_data : *const c_void) -> (&'a mut ui::Property, &'a mut Box<ui::WidgetContainer>)
{
    let wcb : & ui::WidgetCbData = unsafe {mem::transmute(widget_data)};
    let p : &mut ui::Property = unsafe {mem::transmute(wcb.widget)};
    let container : &mut Box<ui::WidgetContainer> = unsafe {mem::transmute(wcb.container)};

    (p, container)
}

fn changed_set<T : Any+Clone+PartialEq>(
    widget_data : *const c_void,
    name : *const c_char,
    old : Option<&T>,
    new : &T,
    action : i32
    )
{
    let path = if let Some(p) = get_str(name) {
        p
    }
    else {
        return;
    };

    let (p, container) = get_widget_data(widget_data);

    let change = match (old, action) {
        (Some(oldd), 1) => {
            if let Some(ref cur) = *p.current.borrow() {
                container.request_operation_property_old_new(
                    (*cur).clone(),
                    path,
                    box oldd.clone(),
                    box new.clone())
            }
            else
            {
                println!("property widget doesn't seem to have a property set to it");
                operation::Change::None
                /*
                container.request_operation_old_new(
                    path,
                    box oldd.clone(),
                    box new.clone())
                    */
            }
        },
        _ => {
            if let Some(ref cur) = *p.current.borrow() {
                match *cur {
                    RefMut::Arc(ref a) =>
                        container.request_direct_change_property(&mut *a.write().unwrap(),path,new),
                        RefMut::Cell(ref c) =>
                            container.request_direct_change_property(&mut *c.borrow_mut(),path,new)
                }
            }
            //container.request_direct_change(path, new)
            else {
                operation::Change::None
            }
        }
    };

    container.handle_change(&change, p.id);
}

fn changed_enum<T : Any+Clone+PartialEq>(
    widget_data : *const c_void,
    name : *const c_char,
    new : &T,
    )
{
    let path = if let Some(p) = get_str(name) {
        p
    }
    else {
        return;
    };

    let (p, container) = get_widget_data(widget_data);

    let change = {
        /*
        container.request_operation_old_new_enum(
            path,
            box new.clone())
            */

        if let Some(ref cur) = *p.current.borrow() {

            let option = match *cur {
                RefMut::Arc(ref a) => a.read().unwrap().get_property_hier(path),
                RefMut::Cell(ref c) => c.borrow().get_property_hier(path)
            };

            if let Some(old) = option {
                container.request_operation_property_old_new_dontcheckequal(
                    (*cur).clone(),
                    path,
                    old,
                    box new.clone())
            }
            else {
                operation::Change::None
            }
        }
        else
        {
            println!("property widget doesn't seem to have a property set to it");
            operation::Change::None
        }

    };

    container.handle_change(&change, p.id);
}

fn changed_option(
    widget_cb_data : *const c_void,
    name : *const c_char,
    old : &str,
    new : &str
    )
{
    let path = if let Some(p) = get_str(name) {
        p
    }
    else {
        return;
    };

    let (p, container) = get_widget_data(widget_cb_data);

    let change = if let Some(ref cur) = *p.current.borrow() {

        if new == "Some" {
            container.request_operation_option_to_some(cur.clone(), path)
        }
        else {

            let option = match *cur {
                RefMut::Arc(ref a) => a.read().unwrap().get_property_hier(path),
                RefMut::Cell(ref c) => c.borrow().get_property_hier(path)
            };

            if let Some(old) = option {
                container.request_operation_option_to_none(
                    (*cur).clone(),
                    path,
                    old)
            }
            else {
                operation::Change::None
            }
            //container.request_operation_option_to_none(path)
        }
    }
    else {
        operation::Change::None
    };

    container.handle_change(&change, p.id);
}

pub extern fn expand(
    widget_cb_data: *const c_void,
    data : *const c_void,
    parent : *const Elm_Object_Item) -> ()
{
    let datachar = data as *const i8;
    let s = unsafe {CStr::from_ptr(datachar).to_bytes()};

    let (p, container) = get_widget_data(widget_cb_data);

    let path = match str::from_utf8(s) {
        Ok(pp) => pp,
        _ => {
            panic!("problem with the path");
        }
    };

    let vs = make_vec_from_str(&path.to_owned());

    //TODO factorize this and others
    println!("factorize this and others, the path is : {:?}", vs);
    if let Some(ref cur) = *p.current.borrow() {
        match *cur {
            RefMut::Arc(ref a) =>
            {
                a.read().unwrap().find_and_create(p, vs.clone(), 0);

            },
            RefMut::Cell(ref c) =>
            {
                c.borrow().find_and_create(p, vs.clone(), 0);
            }
        }

        p.config.expand.insert(path.to_owned());
    }
    else {
        println!("no current prop....... {}", path);
    }

}

pub extern fn contract(
    widget_cb_data: *const c_void,
    data : *const c_void,
    parent : *const Elm_Object_Item) -> ()
{
    let (p,_) = get_widget_data(widget_cb_data);

    unsafe {
        property_list_nodes_remove(
            p.jk_property_list,
            data as *const c_char
            );
    };

    let datachar = data as *const i8;
    let s = unsafe {CStr::from_ptr(datachar).to_bytes()};
    let path = match str::from_utf8(s) {
        Ok(pp) => pp,
        _ => {
            println!("problem with the path");
            return;}
    };

    p.config.expand.remove(path);

    let clone = p.pv.borrow().clone();

    for (key,pv) in &clone {
        let starts_with_path = {
            let ks : &str = key.as_ref();
            ks.starts_with(path) && ks != path
        };

        //if key.as_ref().starts_with(path) && key.as_ref() != path  {
        if starts_with_path {
            match p.pv.borrow_mut().remove(key) {
                Some(_) => println!("yes I removed {}", key),
                None => println!("could not find {}", key)
            }
        }
    }
}

impl WidgetUpdate for Property
{
    fn update_changed(
        &mut self,
        name : &str,
        new : &Any)
    {

        //println!("property update changed {}", name);
        //
        let pvs = self.pv.borrow();

        let pv = match pvs.get(&name.to_owned()) {
            Some(p) => p,
            None => {
                println!("widget update, could not find {}", name);
                return;
            }
        };

        match new.downcast_ref::<f64>() {
            Some(v) => {
                unsafe {
                    property_list_float_update(
                        *pv,
                        *v as c_float);
                };
                return;
            },
            None => {
                println!("cannot downcast to f64");
            }
        }

        match new.downcast_ref::<String>() {
            Some(s) => {
                let v = CString::new(s.as_bytes()).unwrap();
                unsafe {
                    property_list_string_update(
                        *pv,
                        v.as_ptr());
                };
                return;
            },
            None => {
                println!("cannot downcast to string");
            }
        }

    }
}

/*
impl PropertyShow for vec::Quat {

    fn create_entries(
        &mut self,
        property:&mut Property,
        path : Vec<String>)
    {
        println!("quuuuuuuuuuuuuuuuaaaaaaaaaaaaaaaaaaaaaaat {} ", path);
    }
}
*/

impl PropertyShow for f64 {

    fn create_widget(
        &self,
        property : &Property,
        field : &str,
        depth : i32,
        has_container : bool ) -> Option<*const PropertyValue>
    {
        let f = CString::new(field.as_bytes()).unwrap();
        println!("create f64 for : {}", field);
        let pv = unsafe { 
            property_list_float_add(
                property.jk_property_list,
                f.as_ptr(),
                *self as c_float)
        };

        if !has_container {
            property.add_simple_item(field, pv);
            None
        }
        else {
            Some(pv)
        }
    }

    fn update_widget(&self, pv : *const PropertyValue) {
        unsafe {
            property_list_float_update(
                pv,
                *self as c_float);
        };
    }
}

impl PropertyShow for String {

    fn create_widget(
        &self,
        property : &Property,
        field : &str,
        depth : i32,
        has_container : bool ) -> Option<*const PropertyValue>
    {
        let f = CString::new(field.as_bytes()).unwrap();
        let v = CString::new(self.as_bytes()).unwrap();

        let pv = unsafe {
            property_list_string_add(
                property.jk_property_list,
                f.as_ptr(),
                v.as_ptr())
        };

        if !has_container {
            property.add_simple_item(field, pv);
            None
        }
        else {
            Some(pv)
        }
    }

    fn update_widget(&self, pv : *const PropertyValue) {
        let v = CString::new(self.as_bytes()).unwrap();
        unsafe {
            property_list_string_update(
                pv,
                v.as_ptr());
        };
    }
}

impl<T : PropertyShow> PropertyShow for Box<T> {

    fn create_widget(
        &self,
        property : &Property,
        field : &str,
        depth : i32,
        has_container : bool ) -> Option<*const PropertyValue>
    {
        (**self).create_widget(property ,field, depth, has_container)
    }

    fn get_property(&self, field : &str) -> Option<&PropertyShow>
    {
        (**self).get_property(field)
    }

    fn update_property(&self, path : Vec<String>, pv :*const PropertyValue)
    {
        (**self).update_property(path, pv);
    }

    fn find_and_create(&self, property : &Property, path : Vec<String>, start : usize)
    {
        (**self).find_and_create(property, path, start);
    }

    fn is_node(&self) -> bool
    {
        (**self).is_node()
    }

    fn to_update(&self) -> ShouldUpdate
    {
        (**self).to_update()
    }
}

impl<T : PropertyShow> PropertyShow for Rc<RefCell<T>> {

    fn create_widget(
        &self,
        property : &Property,
        field : &str,
        depth : i32,
        has_container : bool ) -> Option<*const PropertyValue>
    {
        self.borrow().create_widget(property ,field, depth, has_container)
    }

    fn get_property(&self, field : &str) -> Option<&PropertyShow>
    {
        //(**self).get_property(field)
        //(**self).get_property(field)
        None
    }

    fn update_property(&self, path : Vec<String>, pv :*const PropertyValue)
    {
        //(**self).update_property(path, pv);
        self.borrow().update_property(path, pv);
    }

    fn find_and_create(&self, property : &Property, path : Vec<String>, start : usize)
    {
        self.borrow().find_and_create(property, path, start);
    }

    fn is_node(&self) -> bool
    {
        //(**self).is_node()
        self.borrow().is_node()
    }

    fn to_update(&self) -> ShouldUpdate
    {
        self.borrow().to_update()
    }
}

impl<T : PropertyShow> PropertyShow for Option<T> {

    fn create_widget(
        &self,
        property : &Property,
        field : &str,
        depth : i32,
        has_container : bool ) -> Option<*const PropertyValue>
    {
        if depth == 0 {
            unsafe {
                property.add_option(field, self.is_some());
                return None;
            }
        }

        if depth == 1 {
            if let Some(ref s) = *self {
                return s.create_widget(property, field, depth, has_container);
            };
        }

        None
    }

    fn get_property(&self, field : &str) -> Option<&PropertyShow>
    {
        match *self {
            Some(ref s) =>
                s.get_property(field),
            None => None
        }
    }

    fn update_widget(&self, pv : *const PropertyValue) {
        let s = match *self {
            Some(_) => "Some",
            None => "None"
        };
        let v = CString::new(s.as_bytes()).unwrap();
        unsafe {
            property_list_option_update(
                pv,
                v.as_ptr());
        };
    }

    fn find_and_create(&self, property : &Property, path : Vec<String>, start : usize)
    {
        if let Some(ref s) = *self {
                s.find_and_create(property, path, start);
        }
    }

    fn to_update(&self) -> ShouldUpdate
    {
        match *self {
            Some(ref s) =>
                s.to_update(),
            None => ShouldUpdate::Nothing
        }
    }

}

impl<T> PropertyShow for resource::ResTT<T>
{
    fn create_widget(
        &self,
        property : &Property,
        field : &str,
        depth : i32,
        has_container : bool ) -> Option<*const PropertyValue>
    {
        if depth < 0 {
            return None;
        }

        if depth == 0 && field != ""
        {
            property.add_node(self, field, has_container, None);
        }

        if depth > 0 {
            let s = field.to_owned() + "/name";
            return self.name.create_widget(property, s.as_ref(), depth-1, has_container);
        }

        None
    }

    fn get_property(&self, field : &str) -> Option<&PropertyShow>
    {
        match field {
            "name" => Some(&self.name as &PropertyShow),
            _ => None
        }
    }
}

impl<T:PropertyShow> PropertyShow for Vec<T>
{
    fn create_widget(
        &self,
        property : &Property,
        field : &str,
        depth : i32,
        has_container : bool ) -> Option<*const PropertyValue>
    {
        if depth < 0 {
            return None;
        }

        if depth == 0 && field != ""
        {
            //TODO
            //TODO
            property.add_vec(field, self.len());
        }

        if depth > 0 {
            if self.is_empty() {
                //add "no item" item
            }

            for (n,i) in self.iter().enumerate() {
                let mut nf = String::from(field);
                nf.push_str("/");
                nf.push_str(n.to_string().as_str());
                if let Some(ref mut pv) = i.create_widget(property, nf.as_str(), depth -1, true) {
                    unsafe {
                        property.add_vec_item(nf.as_str(), *pv, i.is_node());
                    }
                }
                else {
                    println!("___ Vec : failed" );
                }
            }
        }

        None
    }

    fn get_property(&self, field : &str) -> Option<&PropertyShow>
    {
        match field.parse::<usize>() {
            Ok(index) => {
                if self.is_empty() || index > self.len() -1 {
                    println!("5555555555555555555get property of vec :: index is too big, or list is empty : {}, {}", index, self.len());
                    None
                }
                else {
                    Some(&self[index] as &PropertyShow)
                }
            }
            _ => {
                println!("$$$$$$$$$$$$$$$ Vec return none for field {}", field);
                None
            }
        }
    }

    fn update_widget(&self, pv : *const PropertyValue) {
        unsafe { property_list_vec_update(pv, self.len() as c_int); }
        unsafe { property_expand(pv); }
        /*
        for i in self.iter() {
            i.update_widget(pv);
        }
        */
    }
}

impl PropertyShow for CompData
{
    fn create_widget(
        &self,
        property : &Property,
        field : &str,
        depth : i32,
        has_container : bool ) -> Option<*const PropertyValue>
    {
        let kind : String = self.get_kind_string();
        let kindr : &str = kind.as_ref();
        let ss = field.to_owned() + "/" + kindr;
        //let ss = field.to_owned() + ":" + kindr;
        let s : &str = ss.as_ref();

        /*
        let mut v : Vec<&str> = field.split('/').collect();

        if v.len() >= 1 {
            v.pop();
        }

        v.push(kindr);

        //println!("--before compdata property show for : {}, {}, {}", field, depth, kind);

        let yo : String = v.join("/");
        let field = yo.as_ref();
        */


        if depth < 0 {
            return None;
        }

        if depth == 0 && field != ""
        {
            /*
            println!("00--> compdata property show for : {}, {}, {}", s, depth, kind );
            //let pv = property.add_node(self, s, has_container);
            let pv = property.add_node(self, field, has_container, Some(kindr));
            return Some(pv);
            */

            let type_value = self.get_kind_string();

            let types = CompData::get_all_kind();
            let pv = property.add_enum(field, types.as_str(), type_value.as_str(), true, has_container);
            return Some(pv);
        }

        if depth > 0
        {
            match *self {
                CompData::Player(ref p) => {
                    return p.create_widget(property, field, depth, has_container);
                },
                CompData::ArmaturePath(ref p) => {
                    return p.create_widget(property, field, depth, has_container);
                },
                CompData::MeshRender(ref p) => {
                    return p.create_widget(property, field, depth, has_container);
                },
                _ => {println!("not yet implemented"); }
            }
        }
        None
    }

    fn update_widget(&self, pv : *const PropertyValue) {
        match *self {
            CompData::Player(ref p) => {
                p.update_widget(pv);
            },
            CompData::ArmaturePath(ref p) => {
                p.update_widget(pv);
            },
            CompData::MeshRender(ref p) => {
                p.update_widget(pv);
            },
            _ => {println!("not yet implemented");}
        }
    }

    fn get_property(&self, field : &str) -> Option<&PropertyShow>
    {
        if field != self.get_kind_string() {
            return None;
        }

        match *self {
            CompData::Player(ref p) => {
                Some(p)
            },
            CompData::ArmaturePath(ref p) => {
                Some(p)
            },
            CompData::MeshRender(ref p) => {
                Some(p)
            },
            _ => {
                println!("not yet implemented");
                None
            }
        }
    }

    fn is_node(&self) -> bool
    {
        true
    }

    fn to_update(&self) -> ShouldUpdate
    {
        match *self {
            CompData::Player(ref p) => {
                p.to_update()
            },
            CompData::ArmaturePath(ref p) => {
                p.to_update()
            },
            CompData::MeshRender(ref p) => {
                p.to_update()
            },
            _ => {
                println!("not yet implemented");
                ShouldUpdate::Nothing
            }
        }
    }

}

impl ui::PropertyShow for Orientation {

    fn create_widget(
        &self,
        property : &ui::Property,
        field : &str,
        depth : i32,
        has_container : bool ) -> Option<*const ui::PropertyValue>
    {
        if depth == 0 {
            let type_value = match *self {
                Orientation::AngleXYZ(_) => "AngleXYZ",
                Orientation::Quat(_) => "Quat"
            };

            let types = "AngleXYZ/Quat";
            property.add_enum(field, types, type_value, true, has_container);
        }

        if depth == 1 {
            match *self {
                Orientation::AngleXYZ(ref v) =>  {
                    return v.create_widget(property, field, depth, has_container);
                },
                Orientation::Quat(ref q) => {
                    return q.create_widget(property, field, depth, has_container)
                }
            };
        }

        None
    }

    fn update_property(&self, path : Vec<String>, pv :*const PropertyValue)
    {
        println!("update property orientation : {:?} ", path);
        if path.is_empty() {
            self.update_widget(pv);
            return;
        }

        match *self {
            Orientation::AngleXYZ(ref v) =>  {
                match path[0].as_str() {
                    "x" => v.x.update_property(path[1..].to_vec(), pv),
                    "y" => v.y.update_property(path[1..].to_vec(), pv),
                    "z" => v.z.update_property(path[1..].to_vec(), pv),
                    _ => {}
                }
            },
            Orientation::Quat(ref q) => {
                match path[0].as_str() {
                    "x" => q.x.update_property(path[1..].to_vec(), pv),
                    "y" => q.y.update_property(path[1..].to_vec(), pv),
                    "z" => q.z.update_property(path[1..].to_vec(), pv),
                    "w" => q.w.update_property(path[1..].to_vec(), pv),
                    _ => {}
                }
            }
        }
    }


    fn update_widget(&self, pv : *const PropertyValue) {
        let type_value = match *self {
            Orientation::AngleXYZ(_) => "AngleXYZ",
            Orientation::Quat(_) => "Quat"
        };

        let v = CString::new(type_value.as_bytes()).unwrap();
        unsafe {
            property_list_enum_update(pv, v.as_ptr());
        }
    }

    fn get_property(&self, field : &str) -> Option<&ui::PropertyShow>
    {
        match *self {
            Orientation::AngleXYZ(ref v) =>  {
                match field {
                    "x" => Some(&v.x as &ui::PropertyShow),
                    "y" => Some(&v.y as &ui::PropertyShow),
                    "z" => Some(&v.z as &ui::PropertyShow),
                    _ => None
                }
            },
            Orientation::Quat(ref q) => {
                match field {
                    "x" => Some(&q.x as &ui::PropertyShow),
                    "y" => Some(&q.y as &ui::PropertyShow),
                    "z" => Some(&q.z as &ui::PropertyShow),
                    "w" => Some(&q.w as &ui::PropertyShow),
                    _ => None
                }
            }
        }
    }

}


macro_rules! property_show_methods(
    ($my_type:ty, [ $($member:ident),+ ]) => (

            fn create_widget(
                &self,
                property : &Property,
                field : &str,
                depth : i32,
                has_container : bool ) -> Option<*const PropertyValue>
            {
                if depth < 0 {
                    return None;
                }

                println!("macro create widget : {}, {}", field, depth);


                if depth == 0 && field != ""
                {
                    property.add_node(self, field, has_container, None);
                }

                if depth > 0 {
                $(
                    let s = if field != "" {
                        field.to_owned()
                            + "/"
                            + stringify!($member)
                    }else {
                        stringify!($member).to_owned()
                    };
                    self.$member.create_widget(property, s.as_ref(), depth-1, has_container);
                 )+
                }

                None
            }

            /*
            fn get_children(&self) -> Option<LinkedList<&PropertyShow>>
            {
                let mut list = LinkedList::new();
                $(
                    list.push_back(&self.$member as &PropertyShow);
                 )+

                if list.len() > 0 {
                    Some(list)
                }
                else {
                    None
                }
            }
            */

            fn get_property(&self, field : &str) -> Option<&PropertyShow>
            {
                match field {
                $(
                    stringify!($member) => Some(&self.$member as &PropertyShow),
                 )+
                    _ => None
                }
            }

            fn update_property(&self, path : Vec<String>, pv :*const PropertyValue)
            {
                if path.is_empty() {
                    self.update_widget(pv);
                    return;
                }

                match path[0].as_str() {
                $(
                    stringify!($member) => self.$member.update_property(path[1..].to_vec(), pv),
                 )+
                    _ => {}
                }
            }

            fn find_and_create(&self, property : &Property, path : Vec<String>, start : usize)
            {
                if path.is_empty() {
                    println!("macro create property 000000 : empty path");
                    self.create_widget(property, "" , 0, false);
                    return;
                }
                else if start == path.len() -1 {
                    match path[start].as_str() {
                        $(
                            stringify!($member) => {
                                self.$member.create_widget(property, join_string(&path).as_str(),1, false);
                            },
                            )+
                            _ => {}
                    }
                    return;
                }

                match path[start].as_str() {
                $(
                    stringify!($member) => self.$member.find_and_create(property, path, start + 1),
                 )+
                    _ => {}
                }
            }


            fn is_node(&self) -> bool
            {
                true
            }
    )
);

macro_rules! property_show_impl(
    ($my_type:ty, $e:tt) => (
        impl PropertyShow for $my_type {
            property_show_methods!($my_type, $e);
        });
    ($my_type:ty, $e:tt, $up:expr) => (
        impl PropertyShow for $my_type {
            property_show_methods!($my_type, $e);

            fn to_update(&self) -> ShouldUpdate
            {
                $up
            }
        }
        )
    );

property_show_impl!(vec::Vec3,[x,y,z]);
property_show_impl!(vec::Quat,[x,y,z,w]);
property_show_impl!(transform::Transform,[position,orientation]);
//property_show_impl!(mesh_render::MeshRender,[mesh,material]);
property_show_impl!(object::Object,
                     //[name,position,orientation,scale]);
                     [name,position,orientation,scale,comp_data,comp_lua]);

property_show_impl!(component::mesh_render::MeshRender,[mesh,material], ShouldUpdate::Mesh);
property_show_impl!(component::player::Player,[speed]);
property_show_impl!(component::player::Enemy,[name]);
property_show_impl!(component::player::Collider,[name]);
property_show_impl!(armature::ArmaturePath,[name]);

property_show_impl!(scene::Scene,[name,camera]);
property_show_impl!(camera::Camera,[data]);
property_show_impl!(camera::CameraData,[far,near]);

fn make_vec_from_str(s : &str) -> Vec<String>
{
    let v: Vec<&str> = s.split('/').collect();

    let mut vs = Vec::new();
    for i in &v
    {
        vs.push(i.to_string());
    }

    vs
}

pub fn find_property_show(p : &PropertyShow, path : Vec<String>) ->
Option<&PropertyShow>
{
    //let vs = make_vec_from_string(&field.to_owned());

    match path.len() {
        0 =>  None,
        1 => p.get_property(path[0].as_ref()),
        _ => {
             match p.get_property(path[0].as_ref()) {
                 Some(ppp) => {
                     find_property_show(ppp, path[1..].to_vec())
                 },
                 None => {
                     None
                 }
             }
        }
    }
}

pub extern fn panel_move(
    widget_cb_data : *const c_void,
    x : c_int, y : c_int, w : c_int, h : c_int)
{
    let wcb : & ui::WidgetCbData = unsafe {mem::transmute(widget_cb_data)};
    let mut p : &mut Property = unsafe {mem::transmute(wcb.widget)};

    p.config.x = x;
    p.config.y = y;
    p.config.w = w;
    p.config.h = h;
}


pub extern fn vec_add(
    data : *const c_void,
    name : *const c_char,
    old : *const c_void,
    new : *const c_void,
    action : c_int)
{
    let s = unsafe {CStr::from_ptr(name).to_bytes()};

    let path = match str::from_utf8(s) {
        Ok(pp) => pp,
        _ => {
            println!("problem with the path");
            return;}
    };

    let wcb : & ui::WidgetCbData = unsafe {mem::transmute(data)};
    //let p : &Property = unsafe {mem::transmute(wcb.widget)};
    let container : &mut Box<ui::WidgetContainer> = unsafe {mem::transmute(wcb.container)};

    let change = container.request_operation_vec_add(path);
    container.handle_change(&change, uuid::Uuid::nil());//p.id);
    //ui::add_empty(container, action.view_id);
}

pub extern fn vec_del(
    data : *const c_void,
    name : *const c_char,
    old : *const c_void,
    new : *const c_void,
    action : c_int)
{
    println!("TODO vec del");

    let s = unsafe {CStr::from_ptr(name).to_bytes()};

    let path = match str::from_utf8(s) {
        Ok(pp) => pp,
        _ => {
            println!("problem with the path");
            return;}
    };

    let wcb : & ui::WidgetCbData = unsafe {mem::transmute(data)};
    //let p : & Property = unsafe {mem::transmute(wcb.widget)};
    let container : &mut Box<ui::WidgetContainer> = unsafe {mem::transmute(wcb.container)};

    let change = container.request_operation_vec_del(path);
    container.handle_change(&change, uuid::Uuid::nil());//p.id);
    //ui::add_empty(container, action.view_id);
}


fn join_string(path : &Vec<String>) -> String
{
    let mut s = String::new();
    let mut first = true;
    for v in path {
        if !first {
            s.push('/');
        }
        s.push_str(v.as_ref());
        first = false;
    }

    s
}

impl PropertyId for object::Object
{
    fn get_id(&self) -> uuid::Uuid
    {
        return self.id
    }
}

impl PropertyId for scene::Scene
{
    fn get_id(&self) -> uuid::Uuid
    {
        return self.id
    }
}


pub trait PropertyWidget {

    fn add_simple_item(&self, field : &str, item : *const PropertyValue);
    fn add_node_t(&self, field : &str, item : *const PropertyValue);
    fn add_option(&self, field : &str, is_some : bool) -> *const PropertyValue;
    fn add_vec(&self, field : &str, len : usize);
    fn add_vec_item(&self, field : &str, widget_entry : *const PropertyValue, is_node : bool);

    fn update_option(&mut self, widget_entry : *const PropertyValue, is_some : bool);

    fn update_vec(&mut self, widget_entry : *const PropertyValue, len : usize);
    fn update_enum(&mut self, widget_entry : *const PropertyValue, value : &str);

}

impl PropertyWidget for Property
{
    fn add_simple_item(&self, field : &str, item : *const PropertyValue)
    {
        unsafe {
            property_list_single_item_add(
                self.jk_property_list,
                item);
        }

        self.pv.borrow_mut().insert(field.to_owned(), item);
    }

    fn add_node_t(&self, field : &str, item : *const PropertyValue)
    {
        unsafe {
            property_list_single_node_add(
                self.jk_property_list,
                item);
        }

        self.pv.borrow_mut().insert(field.to_owned(), item);

        if self.config.expand.contains(field) {
            unsafe {
                property_expand(item);
            }
        }
    }

    fn add_option(&self, field : &str, is_some : bool) -> *const PropertyValue
    {
        let f = CString::new(field.as_bytes()).unwrap();
        let type_value = match is_some {
            true => "Some",
            false => "None"
        };

        let v = CString::new(type_value.as_bytes()).unwrap();

        unsafe {
            let pv = property_list_option_add(
                self.jk_property_list,
                f.as_ptr(),
                v.as_ptr());

            if pv != ptr::null() {
                self.pv.borrow_mut().insert(field.to_owned(), pv);
            }

            pv
        }
    }

    fn add_vec(&self, name : &str, len : usize)
    {
        let f = CString::new(name.as_bytes()).unwrap();
        let pv = unsafe {
            property_list_vec_add(
                self.jk_property_list,
                f.as_ptr(),
                len as c_int
                )
        };

        if pv != ptr::null() {
            self.pv.borrow_mut().insert(name.to_owned(), pv);
        }

        if self.config.expand.contains(name) {
            unsafe {
                property_expand(pv);
            }
        }
    }

    fn add_vec_item(&self, field : &str, widget_entry : *const PropertyValue, is_node : bool)
    {
        unsafe {
            let pv = property_list_single_vec_add(
                self.jk_property_list,
                widget_entry,
                is_node
                );

            if pv != ptr::null() {
                self.pv.borrow_mut().insert(field.to_owned(), widget_entry);
            }

            if self.config.expand.contains(field) {
                property_expand(widget_entry);
            }
        }
    }

    fn update_option(&mut self, widget_entry : *const PropertyValue, is_some : bool)
    {
        let s = match is_some {
            true => "Some",
            false => "None"
        };

        let v = CString::new(s.as_bytes()).unwrap();
        unsafe {
            property_list_option_update(
                widget_entry,
                v.as_ptr());
        };
    }

    fn update_vec(&mut self, widget_entry : *const PropertyValue, len : usize)
    {
        unsafe { property_list_vec_update(widget_entry, len as c_int); }
        unsafe { property_expand(widget_entry); }
    }

    fn update_enum(&mut self, widget_entry : *const PropertyValue, value : &str)
    {
        let v = CString::new(value.as_bytes()).unwrap();
        unsafe {
            property_list_enum_update(widget_entry, v.as_ptr());
        }
    }
}
