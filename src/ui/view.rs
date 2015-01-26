use std::rc::Rc;
use std::cell::RefCell;
use std::sync::{RwLock, Arc};
use libc::{c_char, c_void, c_int};
use std::mem;
use std::ffi;
use std::ffi::CString;
use std::str;
use object;
use mesh;
use shader;
use transform;

use ui;
use render;
use render::Render;
use factory;
use context;
use resource;
use resource::Create;
use mesh_render;
use vec;
use geometry;
use material;
use ui::dragger;
use camera;

use control;
use control::Control;
use control::WidgetUpdate;

use scene;

#[link(name = "cypher")]
extern {
    pub fn draw_callback_set(
        init_cb: extern fn(*mut View) -> (),
        draw_cb: extern fn(*mut View) -> (),
        resize_cb: extern fn(*mut View, w : c_int, h : c_int) -> (),
        render: *const View
        ) -> ();
}

pub struct View
{
    render : Box<Render>,
    pub control : Rc<RefCell<Control>>,
    pub context : Rc<RefCell<context::Context>>,

    pub window : Option<*const ui::Window>,
    //pub tree : Option<Box<Tree>>,
    pub tree : Option<Rc<RefCell<Box<ui::Tree>>>>,
    pub property : Option<Rc<RefCell<Box<ui::Property>>>>,

    //pub dragger : Arc<RwLock<object::Object>>,
    pub dragger : Rc<RefCell<dragger::DraggerManager>>,

    pub camera : Rc<RefCell<camera::Camera>>
}

impl View
{
    pub fn new(factory: &mut factory::Factory) -> View
    {
        let scene_path = "scene/simple.scene";
        let scene = Arc::new(RwLock::new(scene::Scene::new_from_file(scene_path)));

        let camera = Rc::new(RefCell::new(factory.create_camera()));
        {
            let mut cam = camera.borrow_mut();
            cam.pan(&vec::Vec3::new(100f64,20f64,100f64));
            cam.lookat(vec::Vec3::new(0f64,5f64,0f64));
        }

        let context = Rc::new(RefCell::new(context::Context::new()));
        context.borrow_mut().scene = Some(scene.clone());
        let dragger = Rc::new(RefCell::new(dragger::DraggerManager::new(factory)));

        let control = Rc::new(RefCell::new(
                Control::new(
                    camera.clone(),
                    context.clone(),
                    dragger.clone()
                    )));

        let render = box Render::new(factory, camera.clone());

        let v = View {
            render : render,
            control : control,
            context : context,
            
            window : None,
            tree : None,
            property: None,

            dragger : dragger,

            camera : camera
        };

        return v;
    }

