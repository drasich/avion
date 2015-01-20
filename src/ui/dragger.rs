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
    pub draggers : DList<Arc<RwLock<object::Object>>>,
    pub scale: f64
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
    pub aabox : geometry::AABox,
    pub scale : f64,
    constraint : vec::Vec3,
    kind : Kind,
}

impl DraggerManager
{
    pub fn new(factory : &mut factory::Factory) -> DraggerManager
    {
        let mut draggers = DList::new();
        draggers.push_back(DraggerManager::create_dragger(factory));
        DraggerManager {
            draggers : draggers,
            scale : 1f64
        }
    }

    fn create_dragger(factory : &mut factory::Factory) -> 
        Arc<RwLock<object::Object>>
        {
            let red = vec::Vec4::new(1.0f64,0.247f64,0.188f64,0.5f64);
            let green = vec::Vec4::new(0.2117f64,0.949f64,0.4156f64,0.5f64);
            let blue = vec::Vec4::new(0f64,0.4745f64,1f64,0.5f64);
            let dragger_parent = 
                Arc::new(RwLock::new(factory.create_object("dragger")));

            let dragger_x = create_dragger_tr(
                factory,
                "dragger_x",
                vec::Quat::new_axis_angle_deg(vec::Vec3::new(0f64,1f64,0f64), 90f64), 
                red);

            let dragger_y = create_dragger_tr(
                factory,
                "dragger_y",
                vec::Quat::new_axis_angle_deg(vec::Vec3::new(1f64,0f64,0f64), -90f64), 
                green);

            let dragger_z = create_dragger_tr(
                factory,
                "dragger_z",
                vec::Quat::identity(), 
                blue);

            object::child_add(dragger_parent.clone(), dragger_x);
            object::child_add(dragger_parent.clone(), dragger_y);
            object::child_add(dragger_parent.clone(), dragger_z);

            dragger_parent
        }

    pub fn check_collision(&self, r: geometry::Ray)
    {
        let mut found_length = 0f64;
        let mut closest_obj = None;
        fn _check(
            r : &geometry::Ray,
            objs : &DList<Arc<RwLock<object::Object>>>,
            found_length : &mut f64,
            closest_obj : &mut Option<Arc<RwLock<object::Object>>>)
            {
                for o in objs.iter() {
                    let ir = intersection::ray_object(r, &*o.read().unwrap());
                    if ir.hit {
                        let length = (ir.position - r.start).length2();
                        match *closest_obj {
                            None => {
                                *closest_obj = Some(o.clone());
                                *found_length = length;
                            }
                            Some(_) => {
                                if length < *found_length {
                                    *closest_obj = Some(o.clone());
                                    *found_length = length;
                                }
                            }
                        }
                    }
                    let children = &o.read().unwrap().children;
                    _check(r, children, found_length, closest_obj);
                }
            }

        _check(&r, &self.draggers, &mut found_length, &mut closest_obj);

        match closest_obj {
            None => {},
            Some(o) => println!("dragger collision")
        }
    }

    pub fn set_position(&mut self, p : vec::Vec3) {
        for o in self.draggers.iter_mut() {
            o.write().unwrap().position = p;
        }
    }

    pub fn set_orientation(&mut self, ori : transform::Orientation) {
        for o in self.draggers.iter_mut() {
            o.write().unwrap().orientation = ori;
        }
    }
}

fn create_dragger_tr(
    factory : &mut factory::Factory,
    name : &str,
    ori :vec::Quat,
    color : vec::Vec4) -> Arc<RwLock<object::Object>>
{
    let dragger = 
        Arc::new(RwLock::new(factory.create_object("dragger_x")));
    let mat = create_mat_res(color, name);

    dragger.write().unwrap().mesh_render = 
        Some(mesh_render::MeshRender::new_with_mat(
        "model/dragger_arrow.mesh", mat));

    dragger.write().unwrap().orientation = transform::Orientation::Quat(ori);

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
        aabox : geometry::AABox,
        constraint : vec::Vec3,
        kind : Kind,
        ) -> Dragger
    {
        Dragger {
            aabox : aabox,
            scale : 1f64,
            constraint : constraint,
            kind : kind
        }
    }

    pub fn update_scale(
        &mut self,
        world : &matrix::Matrix4,
        projection : &matrix::Matrix4)
    {
        
    }
}

/*
fn _resize_to_cam(
    world : &matrix::Matrix4, 
    projection : &matrix::Matrix4,
    factor : f64)
{
  let tm = projection * world;
  tm = tm.transpose();

  let zero = vec::Vec4::new(0f64,0f64,0f64,1f64);
  let vw = tm * zero;
  let w = vw.w * factor;
  return w;
}
*/

