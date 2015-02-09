use std::collections::DList;
use std::rc::{Rc,Weak};
use std::cell::RefCell;
use std::sync::{RwLock, Arc};
use std::num::Float;
use object;
use mesh;
use vec;
use resource;
use resource::Create;
use shader;
use material;
use transform;
use mesh_render;
use geometry;
use intersection;
use matrix;
use factory;
use camera;

use dragger::{
    TranslationMove,
    create_dragger_translation_group
};
use dragger::{
    ScaleOperation,
    create_scale_draggers
};
use dragger::{
    RotationOperation,
    create_rotation_draggers
};

pub type DraggerGroup = Vec<Rc<RefCell<Dragger>>>;

pub struct DraggerManager
{

    pub draggers : Vec<DraggerGroup>, //Vec<Rc<RefCell<Dragger>>>,
    pub scale_draggers : DraggerGroup, //Vec<Rc<RefCell<Dragger>>>,
    pub scale : f64,
    mouse_start : vec::Vec2,
    mouse : Option<Box<DraggerMouse+'static>>,
    pub ori : vec::Quat,
    current : usize
}

#[derive(Copy)]
pub enum State
{
    Idle,
    Highlight,
    Selected,
    LowLight,
    Hide,
    ShowSecond
}

pub enum Kind
{
    Translate,
    Scale,
    Rotate
}

pub enum Operation
{
    Translation(vec::Vec3),
    Scale(vec::Vec3),
    Rotation(vec::Quat)
}

#[derive(Copy)]
pub enum Repere
{
    Global,
    Local
}

pub struct Dragger
{
    pub object : Arc<RwLock<object::Object>>,
    //pub aabox : geometry::AABox,
    pub ori : transform::Orientation,
    constraint : vec::Vec3,
    kind : Kind,
    color : vec::Vec4,
    repere : Repere,
    translation_start : vec::Vec3,
}

impl DraggerManager
{
    pub fn new(factory : &mut factory::Factory) -> DraggerManager
    {
        let mut dm = DraggerManager {
            draggers : Vec::with_capacity(3),
            scale_draggers : Vec::with_capacity(4),
            scale : 1f64,
            mouse_start : vec::Vec2::zero(),
            mouse : None,
            ori : vec::Quat::identity(),
            current : 2us
        };

        let tr = create_dragger_translation_group(factory);
        dm.draggers.push(tr);

        //dm.create_scale_draggers(factory);
        let sc = create_scale_draggers(factory);
        dm.draggers.push(sc);

        let sc = create_rotation_draggers(factory);
        dm.draggers.push(sc);

        dm
    }

    pub fn mouse_down(&mut self, c : &camera::Camera, button : i32, x : i32, y : i32) -> bool
    {
        self.mouse_start.x = x as f64;
        self.mouse_start.y = y as f64;
        let r = c.ray_from_screen(x as f64, y as f64, 10000f64);
        return self.check_collision(r, button);
    }

    pub fn mouse_up(&mut self, c : &camera::Camera, button : i32, x : i32, y : i32)
        -> Option<Operation>
    {
        self.set_state(State::Idle);

        let op = if let Some(ref m) = self.mouse {
            m.mouse_move(
                c,
                self.mouse_start,
                vec::Vec2::new(x as f64, y as f64))
        }
        else  {
            return None;
        };

        self.mouse = None;
        return op;
    }

    pub fn check_collision(&mut self, r: geometry::Ray, button : i32) -> bool
    {
        let mut found_length = 0f64;
        let mut closest_dragger = None;
        for dragger in self.draggers[self.current].iter_mut() {
            let mut d = dragger.borrow_mut();
            d.set_state(State::Idle);
            let (hit, len) = d.check_collision(&r, self.scale);
            if hit {
                if let None = closest_dragger {
                    closest_dragger = Some(dragger.clone());
                    found_length = len;
                }
                else if len < found_length {
                    closest_dragger = Some(dragger.clone());
                    found_length = len;
                }
            }
        }

        if let Some(d) = closest_dragger {
            match button {
                0i32 => d.borrow_mut().set_state(State::Highlight),
                1i32 => {
                    let mut dragger = d.borrow_mut();
                    dragger.set_state(State::Selected);
                    match dragger.kind {
                        Kind::Translate => {
                            let ob = dragger.object.read().unwrap();
                            //println!("ori : {:?}", self.ori);

                            self.mouse = Some(box TranslationMove::new(
                                    ob.position.clone(),
                                    dragger.constraint,
                                    dragger.repere,
                                    self.ori
                                    ) as Box<DraggerMouse>);
                        }
                        Kind::Scale => {
                            let ob = dragger.object.read().unwrap();

                            self.mouse = Some(box ScaleOperation::new(
                                    ob.position.clone(),
                                    dragger.constraint,
                                    dragger.repere,
                                    self.ori
                                    ) as Box<DraggerMouse>);
                        }
                        _ => {println!("todo");}
                    }
                }
                _ => {}
            };
            return true;
        }
        else {
            return false;
        }
    }

    pub fn set_position(&mut self, p : vec::Vec3) {
        for d in self.draggers[self.current].iter_mut() {
            d.borrow_mut().object.write().unwrap().position = p;
        }

        for d in self.scale_draggers.iter_mut() {
            d.borrow_mut().object.write().unwrap().position = p;
        }
    }

