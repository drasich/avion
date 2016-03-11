use std::collections::LinkedList;
use std::rc::{Rc,Weak};
use std::cell::RefCell;
use std::sync::{RwLock, Arc};
use dormin::object;
use dormin::mesh;
use dormin::vec;
use dormin::resource;
use dormin::resource::Create;
use dormin::shader;
use dormin::material;
use dormin::transform;
use dormin::component::mesh_render;
use dormin::geometry;
use dormin::intersection;
use dormin::matrix;
use dormin::factory;
use dormin::camera;
use uuid;

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
    mouse_start : vec::Vec2,
    mouse : Option<Box<DraggerMouse+'static>>,
    pub ori : vec::Quat,
    current_group : usize,
    dragger_focus : Option<uuid::Uuid>
}

#[derive(Copy,Clone,Debug)]
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

#[derive(Copy,Clone)]
pub enum Repere
{
    Global,
    Local
}

pub enum Collision
{
    MeshAABox,
    Mesh,
    SpecialMesh(resource::ResTT<mesh::Mesh>)
}

pub struct Dragger
{
    pub object : Arc<RwLock<object::Object>>,
    pub ori : transform::Orientation,
    pub constraint : vec::Vec3,
    kind : Kind,
    color : vec::Vec4,
    repere : Repere,
    collision : Collision,
    state : State,
    scale : f64,
    id : uuid::Uuid
}

impl DraggerManager
{
    pub fn new(factory : &factory::Factory, resource : &resource::ResourceGroup) -> DraggerManager
    {
        let mut dm = DraggerManager {
            draggers : Vec::with_capacity(3),
            mouse_start : vec::Vec2::zero(),
            mouse : None,
            ori : vec::Quat::identity(),
            current_group : 0usize,
            dragger_focus : None
        };

        let tr = create_dragger_translation_group(factory, resource);
        dm.draggers.push(tr);

        let sc = create_scale_draggers(factory, resource);
        dm.draggers.push(sc);

        let sc = create_rotation_draggers(factory, resource);
        dm.draggers.push(sc);

        dm
    }

