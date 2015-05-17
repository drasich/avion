use std::rc::Rc;
use std::cell::{RefCell, BorrowState};
use std::sync::{RwLock, Arc};
use libc::{c_char, c_void, c_int, c_float};
use std::mem;
use std::ffi;
use std::ffi::CStr;
use std::ffi::CString;
use std::str;
use std::ptr;
use object;
use mesh;
use shader;
use transform;

use ui;
use render;
use render::{Render, GameRender};
use factory;
use context;
use resource;
use resource::Create;
use mesh_render;
use vec;
use geometry;
use material;
use dragger;
use camera;
use operation;
use intersection;

use control;
use control::Control;
use control::WidgetUpdate;

use scene;

/*
#[link(name = "cypher")]
extern {
    pub fn draw_callback_set(
        init_cb: extern fn(*mut View),// -> (),
        draw_cb: extern fn(*mut View), // -> (),
        resize_cb: extern fn(*mut View, w : c_int, h : c_int) -> (),
        render: *const View
        ) -> ();
}
*/


#[link(name = "joker")]
extern {
    pub fn window_rect_visible_set(win :*const ui::Window, b : bool);
    pub fn window_rect_set(
        win :*const ui::Window,
        x : c_float,
        y : c_float,
        w : c_float,
        h : c_float);
}

pub struct Holder
{
    pub gameview : Option<Box<GameView>>
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
    pub action : Option<Rc<RefCell<Box<ui::Action>>>>,
    pub command : Option<Rc<RefCell<Box<ui::Command>>>>,

    //pub dragger : Arc<RwLock<object::Object>>,
    pub dragger : Rc<RefCell<dragger::DraggerManager>>,

    pub camera : Rc<RefCell<camera::Camera>>,
    pub holder : Rc<RefCell<Holder>>,
    pub resource : Rc<resource::ResourceGroup>,
}

impl View
{
    pub fn new(factory: &factory::Factory, resource : Rc<resource::ResourceGroup>) -> View
    //pub fn new(factory: Rc<RefCell<factory::Factory>>) -> View
    {
        //let factory = factory.borrow_mut();

        let scene_path = "scene/simple.scene";
        let mut ss = scene::Scene::new_from_file(scene_path);
        if let None = ss.camera {
            let mut cam = factory.create_camera();
            cam.pan(&vec::Vec3::new(-100f64,20f64,100f64));
            cam.lookat(vec::Vec3::new(0f64,5f64,0f64));
            ss.camera = Some(Rc::new(RefCell::new(cam)));
        }
        let scene = Rc::new(RefCell::new(ss));


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

        let render = box Render::new(factory, resource.clone(), camera.clone());

        let v = View {
            render : render,
            control : control,
            context : context,
            
            window : None,
            tree : None,
            property: None,
            action : None,
            command : None,

            dragger : dragger,

            camera : camera,
            holder : Rc::new(RefCell::new(Holder { gameview : None })),
            resource : resource
        };

        return v;
    }

