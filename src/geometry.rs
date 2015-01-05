use vec::{Vec3, Quat};
use std::fmt;

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

pub struct Plane
{
    pub point : Vec3,
    pub normal : Vec3
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

pub struct AABox
{
    pub min : Vec3,
    pub max : Vec3
}

//TODO dragon
/*
impl AABox
{
    pub fn to_obox(&self, v : Vec3, q : Quat, scale : Vec3) -> OBox
    {
        OBox {
        }
    }
}
*/

pub struct OBox
{
    pub v : [Vec3; 8]
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
impl fmt::Show for Segment
{
    fn fmt(&self, fmt :&mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}; {}", self.p0, self.p1)
    }
}


impl fmt::Show for Ray
{
    fn fmt(&self, fmt :&mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}; {}", self.start, self.direction)
    }
}

impl fmt::Show for Triangle
{
    fn fmt(&self, fmt :&mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{} ; {} ; {}", self.v0, self.v1, self.v2)
    }
}

impl fmt::Show for Repere
{
    fn fmt(&self, fmt :&mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{} ; {}", self.origin, self.rotation)
    }
}