    pub fn mouse_down(&mut self, c : &camera::Camera, button : i32, x : i32, y : i32) -> bool
    {
        self.mouse_start.x = x as f64;
        self.mouse_start.y = y as f64;
        let r = c.ray_from_screen(x as f64, y as f64, 10000f64);

        self.check_collision(r, button).is_some()
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

    pub fn check_collision(&mut self, r: geometry::Ray, button : i32) -> Option<uuid::Uuid>
    {
        let mut found_length = 0f64;
        let mut closest_dragger = None;
        for dragger in &mut self.draggers[self.current_group] {
            let mut d = dragger.borrow_mut();
            d.set_state(State::Idle);
            let (hit, len) = d.check_collision(&r, d.scale);
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
                        Kind::Rotate => {
                            let ob = dragger.object.read().unwrap();
                            //println!("ori : {:?}", self.ori);

                            self.mouse = Some(box RotationOperation::new(
                                    ob.position.clone(),
                                    dragger.constraint,
                                    dragger.repere,
                                    self.ori
                                    ) as Box<DraggerMouse>);
                        }
                    }
                }
                _ => {}
            };
            Some(d.borrow().id)
        }
        else {
            None
        }
    }

    pub fn mouse_move_hover(&mut self, r: geometry::Ray, button : i32) -> bool
    {
        let result = self.check_collision(r, button);

        if let Some(id) = result {
            if let Some(focus) = self.dragger_focus {
                if focus == id {
                    return false;
                }
            }
        }
        else {
            if self.dragger_focus.is_none(){
                return false;
            }
        }

        self.dragger_focus = result;
        true
    }

    pub fn set_position(&mut self, p : vec::Vec3) {
        for d in &mut self.draggers[self.current_group] {
            let mut db =d.borrow_mut();
            db.object.write().unwrap().position = p;
        }

    }

    pub fn set_orientation(&mut self, ori : transform::Orientation, camera : &camera::Camera) {
        self.ori = ori.as_quat();
        for d in &mut self.draggers[self.current_group] {
            let mut d = d.borrow_mut();
            if self.current_group == 2usize {
                d.face_camera(camera, self.ori);
            }
            else {
            d.object.write().unwrap().orientation = ori * d.ori;
            }
        }
    }

    pub fn scale_to_camera(&mut self, camera : &camera::Camera)
    {
        let cam_mat = camera.object.read().unwrap().get_world_matrix();
        let projection = camera.get_perspective();
        let cam_mat_inv = cam_mat.get_inverse();

        for d in &mut self.draggers[self.current_group] {

            d.borrow_mut().scale_to_camera_data(&cam_mat_inv, &projection);
        }
    }

    pub fn get_objects(&self) -> Vec<Arc<RwLock<object::Object>>>
    {
        let mut l = Vec::new();
        for d in &self.draggers[self.current_group] {
            l.push(d.borrow().object.clone());
        }

        l
    }

    pub fn set_state(&mut self, state : State) {
        for d in &mut self.draggers[self.current_group] {
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

    pub fn change(&mut self)
    {
        let mut newlen = self.current_group + 1;
        if newlen >= self.draggers.len() {
            newlen = 0;
        }

        self.current_group = newlen;
    }

}

fn create_dragger(
    factory : &factory::Factory,
    resource : &resource::ResourceGroup,
    name : &str,
    mesh : &str,
    color : vec::Vec4) -> object::Object
{
    let mut dragger = factory.create_object(name);
    let mat = create_mat(color, name);

    dragger.mesh_render = Some(mesh_render::MeshRenderer::new_with_mat(
        mesh,
        mat,
        resource));

    dragger
}

fn create_mat(color : vec::Vec4, name : &str) -> Arc<RwLock<material::Material>>
{
    let mut mat : material::Material = Create::create("material/dragger.mat");
    mat.inittt();

    if let Some(ref mut s) = mat.shader {
        s.load_instant_no_manager();
    }

    mat.set_uniform_data(
        "color",
        shader::UniformData::Vec4(color));

    let matarc = Arc::new(RwLock::new(mat));

    matarc
}

fn create_mat_res(color : vec::Vec4, name : &str) -> resource::ResTT<material::Material>
{
    let matarc = create_mat(color, name);

    let rs = resource::ResTest::ResData(matarc);
    let mr = resource::ResTT::new_with_res("dragger_x_mat", rs);
    //let mr = resource::ResTT::new_with_res(name, rs);

    mr
}

impl Dragger
{
    pub fn new(
        factory : &factory::Factory,
        resource : &resource::ResourceGroup,
        name : &str,
        mesh : &str,
        constraint : vec::Vec3,
        ori : transform::Orientation,
        kind : Kind,
        color : vec::Vec4,
        collision : Collision
        ) -> Dragger
    {
        Dragger {
            object : Arc::new(RwLock::new(create_dragger(factory, resource, name, mesh, color))),
            constraint : constraint,
            ori : ori,
            kind : kind,
            color : color,
            repere : Repere::Local,
            collision : collision,
            state : State::Idle,
            scale : 1f64,
            id : uuid::Uuid::new_v4()
        }
    }

    pub fn update_scale(
        &mut self,
        world : &matrix::Matrix4,
        projection : &matrix::Matrix4)
    {
        
    }

    fn scale_to_camera_data(
        &mut self,
        cam_mat_inv : &matrix::Matrix4,
        projection : &matrix::Matrix4)
    {
        let world_inv = cam_mat_inv * &self.object.read().unwrap().get_world_matrix();

        let mut tm = projection * &world_inv;
        tm = tm.transpose();

        let zero = vec::Vec4::new(0f64,0f64,0f64,1f64);
        let vw = &tm * zero;
        let factor = 0.05f64;
        let w = vw.w * factor;
        self.scale = w;
    }

    fn set_state(&mut self, state : State)
    {
        fn set_color(s : &Dragger, color : vec::Vec4){
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
            },
            State::Idle => {
                set_color(self, self.color);
            }
            _ => {}
        }

        self.state = state;
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
                mr.mesh.read().unwrap()
            },
        };

        let aabox = match mesh.aabox {
            None => return (false,0f64),
            Some(ref aa) => aa
        };

        let ir = intersection::intersection_ray_box(r, aabox, &position, &rotation, &scale);
        //let ir = intersection::ray_object(r, &*o.read().unwrap());
        if ir.hit {
            let length = (ir.position - r.start).length2();

            //TODO 
            let m = match self.collision{
                Collision::SpecialMesh(ref r) => {
                    match r.resource {
                        resource::ResTest::ResData(ref m) => {
                            m
                        },
                        _ => { return (true, length); }
                    }
                },
                _ => return (true, length)
            };

            let ir = intersection::ray_mesh(r, &*m.read().unwrap(), &position, &rotation, &scale);
            if ir.hit {
                let length = (ir.position - r.start).length2();
                return (true, length);
            }
            else {
                return (false, 0f64);
            }
        }
        else {
            return (false,0f64);
        }
    }

    pub fn face_camera(
        &self,
        camera : &camera::Camera,
        manager_ori : vec::Quat,
        )
    {
        let qo = manager_ori;
        let mut o = self.object.write().unwrap();
        let constraint = self.constraint;
        let dragger_ori = self.ori.as_quat();


        let camera_object = camera.object.read().unwrap();

        let diff = o.position - camera_object.position;
        let dotx = diff.dot(&qo.rotate_vec3(&vec::Vec3::x()));
        let doty = diff.dot(&qo.rotate_vec3(&vec::Vec3::y()));
        let dotz = diff.dot(&qo.rotate_vec3(&vec::Vec3::z()));
        let mut angle = 0f64;

        if constraint ==  vec::Vec3::new(0f64,0f64,1f64) {
            if dotx > 0f64 {
                if doty > 0f64 {
                    angle = 180f64;
                }
                else {
                    angle = 90f64;
                }
            }
            else if doty > 0f64 {
                angle = -90f64;
            }
        }

        if constraint == vec::Vec3::new(0f64,1f64,0f64) {
            if dotx > 0f64 {
                if dotz > 0f64 {
                    angle = 180f64;
                }
                else {
                    angle = 90f64;
                }
            }
            else if dotz > 0f64 {
                angle = -90f64;
            }
        }

        if constraint == vec::Vec3::new(1f64,0f64,0f64) {
            if doty > 0f64 {
                if dotz > 0f64 {
                    angle = -180f64;
                }
                else {
                    angle = -90f64;
                }
            }
            else if dotz > 0f64 {
                angle = 90f64;
            }
        }


        let q = vec::Quat::new_yaw_pitch_roll_deg(0f64,0f64, angle);
        let qoo = dragger_ori *q;
        let qf = qo * qoo;

        o.orientation = transform::Orientation::Quat(qf);
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

