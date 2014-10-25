use libc::{c_uint, c_int};

#[repr(C)]
pub struct CglFbo;

#[link(name = "cypher")]
extern {
    pub fn cgl_create_fbo() -> *const CglFbo;
    pub fn cgl_fbo_use(fbo : *const CglFbo);
    pub fn cgl_fbo_use_end();
    pub fn cgl_fbo_resize(fbo : *const CglFbo, w : c_int, h : c_int);
    pub fn cgl_fbo_destroy(fbo : *const CglFbo);
}

pub struct Fbo
{
    name : String,
    pub state : i32,
    pub cgl_fbo: Option<*const CglFbo>,
} 

