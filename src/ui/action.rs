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

#[link(name = "joker")]
extern {
    fn window_action_new(window : *const Window) -> *const JkAction;
}

pub struct Action
{
    pub name : String,
    pub jk_action : *const JkAction,
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

        a
    }

}


