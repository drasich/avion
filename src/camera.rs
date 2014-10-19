use std::rc::Rc;
use sync::{RWLock, Arc};
use std::f64::consts;

use vec;
use vec::{Vec3, Quat};
use object;
use matrix;
use geometry;

pub struct CameraData
{
    fovy : f64,
    pub fovy_base : f64,
    pub near : f64,
    pub far : f64,
    pub aspect : f64,
    pub width : f64,
    height : f64,
    pub height_base : i32,
    pub yaw : f64,
    pub pitch : f64,
    pub roll : f64,

    origin : vec::Vec3,
    local_offset : vec::Vec3,
    center : vec::Vec3,

    clear_color : vec::Vec4,
}

pub struct Camera
{
    pub data : CameraData,
    pub object : Arc<RWLock<object::Object>>
}

impl Camera
{
    pub fn new() -> Camera
    {
        Camera { data : CameraData {
            fovy : consts::PI/8.0f64,
            fovy_base : consts::PI/8.0f64,
            near : 1f64,
            far : 1000f64,
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

            clear_color : vec::Vec4::zero()  
        },
        object : Arc::new(RWLock::new(object::Object::new("camera")))
        }
    }

    pub fn perspective_get(&self) -> matrix::Matrix4
    {
        //TODO
        matrix::Matrix4::perspective(0.4f64,1f64,1f64,10000f64)
        //matrix::Matrix4::perspective(fovy,1f64, near, far)
    }

    pub fn ray_from_screen(&self, x : f64, y : f64, length: f64) -> geometry::Ray
    {
        let c = self.data;
        let o = self.object.read();

        let near = c.near;
        let camz = o.orientation.rotate_vec3(&Vec3::forward());
        let up = o.orientation.rotate_vec3(&Vec3::up());
        let h = (camz^up).normalized();
        let vl = (c.fovy/2f64).tan() * near;

        let width = c.width;
        let height = c.height;
        let aspect : f64 = width / height;
        let vh = vl * aspect;

        println!("w,h {},{}, {}", width, height, aspect);

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
}

