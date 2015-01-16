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

use factory;

pub struct Dragger
{
    //pub parent : Option<Arc<RwLock<object::Object>>>,
    pub draggers : DList<Arc<RwLock<object::Object>>>,
}

impl Dragger
{
    pub fn new(factory : &mut factory::Factory) -> Dragger
    {
        let mut draggers = DList::new();
        draggers.push_back(Dragger::create_dragger(factory));
        Dragger {
            draggers : draggers
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

