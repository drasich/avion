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
    pub property : Option<Rc<RefCell<Box<Property>>>>,
    pub scene : Option<Arc<RWLock<scene::Scene>>>,
    pub factory : factory::Factory,
    pub render : render::Render,
    pub state : MasterState,
    //pub objects : DList<Arc<RWLock<object::Object>>>,
    //pub operation_mgr : Option<operation::OperationManager>,
    //pub operation_mgr : Rc<RefCell<operation::OperationManager>>,
    pub control : Option<Rc<RefCell<Control>>>,
    //pub cont : PropertyContainer<'static>
}

impl Master
{
    fn _new() -> Master
    {
        let mut factory = factory::Factory::new();
        let context = Rc::new(RefCell::new(Context::new()));
        //let scene = factory.create_scene("scene/test.scene");
        //scene.save();
        let render = render::Render::new(&mut factory, context.clone());
        //let op_mgr = operation::OperationManager::new();

        let mut m = Master {
            window : None,
            tree : None,
            property : None,
            scene : None,
            factory : factory,
            render : render,
            state : Idle,
            //objects : DList::new(),
            //operation_mgr : Rc::new(RefCell::new(op_mgr)),//None,//op_mgr,
            control : None, //Rc::new(RefCell::new(Control::new(render.camera.clone()))),
            //cont : PropertyContainer::new()
        };

        m.scene = Some(m.render.scene.clone());

        let control = Rc::new(RefCell::new(
                Control::new(
                    m.render.camera.clone(),
                    context
                    )
                )
            );
        control.borrow_mut().context.borrow_mut().scene = Some(m.render.scene.clone());
        m.control = Some(control);


        //m.operation_mgr.closure = Some( |name, new| { m.test2(name, new); });
        //m.operation_mgr.closure = Some( |name| { println!("yep : {}",name); });

        m
    }

    pub fn new() -> Rc<RefCell<Master>>
    {
        let mut m = Master::_new();
        let yep = |name : &str, new : &Any| {
            println!("yep : {}",name);
            //m.test(name, new);
        };
        //op_mgr.closure = Some( |name| { println!("yep : {}",name); });
        //m.operation_mgr.closure = Some(yep);

        let mrc = Rc::new(RefCell::new(m));

        //let mut op_mgr = operation::OperationManager::new();

        //hm.operation_mgr.closure = Some( |name, new| { m.test2(name, new); });
        let yep = |name : &str, new : &Any| {
            println!("yep : {}",name);
            //mrc.borrow_mut().test(name, new);
        };
        //op_mgr.closure = Some( |name| { println!("yep : {}",name); });
        //op_mgr.closure = Some(yep);

        /*
        op_mgr.master = Some(mrc.clone().downgrade());

        match op_mgr.master {
            Some(_) =>
            {
                println!("I set the master !!!!!!!!!");
            },
            None => {
                println!("it is none???????");
            }
        }
        */

        //m.operation_mgr = Some(op_mgr);
        //mrc.borrow_mut().operation_mgr = Some(op_mgr);

        mrc
    }

    fn test(
        &mut self,
        name : &str,
        new : &Any){
         self.update_changed(name, new);
    }

    /*
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
                        Some(ref mut pp) => 
                            match pp.try_borrow_mut() {
                                Some(ref mut p) => 
                                    unsafe {
                                        //property_data_set(p, mem::transmute(box o.clone()));
                                        p.data_set(mem::transmute(box o.clone()));
                                        p.set_object(&*o.read());
                                        //TODO chris
                                    },
                                None => {}
                            },
                        None => {}
                    };

                    match m.tree {
                        Some(ref mut t) => {
                            t.select(&o.read().id);
                        }
                        _ => {}
                    }
                },
                _ => {},
            }
        }
    }
*/

    /*
    pub fn select(&mut self, id : &Uuid)
    {
        println!(".................select is called : {} ", id);
        self.render.objects_selected.clear();

        for o in self.render.scene.read().objects.iter() {
            if o.read().id == *id {
                self.render.objects_selected.push(o.clone());
                match self.property {
                    Some(ref mut pp) =>
                        match pp.try_borrow_mut() {
                            Some(ref mut p) => {
                                //p.data_set(unsafe {mem::transmute(box o.clone())});
                                p.set_object(&*o.read());
                            },
                            None=> {}
                        },
                    None => {}
                }
                break;
            }
        }

    }
    */

