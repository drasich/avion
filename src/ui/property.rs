use std::sync::{RwLock, Arc};
use std::collections::HashMap;
use libc::{c_char, c_void, c_int, c_float};
use std::str;
use std::mem;
//use std::collections::{LinkedList,Deque};
use std::collections::{LinkedList};
use std::ptr;
use std::rc::Rc;
use std::cell::{RefCell, BorrowState};
use std::rc::Weak;
use std::any::{Any};//, AnyRefExt};
use std::ffi::CString;
use std::ffi;
use std::ffi::CStr;
use core::marker;
use uuid;

use scene;
use object;
use ui::Window;
use ui;
use property;
use operation;
use control::Control;
use control::WidgetUpdate;
use vec;
use transform;
use resource;
use mesh;
use material;
use property::PropertyGet;
use component;
use component::CompData;
use armature;

#[repr(C)]
pub struct Elm_Object_Item;

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
    property : *const Property,
    object : *const c_void,
    parent : *const Elm_Object_Item);

#[repr(C)]
pub struct JkProperty;
#[repr(C)]
pub struct JkPropertySet;
#[repr(C)]
pub struct JkPropertyList;
#[repr(C)]
pub struct PropertyValue;

#[link(name = "png")]

#[link(name = "joker")]
extern {
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

    fn jk_property_set_new(window : *const Window) -> *const JkPropertySet;
    fn jk_property_set_data_set(set : *const JkPropertySet, data : *const c_void);

    fn jk_property_set_register_cb(
        property : *const JkPropertySet,
        data : *const Property,
        changed_float : ChangedFunc,
        changed_string : ChangedFunc,
        );

    fn property_set_string_add(
        ps : *const JkPropertySet,
        name : *const c_char,
        value : *const c_char
        );

    fn property_set_float_add(
        ps : *const JkPropertySet,
        name : *const c_char,
        value : c_float
        );

    fn property_set_node_add(
        ps : *const JkPropertySet,
        name : *const c_char
        );

    fn property_set_clear(
        ps : *const JkPropertySet);
        */

    fn jk_property_list_new(window : *const Window) -> *const JkPropertyList;

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
        contract : PropertyTreeFunc
        );

    fn property_list_group_add(
        pl : *const JkPropertyList,
        name : *const c_char
        );

    fn property_list_node_add(
        pl : *const JkPropertyList,
        name : *const c_char
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

}

pub struct Property
{
    pub name : String,
    pub jk_property_list : *const JkPropertyList,
    pub pv : HashMap<String, *const PropertyValue>,
    control : Rc<RefCell<Control>>,
    expand_state : HashMap<String, bool>,
    visible : bool,
    pub resource : Rc<resource::ResourceGroup>,
    id : uuid::Uuid
}

impl Property
{
    pub fn new(
        window : *const Window,
        control : Rc<RefCell<Control>>,
        resource : Rc<resource::ResourceGroup>
        //) -> Box<Property>
        ) -> Property
    {
        let mut p = Property {
            name : String::from("property_name"),
            jk_property_list : unsafe {jk_property_list_new(window)},
            pv : HashMap::new(),
            control : control,
            expand_state : HashMap::new(),
            visible: true,
            resource : resource,
            id : uuid::Uuid::new_v4()
        };

        p.set_visible(false);

        p
    }

    pub fn set_object(&mut self, o : &object::Object)
    {
        unsafe { property_list_clear(self.jk_property_list); }
        self.pv.clear();

        unsafe {
            property_list_group_add(
                self.jk_property_list,
                CString::new("object".as_bytes()).unwrap().as_ptr());
        }
        let mut v = Vec::new();
        v.push("object".to_string());
        o.create_widget(self, "object", 1);

        self.add_tools();
    }

    fn add_tools(&mut self)
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


    pub fn set_nothing(&mut self)
    {
        unsafe { property_list_clear(self.jk_property_list); }
        self.pv.clear();
    }

    pub fn data_set(&self, data : *const c_void)
    {
        //TODO
        //unsafe { property_data_set(self.jk_property, data); }
    }

    pub fn update_object_property(&self, object : &PropertyShow, prop : &str)
    {
        for (f,pv) in self.pv.iter() {
            let starts_with_prop = {
                let fstr : &str = f.as_ref();
                fstr.starts_with(prop)
            };

            if starts_with_prop {
                let yep = make_vec_from_string(f)[1..].to_vec();
                match find_property_show(object, yep.clone()) {
                    Some(ppp) => {
                        ppp.update_widget(*pv);
                    },
                    None => {}
                }
            }
        }
    }

