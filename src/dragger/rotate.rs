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

pub struct RotationOperation
{
    start : vec::Vec3,
    constraint : vec::Vec3,
    repere : Repere,
    ori : vec::Quat,
}

impl RotationOperation {

    fn new(
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

pub fn create_rotation_draggers(factory : &mut factory::Factory)
    -> DraggerGroup
{
    let red = vec::Vec4::new(1.0f64,0.247f64,0.188f64,0.5f64);
    let green = vec::Vec4::new(0.2117f64,0.949f64,0.4156f64,0.5f64);
    let blue = vec::Vec4::new(0f64,0.4745f64,1f64,0.5f64);
    let mesh = "model/dragger_rotate_quarter.mesh";

    let dragger_x = Dragger::new(
        factory,
        "rotate_x",
        mesh,
        vec::Vec3::new(1f64,0f64,0f64),
        transform::Orientation::Quat(vec::Quat::new_axis_angle_deg(vec::Vec3::new(0f64,1f64,0f64), 90f64)),
        Kind::Rotate,
        red);

    let dragger_y = Dragger::new(
        factory,
        "rotate_y",
        mesh,
        vec::Vec3::new(0f64,1f64,0f64),
        transform::Orientation::Quat(vec::Quat::new_axis_angle_deg(vec::Vec3::new(1f64,0f64,0f64), -90f64)), 
        Kind::Rotate,
        green);

    let dragger_z = Dragger::new(
        factory,
        "rotate_z",
        mesh,
        vec::Vec3::new(0f64,0f64,1f64),
        transform::Orientation::Quat(vec::Quat::identity()), 
        Kind::Rotate,
        blue);

    let mut group = Vec::with_capacity(3);

    group.push(Rc::new(RefCell::new(dragger_x)));
    group.push(Rc::new(RefCell::new(dragger_y)));
    group.push(Rc::new(RefCell::new(dragger_z)));

    return group;
}

/*

    pub fn face_camera(&self, c : &camera::Camera, qo : vec::Quat)
        //Quat qo, Object* o, ViewCamera* c)
    {

        Vec3 diff = vec3_sub(o->position, c->object->position);
        double dotx = vec3_dot(diff, quat_rotate_vec3(qo, vec3(1,0,0)));
        double doty = vec3_dot(diff, quat_rotate_vec3(qo, vec3(0,1,0)));
        double dotz = vec3_dot(diff, quat_rotate_vec3(qo, vec3(0,0,1)));
  float angle = 0;
  o->angles.x = 0;
  o->angles.y = 0;
  o->angles.z = 0;
  /*
  Vec3 camx = quat_rotate_vec3(c->object->orientation, vec3(1,0,0));
  printf("camx : %f, %f ,%f\n", camx.x, camx.y, camx.z);
  Vec3 obx = quat_rotate_vec3(qo, vec3(1,0,0));
  double dot = vec3_dot(obx, camx);
  */

  if ( vec3_equal(d->constraint, vec3(0,0,1))) {
    if (dotx >0) {
      if (doty >0)
      angle = 180;
      else
      angle = 90;
    }
    else if (doty >0)
      angle = -90;
  }

  if ( vec3_equal(d->constraint, vec3(0,1,0))) {
    if (dotx >0) {
      if (dotz >0)
      angle = 180;
      else
      angle = 90;
    }
    else if (dotz >0)
      angle = -90;
  }

  if ( vec3_equal(d->constraint, vec3(1,0,0))) {
    if (doty >0) {
      if (dotz >0)
      angle = -180;
      else
      angle = -90;
    }
    else if (dotz >0)
      angle = 90;
  }


  Quat q = quat_yaw_pitch_roll_deg(0,0, angle);

  //o->orientation = quat_mul(q, d->ori);
  o->orientation = quat_mul(d->ori,q);
  o->orientation = quat_mul(qo, o->orientation);
  o->orientation_type = ORIENTATION_QUAT;

}



    }
*/

