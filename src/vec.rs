use std::fmt;
use std::f64::consts;
use std::ops::{Mul, BitXor, Add, Sub, Div};
use std::f64::EPSILON;

#[derive(RustcDecodable, RustcEncodable, Clone, Copy)]
pub struct Vec3
{
    pub x : f64,
    pub y : f64,
    pub z : f64
}

#[derive(RustcDecodable, RustcEncodable, Clone, Copy)]
pub struct Vec2
{
    pub x : f64,
    pub y : f64,
}

#[derive(RustcDecodable, RustcEncodable, Clone, Copy)]
pub struct Vec4
{
    pub x : f64,
    pub y : f64,
    pub z : f64,
    pub w : f64
}

#[derive(RustcDecodable, RustcEncodable, Clone, Copy)]
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

    pub fn x() -> Vec4
    {
        Vec4 {
            x : 1f64,
            y : 0f64,
            z : 0f64,
            w : 0f64
        }
    }

    pub fn y() -> Vec4
    {
        Vec4 {
            x : 0f64,
            y : 1f64,
            z : 0f64,
            w : 0f64
        }
    }

    pub fn z() -> Vec4
    {
        Vec4 {
            x : 0f64,
            y : 0f64,
            z : 1f64,
            w : 0f64
        }
    }

    pub fn w() -> Vec4
    {
        Vec4 {
            x : 0f64,
            y : 0f64,
            z : 0f64,
            w : 1f64
        }
    }
}

impl Vec2
{
    pub fn zero() -> Vec2
    {
        Vec2 {
            x : 0f64,
            y : 0f64,
        }
    }

    pub fn new(x : f64, y : f64) -> Vec2
    {
        Vec2 {
            x : x,
            y : y,
        }
    }

    pub fn dot(self, other : Vec2) -> f64
    {
        self.x * other.x + self.y * other.y
    }

