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
use std::ffi::{CString, CStr};

use scene;
use object;
use ui::Window;
use ui::Master;
use ui::{ButtonCallback,EntryCallback};
use ui;
use control::Control;
use resource;
use uuid;

#[repr(C)]
pub struct JkAction;
pub struct JkLabel;
pub struct JkEntry;


#[link(name = "joker")]
extern {
    fn window_action_new(window : *const Window) -> *const JkAction;
    fn window_action_new_up(window : *const Window) -> *const JkAction;
    fn action_button_new(
        action : *const JkAction,
        name : *const c_char,
        data : *const c_void,
        button_callback : ButtonCallback) -> *const ui::Evas_Object;
    fn action_button_new1(
        action : *const JkAction,
        name : *const c_char)-> *const ui::Evas_Object;

    fn btn_cb_set(
        o : *const ui::Evas_Object,
        button_callback: ButtonCallback,
        data : *const c_void);

    fn action_label_new(
        action : *const JkAction,
        name : *const c_char) -> *const JkLabel;

    fn jk_label_set(
        label : *const JkLabel,
        name : *const c_char);

    fn action_entry_new(
        action : *const JkAction,
        name : *const c_char,
        data : *const c_void,
        entry_callback : EntryCallback ) -> *const JkEntry;

    fn action_show(
        action : *const JkAction,
        b : bool);
}

pub struct Action
{
    name : String,
    jk_action : *const JkAction,
    visible : bool,
    view_id : uuid::Uuid,
    pub entries : HashMap<String, *const JkEntry>
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
            view_id : view_id,
            entries : HashMap::new()
        }
    }

    //pub fn add_button(&self, name : &str, cb : ButtonCallback, data : ActionData)
    pub fn add_button(&self, name : &str, cb : ButtonCallback, data : ui::WidgetCbData) -> *const ui::Evas_Object
    {
        unsafe {
            /*
            action_button_new(
                self.jk_action,
                CString::new(name.as_bytes()).unwrap().as_ptr(),
                mem::transmute(box data),
                cb)
                */

            let b = action_button_new1(
                self.jk_action,
                CString::new(name.as_bytes()).unwrap().as_ptr());

            let mut data = data;
            data.object = Some(b);

            btn_cb_set(b,
                       cb,
                       mem::transmute(box data));
            b
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

    pub fn add_label(&self, name : &str) -> *const JkLabel
    {
        unsafe {
            action_label_new(
                self.jk_action,
                CString::new(name.as_bytes()).unwrap().as_ptr())
        }
    }

    pub fn add_entry(&mut self, key : String, name : &str, cb : EntryCallback, data : ui::WidgetCbData)
        -> *const JkEntry
    {
        let en = unsafe {
            action_entry_new(
                self.jk_action,
                CString::new(name.as_bytes()).unwrap().as_ptr(),
                mem::transmute(box data),
                cb)
        };
        self.entries.insert(key, en);
        en
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

pub extern fn scene_list(data : *const c_void)
{
    let wcb : & ui::WidgetCbData = unsafe {mem::transmute(data)};
    let action : &Action = unsafe {mem::transmute(wcb.widget)};
    let container : &mut Box<ui::WidgetContainer> = unsafe {mem::transmute(wcb.container)};

    ui::scene_list(container, action.view_id, wcb.object);
}


pub extern fn scene_rename(data : *const c_void, name : *const c_char)
{
    let wcb : & ui::WidgetCbData = unsafe {mem::transmute(data)};
    let action : &Action = unsafe {mem::transmute(wcb.widget)};
    let container : &mut Box<ui::WidgetContainer> = unsafe {mem::transmute(wcb.container)};

    let s = unsafe {CStr::from_ptr(name)}.to_str().unwrap();

    println!("todo scene rename to : {}", s);
    ui::scene_rename(container, action.view_id, s);
}

pub extern fn play_scene(data : *const c_void)
{
    let wcb : & ui::WidgetCbData = unsafe {mem::transmute(data)};
    let action : &Action = unsafe {mem::transmute(wcb.widget)};
    let container : &mut Box<ui::WidgetContainer> = unsafe {mem::transmute(wcb.container)};

    if container.play_gameview() {
        return;
    }

    let (camera, scene) = if let Some((camera, scene)) = container.can_create_gameview() {
        (camera, scene)
    }
    else {
        return;
    };

    let win = unsafe {
        ui::jk_window_new(ui::view::gv_close_cb, mem::transmute(wcb.container))
    };

    let gv = ui::view::GameView::new(win, camera, scene, container.resource.clone());
    //container.holder.borrow_mut().gameview = Some(gv);
    container.start_gameview(gv);

        println!("ADDDDDDDD animator");
        unsafe {
            ui::ecore_animator_add(ui::update_play_cb, mem::transmute(wcb.container));
        }

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