    pub fn init(&mut self) -> () {
        let w = unsafe {ui::window_new()};
        self.window = Some(w);

        let control = &self.control;

        /*
        unsafe {
            ui::window_callback_set(
                w,
                mem::transmute(box control.clone()),
                mouse_down,
                mouse_up,
                mouse_move,
                mouse_wheel,
                key_down
                );
        }
        */

        let p = Rc::new(RefCell::new(ui::Property::new(
                    w,
                    control.clone())));

        let t = Rc::new(RefCell::new(ui::Tree::new(
                    w,
                    control.clone())));

        match self.context.borrow().scene {
            Some(ref s) => {
                t.borrow_mut().set_scene(&*s.read().unwrap());
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

        self.tree = Some(t);
        self.property = Some(p);
    }

    fn init_render(&mut self)
    {
        self.render.init();
    }

    fn draw(&mut self)
    {
        let context = self.context.borrow();

        let scene = match context.scene {
            Some(ref s) => s.read().unwrap(),
            None => return
        };

        let obs = &scene.objects;
        let sel = &context.selected;

        let mut center = vec::Vec3::zero();
        let mut ori = vec::Quat::identity();
        for o in sel.iter() {
            center = center + o.read().unwrap().position;
            ori = ori * o.read().unwrap().world_orientation();
        }

        if sel.len() > 0 {
            center = center / (sel.len() as f64);
            let mut dragger = self.dragger.borrow_mut();
            dragger.set_position(center);
            dragger.set_orientation(transform::Orientation::Quat(ori));
            let scale = self.camera.borrow().get_camera_resize_w(0.05f64);
            dragger.set_scale(scale);
        }

        self.render.draw(obs, sel, &self.dragger.borrow().get_objects());
    }

    fn resize(&mut self, w : c_int, h : c_int)
    {
        self.render.resize(w, h);
    }

    fn get_selected_object(&self) -> Option<Arc<RwLock<object::Object>>>
    {
        let c = match self.context.try_borrow(){
            Some(con) => con,
            None => { println!("cannot borrow context"); return None; }
        };

        match c.selected.front() {
            Some(o) => return Some(o.clone()),
            None => {
                println!("no objetcs selected");
                return None;
            }
        };
    }

    fn handle_control_change(&self, change : control::Change)
    {
        let sel = self.get_selected_object();

        let (name,id_list) = if let control::Change::Objects(name, id_list) = change {
            (name,id_list)
        }
        else {
            return;
        };
        
        for id in id_list.iter() {
            if let Some(ref o) = sel {
                if *id == o.read().unwrap().id  {
                    match self.property.clone() {
                        Some(ref mut pp) =>
                            match pp.try_borrow_mut() {
                                Some(ref mut p) => {
                                    p.update_object(&*o.read().unwrap(), "");

                                },
                                None=> {}
                            },
                            None => {}
                    };
                }
            }

            if name.as_slice() == "object/name" {
                match self.tree.clone() {
                    Some(ref mut tt) =>
                        match tt.try_borrow_mut() {
                            Some(ref mut t) => {
                                t.update_object(id);
                            },
                            None=> {}
                        },
                        None => {}
                };
            }
        }
    }

}

/*
pub struct WindowView
{
    pub window : Option<*const Window>,
    pub view : View
}
*/

pub extern fn mouse_down(
    data : *const c_void,
    modifier : c_int,
    button : c_int,
    x : c_int, 
    y : c_int,
    timestamp : c_int
    )
{
    let view : &Box<View> = unsafe {mem::transmute(data)};
    let control_rc = view.control.clone();

    //println!("rust mouse down button {}, pos: {}, {}", button, x, y);
    //let control_rc : &Rc<RefCell<Control>> = unsafe {mem::transmute(data)};
    let mut c = control_rc.borrow_mut();
    c.mouse_down(button,x,y,timestamp);
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
    let view : &Box<View> = unsafe {mem::transmute(data)};
    let control_rc = view.control.clone();
    //let control_rc : &Rc<RefCell<Control>> = unsafe {mem::transmute(data)};
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
    let view : &Box<View> = unsafe {mem::transmute(data)};
    let control_rc = view.control.clone();

    //let control_rc : &Rc<RefCell<Control>> = unsafe {mem::transmute(data)};
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
    let view : &Box<View> = unsafe {mem::transmute(data)};
    let control_rc = view.control.clone();

    //let control_rc : &Rc<RefCell<Control>> = unsafe {mem::transmute(data)};
    let c = control_rc.borrow_mut();
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
    let view : &Box<View> = unsafe {mem::transmute(data)};

    let change = {
        let control_rc = view.control.clone();
        //let control_rc : &Rc<RefCell<Control>> = unsafe {mem::transmute(data)};
        let mut c = control_rc.borrow_mut();

        let key_str = {
            let s = unsafe {ffi::c_str_to_bytes(&key)};
            match str::from_utf8(s) {
                Ok(ss) => ss.to_string(),
                _ => {
                    println!("error");
                    return;
                }
            }
        };

        let keyname_str = {
            let keynameconst = keyname as *const c_char;
            let s = unsafe {ffi::c_str_to_bytes(&keynameconst)};
            match str::from_utf8(s) {
                Ok(ss) => ss.to_string(),
                _ => {
                    println!("error");
                    return
                }
            }
        };

        c.key_down(modifier, keyname_str.as_slice(), key_str.as_slice(), timestamp)
    };

    view.handle_control_change(change);

}


//TODO remove
fn create_repere(m : &mut mesh::Mesh, len : f64)
{
    let red = vec::Vec4::new(1.0f64,0.247f64,0.188f64,1f64);
    let green = vec::Vec4::new(0.2117f64,0.949f64,0.4156f64,1f64);
    let blue = vec::Vec4::new(0f64,0.4745f64,1f64,1f64);

    let s = geometry::Segment::new(
        vec::Vec3::zero(), vec::Vec3::new(len, 0f64, 0f64));
    m.add_line(s, red);

    let s = geometry::Segment::new(
        vec::Vec3::zero(), vec::Vec3::new(0f64, len, 0f64));
    m.add_line(s, green);

    let s = geometry::Segment::new(
        vec::Vec3::zero(), vec::Vec3::new(0f64, 0f64, len));
    m.add_line(s, blue);
}


pub extern fn init_cb(v : *mut View) -> () {
    unsafe {
        return (*v).init_render();
    }
}

pub extern fn draw_cb(v : *mut View) -> () {
    unsafe {
        return (*v).draw();
    }
}

pub extern fn resize_cb(v : *mut View, w : c_int, h : c_int) -> () {
    unsafe {
        return (*v).resize(w, h);
    }
}

