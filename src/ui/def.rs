use libc::{c_char, c_void, c_int};
use std::mem;
use sync::{RWLock, Arc};
use std::c_str::CString;
use std::collections::{DList};
use std::ptr;
use std::rc::Rc;
use std::cell::RefCell;

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

//use tree;
//pub use Tree;

#[repr(C)]
pub struct Window;

#[link(name = "joker")]
extern {
    //fn simple_window_init();
    pub fn elm_simple_window_main();
    fn window_new() -> *const Window;
    fn window_button_new(window : *const Window);
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
        //cb: extern fn(*mut Rc<RefCell<Master>>) -> (),
        //master: *const Rc<RefCell<Master>>
        cb: extern fn(*mut c_void) -> (),
        master: *const c_void
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
    pub tree : Option<Box<Tree>>,
    //pub property : Option<*const JkProperty>,
    pub property : Option<Box<Property>>,
    pub scene : Option<Arc<RWLock<scene::Scene>>>,
    pub factory : factory::Factory,
    pub render : render::Render,
    pub state : MasterState,
    //pub objects : DList<Arc<RWLock<object::Object>>>,
    pub operation_mgr : operation::OperationManager
}

impl Master
{
    pub fn new() -> Master
    {
        let mut factory = factory::Factory::new();
        //let scene = factory.create_scene("scene/test.scene");
        //scene.save();
        let render = render::Render::new(&mut factory);
        let op_mgr = operation::OperationManager::new();

        let mut m = Master {
            window : None,
            tree : None,
            property : None,
            scene : None,
            factory : factory,
            render : render,
            state : Idle,
            //objects : DList::new(),
            operation_mgr : op_mgr
        };

        m.scene = Some(m.render.scene.clone());

        m
    }

    pub fn mouse_up(
            &mut self, 
            button : i32,
            x : i32, 
            y : i32,
            timestamp : i32)
    {
        let mut m = self;
        match m.tree {
            Some(ref yep) => {},
            _ => {}
        }

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
                    resource::ResData(ref mesh) => {
                        mesh.write().add_line(
                            geometry::Segment::new(r.start, r.start + r.direction),
                            vec::Vec4::zero()); },
                            _ => {}
                }
            },
            None => {}
        }

        m.render.objects_selected.clear();

        for o in m.render.scene.read().objects.iter() {
            let ir = intersection::ray_object(&r, &*o.read());
            if ir.hit {
                println!(" I hit object {} ", o.read().name);
                m.render.objects_selected.push(o.clone());
            }
        }

        if m.render.objects_selected.len() == 1 {
            //TODO select tree
            match (m.render.objects_selected.front()) {
                Some(o) => {
                    match m.property {
                        Some(ref mut p) => unsafe {
                            //property_data_set(p, mem::transmute(box o.clone()));
                            p.data_set(mem::transmute(box o.clone()));
                            p.set_object(&*o.read());
                        },
                        None => {}
                    }
                },
                _ => {},
            }

            match m.render.objects_selected.front() {
                Some(o) => {
                    match m.tree {
                        Some(ref mut t) => {
                            t.select(&o.read().id);
                        }
                        _ => {}
                    }
                },
                _ => {}
            }
        }
    }

    pub fn select(&mut self, id : &Uuid)
    {
        println!(".................select is called : {} ", id);
        self.render.objects_selected.clear();

        for o in self.render.scene.read().objects.iter() {
            if o.read().id == *id {
                self.render.objects_selected.push(o.clone());
                match self.property {
                    Some(ref p) => {
                        p.data_set(unsafe {mem::transmute(box o.clone())});
                        p.set_object(&*o.read());
                    },
                    None => {}
                }
                break;
            }
        }

    }
}


/*
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
*/

