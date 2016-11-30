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

use dormin::{camera, object, scene};
use ui::Window;
use ui::Master;
use ui::{ButtonCallback,EntryCallback};
use ui;
use control::Control;
use dormin::resource;
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
        name : *const c_char) -> *const ui::Evas_Object;

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

    pub fn add_label(&self, name : &str) -> *const ui::Evas_Object
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

pub extern fn open_game_view(data : *const c_void)
{
    let wcb : & ui::WidgetCbData = unsafe {mem::transmute(data)};
    let action : &Action = unsafe {mem::transmute(wcb.widget)};
    let container : &mut Box<ui::WidgetContainer> = unsafe {mem::transmute(wcb.container)};

    if container.open_gameview() {
        return;
    }

    let (camera, scene) = if let Some((camera, scene)) = container.can_create_gameview() {
        (camera, scene)
    }
    else {
        return;
    };

    let gv = ui::create_gameview_window(wcb.container, camera, scene, &ui::WidgetConfig::new());

    container.set_gameview(gv);
}

pub extern fn play_scene(data : *const c_void)
{
    let wcb : & ui::WidgetCbData = unsafe {mem::transmute(data)};
    let action : &Action = unsafe {mem::transmute(wcb.widget)};
    let container : &mut Box<ui::WidgetContainer> = unsafe {mem::transmute(wcb.container)};

    if container.play_gameview() {
        if container.anim.is_none() {
            container.anim = Some( unsafe {
                ui::ecore_animator_add(ui::update_play_cb, mem::transmute(wcb.container))
            });
        }
        return;
    }

    let (camera, scene) = if let Some((camera, scene)) = container.can_create_gameview() {
        (camera, scene)
    }
    else {
        return;
    };

    let gv = ui::create_gameview_window(wcb.container, camera, scene, &ui::WidgetConfig::new());
    container.set_gameview(gv);

    //println!("ADDDDDDDD animator");
    if container.anim.is_none() {
        container.anim = Some( unsafe {
            ui::ecore_animator_add(ui::update_play_cb, mem::transmute(wcb.container))
        });
    }
}

pub extern fn pause_scene(data : *const c_void)
{
    let wcb : & ui::WidgetCbData = unsafe {mem::transmute(data)};
    let action : &Action = unsafe {mem::transmute(wcb.widget)};
    let container : &mut Box<ui::WidgetContainer> = unsafe {mem::transmute(wcb.container)};


    if let Some(ref mut gv) = container.gameview {
        gv.state = 0;
        //pause
    }
}

use std::env;
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;
extern crate libloading;
pub extern fn compile_test(data : *const c_void)
{
    //let wcb : & ui::WidgetCbData = unsafe {mem::transmute(data)};
    //let action : &Action = unsafe {mem::transmute(wcb.widget)};
    //let container : &mut Box<ui::WidgetContainer> = unsafe {mem::transmute(wcb.container)};
    

    thread::spawn(|| {
    
        let child = Command::new("cargo").arg("build")
        .current_dir("/home/chris/code/compload")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to execute");

        let output = child.wait_with_output().expect("failed with child");
        println!("this is the out output {}", String::from_utf8_lossy(&output.stdout));
        println!("this is the error output {}", String::from_utf8_lossy(&output.stderr));

        let lib = if let Ok(l) = libloading::Library::new("/home/chris/code/compload/target/debug/libcompload.so") {
            l
        }
        else {
            println!("no lib");
            return;
        };

        unsafe {
            let fun  : libloading::Symbol<unsafe extern fn() ->i32> = if let Ok(f) = lib.get(b"get_my_i32") {
                f
            }
            else {
                return;
            };
            println!("{}",fun());
            /*
            let func  : libloading::Symbol<unsafe extern fn() ->i32> = if let Ok(f) = lib.get(b"get_my_i32") {
                f
            }
            else {
            println!("no symbol");
                return;
            };
            let yep = func();
            //println!("no prob : {}", yep);
            */
        }
    });
}


