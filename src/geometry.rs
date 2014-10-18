use vec::{Vec3, Quat};
use std::fmt;

pub struct Ray
{
    pub start : Vec3,
    pub direction : Vec3
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

pub struct Triangle
{
    pub v0 : Vec3,
    pub v1 : Vec3,
    pub v2 : Vec3
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
    pub v : [Vec3,..8]
}

pub struct Segment
{
    pub p0 : Vec3,
    pub p1 : Vec3
}

impl fmt::Show for Ray
{
    fn fmt(&self, fmt :&mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}; {}", self.start, self.direction)
    }
}

