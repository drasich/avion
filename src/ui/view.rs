use std::rc::Rc;
use std::cell::{Cell,RefCell, BorrowState};
use std::sync::{RwLock, Arc,Mutex};
use libc::{c_char, c_void, c_int, c_uint, c_float};
use std::collections::{LinkedList};
use std::mem;
use std::ffi;
use std::ffi::CStr;
use std::ffi::CString;
use std::str;
use std::ptr;
use uuid;
use dormin::object;
use dormin::mesh;
use dormin::shader;
use dormin::transform;

use ui;
use dormin::render;
use dormin::render::{Render, GameRender};
use dormin::factory;
use context;
use dormin::resource;
use dormin::resource::Create;
use dormin::vec;
use dormin::geometry;
use dormin::material;
use dragger;
use dormin::camera;
use operation;
use dormin::intersection;
use control;
use control::Control;
use control::WidgetUpdate;
use dormin::scene;
use dormin::component;
use util;
use dormin::input;

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

   pub fn jk_glview_request_update(glview : *const ui::JkGlview);
}

pub struct View
{
    render : Box<Render>,
    pub control : Rc<RefCell<Control>>,

    pub window : Option<*const ui::Window>,

    //pub dragger : Arc<RwLock<object::Object>>,
    dragger : Rc<RefCell<dragger::DraggerManager>>,

    pub camera : Rc<RefCell<camera::Camera>>,
    pub resource : Rc<resource::ResourceGroup>,
    pub uuid : uuid::Uuid,

    pub width : i32,
    pub height : i32,
    pub updating : Cell<bool>,
    pub loading_resource : Arc<Mutex<usize>>,
    pub last_draw : usize
}

impl View
{
    pub fn new(
        resource : Rc<resource::ResourceGroup>,
        container : &mut Box<ui::WidgetContainer>,
        w : i32,
        h : i32,
        camera : camera::Camera
        ) -> View
    {
        /*
        let camera = Rc::new(RefCell::new(container.factory.create_camera()));
        {
            let mut cam = camera.borrow_mut();
            cam.pan(&vec::Vec3::new(100f64,20f64,100f64));
            cam.lookat(vec::Vec3::new(0f64,5f64,0f64));
        }
        */
        let camera = Rc::new(RefCell::new(camera));


        let dragger = Rc::new(RefCell::new(dragger::DraggerManager::new(&container.factory, &*resource)));

        let control = Rc::new(RefCell::new(
                Control::new(
                    camera.clone(),
                    dragger.clone()
                    )));

        let render = box Render::new(&container.factory, resource.clone(), camera.clone());

        let v = View {
            render : render,
            control : control,

            window : None,

            dragger : dragger,

            camera : camera,
            resource : resource,
            uuid : uuid::Uuid::new_v4(),

            width : w,
            height : h,
            updating : Cell::new(false),
            loading_resource : Arc::new(Mutex::new(0)),
            last_draw : 0
        };

        return v;
    }

