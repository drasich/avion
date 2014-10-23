use libc::{c_char, c_void, c_int};
use render;
use object;
use std::collections::{DList};

use std::mem;
use sync::{RWLock, Arc};
use std::c_str::CString;
use std::ptr;
use std::f64::consts;
use scene;
use property::TProperty;
use property;
use intersection;
use resource;
use geometry;
use vec;

#[repr(C)]
pub struct Tree;
#[repr(C)]
pub struct JkProperty;
#[repr(C)]
pub struct Window;

#[link(name = "joker")]
extern {
    //fn simple_window_init();
    pub fn elm_simple_window_main();
    fn tree_widget_new() -> *const Tree;
    fn tree_register_cb(
        tree : *const Tree,
        name_get : extern fn(data : *const c_void) -> *const c_char,
        select : extern fn(data : *const c_void) -> (),
        can_expand : extern fn(data : *const c_void) -> bool,
        expand : extern fn(tree: *const Tree, data : *const c_void, parent: *const c_void) -> (),
        );

    fn tree_object_add(
        tree : *const Tree,
        object : *const c_void,
        parent : *const c_void,
        );
    fn window_new() -> *const Window;
    fn window_tree_new(window : *const Window) -> *const Tree;
    fn window_button_new(window : *const Window);
    fn window_property_new(window : *const Window) -> *const JkProperty;
    fn window_callback_set(
        window : *const Window,
        data: *const c_void,
        mouse_down : extern fn(
            data : *const c_void,
            modifier : *const c_char,
            button : c_int,
            x : c_int, 
            y : c_int,
            timestamp : c_int
            ),
        mouse_up : extern fn(
            data : *const c_void,
            modifier : *const c_char,
            button : c_int,
            x : c_int, 
            y : c_int,
            timestamp : c_int
            ),
        mouse_move : extern fn(
            data : *const c_void,
            modifier : *const c_char,
            button : c_int,
            curx : c_int, 
            cury : c_int,
            prevx : c_int, 
            prevy : c_int,
            timestamp : c_int
            ),
        mouse_wheel : extern fn(
            data : *const c_void,
            modifier : *const c_char,
            direction : c_int,
            z : c_int, 
            x : c_int, 
            y : c_int,
            timestamp : c_int
            ),
        key_down : extern fn(
            data : *const c_void,
            modifier : *const c_char,
            keyname : *mut c_char,
            key : *const c_char,
            timestamp : c_int
            ),
        );

    pub fn init_callback_set(
        cb: extern fn(*mut Master) -> (),
        master: *const Master 
        ) -> ();

    /*
    fn property_object_set(
        Property : *mut Property,
        object : *const c_void
        );

    fn property_object_update(
        Property : *mut Property
        );
        */

    /*
    fn property_set(
        Property : *mut Property,
        name : *const c_char,
        value : *const c_char
        );
        */

    fn property_register_cb(
        property : *const JkProperty,
        changed : extern fn(object : *const c_void, data : *const c_void),
        get : extern fn(data : *const c_void) -> *const c_char
        );

    fn property_data_set(
        property : *const JkProperty,
        data : *const c_void
        );
}

enum MasterState
{
    Idle,
    CameraRotation
}

pub struct Master
{
    //windows : DList<Window>
    pub window : Option<*const Window>,
    pub tree : Option<*const Tree>,
    pub property : Option<*const JkProperty>,
    pub scene : Option<Arc<RWLock<scene::Scene>>>,
    pub render : render::Render,
    pub state : MasterState

}

impl Master
{
    pub fn new() -> Master
    {
        let mut m = Master {
            window : None,
            tree : None,
            property : None,
            scene : None,
            render : render::Render::new(),
            state : Idle
        };

        m.scene = Some(m.render.scene.clone());

        m
    }
}

pub extern fn name_get(data : *const c_void) -> *const c_char {

    let o : &Arc<RWLock<object::Object>> = unsafe {
        mem::transmute(data)
    };

    //println!("name get {:?}", o);

    let cs = o.read().name.to_c_str();

    unsafe {
        cs.unwrap()
    }
}