    pub fn update_object(&self, object : &PropertyShow, but : &str)
    {
        for (f,pv) in self.pv.iter() {
            println!("UPDATEOBJECt contains property : Val : {}", f);
        }

        for (f,pv) in self.pv.iter() {
            let fstr : &str = f.as_ref();
            //if f.as_ref() as &str == but {
            if fstr == but {
                println!("buuuuuuuuuuuuuuuuuuuuuuuuuut: {} ", f);
                continue;
            }
            let yep = make_vec_from_string(f)[1..].to_vec();
            match find_property_show(object, yep.clone()) {
                Some(ppp) => {
                    println!("I find the property : {:?}", yep);
                    ppp.update_widget(*pv);
                },
                None => {
                    println!("could not find prop : {:?}", yep);
                }
            }
        }
    }

    pub fn add_node(&mut self, ps : &PropertyShow, name : &str) {
        println!("____added node : {}", name);
        let f = CString::new(name.as_bytes()).unwrap();
        let pv = unsafe {
            property_list_node_add(
                self.jk_property_list,
                f.as_ptr()
                )
        };

        let es = match self.expand_state.get(&name.to_string()) {
            Some(b) => *b,
            None => return,
        };

        if es {
            unsafe {
                property_expand(pv);
            }
        }
    }

    pub fn add_enum(
        &mut self,
        ps : &PropertyShow,
        path : &str,
        types : &str,
        value : &str
        )
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

        if pv != ptr::null() {
            self.pv.insert(path.to_string(), pv);
        }

        let es = match self.expand_state.get(&path.to_string()) {
            Some(b) => *b,
            None => return,
        };

        if es {
            unsafe {
                property_expand(pv);
            }
        }
    }

    pub fn set_visible(&mut self, b : bool)
    {
        self.visible = b;
        unsafe {
            property_show(self.jk_property_list, b);
        }
    }

    pub fn visible(&self) -> bool
    {
        self.visible
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
        Ok(sss) => sss.to_string(),
        _ => {
            return;
        }
    };
    println!("the string is {}", ss);
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
        Ok(sss) => sss.to_string(),
        _ => {
            println!("error");
            return;
        }
    };

    println!("register change string,,, the string is {}", ss);
    if action == 1 && old != ptr::null() {
        let oldchar = old as *const i8;
        let so = unsafe {CStr::from_ptr(oldchar).to_bytes()};
        let sso = match str::from_utf8(so) {
            Ok(ssso) => ssso.to_string(),
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
    property : *const c_void,
    name : *const c_char,
    old : *const c_void,
    new : *const c_void,
    action : c_int
    ) {

    let newchar = new as *const i8;
    let s = unsafe {CStr::from_ptr(newchar).to_bytes()};
    let ss = match str::from_utf8(s) {
        Ok(sss) => sss.to_string(),
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
            Ok(ssso) => ssso.to_string(),
            _ => {
                println!("error");
                return
            }
        };
        changed_set(property, name, Some(&sso), &ss, action);
    }
    else {
        changed_set(property, name, None, &ss, action);
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
        Ok(sss) => sss.to_string(),
        _ => {
            println!("error");
            return
        }
    };

    //println!("the string is {}", ss);
    if old == ptr::null() {
        println!("old is null, return");
        return;
    }

    let oldchar = old as *const i8;
    let so = unsafe {CStr::from_ptr(oldchar).to_bytes()};
    let sso = match str::from_utf8(so) {
        Ok(ssso) => ssso.to_string(),
        _ => {
            println!("error");
            return
        }
    };

    changed_option(widget_cb_data, name, &sso, &ss);
}


