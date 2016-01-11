use libc::{c_char, c_void, c_int, size_t};
use std::mem;
use std::sync::{RwLock, Arc};
use std::collections::{LinkedList};
use std::ptr;
use std::rc::Rc;
use std::cell::{RefCell, BorrowState};
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::any::{Any};//, AnyRefExt};
use std::path::Path;
use std::fs;
use std::fs::File;
use rustc_serialize::{json, Encodable, Encoder, Decoder, Decodable};
use std::io::{Read,Write};
use std::ffi::{CString,CStr};

use uuid::Uuid;

use intersection;
use resource;
use geometry;
use vec;
use scene;
use object;
use ui::{Tree,Property,PropertyConfig,Holder,View,Command,Action};
use ui;
use factory;
use operation;
use camera;
use property;
use context;
use control;
use control::Control;
use control::WidgetUpdate;
use uuid;
use component;
use dragger;
use property::{PropertyWrite,PropertyGet};
use transform;
use util;

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

pub type PanelGeomFunc = extern fn(
    object : *const c_void,
    x : c_int,
    y : c_int,
    w : c_int,
    h : c_int);

pub type ButtonCallback = extern fn(
    data : *const c_void);

pub type SelectCallback = extern fn(
    data : *const c_void,
    name : *const c_char);

/*
        init_cb: extern fn(*mut View),// -> (),
        draw_cb: extern fn(*mut View), // -> (),
        resize_cb: extern fn(*mut View, w : c_int, h : c_int) -> (),
        render: *const View
        */

#[link(name = "joker")]
extern {
    pub fn elm_simple_window_main();
    pub fn window_new(w : c_int, h : c_int) -> *const Window;
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

    fn jk_list_wdg_new(win : *const Window, name : *const c_char) -> *const Evas_Object;

    fn jk_list_fn_set(
        o : *const ui::Evas_Object,
        sel_callback: SelectCallback,
        data : *const c_void);

    //fn window_object_get(
    //    obj : *const Window) -> *const Evas_Object;

    fn evas_object_geometry_get(
        obj : *const Evas_Object,
        x : *mut c_int,
        y : *mut c_int,
        w : *mut c_int,
        h : *mut c_int);

    fn elm_object_part_text_set(
        obj : *const Evas_Object,
        part : *const c_char,
        text : *const c_char);

    fn evas_object_show(o : *const Evas_Object);
    fn evas_object_move(o : *const Evas_Object, x : c_int, y : c_int);
    fn evas_object_resize(o : *const Evas_Object, w : c_int, h : c_int);


    fn jklist_set_names(o : *const Evas_Object, names : *const c_void, len : size_t);
}

fn object_geometry_get(obj : *const Evas_Object) -> (i32, i32, i32, i32)
{
    let (mut x, mut y, mut w, mut h) : (c_int, c_int, c_int, c_int) = (5,6,7,8);
    //let (mut x, mut y, mut w, mut h) = (5,6,7,8);

    println!("starrrrrrrrrrrrrrrrrrrrrrrrrrrrrrr : {:?}", obj);

    unsafe { evas_object_geometry_get(obj, mem::transmute(&mut x), &mut y, &mut w, &mut h); }

    println!("caca : {:?}, {}, {}, {}, {}", obj, x, y, w, h);

    (x, y, w, h)
}

fn elm_object_text_set(
        obj : *const Evas_Object,
        text : *const c_char)
{
    unsafe { elm_object_part_text_set(obj, ptr::null(), text); }
}

pub struct Master
{
    pub resource : Rc<resource::ResourceGroup>,
    views : LinkedList<Box<View>>,
}

