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

use dragger::manager::{
    Repere,
    Operation,
    DraggerMouse,
    DraggerGroup,
    Kind,
    Dragger
};


pub struct ScaleOperation
{
    start : vec::Vec3,
    constraint : vec::Vec3,
    repere : Repere,
    ori : vec::Quat
}

impl ScaleOperation {

    pub fn new(
        start : vec::Vec3,
        constraint : vec::Vec3, 
        repere : Repere,
        ori : vec::Quat
        ) -> ScaleOperation
    {
        ScaleOperation {
            start : start,
            constraint : constraint,
            repere : repere,
            ori : ori
        }
    }

    fn local(
        &self,
        camera : &camera::Camera,
        mouse_start : vec::Vec2,
        mouse_end : vec::Vec2) -> Option<Operation>
    {
        let ss = camera.world_to_screen(self.start);

        let sss = mouse_start - ss;
        let l1 = sss.length2();
        let sd = mouse_end - ss;
        let l2 = sd.length2();

        let mut fac = l2/l1;
        let dot = sss.dot(sd);
        if dot < 0f64 {
            fac *= -1f64;
        }

        let mut scale_factor = vec::Vec3::new(fac,fac,fac);
        if self.constraint.x == 0f64 {
            scale_factor.x = 1f64;
        }
        if self.constraint.y == 0f64 {
            scale_factor.y = 1f64;
        }
        if self.constraint.z == 0f64 {
            scale_factor.z = 1f64;
        }

        return Some(Operation::Scale(scale_factor));
    }
}


impl DraggerMouse for ScaleOperation {

    fn mouse_move(
        &self,
        camera : &camera::Camera,
        mouse_start : vec::Vec2,
        mouse_end : vec::Vec2) -> Option<Operation>
    {
        return self.local(camera, mouse_start, mouse_end);
    }
}

pub fn create_scale_draggers(factory : &mut factory::Factory)
    -> DraggerGroup
{
    let red = vec::Vec4::new(1.0f64,0.247f64,0.188f64,0.5f64);
    let green = vec::Vec4::new(0.2117f64,0.949f64,0.4156f64,0.5f64);
    let blue = vec::Vec4::new(0f64,0.4745f64,1f64,0.5f64);
    let mesh = "model/dragger_scale.mesh";

    let dragger_x = Dragger::new(
        factory,
        "scale_x",
        mesh,
        vec::Vec3::new(1f64,0f64,0f64),
        transform::Orientation::Quat(vec::Quat::new_axis_angle_deg(vec::Vec3::new(0f64,1f64,0f64), 90f64)),
        Kind::Scale,
        red);

    let dragger_y = Dragger::new(
        factory,
        "scale_y",
        mesh,
        vec::Vec3::new(0f64,1f64,0f64),
        transform::Orientation::Quat(vec::Quat::new_axis_angle_deg(vec::Vec3::new(1f64,0f64,0f64), -90f64)), 
        Kind::Scale,
        green);

    let dragger_z = Dragger::new(
        factory,
        "scale_z",
        mesh,
        vec::Vec3::new(0f64,0f64,1f64),
        transform::Orientation::Quat(vec::Quat::identity()), 
        Kind::Scale,
        blue);

    let mut group = Vec::with_capacity(3);

    group.push(Rc::new(RefCell::new(dragger_x)));
    group.push(Rc::new(RefCell::new(dragger_y)));
    group.push(Rc::new(RefCell::new(dragger_z)));

    return group;
}

