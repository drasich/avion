use sync::{RWLock, Arc};
use std::collections::HashMap;
use libc::{c_char, c_void, c_int, c_float};
use std::mem;
use std::c_str::CString;
//use std::collections::{DList,Deque};
use std::collections::{DList};
use std::ptr;
use std::rc::Rc;
use std::cell::RefCell;
use std::rc::Weak;
use std::any::{Any, AnyRefExt};

use scene;
use object;
use ui::Window;
use ui::Master;
use ui;
use property;
use property::TProperty;
use property::ChrisProperty;
use operation;
use control::Control;
use control::WidgetUpdate;

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

pub type PropertyTreeExpandFunc = extern fn(
    property : *const Property,
    object : *const c_void,
    parent : *const Elm_Object_Item);

pub type data_cast_fn = extern fn(
    name : *const c_char,
    data : *const c_void);

#[repr(C)]
pub struct JkProperty;
#[repr(C)]
pub struct JkPropertySet;
#[repr(C)]
pub struct JkPropertyList;
#[repr(C)]
pub struct PropertyValue;


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

    fn jk_property_list_register_cb(
        property : *const JkPropertyList,
        data : *const Property,
        changed_float : ChangedFunc,
        changed_string : ChangedFunc,
        register_change_string : RegisterChangeFunc,
        register_change_float : RegisterChangeFunc,
        expand : PropertyTreeExpandFunc
        );

    fn property_list_group_add(
        pl : *const JkPropertyList,
        name : *const c_char
        );

    fn property_list_node_add(
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

    fn property_list_string_update(
        pv : *const PropertyValue,
        value : *const c_char);

    fn property_list_float_update(
        pv : *const PropertyValue,
        value : c_float);
}

pub struct Property
{
    pub name : String,
    jk_property_list : *const JkPropertyList,
    master : Weak<RefCell<ui::Master>>,
    pv : HashMap<String, *const PropertyValue>,
    control : Rc<RefCell<Control>>
}

impl Property
{
    pub fn new(
        window : *const Window,
        master : Weak<RefCell<ui::Master>>,
        control : Rc<RefCell<Control>>
        ) -> Box<Property>
    {
        let p = box Property {
            name : String::from_str("property_name"),
            jk_property_list : unsafe {jk_property_list_new(window)},
            master : master,
            pv : HashMap::new(),
            control : control
        };

        unsafe {
            jk_property_list_register_cb(
                p.jk_property_list,
                &*p, //ptr::null(),
                changed_set_float,
                changed_set_string,
                register_change_string,
                register_change_float,
                expand
                ); 
        }

        p
    }

    pub fn set_object(&mut self, o : &object::Object)
    {
        unsafe { property_list_clear(self.jk_property_list); }

        unsafe {
            property_list_group_add(
                self.jk_property_list,
                "object".to_c_str().unwrap());
        }
        let mut v = Vec::new();
        v.push("object".to_string());
        self.create_entries(o, v);//Vec::new());
    }

    pub fn create_entries(
        &mut self,
        o : &ChrisProperty,
        path : Vec<String>)
    {
        self.pv.clear();

        fn get_node_path(path : &Vec<String>) -> String
        {
            let mut s = String::new();
            let mut first = true;
            for v in path.iter() {
                if !first {
                    s.push('/');
                }
                s.push_str(v.as_slice());
                first = false;
            }

            s
        }

        fn create_node_path(path : &Vec<String>, field : &str) -> String
        {
            let mut s = get_node_path(path);
            if !path.is_empty(){
                s.push('/');
            }
            s.push_str(field);

            s
        }

        //TODO this function for jk_property_list
            println!("entries!!! {}", path);
            for field in o.cfields().iter()
            {
                match o.cget_property(field.as_slice()) {
                    property::BoxAny(p) => {
                        match p.downcast_ref::<String>() {
                            Some(s) => {
                                let field = create_node_path(
                                    &path,
                                    field.as_slice());
                                let v = s.to_c_str();
                                let f = field.to_c_str();
                                unsafe {
                                    let pv = property_list_string_add(
                                        self.jk_property_list,
                                        f.unwrap(),
                                        v.unwrap());
                                    if pv != ptr::null() {
                                        self.pv.insert(field, pv);
                                    }
                                }
                            },
                            None => {}
                        }
                        match p.downcast_ref::<f64>() {
                            Some(v) => {
                                let field = create_node_path(
                                    &path,
                                    field.as_slice());
                                let f = field.to_c_str();
                                println!("field : {}", field);
                                unsafe {
                                    let pv = property_list_float_add(
                                        self.jk_property_list,
                                        f.unwrap(),
                                        *v as c_float);
                                    if pv != ptr::null() {
                                        self.pv.insert(field, pv);
                                    }
                                }
                            },
                            None => {}
                        }
                        //TODO other type
                    },
                    property::BoxChrisProperty(p) => {
                        let field = create_node_path(&path, field.as_slice());
                        println!("I come here and field is : {}", field);
                        let f = field.to_c_str();
                        unsafe {
                            property_list_node_add(
                                self.jk_property_list,
                                f.unwrap());
                        }
                        /*
                        let mut yep = path.clone();
                        yep.push(field.clone());
                        self.create_entries(&*p, yep);
                        */
                    },
                    property::ChrisNone => {}
                }

            }
        }


