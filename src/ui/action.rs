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
    pub name : String,
    pub jk_action : *const JkAction,
}

pub struct ActionData
{
    pub tree : Rc<RefCell<Box<ui::Tree>>>,
    pub property : Rc<RefCell<Box<ui::Property>>>,
    pub control : Rc<RefCell<Control>>,
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

        a.add_button("Add empty", add_empty);

        a
    }

    fn add_button(&self, name : &str, cb : ButtonCallback)
    {
        unsafe {
            action_button_new(
                self.jk_action,
                CString::from_slice(name.as_bytes()).as_ptr(),
                ptr::null(),
                cb);
        }
    }
}

extern fn add_empty(data : *const c_void)
{
    println!("add empty____________");
}


