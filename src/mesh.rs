extern crate libc;
use std::collections::{DList,Deque};
use std::collections::HashMap;

use self::libc::{c_char, c_int, c_uint, c_void};
use std::mem;
use resource;
use shader;

pub struct CglBuffer;

#[link(name = "cypher")]
extern {
    pub fn cgl_buffer_init(
        vertex : *const c_void,
        count : c_uint
        ) -> *const CglBuffer;

    pub fn cgl_shader_attribute_send(
        att : *const shader::CglShaderAttribute,
        name : *const c_char,
        buffer : *const CglBuffer) -> ();
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
    fn send(&mut self) -> ();
    fn utilise(&self, att : *const shader::CglShaderAttribute) ->();
}

impl<T> BufferSend for Buffer<T> {
    fn send(&mut self) -> ()
    {
        unsafe {
            let cgl_buffer = cgl_buffer_init(
                mem::transmute(self.data.as_ptr()),
                self.data.len() as c_uint);
            self.cgl_buffer = Some(cgl_buffer);
        }
    }

    fn utilise(&self, att : *const shader::CglShaderAttribute) ->()
    {
        match self.cgl_buffer {
            Some(b) =>
                unsafe {
                    cgl_shader_attribute_send(att, self.name.to_c_str().as_ptr(), b);
                },
                None => ()
        }
    }
}

pub struct Mesh
{
    pub name : String,
    pub state : i32,
    pub vertex : Vec<f32>,
    pub buffers : HashMap<String, Box<BufferSend>>,
}

impl Mesh
{
    pub fn new() -> Mesh
    {
       let mut m = Mesh {
           name : String::from_str("mesh_new"),
           state : 0,
           vertex : Vec::from_slice(VERTEX_DATA),
           buffers : HashMap::new(),
       };

       let bufname = String::from_str("position");

       m.buffers.insert(bufname.clone(), box Buffer::new(
               bufname.clone(),
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
                for (_,b) in self.buffers.mut_iter() {
                    Some(b.send());
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

