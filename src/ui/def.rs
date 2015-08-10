use libc::{c_char, c_void, c_int};
use std::mem;
use std::sync::{RwLock, Arc};
use std::collections::{LinkedList};
use std::ptr;
use std::rc::Rc;
use std::cell::{RefCell, BorrowState};
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
use uuid;

#[repr(C)]
pub struct Window;
#[repr(C)]
pub struct Evas_Object;
#[repr(C)]
pub struct JkGlview;

pub type RustCb = extern fn(data : *mut c_void);
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
    pub fn jk_window_new(cb : RustCb, cb_data : *const c_void) -> *const Evas_Object;
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
    pub resource : Rc<resource::ResourceGroup>,
    views : LinkedList<Box<View>>,
}

impl Master
{
    fn _new(container : &mut Box<WidgetContainer>) -> Master
    {
        let factory = factory::Factory::new();
        let resource = Rc::new(resource::ResourceGroup::new());

        let mut m = Master {
            factory : factory,
            resource : resource,
            views : LinkedList::new(),
        };

        let v = box View::new(&m.factory, m.resource.clone(), container);
        m.views.push_back(v);
        //container.views.push(v);

        m
    }

    pub fn new(container : &mut Box<WidgetContainer>) -> Rc<RefCell<Master>>
    {
        let m = Master::_new(container);
        let mrc = Rc::new(RefCell::new(m));

        mrc
    }

}

pub extern fn init_cb(data: *mut c_void) -> () {
    let app_data : &AppCbData = unsafe {mem::transmute(data)};
    //let master_rc : &Rc<RefCell<Master>> = unsafe {mem::transmute(data)};
    let master_rc : &Rc<RefCell<Master>> = unsafe {mem::transmute(app_data.master)};
    let container : &mut Box<WidgetContainer> = unsafe {mem::transmute(app_data.container)};
    let mut master = master_rc.borrow_mut();

    /*
    for v in master.views.iter_mut()
    {
        v.init(container);

        if let Some(w) = v.window {
            unsafe {
                {
                let view : *const c_void = mem::transmute(&**v);
                let wcb = ui::WidgetCbData::with_ptr(container, view);

                ui::window_callback_set(
                    w,
                    mem::transmute(box wcb),
                    //view
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
    */

    /*
    for v in master.views.iter()
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
    */

    while let Some(mut v) = master.views.pop_front() {

        v.init(container);

        if let Some(w) = v.window {
            unsafe {
                {
                let view : *const c_void = mem::transmute(&*v);
                let wcb = ui::WidgetCbData::with_ptr(container, view);

                ui::window_callback_set(
                    w,
                    mem::transmute(box wcb),
                    //view
                    //mem::transmute(v),
                    ui::view::mouse_down,
                    ui::view::mouse_up,
                    ui::view::mouse_move,
                    ui::view::mouse_wheel,
                    ui::view::key_down
                    );

                let wcb = ui::WidgetCbData::with_ptr(container, view);

                tmp_func(
                    w,
                    //view, //mem::transmute(&*v),
                    mem::transmute(box wcb),
                    ui::view::init_cb,
                    ui::view::draw_cb,
                    ui::view::resize_cb);
                }
            }
        }
        container.views.push(v);
    }
}

pub extern fn exit_cb(data: *mut c_void) -> () {
    let master_rc : &Rc<RefCell<Master>> = unsafe {mem::transmute(data)};
    let master = master_rc.borrow();

    for v in master.views.iter()
    {
        match v.context.borrow().scene {
            Some(ref s) => {
                s.borrow().save();

                //old
                //s.read().unwrap().save();
                //s.read().unwrap().savetoml();
                //s.borrow().savetoml();
            },
            None => {}
        }
    }
}

pub trait Widget
{
    fn update(&self, change : operation::Change)
    {
        println!("please implement me");
    }

    fn set_visible(&self, b : bool)
    {
        println!("please implement me");
    }
}

pub struct WidgetContainer
{
    pub widgets : Vec<Box<Widget>>,
    pub tree : Option<Box<Tree>>,
    pub property : Option<Box<Property>>,
    views : Vec<Box<View>>,
}

/*
pub struct ControlContainer
{
    pub control : Box<Control>,
    pub context : Box<Context>
}
*/


impl WidgetContainer
{
    pub fn new() -> WidgetContainer
    {
        WidgetContainer {
            widgets : Vec::new(),
            tree : None,
            property : None,
            views : Vec::new()
        }
    }

    pub fn handle_change(&self, change : &operation::Change, widget_origin: uuid::Uuid)
    {
        match *change {
            operation::Change::DirectChange(ref name) => {
                let o = match self.get_selected_object() {
                    Some(ob) => ob,
                    None => {
                        println!("direct change, no objetcs selected");
                        return;
                    }
                };

                if name == "object/name" {
                    match self.tree {
                        Some(ref t) => {
                            t.update_object(&o.read().unwrap().id);
                            },
                            None => {}
                    };
                }
            },
            operation::Change::SelectedChange => {
            },
            _ => {}
        }
    }

    pub fn handle_event(&self, event : ui::Event, widget_origin: uuid::Uuid)
    {
        match event {
            Event::SelectObject(ob) => {
                println!("selected : {}", ob.read().unwrap().name);

                for v in self.views.iter() {
                    match v.control.borrow_state() {
                        BorrowState::Unused => {
                            let mut l = Vec::new();
                            l.push(ob.read().unwrap().id.clone());
                            v.control.borrow_mut().select_by_id(&mut l);
                        },
                        _ => { println!("control already borrowed : tree sel ->add_ob"); return;}
                    };
                }




            },
            Event::UnselectObject(ob) => {
                println!("unselected : {}", ob.read().unwrap().name);


                for v in self.views.iter() {
                    let o = match v.control.borrow_state() {
                        BorrowState::Unused => {
                            {
                                let mut l = LinkedList::new();
                                l.push_back(ob.read().unwrap().id.clone());
                                v.control.borrow_mut().unselect(&l);
                            }
                            v.control.borrow().get_selected_object()
                        },
                        _ => {
                            println!("already borrowed : mouse_up add_ob ->sel ->add_ob");
                        return;
                        }
                    };
                }


            },
            _ => {}
        }

    }

    fn get_selected_object(&self) -> Option<Arc<RwLock<object::Object>>>
    {
        //TODO
        for v in self.views.iter() {
            if let Some(ob) = v.get_selected_object()
            {
                return Some(ob);
            }
        }

        None
    }
}

//Send to c with mem::transmute(box data)  and free in c
pub struct WidgetCbData
{
    pub container : *const WidgetContainer,
    //widget : *const Widget
    pub widget : *const c_void
}

impl WidgetCbData {
    //pub fn new(c : &Box<WidgetContainer>, widget : &Box<Widget>)
    pub fn with_ptr(c : &Box<WidgetContainer>, widget : *const c_void) -> WidgetCbData
    {
        println!("TODO free me in c");
        WidgetCbData {
            container : unsafe {mem::transmute(c)},
            widget : widget
        }

    }
}


pub struct AppCbData
{
    pub master : *const c_void,
    pub container : *const c_void
}

//TODO choose how deep is the event, like between those 3 things
pub enum Event
{
    KeyPressed(String),
    ViewKeyPressed(String),
    ShowTree(String),
    //SelectObject(Vec<Arc<RwLock<object::Object>>>)
    SelectObject(Arc<RwLock<object::Object>>),
    UnselectObject(Arc<RwLock<object::Object>>),
    Empty
}
