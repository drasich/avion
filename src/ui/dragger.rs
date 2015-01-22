use std::collections::DList;
//use std::rc::Rc;
//use std::cell::RefCell;
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

pub struct DraggerManager
{
    //pub parent : Option<Arc<RwLock<object::Object>>>,
    //pub draggers : DList<Arc<RwLock<object::Object>>>,
    pub draggers : DList<Dragger>,
    pub scale : f64
}

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

pub struct Dragger
{
    pub object : Arc<RwLock<object::Object>>,
    //pub aabox : geometry::AABox,
    pub ori : transform::Orientation,
    constraint : vec::Vec3,
    kind : Kind,
    color : vec::Vec4
}

impl DraggerManager
{
    pub fn new(factory : &mut factory::Factory) -> DraggerManager
    {
        let mut dm = DraggerManager {
            draggers : DList::new(),
            scale : 1f64
        };

        dm.create_dragger(factory);

        dm
    }

    fn create_dragger(&mut self, factory : &mut factory::Factory)
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

        self.draggers.push_back(dragger_x);
        self.draggers.push_back(dragger_y);
        self.draggers.push_back(dragger_z);
    }


    pub fn check_collision(&mut self, r: geometry::Ray)
    {
        let mut found_length = 0f64;
        let mut closest_dragger = None;
        for d in self.draggers.iter_mut() {
            d.set_state(State::Idle);
            let (hit, len) =d.check_collision(&r, self.scale);
            if hit {
                if let None = closest_dragger {
                    closest_dragger = Some(d);
                    found_length = len;
                }
                else if len < found_length {
                    closest_dragger = Some(d);
                    found_length = len;
                }
            }
        }

        if let Some(d) = closest_dragger {
            d.set_state(State::Highlight);
        }
    }

    pub fn set_position(&mut self, p : vec::Vec3) {
        for d in self.draggers.iter_mut() {
            d.object.write().unwrap().position = p;
        }
    }

    pub fn set_orientation(&mut self, ori : transform::Orientation) {
        for d in self.draggers.iter_mut() {
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
            l.push_back(d.object.clone());
        }

        l
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
            color : color
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
        let set_color = |&: color : vec::Vec4|
        {
            if let Some(mat) = self.object.write().unwrap().get_material() {
                mat.write().unwrap().set_uniform_data(
                    "color",
                    shader::UniformData::Vec4(color));
            }
        };

        match state {
            State::Highlight => {
                set_color(vec::Vec4::new(1f64,1f64,0f64, 1f64));
            },
            State::Idle => {
                set_color(self.color);
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

