use std::sync::{RwLock, Arc};
use std::collections::HashMap;
use libc::{c_char, c_void, c_int};
use std::mem;
use std::collections::{DList};//,Deque};
use std::ptr;
use std::cell::{RefCell, BorrowState};
use std::rc::Weak;
use std::rc::Rc;
use uuid::Uuid;
use std::ffi::CString;

use scene;
use object;
use ui::Window;
use ui::Master;
use ui;
use control::Control;

#[repr(C)]
pub struct JkAction;

pub type ButtonCallback = extern fn(
    data : *const c_void);

#[link(name = "joker")]
extern {
    fn window_action_new(window : *const Window) -> *const JkAction;
    fn action_button_new(
        action : *const JkAction,
        name : *const c_char,
        data : *const c_void,
        button_callback : ButtonCallback);
}

pub struct Action
{
    name : String,
    jk_action : *const JkAction,
}

pub struct ActionData
{
    tree : Rc<RefCell<Box<ui::Tree>>>,
    property : Rc<RefCell<Box<ui::Property>>>,
    control : Rc<RefCell<Control>>,
}

impl ActionData
{
    pub fn new(
    tree : Rc<RefCell<Box<ui::Tree>>>,
    property : Rc<RefCell<Box<ui::Property>>>,
    control : Rc<RefCell<Control>>) -> ActionData
    {
        ActionData {
            tree : tree,
            property : property,
            control : control
        }
    }
}

impl Action
{
    pub fn new(
        window : *const Window)
        -> Box<Action>
    {
        let a = box Action {
            name : String::from_str("action_name"),
            jk_action : unsafe {window_action_new(window)},
        };

        //a.add_button("Add empty", add_empty);

        a
    }

    pub fn add_button(&self, name : &str, cb : ButtonCallback, data : ActionData)
    {
        unsafe {
            action_button_new(
                self.jk_action,
                CString::from_slice(name.as_bytes()).as_ptr(),
                mem::transmute(box data),
                cb);
        }
    }
}

pub extern fn add_empty(data : *const c_void)
{
    let ad : &ActionData = unsafe {mem::transmute(data)};

    if ad.control.borrow_state() == BorrowState::Writing {
        println!("control already borrowed ");
        return;
    }

    let mut control = ad.control.borrow_mut();
    let o = control.add_empty("new object");

    match ad.property.borrow_state() {
        BorrowState::Unused => {
            ad.property.borrow_mut().set_object(&*o.read().unwrap());
        },
        _ => {println!("cannot borrow property");}
    };

    match ad.tree.borrow_state() {
        BorrowState::Unused => {
            let mut t = ad.tree.borrow_mut();
            t.add_object(o.clone());
            t.select(&o.read().unwrap().id);
        }
        _ => {}
    }
}


