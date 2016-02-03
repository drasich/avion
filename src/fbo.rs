use libc::{c_uint, c_int};

#[repr(C)]
pub struct CglFbo;

unsafe impl Send for CglFbo {}
unsafe impl Sync for CglFbo {}

#[link(name = "cypher")]
extern {
    pub fn cgl_create_fbo() -> *const CglFbo;
    pub fn cgl_fbo_use(fbo : *const CglFbo);
    pub fn cgl_fbo_use_end();
    pub fn cgl_fbo_resize(fbo : *const CglFbo, w : c_int, h : c_int);
    pub fn cgl_fbo_destroy(fbo : *const CglFbo);
}

#[derive(RustcDecodable, RustcEncodable, Copy, Clone)]
pub enum Attachment
{
    Depth,
    Color
}

pub struct Fbo
{
    pub name : String,
    pub state : i32,
    pub cgl_fbo: Option<*const CglFbo>,
} 

unsafe impl Send for Fbo {}
unsafe impl Sync for Fbo {}

impl Fbo
{
    pub fn new(name : &str) -> Fbo
    {
        Fbo {
            name : String::from(name),
            state : 0,
            cgl_fbo : None,
        }
    }

    pub fn cgl_create(&mut self)
    {
        if self.state != 0 {
            return
        }

        self.cgl_fbo = unsafe {Some(cgl_create_fbo())};

        self.state = 1;
    }

    pub fn cgl_resize(&self, w : c_int, h : c_int)
    {
        if let Some(f) = self.cgl_fbo {
            unsafe { cgl_fbo_resize(f, w, h); }
        }
    }

    pub fn cgl_use(&self)
    {
        if let Some(f) = self.cgl_fbo {
            unsafe { cgl_fbo_use(f); }
        }
    }

    pub fn cgl_use_end()
    {
        unsafe { cgl_fbo_use_end(); }
    }
}