    pub fn init(
        &mut self,
        container : &mut Box<ui::WidgetContainer>,
        property_config : &ui::WidgetPanelConfig,
        tree_config : &ui::WidgetConfig,
        ) -> () {
        let w = unsafe {ui::window_new(self.width,self.height)};
        self.window = Some(w);

        //let p = Rc::new(ui::PropertyList::new(w, property_config));
        container.panel.config = property_config.clone();
        container.panel.create(w);

        let p = Rc::new(ui::PropertyBox::new(&*container.panel));//, property_config));
        let mut t = box ui::Tree::new(w, tree_config);

        container.list.create(w);

        let mut menu = box ui::Action::new(w, ui::action::Position::Top, self.uuid);

        let a = box ui::Action::new(w, ui::action::Position::Bottom, self.uuid);
        let command = box ui::Command::new(w);

        let tsd = ui::WidgetCbData::with_ptr(container, unsafe { mem::transmute(&*t)});
        //let pd = ui::WidgetCbData::with_ptr(container, unsafe { mem::transmute(&*p)});
        let pd = ui::WidgetCbData::new_with_widget(container, p.clone());
        let ad = ui::WidgetCbData::with_ptr(container, unsafe { mem::transmute(&*a)});

        a.add_button("new scene", ui::action::scene_new, ad.clone());
        a.add_button("add empty", ui::action::add_empty, ad.clone());
        a.add_button(
            "open game view",
            ui::action::open_game_view,
            ad.clone());
        a.add_button(
            "pause",
            ui::action::pause_scene,
            ad.clone());
        a.add_button(
            "play",
            ui::action::play_scene,
            ad.clone());

        a.add_button(
            "compile_test",
            ui::action::compile_test,
            ad.clone());

        unsafe {
            ui::tree::tree_register_cb(
                t.jk_tree,
                mem::transmute(box tsd),
                ui::tree::name_get,
                ui::tree::item_selected,
                ui::tree::can_expand,
                ui::tree::expand,
                ui::tree::selected,
                ui::tree::unselected,
                ui::tree::panel_move,
                );
        }

        unsafe {
            ui::property::jk_property_cb_register(
                ui::property_box::property_box_cb_get(p.jk_property),
                mem::transmute(box pd),
                ui::property_list::changed_set_float,
                ui::property_list::changed_set_string,
                ui::property_list::changed_set_enum,
                ui::property_list::register_change_string,
                ui::property_list::register_change_float,
                ui::property_list::register_change_enum,
                ui::property_list::register_change_option,
                ui::property_list::expand,
                ui::property_list::contract,
                ui::property::vec_add,
                ui::property::vec_del);
        }

        let name = match container.context.scene {
            Some(ref s) => {
                //t.borrow_mut().set_scene(&*s.borrow());
                let sb = &*s.borrow();
                //menu.add_label(&sb.name);
                t.set_scene(sb);
                sb.name.clone()
            },
            None => {
                String::from("none")
            }
        };

        menu.add_button(">", ui::action::scene_list, ad.clone());
        menu.add_entry(String::from("scene"),&name, ui::action::scene_rename, ad.clone());
        menu.add_button("+", ui::action::scene_new, ad.clone());

        container.tree = Some(t);
        container.property = Some(p.clone());
        container.panel.widget = Some(p);
        container.action = Some(a);
        container.command = Some(command);
        container.menu = Some(menu);

        //container.list.create(w);

    }

    fn init_render(&mut self)
    {
        self.render.init();
    }

    fn draw(&mut self, context : &context::Context) -> bool
    {
        let scene = match context.scene {
            Some(ref s) => s.borrow(),
            None => return false
        };

        let obs = &scene.objects;
        let sel = &context.selected;

        let mut center = vec::Vec3::zero();
        let mut ori = vec::Quat::identity();
        for o in sel {
            center = center + o.read().unwrap().world_position();
            ori = ori * o.read().unwrap().world_orientation();
        }

        if !sel.is_empty() {
            center = center / (sel.len() as f64);

            //TODO println!("remove this code from here, put in update or when moving the camera");
            let mut dragger = self.dragger.borrow_mut();
            dragger.set_position(center);
            dragger.set_orientation(transform::Orientation::Quat(ori), &*self.camera.borrow());
            //let scale = self.camera.borrow().get_camera_resize_w(0.05f64);
            //dragger.set_scale(scale);
            dragger.scale_to_camera(&*self.camera.borrow());
        }

        let win = if let Some(w) = self.window {
            w
        }
        else {
            ptr::null()
        };

        let finish = |b| {
            //println!("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~render finished");
        };

        let not_loaded = self.render.draw(obs, sel, &self.dragger.borrow().get_objects(), &finish, self.loading_resource.clone());
        self.updating.set(false);
        self.last_draw = *self.loading_resource.lock().unwrap();

        not_loaded > 0
    }

    fn resize(&mut self, w : c_int, h : c_int)
    {
        self.width = w;
        self.height = h;
        self.render.resize(w, h);
    }

    pub fn handle_control_change(&self, change : &operation::Change)
    {
        match *change {
            operation::Change::DirectChange(ref name) => {
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
            },
            _ => {}
        }
    }

    pub fn get_camera_transform(&self) -> (vec::Vec3, vec::Quat)
    {
        let c = self.camera.borrow();
        let c = c.object.read().unwrap();
        (c.position, c.orientation.as_quat())
    }