    /*
    pub fn operation_add(&mut self, op : operation::Operation )
    {
        match self.operation_mgr {
            Some(ref mut opmgr) =>
                opmgr.add(op),
                None => {}
        };
    }

    pub fn undo(&mut self)
    {
        match self.operation_mgr {
            Some(ref mut opmgr) =>
                opmgr.undo(),
                None => {
                    println!("no operation manager.............");
                }
        };

    }
    */
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
            //mem::transmute(box master_rc.clone()), //ptr::null(),//TODO
            mem::transmute(box control.clone()), //ptr::null(),//TODO
            mouse_down,
            mouse_up,
            mouse_move,
            mouse_wheel,
            key_down
            );
    }

    master.window = Some(w);

    let mut p = Rc::new(RefCell::new(ui::Property::new(
        w,
        master_rc.clone().downgrade(),
        control.clone())));
    //let mut t = ui::Tree::new(w, master_rc.clone().downgrade());
    let mut t = ui::Tree::new(w, control.clone());

    match (*master).scene {
        Some(ref s) => {
            t.set_scene(&*s.read());
            /*
               let oo = s.read().object_find("yepyoyo");
               match oo {
               Some(o) => { 
            //property_data_set(p, mem::transmute(box o.clone()));
            p.data_set(mem::transmute(box o.clone()));
            p.set_object(&*o.read());
            }
            None => {}
            };
                */
        },
        None => {}
    };

    match control.try_borrow_mut() {
        Some(ref mut c) => {
            c.property = Some(p.clone())
        },
        None => {}
    };

    master.tree = Some(t);
    master.property = Some(p);
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