//fn changed_set(property : *const c_void, name : *const c_char, data : &Any) {
fn changed_set<T : Any+Clone+PartialEq>(
    widget_data : *const c_void,
    name : *const c_char,
    old : Option<&T>,
    new : &T,
    action : c_int
    ) {
    let s = unsafe {CStr::from_ptr(name).to_bytes()};

    let path = match str::from_utf8(s) {
        Ok(pp) => pp,
        _ => {
            println!("problem with the path");
            return;}
    };

    println!("I changed the value {} ", path);

    let v: Vec<&str> = path.split('/').collect();

    let mut vs = Vec::new();
    for i in v.iter()
    {
        vs.push(i.to_string());
    }

    //let vs = vs[1..].to_vec();

    let wcb : & ui::WidgetCbData = unsafe {mem::transmute(widget_data)};
    let p : &ui::Property = unsafe {mem::transmute(wcb.widget)};
    let container : &mut Box<ui::WidgetContainer> = unsafe {mem::transmute(wcb.container)};
    //let p : & Property = unsafe {mem::transmute(property)};
    let resource = &p.resource;

    let mut control = match p.control.borrow_state() {
        BorrowState::Unused => p.control.borrow_mut(),
        _ => { println!("cannot borrow control"); return; }
    };

    let change = match (old, action) {
        (Some(oldd), 1) => {
            println!(".....adding operation!!");
            container.request_operation_old_new(
                vs,
                box oldd.clone(),
                box new.clone())
        },
        _ => {
            container.request_direct_change(vs, new)
        }
    };

    container.handle_change(&change, p.id);
}

fn changed_option(
//fn changed_option<T : Any+Clone+PartialEq>(
    widget_cb_data : *const c_void,
    name : *const c_char,
    old : &str,
    new : &str
    ) {
    let s = unsafe {CStr::from_ptr(name).to_bytes()};

    let path = match str::from_utf8(s) {
        Ok(pp) => pp,
        _ => {
            println!("problem with the path");
            return;}
    };

    println!("OPTIONNNNNNNNNN I changed the value {} ", path);

    let v: Vec<&str> = path.split('/').collect();

    let mut vs = Vec::new();
    for i in v.iter()
    {
        vs.push(i.to_string());
    }

    //let vs = vs[1..].to_vec();

    //let p : & Property = unsafe {mem::transmute(property)};
    let wcb : & ui::WidgetCbData = unsafe {mem::transmute(widget_cb_data)};
    let p : &ui::Property = unsafe {mem::transmute(wcb.widget)};
    let container : &mut Box<ui::WidgetContainer> = unsafe {mem::transmute(wcb.container)};

    /*
    let mut control = match p.control.borrow_state() {
        BorrowState::Unused => p.control.borrow_mut(),
        _ => { println!("cannot borrow control"); return; }
    };
    */

    /*
    let (id, prop) = if let Some(o) = control.get_selected_object(){
        let p : Option<Box<Any>> = o.read().unwrap().get_property_hier(path);
        (o.read().unwrap().id.clone(),p)
    }
    else {
        return;
    };
    */

    let change = if new == "Some" {
        container.request_operation_option_to_some(vs)
    }
    else {
        container.request_operation_option_to_none(path)
        /*
        if let Some(propget) = prop {
                let test = *propget;
            control.request_operation_option_to_none(
                vs,
                box test.clone())
        }
        else {
            return
        }
        */
    };

    /*
    match change {
        operation::Change::DirectChange(s) |
        operation::Change::Objects(s, _) => {
            if s == "object/name" {
                match control.tree {
                    Some(ref t) =>
                        match t.borrow_state() {
                            BorrowState::Writing => {}
                            _ => {
                                t.borrow().update_object(&id);
                            },
                        },
                        None => {}
                };
            }
        },
        _ => {}
    }
    */

}



pub extern fn expand(
    property: *const Property,
    data : *const c_void,
    parent : *const Elm_Object_Item) -> ()
{
    let datachar = data as *const i8;
    let s = unsafe {CStr::from_ptr(datachar).to_bytes()};
    let wcb : & ui::WidgetCbData = unsafe {mem::transmute(property)};
    //let mut p : &mut Property = unsafe {mem::transmute(property)};
    let mut p : &mut Property = unsafe {mem::transmute(wcb.widget)};
    //let container : &Box<ui::WidgetContainer> = unsafe {mem::transmute(wcb.container)};

    let path = match str::from_utf8(s) {
        Ok(pp) => pp,
        _ => {
            println!("problem with the path");
            return;}
    };

    println!("I expand the value {} ", path);

    let vs = make_vec_from_string(&path.to_string());

    let yep = vs[1..].to_vec();
    println!("expand : {:?}", vs);

    let o = match p.control.clone().borrow_state() {
        BorrowState::Writing => {
            return;
        },
        _ => {
            match p.control.borrow().get_selected_object() {
                Some(ob) => ob,
                None => {
                    println!("no selected objectttttttttttttt");
                    return;
                }
            }
        }
    };

    let or  = o.read().unwrap();
    match find_property_show(&*or, yep.clone()) {
        Some(ppp) => {
            //p.create_entries(&*ppp, vs.clone());
            println!("I found and create {:?} ", vs);
            ppp.create_widget(p, path , 1);
            p.expand_state.insert(path.to_string(), true);
        },
        None => {
            println!("could not find property {:?} ", vs);
        }
    };
}

