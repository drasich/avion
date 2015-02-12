use std::rc::Rc;
use std::cell::{RefCell, BorrowState};
use std::any::{Any};//, AnyRefExt};
use std::sync::{RwLock, Arc};
use std::collections::DList;
use std::f64::consts;
use transform;

use uuid::Uuid;

use operation;
use camera;
use context;
use ui;
use dragger;
use intersection;
use vec;
use object;
use property::PropertyWrite;

pub enum State
{
    Idle,
    CameraRotation,
    Dragger,
    MultipleSelect
}

pub struct Control
{
    pub op_mgr : operation::OperationManager,
    pub camera : Rc<RefCell<camera::Camera>>,
    pub state : State,
    pub context : Rc<RefCell<context::Context>>,

    //TODO control listener
    //pub property : Option<Rc<RefCell<ui::Property>>>, //TODO change to weak
    //pub tree : Option<Rc<RefCell<ui::Tree>>>, //TODO change to weak
    //pub property : Option<Rc<RefCell<Box<ui::Property>>>>, //TODO change to weak
    pub tree : Option<Rc<RefCell<Box<ui::Tree>>>>, //TODO change to weak
    pub dragger : Rc<RefCell<dragger::DraggerManager>>,

    pub mouse_start : Option<vec::Vec2>
}

impl Control
{
    pub fn new(
        camera : Rc<RefCell<camera::Camera>>,
        context : Rc<RefCell<context::Context>>,
        dragger : Rc<RefCell<dragger::DraggerManager>>,
        ) -> Control
    {
        Control {
            op_mgr : operation::OperationManager::new(),
            camera : camera,
            //property : None,
            tree : None,
            state : State::Idle,
            context : context,
            dragger : dragger,

            mouse_start : None
        }
    }

    pub fn mouse_down(
            &mut self, 
            modifier : i32,
            button : i32,
            x : i32, 
            y : i32,
            timestamp : i32) -> DList<operation::Change>
    {
        let mut list = DList::new();

        if modifier & (1 << 0) != 0 {
            println!("pressed shifr");
        }
        else if modifier & (1 << 1) != 0 {
            self.mouse_start = Some(vec::Vec2::new(x as f64, y as f64));
            self.state = State::MultipleSelect;
            list.push_back(operation::Change::RectVisibleSet(true));
            list.push_back(operation::Change::RectSet(x as f32, y as f32, 1f32, 1f32));
            println!("pressed control");
            return list;
        }

        let click = self.dragger.borrow_mut().mouse_down(
            &*self.camera.borrow(),button, x, y);
        if click {
            self.state = State::Dragger;
            let objs = self.context.borrow().selected.clone();
            if objs.len() > 0 {
                self.context.borrow_mut().save_positions();
                self.context.borrow_mut().save_scales();
                self.context.borrow_mut().save_oris();
            }
        }

        return list;
    }

