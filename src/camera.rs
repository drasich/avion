use std::sync::{RWLock, Arc};
use std::f64::consts;
use std::default::Default;
use std::num::FloatMath;
use std::num::Float;

use vec;
use vec::{Vec3};
use object;
use matrix;
use geometry;
use transform::Orientation;
use uuid;

pub enum Projection
{
    Perspective,
    Orthographic
}

pub struct CameraData
{
    fovy : f64,
    pub fovy_base : f64,
    pub near : f64,
    pub far : f64,
    pub aspect : f64,
    pub width : f64,
    pub height : f64,
    pub height_base : i32,
    pub yaw : f64,
    pub pitch : f64,
    pub roll : f64,

    pub clear_color : vec::Vec4,
    pub projection : Projection,

    origin : vec::Vec3,
    local_offset : vec::Vec3,
    center : vec::Vec3,

    //pub euler : vec::Vec3
}

impl Default for CameraData
{
    fn default() -> CameraData
    {
        CameraData {
            fovy : consts::PI/8.0f64,
            fovy_base : consts::PI/8.0f64,
            near : 1f64,
            far : 10000f64,
            aspect : 1.6f64,
            width : 800f64,
            height : 500f64,
            height_base : 500i32,
            yaw : 0f64,
            pitch : 0f64,
            roll : 0f64,

            origin : vec::Vec3::zero(),
            local_offset : vec::Vec3::zero(),
            center : vec::Vec3::zero(),

            clear_color : vec::Vec4::zero(),

            projection : Projection::Perspective,

            //euler : vec::Vec3::zero(),
        }
    }
}

pub struct Camera
{
    pub data : CameraData,
    pub object : Arc<RWLock<object::Object>>,
    pub id : uuid::Uuid
}

impl Camera
{
    /*
    pub fn new() -> Camera
    {
        let c = Camera {
            data : Default::default(),
            object : Arc::new(RWLock::new(object::Object::new("camera")))
        };

        c.object.write().position = vec::Vec3::new(0.1f64, 0f64, 0f64);

        c
    }
    */

    pub fn perspective_get(&self) -> matrix::Matrix4
    {
        //TODO
        //matrix::Matrix4::perspective(0.4f64,1f64,1f64,10000f64)
        match self.data.projection {
            Projection::Perspective =>
                matrix::Matrix4::perspective(
                    self.data.fovy,
                    self.data.aspect,
                    self.data.near,
                    self.data.far),
            Projection::Orthographic => 
                matrix::Matrix4::orthographic(
                    (self.data.width / 2f64) as u32,
                    (self.data.height / 2f64) as u32,
                    self.data.near,
                    self.data.far)
        }
    }

    pub fn ray_from_screen(&self, x : f64, y : f64, length: f64) -> geometry::Ray
    {
        let c = &self.data;
        let o = self.object.read().unwrap();

        let near = c.near;
        let camz = o.orientation.rotate_vec3(&Vec3::forward());
        let up = o.orientation.rotate_vec3(&Vec3::up());
        let h = (camz^up).normalized();
        let vl = (c.fovy/2f64).tan() * near;

        let width = c.width;
        let height = c.height;
        let aspect : f64 = width / height;
        let vh = vl * aspect;

        let up = up * vl;
        let h = h * vh;

        let x : f64 = x - (width /2.0f64);
        let y : f64 = y - (height /2.0f64);

        let x : f64 = x / (width /2.0f64);
        let y : f64 = y / (height /2.0f64);


        let pos = o.position + (camz * near) + ( (h * x) + (up * -y));
        let dir = pos - o.position;
        let dir = dir.normalized();
        let dir = dir * length;

        geometry::Ray {
            start : pos,
            direction : dir
        }
    }

    pub fn resolution_set(&mut self, w : i32, h : i32)
    {
        if w as f64 != self.data.width || h as f64 != self.data.height {
            self.data.width = w as f64;
            self.data.height = h as f64;
            self.update_projection();
            //cam.update_orthographic(c);
        }
    }

    pub fn update_projection(&mut self)
    {
        self.data.aspect = self.data.width/ self.data.height;
        self.data.fovy = self.data.fovy_base * self.data.height/ (self.data.height_base as f64);
        //mat4_set_perspective(c->projection, c->fovy, c->aspect , c->near, c->far);
    }

    pub fn rotate_around_center(&self, q : &vec::Quat)
    {
        let o = &self.object;
        let c = &self.data;

        let def = q.rotate_vec3_around(&c.origin, &c.center);
        let doff = q.rotate_vec3(&c.local_offset);
        o.write().unwrap().position = def + doff;
    }

    fn recalculate_origin(&mut self)
    {
        let o = &self.object.read().unwrap();
        let c = &mut self.data;

        let offset = o.orientation.rotate_vec3(&c.local_offset);
        let origin = o.position - offset;
        let qi = o.orientation.as_quat().inverse();
        c.origin = qi.rotate_vec3_around(&origin, &c.center);
    }

    pub fn set_center(&mut self, c : &Vec3)
    {
        self.data.center = *c;
        self.recalculate_origin();
    }

    pub fn pan(&mut self, t : &vec::Vec3)
    {
        let o = &mut self.object.write().unwrap();
        let c = &mut self.data;

        c.local_offset = c.local_offset + *t;
        let tt = o.orientation.rotate_vec3(t);
        o.position = o.position + tt;
    }

    pub fn lookat(&mut self, at : vec::Vec3)
    {
        {
            let mut o = &mut self.object.write().unwrap();
            let c = &mut self.data;

            let d = at - o.position;

            c.yaw = (-d.x).atan2(-d.z);
            let r = ( d.x*d.x + d.z*d.z ).sqrt();
            c.pitch = d.y.atan2(r);

            c.center = at;

            let ori = &mut o.orientation;

            match *ori {
                Orientation::AngleXYZ(ref mut a) => {
                    (*a).x = c.pitch/consts::PI*180.0;
                    (*a).y = c.yaw/consts::PI*180.0;
                },
                Orientation::Quat(ref mut q) => 
                    *q = vec::Quat::new_yaw_pitch_roll_rad(c.yaw, c.pitch, 0f64)
            }
        }

        self.recalculate_origin();
    }
}