pub extern fn contract(
    property: *const Property,
    data : *const c_void,
    parent : *const Elm_Object_Item) -> ()
{
    //let mut p : &mut Property = unsafe {mem::transmute(property)};
    let wcb : & ui::WidgetCbData = unsafe {mem::transmute(property)};
    let mut p : &mut Property = unsafe {mem::transmute(wcb.widget)};
    //let container : &Box<ui::WidgetContainer> = unsafe {mem::transmute(wcb.container)};

    unsafe {
        property_list_nodes_remove(
            p.jk_property_list,
            data as *const c_char
            //s.unwrap(),
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

    println!("I contract the path {} ", path);

    p.expand_state.insert(path.to_string(), false);

    let vs = make_vec_from_string(&path.to_string());

    let yep = vs[1..].to_vec();
    println!("contract : {:?}", vs);

    let clone = p.pv.clone();

    for (key,pv) in clone.iter() {
        println!("cccccccccccccccccontract  start with key '{}' ", key);
        let starts_with_path = {
            let ks : &str = key.as_ref();
            ks.starts_with(path) && ks != path
        };

        //if key.as_ref().starts_with(path) && key.as_ref() != path  {
        if starts_with_path {
            println!("yes, '{}' starts with '{}'", key, path);
            match p.pv.remove(key) {
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

        let pv = match self.pv.get(&name.to_string()) {
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

pub trait PropertyShow
{
    fn create_widget(
        &self,
        property : &mut Property,
        field : &str,
        depth : i32
        );

    fn update_widget(&self, pv : *const PropertyValue) {
        println!("update_widget not implemented for this type");
    }

    fn get_property(&self, field : &str) -> Option<&PropertyShow>
    {
        return None;
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

    fn create_widget(&self, property : &mut Property, field : &str, depth : i32)
    {
        println!("adding field : {}", field);
        let f = CString::new(field.as_bytes()).unwrap();
        unsafe {
            let pv = property_list_float_add(
                property.jk_property_list,
                f.as_ptr(),
                *self as c_float);
            if pv != ptr::null() {
                property.pv.insert(field.to_string(), pv);
            }
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

    fn create_widget(&self, property : &mut Property, field : &str, depth : i32)
    {
        println!("adding string field : {}", field);
        let f = CString::new(field.as_bytes()).unwrap();
        let v = CString::new(self.as_bytes()).unwrap();

        unsafe {
            let pv = property_list_string_add(
                property.jk_property_list,
                f.as_ptr(),
                v.as_ptr());
            if pv != ptr::null() {
                property.pv.insert(field.to_string(), pv);
            }
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
        property : &mut Property,
        field : &str,
        depth : i32)
    {
        (**self).create_widget(property ,field, depth);
    }

    fn get_property(&self, field : &str) -> Option<&PropertyShow>
    {
        (**self).get_property(field)
    }
}

impl<T : PropertyShow> PropertyShow for Option<T> {

    fn create_widget(
        &self,
        property : &mut Property,
        field : &str,
        depth : i32)
    {
        if depth == 0 {
            let f = CString::new(field.as_bytes()).unwrap();
            let type_value = match *self {
                Some(_) => "Some",
                None => "None"
            };

            println!(".......type value : {}", type_value);
            let v = CString::new(type_value.as_bytes()).unwrap();

            unsafe {
                let pv = property_list_option_add(
                    property.jk_property_list,
                    f.as_ptr(),
                    v.as_ptr());

                if pv != ptr::null() {
                    println!("ADDING : {}", field);
                    property.pv.insert(field.to_string(), pv);
                }
            }
        }

        if depth == 1 {
            match *self {
                Some(ref s) =>  {
                    s.create_widget(property, field, depth);
                },
                None => {}
            };
        }
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
}

impl<T> PropertyShow for resource::ResTT<T>
{
    fn create_widget(
        &self,
        property : &mut Property,
        field : &str,
        depth : i32)
    {
        if depth < 0 {
            return;
        }

        if depth == 0 && field != ""
        {
            property.add_node(self, field);
        }

        if depth > 0 {
            let s = field.to_string() + "/name";
            self.name.create_widget(property, s.as_ref(), depth-1);
        }
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
        property : &mut Property,
        field : &str,
        depth : i32)
    {
        println!("___ add vec ::: field::::: {}, {}", field, depth);

        if depth < 0 {
            return;
        }

        if depth == 0 && field != ""
        {
            property.add_node(self, field);
        }

        if depth > 0 {
            for i in self.iter() {
                i.create_widget(property, field, depth -1);
            }
        }
    }

    fn get_property(&self, field : &str) -> Option<&PropertyShow>
    {
        for i in self.iter() {
            let r = i.get_property(field);
            if r.is_some() {
                println!("$$$$$$$$$$$$$$$ Vec return some : {}", field);
                return r;
            }
        }

        println!("$$$$$$$$$$$$$$$ Vec return none for field {}", field);
        None
    }

    fn update_widget(&self, pv : *const PropertyValue) {
        for i in self.iter() {
            i.update_widget(pv);
        }
    }
}

impl PropertyShow for CompData
{
    fn create_widget(
        &self,
        property : &mut Property,
        field : &str,
        depth : i32)
    {
        let kind : String = self.get_kind_string();
        let kindr : &str = kind.as_ref();
        let ss = field.to_string() + "/" + kindr;
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
            return;
        }

        if depth == 0 && field != ""
        {
            println!("--> compdata property show for : {}, {}, {}", s, depth, kind );
            property.add_node(self, s);
        }

        if depth > 0
        {
            println!("--> compdata property show for : {}, {}, {}", field, depth, kind );

        match *self {
            CompData::Player(ref p) => {
                p.create_widget(property, field, depth);
            },
            CompData::Armature(ref p) => {
                p.create_widget(property, field, depth);
            },
            CompData::MeshRender(ref p) => {
                p.create_widget(property, field, depth);
            },
            _ => {println!("not yet implemented");}
        }
    }
    }

    fn update_widget(&self, pv : *const PropertyValue) {
        match *self {
            CompData::Player(ref p) => {
                p.update_widget(pv);
            },
            CompData::Armature(ref p) => {
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
            CompData::Armature(ref p) => {
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
}



macro_rules! property_show_impl(
    ($my_type:ty, [ $($member:ident),+ ]) => (

        impl PropertyShow for $my_type
        {
            fn create_widget(
                &self,
                property : &mut Property,
                field : &str,
                depth : i32)
            {
                if depth < 0 {
                    return;
                }

                if depth == 0 && field != ""
                {
                    property.add_node(self, field);
                }

                if depth > 0 {
                $(
                    let s = field.to_string()
                    + "/"//.to_string()
                    + stringify!($member);//.to_string();
                    self.$member.create_widget(property, s.as_ref(), depth-1);
                 )+
                }
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
        }
    )
);

property_show_impl!(vec::Vec3,[x,y,z]);
property_show_impl!(vec::Quat,[x,y,z,w]);
property_show_impl!(transform::Transform,[position,orientation]);
//property_show_impl!(mesh_render::MeshRender,[mesh,material]);
property_show_impl!(object::Object,
                     //[name,position,orientation,scale]);
                     [name,position,orientation,scale,comp_data]);

property_show_impl!(component::mesh_render::MeshRender,[mesh,material]);
property_show_impl!(component::player::Player,[speed]);
property_show_impl!(component::player::Enemy,[name]);
property_show_impl!(component::player::Collider,[name]);
property_show_impl!(armature::ArmaturePath,[name]);

fn make_vec_from_string(s : &String) -> Vec<String>
{
    let v: Vec<&str> = s.split('/').collect();

    let mut vs = Vec::new();
    for i in v.iter()
    {
        vs.push(i.to_string());
    }

    vs
}

pub fn find_property_show(p : &PropertyShow, path : Vec<String>) ->
Option<&PropertyShow>
{
    //let vs = make_vec_from_string(&field.to_string());

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
