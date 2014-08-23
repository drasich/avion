use vec;

pub struct Matrix4
{
    data : [f64, .. 16]
}

pub struct Matrix<T>
{
    data : [T, .. 16]
}

type Mat4 = Matrix<f64>;
impl Matrix<f64>
{
    pub fn identity() -> Mat4
    {
        Mat4 { data : [
            1f64,0f64,0f64,0f64,
            0f64,1f64,0f64,0f64,
            0f64,0f64,1f64,0f64,
            0f64,0f64,0f64,1f64
        ]
        }
    }
}

impl Matrix4
{
    pub fn identity_set(&mut self)
    {
        self.data[0] = 1f64;
        self.data[4] = 1f64;
        self.data[8] = 1f64;
        self.data[12] = 1f64;

        self.data[0] = 0f64;
        self.data[1] = 0f64;
        self.data[2] = 0f64;
        self.data[3] = 0f64;

        self.data[4] = 0f64;
        self.data[5] = 0f64;
        self.data[6] = 0f64;
        self.data[7] = 0f64;

        self.data[8] = 0f64;
        self.data[9] = 0f64;
        self.data[10] = 0f64;
        self.data[11] = 0f64;

        self.data[12] = 0f64;
        self.data[13] = 0f64;
        self.data[14] = 0f64;
        self.data[15] = 0f64;
    }

    pub fn identity() -> Matrix4
    {
        Matrix4 { data : [
            1f64,0f64,0f64,0f64,
            0f64,1f64,0f64,0f64,
            0f64,0f64,1f64,0f64,
            0f64,0f64,0f64,1f64
        ]
        }
    }

    pub fn to_f32(&self) -> [f32, ..16]
    {
        let mut m : [f32, ..16] = [0f32, ..16];
        for i in range(0u, 16)
        {
            m[i] = self.data[i] as f32;
        }

        m
    }

    pub fn frustum(
        left : f64,
        right : f64,
        bottom : f64,
        top : f64,
        near : f64,
        far : f64) -> Matrix4
    {
        let mut m : [f64, ..16] = [0f64, ..16];
        m[0] = 2f64 * near / (right - left);
        m[8] = (right + left) / (right - left);
        m[5] = 2f64 * near / (top - bottom);
        m[9] = (top + bottom) / (top - bottom);
        m[10] = -(far + near) / (far - near);
        m[14] = -(2f64 * far * near) / (far - near);
        m[11] = -1f64;
        m[15] = 0f64;

        Matrix4 { data : m }
    }

    pub fn perspective(
        fovy : f64,
        aspect : f64,
        near : f64,
        far : f64) -> Matrix4
    {
        let half_height = ((fovy/2.0)*near).tan();
        let half_width = half_height* aspect;

        Matrix4::frustum(-half_width, half_width, -half_height, half_height, near, far)
    }

    pub fn orthographic(
        hw : u32,
        hh : u32,
        near : f64,
        far : f64 ) ->Matrix4
    {
        let mut m : [f64, ..16] = [0f64, ..16];

        m[0] =  1f64/ hw as f64;
        m[5] =  1f64/ hh as f64;
        m[10] = -2f64 / (far - near);
        m[14] = - (far + near) / (far - near);
        m[15] = 1f64;

        Matrix4 { data : m }
    }

    pub fn translation(t : vec::Vec3) -> Matrix4
    {
        let mut m : [f64, ..16] = [0f64, ..16];

        m[0] = 1f64;
        m[5] = 1f64;
        m[10] = 1f64;
        m[15] = 1f64;

        m[12] = t.x;
        m[13] = t.y;
        m[14] = t.z;

        Matrix4 { data : m }
    }

