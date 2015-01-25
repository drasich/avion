use std::sync::{RwLock, Arc};
use std::collections::HashMap;
use libc::{c_char, c_void, c_int, c_float};
use std::str;
use std::mem;
//use std::collections::{DList,Deque};
use std::collections::{DList};
use std::ptr;
use std::rc::Rc;
use std::cell::RefCell;
use std::rc::Weak;
use std::any::{Any};//, AnyRefExt};
use std::ffi::CString;
use std::ffi;

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
        changed_enum : ChangedFunc,
        register_change_string : RegisterChangeFunc,
        register_change_float : RegisterChangeFunc,
        register_change_enum : RegisterChangeFunc,
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
        );

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
    pub jk_property_list : *const JkPropertyList,
    pub pv : HashMap<String, *const PropertyValue>,
    control : Rc<RefCell<Control>>
}

impl Property
{
    pub fn new(
        window : *const Window,
        control : Rc<RefCell<Control>>
        ) -> Box<Property>
    {
        let p = box Property {
            name : String::from_str("property_name"),
            jk_property_list : unsafe {jk_property_list_new(window)},
            pv : HashMap::new(),
            control : control
        };

        unsafe {
            jk_property_list_register_cb(
                p.jk_property_list,
                //mem::transmute(box p.control.clone()),
                &*p, //ptr::null(),
                changed_set_float,
                changed_set_string,
                changed_set_enum,
                register_change_string,
                register_change_float,
                register_change_enum,
                expand,
                contract
                ); 
        }

        p
    }