    pub fn length2(self) -> f64
    {
        self.x*self.x + self.y*self.y
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

    pub fn right() -> Vec3
    {
        Vec3 {
            x : 1f64,
            y : 0f64,
            z : 0f64
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

    pub fn x() -> Vec3
    {
        Vec3 {
            x : 1f64,
            y : 0f64,
            z : 0f64
        }
    }

    pub fn y() -> Vec3
    {
        Vec3 {
            x : 0f64,
            y : 1f64,
            z : 0f64
        }
    }

    pub fn z() -> Vec3
    {
        Vec3 {
            x : 0f64,
            y : 0f64,
            z : 1f64
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

    /*
    pub fn mul(self, v: Vec3) -> Vec3 {
        Vec3::new(self.x*v.x, self.y*v.y, self.z*v.z)
    }
    */

    pub fn normalized(&self) -> Vec3
    {
        *self * (1f64/ self.length())
    }

    pub fn normalize(&mut self)
    {
        *self = *self * (1f64/ self.length())
    }

    pub fn dot(&self, other : &Vec3) -> f64
    {
        self.x * other.x + self.y * other.y + self.z * other.z
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

    pub fn new(x : f64, y : f64, z : f64, w : f64) -> Quat
    {
        Quat {
            x : x, 
            y : y,
            z : z,
            w : w
        }
    }

    pub fn length2(&self) -> f64
    {
        self.x*self.x + self.y*self.y + self.z*self.z + self.w*self.w
    }

    pub fn length(&self) -> f64
    {
        self.length2().sqrt()
    }

    pub fn new_axis_angle_rad(axis : Vec3, angle_radian : f64) -> Quat
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

    pub fn new_axis_angle_deg(axis : Vec3, angle_deg : f64) -> Quat
    {
        let r = consts::PI / 180f64;

        Quat::new_axis_angle_rad(axis, angle_deg * r)
    }


    pub fn new_yaw_pitch_roll_rad(yaw : f64, pitch : f64, roll : f64) -> Quat
    {
        let qy = Quat::new_axis_angle_rad(Vec3::new(0f64,1f64,0f64), yaw);
        let qp = Quat::new_axis_angle_rad(Vec3::new(1f64,0f64,0f64), pitch);
        let qr = Quat::new_axis_angle_rad(Vec3::new(0f64,0f64,1f64), roll);

        let q1 = qy * qp;

        q1 * qr
    }

    pub fn new_yaw_pitch_roll_deg(yaw : f64, pitch : f64, roll : f64) -> Quat
    {
        let r = consts::PI / 180f64;

        Quat::new_yaw_pitch_roll_rad(
            yaw*r,
            pitch*r,
            roll*r)
    }

    pub fn new_angles_rad(angles : &Vec3) -> Quat
    {
        let qx = Quat::new_axis_angle_rad(Vec3::new(1f64,0f64,0f64), angles.x);
        let qy = Quat::new_axis_angle_rad(Vec3::new(0f64,1f64,0f64), angles.y);
        let qz = Quat::new_axis_angle_rad(Vec3::new(0f64,0f64,1f64), angles.z);

        let q1 =  qz * qy;
        q1 * qx
    }

    pub fn new_angles_deg(angles : &Vec3) -> Quat
    {
        let r = consts::PI / 180f64;

        Quat::new_angles_rad(&(*angles * r))
    }

    pub fn rotate_vec3(&self, v : &Vec3) -> Vec3
    {
        let qvec = Vec3::new(self.x, self.y, self.z);
        let uv = qvec^*v;
        let uuv = qvec^uv;
        let uv = uv * (2f64* self.w);
        let uuv = uuv * 2f64;

        *v + uv + uuv
    }

    pub fn rotate_vec3_around(&self, v : &Vec3, pivot : &Vec3) -> Vec3
    {
        let mut p = *v - *pivot;
        p = self.rotate_vec3(&p);
        p = p + *pivot;
        p
    }

    
    pub fn conj(&self) -> Quat
    {
        Quat {
            x : -self.x,
            y : -self.y,
            z : -self.z,
            w : self.w,
        }
        //let q1 =  qx * qy;
        //q1 * qz
    }

    pub fn inverse(&self) -> Quat
    {
        let l = self.length2();
        Quat {
            x : -self.x/l,
            y : -self.y/l,
            z : -self.z/l,
            w : self.w/l,
        }
    }

    pub fn to_euler_rad(&self) -> Vec3
    {
        let q = self.normalized();
        /*
        Vec3 {
            x : (2f64*(self.w*self.x + self.y*self.z)).atan2(1f64 - 2f64*(self.x*self.x + self.y*self.y)),
            y : (2f64*(self.w*self.y - self.z*self.x)).asin(),
            z : (2f64*(self.w*self.z + self.x*self.y)).atan2(1f64- 2f64*(self.y*self.y + self.z*self.z))
        }
        */
        Vec3 {
            x : (2f64*(q.w*q.x + q.y*q.z)).atan2(1f64 - 2f64*(q.x*q.x + q.y*q.y)),
            y : (2f64*(q.w*q.y - q.z*q.x)).asin(),
            z : (2f64*(q.w*q.z + q.x*q.y)).atan2(1f64- 2f64*(q.y*q.y + q.z*q.z))
        }
    }

    pub fn to_euler_deg(&self) -> Vec3
    {
        let mut v = self.to_euler_rad();
        v = v * (180f64 / consts::PI);
        v
    }

    pub fn normalized(&self) -> Quat
    {
        *self * (1f64/ self.length())
    }

    pub fn dot(&self, other : &Quat) -> f64
    {
        self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w
    }

    pub fn between(q1 : &Quat, q2 : &Quat) -> Quat
    {
        q1.conj() * *q2
    }

}

impl fmt::Debug for Vec3
{
    fn fmt(&self, fmt :&mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "({}, {}, {})", self.x, self.y, self.z)
    }
}

impl fmt::Debug for Vec4
{
    fn fmt(&self, fmt :&mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "({}, {}, {}, {})", self.x, self.y, self.z, self.w)
    }
}


impl fmt::Debug for Quat
{
    fn fmt(&self, fmt :&mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "({}, {}, {}, {})", self.x, self.y, self.z, self.w)
    }
}

//impl BitXor<Vec3, Vec3> for Vec3 {
impl BitXor for Vec3 {
    type Output = Vec3;

    fn bitxor(self, other: Vec3) -> Vec3 {
        Vec3::new(
            self.y * other.z - self.z*other.y,
            self.z * other.x - self.x*other.z,
            self.x * other.y - self.y*other.x,
            )
    }
}

//impl Add<Vec3, Vec3> for Vec3 {
impl Add for Vec3 {
    type Output = Vec3;

    fn add(self, other: Vec3) -> Vec3 {
        Vec3::new(
            self.x + other.x,
            self.y + other.y,
            self.z + other.z)
    }
}

//impl Sub<Vec3, Vec3> for Vec3 {
impl Sub for Vec3 {
    type Output = Vec3;
    fn sub(self, other: Vec3) -> Vec3 {
        Vec3::new(
            self.x - other.x,
            self.y - other.y,
            self.z - other.z)
    }
}

//impl Mul<f64, Vec3> for Vec3 {
impl Mul<f64> for Vec3 {
    type Output = Vec3;

    fn mul(self, f: f64) -> Vec3 {
        Vec3::new(self.x*f, self.y*f, self.z*f)
    }
}

impl Mul<f32> for Vec3 {
    type Output = Vec3;

    fn mul(self, f: f32) -> Vec3 {
        let f = f as f64;
        Vec3::new(self.x*f, self.y*f, self.z*f)
    }
}


//impl Mul<Vec3, f64> for Vec3 {
/*
impl Mul<Vec3> for Vec3 {
    type Output = f64;

    fn mul(self, v: Vec3) -> f64 {
        self.x*v.x + self.y*v.y + self.z*v.z
    }
}
*/

//impl Mul<Vec3, Vec3> for Vec3 {
impl Mul<Vec3> for Vec3 {
    type Output = Vec3;

    fn mul(self, v: Vec3) -> Vec3 {
        Vec3::new(self.x*v.x, self.y*v.y, self.z*v.z)
    }
}


//impl Div<f64, Vec3> for Vec3 {
impl Div<f64> for Vec3 {
    type Output = Vec3;

    fn div(self, f: f64) -> Vec3 {
        Vec3::new(self.x/f, self.y/f, self.z/f)
    }
}


//impl Mul<Quat, Quat> for Quat {
impl Mul<Quat> for Quat {
    type Output = Quat;
    fn mul(self, other: Quat) -> Quat {
        Quat {
            x : self.w * other.x + self.x * other.w + self.y * other.z - self.z * other.y,
            y : self.w * other.y + self.y * other.w + self.z * other.x - self.x * other.z,
            z : self.w * other.z + self.z * other.w + self.x * other.y - self.y * other.x,
            w : self.w * other.w - self.x * other.x - self.y * other.y - self.z * other.z,
        }
    }
}

//impl Mul<f64, Quat> for Quat {
impl Mul<f64> for Quat {
    type Output = Quat;
    fn mul(self, f: f64) -> Quat {
        Quat {
            x : self.x * f,
            y : self.y * f,
            z : self.z * f,
            w : self.w * f
        }
    }
}

impl Add<Quat> for Quat {
    type Output = Quat;
    fn add(self, q: Quat) -> Quat {
        Quat {
            x : self.x + q.x,
            y : self.y + q.y,
            z : self.z + q.z,
            w : self.w + q.w
        }
    }
}


impl PartialEq for Vec3 {

    fn eq(&self, other: &Vec3) -> bool
    {
        self.x == other.x && self.y == other.y && self.z == other.z
    }
}

impl Sub for Vec2 {
    type Output = Vec2;
    fn sub(self, other: Vec2) -> Vec2 {
        Vec2::new(
            self.x - other.x,
            self.y - other.y)
    }
}


#[test]
fn test_quat_rotate() {
    //TODO
    assert_eq!(1isize, 1isize);
}


pub fn quat_slerp(from : Quat, to : Quat, t : f64) -> Quat
{
  //double omega, cosomega, sinomega, scale_from, scale_to ;

  let mut quat_to = to.clone();
  //printf("  quatto : %f, %f, %f, %f\n", quat_to.x, quat_to.y, quat_to.z, quat_to.w);
  let mut cosomega = from.dot(&to);

  if cosomega < 0f64 { 
    cosomega = -cosomega; 
    quat_to = to * -1f64; //quat_to = -to;
  }

  let (scale_from, scale_to) = {
      if (1f64 - cosomega) > EPSILON {
          let omega = cosomega.acos() ;  // 0 <= omega <= Pi (see man acos)
          let sinomega = omega.sin() ;  // this sinomega should always be +ve so
          // could try sinomega=sqrt(1-cosomega*cosomega) to avoid a sin()?
          let scale_from = ((1.0-t)*omega).sin()/sinomega;
          let scale_to = (t*omega).sin()/sinomega ;
          (scale_from,scale_to)
      } else {
          // --------------------------------------------------
          //   The ends of the vectors are very close
          //   we can use simple linear interpolation - no need
          //   to worry about the "spherical" interpolation
          //   --------------------------------------------------
          let scale_from = 1.0 - t ;
          let scale_to = t ;
          (scale_from,scale_to)
      }
  };

  //Quat q = quat_add(quat_mul_scalar(from, scale_from),quat_mul_scalar(quat_to,scale_to));
  let q = (from * scale_from) + (quat_to * scale_to);

  q
}



