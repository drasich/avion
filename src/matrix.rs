
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
}

