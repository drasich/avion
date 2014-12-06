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
use ui;
use property;
use property::ChrisProperty;
use operation;
use control::Control;
use control::WidgetUpdate;
use vec;

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
    pv : HashMap<String, *const PropertyValue>,
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
        self.pv.clear();

        unsafe {
            property_list_group_add(
                self.jk_property_list,
                "object".to_c_str().unwrap());
        }
        let mut v = Vec::new();
        v.push("object".to_string());
        //self.create_entries(o, v);//Vec::new());
        o.create_widget(self, "object");
    }

    pub fn create_entries(
        &mut self,
        o : &ChrisProperty,
        path : Vec<String>)
    {
        //o.create_entries(self, path);
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
            for field in o.fields().iter()
            {
                match o.get_property(field.as_slice()) {
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

    pub fn update_object(&mut self, object : &ChrisProperty, but : &str)
    {
        let path = vec!("object".to_string());
        //let path = vec!();

        fn _update_obj_rec(
            prop :&mut Property,
            object : &ChrisProperty,
            child : &ChrisProperty,
            path : &Vec<String>,
            but : &str
            )
        {
            for f in child.fields().iter() {
                let mut yep = path.clone();
                yep.push(f.clone());
                //println!("try to find : {}", yep);
                match object.get_property_hier(yep.tail().to_vec()) {
                    property::BoxAny(a) => {
                        let s = join_string(&yep);
                        if s.as_slice() != but{
                        prop.update_changed(
                            s.as_slice(),
                            &*a);
                        }
                    },
                    property::BoxChrisProperty(cp) => {
                        //_update_obj_rec(prop, &*cp, &yep);
                        _update_obj_rec(prop, object, &*cp, &yep, but);
                    }
                    property::ChrisNone => {
                        println!("its none : {}", yep);
                    }
                }
            }
        }

        println!("updating object.......");
        _update_obj_rec(self, object, object, &path, but);
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
    let s = unsafe {CString::new(data as *const i8, false) };
    let mut p : &mut Property = unsafe {mem::transmute(property)};

    //println!("expanding ! property name {} ", p.name);

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

    let yep = vs.tail().to_vec();
    println!("expand : {}", vs);

    match p.control.clone().try_borrow() {
    //match control_rc.clone().try_borrow() {
        Some(c) => {

            let o = match c.get_selected_object() {
                Some(ob) => ob,
                None => return
            };

            match property::find_property(&*o.read(), yep.clone()) {
                Some(ppp) => {
                    p.create_entries(&*ppp, vs.clone());
                },
                None => {
                    println!("could not find property {} ", vs);
                }
            }
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

pub trait PropertyShow
{
    fn create_widget(
        &self,
        property : &mut Property,
        field : &str);
    /*
    {
        for v in self.fields().iter()
        {
            println!("__________normal chris_______");
        }
    }
    */
}

/*

impl<T : ChrisProperty> PropertyShow for T {

    fn create_entries(
        &self,
        property:&mut Property,
        path : Vec<String>)
    {
    }

}
*/

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

    fn create_widget(&self, property : &mut Property, field : &str)
    {
        println!("adding field : {}", field);
        let f = field.to_c_str();
        unsafe {
            let pv = property_list_float_add(
                property.jk_property_list,
                f.unwrap(),
                *self as c_float);
            if pv != ptr::null() {
                property.pv.insert(field.to_string(), pv);
            }
        }
    }
}

impl PropertyShow for String {

    fn create_widget(&self, property : &mut Property, field : &str)
    {
        let f = field.to_c_str();
        let v = self.to_c_str();

        unsafe {
            let pv = property_list_string_add(
                property.jk_property_list,
                f.unwrap(),
                v.unwrap());
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

pub macro_rules! property_show_impl(
    ($my_type:ty, [ $($member:ident),+ ]) => ( 

        impl PropertyShow for $my_type
        {
            fn create_widget(&self, property : &mut Property, field : &str)
            {
                let f = field.to_c_str();
                unsafe {
                    property_list_node_add(
                        property.jk_property_list,
                        f.unwrap());
                }

                $(
                    let s = field.to_string()
                    + "/".to_string()
                    + stringify!($member).to_string();
                    self.$member.create_widget(property, s.as_slice());
                 )+
            }
        }
    )
)

property_show_impl!(vec::Vec3,
                     [x,y,z])

property_show_impl!(vec::Quat,[x,y,z,w])

property_show_impl!(object::Object,
                     [name,position,orientation])



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