impl Master
{
    fn _new(container : &mut Box<WidgetContainer>) -> Master
    {
        let resource = container.resource.clone();

        let m = Master {
            resource : resource,
            views : LinkedList::new(),
        };

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

    let wc = WindowConfig::load();

    for v in wc.views.iter() {
        let wc = &v.window;
        let v = box View::new(master.resource.clone(), container,wc.w,wc.h);
        master.views.push_back(v);
    }

    while let Some(mut v) = master.views.pop_front() {

        let pc = if let Some(ref p) = wc.property {
            p.clone()
        }
        else {
            ui::PropertyConfig::new()
        };
        let tc = if let Some(ref t) = wc.tree {
            t.clone()
        }
        else {
            ui::WidgetConfig::new()
        };
        v.init(container, &pc, &tc);

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

#[derive(RustcDecodable, RustcEncodable, Clone)]
pub struct WidgetConfig
{
    pub visible : bool,
    pub x : i32,
    pub y : i32,
    pub w : i32,
    pub h : i32,
}

impl WidgetConfig
{
    fn new_from_obj(obj : *const Evas_Object) -> WidgetConfig
    {
        let (x, y, w, h) = object_geometry_get(obj);

        WidgetConfig {
            x : x,
            y : y,
            w : w,
            h : h,
            visible : true
        }
    }

    fn new() -> WidgetConfig
    {
        WidgetConfig {
            x : 10,
            y : 10,
            w : 300,
            h : 400,
            visible : true
        }
    }

}

#[derive(RustcDecodable, RustcEncodable, Clone)]
pub struct ViewConfig
{
    window : WidgetConfig
}

#[derive(RustcDecodable, RustcEncodable, Clone)]
pub struct WindowConfig
{
    views: Vec<ViewConfig>,
    property : Option<PropertyConfig>,
    tree : Option<WidgetConfig>
}

impl WindowConfig {

    fn new(c : &WidgetContainer) ->  WindowConfig
    {
        let mut wc = WindowConfig {
            views : Vec::new(),
            property : match c.property {
                None => None,
                Some(ref p) => Some(p.config.clone())
            },
            tree : match c.tree {
                None => None,
                Some(ref t) => Some(t.get_config())
            }
        };

        for v in c.views.iter() {
            let vc = ViewConfig {
                //window : WidgetConfig::new( unsafe { window_object_get(win) })
                window : WidgetConfig{
                    x : 0,
                    y : 0,
                    w : v.width,
                    h : v.height,
                    visible : true
                },
            };
            wc.views.push(vc);
        }

        wc
    }

    fn default() ->  WindowConfig
    {
        let mut wc = WindowConfig {
            views : Vec::new(),
            property : None,
            tree : None
        };

        let vc = ViewConfig {
            //window : WidgetConfig::new( unsafe { window_object_get(win) })
            window : WidgetConfig{
                x : 0,
                y : 0,
                w : 800,
                h : 500,
                visible : true
            },
            /*
            property : WidgetConfig{
                x : 0,
                y : 0,
                w : 100,
                h : 400,
                visible : true
            }
            */
        };

        wc.views.push(vc);

        wc
    }


    fn save(&self)
    {
        println!("save scene todo serialize");
        //let path : &Path = self.name.as_ref();
        let path : &Path = Path::new("windowconf");
        let mut file = File::create(path).ok().unwrap();
        let mut s = String::new();
        {
            let mut encoder = json::Encoder::new_pretty(&mut s);
            let _ = self.encode(&mut encoder);
        }

        //let result = file.write(s.as_ref().as_bytes());
        let result = file.write(s.as_bytes());
    }

    fn load() -> WindowConfig
    {
        let mut file = String::new();
        let wc : WindowConfig = match File::open(&Path::new("windowconf")){
            Ok(ref mut f) => {
                f.read_to_string(&mut file);
                json::decode(file.as_ref()).unwrap()
            },
            _ => {
                WindowConfig::default()
            }
        };

        wc
    }

}

pub extern fn exit_cb(data: *mut c_void) -> () {
    //let master_rc : &Rc<RefCell<Master>> = unsafe {mem::transmute(data)};
    //let master = master_rc.borrow();

    let app_data : &AppCbData = unsafe {mem::transmute(data)};
    let container : &mut Box<WidgetContainer> = unsafe {mem::transmute(app_data.container)};
    //let master_rc : &Rc<RefCell<Master>> = unsafe {mem::transmute(app_data.master)};
    //let master = master_rc.borrow();

    match container.context.scene {
        Some(ref s) => {
            println!("going to save: {}", s.borrow().name);
            s.borrow().save();
            //old
            //s.read().unwrap().save();
            //s.read().unwrap().savetoml();
            //s.borrow().savetoml();
        },
        None => {}
    }

    //TODO save window pos/size widgets pos/size
    //save views
    //save proerty

    let wc = WindowConfig::new(&*container);
    wc.save();
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
    pub command : Option<Box<Command>>,
    pub action : Option<Box<Action>>,
    views : Vec<Box<View>>,
    //pub context : Rc<RefCell<context::Context>>,
    pub context : Box<context::Context>,
    pub resource : Rc<resource::ResourceGroup>,
    //control : Rc<RefCell<control::Control>>
    pub factory : factory::Factory,
    pub op_mgr : operation::OperationManager,
    pub holder : Rc<RefCell<Holder>>,
    pub menu : Option<Box<Action>>,

    pub list : Box<ListWidget>,
    pub name : String
}

pub struct ListWidget
{
    object : Option<*const Evas_Object>,
    entries : Vec<*const c_char>
}

impl ListWidget
{
    pub fn create(&mut self, win : *const Window)
    {
        let name = CString::new("xaca".as_bytes()).unwrap().as_ptr();
        self.object = Some(unsafe { jk_list_wdg_new(win, name) });
    }

    pub fn set_fn(&self, cb : SelectCallback, data : ui::WidgetCbData)
    {
        unsafe {
            if let Some(o) = self.object {
                jk_list_fn_set(o,
                           cb,
                           mem::transmute(box data));
            }
        }
    }

    fn show_list(&mut self, entries : Vec<String>, x : i32, y : i32)
    {
        if let Some(o) = self.object {
            unsafe { 
                evas_object_show(o);
                evas_object_move(o, x, y);
                evas_object_resize(o, 150, 300);
            }

            let cs = util::string_to_cstring(entries);
            self.entries = cs.into_iter().map( |x| x.as_ptr()).collect();

            unsafe { 
                jklist_set_names(o, self.entries.as_ptr() as *const c_void, self.entries.len() as size_t);
            }
        }
    }
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
            command : None,
            action : None,
            menu : None,
            views : Vec::new(),
            //context : Rc::new(RefCell::new(context::Context::new())),
            context : box context::Context::new(),
            resource : Rc::new(resource::ResourceGroup::new()),
            factory : factory::Factory::new(),
            op_mgr : operation::OperationManager::new(),
            holder : Rc::new(RefCell::new(Holder { gameview : None })),
            list : box ListWidget { object : None, entries : Vec::new() },
            name : String::from("yoplaboum")

        }
    }

    pub fn handle_change(&mut self, change : &operation::Change, widget_origin: uuid::Uuid)
    {
        //if *change == operation::Change::None {
        if let operation::Change::None = *change {
            return;
        }

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
                    if let Some(ref t) = self.tree {
                        if widget_origin != t.id {
                            t.update_object(&o.read().unwrap().id);
                        }
                    };
                }
                else if name.starts_with("object/comp_data/MeshRender") {
                    let mut ob = o.write().unwrap();
                    let omr = ob.get_comp_data_value::<component::mesh_render::MeshRender>();
                    if let Some(ref mr) = omr {
                        ob.mesh_render =
                            Some(component::mesh_render::MeshRenderer::with_mesh_render(mr,&self.resource));
                    }
                }

                match self.property {
                    Some(ref p) => {
                         //p.update_object(&*o.read().unwrap(), s);
                        if widget_origin != p.id {
                            p.update_object_property(&*o.read().unwrap(), name);
                        }
                     },
                    None => {}
                };
            },
            operation::Change::Objects(ref name, ref id_list) => {
                let sel = self.get_selected_object();
                for id in id_list.iter() {

                    if name == "object/name" {
                        if let Some(ref t) = self.tree {
                            if widget_origin != t.id {
                                t.update_object(id);
                            }
                        };
                    }
                    else if name.starts_with("object/comp_data/MeshRender") {
                        let scene = self.get_scene();
                        let oob = if let Some(ref sc) = scene {
                            let s = sc.borrow();
                            s.find_object_by_id(&id)
                        } else {
                            None
                        };

                        if let Some(o) = oob {
                            let mut ob = o.write().unwrap();
                            println!("please update mesh");
                            let omr = ob.get_comp_data_value::<component::mesh_render::MeshRender>();
                            if let Some(ref mr) = omr {
                                ob.mesh_render =
                                    Some(component::mesh_render::MeshRenderer::with_mesh_render(mr,&self.resource));
                            }
                        }
                    }

                    if let Some(ref o) = sel {
                        let ob = o.read().unwrap();

                        if *id == ob.id  {
                            if let Some(ref mut p) = self.property {
                                if widget_origin != p.id {
                                    p.update_object(&*ob, "");
                                }
                            }
                        }
                    }
                }
            },
            operation::Change::ComponentChanged(uuid, ref comp_name) => {
                println!("comp changed : {} ", comp_name);
                let sel = self.get_selected_object();
                if let Some(ref o) = sel {
                    {
                    let ob = o.read().unwrap();
                    if uuid == ob.id  {
                        match self.property {
                            Some(ref mut p) =>
                                {
                                    if widget_origin != p.id {
                                        p.update_object(&*ob, "");
                                    }
                                },
                            None => {}
                        }
                    }
                    }
                }

                if comp_name.starts_with("MeshRender") {
                    let scene = self.get_scene();
                    let oob = if let Some(ref sc) = scene {
                        let s = sc.borrow();
                        s.find_object_by_id(&uuid)
                    } else {
                        None
                    };

                    if let Some(o) = oob {
                            let mut ob = o.write().unwrap();
                            println!("please update mesh");
                            let omr = ob.get_comp_data_value::<component::mesh_render::MeshRender>();
                            if let Some(ref mr) = omr {
                                ob.mesh_render =
                                    Some(component::mesh_render::MeshRenderer::with_mesh_render(mr,&self.resource));
                            }
                    }
                }
            },
            operation::Change::ChangeSelected(ref list) => {
                self.context.selected = list.clone();
                self.handle_change(&operation::Change::SelectedChange, widget_origin);
            },
            operation::Change::SelectedChange => {
                let sel = self.get_selected_objects();
                println!("container, object seclected : {}",  sel.len());

                if let Some(ref mut t) = self.tree {
                    if widget_origin != t.id {
                        let ids = self.context.get_vec_selected_ids();
                        t.select_objects(ids);
                    }
                }

                if sel.len() != 1 {
                    match self.property {
                        Some(ref mut p) => {
                            if widget_origin != p.id {
                                    p.set_nothing();
                            }
                        },
                        None => {
                            println!("container no property");
                        }
                    }
                }
                else {
                    match sel.front() {
                        Some(o) => {
                            match self.property {
                                Some(ref mut p) => {
                                    if widget_origin != p.id {
                                        p.set_object(&*o.read().unwrap());
                                    }
                                },
                                None => {
                                    println!("container no property");
                                }
                            }
                        },
                        _ => {},
                    }
                }
            },
            operation::Change::SceneRemove(ref id, ref obs) => {
                {
                    println!("container, sceneremove!!!!!!!!");
                    self.context.remove_objects_by_id(obs.clone());
                }
                if let Some(ref mut t) = self.tree {
                    if widget_origin != t.id {
                        t.remove_objects_by_id(obs.clone());
                    }
                }
                //TODO
                println!("do something for the other widget");
                self.handle_change(&operation::Change::SelectedChange, widget_origin);
            },
            operation::Change::SceneAdd(ref id, ref obs) => {
                let scene = match self.get_scene() {
                    Some(s) => s,
                    None => return
                };

                let objects = scene.borrow().find_objects_by_id(&mut obs.clone());

                // todo
                match self.tree {
                    Some(ref mut t) => {
                        if widget_origin != t.id {
                            t.add_objects(objects);
                        }
                    },
                    None => {
                        println!("control no tree");
                    }
                }
            },
            operation::Change::DraggerOperation(ref op) => {
                let (prop, operation) = {
                    let context = &self.context;;
                    match *op {
                        dragger::Operation::Translation(v) => {
                            let prop = vec!["object".to_string(),"position".to_string()];
                            let cxpos = context.saved_positions.clone();
                            let mut saved_positions = Vec::with_capacity(cxpos.len());
                            for p in cxpos.iter() {
                                saved_positions.push((box *p ) as Box<Any>);
                            }
                            let mut new_pos = Vec::with_capacity(cxpos.len());
                            for p in cxpos.iter() {
                                let np = *p + v;
                                new_pos.push((box np) as Box<Any>);
                            }
                            let change = operation::OperationData::Vector(
                                saved_positions,
                                new_pos);

                            (prop, change)
                        },
                        dragger::Operation::Scale(v) => {
                            let prop = vec!["object".to_string(),"scale".to_string()];
                            let cxsc = context.saved_scales.clone();
                            let mut saved_scales = Vec::with_capacity(cxsc.len());
                            for p in cxsc.iter() {
                                saved_scales.push((box *p ) as Box<Any>);
                            }
                            let mut new_sc = Vec::with_capacity(cxsc.len());
                            for s in cxsc.iter() {
                                let ns = *s * v;
                                new_sc.push((box ns) as Box<Any>);
                            }
                            let change = operation::OperationData::Vector(
                                saved_scales,
                                new_sc);

                            (prop, change)
                        },
                        dragger::Operation::Rotation(q) => {
                            let prop = vec!["object".to_string(),"orientation".to_string()];
                            let cxoris = context.saved_oris.clone();
                            let mut saved_oris = Vec::with_capacity(cxoris.len());
                            for p in cxoris.iter() {
                                saved_oris.push((box *p ) as Box<Any>);
                            }
                            let mut new_ori = Vec::with_capacity(cxoris.len());
                            for p in cxoris.iter() {
                                let no = *p * q;
                                new_ori.push((box no) as Box<Any>);
                            }
                            let change = operation::OperationData::Vector(
                                saved_oris,
                                new_ori);

                            (prop, change)
                        }
                    }
                };
                self.request_operation(prop, operation);
            },
            operation::Change::Undo => {
                let change = self.undo();
                self.handle_change(&change, widget_origin);
            },
            operation::Change::Redo => {
                let change = self.redo();
                self.handle_change(&change, widget_origin);
            },
            operation::Change::DraggerTranslation(t) => {
                let change = self.request_translation(t);
                self.handle_change(&change, widget_origin);
            },
            operation::Change::DraggerScale(s) => {
                let change = self.request_scale(s);
                self.handle_change(&change, widget_origin);
            },
            operation::Change::DraggerRotation(r) => {
                let change = self.request_rotation(r);
                self.handle_change(&change, widget_origin);
            },
            _ => {}
        }
    }

    pub fn handle_event(&mut self, event : ui::Event, widget_origin: uuid::Uuid)
    {
        match event {
            Event::SelectObject(ob) => {
                println!("selected : {}", ob.read().unwrap().name);
                let mut l = Vec::new();
                l.push(ob.read().unwrap().id.clone());
                self.select_by_id(&mut l);
                self.handle_change(&operation::Change::SelectedChange, widget_origin);
            },
            Event::UnselectObject(ob) => {
                println!("unselected : {}", ob.read().unwrap().name);
                let mut l = LinkedList::new();
                l.push_back(ob.read().unwrap().id.clone());
                self.unselect(&l);
                self.handle_change(&operation::Change::SelectedChange, widget_origin);
            },
            _ => {}
        }

    }

    pub fn get_selected_object(&self) -> Option<Arc<RwLock<object::Object>>>
    {
        match self.context.selected.front() {
            Some(o) => return Some(o.clone()),
            None => {
                println!("view get selected objects, no objects selected");
                return None;
            }
        };
    }

    fn get_scene(&self) -> Option<Rc<RefCell<scene::Scene>>>
    {
        match self.context.scene {
            Some(ref s) => Some(s.clone()),
            None => None
        }
    }

    fn find_object(&self, uuid : &Uuid) -> Option<Arc<RwLock<object::Object>>>
    {
        if let Some(ref s) = self.get_scene() {
            s.borrow().find_object_by_id(uuid)
        }
        else {
            None
        }
    }

    fn get_selected_objects(&self) -> LinkedList<Arc<RwLock<object::Object>>>
    {
        self.context.selected.clone()
    }

    pub fn request_operation(
        &mut self,
        name : Vec<String>,
        change : operation::OperationData
        ) -> operation::Change
    {
        let op = operation::Operation::new(
            self.get_selected_objects(),
            name.clone(),
            change
            );

        let change = self.op_mgr.add(op);
        change

        //let s = join_string(&name);
        //return operation::Change::Objects(s,self.context.borrow().get_selected_ids());
    }

    pub fn undo(&mut self) -> operation::Change
    {
        self.op_mgr.undo()
    }

    pub fn redo(&mut self) -> operation::Change
    {
        self.op_mgr.redo()
    }

    pub fn request_operation_old_new<T : Any+PartialEq>(
        &mut self,
        name : Vec<String>,
        old : Box<T>,
        new : Box<T>) -> operation::Change
    {
        if *old == *new {
            return operation::Change::None;
        }

        self.request_operation(
            name,
            operation::OperationData::OldNew(old,new)
            )
    }

    pub fn request_direct_change(
        &mut self,
        name : Vec<String>,
        new : &Any) -> operation::Change
    {
        println!("request direct change {:?}", name);
        let o = match self.get_selected_object() {
            Some(ob) => ob,
            None => {
                println!("direct change, no objects selected");
                return operation::Change::None;
            }
        };

        let vs = name[1..].to_vec();

        //o.write().set_property_hier(vs, new);
        o.write().unwrap().test_set_property_hier(join_string(&vs).as_ref(), new);

        let s = join_string(&name);
        return operation::Change::DirectChange(s);
    }

    pub fn request_operation_option_to_none(
        &mut self,
        path : &str)
        -> operation::Change
    {
        let v: Vec<&str> = path.split('/').collect();

        let mut vs = Vec::new();
        for i in v.iter()
        {
            vs.push(i.to_string());
        }


        let  prop = if let Some(o) = self.get_selected_object(){
            let p : Option<Box<Any>> = o.read().unwrap().get_property_hier(path);
            match p {
                Some(pp) => pp,
                None => return operation::Change::None
            }
        }
        else {
            return operation::Change::None;
        };

        self.request_operation(
            vs,
            operation::OperationData::ToNone(prop)
            )
    }

    pub fn request_operation_option_to_some(
        &mut self,
        name : Vec<String>) -> operation::Change
    {
        /*
        let n = if new == "None" {
            None
        }
        else {
            //let r : T = resource::Create::create("yep");
            //Some(r)
            None
        };
        */


        //todo chris
        //return operation::Change::None;
        self.request_operation(
            name,
            operation::OperationData::ToSome
            )
    }

    pub fn request_operation_vec_add(
        &mut self,
        path : &str)
        -> operation::Change
    {
        let v: Vec<&str> = path.split('/').collect();

        let mut vs = Vec::new();
        for i in v.iter()
        {
            vs.push(i.to_string());
        }

        let index = match v[v.len()-1].parse::<usize>() {
            Ok(index) => index,
            _ => 0
        };

        self.request_operation(
            vs,
            operation::OperationData::VecAdd(index)
            )
    }

    pub fn request_operation_vec_del(
        &mut self,
        path : &str
        )
        -> operation::Change
    {
        let v: Vec<&str> = path.split('/').collect();

        let mut vs = Vec::new();
        for i in v.iter()
        {
            vs.push(i.to_string());
        }

        let  prop = if let Some(o) = self.get_selected_object(){
            let p : Option<Box<Any>> = o.read().unwrap().get_property_hier(path);
            match p {
                Some(pp) => pp,
                None => return operation::Change::None
            }
        }
        else {
            return operation::Change::None;
        };

        match v[v.len()-1].parse::<usize>() {
            Ok(index) => self.request_operation(
                vs,
                operation::OperationData::VecDel(index, prop)//TODO index
                ),
                _ => operation::Change::None
        }
    }



    pub fn remove_selected_objects(&mut self) -> operation::Change
    {
        println!("control remove sel");

        let s = match self.get_scene() {
            Some(s) => s,
            None => return operation::Change::None
        };


        let list = self.get_selected_objects();
        let mut vec = Vec::new();
        for o in list.iter() {
            vec.push(o.clone());
        }

        let vs = Vec::new();
        return self.request_operation(
            vs,
            operation::OperationData::SceneRemoveObjects(s.clone(),vec.clone())
            );

        //return operation::Change::SceneRemove(s.read().unwrap().id, vec);
    }

    pub fn copy_selected_objects(&mut self) -> operation::Change
    {
        let s = match self.get_scene() {
            Some(s) => s,
            None => return operation::Change::None
        };

        let list = self.get_selected_objects();
        let mut vec = Vec::new();
        for o in list.iter() {
            //vec.push(o.clone());
            vec.push(Arc::new(RwLock::new(self.factory.copy_object(&*o.read().unwrap()))));
        }

        let vs = Vec::new();
        return self.request_operation(
            vs,
            operation::OperationData::SceneAddObjects(s.clone(),vec.clone())
            );

        //return operation::Change::SceneRemove(s.read().unwrap().id, vec);
    }


    pub fn add_component(&mut self, component_name : &str) -> operation::Change
    {

        let list = self.get_selected_objects();
        let o = if list.len() == 1 {
            list.front().unwrap()
        }
        else
        {
            return operation::Change::None;
        };

        let cp = if component_name == "MeshRender" {
            box component::CompData::MeshRender(component::mesh_render::MeshRender::new("model/skeletonmesh.mesh", "material/simple.mat"))
        }
        else {
            return operation::Change::None;
        };

        let vs = Vec::new();

        self.request_operation(
            vs,
            operation::OperationData::AddComponent(o.clone(), cp)
            )
    }

    pub fn set_scene_camera(&mut self) -> operation::Change
    {
        println!("control remove sel");

        let s = match self.get_scene() {
            Some(s) => s,
            None => return operation::Change::None
        };

        let current = match s.borrow().camera {
            None => None,
            Some(ref c) => Some(c.borrow().object.clone())
        };

        let o = self.get_selected_object();
        println!("control set camera");

        let vs = Vec::new();
        return self.request_operation(
            vs,
            operation::OperationData::SetSceneCamera(s.clone(),current, o.clone())
            );

        //return operation::Change::SceneRemove(s.read().unwrap().id, vec);

    }

    fn select_by_id(&mut self, ids : &mut Vec<Uuid>)
    {
        //TODO same as the code at the end of mouse_up, so factorize
        println!("TODO check: is this find by id ok? : control will try to find object by id, .................select is called ");
        let c = &mut self.context;

        //c.selected.clear();

        let scene = match c.scene {
            Some(ref s) => s.clone(),
            None => return
        };

        let mut obs = scene.borrow().find_objects_by_id(ids);
        c.selected.append(&mut obs);

        //for id in ids.iter() {
            //match scene.read().unwrap().find_object_by_id(id) {
                //Some(o) =>
                    //c.selected.push_back(o.clone()),
                //None => {}
            //};
        //}

    }

    fn unselect(&mut self, ids : &LinkedList<Uuid>)
    {
        let c = &mut self.context;

        let scene = match c.scene {
            Some(ref s) => s.clone(),
            None => return
        };

        let mut newlist = LinkedList::new();

        for o in c.selected.iter() {
            let mut should_remove = false;
            for id_to_rm in ids.iter() {
                if o.read().unwrap().id == *id_to_rm {
                    should_remove = true;
                    break;
                }
            }

            if !should_remove {
                newlist.push_back(o.clone());
            }
        }

        c.selected = newlist;


        /* TODO notify property
        match self.property {
            Some(ref mut pp) =>
                match pp.try_borrow_mut() {
                    Some(ref mut p) => {
                        p.set_object(&*o.read().unwrap());
                    },
                    None=> {}
                },
                None => {}
        }
        */
    }

    fn request_translation(
        &mut self,
        translation : vec::Vec3) -> operation::Change
    {
        let sp = self.context.saved_positions.clone();
        let mut obs = self.get_selected_objects();

        let mut i = 0;
        for o in obs.iter_mut() {
            //o.write().unwrap().test_set_property_hier(join_string(&vs).as_ref(), new);
            o.write().unwrap().position = sp[i] + translation;
            i = i+1;
        }

        return operation::Change::DirectChange("object/position".to_string());
    }

    fn request_scale(
        &mut self,
        scale : vec::Vec3) -> operation::Change
    {
        let sp = self.context.saved_scales.clone();
        let mut obs = self.get_selected_objects();

        let mut i = 0;
        for o in obs.iter_mut() {
            //o.write().unwrap().test_set_property_hier(join_string(&vs).as_ref(), new);
            o.write().unwrap().scale = sp[i] * scale;
            i = i+1;
        }

        return operation::Change::DirectChange("object/scale".to_string());
    }

    fn request_rotation(
        &mut self,
        rotation : vec::Quat) -> operation::Change
    {
        let so = self.context.saved_oris.clone();
        let mut obs = self.get_selected_objects();

        let mut i = 0;
        for o in obs.iter_mut() {
            o.write().unwrap().orientation = so[i] * transform::Orientation::new_with_quat(&rotation);
            i = i+1;
        }

        return operation::Change::DirectChange("object/orientation".to_string());
    }


    pub fn find_view(&self, id : Uuid) -> Option<&View>
    {
        for v in self.views.iter()
        {
            if v.uuid == id {
                return Some(v)
            }
        }
        None
    }

    pub fn set_scene(&mut self, s : scene::Scene)
    {
        if let Some(ref mut t) = self.tree {
            t.set_scene(&s);
        }

        if let Some(ref mut p) = self.property {
            p.set_nothing();
        }

        if let Some(ref mut m) = self.menu {
            if let Entry::Occupied(en) = m.entries.entry(String::from("scene")) {
                elm_object_text_set(
                    unsafe {mem::transmute(*en.get())},
                    CString::new(s.name.as_str()).unwrap().as_ptr());
            }
        }

        let rs =  Rc::new(RefCell::new(s));
        self.context.set_scene(rs);
    }
}

