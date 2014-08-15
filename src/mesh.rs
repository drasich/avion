extern crate libc;

use self::libc::{c_int, c_void};
use std::mem;

pub struct Buffer;

pub struct Mesh
{
    pub name : String,
    pub buffer: Option<*const Buffer>,
    pub state : i32
}

impl Mesh
{
    pub fn set_buffer(&self, name : String)
    {

    }

    pub fn new() -> Mesh
    {
       Mesh { name : String::from_str("fromnew"), buffer : None, state : 0 }
    }
}

pub extern fn mesh_cb_result(mesh : *mut Mesh, answer_code :i32, buffer : *const Buffer) {
    println!("mesh cb result I am called from C with value {}", answer_code);
    //*
    unsafe {
        println!(" mesh cb result I am called from C, mesh is : {}", (*mesh).name);
        (*mesh).state = answer_code;
        (*mesh).buffer = Some(buffer);
    }
    //*/
    //TODO add this shader to list of shader
}


#[link(name = "cypher")]
extern {
    pub fn buffer_request_add(
        mesh : *mut Mesh,
        vertex : *const c_void,
        count : c_int,
        cb: extern fn(*mut Mesh, i32, *const Buffer)
        ) -> c_int;

    fn draw_data_set(
        data : *const c_void
        ) -> ();
}

//static VERTEX_DATA: [GLfloat, ..6] = [
static VERTEX_DATA: [f32, ..6] = [
    0.0,  0.5,
    0.5, -0.5,
    -0.5, -0.5
      ];

pub fn mesh_init()
{
    let mut mesh = box Mesh { name : String::from_str("mesh_name"), buffer : None, state : 0 };
    unsafe {
        buffer_request_add(&mut *mesh, mem::transmute(&VERTEX_DATA[0]), 6, mesh_cb_result);
    }
}

pub fn mesh_buffer_init(mesh : &mut Mesh)
{
    unsafe {
        buffer_request_add(&mut *mesh, mem::transmute(&VERTEX_DATA[0]), 6, mesh_cb_result);
    }

    mesh.state = 1;
}

