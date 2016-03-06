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
use dormin::geometry;
use dormin::intersection;
use dormin::matrix;
use dormin::factory;
use dormin::camera;

use dragger::manager::{
    Repere,
    Operation,
    DraggerMouse,
    DraggerGroup,
    Kind,
    Collision,
    Dragger
};

pub struct RotationOperation
{
    start : vec::Vec3,
    constraint : vec::Vec3,
    repere : Repere,
    ori : vec::Quat,
}

impl RotationOperation {

    pub fn new(
        start : vec::Vec3,
        constraint : vec::Vec3, 
        repere : Repere,
        ori : vec::Quat
        ) -> RotationOperation
    {
        RotationOperation {
            start : start,
            constraint : constraint,
            repere : repere,
            ori : ori
        }
    }

    pub fn local_global( 
        &self,
        camera : &camera::Camera,
        mouse_start : vec::Vec2,
        mouse_end : vec::Vec2) -> Option<Operation>
    {
        let rstart = camera.ray_from_screen(mouse_start.x, mouse_start.y, 1f64);

        let r = camera.ray_from_screen(mouse_end.x, mouse_end.y, 1f64);

        let normal = camera.object.read().unwrap().orientation.rotate_vec3(&vec::Vec3::new(0f64,0f64,1f64));
        let p = geometry::Plane { point : self.start, normal : normal };

        let irstart =  intersection::intersection_ray_plane(&rstart, &p);
        let ir =  intersection::intersection_ray_plane(&r, &p);

        let yos = (irstart.position - self.start).normalized();
        let yoe = (ir.position - self.start).normalized();

        let mdot = yos.dot(&yoe);

        let cross = yos^yoe;
        let sign = normal.dot(&cross);
        let mut angle = mdot.acos();

        let diff = self.start - camera.object.read().unwrap().position;
        let cons = if let Repere::Local = self.repere {
            self.ori.rotate_vec3(&self.constraint)
        }
        else {
            self.constraint
        };

        let dotori = diff.dot(&cons);

        if dotori > 0f64 {
            if sign > 0f64 { 
                angle *= -1f64;
            }
        }
        else {
            if sign < 0f64 {
                angle *= -1f64;
            }
        }

        let qrot = vec::Quat::new_axis_angle_rad(self.constraint, angle);

        return Some(Operation::Rotation(qrot));
    }
}

pub fn create_rotation_draggers(factory : &factory::Factory, resource : &resource::ResourceGroup)
    -> DraggerGroup
{
    let red = vec::Vec4::new(1.0f64,0.247f64,0.188f64,0.5f64);
    let green = vec::Vec4::new(0.2117f64,0.949f64,0.4156f64,0.5f64);
    let blue = vec::Vec4::new(0f64,0.4745f64,1f64,0.5f64);
    let mesh = "model/dragger_rotate_quarter.mesh";
    let collider = "model/dragger_rotate_collider_quarter.mesh";
    let collider_mesh : resource::ResTT<mesh::Mesh> = 
        resource::ResTT::new_instant(collider, &mut *factory.mesh_manager.write().unwrap());

    let dragger_x = Dragger::new(
        factory,
        resource,
        "rotate_x",
        mesh,
        vec::Vec3::new(1f64,0f64,0f64),
        transform::Orientation::Quat(vec::Quat::new_axis_angle_deg(
                vec::Vec3::new(0f64,1f64,0f64), -90f64)),
        Kind::Rotate,
        red,
        Collision::SpecialMesh(collider_mesh.clone())
        );

    let dragger_y = Dragger::new(
        factory,
        resource,
        "rotate_y",
        mesh,
        vec::Vec3::new(0f64,1f64,0f64),
        transform::Orientation::Quat(vec::Quat::new_axis_angle_deg(
                vec::Vec3::new(1f64,0f64,0f64), 90f64)), 
        Kind::Rotate,
        green,
        Collision::SpecialMesh(collider_mesh.clone())
        );

    let dragger_z = Dragger::new(
        factory,
        resource,
        "rotate_z",
        mesh,
        vec::Vec3::new(0f64,0f64,1f64),
        transform::Orientation::Quat(vec::Quat::identity()), 
        Kind::Rotate,
        blue,
        Collision::SpecialMesh(collider_mesh)
        );

    let mut group = Vec::with_capacity(3);

    group.push(Rc::new(RefCell::new(dragger_x)));
    group.push(Rc::new(RefCell::new(dragger_y)));
    group.push(Rc::new(RefCell::new(dragger_z)));

    return group;
}

impl DraggerMouse for RotationOperation {

    fn mouse_move(
        &self,
        camera : &camera::Camera,
        mouse_start : vec::Vec2,
        mouse_end : vec::Vec2) -> Option<Operation>
    {
        return self.local_global(camera, mouse_start, mouse_end);
    }
}

