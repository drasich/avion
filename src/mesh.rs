extern crate libc;
use std::collections::{DList,Deque};

use self::libc::{c_int, c_uint, c_void};
use std::mem;
use resource;

pub struct CglBuffer;

#[link(name = "cypher")]
extern {
    pub fn cgl_buffer_init(
        vertex : *const c_void,
        count : c_uint
        ) -> *const CglBuffer;
}

pub struct Buffer<T>
{
    pub name: String,
    pub data : Vec<T>,
    pub cgl_buffer: Option<*const CglBuffer>,
}

impl<T> Buffer<T>
{
    pub fn new(name: String, data : Vec<T>) -> Buffer<T>
    {
        Buffer {
            name : name,
            data : data,
            cgl_buffer : None}
    }
}

pub trait BufferSend
{
    fn send(&mut self) -> *const CglBuffer;
}

impl<T> BufferSend for Buffer<T> {
    fn send(&mut self) -> *const CglBuffer
    {
        unsafe {
            let cgl_buffer = cgl_buffer_init(
                mem::transmute(self.data.as_ptr()),
                self.data.len() as c_uint);
            self.cgl_buffer = Some(cgl_buffer);
            return cgl_buffer;
        }
    }
}

pub struct Mesh
{
    pub name : String,
    //TODO remove buffer
    pub buffer: Option<*const CglBuffer>,
    pub state : i32,
    pub vertex : Vec<f32>,
    pub buffers : DList<Box<BufferSend>>,
}

impl Mesh
{
    pub fn new() -> Mesh
    {
       let mut m = Mesh {
           name : String::from_str("mesh_new"),
           buffer : None,
           state : 0,
           vertex : Vec::from_slice(VERTEX_DATA),
           buffers : DList::new(),
       };

       m.buffers.push( box Buffer::new(
               String::from_str("mybuf"),
               Vec::from_slice(VERTEX_DATA)));

       return m;
    }
}

impl resource::ResourceT for Mesh
{
    fn init(&mut self)
    {
        if self.state != 11 {
            unsafe {
                for b in self.buffers.mut_iter() {
                    self.buffer = Some(b.send());
                }
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