//pub extern fn init_cb(master_rc: *mut Rc<RefCell<Master>>) -> () {
pub extern fn init_cb(data: *mut c_void) -> () {
    unsafe {
        let master_rc : &Rc<RefCell<Master>> = mem::transmute(data);
        let w = window_new();
        window_callback_set(
            w,
            mem::transmute(box master_rc.clone()), //ptr::null(),//TODO
            mouse_down,
            mouse_up,
            mouse_move,
            mouse_wheel,
            key_down
            );

        /*
        let p = window_property_new(w);
        
        property_register_cb(
            p,
            changed,
            name_get
            ); 
            */

        let p = ui::Property::new(w, master_rc.clone().downgrade());

        //*
        let mut t = ui::Tree::new(w, master_rc.clone().downgrade());

        let mut master = master_rc.borrow_mut();

        match (*master).scene {
            Some(ref s) => {
                t.set_scene(&*s.read());
                let oo = s.read().object_find("yepyoyo");
                match oo {
                    Some(o) => { 
                        //property_data_set(p, mem::transmute(box o.clone()));
                        p.data_set(mem::transmute(box o.clone()));
                        p.set_object(&*o.read());
                    }
                    None => {}
                };
            },
            None => {}
        };
        //*/

        master.tree = Some(t);

        //window_button_new(w);

        (*master).window = Some(w);
        (*master).property = Some(p);
    }
}

/*
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
*/

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
  let cori = camera.object.read().orientation;


  let result = {
  let cam = &mut camera.data;

  //if vec::Vec3::up().dot(&c.orientation.rotate_vec3(&vec::Vec3::up())) <0f64 {
  if vec::Vec3::up().dot(&cori.rotate_vec3(&vec::Vec3::up())) <0f64 {
      cam.yaw = cam.yaw + 0.005*x;
  }
  else {
      cam.yaw = cam.yaw - 0.005*x;
  }

  cam.pitch -= 0.005*y;

  //TODO angles
  let qy = vec::Quat::new_axis_angle(vec::Vec3::up(), cam.yaw);
  let qp = vec::Quat::new_axis_angle(vec::Vec3::right(), cam.pitch);
  //TODO
  qy * qp
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
    let master_rc : &Rc<RefCell<Master>> = unsafe {mem::transmute(data)};

    //let m : &mut Master = unsafe {mem::transmute(data)};
    let mut m = master_rc.borrow_mut();
    m.mouse_up(button,x,y,timestamp);

    /*
    match m.tree {
        Some(ref yep) => {},
        _ => {}
    }

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
                resource::ResData(ref mesh) => {
                    mesh.write().add_line(
                        geometry::Segment::new(r.start, r.start + r.direction),
                        vec::Vec4::zero()); },
                _ => {}
            }
        },
        None => {}
    }

    m.render.objects_selected.clear();

    for o in m.render.scene.read().objects.iter() {
        let ir = intersection::ray_object(&r, &*o.read());
        if ir.hit {
            println!(" I hit object {} ", o.read().name);
            m.render.objects_selected.push(o.clone());
        }
    }

    if m.render.objects_selected.len() == 1 {
        //TODO select tree
        match (m.render.objects_selected.front(), m.property) {
            (Some(o), Some(p)) => unsafe {
                property_data_set(p, mem::transmute(box o.clone()));
            },
            _ => {},
        }

        match m.render.objects_selected.front() {
            Some(o) => {
                match m.tree {
                    Some(ref t) => {
                        t.select(&o.read().name);
                    }
                    _ => {}
                }
            },
            _ => {}
        }
    }
    */

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
    //let m : &mut Master = unsafe {mem::transmute(data)};
    let master_rc : &Rc<RefCell<Master>> = unsafe {mem::transmute(data)};
    let mut m = master_rc.borrow_mut();

    let x : f64 = curx as f64 - prevx as f64;
    let y : f64 = cury as f64 - prevy as f64;
    _rotate_camera(&mut *m, x, y);
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
    //let m : &mut Master = unsafe {mem::transmute(data)};
    let master_rc : &Rc<RefCell<Master>> = unsafe {mem::transmute(data)};
    let mut m = master_rc.borrow_mut();

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
        "z" => m.operation_mgr.undo(),
        _ => {}
    }

    {
    let mut camera = m.render.camera.borrow_mut();
    let p = camera.object.read().position;
    camera.object.write().position = p + t;
    }
}