fn _rotate_camera(control : &mut Control, x : f64, y : f64)
{
  let mut camera = control.camera.borrow_mut();
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

  control.state = CameraRotation;

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
    println!("extern fn mouse up");
    //let master_rc : &Rc<RefCell<Master>> = unsafe {mem::transmute(data)};
    let control_rc : &Rc<RefCell<Control>> = unsafe {mem::transmute(data)};

    //let mut m = master_rc.borrow_mut();
    let mut c = control_rc.borrow_mut();
    c.mouse_up(button,x,y,timestamp);

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
    //let master_rc : &Rc<RefCell<Master>> = unsafe {mem::transmute(data)};
    //let mut m = master_rc.borrow_mut();

    let control_rc : &Rc<RefCell<Control>> = unsafe {mem::transmute(data)};
    let mut c = control_rc.borrow_mut();

    let x : f64 = curx as f64 - prevx as f64;
    let y : f64 = cury as f64 - prevy as f64;
    _rotate_camera(&mut *c, x, y);
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
    //let master_rc : &Rc<RefCell<Master>> = unsafe {mem::transmute(data)};
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
            //TODO now
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

pub trait WidgetUpdate {

    //fn update_changed<T : Any+Clone>(
    fn update_changed(
        &mut self,
        name : &str,
        //old : Option<&T>,
        new : &Any);
}

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

pub struct Control
{
    pub op_mgr : operation::OperationManager,
    pub camera : Rc<RefCell<camera::Camera>>,
    pub state : MasterState,
    pub context : Rc<RefCell<Context>>,

    //TODO control listener
    //pub property : Option<Rc<RefCell<ui::Property>>>, //TODO change to weak
    pub property : Option<Rc<RefCell<Box<ui::Property>>>>, //TODO change to weak
    pub tree : Option<Rc<RefCell<ui::Tree>>>, //TODO change to weak
}

pub struct Context
{
    pub selected : DList<Arc<RWLock<object::Object>>>,
    pub scene : Option<Arc<RWLock<scene::Scene>>>,
}

impl Context
{
    //pub fn new(scene : Arc<RWLock<scene::Scene>>) -> Context
    pub fn new() -> Context
    {
        Context {
            selected: DList::new(),
            scene : None
        }
    }
}

impl Control
{
    pub fn new(
        camera : Rc<RefCell<camera::Camera>>,
        context : Rc<RefCell<Context>>,
        ) -> Control
    {
        Control {
            op_mgr : operation::OperationManager::new(),
            camera : camera,
            property : None,
            tree : None,
            state : Idle,
            context : context
        }
    }

    pub fn mouse_up(
            &mut self, 
            button : i32,
            x : i32, 
            y : i32,
            timestamp : i32)
    {
        println!("control fn mouse up");
        match self.state {
            CameraRotation => {
                self.state = Idle;
                return;
            },
            _ => {}
        }

        println!("control fn mouse up 00");

        //println!("rust mouse up button {}, pos: {}, {}", button, x, y);
        let r = match self.camera.try_borrow(){
            Some(c) => {
                c.ray_from_screen(x as f64, y as f64, 10000f64)
            },
            None => { println!("cannot borrow camera"); return; }
        };

        println!("control fn mouse up 10");

        //TODO
        /*
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
        */

        let mut c = match self.context.try_borrow_mut(){
            Some(con) => con,
            None => { println!("cannot borrow context"); return; }
        };

        c.selected.clear();

        let scene = match c.scene {
            Some(ref s) => s.clone(),
            None => return
        };

        println!("control fn mouse up 50");

        for o in scene.read().objects.iter() {
            let ir = intersection::ray_object(&r, &*o.read());
            if ir.hit {
                println!(" I hit object {} ", o.read().name);
                c.selected.push(o.clone());
            }
        }

        if c.selected.len() == 1 {
            //TODO select tree
            match (c.selected.front()) {
                Some(o) => {
                    match self.property {
                        Some(ref pp) => {
                            match pp.try_borrow_mut() {
                                Some(ref mut p) => unsafe {
                                    p.set_object(&*o.read());
                                },
                                None => {println!("cannot borrow property");}
                            };
                        },
                        None => {
                            println!("control fn mouse up 100");
                        }
                    }

                    match self.tree {
                        Some(ref tt) => {
                            match tt.try_borrow_mut() {
                                Some(ref mut t) => {
                                    t.select(&o.read().id);
                                }
                                _ => {}
                            }
                        },
                        None => {
                            println!("control fn mouse up 200");
                        }
                    }
                },
                _ => {},
            }
        }
    }

    pub fn select(&mut self, id : &Uuid)
    {
        //TODO same as the code at the end of mouse_up, so factorize
        println!("control .................select is called : {} ", id);
        let mut c = match self.context.try_borrow_mut(){
            Some(con) => con,
            None => { println!("cannot borrow context"); return; }
        };

        c.selected.clear();

        let scene = match c.scene {
            Some(ref s) => s.clone(),
            None => return
        };

        for o in scene.read().objects.iter() {
            if o.read().id == *id {
                c.selected.push(o.clone());
                match self.property {
                    Some(ref mut pp) =>
                        match pp.try_borrow_mut() {
                            Some(ref mut p) => {
                                p.set_object(&*o.read());
                            },
                            None=> {}
                        },
                    None => {}
                }
                break;
            }
        }
    }

    pub fn request_operation(
        &mut self,  
        name : Vec<String>,
        old : Box<Any>,
        new : Box<Any>)
    {
        println!("request operation : {}", name);
        let mut c = match self.context.try_borrow_mut(){
            Some(con) => con,
            None => { println!("cannot borrow context"); return; }
        };

        let o = match c.selected.front() {
            Some(o) => o.clone(),
            None => {
                println!("no objetcs selected");
                return;
            }
        };

        //let mut name = name;//.clone();
        //name.insert(0, "object".to_string());

        println!("after request operation : {}", name);

        let op = operation::Operation::new(
            o.clone(), 
            name,//path.to_string(),  
            old,//data, //o.write().cget_property_hier(vs),
            new); //data);

        op.apply();

        self.op_mgr.add(op);
    }

    pub fn request_direct_change(
        &mut self,  
        name : Vec<String>,
        new : &Any)
    {
        let mut c = match self.context.try_borrow_mut(){
            Some(con) => con,
            None => { println!("cannot borrow context"); return; }
        };

        let o = match c.selected.front() {
            Some(o) => o.clone(),
            None => {
                println!("no objetcs selected");
                return;
            }
        };

        let vs = name.tail().to_vec();

        o.write().cset_property_hier(vs, new);
    }

    pub fn get_selected_object(&self) -> Option<Arc<RWLock<object::Object>>>
    {
        let mut c = match self.context.try_borrow_mut(){
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

    //TODO move this out of control?
    pub fn request_display_property(
        &self,
        property : &mut Property,
        name : Vec<String>,
        )
    {
        //TODO remove property tail
        let yep = name.tail().to_vec();

        match self.get_selected_object() {
            Some(o) => {
                match property::find_property(&*o.read(), yep.clone()) {
                    Some(ppp) => {
                        property.create_entries(&*ppp, name.clone());
                    },
                    None => {
                        println!("could not find property {} ", name);
                    }
                }
            },
            None => {
                println!("no objetcs selected");
            }
        }
    }

    pub fn undo(&mut self)
    {
        let op = match self.op_mgr.pop_undo() {
            Some(o) => o,
            None => return
        };

        op.undo();

        match self.property {
            Some(ref mut pp) =>
                match pp.try_borrow_mut() {
                    Some(ref mut p) => {
                        let s = join_string(&op.name);
                        println!("join string : {}", s);
                        p.update_changed(s.as_slice(), &*op.old);
                    },
                    None=> {}
                },
                None => {}
        };

        /*
        match self.master {
            Some(ref masterr) => {
                match masterr.upgrade() {
                    Some(m) => { 
                        match m.try_borrow_mut() {
                            Some(ref mut mm) => {
                                let s = join_string(&op.name);
                                mm.update_changed(s.as_slice(), &*op.new);
                                //mm.update_changed(s.as_slice(), &1f32);
                            },
                            _ => { println!("already borrowed : operation undo")}
                        }
                    },
                    None => { println!("the master of the operation doesn't exist anymore");}
                }
            },
            None => {
                println!("no master !!!!!!!!");
            }
        }
        */
    }
}

fn join_string(path : &Vec<String>) -> String
{
    let mut s = String::new();
    let mut first = true;
    for v in path.iter() {
        if !first {
            s.push('/');
        }
        s.push_str(v.as_slice());
        first = false;
    }

    s
}