//Send to c with mem::transmute(box data)  and free in c
pub struct WidgetCbData
{
    pub container : *const WidgetContainer,
    pub widget : *const c_void,
    pub object : Option<*const Evas_Object>
}

impl Clone for WidgetCbData {
    fn clone(&self) -> WidgetCbData 
    {
        WidgetCbData {
            container : self.container,
            widget : self.widget,
            object : self.object,
        }
    }
}

impl WidgetCbData {
    //pub fn new(c : &Box<WidgetContainer>, widget : &Box<Widget>)
    pub fn with_ptr(c : &Box<WidgetContainer>, widget : *const c_void) -> WidgetCbData
    {
        println!("TODO free me");
        WidgetCbData {
            container : unsafe {mem::transmute(c)},
            widget : widget,
            object : None
        }
    }

    pub fn new(c : &WidgetContainer, widget : *const c_void) -> WidgetCbData
    {
        println!("TODO free me");
        WidgetCbData {
            container : unsafe {mem::transmute(c)},
            widget : widget,
            object : None
        }
    }

    pub fn with_ptr_obj(c : &Box<WidgetContainer>, widget : *const c_void, object : *const Evas_Object) -> WidgetCbData
    {
        println!("TODO free me");
        WidgetCbData {
            container : unsafe {mem::transmute(c)},
            widget : widget,
            object : Some(object)
        }
    }
}

