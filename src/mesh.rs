extern crate libc;

use self::libc::{c_int, c_void};
use std::mem;

#[link(name = "cypher")]
extern {
    fn buffer_request_add(vertex : *const c_void  , count : c_int) -> c_int;
}

//static VERTEX_DATA: [GLfloat, ..6] = [
static VERTEX_DATA: [f32, ..6] = [
    0.0,  0.5,
    0.5, -0.5,
    -0.5, -0.5
      ];

pub fn mesh_init()
{
    unsafe {
        buffer_request_add(mem::transmute(&VERTEX_DATA[0]), 6);
    }
}