    pub fn request_update(&self)
    {
        if self.updating.get() {
            return;
        }

        if let Some(w) = self.window {
            self.updating.set(true);
            unsafe {ui::jk_window_request_update(w);}
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
    //let view : &Box<View> = unsafe {mem::transmute(data)};
    let wcb : & ui::WidgetCbData = unsafe {mem::transmute(data)};
    let view : &View = unsafe {mem::transmute(wcb.widget)};
    let container : &mut Box<ui::WidgetContainer> = unsafe {mem::transmute(wcb.container)};

    let op_list = {
        let control_rc = view.control.clone();

        //println!("rust mouse down button {}, pos: {}, {}", button, x, y);
        //let control_rc : &Rc<RefCell<Control>> = unsafe {mem::transmute(data)};
        let mut c = control_rc.borrow_mut();
        c.mouse_down(&*container.context, modifier, button,x,y,timestamp)
    };

    for op in &op_list {
        if let operation::Change::DraggerClicked = *op {
            let c = &mut container.context;;
            c.save_positions();
            c.save_scales();
            c.save_oris();
        }
        view.handle_control_change(op);
        container.handle_change(op, view.uuid);
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
    //let view : &Box<View> = unsafe {mem::transmute(data)};
    let wcb : & ui::WidgetCbData = unsafe {mem::transmute(data)};
    let view : &View = unsafe {mem::transmute(wcb.widget)};
    let container : &mut Box<ui::WidgetContainer> = unsafe {mem::transmute(wcb.container)};

    let change = {
        let control_rc = view.control.clone();
        let mut c = control_rc.borrow_mut();
        c.mouse_up(&*container.context,button,x,y,timestamp)
    };

    view.handle_control_change(&change);
    container.handle_change(&change, view.uuid);
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
    //let view : &Box<View> = unsafe {mem::transmute(data)};
    let wcb : & ui::WidgetCbData = unsafe {mem::transmute(data)};
    let container : &mut Box<ui::WidgetContainer> = unsafe {mem::transmute(wcb.container)};
    let view : &View = unsafe {mem::transmute(wcb.widget)};
    let control_rc = view.control.clone();

    let change_list = {
        //let control_rc : &Rc<RefCell<Control>> = unsafe {mem::transmute(data)};
        let mut c = control_rc.borrow_mut();
        c.mouse_move(
            &*container.context,
            modifiers_flag,
            button,
            curx,
            cury,
            prevx,
            prevy,
            timestamp)
    };

    for change in &change_list {
        view.handle_control_change(change);
        container.handle_change(change, view.uuid);
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
    //let view : &Box<View> = unsafe {mem::transmute(data)};
    let wcb : & ui::WidgetCbData = unsafe {mem::transmute(data)};
    let view : &View = unsafe {mem::transmute(wcb.widget)};
    let control_rc = view.control.clone();

    //let control_rc : &Rc<RefCell<Control>> = unsafe {mem::transmute(data)};
    let c = control_rc.borrow_mut();
    c.mouse_wheel(modifiers_flag, direction, z, x, y, timestamp);

    view.request_update();
}

pub extern fn key_down(
    data : *const c_void,
    modifier : c_int,
    keyname : *const c_char,
    key : *const c_char,
    keycode : c_uint,
    timestamp : c_int
    )
{
    //let view : &Box<View> = unsafe {mem::transmute(data)};
    let wcb : & ui::WidgetCbData = unsafe {mem::transmute(data)};
    let view : &View = unsafe {mem::transmute(wcb.widget)};
    let container : &mut Box<ui::WidgetContainer> = unsafe {mem::transmute(wcb.container)};

    let change = {
        //let control_rc = view.control.clone();
        //let control_rc : &Rc<RefCell<Control>> = unsafe {mem::transmute(data)};
        //let mut c = control_rc.borrow_mut();

        let key_str = {
            let s = unsafe {CStr::from_ptr(key).to_bytes()};
            match str::from_utf8(s) {
                Ok(ss) => ss.to_owned(),
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
                Ok(ss) => ss.to_owned(),
                _ => {
                    println!("error");
                    return
                }
            }
        };

        match key_str.as_ref() {
            "Return" => {
                if let Some(ref mut cmd) = container.command {
                    println!("pressed return show popup");

                    cmd.clean();

                    let scene_actions : &[(&str, extern fn(*const c_void, *const c_char))]
                    = &[
                    ("add empty", ui::command::add_empty),
                    ("remove selected22", ui::command::remove_selected2),
                    ("set camera2", ui::command::set_camera2),
                    ("add component", ui::command::add_component),
                    ("copy selected", ui::command::copy_selected),
                    ];

                    for a in scene_actions {
                        let (ref name, f) = *a;
                        cmd.add_ptr(name, f, data);
                    }

                    cmd.show();
                }
                return;
            },
            "t" => {
                if let Some(ref mut t) = container.tree {
                    let b = t.visible();
                    t.set_visible(!b);
                }
                else {
                    println!("container does not have a tree");
                }
                return;
            },
            "p" => {
                let p = &mut container.panel;
                let b = p.visible();
                p.set_visible(!b);
                return;
            },
            "a" => {
                if let Some(ref mut a) = container.action {
                    let b = a.visible();
                    a.set_visible(!b);
                }
                return;
            },
            "c" => {
                let center = vec::Vec3::zero();
                let mut cam = view.camera.borrow_mut();
                let pos = center + cam.object.read().unwrap().orientation.rotate_vec3(&vec::Vec3::new(0f64,0f64,100f64));
                cam.set_position(pos);
                cam.set_center(&center);
                view.request_update();
                return;
            },
            "f" => {
                let center = util::objects_center(&container.context.selected);
                let mut cam = view.camera.borrow_mut();
                let pos = center + cam.object.read().unwrap().orientation.rotate_vec3(&vec::Vec3::new(0f64,0f64,100f64));
                cam.set_position(pos);
                view.request_update();
                return;
            },
            _ => {
                println!("key not implemented : {}", key_str);
            }
        }

        {
            let control_rc = view.control.clone();
            let mut c = control_rc.borrow_mut();
            c.key_down(modifier, keyname_str.as_ref(), key_str.as_ref(), timestamp)
        }
    };

    view.handle_control_change(&change);
    container.handle_change(&change, view.uuid);
}


/*
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
*/


pub extern fn init_cb(v : *mut View) -> () {
    let wcb : & ui::WidgetCbData = unsafe {mem::transmute(v)};
    let view : &mut View = unsafe {mem::transmute(wcb.widget)};
    let container : &Box<ui::WidgetContainer> = unsafe {mem::transmute(wcb.container)};

    return view.init_render();
}

pub extern fn request_update_again(data : *const c_void) -> bool
{
    let wcb : & ui::WidgetCbData = unsafe {mem::transmute(data)};
    let view : &mut View = unsafe {mem::transmute(wcb.widget)};
    let container : &Box<ui::WidgetContainer> = unsafe {mem::transmute(wcb.container)};

    if let Ok(lr) = view.loading_resource.try_lock() {
        if *lr == 0 {
            view.request_update();
            return false;
        }
    }
    true
}


pub extern fn draw_cb(v : *mut View) -> () {

    let wcb : & ui::WidgetCbData = unsafe {mem::transmute(v)};
    let view : &mut View = unsafe {mem::transmute(wcb.widget)};
    let container : &Box<ui::WidgetContainer> = unsafe {mem::transmute(wcb.container)};

    let draw_not_done = view.draw(&*container.context);

    if draw_not_done {
        unsafe {
            ui::ecore_animator_add(request_update_again, mem::transmute(wcb));
        }
    }
}

pub extern fn resize_cb(v : *mut View, w : c_int, h : c_int) -> () {
    let wcb : & ui::WidgetCbData = unsafe {mem::transmute(v)};
    let view : &mut View = unsafe {mem::transmute(wcb.widget)};
    let container : &Box<ui::WidgetContainer> = unsafe {mem::transmute(wcb.container)};

    return view.resize(w, h);
}

pub struct GameView
{
    window : *const ui::Evas_Object,
    glview : *const ui::JkGlview,
    render : Box<GameRender>,
    scene : Rc<RefCell<scene::Scene>>,
    name : String,
    pub state : i32,
    input : input::Input,
    pub config : ui::WidgetConfig,
    pub loading_resource : Arc<Mutex<usize>>,
}



impl GameView {
    pub fn new(
        //factory: &mut factory::Factory,
        win : *const ui::Evas_Object,
        camera : Rc<RefCell<camera::Camera>>,
        scene : Rc<RefCell<scene::Scene>>,
        resource : Rc<resource::ResourceGroup>,
        config : ui::WidgetConfig
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

        /*
        let win = unsafe {
            ui::jk_window_new(gv_close_cb, ptr::null())
        };
        */

        //let render = box GameRender::new(factory, camera);
        let render = box GameRender::new(camera, resource);

        let mut v = box GameView {
            render : render,
            window : win,
            scene : scene,
            name : "cacayop".to_owned(),
            state : 0,
            glview : ptr::null(),
            input : input::Input::new(),
            config : config.clone(),
            loading_resource : Arc::new(Mutex::new(0))
            //camera : camera todo
        };


        v.glview = unsafe { ui::jk_glview_new(
                win,
                //mem::transmute(&*v.render),
                mem::transmute(&*v),
                gv_init_cb,
                gv_draw_cb,
                gv_resize_cb,
                gv_key_down,
                ) };

        v.set_visible(config.visible);

        return v;
    }

    pub fn update(&mut self) -> bool {
        if self.state == 1 {
            self.scene.borrow_mut().update(0.01f64, &self.input);
            unsafe { jk_glview_request_update(self.glview); }
            self.input.clear();
            true
        }
        else {
            //unsafe { jk_glview_request_update(self.glview); }
            false
        }
    }
    
    pub fn request_update(&self)
    {
        unsafe { jk_glview_request_update(self.glview); }
    }

    fn draw(&mut self) -> bool
    {
        self.render.draw(&self.scene.borrow().objects, self.loading_resource.clone())
    }

    fn init(&mut self) {
        self.render.init();
    }

    fn resize(&mut self, w : c_int, h : c_int)
    {
        self.render.resize(w, h);
        self.config.w = w;
        self.config.h = h;
    }

    pub fn visible(&self) -> bool
    {
        self.config.visible
    }

    pub fn get_config(&self) -> ui::WidgetConfig
    {
        self.config.clone()
    }

    pub fn set_visible(&mut self, b : bool)
    {
        self.config.visible = b;

        if b {
            unsafe { ui::evas_object_show(self.window); }
        }
        else {
            unsafe { ui::evas_object_hide(self.window); }
        }
    }
}

pub extern fn gv_init_cb(v : *const c_void) {
    unsafe {
        let gv : *mut GameView = mem::transmute(v);
        //println!("AAAAAAAAAAAAAAAAAAAAAA gv init cb {}", (*gv).name);
        (*gv).init();
    }
}

pub extern fn request_update_again_gv(data : *const c_void) -> bool
{
    unsafe {
    //let gv : *mut GameView =  unsafe {mem::transmute(data)};
    let gv : &mut GameView =  unsafe {mem::transmute(data)};

    //if let Ok(lr) = (*gv).loading_resource.try_lock() {
    if let Ok(lr) = gv.loading_resource.try_lock() {
        if *lr == 0 {
            //(*gv).request_update();
            gv.request_update();
            return false;
        }
    }
    }
    true
}


pub extern fn gv_draw_cb(v : *const c_void) {
    unsafe {
        let gv : *mut GameView = mem::transmute(v);
        //println!("draw {}", (*gv).name);
        let draw_not_done = (*gv).draw();

        if draw_not_done && (*gv).state == 0 {
            unsafe {
                ui::ecore_animator_add(request_update_again_gv, mem::transmute(v));
            }
    }
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

pub extern fn gv_close_cb(data : *mut c_void) {
    let container : &mut Box<ui::WidgetContainer> = unsafe {mem::transmute(data)};
    if let Some(ref mut gv) = container.gameview {
        gv.set_visible(false);
    }
}

extern fn gv_key_down(
    data : *const c_void,
    modifier : c_int,
    keyname : *const c_char,
    key : *const c_char,
    keycode : c_uint,
    timestamp : c_int)
{
    let gv : *mut GameView = unsafe { mem::transmute(data) };
    let gv : &mut GameView = unsafe { &mut *gv };
    //unsafe { (*gv).input.add_key(keycode as u8); }
    gv.input.add_key(keycode as u8);
}