    pub fn mouse_up(
            &mut self, 
            button : i32,
            x : i32, 
            y : i32,
            timestamp : i32) -> operation::Change
    {
        println!("control fn mouse up");
        match self.state {
            State::CameraRotation => {
                self.state = State::Idle;
                println!("state was cam rotate ");
                return operation::Change::None;
            },
            State::Dragger => {
                self.state = State::Idle;
                let o = self.dragger.borrow_mut().mouse_up(
                    &*self.camera.borrow(),
                    button,
                    x,
                    y);

                if let Some(op) = o {
                    match op {
                        dragger::Operation::Translation(v) => {
                            let prop = vec!["object".to_string(),"position".to_string()];
                            let cxpos = self.context.borrow().saved_positions.clone();
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

                            self.request_operation(prop, change);
                        },
                        dragger::Operation::Scale(v) => {
                            let prop = vec!["object".to_string(),"scale".to_string()];
                            let cxsc = self.context.borrow().saved_scales.clone();
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

                            self.request_operation(prop, change);
                        },
                        dragger::Operation::Rotation(q) => {
                            let prop = vec!["object".to_string(),"orientation".to_string()];
                            let cxoris = self.context.borrow().saved_oris.clone();
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

                            self.request_operation(prop, change);
                        },
                    }
                }
                return operation::Change::None;
            },
            State::MultipleSelect => {
                self.state = State::Idle;
                return operation::Change::RectVisibleSet(false);
            },
            _ => {}
        }

        //println!("rust mouse up button {}, pos: {}, {}", button, x, y);
        let r = match self.camera.borrow_state(){
            BorrowState::Writing => {
                println!("cannot borrow camera");
                return operation::Change::None;
            },
            _ => {
                self.camera.borrow().ray_from_screen(x as f64, y as f64, 10000f64)
            }
        };

        let mut c = match self.context.borrow_state(){
            BorrowState::Unused => self.context.borrow_mut(),
            _ => { println!("cannot borrow context"); return operation::Change::None; }
        };

        c.selected.clear();

        let scene = match c.scene {
            Some(ref s) => s.clone(),
            None => {
                println!("no scene ");
                return operation::Change::None;
            }
        };

        println!("objects in the scene : {}", scene.read().unwrap().objects.len());

        let mut found_length = 0f64;
        let mut closest_obj = None;
        for o in scene.read().unwrap().objects.iter() {
            let ir = intersection::ray_object(&r, &*o.read().unwrap());
            if ir.hit {
                let length = (ir.position - r.start).length2();
                match closest_obj {
                    None => {
                        closest_obj = Some(o.clone());
                        found_length = length;
                    }
                    Some(_) => {
                        if length < found_length {
                            closest_obj = Some(o.clone());
                            found_length = length;
                        }
                    }
                }
            }
        }

        match closest_obj {
            None => {},
            Some(o) => c.selected.push_back(o)
        }

        return operation::Change::SelectedChange;
    }

    //pub fn select(&mut self, ids : &DList<Uuid>)
    pub fn select(&mut self, ids : &mut Vec<Uuid>)
    {
        //TODO same as the code at the end of mouse_up, so factorize
        println!("TODO check: is this find by id ok? : control will try to find object by id, .................select is called ");
        let mut c = match self.context.borrow_state(){
            BorrowState::Unused => self.context.borrow_mut(),
            _ => { println!("cannot borrow context"); return; }
        };

        //c.selected.clear();

        let scene = match c.scene {
            Some(ref s) => s.clone(),
            None => return
        };

        let mut obs = scene.read().unwrap().find_objects_by_id(ids);
        c.selected.append(&mut obs);

        /*
        for id in ids.iter() {
            match scene.read().unwrap().find_object_by_id(id) {
                Some(o) =>
                    c.selected.push_back(o.clone()),
                None => {}
            };
        }
        */

    }

