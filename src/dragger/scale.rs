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

use dragger::manager::{Repere, Operation, DraggerMouse};

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

