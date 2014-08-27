use std::f64::consts;
use std::fmt;

pub struct Vec3
{
    pub x : f64,
    pub y : f64,
    pub z : f64
}

pub struct Vec4
{
    pub x : f64,
    pub y : f64,
    pub z : f64,
    pub w : f64
}

pub struct Quat
{
    pub x : f64,
    pub y : f64,
    pub z : f64,
    pub w : f64
}


impl Vec4
{
    pub fn zero() -> Vec4
    {
        Vec4 {
            x : 0f64,
            y : 0f64,
            z : 0f64,
            w : 0f64
        }
    }

    pub fn new(x : f64, y : f64, z : f64, w : f64) -> Vec4
    {
        Vec4 {
            x : x,
            y : y,
            z : z,
            w : w 
        }
    }
}


impl Vec3
{
    pub fn zero() -> Vec3
    {
        Vec3 {
            x : 0f64,
            y : 0f64,
            z : 0f64
        }
    }

    pub fn new(x : f64, y : f64, z : f64) -> Vec3
    {
        Vec3 {
            x : x,
            y : y,
            z : z
        }
    }

    pub fn length(&self) -> f64
    {
        self.length2().sqrt()
    }

    pub fn length2(&self) -> f64
    {
        self.x*self.x + self.y*self.y + self.z*self.z
    }
}

impl Quat
{
    pub fn identity() -> Quat
    {
        Quat {
            x : 0f64,
            y : 0f64,
            z : 0f64,
            w : 1f64
        }
    }

    pub fn length2_get(&self) -> f64
    {
        self.x*self.x + self.y*self.y + self.z*self.z + self.w*self.w
    }

    pub fn new_axis_angle(axis : Vec3, angle_radian : f64) -> Quat
    {
        let length = axis.length();

        /*
           let epsilon = 0.0000001f64;
        if length < consts::E {
            return Quat::identity();
        }
        */

        let inverse_norm = 1f64/length;
        let cos_half_angle = (0.5f64*angle_radian).cos();
        let sin_half_angle = (0.5f64*angle_radian).sin();

        Quat { 
            x : axis.x * sin_half_angle * inverse_norm,
            y : axis.y * sin_half_angle * inverse_norm,
            z : axis.z * sin_half_angle * inverse_norm,
            w : cos_half_angle
        }
    }
}

impl fmt::Show for Quat
{
    fn fmt(&self, fmt :&mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}, {}, {}, {}", self.x, self.y, self.z, self.w)
    }
}