    pub fn set_orientation(&mut self, ori : transform::Orientation) {
        self.ori = ori.as_quat();
        for d in self.draggers[self.current].iter_mut() {
            let mut d = d.borrow_mut();
            d.object.write().unwrap().orientation = ori * d.ori;
        }

        for d in self.scale_draggers.iter_mut() {
            let mut d = d.borrow_mut();
            d.object.write().unwrap().orientation = ori * d.ori;
        }
    }

    pub fn set_scale(&mut self, scale : f64) {
        self.scale = scale;
    }

    pub fn get_objects(&self) -> DList<Arc<RwLock<object::Object>>>
    {
        let mut l = DList::new();
        for d in self.draggers[self.current].iter() {
        //for d in self.scale_draggers.iter() {
            l.push_back(d.borrow().object.clone());
        }

        l
    }

    pub fn set_state(&mut self, state : State) {
        for d in self.draggers[self.current].iter_mut() {
            d.borrow_mut().set_state(state);
        }

        for d in self.scale_draggers.iter_mut() {
            d.borrow_mut().set_state(state);
        }
    }

    pub fn mouse_move(
        &mut self,
        camera : &camera::Camera,
        cur_x : f64,
        cur_y : f64) -> Option<Operation>
    {
        if let Some(ref m) = self.mouse {
            return m.mouse_move(
                camera,
                self.mouse_start,
                vec::Vec2::new(cur_x, cur_y));
        }

        None
    }
}

fn create_dragger(
    factory : &mut factory::Factory,
    name : &str,
    mesh : &str,
    color : vec::Vec4) -> object::Object
{
    let mut dragger = factory.create_object(name);
    let mat = create_mat_res(color, name);

    dragger.mesh_render = 
        Some(mesh_render::MeshRender::new_with_mat(
        mesh, mat));

    dragger
}


fn create_mat_res(color : vec::Vec4, name : &str) -> resource::ResTT<material::Material>
{
    let mut mat : material::Material = Create::create("material/dragger.mat");
    mat.inittt();
    mat.set_uniform_data(
        "color",
        shader::UniformData::Vec4(color));
    let matarc = Arc::new(RwLock::new(mat));

    let rs = resource::ResTest::ResData(matarc);
    let mr = resource::ResTT::new_with_res("dragger_x_mat", rs);
    //let mr = resource::ResTT::new_with_res(name, rs);

    mr
}

impl Dragger
{
    pub fn new(
        factory : &mut factory::Factory,
        name : &str,
        mesh : &str,
        constraint : vec::Vec3,
        ori : transform::Orientation,
        kind : Kind,
        color : vec::Vec4
        ) -> Dragger
    {
        Dragger {
            //object : Arc::new(RwLock::new(create_dragger_tr(factory, name, color))),
            object : Arc::new(RwLock::new(create_dragger(factory, name, mesh, color))),
            constraint : constraint,
            ori : ori,
            kind : kind,
            color : color,
            repere : Repere::Local,
            translation_start : vec::Vec3::zero(),
        }
    }

    pub fn update_scale(
        &mut self,
        world : &matrix::Matrix4,
        projection : &matrix::Matrix4)
    {
        
    }

    fn set_state(&mut self, state : State)
    {
        fn set_color(s : &Self, color : vec::Vec4){
            if let Some(mat) = s.object.write().unwrap().get_material() {
                mat.write().unwrap().set_uniform_data(
                    "color",
                    shader::UniformData::Vec4(color));
            }
        }

        match state {
            State::Highlight => {
                set_color(self, vec::Vec4::new(1f64,1f64,0f64, 1f64));
            },
            State::Selected => {
                set_color(self, vec::Vec4::new(1f64,1f64,1f64, 1f64));
                self.translation_start = self.object.read().unwrap().position.clone();
            },
            State::Idle => {
                set_color(self, self.color);
            }
            _ => {}
        }
    }

    fn check_collision(&self, r : &geometry::Ray, s : f64) -> (bool, f64)
    {
        let ob = self.object.read().unwrap();
        let position = ob.position;
        let rotation = ob.orientation.as_quat();
        let scale = vec::Vec3::new(s, s, s);
        let mesh = match ob.mesh_render {
            None => return (false,0f64),
            Some(ref mr) => {
                match mr.mesh.resource {
                    resource::ResTest::ResData(ref m) => {
                        m.read().unwrap()
                    },
                    _ => return (false,0f64)
                }
            }
        };

        let aabox = match mesh.aabox {
            None => return (false,0f64),
            Some(ref aa) => aa
        };

        let ir = intersection::intersection_ray_box(r, aabox, &position, &rotation, &scale);
        //let ir = intersection::ray_object(r, &*o.read().unwrap());
        if ir.hit {
            let length = (ir.position - r.start).length2();
            return (true, length);
        }
        else {
            return (false,0f64);
        }
    }

}

pub trait DraggerMouse
{
    fn mouse_move(
        &self,
        camera : &camera ::Camera,
        mouse_start : vec::Vec2,
        mouse_end : vec::Vec2)
        -> Option<Operation>;
}

