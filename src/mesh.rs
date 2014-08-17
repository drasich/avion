extern crate libc;

use self::libc::{c_int, c_void};
use std::mem;

#[link(name = "cypher")]
extern {
    /*
    pub fn buffer_request_add(
        mesh : *mut Mesh,
        vertex : *const c_void,
        count : c_int,
        cb: extern fn(*mut Mesh, i32, *const Buffer)
        ) -> c_int;
        */

    pub fn buffer_init(
        vertex : *const c_void,
        count : c_int
        ) -> *const Buffer;
}

pub struct Buffer;

pub struct Mesh
{
    pub name : String,
    pub buffer: Option<*const Buffer>,
    pub state : i32
}

impl Mesh
{
    pub fn new() -> Mesh
    {
       Mesh { name : String::from_str("mesh_new"), buffer : None, state : 0 }
    }

    pub fn init(&mut self)
    {
        if self.state != 11 {
            unsafe {
                self.buffer = Some( buffer_init(mem::transmute(&VERTEX_DATA[0]), 6) );
            }
            self.state = 11;
        }
    }
}

//static VERTEX_DATA: [GLfloat, ..6] = [
static VERTEX_DATA: [f32, ..6] = [
    0.0,  0.5,
    0.5, -0.5,
    -0.5, -0.5
      ];