pub extern fn select(data : *const c_void) -> () {
    let o : &Arc<RWLock<object::Object>> = unsafe {
        mem::transmute(data)
    };
    println!("select ! {} ", o.read().name);
}

pub extern fn can_expand(data : *const c_void) -> bool {
    let o : &Arc<RWLock<object::Object>> = unsafe {
        mem::transmute(data)
    };

    println!("can expand :{}", o.read().children.is_empty());
    return !o.read().children.is_empty();
}

pub extern fn expand(tree: *const Tree, data : *const c_void, parent : *const c_void) -> () {
    let o : &Arc<RWLock<object::Object>> = unsafe {
        mem::transmute(data)
    };

    println!("expanding ! {} ", o.read().name);

    for c in o.read().children.iter() {
        println!("expanding ! with child {} ", (*c).read().name);
        unsafe {
            tree_object_add(tree, mem::transmute(c), parent);
        }
    }
}

struct PropertySet
{
    name : String
}

trait PropertyTest
{
    fn set(property_set :&PropertySet, field : String, value : Self);
    
}

impl PropertyTest for int
{
    fn set(property_set: &PropertySet, field : String, value : int)
    {
        let test = value;
    }
}

pub extern fn changed(object : *const c_void, data : *const c_void) {
    let o : &Arc<RWLock<object::Object>> = unsafe {
        mem::transmute(object)
    };

    let s = unsafe {CString::new(data as *const i8, false) };

    match s.as_str() {
        Some(ss) => {
            let sss = property::SString(String::from_str(ss));
            o.clone().write().set_property("name", &sss);
        },
        _ => ()
    }

}


pub extern fn init_cb(master: *mut Master) -> () {
    unsafe {
        let w = window_new();
        window_callback_set(
            w,
            mem::transmute(master), //ptr::null(),//TODO
            mouse_down,
            mouse_up,
            mouse_move,
            mouse_wheel,
            key_down
            );

        let p = window_property_new(w);
        
        property_register_cb(
            p,
            changed,
            name_get
            ); 

        /*
        let t = window_tree_new(w);
        (*master).tree = Some(t);
        tree_register_cb(
            t,
            name_get,
            select,
            can_expand,
            expand);

        match (*master).scene {
            Some(ref s) => {
                for o in s.read().objects.iter() {
                    //tree_object_add(t, mem::transmute(box o.clone()), ptr::null());
                }

                let oo = s.read().object_find("yepyoyo");
                match oo {
                    Some(o) => { 
                        property_data_set(p, mem::transmute(box o.clone()));
                    }
                    None => {}
                };
            }
            None => {}
        };
        */


        //window_button_new(w);

        (*master).window = Some(w);
        (*master).property = Some(p);
    }
}

pub struct PropertyWidget {
    name : String,
}

trait PropertyShow
{
    //fn create_widget() -> Widget;
    fn create_widget(window : &Window);
}

impl PropertyShow for String
{
    fn create_widget(window : &Window)
    {
        let c = unsafe { window_property_new(window) };
    }
}

pub extern fn mouse_down(
    data : *const c_void,
    modifier : *const c_char,
    button : c_int,
    x : c_int, 
    y : c_int,
    timestamp : c_int
    )
{
    //println!("rust mouse down button {}, pos: {}, {}", button, x, y);
}

