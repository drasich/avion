use std::rc::Rc;
use std::cell::{RefCell, BorrowState};
use std::any::{Any};//, AnyRefExt};
use std::sync::{RwLock, Arc};
use std::collections::LinkedList;
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
use resource;
use property::PropertyGet;
use factory;
use component;

pub enum State
{
    Idle,
    CameraRotation,
    Dragger,
    MultipleSelect
}

pub struct Control
{
    pub camera : Rc<RefCell<camera::Camera>>,
    state : State,
    context : Rc<RefCell<context::Context>>,
    dragger : Rc<RefCell<dragger::DraggerManager>>,
    mouse_start : Option<vec::Vec2>,
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
            //factory : Rc::new(RefCell::new(factory::Factory::new())),
            camera : camera,
            //tree : None,
            state : State::Idle,
            context : context,
            dragger : dragger,

            mouse_start : None
        }
    }

    pub fn mouse_down(
            &mut self,
            context : &context::Context,
            modifier : i32,
            button : i32,
            x : i32,
            y : i32,
            timestamp : i32) -> LinkedList<operation::Change>
    {
        let mut list = LinkedList::new();

        if modifier & (1 << 0) != 0 {
            println!("pressed shift");
        }
        else if modifier & (1 << 1) != 0 {
            self.mouse_start = Some(vec::Vec2::new(x as f64, y as f64));
            self.state = State::MultipleSelect;
            list.push_back(operation::Change::RectVisibleSet(true));
            list.push_back(operation::Change::RectSet(x as f32, y as f32, 1f32, 1f32));
            println!("pressed control");
            return list;
        }

        let objs = context.selected.clone();
        if objs.len() > 0 {
            let click = self.dragger.borrow_mut().mouse_down(
                &*self.camera.borrow(),button, x, y);
            if click {
                self.state = State::Dragger;
                list.push_back(operation::Change::DraggerClicked);
                /*
                self.context.borrow_mut().save_positions();
                self.context.borrow_mut().save_scales();
                self.context.borrow_mut().save_oris();
                */
            }
        }

        return list;
    }

    pub fn mouse_up(
            &mut self,
            context : &context::Context,
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
                    return operation::Change::DraggerOperation(op);
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

        /*
        let scene = match self.context.borrow_state(){
            BorrowState::Writing => { println!("cannot borrow context"); return operation::Change::None; }
            _ => {
                let c = self.context.borrow();
                let scene = match c.scene {
                    Some(ref s) => s.clone(),
                    None => {
                        println!("no scene ");
                        return operation::Change::None;
                    }
                };
                scene
            }
        };
        */
        let scene = match context.scene {
             Some(ref s) => s.clone(),
             None => {
                 println!("no scene ");
                 return operation::Change::None;
             }
        };

        println!("objects in the scene : {}", scene.borrow().objects.len());

        let mut found_length = 0f64;
        let mut closest_obj = None;
        for o in scene.borrow().objects.iter() {
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

        let mut list = LinkedList::new();
        match closest_obj {
            None => {},
            Some(o) => list.push_back(o)
        }

        return operation::Change::ChangeSelected(list);
    }

    //pub fn select(&mut self, ids : &LinkedList<Uuid>)
    pub fn select_by_id(&mut self, ids : &mut Vec<Uuid>)
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

    pub fn unselect(&mut self, ids : &LinkedList<Uuid>)
    {
        let mut c = match self.context.borrow_state(){
            BorrowState::Unused => self.context.borrow_mut(),
            _ => { println!("cannot borrow context"); return; }
        };

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

    pub fn request_translation(
        &mut self,
        translation : vec::Vec3) -> operation::Change
    {
        let sp = self.context.borrow().saved_positions.clone();
        let mut obs = self.get_selected_objects();

        let mut i = 0;
        for o in obs.iter_mut() {
            //o.write().unwrap().test_set_property_hier(join_string(&vs).as_ref(), new);
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
            //o.write().unwrap().test_set_property_hier(join_string(&vs).as_ref(), new);
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
                println!("get_selected_objects : no objetcs selected");
                return None;
            }
        };
    }

    pub fn get_selected_objects(&self) -> LinkedList<Arc<RwLock<object::Object>>>
    {
        match self.context.borrow_state(){
            BorrowState::Writing => LinkedList::new(),
            _ => self.context.borrow().selected.clone(),
        }
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
        timestamp : i32) -> LinkedList<operation::Change>
    {
        let mut list = LinkedList::new();

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
                let opsome = self.dragger.borrow_mut().mouse_move(&*camera,x,y);
                if let Some(op) = opsome {
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

                    let s = match c.scene {
                        Some(ref s) => s.clone(),
                        None => return list
                    };

                    let mut oblist = LinkedList::new();
                    for o in s.borrow().objects.iter() {
                        let b = intersection::is_object_in_planes(planes.as_ref(), &*o.read().unwrap());
                        if b {
                            oblist.push_back(o.clone());
                        }
                    }

                    list.push_back(operation::Change::ChangeSelected(oblist));

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
                return operation::Change::Undo;
            },
            "r" => {
                return operation::Change::Redo;
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

    /*
    pub fn add_empty(&mut self, name : &str) -> Arc<RwLock<object::Object>>
    {
        let mut o = self.factory.borrow_mut().create_object(name);
        println!("TODO add mesh");
        //o.mesh_render = Some(mesh_render::MeshRender::new("model/skeletonmesh.mesh","material/simple.mat"));
        {
            let c = self.camera.borrow();
            let c = c.object.read().unwrap();
            o.position = c.position + c.orientation.rotate_vec3(&vec::Vec3::new(0f64,0f64,-100f64));
        }

        let ao =  Arc::new(RwLock::new(o));

        let mut list = LinkedList::new();
        list.push_back(ao.clone());
        self.select(list);

        let s = if let Some(ref s) = self.context.borrow_mut().scene {
            s.clone()
            //let mut s = s.write().unwrap();
            //s.objects.push_back(ao.clone());
        }
        else {
            return ao;
        };

        let mut vec = Vec::new();
        vec.push(ao.clone());

        let vs = Vec::new();
        self.request_operation(
            vs,
            operation::OperationData::SceneAddObjects(s.clone(),vec)
            );

        ao

    }o*/

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

fn objects_center(objects : &LinkedList<Arc<RwLock<object::Object>>>) -> vec::Vec3
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

