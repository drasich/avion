use std::collections::{DList,Deque};
use std::collections::HashMap;
use std::io::File;
use serialize::{json, Encodable, Encoder, Decoder, Decodable};

use libc::{c_char, c_int, c_uint, c_void};
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

    pub fn cgl_buffer_index_init(
        data : *const c_void,
        count : c_uint
        ) -> *const CglBuffer;

    pub fn cgl_shader_attribute_send(
        att : *const shader::CglShaderAttribute,
        name : *const c_char,
        buffer : *const CglBuffer) -> ();
}

pub enum BufferType
{
    Vertex,
    Index
}

pub struct Buffer<T>
{
    pub name: String,
    pub data : Vec<T>,
    pub cgl_buffer: Option<*const CglBuffer>,
    buffer_type : BufferType
}

impl<T> Buffer<T>
{
    pub fn new(name: String, data : Vec<T>, buffer_type : BufferType) -> Buffer<T>
    {
        Buffer {
            name : name,
            data : data,
            cgl_buffer : None,
            buffer_type : buffer_type
        }
    }
}

pub trait BufferSend
{
    fn send(&mut self) -> ();
    fn utilise(&self, att : *const shader::CglShaderAttribute) ->();
    fn size_get(&self) -> uint;
    fn buffer_get(&self) -> Option<*const CglBuffer>;
}

impl<T> BufferSend for Buffer<T> {
    fn send(&mut self) -> ()
    {
        match (self.buffer_type) {
            Vertex => unsafe {
                let cgl_buffer = cgl_buffer_init(
                    mem::transmute(self.data.as_ptr()),
                    self.data.len() as c_uint);
                self.cgl_buffer = Some(cgl_buffer);
            },
            Index => unsafe {
                let cgl_buffer = cgl_buffer_index_init(
                    mem::transmute(self.data.as_ptr()),
                    self.data.len() as c_uint);
                self.cgl_buffer = Some(cgl_buffer);
            }
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

    fn size_get(&self) -> uint
    {
        return self.data.len();
    }

    fn buffer_get(&self) -> Option<*const CglBuffer>
    {
        return self.cgl_buffer;
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

       m.buffers.insert(bufname.clone(), 
                        box Buffer::new(
                            bufname.clone(),
                            Vec::from_slice(VERTEX_DATA),
                            Vertex));

       return m;
    }

    pub fn new_from_file(path : &str) -> Mesh
    {
       let mut m = Mesh {
           name : String::from_str(path),
           state : 0,
           vertex : Vec::new(),
           buffers : HashMap::new(),
       };

       let mut file = File::open(&Path::new(path));

       let typelen = file.read_le_u16().unwrap();
       println!("number : {} ", typelen);
       let typevec = file.read_exact(typelen as uint).unwrap();
       let typename = String::from_utf8(typevec).unwrap();
       println!("type name : {} ", typename);

       let len = file.read_le_u16().unwrap();
       println!("number : {} ", len);
       let namevec = file.read_exact(len as uint).unwrap();
       let name = String::from_utf8(namevec).unwrap();
       println!("name : {} ", name);

       {
       let vertex_count = file.read_le_u16().unwrap();
       let count = (vertex_count as uint) * 3u;
       let mut vvv : Vec<f32> = Vec::with_capacity(count);

       println!("vertex count : {} ", vertex_count);
       for i in range(0u, count)
       {
           let x = file.read_le_f32().unwrap();
           vvv.push(x);
       }

       let bufname = String::from_str("position");

       m.buffers.insert(bufname.clone(), box Buffer::new(
               bufname.clone(),
               vvv,
               Vertex));
       }

       {
       let faces_count = file.read_le_u16().unwrap();
       let count = (faces_count as uint) * 3u;
       let mut fff : Vec<u32> = Vec::with_capacity(count);

       println!("faces count : {} ", faces_count);
       for i in range(0u, count)
       {
           let x = file.read_le_u16().unwrap();
           fff.push(x as u32);
       }

       let bufname = String::from_str("faces");

       m.buffers.insert(bufname.clone(), box Buffer::new(
               bufname.clone(),
               fff,
               Index));
       }

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

impl <S: Encoder<E>, E> Encodable<S, E> for Mesh {
  fn encode(&self, encoder: &mut S) -> Result<(), E> {
      encoder.emit_struct("Mesh", 1, |encoder| {
          try!(encoder.emit_struct_field( "name", 0u, |encoder| self.name.encode(encoder)));
          Ok(())
      })
  }
}

impl<S: Decoder<E>, E> Decodable<S, E> for Mesh {
  fn decode(decoder: &mut S) -> Result<Mesh, E> {
    decoder.read_struct("root", 0, |decoder| {
         Ok(Mesh{
          name: try!(decoder.read_struct_field("name", 0, |decoder| Decodable::decode(decoder))),
           state : 0,
           vertex : Vec::from_slice(VERTEX_DATA),
           buffers : HashMap::new()
        })
    })
  }
}

//static VERTEX_DATA: [GLfloat, ..6] = [
static VERTEX_DATA: [f32, ..6] = [
    0.0,  0.5,
    0.5, -0.5,
    -0.5, -0.5
      ];