    pub fn init(&mut self) -> () {
        let w = unsafe {ui::window_new()};
        self.window = Some(w);

        let control = &self.control;

        let p = Rc::new(RefCell::new(ui::Property::new(
                    w,
                    control.clone())));

        let t = Rc::new(RefCell::new(ui::Tree::new(
                    w,
                    control.clone())));

        let a = Rc::new(RefCell::new(ui::Action::new(
                    w)));

        let command = Rc::new(RefCell::new(ui::Command::new(
                    w)));

        match control.borrow_state() {
            BorrowState::Unused => {
                let mut c = control.borrow_mut();
                //c.property = Some(p.clone());
                c.tree = Some(t.clone());
            },
            _ => {}
        };

        println!("TODO must free this in c");
        let tsd = ui::tree::TreeSelectData {
            tree : t.clone(),
            property : p.clone(),
            control : control.clone()
        };
        let ad = ui::action::ActionData::new(
            t.clone(),
            p.clone(),
            control.clone(),
            self.holder.clone(),
            self.resource.clone()
        );

        a.borrow().add_button("christest", ui::action::add_empty, ad.clone());
        a.borrow().add_button(
            "play_scene",
            ui::action::play_scene,
            ad.clone());

        let cd = ui::command::CommandData::new(
            t.clone(),
            p.clone(),
            control.clone(),
            self.holder.clone()
        );
        command.borrow().add("add empty", ui::command::add_empty, cd.clone());
        command.borrow().add("remove selected", ui::command::remove_selected, cd.clone());
        command.borrow().add("set scene camera", ui::command::set_scene_camera, cd.clone());

        {
            let tree = t.borrow();
            unsafe {
                ui::tree::tree_register_cb(
                    tree.jk_tree,
                    mem::transmute(box tsd),
                    ui::tree::name_get,
                    ui::tree::item_selected,
                    ui::tree::can_expand,
                    ui::tree::expand,
                    ui::tree::selected,
                    ui::tree::unselected
                    );
            }
        }

        match self.context.borrow().scene {
            Some(ref s) => {
                t.borrow_mut().set_scene(&*s.borrow());
            },
            None => {}
        };

        self.tree = Some(t);
        self.property = Some(p);
        self.action = Some(a);
        self.command = Some(command);

    }

    fn init_render(&mut self)
    {
        self.render.init();
    }

    fn draw(&mut self)
    {
        let context = self.context.borrow();

        let scene = match context.scene {
            Some(ref s) => s.borrow(),
            None => return
        };

        let obs = &scene.objects;
        let sel = &context.selected;

        let mut center = vec::Vec3::zero();
        let mut ori = vec::Quat::identity();
        for o in sel.iter() {
            center = center + o.read().unwrap().world_position();
            ori = ori * o.read().unwrap().world_orientation();
        }

        if sel.len() > 0 {
            center = center / (sel.len() as f64);

            //TODO println!("remove this code from here, put in update or when moving the camera");
            let mut dragger = self.dragger.borrow_mut();
            dragger.set_position(center);
            dragger.set_orientation(transform::Orientation::Quat(ori), &*self.camera.borrow());
            //let scale = self.camera.borrow().get_camera_resize_w(0.05f64);
            //dragger.set_scale(scale);
            dragger.scale_to_camera(&*self.camera.borrow());
        }

        self.render.draw(obs, sel, &self.dragger.borrow().get_objects());
    }

    fn resize(&mut self, w : c_int, h : c_int)
    {
        self.render.resize(w, h);
    }

    fn get_selected_object(&self) -> Option<Arc<RwLock<object::Object>>>
    {
        let c = match self.context.borrow_state(){
            BorrowState::Writing => { println!("cannot borrow context"); return None; }
            _ => self.context.borrow(),
        };

        match c.selected.front() {
            Some(o) => return Some(o.clone()),
            None => {
                println!("view get selected objects, no objects selected");
                return None;
            }
        };
    }

    fn handle_direct_change(&self, s: &str)
    {
        let o = match self.get_selected_object() {
            Some(ob) => ob,
            None => {
                println!("direct change, no objetcs selected");
                return;
            }
        };

        println!("we have a direct change: {}", s);

        if s == "object/name" {
            match self.tree {
                Some(ref t) =>
                    match t.borrow_state() {
                        BorrowState::Writing => {},
                        _ => {
                            t.borrow().update_object(&o.read().unwrap().id);
                        },
                    },
                    None => {}
            };
        }
            
        match self.property {
            Some(ref p) =>
                match p.borrow_state() {
                    BorrowState::Writing => {},
                    _ => {
                        println!("direct change : {}", s);
                        //p.update_object(&*o.read().unwrap(), s);
                        p.borrow().update_object_property(&*o.read().unwrap(), s);
                    },
                },
                None => {}
        };
    }