    pub fn inverse_get(&self) -> Matrix4
    {
        let m = &self.data;

        let m00 = m[0];
        let m01 = m[4];
        let m02 = m[8];
        let m03 = m[12];
        let m10 = m[1];
        let m11 = m[5];
        let m12 = m[9];
        let m13 = m[13];
        let m20 = m[2];
        let m21 = m[6];
        let m22 = m[10];
        let m23 = m[14];
        let m30 = m[3];
        let m31 = m[7];
        let m32 = m[11];
        let m33 = m[15];

        let mut v0 = m20 * m31 - m21 * m30;
        let mut v1 = m20 * m32 - m22 * m30;
        let mut v2 = m20 * m33 - m23 * m30;
        let mut v3 = m21 * m32 - m22 * m31;
        let mut v4 = m21 * m33 - m23 * m31;
        let mut v5 = m22 * m33 - m23 * m32;

        let t00 = (v5 * m11 - v4 * m12 + v3 * m13);
        let t10 = - (v5 * m10 - v2 * m12 + v1 * m13);
        let t20 = (v4 * m10 - v2 * m11 + v0 * m13);
        let t30 = - (v3 * m10 - v1 * m11 + v0 * m12);

        let invDet = 1f64 / (t00 * m00 + t10 * m01 + t20 * m02 + t30 * m03);

        let d00 = t00 * invDet;
        let d10 = t10 * invDet;
        let d20 = t20 * invDet;
        let d30 = t30 * invDet;

        let d01 = - (v5 * m01 - v4 * m02 + v3 * m03) * invDet;
        let d11 = (v5 * m00 - v2 * m02 + v1 * m03) * invDet;
        let d21 = - (v4 * m00 - v2 * m01 + v0 * m03) * invDet;
        let d31 = (v3 * m00 - v1 * m01 + v0 * m02) * invDet;

        v0 = m10 * m31 - m11 * m30;
        v1 = m10 * m32 - m12 * m30;
        v2 = m10 * m33 - m13 * m30;
        v3 = m11 * m32 - m12 * m31;
        v4 = m11 * m33 - m13 * m31;
        v5 = m12 * m33 - m13 * m32;

        let d02 = (v5 * m01 - v4 * m02 + v3 * m03) * invDet;
        let d12 = - (v5 * m00 - v2 * m02 + v1 * m03) * invDet;
        let d22 = (v4 * m00 - v2 * m01 + v0 * m03) * invDet;
        let d32 = - (v3 * m00 - v1 * m01 + v0 * m02) * invDet;

        v0 = m21 * m10 - m20 * m11;
        v1 = m22 * m10 - m20 * m12;
        v2 = m23 * m10 - m20 * m13;
        v3 = m22 * m11 - m21 * m12;
        v4 = m23 * m11 - m21 * m13;
        v5 = m23 * m12 - m22 * m13;

        let d03 = - (v5 * m01 - v4 * m02 + v3 * m03) * invDet;
        let d13 = (v5 * m00 - v2 * m02 + v1 * m03) * invDet;
        let d23 = - (v4 * m00 - v2 * m01 + v0 * m03) * invDet;
        let d33 = (v3 * m00 - v1 * m01 + v0 * m02) * invDet;

        Matrix4 { data : [
            d00, d01, d02, d03,
            d10, d11, d12, d13,
            d20, d21, d22, d23,
            d30, d31, d32, d33]
        }
    }

}

impl Mul<Matrix4, Matrix4> for Matrix4 {
    fn mul(&self, other: &Matrix4) -> Matrix4 {
        let mut out : [f64, ..16] = [0f64, ..16];

        let m = self.data;
        let n = other.data;

        out[0]  = m[0] * n[0]  + m[4] * n[1]  + m[8] * n[2]  + m[12] * n[3];
        out[4]  = m[0] * n[4]  + m[4] * n[5]  + m[8] * n[6]  + m[12] * n[7];
        out[8]  = m[0] * n[8]  + m[4] * n[9]  + m[8] * n[10] + m[12] * n[11];
        out[12] = m[0] * n[12] + m[4] * n[13] + m[8] * n[14] + m[12] * n[15];

        out[1]  = m[1] * n[0]  + m[5] * n[1]  + m[9] * n[2]  + m[13] * n[3];
        out[5]  = m[1] * n[4]  + m[5] * n[5]  + m[9] * n[6]  + m[13] * n[7];
        out[9]  = m[1] * n[8]  + m[5] * n[9]  + m[9] * n[10] + m[13] * n[11];
        out[13] = m[1] * n[12] + m[5] * n[13] + m[9] * n[14] + m[13] * n[15];

        out[2]  = m[2] * n[0]  + m[6] * n[1]  + m[10] * n[2]  + m[14] * n[3];
        out[6]  = m[2] * n[4]  + m[6] * n[5]  + m[10] * n[6]  + m[14] * n[7];
        out[10] = m[2] * n[8]  + m[6] * n[9]  + m[10] * n[10] + m[14] * n[11];
        out[14] = m[2] * n[12] + m[6] * n[13] + m[10] * n[14] + m[14] * n[15];

        out[3]  = m[3] * n[0]  + m[7] * n[1]  + m[11] * n[2]  + m[15] * n[3];
        out[7]  = m[3] * n[4]  + m[7] * n[5]  + m[11] * n[6]  + m[15] * n[7];
        out[11] = m[3] * n[8]  + m[7] * n[9]  + m[11] * n[10] + m[15] * n[11];
        out[15] = m[3] * n[12] + m[7] * n[13] + m[11] * n[14] + m[15] * n[15];

        Matrix4 { data : out }
    }
}

