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

    fn action_show(
        action : *const JkAction,
        b : bool);
}

pub struct Action
{
    name : String,
    jk_action : *const JkAction,
    visible : bool,
    resource : Rc<resource::ResourceGroup>,
    view_id : uuid::Uuid
}

/*
#[derive(Clone)]
pub struct ActionData
{
    //tree : Rc<RefCell<Box<ui::Tree>>>,
    //property : Rc<RefCell<Box<ui::Property>>>,
    control : Rc<RefCell<Control>>,
    holder : Rc<RefCell<ui::view::Holder>>,
    resource : Rc<resource::ResourceGroup>
}

impl ActionData
{
    pub fn new(
    //tree : Rc<RefCell<Box<ui::Tree>>>,
    //property : Rc<RefCell<Box<ui::Property>>>,
    control : Rc<RefCell<Control>>,
    holder : Rc<RefCell<ui::view::Holder>>,
    resource : Rc<resource::ResourceGroup>
    ) -> ActionData
    {
        ActionData {
     //       tree : tree,
     //       property : property,
            control : control,
            holder : holder,
            resource: resource
        }
    }
}
*/

impl Action
{
    pub fn new(
        window : *const Window,
        resource : Rc<resource::ResourceGroup>,
        view_id : uuid::Uuid)
        -> Action
    {
        let mut a = Action {
            name : String::from("action_name"),
            jk_action : unsafe {window_action_new(window)},
            visible : true,
            resource : resource.clone(),
            view_id : view_id
        };

        //a.set_visible(false);

        //a.add_button("Add empty", add_empty);

        a
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
    //let ad : &ActionData = unsafe {mem::transmute(data)};
    let wcb : & ui::WidgetCbData = unsafe {mem::transmute(data)};
    let action : &Action = unsafe {mem::transmute(wcb.widget)};
    let container : &mut Box<ui::WidgetContainer> = unsafe {mem::transmute(wcb.container)};
    println!("TODO FIX FIX FIX add_empty");

    /*
    let view = if let Some(v) = container.find_view(action.view_id) {
        v
    }
    else {
        return;
    };
    */

    ui::add_empty(container, action.view_id);

    /*
    if ad.control.borrow_state() != BorrowState::Unused {
        println!("control already borrowed ");
        return;
    }

    let mut control = ad.control.borrow_mut();
    let o = control.add_empty("new object");
    */

    //let o = container.control

    /*
    match ad.property.borrow_state() {
        BorrowState::Unused => {
            ad.property.borrow_mut().set_object(&*o.read().unwrap());
        },
        _ => {println!("cannot borrow property");}
    };
    */

    /*
    match ad.tree.borrow_state() {
        BorrowState::Unused => {
            let mut t = ad.tree.borrow_mut();
            t.add_object(o.clone());
            t.select(&o.read().unwrap().id);
        }
        _ => {}
    }
    */
}

pub extern fn play_scene(data : *const c_void)
{
    let wcb : & ui::WidgetCbData = unsafe {mem::transmute(data)};
    let action : &Action = unsafe {mem::transmute(wcb.widget)};
    let container : &mut Box<ui::WidgetContainer> = unsafe {mem::transmute(wcb.container)};

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

    let gv = ui::view::GameView::new(camera, scene, container.resource.clone());
    container.holder.borrow_mut().gameview = Some(gv);
}


