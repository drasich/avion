use std::sync::{RwLock, Arc};
use std::collections::HashMap;
use libc::{c_char, c_void, c_int};
use std::mem;
use std::collections::{LinkedList};//,Deque};
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
pub struct JkCommand;

pub type CommandCallback = extern fn(
    data : *const c_void);

#[link(name = "joker")]
extern {
    fn window_command_new(window : *const Window) -> *const JkCommand;
    fn command_new(
        command : *const JkCommand,
        name : *const c_char,
        data : *const c_void,
        button_callback : CommandCallback);
    fn command_show(
        command : *const JkCommand);
}

pub struct Command
{
    name : String,
    jk_command : *const JkCommand,
}

#[derive(Clone)]
pub struct CommandData
{
    tree : Rc<RefCell<Box<ui::Tree>>>,
    property : Rc<RefCell<Box<ui::Property>>>,
    control : Rc<RefCell<Control>>,
    holder : Rc<RefCell<ui::view::Holder>>
}

impl CommandData
{
    pub fn new(
    tree : Rc<RefCell<Box<ui::Tree>>>,
    property : Rc<RefCell<Box<ui::Property>>>,
    control : Rc<RefCell<Control>>,
    holder : Rc<RefCell<ui::view::Holder>>,
    ) -> CommandData
    {
        CommandData {
            tree : tree,
            property : property,
            control : control,
            holder : holder
        }
    }
}

impl Command
{
    pub fn new(
        window : *const Window)
        -> Box<Command>
    {
        let c = box Command {
            name : String::from_str("command_name"),
            jk_command : unsafe {window_command_new(window)},
        };

        c
    }

    pub fn show(&self)
    {
        unsafe { command_show(self.jk_command); }
    }

    pub fn add(&self, name : &str, cb : CommandCallback, data : CommandData)
    {
        unsafe {
            command_new(
                self.jk_command,
                CString::new(name.as_bytes()).unwrap().as_ptr(),
                mem::transmute(box data),
                cb);
        }
    }

    /*
    pub fn add_button_ptr(
        &self,
        name : &str,
        cb : CommandCallback, data : *const c_void)
    {
        unsafe {
            action_button_new(
                self.jk_action,
                CString::new(name.as_bytes()).unwrap().as_ptr(),
                data,
                cb);
        }
    }
    */
}

pub extern fn add_empty(data : *const c_void)
{
    println!("command ::: add empty");
}


