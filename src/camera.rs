use std::sync::{RwLock, Arc};
use std::f64::consts;
use std::default::Default;
use rustc_serialize::{json, Encodable, Encoder, Decoder, Decodable};

use vec;
use vec::{Vec3};
use object;
use matrix;
use geometry;
use transform::Orientation;
use uuid;

#[derive(RustcDecodable,RustcEncodable)]
pub enum Projection
{
    Perspective,
    Orthographic
}

#[derive(RustcDecodable,RustcEncodable)]
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

//#[derive(RustcDecodable,RustcEncodable)]
pub enum ObjectKind
{
    Own(Arc<RwLock<object::Object>>),
    Ref(object::ObjectRef),
}

impl ObjectKind {

    fn get_orientation(&self) -> Orientation
    {
        match *self {
            ObjectKind::Own(ref o) => o.read().unwrap().orientation,
            ObjectKind::Ref(ref r) => match r.object {
                None => Orientation::new_quat(),
                Some(ref o) => o.read().unwrap().orientation
            }
        }
    }

    fn set_orientation(&mut self, ori : Orientation)
    {
        match *self {
            ObjectKind::Own(ref o) => o.write().unwrap().orientation = ori,
            ObjectKind::Ref(ref r) => if let Some(ref o) = r.object {
                o.write().unwrap().orientation = ori;
            }
        }
    }

    fn get_position(&self) -> vec::Vec3
    {
        match *self {
            ObjectKind::Own(ref o) => o.read().unwrap().position,
            ObjectKind::Ref(ref r) => match r.object {
                None => vec::Vec3::zero(),
                Some(ref o) => o.read().unwrap().position
            }
        }
    }

    fn set_position(&mut self, pos : vec::Vec3)
    {
        match *self {
            ObjectKind::Own(ref o) => o.write().unwrap().position = pos,
            ObjectKind::Ref(ref r) => if let Some(ref o) = r.object {
                o.write().unwrap().position = pos;
            }
        }
    }

    fn get_object(&self) -> Option<Arc<RwLock<object::Object>>>
    {
        match *self {
            ObjectKind::Own(ref o) => Some(o.clone()),
            ObjectKind::Ref(ref r) => match r.object {
                None => None,
                Some(ref o) => Some(o.clone())
            }
        }

    }
}

#[derive(RustcDecodable,RustcEncodable)]
pub struct Camera
{
    pub data : CameraData,
    //pub position : vec::Vec3,
    //pub orientation : Orientation,
    pub object : Arc<RwLock<object::Object>>,
    //pub object : ObjectKind, 
    pub id : uuid::Uuid,
    pub object_id : Option<uuid::Uuid>
}

pub struct CameraRom
{
    pub data : CameraData,
    pub id : uuid::Uuid,
    pub object_id : Option<uuid::Uuid>
}

impl Camera
{
    /*
    pub fn new() -> Camera
    {
        let c = Camera {
            data : Default::default(),
            object : Arc::new(RwLock::new(object::Object::new("camera")))
        };

        c.object.write().position = vec::Vec3::new(0.1f64, 0f64, 0f64);

        c
    }
    */

    pub fn get_perspective(&self) -> matrix::Matrix4
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

    pub fn set_resolution(&mut self, w : i32, h : i32)
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

    pub fn get_frustum_planes_rect(
        /*
           ViewCamera* cam,
           Plane* p,
           float left, float top, float width, float height)
           */
        &self,
        left : f64, top : f64, width : f64, height : f64) -> [geometry::Plane;6]
    {

        let o = self.object.read().unwrap();
        let c = &self.data;

        let direction = o.orientation.rotate_vec3(&vec::Vec3::new(0f64,0f64,-1f64));
        let right = o.orientation.rotate_vec3(&vec::Vec3::new(1f64,0f64,0f64));
        let up = o.orientation.rotate_vec3(&vec::Vec3::new(0f64,1f64,0f64));

        //plane order:
        //near, far, up, down, right, left
        let mut p = [geometry::Plane::xz();6];

        //near plane
        p[0].point = o.position + (direction * c.near);
        p[0].normal = direction;

        //far plane
        p[1].point = o.position + (direction * c.far);
        p[1].normal = direction * -1f64;

        //up plane
        let hh = (c.fovy/2f64).tan()* c.near;
        let top = top * hh / (c.height/2.0f64);
        let height = height * hh / (c.height/2.0f64);

        let th = hh - top;
        let upd = (direction * c.near) + (up * th);

        p[2].point = o.position;
        let nn = (right ^ upd).normalized();
        p[2].normal = nn * -1f64;

        //down plane
        let bh = hh - (top + height);
        p[3].point = o.position;
        let downd = (direction * c.near) + (up * bh);
        let nn = (right ^ downd).normalized();
        //p[3].normal = vec3_mul(nn, -1);
        p[3].normal = nn;


        //right plane
        let hw = hh * c.aspect;
        let left = left * hw / (c.width/2.0f64);
        let width = width * hw / (c.width/2.0f64);
        
        let rw = -hw + (left + width);
        p[4].point = o.position;
        let rightd = (direction * c.near) + (right* rw);
        let nn = (up ^ rightd).normalized();
        //p[4].normal = vec3_mul(nn, -1);
        p[4].normal = nn;

        //left plane
        let lw = -hw + left;
        p[5].point = o.position;
        let leftd = (direction* c.near) + (right* lw);
        let nn = (up ^ leftd).normalized();
        p[5].normal = nn * -1f64;

        /*
           printf(" leftd : %f, %f, %f \n", leftd.x, leftd.y, leftd.z);
           printf(" up : %f, %f, %f \n", up.x, up.y, up.z);
           printf(" up plane normal : %f, %f, %f \n", nn.x, nn.y, nn.z);
           */
        return p;
    }

    pub fn world_to_screen(&self, p : vec::Vec3) -> vec::Vec2
    {
        let world = self.object.read().unwrap().get_world_matrix();
        let cam_inv = world.get_inverse();
        let projection = self.get_perspective();

        let tm = &projection * &cam_inv;

        let p4 = vec::Vec4::new(p.x, p.y, p.z, 1f64);
        let sp = &tm * p4;

        let n  = vec::Vec3::new(sp.x/sp.w, sp.y/sp.w, sp.z/sp.w);

        let screen  = vec::Vec2::new(
            (n.x+1.0f64)* self.data.width/2.0f64,
            -(n.y-1.0f64)* self.data.height/2.0f64);

        //printf("screen : %f, %f \n", screen.x, screen.y);

        return screen;
    }

    pub fn get_orientation(&self) -> Orientation
    {
        self.object.read().unwrap().orientation
    }

    pub fn set_orientation(&mut self, ori : Orientation)
    {
        self.object.write().unwrap().orientation = ori;
    }

    pub fn get_position(&self) -> vec::Vec3
    {
        self.object.read().unwrap().position
    }

    pub fn set_position(&mut self, pos : vec::Vec3)
    {
        self.object.write().unwrap().position = pos;
    }

    pub fn get_object(&self) -> Option<Arc<RwLock<object::Object>>>
    {
        Some(self.object.clone())
    }

}