pub struct AppCbData
{
    pub master : *const c_void,
    pub container : *const c_void
}

impl Clone for AppCbData {
    fn clone(&self) -> AppCbData 
    {
        AppCbData {
            master : self.master,
            container : self.container
        }
    }
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

fn join_string(path : &Vec<String>) -> String
{
    let mut s = String::new();
    let mut first = true;
    for v in path.iter() {
        if !first {
            s.push('/');
        }
        s.push_str(v.as_ref());
        first = false;
    }

    s
}

pub fn add_empty(container : &mut WidgetContainer, view_id : Uuid)
{
    println!("add empty");

    let mut o = container.factory.create_object("new object");

    let position = if let Some(v) = container.find_view(view_id) {
        let (p,q) = v.get_camera_transform();
        p + q.rotate_vec3(&vec::Vec3::new(0f64,0f64,-100f64))
    }
    else {
        vec::Vec3::zero()
    };

    o.position = position;


    let ao =  Arc::new(RwLock::new(o));

    let mut list = LinkedList::new();
    list.push_back(ao.clone());

    let s = if let Some(ref s) = container.context.scene {
        s.clone()
    }
    else {
        return;
    };

    let mut vec = Vec::new();
    vec.push(ao.clone());

    let mut ops = Vec::new();
    let vs = Vec::new();
    let addob = container.request_operation(
            vs,
            operation::OperationData::SceneAddObjects(s.clone(),vec)
            );

    ops.push(addob);
    ops.push(operation::Change::ChangeSelected(list));

    for op in ops.iter() {
        container.handle_change(op, view_id);
    }
}

pub fn scene_new(container : &mut WidgetContainer, view_id : Uuid)
{
    let suffix = ".scene";
    let newname = match container.context.scene {
        Some(ref sc) => {
            let s = sc.borrow();
            let old = if s.name.ends_with(suffix) {
                let i = s.name.len() - suffix.len();
                let (yep,_) = s.name.split_at(i);
                yep
            }
            else {
                s.name.as_ref()
            };    
            String::from(old)
        },
        None => String::from("scene/new.scene")
    };

    let mut i = 0i32;
    let mut ss = newname.clone();
    loop {
        ss.push_str(format!("{:03}",i).as_str());
        ss.push_str(suffix);

        if let Err(_) = fs::metadata(ss.as_str()) {
            break;
        }

        i = i+1;
        ss = newname.clone();
    }

    let s = container.factory.create_scene(ss.as_ref());
    container.set_scene(s);
}

pub fn scene_list(container : &mut WidgetContainer, view_id : Uuid, obj : Option<*const Evas_Object>)
{
    let (x, y) = if let Some(o) = obj {
        println!("TODO show the list of scene, there is an obj");
        let (mut x, mut y, mut w, mut h) : (c_int, c_int, c_int, c_int) = (5,6,7,8);
        unsafe { evas_object_geometry_get(o, &mut x, &mut y, &mut w, &mut h); }
        (x, y + h + 5)
    }
    else {
        println!("TODO show the list of scene, no obj");
        (250, 50)
    };

    let files = util::get_files_in_dir("scene");
    let filesstring : Vec<String> = files.iter().map(|x| String::from(x.to_str().unwrap())).collect();

    //container.list.show_list(Vec::new(), x, y);
    container.list.show_list(filesstring, x, y);

    let listwd = ui::WidgetCbData::new(container, unsafe { mem::transmute(&*container.list)});
    container.list.set_fn(select_list, listwd);
}

pub extern fn select_list(data : *const c_void, name : *const c_char)
{
    let wcb : & ui::WidgetCbData = unsafe {mem::transmute(data)};
    let action : &Action = unsafe {mem::transmute(wcb.widget)};
    let container : &mut ui::WidgetContainer = unsafe {mem::transmute(wcb.container)};
    
    let s = unsafe {CStr::from_ptr(name)}.to_str().unwrap();
    println!("selection ..........{},  {}", container.name, s);
    //let scene = container.factory.create_scene(s);
    let mut scene = scene::Scene::new_from_file(s, &*container.resource);
    if let None = scene.camera {
        let mut cam = container.factory.create_camera();
        cam.pan(&vec::Vec3::new(-100f64,20f64,100f64));
        cam.lookat(vec::Vec3::new(0f64,5f64,0f64));
        scene.camera = Some(Rc::new(RefCell::new(cam)));
    }
    //let scene = Rc::new(RefCell::new(ss));

    container.set_scene(scene);
}

