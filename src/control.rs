use std::rc::Rc;
use std::cell::RefCell;
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
//use ui::property;
//use ui::tree;
use ui::dragger;
use intersection;
use vec;
use object;
use property;
use property::PropertyWrite;
use uuid;

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
    pub property : Option<Rc<RefCell<Box<ui::Property>>>>, //TODO change to weak
    pub tree : Option<Rc<RefCell<Box<ui::Tree>>>>, //TODO change to weak
    pub dragger : Rc<RefCell<ui::dragger::DraggerManager>>,

    pub saved_positions : Option<DList<vec::Vec3>>,
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
            property : None,
            tree : None,
            state : State::Idle,
            context : context,
            dragger : dragger,

            saved_positions : None,
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
            self.mouse_start = Some(vec::Vec2::new(x as f64, y as f64));
            self.state = State::MultipleSelect;
            list.push_back(operation::Change::RectVisibleSet(true));
            list.push_back(operation::Change::RectSet(x as f32, y as f32, 1f32, 1f32));
            println!("pressed shift");
            return list;
        }
        else if modifier & (1 << 1) != 0 {
            println!("pressed control");
        }

        let click = self.dragger.borrow_mut().mouse_down(
            &*self.camera.borrow(),button, x, y);
        if click {
            self.state = State::Dragger;
            let objs = self.context.borrow().selected.clone();
            if objs.len() > 0 {
                let mut l = DList::new();
                for o in objs.iter() {
                    l.push_back(o.read().unwrap().position);
                }
                self.saved_positions = Some(l);
                //todo save the position if translation
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
                            let pos = if let Some(ref p) = self.saved_positions {
                                if let Some(pp) = p.front() {
                                    Some(pp.clone())
                                }
                                else {
                                    None
                                }
                            }
                            else {
                                None
                            };

                            if let Some(p) = pos {
                                //TODO
                                println!("todo multiple objects");
                                self.request_operation(prop, box p, box v);
                            }
                        },
                        _ => {}
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
        let r = match self.camera.try_borrow(){
            Some(c) => {
                c.ray_from_screen(x as f64, y as f64, 10000f64)
            },
            None => { println!("cannot borrow camera"); return operation::Change::None; }
        };

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
            None => { println!("cannot borrow context"); return operation::Change::None; }
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

        println!("object seclected : {}",  c.selected.len());

        if c.selected.len() == 0 {
            match self.property {
                Some(ref pp) => {
                    match pp.try_borrow_mut() {
                        Some(ref mut p) => {
                            p.set_nothing();
                        },
                        None => {println!("cannot borrow property");}
                    };
                },
                None => {
                    println!("control no property");
                }
            }
        }
        else if c.selected.len() == 1 {
            //TODO select tree
            match c.selected.front() {
                Some(o) => {
                    match self.property {
                        Some(ref pp) => {
                            match pp.try_borrow_mut() {
                                Some(ref mut p) => {
                                    p.set_object(&*o.read().unwrap());
                                },
                                None => {println!("cannot borrow property");}
                            };
                        },
                        None => {
                            println!("control no property");
                        }
                    }

                    match self.tree {
                        Some(ref tt) => {
                            match tt.try_borrow_mut() {
                                Some(ref mut t) => {
                                    t.select(&o.read().unwrap().id);
                                }
                                _ => {}
                            }
                        },
                        None => {
                            println!("control no tree");
                        }
                    }
                },
                _ => {},
            }
        }

        return operation::Change::None;
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

        for o in scene.read().unwrap().objects.iter() {
            if o.read().unwrap().id == *id {
                c.selected.push_back(o.clone());
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
                break;
            }
        }
    }

    pub fn request_operation<T : Any+PartialEq>(
        &mut self,  
        name : Vec<String>,
        old : Box<T>,
        new : Box<T>) -> operation::Change
    {
        if *old == *new {
            return operation::Change::None;
        }

        let o = match self.get_selected_object() {
            Some(o) => o.clone(),
            None => {
                println!("no objetcs selected");
                return operation::Change::None;
            }
        };

        let op = operation::Operation::new(
            o.clone(), 
            name.clone(),
            old,
            new); 

        op.apply();

        //TODO update the widget that has this object/property

        self.op_mgr.add(op);

        let mut list = DList::new();
        list.push_back(o.read().unwrap().id.clone());
        let s = join_string(&name);
        return operation::Change::Objects(s,list);
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

        /*
        //TODO it might do more than just update this property
        // for example for orientation enum, you change the enum
        // the widget must change.
        // the proeprty might also be displayed somewhere else

        //TODO update the widget that has this object/property, but not the
        // widget where the change came from
        // add widget origin uuid in request_operation and request_direct_change

        //PROBLEM -> property borrow -> control borrow -> property borrow

        let s = join_string(&name);
        if s.as_slice() == "object/name" {
            match self.tree {
                Some(ref mut tt) =>
                    match tt.try_borrow_mut() {
                        Some(ref mut t) => {
                            t.update_object(&o.read().unwrap().id);
                        },
                        None=> {}
                    },
                    None => {}
            };
        }
            
        match self.property {
            Some(ref mut pp) =>
                match pp.try_borrow_mut() {
                    Some(ref mut p) => {
                        println!("direct change : {}", s);
                        p.update_object(&*o.read().unwrap(), s.as_slice());
                    },
                    None=> {}
                },
                None => {}
        };
        */
    }

    pub fn get_selected_object(&self) -> Option<Arc<RwLock<object::Object>>>
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
        timestamp : i32) -> operation::Change
    {
        match self.state {
            State::Idle | State::CameraRotation => {
                let x : f64 = curx as f64;
                let y : f64 = cury as f64;

                let r = match self.camera.try_borrow(){
                    Some(c) => {
                        c.ray_from_screen(x as f64, y as f64, 10000f64)
                    },
                    None => {
                        println!("cannot borrow camera"); 
                        return operation::Change::None;
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
                    }
                }
            },
            State::Dragger =>
            {
                let camera_clone = self.camera.clone();
                let camera = match camera_clone.try_borrow(){
                    Some(c) =>c,
                    None => { 
                        println!("cannot borrow camera");
                        return operation::Change::None;
                    }
                };

                let x : f64 = curx as f64;// - prevx as f64;
                let y : f64 = cury as f64;// - prevy as f64;
                if let Some(op) = self.dragger.borrow_mut().mouse_move(&*camera,x,y) {
                    match op {
                        dragger::Operation::Translation(v) => {
                            let prop = vec!["object".to_string(),"position".to_string()];
                            return self.request_direct_change(prop, &v);
                        },
                        _ => {}
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
                    return operation::Change::RectSet(startx, starty, endx, endy);
                }
            }
        }

        return operation::Change::None;
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
            _ => {}
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

