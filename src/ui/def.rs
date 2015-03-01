use libc::{c_char, c_void, c_int};
use std::mem;
use std::sync::{RwLock, Arc};
use std::collections::{LinkedList};
use std::ptr;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use std::any::{Any};//, AnyRefExt};

use uuid::Uuid;

use intersection;
use resource;
use geometry;
use vec;
use object;
use ui::{Tree,Property,View};
use ui;
use factory;
use operation;
use camera;
use property;
use control;
use control::Control;
use control::WidgetUpdate;

#[repr(C)]
pub struct Window;
#[repr(C)]
pub struct Evas_Object;
#[repr(C)]
pub struct JkGlview;

pub type RenderFunc = extern fn(data : *const c_void);
pub type ResizeFunc = extern fn(data : *const c_void, w : c_int, h : c_int);

pub type RenderFuncTmp = extern fn(data : *mut View);
pub type ResizeFuncTmp = extern fn(data : *mut View, w : c_int, h : c_int);

/*
        init_cb: extern fn(*mut View),// -> (),
        draw_cb: extern fn(*mut View), // -> (),
        resize_cb: extern fn(*mut View, w : c_int, h : c_int) -> (),
        render: *const View
        */

#[link(name = "joker")]
extern {
    pub fn elm_simple_window_main();
    pub fn window_new() -> *const Window;
    pub fn jk_window_new() -> *const Evas_Object;
    pub fn jk_glview_new(
        win : *const Evas_Object,
        data : *const c_void,
        init : RenderFunc,
        draw : RenderFunc,
        resize : ResizeFunc
        ) -> *const JkGlview;
    pub fn tmp_func(
        window: *const Window,
        data : *const c_void,
        init : RenderFuncTmp,
        draw : RenderFuncTmp,
        resize : ResizeFuncTmp
        );
    //fn window_button_new(window : *const Window);
    pub fn window_callback_set(
        window : *const Window,
        data: *const c_void,
        mouse_down : extern fn(
            data : *const c_void,
            modifier : c_int,
            button : c_int,
            x : c_int, 
            y : c_int,
            timestamp : c_int
            ),
        mouse_up : extern fn(
            data : *const c_void,
            modifier : c_int,
            button : c_int,
            x : c_int, 
            y : c_int,
            timestamp : c_int
            ),
        mouse_move : extern fn(
            data : *const c_void,
            modifier : c_int,
            button : c_int,
            curx : c_int, 
            cury : c_int,
            prevx : c_int, 
            prevy : c_int,
            timestamp : c_int
            ),
        mouse_wheel : extern fn(
            data : *const c_void,
            modifier : c_int,
            direction : c_int,
            z : c_int, 
            x : c_int, 
            y : c_int,
            timestamp : c_int
            ),
        key_down : extern fn(
            data : *const c_void,
            modifier : c_int,
            keyname : *mut c_char,
            key : *const c_char,
            timestamp : c_int
            ),
        );

    pub fn init_callback_set(
        //cb: extern fn(*mut Rc<RefCell<Master>>) -> (),
        //master: *const Rc<RefCell<Master>>
        cb: extern fn(*mut c_void) -> (),
        master: *const c_void
        ) -> ();
    pub fn exit_callback_set(
        cb: extern fn(*mut c_void) -> (),
        master: *const c_void
        ) -> ();

}

pub struct Master
{
    pub factory : factory::Factory,
    views : LinkedList<Box<View>>,
}

impl Master
{
    fn _new() -> Master
    {
        let factory = factory::Factory::new();

        let mut m = Master {
            factory : factory,
            views : LinkedList::new(),
        };

        let v = box View::new(&mut m.factory);
        m.views.push_back(v);

        m
    }

    pub fn new() -> Rc<RefCell<Master>>
    {
        let m = Master::_new();
        let mrc = Rc::new(RefCell::new(m));

        mrc
    }
}

pub extern fn init_cb(data: *mut c_void) -> () {
    let master_rc : &Rc<RefCell<Master>> = unsafe {mem::transmute(data)};
    let mut master = master_rc.borrow_mut();

    for v in master.views.iter_mut()
    {
        v.init();

        if let Some(w) = v.window {
            unsafe {
                {
                let yo : *const c_void = unsafe { mem::transmute(v) };
                ui::window_callback_set(
                    w,
                    yo, 
                    //mem::transmute(v),
                    ui::view::mouse_down,
                    ui::view::mouse_up,
                    ui::view::mouse_move,
                    ui::view::mouse_wheel,
                    ui::view::key_down
                    );
                }
            }
        }
    }

    for v in master.views.iter_mut()
    {
        if let Some(w) = v.window {
            unsafe {
                tmp_func(
                    w,
                    mem::transmute(&**v),
                    ui::view::init_cb,
                    ui::view::draw_cb,
                    ui::view::resize_cb);
            }
        }

    }
}

pub extern fn exit_cb(data: *mut c_void) -> () {
    let master_rc : &Rc<RefCell<Master>> = unsafe {mem::transmute(data)};
    let master = master_rc.borrow();

    for v in master.views.iter()
    {
        match v.context.borrow().scene {
            Some(ref s) => {
                s.read().unwrap().save();
                s.read().unwrap().savetoml();
            },
            None => {}
        }
    }
}

