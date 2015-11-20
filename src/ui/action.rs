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
use resource;
use uuid;

#[repr(C)]
pub struct JkAction;
pub struct JkLabel;

pub type ButtonCallback = extern fn(
    data : *const c_void);

#[link(name = "joker")]
extern {
    fn window_action_new(window : *const Window) -> *const JkAction;
    fn window_action_new_up(window : *const Window) -> *const JkAction;
    fn action_button_new(
        action : *const JkAction,
        name : *const c_char,
        data : *const c_void,
        button_callback : ButtonCallback);
    fn action_label_new(
        action : *const JkAction,
        name : *const c_char) -> *const JkLabel;

    fn jk_label_set(
        label : *const JkLabel,
        name : *const c_char);

    fn action_show(
        action : *const JkAction,
        b : bool);
}

pub struct Action
{
    name : String,
    jk_action : *const JkAction,
    visible : bool,
    view_id : uuid::Uuid
}

pub enum Position
{
    Top,
    Bottom
}

impl Action
{
    pub fn new(
        window : *const Window,
        pos : Position,
        view_id : uuid::Uuid)
        -> Action
    {
        Action {
            name : String::from("action_name"),
            jk_action : match pos {
                Position::Bottom => unsafe {window_action_new(window)},
                Position::Top => unsafe {window_action_new_up(window)}
            },
            visible : true,
            view_id : view_id
        }
    }

    //pub fn add_button(&self, name : &str, cb : ButtonCallback, data : ActionData)
    pub fn add_button(&self, name : &str, cb : ButtonCallback, data : ui::WidgetCbData)
    {
        unsafe {
            action_button_new(
                self.jk_action,
                CString::new(name.as_bytes()).unwrap().as_ptr(),
                mem::transmute(box data),
                cb);
        }
    }

    pub fn add_button_ptr(
        &self,
        name : &str,
        cb : ButtonCallback, data : *const c_void)
    {
        unsafe {
            action_button_new(
                self.jk_action,
                CString::new(name.as_bytes()).unwrap().as_ptr(),
                data,
                cb);
        }
    }

    pub fn add_label(&self, name : &str)
    {
        unsafe {
            action_label_new(
                self.jk_action,
                CString::new(name.as_bytes()).unwrap().as_ptr());
        }
    }


    pub fn set_visible(&mut self, b : bool)
    {
        self.visible = b;
        unsafe {
            action_show(self.jk_action, b);
        }
    }

    pub fn visible(&self) -> bool
    {
        self.visible
    }

}

pub extern fn add_empty(data : *const c_void)
{
    let wcb : & ui::WidgetCbData = unsafe {mem::transmute(data)};
    let action : &Action = unsafe {mem::transmute(wcb.widget)};
    let container : &mut Box<ui::WidgetContainer> = unsafe {mem::transmute(wcb.container)};

    ui::add_empty(container, action.view_id);
}

pub extern fn scene_new(data : *const c_void)
{
    let wcb : & ui::WidgetCbData = unsafe {mem::transmute(data)};
    let action : &Action = unsafe {mem::transmute(wcb.widget)};
    let container : &mut Box<ui::WidgetContainer> = unsafe {mem::transmute(wcb.container)};

    ui::scene_new(container, action.view_id);
}


pub extern fn play_scene(data : *const c_void)
{
    let wcb : & ui::WidgetCbData = unsafe {mem::transmute(data)};
    let action : &Action = unsafe {mem::transmute(wcb.widget)};
    let container : &mut Box<ui::WidgetContainer> = unsafe {mem::transmute(wcb.container)};

    if let Some(ref mut gv) = container.holder.borrow_mut().gameview {
        gv.state = 0;
        return;
    }

    let scene = if let Some(ref s) = container.context.scene {
        let scene = s.clone();
        scene.borrow_mut().init_components(&container.resource);
        scene
    }
    else {
        return;
    };

    let camera = if let Some(ref c) = scene.borrow().camera {
        c.clone()
    }
    else {
        return;
    };

    let win = unsafe {
        ui::jk_window_new(ui::view::gv_close_cb, mem::transmute(wcb.container))
    };

    let gv = ui::view::GameView::new(win, camera, scene, container.resource.clone());
    container.holder.borrow_mut().gameview = Some(gv);
}

pub extern fn pause_scene(data : *const c_void)
{
    let wcb : & ui::WidgetCbData = unsafe {mem::transmute(data)};
    let action : &Action = unsafe {mem::transmute(wcb.widget)};
    let container : &mut Box<ui::WidgetContainer> = unsafe {mem::transmute(wcb.container)};


    if let Some(ref mut gv) = container.holder.borrow_mut().gameview {
        gv.state = 1;
        //pause
    }
}