fn _rotate_camera(master : &mut Master, x : f64, y : f64)
{
  let mut camera = master.render.camera.borrow_mut();
  let cori = {
      camera.object.read().orientation
  };


  let result = {
  let mut cam = &mut camera.data;

  //if vec::Vec3::up().dot(&c.orientation.rotate_vec3(&vec::Vec3::up())) <0f64 {
  if vec::Vec3::up().dot(&cori.rotate_vec3(&vec::Vec3::up())) <0f64 {
      cam.yaw = cam.yaw + 0.005*x;
    println!("cam yaw ++");
  }
  else {
    println!("cam yaw --");
      cam.yaw = cam.yaw - 0.005*x;
  }

  //cam.pitch -= 0.005f*y;
  println!("cam yaw {}", cam.yaw);

  //TODO angles
  let qy = vec::Quat::new_axis_angle(vec::Vec3::up(), cam.yaw);
  //let qp = vec::Quat::new_axis_angle(vec::Vec3::right(), cam.pitch);
  //TODO
  //let result = qy;// * qp; 
  qy
  };

  let mut c = camera.object.write();
  (*c).orientation = result;

  master.state = CameraRotation;

  //c.angles.x = cam.pitch/M_PI*180.0;
  //(*c).angles.y = cam.yaw/consts::PI*180.0;

  /*
  Eina_List* objects = context_objects_get(v->context);

  if (eina_list_count(objects) > 0) {
    Vec3 objs_center = _objects_center(objects);
    if (!vec3_equal(objs_center, cam->center)) {
       cam->center = objs_center;
      camera_recalculate_origin(v->camera);
    }
  }
  */

  //camera_rotate_around(v->camera, result, cam->center);
}

pub extern fn mouse_up(
    data : *const c_void,
    modifier : *const c_char,
    button : c_int,
    x : c_int, 
    y : c_int,
    timestamp : c_int
    )
{

    let m : &mut Master = unsafe {mem::transmute(data)};

    match m.state {
        CameraRotation => {
            m.state = Idle;
            return;
        },
        _ => {}
    }

    //println!("rust mouse up button {}, pos: {}, {}", button, x, y);
    let r = m.render.camera.borrow().ray_from_screen(x as f64, y as f64, 10000f64);
    //TODO
    match m.render.line.write().mesh_render {
        Some (ref mr) => {
            match mr.mesh.resource {
                resource::ResData(ref mesh) => { mesh.write().add_line(geometry::Segment::new(r.start, r.start + r.direction), vec::Vec4::zero()); },
                _ => {}
            }
        },
        None => {}
    }
    //    add_line(r.start, r.direction - r.start);
    //println!("ray : {} ", r);

    for o in m.render.scene.read().objects.iter() {
        let ir = intersection::ray_object(r, &*o.read());
        if ir.hit {
            println!(" I hit object {} ", o.read().name);
        }
    }

}

pub extern fn mouse_move(
    data : *const c_void,
    modifier : *const c_char,
    button : c_int,
    curx : c_int, 
    cury : c_int,
    prevx : c_int, 
    prevy : c_int,
    timestamp : c_int
    )
{
    if button == 0 {
        return;
    }
    //println!("rust mouse move");
    let m : &mut Master = unsafe {mem::transmute(data)};
    let x : f64 = curx as f64 - prevx as f64;
    let y : f64 = cury as f64 - prevy as f64;
    _rotate_camera(m, x, y);
}

pub extern fn mouse_wheel(
    data : *const c_void,
    modifier : *const c_char,
    direction : c_int,
    z : c_int, 
    x : c_int, 
    y : c_int,
    timestamp : c_int
    )
{
    println!("move wheel");
}

pub extern fn key_down(
    data : *const c_void,
    modifier : *const c_char,
    keyname : *mut c_char,
    key : *const c_char,
    timestamp : c_int
    )
{
    println!("rust key_down");
    let m : &mut Master = unsafe {mem::transmute(data)};
    let mut camera = m.render.camera.borrow_mut();

    let s = unsafe {CString::new(key as *const i8, false) };

    let yep = match s.as_str() {
        Some(ss) => ss,
        _ => return
    };

    let mut t = vec::Vec3::zero();

    if yep == "e" {
        t.z = -0.1f64;
    }
    else if yep == "d" {
        t.z = 0.1f64;
    }
    else if yep == "f" {
        t.x = 0.1f64;
    }
    else if yep == "s" {
        t.x = -0.1f64;
    }

    let p = camera.object.read().position;

    camera.object.write().position = p + t;

    let ob = camera.object.read();

    println!("pos {}, rot {}, scale {} ", ob.position, ob.orientation, ob.scale);

}
