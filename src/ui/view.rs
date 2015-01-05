use std::rc::Rc;
use std::cell::RefCell;
use std::sync::{RWLock, Arc};
use libc::{c_char, c_void, c_int};
use std::mem;
use std::c_str::CString;
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

use control;
use control::Control;
use control::WidgetUpdate;

use scene;

pub struct View
{
    render : Box<Render>,
    pub control : Option<Rc<RefCell<Control>>>,

    pub window : Option<*const ui::Window>,
    //pub tree : Option<Box<Tree>>,
    pub tree : Option<Rc<RefCell<Box<ui::Tree>>>>,
    pub property : Option<Rc<RefCell<Box<ui::Property>>>>,
    pub scene : Option<Arc<RWLock<scene::Scene>>>,

    pub dragger : Arc<RWLock<object::Object>>,
}

impl View
{
    pub fn new(factory: &mut factory::Factory) -> View
    {
        //let dragger = Arc::new(RWLock::new(create_dragger(factory)));
        let dragger = create_dragger(factory);

        let context = Rc::new(RefCell::new(context::Context::new()));
        let render = box Render::new(factory, context.clone(), dragger.clone());
        let control = Rc::new(RefCell::new(
                Control::new(
                    render.camera.clone(),
                    context.clone()
                    )
                )
            );

        control.borrow_mut().context.borrow_mut().scene = Some(render.scene.clone());

        let mut v = View {
            render : render,
            control : Some(control),
            
            window : None,
            tree : None,
            property: None,

            scene : None,
            dragger : dragger
        };

        v.scene = Some(v.render.scene.clone());

        unsafe {
        render::draw_callback_set(
            render::init_cb,
            render::draw_cb,
            render::resize_cb,
            //&m.render);
            &*v.render);
        }

        return v;

    }

    pub fn init(&mut self) -> () {
        let w = unsafe {ui::window_new()};
        self.window = Some(w);

        let control = match self.control {
            Some(ref c) => c.clone(),
            None => { println!("no control"); return; }
        };   

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

        let p = Rc::new(RefCell::new(ui::Property::new(
                    w,
                    control.clone())));

        let t = Rc::new(RefCell::new(ui::Tree::new(
                    w,
                    control.clone())));

        match self.scene {
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
    println!("view control fn mouse up");
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

    let key_str = {
        let s = unsafe {CString::new(key as *const i8, false) };
        match s.as_str() {
            Some(ss) => ss.to_string(),
            _ => return
        }
    };

    let keyname_str = {
        let s = unsafe {CString::new(keyname as *const i8, false) };
        match s.as_str() {
            Some(ss) => ss.to_string(),
            _ => return
        }
    };

    c.key_down(modifier, keyname_str.as_slice(), key_str.as_slice(), timestamp);
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

fn create_mat_res(color : vec::Vec4, name : &str) -> resource::ResTT<material::Material>
{
    let mut mat : material::Material = Create::create("material/dragger.mat");
    mat.inittt();
    mat.set_uniform_data(
        "color",
        shader::UniformData::Vec4(color));
    let matarc = Arc::new(RWLock::new(mat));

    let rs = resource::ResTest::ResData(matarc);
    let mr = resource::ResTT::new_with_res("dragger_x_mat", rs);
    //let mr = resource::ResTT::new_with_res(name, rs);

    mr
}

fn create_dragger_tr(
    factory : &mut factory::Factory,
    name : &str,
    ori :vec::Quat,
    color : vec::Vec4) -> Arc<RWLock<object::Object>>
{
    let mut dragger = 
        Arc::new(RWLock::new(factory.create_object("dragger_x")));
    let mat = create_mat_res(color, name);

    dragger.write().unwrap().mesh_render = 
        Some(mesh_render::MeshRender::new_with_mat(
        "model/dragger_arrow.mesh", mat));

    dragger.write().unwrap().orientation = transform::Orientation::Quat(ori);

    dragger
}

fn create_dragger(factory : &mut factory::Factory) -> 
        Arc<RWLock<object::Object>>
{
    let dragger_parent = 
        Arc::new(RWLock::new(factory.create_object("dragger")));

    let dragger_x = create_dragger_tr(
        factory,
        "dragger_x",
        vec::Quat::identity(), 
        vec::Vec4::new(1f64, 0f64, 1f64, 1f64));

    let dragger_y = create_dragger_tr(
        factory,
        "dragger_y",
        vec::Quat::new_axis_angle(vec::Vec3::new(1f64,0f64,0f64), 90f64), 
        vec::Vec4::new(0f64, 1f64, 0f64, 1f64));

    object::child_add(dragger_parent.clone(), dragger_x);
    object::child_add(dragger_parent.clone(), dragger_y);

    dragger_parent
}