    pub fn data_set(&self, data : *const c_void)
    {
        //TODO
        //unsafe { property_data_set(self.jk_property, data); }
    }
}

pub extern fn name_get(data : *const c_void) -> *const c_char {

    let o : &Arc<RWLock<object::Object>> = unsafe {
        mem::transmute(data)
    };

    //println!("name get {:?}", o);

    let cs = o.read().name.to_c_str();

    unsafe {
        cs.unwrap()
    }
}


pub extern fn changed(object : *const c_void, data : *const c_void) {
    let o : &Arc<RWLock<object::Object>> = unsafe {
        mem::transmute(object)
    };

    let s = unsafe {CString::new(data as *const i8, false) };

    match s.as_str() {
        Some(ss) => {
            let sss = property::SString(String::from_str(ss));
            o.clone().write().set_property("name", &sss);
        },
        _ => ()
    }
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

    let s = unsafe {CString::new(data as *const i8, false) };
    let ss = match s.as_str() {
        Some(sss) => sss.to_string(),
        None => return
    };
    //println!("the string is {}", ss);
    changed_set(property, name, None, &ss, 0);
}

pub extern fn register_change_string(
    property : *const c_void,
    name : *const c_char,
    old : *const c_void,
    new : *const c_void,
    action : c_int
    ) {

    let s = unsafe {CString::new(new as *const i8, false) };
    let ss = match s.as_str() {
        Some(sss) => sss.to_string(),
        None => return
    };

    //println!("the string is {}", ss);
    if action == 1 && old != ptr::null() {
        let so = unsafe {CString::new(old as *const i8, false) };
        let sso = match so.as_str() {
            Some(ssso) => ssso.to_string(),
            None => return
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

//fn changed_set(property : *const c_void, name : *const c_char, data : &Any) {
fn changed_set<T : Any+Clone+PartialEq>(
    property : *const c_void,
    name : *const c_char,
    old : Option<&T>,
    new : &T,
    action : c_int
    ) {
    let s = unsafe {CString::new(name as *const i8, false) };
    println!("I changed the value {} ", s);

    let path = match s.as_str() {
        Some(pp) => pp,
        None => {
            println!("problem with the path");
            return;}
    };

    let v: Vec<&str> = path.split('/').collect();

    let mut vs = Vec::new();
    for i in v.iter()
    {
        vs.push(i.to_string());
    }

    //let vs = vs.tail().to_vec();

    let p : & Property = unsafe {mem::transmute(property)};

    let mut control = match p.control.try_borrow_mut() {
        Some(c) => c,
        None => { println!("cannot borrow control"); return; }
    };

    match (old, action) {
        (Some(oldd), 1) => {
            control.request_operation(
                vs,
                box oldd.clone(),
                box new.clone());
            println!(".....adding operation!!");
        },
        _ => {
            control.request_direct_change(vs, new);
        }
    };

    //TODO remove and put in control
    // add widget origin uuid in request_operation and request_direct_change
    match p.master.upgrade() {
        Some(m) => { 
            match m.try_borrow_mut() {
                Some(ref mut mm) => {
                    match mm.tree {
                        Some(ref t) => { 
                            t.borrow_mut().update();
                            println!("todo call this only when the name or object representation has changed...");
                            println!("... or do the object references in ui stuff /see todo file");
                        },
                        None => {}
                    }
                },
                _ => { println!("already borrowed : mouse_up add_ob ->sel ->add_ob")}
            }
        },
        None => { println!("the master of the property doesn't exist anymore");}
    }
}


extern fn expand(
    property: *const Property,
    data : *const c_void,
    parent : *const Elm_Object_Item) -> ()
{
    let s = unsafe {CString::new(data as *const i8, false) };
    let mut p : &mut Property = unsafe {mem::transmute(property)};

    println!("expanding ! property name {} ", p.name);

    let s = unsafe {CString::new(data as *const i8, false) };
    println!("I changed the value {} ", s);

    let path = match s.as_str() {
        Some(pp) => pp,
        None => {
            println!("problem with the path");
            return;}
    };

    let v: Vec<&str> = path.split('/').collect();

    let mut vs = Vec::new();
    for i in v.iter()
    {
        vs.push(i.to_string());
        println!("pushing {}", i);
    }

    //let yep = vs.tail().to_vec();
    println!("expand : {}", vs);

    match p.control.clone().try_borrow() {
        Some(c) => {
            c.request_display_property(
                p,
                vs);
        },
        None => return
    };
}

impl WidgetUpdate for Property
{
    fn update_changed(
        &mut self,
        name : &str,
        new : &Any)
    {
        println!("property update changed {}", name);

        let pv = match self.pv.find(&name.to_string()) {
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
                let v = s.to_c_str();
                unsafe {
                    property_list_string_update(
                        *pv,
                        v.unwrap());
                };
                return;
            },
            None => {
                println!("cannot downcast to string");
            }
        }

    }
}