    pub fn handle_control_change(&self, change : &operation::Change)
    {
        if *change == operation::Change::None {
            return;
        }

        let sel = self.get_selected_object();

        match *change {
            operation::Change::Objects(ref name, ref id_list) => {

                for id in id_list.iter() {
                    if let Some(ref o) = sel {
                        if *id == o.read().unwrap().id  {
                            match self.property.clone() {
                                Some(ref p) =>
                                    match p.borrow_state() {
                                        BorrowState::Unused => {
                                            p.borrow_mut().update_object(&*o.read().unwrap(), "");

                                        },
                                        _=> {}
                                    },
                                    None => {}
                            };
                        }
                    }

                    if name == "object/name" {
                        match self.tree.clone() {
                            Some(ref mut t) =>
                                match t.borrow_state() {
                                    BorrowState::Unused => {
                                        t.borrow_mut().update_object(id);
                                    },
                                    _ => {}
                                },
                                None => {}
                        };
                    }
                }
            },
            operation::Change::DirectChange(ref name) => {
                self.handle_direct_change(name.as_ref());
            },
            operation::Change::RectVisibleSet(b) => {
                if let Some(w) = self.window {
                    unsafe {
                        window_rect_visible_set(w, b);
                    }
                }
            },
            operation::Change::RectSet(x,y,w,h) => {
                if let Some(win) = self.window {
                    unsafe {
                        window_rect_set(win, x,y,w,h);
                    }
                }
            },
            operation::Change::SelectedChange => {

                let c = match self.context.borrow_state(){
                    BorrowState::Writing => { println!("cannot borrow context"); return; }
                    _ => self.context.borrow(),
                };

                println!("object seclected : {}",  c.selected.len());

                if c.selected.len() != 1 {
                    match self.property {
                        Some(ref p) => {
                            match p.borrow_state() {
                                BorrowState::Unused => {
                                    p.borrow_mut().set_nothing();
                                },
                                _ => {println!("cannot borrow property");}
                            };
                        },
                        None => {
                            println!("control no property");
                        }
                    }

                }
                else {
                    match c.selected.front() {
                        Some(o) => {
                            match self.property {
                                Some(ref p) => {
                                    match p.borrow_state() {
                                        BorrowState::Unused => {
                                            p.borrow_mut().set_object(&*o.read().unwrap());
                                        },
                                        _ => {println!("cannot borrow property");}
                                    };
                                },
                                None => {
                                    println!("control no property");
                                }
                            }
                        },
                        _ => {},
                    }
                }

                match self.tree {
                    Some(ref t) => {
                        match t.borrow_state() {
                            BorrowState::Unused => {
                                println!("view, select objects");
                                t.borrow_mut().select_objects(c.get_vec_selected_ids());
                            }
                            _ => {
                                println!("view, selectchange!!!!!!!!, tree already borrowed");
                            }
                        }
                    },
                    None => {
                        println!("control no tree");
                    }
                }
            },
            operation::Change::SceneRemove(ref id, ref obs) => {
                {
                    println!("view, sceneremove!!!!!!!!");
                    let mut c = self.context.borrow_mut();
                    c.remove_objects_by_id(obs.clone());

                    match self.tree {
                        Some(ref t) => {
                            match t.borrow_state() {
                                BorrowState::Unused => {
                                    println!("view, sceneremove!!!!!!!! tree remove");
                                    t.borrow_mut().remove_objects_by_id(obs.clone());
                                }
                                _ => {
                                    println!("view, sceneremove!!!!!!!!, tree already borrowed");
                                }
                            }
                        },
                        None => {
                            println!("control no tree");
                        }
                    }
                }
                self.handle_control_change(&operation::Change::SelectedChange);
            },
            operation::Change::SceneAdd(ref id, ref obs) => {
                let c = self.context.borrow();
                let scene = match c.scene {
                    Some(ref s) => s.clone(),
                    None => return
                };

                let objects = scene.borrow().find_objects_by_id(&mut obs.clone());

                // todo
                match self.tree {
                    Some(ref t) => {
                        match t.borrow_state() {
                            BorrowState::Unused => {
                                t.borrow_mut().add_objects(objects);
                            }
                            _ => {}
                        }
                    },
                    None => {
                        println!("control no tree");
                    }
                }
            },
            _ => {}
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

    let op_list = {
        let control_rc = view.control.clone();

        //println!("rust mouse down button {}, pos: {}, {}", button, x, y);
        //let control_rc : &Rc<RefCell<Control>> = unsafe {mem::transmute(data)};
        let mut c = control_rc.borrow_mut();
        c.mouse_down(modifier, button,x,y,timestamp)
    };

    for op in op_list.iter() {
        view.handle_control_change(op);
    }
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

    let change = {
        let control_rc = view.control.clone();
        let mut c = control_rc.borrow_mut();
        c.mouse_up(button,x,y,timestamp)
    };

    view.handle_control_change(&change);
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

    let change_list = {
        //let control_rc : &Rc<RefCell<Control>> = unsafe {mem::transmute(data)};
        let mut c = control_rc.borrow_mut();
        c.mouse_move(modifiers_flag, button, curx, cury, prevx, prevy, timestamp)
    };

    for change in change_list.iter() {
        view.handle_control_change(change);
    }
    
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
            let s = unsafe {CStr::from_ptr(key).to_bytes()};
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
            let s = unsafe {CStr::from_ptr(keynameconst).to_bytes()};
            match str::from_utf8(s) {
                Ok(ss) => ss.to_string(),
                _ => {
                    println!("error");
                    return
                }
            }
        };

        match key_str.as_ref() {
            "Return" => {
                if let Some(ref cmd) = view.command {
                    println!("pressed return show popup");
                    cmd.borrow().show();
                }
                return;
            },
            _ => {
                println!("key not implemented : {}", key_str);
            }
        }

        c.key_down(modifier, keyname_str.as_ref(), key_str.as_ref(), timestamp)
    };