    pub fn unselect(&mut self, ids : &DList<Uuid>)
    {
        let mut c = match self.context.borrow_state(){
            BorrowState::Unused => self.context.borrow_mut(),
            _ => { println!("cannot borrow context"); return; }
        };

        let scene = match c.scene {
            Some(ref s) => s.clone(),
            None => return
        };

        let mut newlist = DList::new();

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

            /*
        let op = operation::Operation::new(
            self.get_selected_objects(),
            name.clone(),
            operation::OperationData::OldNew(old,new)
            //old,
            //new
            ); 

        op.apply();

        self.op_mgr.add(op);

        let s = join_string(&name);
        return operation::Change::Objects(s,self.context.borrow().get_selected_ids());
        */
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

        op.apply();

        self.op_mgr.add(op);

        let s = join_string(&name);
        return operation::Change::Objects(s,self.context.borrow().get_selected_ids());
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
                println!("direct change, no objetcs selected");
                return operation::Change::None;
            }
        };

        let vs = name.tail().to_vec();

        //o.write().set_property_hier(vs, new);
        o.write().unwrap().test_set_property_hier(join_string(&vs).as_slice(), new);

        let s = join_string(&name);
        return operation::Change::DirectChange(s);
    }

    pub fn request_translation(
        &mut self,  
        translation : vec::Vec3) -> operation::Change
    {
        let sp = self.context.borrow().saved_positions.clone();
        let mut obs = self.get_selected_objects();

        let mut i = 0;
        for o in obs.iter_mut() {
            //o.write().unwrap().test_set_property_hier(join_string(&vs).as_slice(), new);
            o.write().unwrap().position = sp[i] + translation;
            i = i+1;
        }

        return operation::Change::DirectChange("object/position".to_string());
    }

    pub fn request_scale(
        &mut self,  
        scale : vec::Vec3) -> operation::Change
    {
        let sp = self.context.borrow().saved_scales.clone();
        let mut obs = self.get_selected_objects();

        let mut i = 0;
        for o in obs.iter_mut() {
            //o.write().unwrap().test_set_property_hier(join_string(&vs).as_slice(), new);
            o.write().unwrap().scale = sp[i] * scale;
            i = i+1;
        }

        return operation::Change::DirectChange("object/scale".to_string());
    }

    pub fn request_rotation(
        &mut self,  
        rotation : vec::Quat) -> operation::Change
    {
        let so = self.context.borrow().saved_oris.clone();
        let mut obs = self.get_selected_objects();

        let mut i = 0;
        for o in obs.iter_mut() {
            o.write().unwrap().orientation = so[i] * transform::Orientation::new_with_quat(&rotation);
            i = i+1;
        }

        return operation::Change::DirectChange("object/orientation".to_string());
    }


    pub fn get_selected_object(&self) -> Option<Arc<RwLock<object::Object>>>
    {
        let c = match self.context.borrow_state(){
            BorrowState::Writing => { println!("cannot borrow context"); return None; }
            _ => self.context.borrow(),
        };

        match c.selected.front() {
            Some(o) => return Some(o.clone()),
            None => {
                println!("no objetcs selected");
                return None;
            }
        };
    }

    pub fn get_selected_objects(&self) -> DList<Arc<RwLock<object::Object>>>
    {
        match self.context.borrow_state(){
            BorrowState::Writing => DList::new(),
            _ => self.context.borrow().selected.clone(),
        }
    }

    pub fn undo(&mut self) -> operation::Change
    {
        self.op_mgr.undo()
    }

    pub fn redo(&mut self) -> operation::Change
    {
        self.op_mgr.redo()
    }


    fn rotate_camera(&mut self, x : f64, y : f64)
    {
        self.state = State::CameraRotation;

        let mut camera = self.camera.borrow_mut();
        let cori = camera.object.read().unwrap().orientation;

        let (result, angle_x, angle_y) = {
            let cam = &mut camera.data;

            if vec::Vec3::up().dot(&cori.rotate_vec3(&vec::Vec3::up())) <0f64 {
                cam.yaw = cam.yaw + 0.005*x;
            }
            else {
                cam.yaw = cam.yaw - 0.005*x;
            }

            cam.pitch -= 0.005*y;

            let qy = vec::Quat::new_axis_angle_rad(vec::Vec3::up(), cam.yaw);
            let qp = vec::Quat::new_axis_angle_rad(vec::Vec3::right(), cam.pitch);

            (
                qy * qp,
                cam.pitch/consts::PI*180f64,
                cam.yaw/consts::PI*180f64,
                )
        };

        let context = self.context.borrow();
        if self.context.borrow().selected.len() > 0 {
            let center = objects_center(&context.selected);
            camera.set_center(&center);
        }

        camera.rotate_around_center(&result);

        let mut c = camera.object.write().unwrap();
        //(*c).orientation = vec::Quat::new_yaw_pitch_roll_deg(angle_y, angle_x, 0f64);
        (*c).orientation = transform::Orientation::Quat(vec::Quat::new_yaw_pitch_roll_deg(angle_y, angle_x, 0f64));
        //self.state = CameraRotation;
    }

    pub fn mouse_move(
        &mut self, 
        mod_flag : i32,
        button : i32,
        curx : i32, 
        cury : i32,
        prevx : i32, 
        prevy : i32,
        timestamp : i32) -> DList<operation::Change>
    {
        let mut list = DList::new();

        match self.state {
            State::Idle | State::CameraRotation => {
                let x : f64 = curx as f64;
                let y : f64 = cury as f64;

                let r = match self.camera.borrow_state(){
                    BorrowState::Writing => {
                        println!("cannot borrow camera"); 
                        return list;
                    },
                    _ => {
                        self.camera.borrow().ray_from_screen(x as f64, y as f64, 10000f64)
                    }
                };

                self.dragger.borrow_mut().check_collision(r, button);
                
                if button == 1 {

                    self.dragger.borrow_mut().set_state(dragger::State::Idle);

                    let x : f64 = curx as f64 - prevx as f64;
                    let y : f64 = cury as f64 - prevy as f64;

                    if (mod_flag & (1 << 0)) != 0 {
                        let t = vec::Vec3::new(-x*0.5f64, y*0.5f64, 0f64);
                        let mut camera = self.camera.borrow_mut();
                        camera.pan(&t);
                    }
                    else {
                        self.rotate_camera(x, y);
                        let camera = self.camera.borrow();
                        println!("remove from update and move here");
                        //self.dragger.borrow_mut().set_orienation(&*camera);
                    }
                }
            },
            State::Dragger =>
            {
                let camera_clone = self.camera.clone();
                let camera = match camera_clone.borrow_state(){
                    BorrowState::Writing => { 
                        println!("cannot borrow camera");
                        return list;
                    },
                    _ => camera_clone.borrow(),
                };

                let x : f64 = curx as f64;// - prevx as f64;
                let y : f64 = cury as f64;// - prevy as f64;
                if let Some(op) = self.dragger.borrow_mut().mouse_move(&*camera,x,y) {
                    match op {
                        dragger::Operation::Translation(v) => {
                            list.push_back(self.request_translation(v));
                        },
                        dragger::Operation::Scale(v) => {
                            list.push_back(self.request_scale(v));
                        },
                        dragger::Operation::Rotation(q) => {
                            list.push_back(self.request_rotation(q));
                        }
                    }
                }
            }
            State::MultipleSelect => {
                if let Some(ms) = self.mouse_start {
                    let x = curx as f32;
                    let y = cury as f32;
                    let ex = ms.x as f32;
                    let ey = ms.y as f32;
                    let (startx, endx) = if x < ex {(x, ex - x)} else {(ex, x - ex)};
                    let (starty, endy) = if y < ey {(y, ey - y)} else {(ey, y - ey)};
                    list.push_back(operation::Change::RectSet(startx, starty, endx, endy));

                    let planes = self.camera.borrow().get_frustum_planes_rect(
                        startx as f64, 
                        starty as f64, 
                        endx as f64, 
                        endy as f64);

                    let mut c = match self.context.borrow_state(){
                        BorrowState::Unused => self.context.borrow_mut(),
                        _ => { println!("cannot borrow context, because being used"); return list; }
                    };

                    c.selected.clear();

                    let s = match c.scene {
                        Some(ref s) => s.clone(),
                        None => return list
                    };

                    for o in s.read().unwrap().objects.iter() {
                        let b = intersection::is_object_in_planes(planes.as_slice(), &*o.read().unwrap());
                        if b {
                            c.selected.push_back(o.clone());
                        }
                    }

                    list.push_back(operation::Change::SelectedChange);

                }
            }
        }

        return list;
    }

    pub fn mouse_wheel(
        &self,
        modifier : i32,
        direction : i32,
        z : i32, 
        x : i32, 
        y : i32,
        timestamp : i32
        )
    {
        let mut axis = vec::Vec3::new(0f64, 0f64, z as f64);
        axis = axis * 10.5f64;
        let mut camera = self.camera.borrow_mut();
        camera.pan(&axis);
    }

    pub fn key_down(
        &mut self,
        modifier : i32,
        keyname : &str,
        key : &str,
        timestamp : i32
        ) ->  operation::Change
    {
        let mut t = vec::Vec3::zero();

        match key {
            "e" => t.z = -50f64,
            "d" => t.z = 50f64,
            "f" => t.x = 50f64,
            "s" => t.x = -50f64,
            "z" => {
                return self.undo();
            },
            "r" => {
                return self.redo();
            },
            "space" => {
                self.dragger.borrow_mut().change();
            },
            _ => {
                println!("key not implemented : {}", key);
            }
        }

        {
            let mut camera = self.camera.borrow_mut();
            let p = camera.object.read().unwrap().position;
            camera.object.write().unwrap().position = p + t;
        }

        return operation::Change::None;
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

fn objects_center(objects : &DList<Arc<RwLock<object::Object>>>) -> vec::Vec3
{
    let mut v = vec::Vec3::zero();
    for o in objects.iter()
    {
        v = v + o.read().unwrap().position;
    }

    v = v / objects.len() as f64;

    v
}

pub trait WidgetUpdate {

    //fn update_changed<T : Any+Clone>(
    fn update_changed(
        &mut self,
        name : &str,
        //old : Option<&T>,
        new : &Any);
}

