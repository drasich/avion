use libc::{c_char, c_void, c_int};
use std::mem;
use sync::{RWLock, Arc};
use std::c_str::CString;
use std::collections::{DList};
use std::ptr;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use std::any::{Any, AnyRefExt};
use property::ChrisProperty;

use uuid::Uuid;

use scene;
use intersection;
use resource;
use geometry;
use vec;
use render;
use object;
use ui::{Tree,Property};
use ui;
use factory;
use operation;
use camera;
use property;
use context;
use control;
use control::Control;
use control::WidgetUpdate;

#[repr(C)]
pub struct Window;

#[link(name = "joker")]
extern {
    pub fn elm_simple_window_main();
    fn window_new() -> *const Window;
    fn window_button_new(window : *const Window);
    fn window_callback_set(
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

}

pub struct Master
{
    //windows : DList<Window>
    pub window : Option<*const Window>,
    //pub tree : Option<Box<Tree>>,
    pub tree : Option<Rc<RefCell<Box<Tree>>>>,
    pub property : Option<Rc<RefCell<Box<Property>>>>,
    pub scene : Option<Arc<RWLock<scene::Scene>>>,
    pub factory : factory::Factory,
    pub render : render::Render,
    pub control : Option<Rc<RefCell<Control>>>,
    //pub cont : PropertyContainer<'static>
}

impl Master
{
    fn _new() -> Master
    {
        let mut factory = factory::Factory::new();
        let context = Rc::new(RefCell::new(context::Context::new()));
        //let scene = factory.create_scene("scene/test.scene");
        //scene.save();
        let render = render::Render::new(&mut factory, context.clone());
        //let op_mgr = operation::OperationManager::new();

        let control = Rc::new(RefCell::new(
                Control::new(
                    render.camera.clone(),
                    context
                    )
                )
            );

        control.borrow_mut().context.borrow_mut().scene = Some(render.scene.clone());

        let mut m = Master {
            window : None,
            tree : None,
            property : None,
            scene : None,
            factory : factory,
            render : render,
            control : Some(control),
            //cont : PropertyContainer::new()
        };

        m.scene = Some(m.render.scene.clone());

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
    let w = unsafe {window_new()};

    let mut master = master_rc.borrow_mut();
    let control = match master.control {
        Some(ref c) => c.clone(),
        None => { println!("no control"); return; }
    };   

    unsafe {
        window_callback_set(
            w,
            mem::transmute(box control.clone()),
            mouse_down,
            mouse_up,
            mouse_move,
            mouse_wheel,
            key_down
            );
    }

    master.window = Some(w);

    let p = Rc::new(RefCell::new(ui::Property::new(
                w,
                master_rc.clone().downgrade(),
                control.clone())));

    let t = Rc::new(RefCell::new(ui::Tree::new(
                w,
                control.clone())));

    match (*master).scene {
        Some(ref s) => {
            t.borrow_mut().set_scene(&*s.read());
        },
        None => {}
    };

    match control.try_borrow_mut() {
        Some(ref mut c) => {
            c.property = Some(p.clone());
            c.tree = Some(t.clone());
        },
        None => {}
    };

    master.tree = Some(t);
    master.property = Some(p);
}

pub extern fn mouse_down(
    data : *const c_void,
    modifier : c_int,
    button : c_int,
    x : c_int, 
    y : c_int,
    timestamp : c_int
    )
{
    //println!("rust mouse down button {}, pos: {}, {}", button, x, y);
}

pub extern fn mouse_up(
    data : *const c_void,
    modifier : c_int,
    button : c_int,
    x : c_int, 
    y : c_int,
    timestamp : c_int
    )
{
    let control_rc : &Rc<RefCell<Control>> = unsafe {mem::transmute(data)};
    let mut c = control_rc.borrow_mut();
    c.mouse_up(button,x,y,timestamp);
}

pub extern fn mouse_move(
    data : *const c_void,
    //modifier : *const c_char,
    modifiers_flag : c_int,
    button : c_int,
    curx : c_int, 
    cury : c_int,
    prevx : c_int, 
    prevy : c_int,
    timestamp : c_int
    )
{
    let control_rc : &Rc<RefCell<Control>> = unsafe {mem::transmute(data)};
    let mut c = control_rc.borrow_mut();
    c.mouse_move(modifiers_flag, button, curx, cury, prevx, prevy, timestamp);
}

pub extern fn mouse_wheel(
    data : *const c_void,
    modifiers_flag: c_int,
    direction : c_int,
    z : c_int, 
    x : c_int, 
    y : c_int,
    timestamp : c_int
    )
{
    let control_rc : &Rc<RefCell<Control>> = unsafe {mem::transmute(data)};
    let mut c = control_rc.borrow_mut();
    c.mouse_wheel(modifiers_flag, direction, z, x, y, timestamp);
}

pub extern fn key_down(
    data : *const c_void,
    modifier : c_int,
    keyname : *mut c_char,
    key : *const c_char,
    timestamp : c_int
    )
{
    let control_rc : &Rc<RefCell<Control>> = unsafe {mem::transmute(data)};
    let mut c = control_rc.borrow_mut();

    let s = unsafe {CString::new(key as *const i8, false) };

    let yep = match s.as_str() {
        Some(ss) => ss,
        _ => return
    };

    let mut t = vec::Vec3::zero();

    match yep {
        "e" => t.z = -50f64,
        "d" => t.z = 50f64,
        "f" => t.x = 50f64,
        "s" => t.x = -50f64,
        "z" => {
            c.undo();
        },
        _ => {}
    }

    {
    let mut camera = c.camera.borrow_mut();
    let p = camera.object.read().position;
    camera.object.write().position = p + t;
    }
}

pub struct PropertyContainer<'a>
{
    //pub yo : HashMap<Uuid, DList<&'a WidgetUpdate+'a>>
    pub yo : HashMap<Uuid, &'a WidgetUpdate+'a>
    //pub yo : &'a WidgetUpdate+'a
}

impl<'a> PropertyContainer<'a>
{
    pub fn new() -> PropertyContainer<'a>
    {
        PropertyContainer {
            yo : HashMap::new()
        }
    }

    //pub fn add(&mut self, w : &'a WidgetUpdate+'a)
    //pub fn add(&mut self, w : &'a WidgetUpdate)
    pub fn add(&mut self, w : &'a WidgetUpdate)
    {
        self.yo.insert(Uuid::new_v4(), w);
    }
}

/*
impl WidgetUpdate for Master
{
    //fn update_changed<T : Any+Clone>(
    fn update_changed(
        &mut self,
        name : &str,
        //old : Option<&T>,
        new : &Any)
    {
        match self.property {
            Some(ref mut pp) => {
                match pp.try_borrow_mut() {
                    Some(ref mut p) =>
                        p.update_changed(name, new),
                    None => {}
                }
            },
            None => {}
        };

        /*
        match m.tree {
            Some(ref mut t) => {
                t.select(&o.read().id);
            }
            _ => {}
        }
        */
    }

}
*/


