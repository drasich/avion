use vec::{Vec3, Quat};
use std::fmt;
use std::ops::{Mul};//, BitXor, Add, Sub, Div};

pub struct Ray
{
    pub start : Vec3,
    pub direction : Vec3
}

impl Ray
{
    pub fn new(start : Vec3, direction : Vec3) -> Ray
    {
        Ray {
            start : start,
            direction : direction
        }
    }
}

#[derive(Copy, Clone)]
pub struct Plane
{
    pub point : Vec3,
    pub normal : Vec3
}

impl Plane
{
    pub fn new(point : Vec3, normal : Vec3) -> Plane
    {
        Plane {
            point : point,
            normal : normal
        }
    }

    pub fn xz() -> Plane
    {
        Plane {
            point : Vec3::zero(),
            normal : Vec3::y()
        }
    }
}


pub struct Sphere
{
    pub center : Vec3,
    pub radius : f32
}

pub struct Repere
{
    pub origin : Vec3,
    pub rotation : Quat
}

impl Repere {
    pub fn new(origin : Vec3, rotation : Quat) -> Repere
    {
        Repere {
            origin : origin,
            rotation : rotation
        }
    }

    pub fn world_to_local(&self, v : &Vec3) -> Vec3
    {
        let iq = self.rotation.conj();
        let out = iq.rotate_vec3(&(*v - self.origin));
        out
    }

    pub fn local_to_world(&self, v : &Vec3) -> Vec3
    {
        let iq = self.rotation.conj();
        let out = self.rotation.rotate_vec3(v) + self.origin;
        out
    }
}

pub struct Triangle
{
    pub v0 : Vec3,
    pub v1 : Vec3,
    pub v2 : Vec3
}

impl Triangle
{
    pub fn new(v0 : Vec3, v1 : Vec3, v2 : Vec3) -> Triangle
    {
        Triangle {
            v0 : v0,
            v1 : v1,
            v2 : v2
        }
    }
}

pub struct Frustum
{
    near : f32,
    far : f32,
    start : Vec3,
    direction : Vec3,
    up : Vec3,
    fovy : f32,
    aspect : f32
}

//TODO watch dragon
/*
impl Frustum
{
}
*/

#[derive(Clone)]
pub struct AABox
{
    pub min : Vec3,
    pub max : Vec3
}

impl AABox
{
    pub fn new(min : Vec3, max : Vec3) -> AABox
    {
        AABox {
            min : min,
            max : max
        }
    }

    pub fn to_obox(&self, v : Vec3, q : Quat, scale : Vec3) -> OBox
    {
        //TODO check
        println!("check this function before using");
        let x = q.rotate_vec3(&Vec3::x());
        let y = q.rotate_vec3(&Vec3::y());
        let z = q.rotate_vec3(&Vec3::z());

        let a = AABox::new(
            self.min.mul(scale),
            self.max.mul(scale));

        let mut o : [Vec3; 8] = [Vec3::zero(); 8];

        o[0] = (x * a.min.x) +
            (y * a.min.y) +
            (z * a.min.z);
        o[1] = o[0] + (x * (a.max.x - a.min.x));
        o[2] = o[0] + (y * (a.max.y - a.min.y));
        o[3] = o[0] + (z * (a.max.z - a.min.z));

        o[4] = (x * a.max.x) +
            (y * a.max.y) +
            (z * a.max.z);

        o[5] = o[4] + (x * (a.min.x - a.max.x));
        o[6] = o[4] + (y * (a.min.y - a.max.y));
        o[7] = o[4] + (z * (a.min.z - a.max.z));

        for oi in &mut o {
            *oi = *oi + v;
        }

        OBox::new(o)
    }
}

impl<'a> Mul<f64> for &'a AABox {
    type Output = AABox;

    fn mul(self, f: f64) -> AABox {
        AABox::new(self.min * f, self.max * f)
    }
}

impl<'a> Mul<f32> for &'a AABox {
    type Output = AABox;

    fn mul(self, f: f32) -> AABox {
        AABox::new(self.min * f, self.max * f)
    }
}

impl fmt::Debug for AABox
{
    fn fmt(&self, fmt :&mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "(min : {:?}, max : {:?})", self.min, self.max)
    }
}

pub struct OBox
{
    pub v : [Vec3; 8]
}

impl OBox {

    pub fn new(v : [Vec3; 8]) -> OBox
    {
        OBox { v : v }
    }
}

pub struct Segment
{
    pub p0 : Vec3,
    pub p1 : Vec3
}

impl Segment
{
    pub fn new(p0 : Vec3, p1 : Vec3) -> Segment
    {
        Segment {
            p0 : p0,
            p1 : p1,
        }
    }

}

impl fmt::Debug for Segment
{
    fn fmt(&self, fmt :&mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{:?}; {:?}", self.p0, self.p1)
    }
}


impl fmt::Debug for Ray
{
    fn fmt(&self, fmt :&mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{:?}; {:?}", self.start, self.direction)
    }
}

impl fmt::Debug for Triangle
{
    fn fmt(&self, fmt :&mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{:?} ; {:?} ; {:?}", self.v0, self.v1, self.v2)
    }
}

impl fmt::Debug for Repere
{
    fn fmt(&self, fmt :&mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{:?} ; {:?}", self.origin, self.rotation)
    }
}

