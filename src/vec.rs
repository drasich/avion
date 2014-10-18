use std::fmt;

#[deriving(Decodable, Encodable)]
pub struct Vec3
{
    pub x : f64,
    pub y : f64,
    pub z : f64
}

#[deriving(Decodable, Encodable)]
pub struct Vec4
{
    pub x : f64,
    pub y : f64,
    pub z : f64,
    pub w : f64
}

#[deriving(Decodable, Encodable, Clone)]
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

    pub fn one() -> Vec3
    {
        Vec3 {
            x : 1f64,
            y : 1f64,
            z : 1f64
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

    pub fn forward() -> Vec3
    {
        Vec3 {
            x : 0f64,
            y : 0f64,
            z : -1f64
        }
    }

    pub fn up() -> Vec3
    {
        Vec3 {
            x : 0f64,
            y : 1f64,
            z : 0f64
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

    pub fn normalized(&self) -> Vec3
    {
        self * (1f64/ self.length())
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

    pub fn rotate_vec3(&self, v : &Vec3) -> Vec3
    {
        let qvec = Vec3::new(self.x, self.y, self.z);
        let uv = qvec^*v;
        let uuv = qvec^uv;
        let uv = uv * (2f64* self.w);
        let uuv = uuv * 2f64;

        v + uv + uuv
    }
}

impl fmt::Show for Vec3
{
    fn fmt(&self, fmt :&mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "({}, {}, {})", self.x, self.y, self.z)
    }
}


impl fmt::Show for Quat
{
    fn fmt(&self, fmt :&mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}, {}, {}, {}", self.x, self.y, self.z, self.w)
    }
}

impl BitXor<Vec3, Vec3> for Vec3 {
    fn bitxor(&self, other: &Vec3) -> Vec3 {
        Vec3::new(
            self.y * other.z - self.z*other.y,
            self.z * other.x - self.x*other.z,
            self.x * other.y - self.y*other.x,
            )
    }
}

impl Add<Vec3, Vec3> for Vec3 {
    fn add(&self, other: &Vec3) -> Vec3 {
        Vec3::new(
            self.x + other.x,
            self.y + other.y,
            self.z + other.z)
    }
}

impl Sub<Vec3, Vec3> for Vec3 {
    fn sub(&self, other: &Vec3) -> Vec3 {
        Vec3::new(
            self.x - other.x,
            self.y - other.y,
            self.z - other.z)
    }
}



impl Mul<f64, Vec3> for Vec3 {
    fn mul(&self, f: &f64) -> Vec3 {
        Vec3::new(self.x**f, self.y**f, self.z**f)
    }
}

#[test]
fn test_quat_rotate() {
    //TODO
    assert_eq!(1i, 1i);
}
