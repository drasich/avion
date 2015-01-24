use std::collections::DList;
use std::rc::{Rc,Weak};
use std::cell::RefCell;
use std::sync::{RwLock, Arc};
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

pub struct DraggerManager
{
    //pub parent : Option<Arc<RwLock<object::Object>>>,
    //pub draggers : DList<Arc<RwLock<object::Object>>>,
    //pub draggers : DList<Rc<RefCell<Box<DraggerMouse>>>>,
    pub draggers : DList<Rc<RefCell<Dragger>>>,
    pub scale : f64,
    current : Option<Weak<RefCell<Dragger>>>,
    mouse_start : vec::Vec2,
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
    Rotation(transform::Orientation)
}

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
    translation_start : vec::Vec3
}

impl DraggerManager
{
    pub fn new(factory : &mut factory::Factory) -> DraggerManager
    {
        let mut dm = DraggerManager {
            draggers : DList::new(),
            scale : 1f64,
            current : None,
            mouse_start : vec::Vec2::zero(),
        };

        dm.create_draggers(factory);

        dm
    }

    fn create_draggers(&mut self, factory : &mut factory::Factory)
    {
        let red = vec::Vec4::new(1.0f64,0.247f64,0.188f64,0.5f64);
        let green = vec::Vec4::new(0.2117f64,0.949f64,0.4156f64,0.5f64);
        let blue = vec::Vec4::new(0f64,0.4745f64,1f64,0.5f64);
        let dragger_parent = 
            Arc::new(RwLock::new(factory.create_object("dragger")));

        let dragger_x = Dragger::new(
            factory,
            "dragger_x",
            vec::Vec3::new(1f64,0f64,0f64),
            transform::Orientation::Quat(vec::Quat::new_axis_angle_deg(vec::Vec3::new(0f64,1f64,0f64), 90f64)),
            Kind::Translate,
            red);

        let dragger_y = Dragger::new(
            factory,
            "dragger_y",
            vec::Vec3::new(0f64,1f64,0f64),
            transform::Orientation::Quat(vec::Quat::new_axis_angle_deg(vec::Vec3::new(1f64,0f64,0f64), -90f64)), 
            Kind::Translate,
            green);

        let dragger_z = Dragger::new(
            factory,
            "dragger_z",
            vec::Vec3::new(0f64,0f64,1f64),
            transform::Orientation::Quat(vec::Quat::identity()), 
            Kind::Translate,
            blue);

        self.draggers.push_back(Rc::new(RefCell::new(dragger_x)));
        self.draggers.push_back(Rc::new(RefCell::new(dragger_y)));
        self.draggers.push_back(Rc::new(RefCell::new(dragger_z)));
    }

    pub fn mouse_down(&mut self, c : &camera::Camera, button : i32, x : i32, y : i32) -> bool
    {
        self.mouse_start.x = x as f64;
        self.mouse_start.y = y as f64;
        let r = c.ray_from_screen(x as f64, y as f64, 10000f64);
        return self.check_collision(r, button);
    }


    pub fn check_collision(&mut self, r: geometry::Ray, button : i32) -> bool
    {
        let mut found_length = 0f64;
        let mut closest_dragger = None;
        for dragger in self.draggers.iter_mut() {
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
                    d.borrow_mut().set_state(State::Selected);
                    self.current = Some(d.downgrade());
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
        for d in self.draggers.iter_mut() {
            d.borrow_mut().object.write().unwrap().position = p;
        }
    }

    pub fn set_orientation(&mut self, ori : transform::Orientation) {
        for d in self.draggers.iter_mut() {
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
        for d in self.draggers.iter() {
            l.push_back(d.borrow().object.clone());
        }

        l
    }

    pub fn set_state(&mut self, state : State) {
        for d in self.draggers.iter_mut() {
            d.borrow_mut().set_state(state);
        }
    }

    pub fn mouse_move(
        &mut self,
        camera : &camera::Camera,
        cur_x : f64,
        cur_y : f64) -> Option<Operation>
    {
        if let Some(ref d) = self.current {
            if let Some(dd) = d.upgrade() {
                let dragger = dd.borrow_mut();
                return dragger.mouse_move(
                    camera,
                    self.mouse_start,
                    vec::Vec2::new(cur_x, cur_y));
            }
        }

        None
    }
}

fn create_dragger_tr(
    factory : &mut factory::Factory,
    name : &str,
    //ori :vec::Quat,
    color : vec::Vec4) -> object::Object
{
    let mut dragger = factory.create_object(name);
    let mat = create_mat_res(color, name);

    dragger.mesh_render = 
        Some(mesh_render::MeshRender::new_with_mat(
        "model/dragger_arrow.mesh", mat));

    //dragger.orientation = transform::Orientation::Quat(ori);

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
        //aabox : geometry::AABox,
        constraint : vec::Vec3,
        ori : transform::Orientation,
        kind : Kind,
        color : vec::Vec4
        ) -> Dragger
    {
        Dragger {
            object : Arc::new(RwLock::new(create_dragger_tr(factory, name, color))),
            //aabox : aabox,
            constraint : constraint,
            ori : ori,
            kind : kind,
            color : color,
            repere : Repere::Global,
            translation_start : vec::Vec3::zero()
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


trait DraggerMouse
{
    fn mouse_move(
        &self,
        camera : &camera ::Camera,
        mouse_start : vec::Vec2,
        mouse_end : vec::Vec2)
        -> Option<Operation>;
}

impl DraggerMouse for Dragger {
fn mouse_move(
    &self,
    camera : &camera::Camera,
    mouse_start : vec::Vec2,
    mouse_end : vec::Vec2) -> Option<Operation>
{
    let mut p = geometry::Plane {
        point : self.translation_start,
        normal : camera.object.read().unwrap().orientation.rotate_vec3(
            &vec::Vec3::new(0f64,0f64,-1f64))
    };

    let constraint = self.constraint;

    if constraint != vec::Vec3::new(1f64,1f64,1f64) {
        if constraint.z == 1f64 {
            p.normal.z = 0f64;
        }
        if constraint.y == 1f64 {
            p.normal.y = 0f64;
        }
        if constraint.x == 1f64 {
            p.normal.x = 0f64;
        }
    }

    p.normal = p.normal.normalized();

    let rstart = camera.ray_from_screen(mouse_start.x, mouse_start.y, 1f64);
    let r = camera.ray_from_screen(mouse_end.x, mouse_end.y, 1f64);

    let irstart = intersection::intersection_ray_plane(&rstart, &p);
    let ir = intersection::intersection_ray_plane(&r, &p);

    if ir.hit && irstart.hit {
        let mut translation = ir.position - irstart.position;
        translation = translation * constraint;

        let pos = self.translation_start + translation;
        return Some(Operation::Translation(pos));
    }
    else {
        return None;
    }
}
}