    view.handle_control_change(&change);
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

pub struct GameView
{
    window : *const ui::Evas_Object,
    render : Box<GameRender>,
    scene : Rc<RefCell<scene::Scene>>,
    name : String
}



impl GameView {
    pub fn new(
        //factory: &mut factory::Factory,
        camera : Rc<RefCell<camera::Camera>>,
        scene : Rc<RefCell<scene::Scene>>,
        resource : Rc<resource::ResourceGroup>
        ) -> Box<GameView>
    {
        /*
        let camera = Rc::new(RefCell::new(factory.create_camera()));
        {
            let mut cam = camera.borrow_mut();
            cam.pan(&vec::Vec3::new(100f64,20f64,100f64));
            cam.lookat(vec::Vec3::new(0f64,5f64,0f64));
        }
        */

        let win = unsafe { 
            ui::jk_window_new(gv_close_cb, ptr::null())
        };

        //let render = box GameRender::new(factory, camera);
        let render = box GameRender::new(camera, resource);

        let v = box GameView {
            render : render,
            window : win,
            scene : scene,
            name : "cacayop".to_string()
            //camera : camera todo
        };

        let glview = unsafe { ui::jk_glview_new(
                win, 
                //mem::transmute(&*v.render),
                mem::transmute(&*v),
                gv_init_cb,
                gv_draw_cb,
                gv_resize_cb
                ) };

        return v;
    }

    fn draw(&mut self) {
        self.scene.borrow_mut().update(0.01f64);

        self.render.draw(&self.scene.borrow().objects);
    }

    fn init(&mut self) {
        self.render.init();
    }

    fn resize(&mut self, w : c_int, h : c_int)
    {
        self.render.resize(w, h);
    }
}

pub extern fn gv_init_cb(v : *const c_void) {
    unsafe {
        let gv : *mut GameView = mem::transmute(v);
        //println!("AAAAAAAAAAAAAAAAAAAAAA gv init cb {}", (*gv).name);
        (*gv).init();
    }
}

pub extern fn gv_draw_cb(v : *const c_void) {
    unsafe {
        let gv : *mut GameView = mem::transmute(v);
        //println!("draw {}", (*gv).name);
        (*gv).draw();
    }
}

pub extern fn gv_resize_cb(v : *const c_void, w : c_int, h : c_int) {
    unsafe {
        //return (*v).resize(w, h);
        let gv : *mut GameView = mem::transmute(v);
        //println!("resize {}", (*gv).name);
        (*gv).resize(w, h);
    }
}

pub extern fn gv_close_cb(v : *mut c_void) {
    println!("close cb");
}

