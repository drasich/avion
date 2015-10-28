use libc::{c_char, c_void, c_int};
use std::mem;
use std::sync::{RwLock, Arc};
use std::collections::{LinkedList};
use std::ptr;
use std::rc::Rc;
use std::cell::{RefCell, BorrowState};
use std::collections::HashMap;
use std::any::{Any};//, AnyRefExt};
use std::path::Path;
use std::fs::File;
use rustc_serialize::{json, Encodable, Encoder, Decoder, Decodable};
use std::io::{Read,Write};

use uuid::Uuid;

use intersection;
use resource;
use geometry;
use vec;
use scene;
use object;
use ui::{Tree,Property,PropertyConfig,View,Command,Action};
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

    fn evas_object_geometry_get(
        obj : *const Evas_Object,
        x : *mut c_int,
        y : *mut c_int,
        w : *mut c_int,
        h : *mut c_int);

    //fn window_object_get(
    //    obj : *const Window) -> *const Evas_Object;

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

        let mut m = Master {
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

#[derive(RustcDecodable, RustcEncodable, Clone)]
pub struct WidgetConfig
{
    visible : bool,
    x : i32,
    y : i32,
    w : i32,
    h : i32,
}

impl WidgetConfig
{
    fn new(obj : *const Evas_Object) -> WidgetConfig
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
    property : Option<PropertyConfig>
}

impl WindowConfig {

    fn new(c : &WidgetContainer) ->  WindowConfig
    {
        let mut wc = WindowConfig {
            views : Vec::new(),
            property : match c.property {
                None => None,
                Some(ref p) => Some(p.config.clone())
            }
        };

        //chris
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
            property : None
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
            views : Vec::new(),
            //context : Rc::new(RefCell::new(context::Context::new())),
            context : box context::Context::new(),
            resource : Rc::new(resource::ResourceGroup::new()),
            factory : factory::Factory::new(),
            op_mgr : operation::OperationManager::new(),

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
                    match self.tree {
                        Some(ref t) => {
                            if widget_origin != t.id {
                                t.update_object(&o.read().unwrap().id);
                            }
                        },
                        None => {}
                    };
                }
                else if name.starts_with("object/comp_data/MeshRender") {
                    let mut ob = o.write().unwrap();
                    println!("please update mesh");
                    let omr = ob.get_comp_data_value::<component::mesh_render::MeshRender>();
                    if let Some(ref mr) = omr {
                        ob.mesh_render =
                            Some(component::mesh_render::MeshRenderer::with_mesh_render(mr,&self.resource));
                    }
                }

                match self.property {
                    Some(ref p) => {
                        println!("direct change : {}", name);
                         //p.update_object(&*o.read().unwrap(), s);
                        if widget_origin != p.id {
                            p.update_object_property(&*o.read().unwrap(), name);
                        }
                     },
                    None => {}
                };
            },
            _ => {}
        }


        match *change {
            operation::Change::Objects(ref name, ref id_list) => {
                let sel = self.get_selected_object();
                for id in id_list.iter() {
                    if let Some(ref o) = sel {
                        {
                        let mut ob = o.read().unwrap();

                        if *id == ob.id  {
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

                        if name.starts_with("object/comp_data/MeshRender") {
                            let mut ob = o.write().unwrap();
                            println!("please update mesh");
                            let omr = ob.get_comp_data_value::<component::mesh_render::MeshRender>();
                            if let Some(ref mr) = omr {
                                ob.mesh_render =
                                    Some(component::mesh_render::MeshRenderer::with_mesh_render(mr,&self.resource));
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
                    let mut ob = o.read().unwrap();
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





}

//Send to c with mem::transmute(box data)  and free in c
#[derive(Clone)]
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


#[derive(Clone)]
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