    pub fn set_object(&mut self, o : &object::Object)
    {
        unsafe { property_list_clear(self.jk_property_list); }
        self.pv.clear();

        unsafe {
            property_list_group_add(
                self.jk_property_list,
                CString::from_slice("object".as_bytes()).as_ptr());
        }
        let mut v = Vec::new();
        v.push("object".to_string());
        o.create_widget(self, "object", 1);
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

    pub fn update_object(&mut self, object : &PropertyShow, but : &str)
    {
        for (f,pv) in self.pv.iter() {
            if f.as_slice() == but {
                continue;
            }
            let yep = make_vec_from_string(f).tail().to_vec();
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
}

pub extern fn name_get(data : *const c_void) -> *const c_char {

    let o : &Arc<RwLock<object::Object>> = unsafe {
        mem::transmute(data)
    };

    let cs = CString::from_slice(o.read().unwrap().name.as_bytes());
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
    let s = unsafe {ffi::c_str_to_bytes(&datachar)};
    let ss = match str::from_utf8(s) {
        Ok(sss) => sss.to_string(),
        _ => {
            return;
        }
    };
    //println!("the string is {}", ss);
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
    let s = unsafe {ffi::c_str_to_bytes(&newchar)};
    let ss = match str::from_utf8(s) {
        Ok(sss) => sss.to_string(),
        _ => {
            println!("error");
            return;
        }
    };

    //println!("the string is {}", ss);
    if action == 1 && old != ptr::null() {
        let oldchar = new as *const i8;
        let so = unsafe {ffi::c_str_to_bytes(&oldchar)};
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
    let s = unsafe {ffi::c_str_to_bytes(&newchar)};
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
        let so = unsafe {ffi::c_str_to_bytes(&oldchar)};
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

//fn changed_set(property : *const c_void, name : *const c_char, data : &Any) {
fn changed_set<T : Any+Clone+PartialEq>(
    property : *const c_void,
    name : *const c_char,
    old : Option<&T>,
    new : &T,
    action : c_int
    ) {
    let s = unsafe {ffi::c_str_to_bytes(&name)};

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

    //let vs = vs.tail().to_vec();

    let p : & Property = unsafe {mem::transmute(property)};
    //let control_rc : &Rc<RefCell<Control>>  = unsafe {mem::transmute(control_void)};

    let mut control = match p.control.try_borrow_mut() {
    //let mut control = match control_rc.try_borrow_mut() {
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
}


extern fn expand(
    property: *const Property,
    data : *const c_void,
    parent : *const Elm_Object_Item) -> ()
{
    let datachar = data as *const i8;
    let s = unsafe {ffi::c_str_to_bytes(&datachar)};
    let mut p : &mut Property = unsafe {mem::transmute(property)};

    //println!("expanding ! property name {} ", p.name);

    let path = match str::from_utf8(s) {
        Ok(pp) => pp,
        _ => {
            println!("problem with the path");
            return;}
    };

    println!("I expand the value {} ", path);

    let vs = make_vec_from_string(&path.to_string());

    let yep = vs.tail().to_vec();
    println!("expand : {:?}", vs);

    match p.control.clone().try_borrow() {
    //match control_rc.clone().try_borrow() {
        Some(c) => {

            let o = match c.get_selected_object() {
                Some(ob) => ob,
                None => return
            };

            //match property::find_property(&*o.read(), yep.clone()) {
            match find_property_show(&*o.read().unwrap(), yep.clone()) {
                Some(ppp) => {
                    //p.create_entries(&*ppp, vs.clone());
                    ppp.create_widget(p, path , 1);
                },
                None => {
                    println!("could not find property {:?} ", vs);
                }
            }
        },
        None => return
    };

}

extern fn contract(
    property: *const Property,
    data : *const c_void,
    parent : *const Elm_Object_Item) -> ()
{
    let mut p : &mut Property = unsafe {mem::transmute(property)};

    unsafe {
        property_list_nodes_remove(
            p.jk_property_list,
            data as *const c_char
            //s.unwrap(),
            );
    };

    let datachar = data as *const i8;
    let s = unsafe {ffi::c_str_to_bytes(&datachar)};
    let path = match str::from_utf8(s) {
        Ok(pp) => pp,
        _ => {
            println!("problem with the path");
            return;}
    };

    println!("I contract the path {} ", path);

    let vs = make_vec_from_string(&path.to_string());

    let yep = vs.tail().to_vec();
    println!("contract : {:?}", vs);

    let clone = p.pv.clone();

    for (key,pv) in clone.iter() {
        println!("start with key '{}' ", key);
        if key.as_slice().starts_with(path) {
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
                let v = CString::from_slice(s.as_bytes());
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
        let f = CString::from_slice(field.as_bytes());
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
        let f = CString::from_slice(field.as_bytes());
        let v = CString::from_slice(self.as_bytes());

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
}


/*
impl PropertyShow for vec::Quat {

    fn create_widget(&self, property : &mut Property, field : &str)
    {
        let s = field.to_string() + "/x".to_string();
        self.x.create_widget(property, s.as_slice());

        let s = field.to_string() + "/y".to_string();
        self.y.create_widget(property, s.as_slice());

        let s = field.to_string() + "/z".to_string();
        self.z.create_widget(property, s.as_slice());

        let s = field.to_string() + "/w".to_string();
        self.w.create_widget(property, s.as_slice());
    }
}
*/

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

pub macro_rules! property_show_impl(
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
                    let f = CString::from_slice(field.as_bytes());
                    unsafe {
                        property_list_node_add(
                            property.jk_property_list,
                            f.as_ptr());
                    }
                }

                if depth > 0 {
                $(
                    let s = field.to_string()
                    + "/"//.to_string()
                    + stringify!($member);//.to_string();
                    self.$member.create_widget(property, s.as_slice(), depth-1);
                 )+
                }
            }

            /*
            fn get_children(&self) -> Option<DList<&PropertyShow>>
            {
                let mut list = DList::new();
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

property_show_impl!(vec::Vec3,
                     [x,y,z]);

property_show_impl!(vec::Quat,[x,y,z,w]);

property_show_impl!(transform::Transform,[position,orientation]);

property_show_impl!(object::Object,
                     [name,position,orientation,scale]);

fn join_string(path : &Vec<String>) -> String
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
        1 => p.get_property(path[0].as_slice()),
        _ => { 
             match p.get_property(path[0].as_slice()) {
                 Some(ppp) => {
                     find_property_show(ppp, path.tail().to_vec())
                 },
                 None => {
                     None
                 }
             }
        }
    }
}
