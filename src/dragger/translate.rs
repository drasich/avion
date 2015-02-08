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

pub struct TranslationMove
{
    translation_start : vec::Vec3,
    constraint : vec::Vec3,
    repere : Repere,
    ori : vec::Quat
}


impl TranslationMove {
    pub fn new(
        start : vec::Vec3,
        constraint : vec::Vec3, 
        repere : Repere,
        ori : vec::Quat
        ) -> TranslationMove
    {
        TranslationMove {
            translation_start : start,
            constraint : constraint,
            repere : repere,
            ori : ori
        }
    }

    fn global(
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

            //let pos = self.translation_start + translation;
            //return Some(Operation::Translation(pos));
            return Some(Operation::Translation(translation));
        }
        else {
            return None;
        }
    }

    fn local(
        &self,
        camera : &camera::Camera,
        mouse_start : vec::Vec2,
        mouse_end : vec::Vec2) -> Option<Operation>
    {
        let constraint = self.constraint;
        let ori = self.ori;

        println!("constraint : {:?}, ori: {:?}", constraint, ori);

        let camup = camera.object.read().unwrap().orientation.rotate_vec3(&vec::Vec3::new(0f64,1f64,0f64));

        //printf("dragger ori : %f, %f, %f %f \n ", c->dragger_ori.x, c->dragger_ori.y, c->dragger_ori.z, c->dragger_ori.w);
        let ca = ori.rotate_vec3(&constraint);
        let cax = ori.rotate_vec3(&vec::Vec3::new(constraint.x,0f64,0f64));
        let cay = ori.rotate_vec3(&vec::Vec3::new(0f64,constraint.y,0f64));
        let caz = ori.rotate_vec3(&vec::Vec3::new(0f64,0f64,constraint.z));
        //printf("ca %f, %f, %f \n", ca.x, ca.y, ca.z);
        let mut n = camup ^ ca;
        if constraint == vec::Vec3::new(1f64,1f64,0f64) {
            n = ori.rotate_vec3(&vec::Vec3::new(0f64,0f64,1f64));
        }
        else if constraint == vec::Vec3::new(1f64,0f64,1f64) {
            n = ori.rotate_vec3(&vec::Vec3::new(0f64,1f64,0f64));
        }
        else if constraint == vec::Vec3::new(0f64,1f64,1f64) {
            n = ori.rotate_vec3(&vec::Vec3::new(1f64,0f64,0f64));
        }

        n.normalize();
        let mut p = geometry::Plane{ point : self.translation_start, normal : n };
        //printf("n %f, %f, %f \n", n.x, n.y, n.z);

        if constraint == vec::Vec3::new(0f64,1f64,0f64) {//TODO change this by checking the angle between camup and ca
            let camright = camera.object.read().unwrap().orientation.rotate_vec3(&vec::Vec3::new(1f64,0f64,0f64));
            p.normal = camright ^ ca;
        }

        let rstart = camera.ray_from_screen(mouse_start.x, mouse_start.y, 1f64);
        let r = camera.ray_from_screen(mouse_end.x, mouse_end.y, 1f64);

        let ir = intersection::intersection_ray_plane(&r, &p);
        let irstart = intersection::intersection_ray_plane(&rstart, &p);

        if ir.hit && irstart.hit {
            let mut translation = ir.position - irstart.position;
            //printf("translation %f, %f, %f \n", translation.x, translation.y, translation.z);
            if constraint == vec::Vec3::new(1f64,0f64,0f64) ||
               constraint == vec::Vec3::new(0f64,1f64,0f64) ||
               constraint == vec::Vec3::new(0f64,0f64,1f64) {
                   let dot = ca.dot(&translation);
                   translation = ca * dot;
            }

            //let pos = self.translation_start + translation;
            //return Some(Operation::Translation(pos));
            return Some(Operation::Translation(translation));
        }
        else {
            return None;
        }
    }

}

impl DraggerMouse for TranslationMove {

    fn mouse_move(
        &self,
        camera : &camera::Camera,
        mouse_start : vec::Vec2,
        mouse_end : vec::Vec2) -> Option<Operation>
    {
        match self.repere {
            Repere::Global => {
                return self.global(camera, mouse_start, mouse_end);
            },
            Repere::Local => {
                return self.local(camera, mouse_start, mouse_end);
            },
        }
    }
}

