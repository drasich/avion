
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
}
