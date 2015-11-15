use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied,Vacant};
use std::fs::File;
use rustc_serialize::{Encodable, Encoder, Decoder, Decodable};
use byteorder::{LittleEndian, ReadBytesExt};
use std::path::Path;
use std::io::Read;

//use libc::{c_char, c_int, c_uint, c_void};
use libc::{c_uint, c_void};
use std::mem;
use resource;
use shader;
use geometry;
use vec;

use self::DrawType::{Faces,Vertices,Lines};

#[repr(C)]
pub struct CglBuffer;

#[link(name = "cypher")]
extern {
    pub fn cgl_buffer_init(
        data : *const c_void,
        count : c_uint
        ) -> *const CglBuffer;

    pub fn cgl_buffer_index_init(
        data : *const c_void,
        count : c_uint
        ) -> *const CglBuffer;

    pub fn cgl_shader_attribute_send(
        att : *const shader::CglShaderAttribute,
        buffer : *const CglBuffer) -> ();

    pub fn cgl_buffer_update(
        buffer : *const CglBuffer,
        data : *const c_void,
        count : c_uint);
}

#[derive(Copy, Clone)]
pub enum BufferType
{
    Vertex,
    Index,
    Normal,
    Uv
}

/*
enum BufferState
{
    Created,
    Init,
    Usable,
    ToUpdate
}
*/

pub struct Buffer<T>
{
    pub name: String,
    pub data : Vec<T>,
    cgl_buffer: Option<*const CglBuffer>,
    buffer_type : BufferType,
    //state : BufferState
}

impl<T:Clone> Clone for Buffer<T>
{
    fn clone(&self) -> Buffer<T>
    {
        Buffer {
            name : self.name.clone(),
            data : self.data.clone(),
            cgl_buffer : self.cgl_buffer,
            buffer_type : self.buffer_type
        }
    }
}

unsafe impl Send for Mesh {}
unsafe impl Sync for Mesh {}

unsafe impl<T> Send for Buffer<T> {}
unsafe impl<T> Sync for Buffer<T> {}

impl<T:Clone> Buffer<T>
{
    pub fn new(name: String, data : Vec<T>, buffer_type : BufferType) -> Buffer<T>
    {
        Buffer {
            name : name,
            data : data,
            cgl_buffer : None,
            buffer_type : buffer_type,
            //state : BufferState::Created
        }
    }

    pub fn copy(&self) -> Buffer<T>
    {
        Buffer {
            name : self.name.clone(),
            data : self.data.clone(),
            cgl_buffer : None,
            buffer_type : self.buffer_type
        }
    }
}

pub trait BufferSend
{
    fn send(&mut self) -> ();
    fn utilise(&self, att : *const shader::CglShaderAttribute) ->();
    fn size_get(&self) -> usize;
    fn cgl_buffer_get(&self) -> Option<*const CglBuffer>;
    //fn update(&mut self) -> ();
}

/*
pub trait BufferSendTest : BufferSend + Clone{}
impl<T: BufferSend + Clone> BufferSendTest for T {}
*/

impl<T> BufferSend for Buffer<T> {
    fn send(&mut self) -> ()
    {
        match self.cgl_buffer {
            Some(b) => unsafe {
                cgl_buffer_update(
                    b,
                    mem::transmute(self.data.as_ptr()),
                    self.data.len() as c_uint);
                return;
            },
            None => {}
        };

        match self.buffer_type {
            BufferType::Vertex => unsafe {
                let cgl_buffer = cgl_buffer_init(
                    mem::transmute(self.data.as_ptr()),
                    self.data.len() as c_uint);
                self.cgl_buffer = Some(cgl_buffer);
            },
            BufferType::Index => unsafe {
                let cgl_buffer = cgl_buffer_index_init(
                    mem::transmute(self.data.as_ptr()),
                    self.data.len() as c_uint);
                self.cgl_buffer = Some(cgl_buffer);
            },
            _ => unsafe {
                let cgl_buffer = cgl_buffer_init(
                    mem::transmute(self.data.as_ptr()),
                    self.data.len() as c_uint);
                self.cgl_buffer = Some(cgl_buffer);
            }
        }
    }

    /*
    fn update(&mut self) -> ()
    {
        let cb = match self.cgl_buffer {
            Some(b) => b,
            None => return
        };

        unsafe {
            cgl_buffer_update(
                cb,
                mem::transmute(self.data.as_ptr()),
                self.data.len() as c_uint);
        }
    }
    */

    fn utilise(&self, att : *const shader::CglShaderAttribute) ->()
    {
        match self.cgl_buffer {
            Some(b) => unsafe {
                cgl_shader_attribute_send(att, b);
            },
            None => ()
        }
    }

    fn size_get(&self) -> usize
    {
        return self.data.len();
    }

    fn cgl_buffer_get(&self) -> Option<*const CglBuffer>
    {
        return self.cgl_buffer;
    }
}

#[derive(Clone)]
pub enum DrawType
{
    Faces,
    Vertices,
    Lines
}

pub struct VertexInfo
{
    pub position : vec::Vec3,
    pub normal : vec::Vec3
}

#[derive(Clone)]
pub struct Weight
{
    pub index : u16, // bone index
    pub weight : f32
}


#[derive(Clone)]
pub struct Mesh
{
    pub name : String,
    pub state : i32,
    //buffers : HashMap<String, Box<BufferSend+Send+Sync>>, //TODO check
    buffers_f32 : HashMap<String, Box<Buffer<f32>>>, //TODO check
    buffers_u32 : HashMap<String, Box<Buffer<u32>>>, //TODO check
    pub draw_type : DrawType,
    pub aabox : Option<geometry::AABox>,
    buffers_f32_base : HashMap<String, Box<Buffer<f32>>>, //TODO check
    pub weights : Vec<Vec<Weight>>
}

impl Mesh
{
    pub fn new() -> Mesh
    {
       let m = Mesh {
           name : String::from("mesh_new"),
           state : 0,
           buffers_f32 : HashMap::new(),
           buffers_u32 : HashMap::new(),
           draw_type : Faces,
           aabox : None,
           buffers_f32_base : HashMap::new(),
           weights : Vec::new()
       };

       /*
       let bufname = String::from("position");

       m.buffers.insert(bufname.clone(), 
                        box Buffer::new(
                            bufname.clone(),
                            Vec::from_slice(VERTEX_DATA),
                            Vertex));
                            */

       return m;
    }

    pub fn new_from_file(path : &str) -> Mesh
    {
       let m = Mesh {
           name : String::from(path),
           state : 0,
           buffers_f32 : HashMap::new(),
           buffers_u32 : HashMap::new(),
           draw_type : Faces,
           aabox : None,
           buffers_f32_base : HashMap::new(),
           weights : Vec::new(),
       };
        
       m
    }

    pub fn file_read(&mut self) 
    {
        if self.state != 0 {
            return;
        }

        let path : &Path = self.name.as_ref();
        let mut file = match File::open(path) {
            Ok(f) => {f},
            Err(e) => {
                println!("Error reading file '{}'. Error: {}", self.name, e);
                return;
            }
        };

        match file.metadata() {
            Ok(mt) => {
                if mt.is_dir() {
                    return;
                }
            },
            Err(e) => {
                println!("cannot get metadata");
                return;
            }
        }

       {
           let typelen = file.read_u16::<LittleEndian>().unwrap();
           println!("number : {} ", typelen);
           let mut typevec = vec![0u8; typelen as usize];
           file.read(&mut typevec);
           let typename = String::from_utf8(typevec).unwrap();
           println!("type name : {} ", typename);

           let len = file.read_u16::<LittleEndian>().unwrap();
           println!("number : {} ", len);
           let mut namevec = vec![0u8; len as usize];
           file.read(&mut namevec);
           let name = String::from_utf8(namevec).unwrap();
           println!("name : {} ", name);
       }

       {
           let vertex_count = file.read_u16::<LittleEndian>().unwrap();
           let count = (vertex_count as usize) * 3usize;
           let mut vvv : Vec<f32> = Vec::with_capacity(count);

           println!("vertex count : {} ", vertex_count);
           let mut min = vec::Vec3::zero();
           let mut max = vec::Vec3::zero();
           for i in 0usize..count
           {
               let x = file.read_f32::<LittleEndian>().unwrap();
               vvv.push(x);

               let x = x as f64;
               match i % 3 {
                   0 => {
                       if x < min.x { min.x = x; };
                       if x > max.x { max.x = x; };
                   },
                   1 => {
                       if x < min.y { min.y = x; };
                       if x > max.y { max.y = x; };
                   },
                   2 => {
                       if x < min.z { min.z = x; };
                       if x > max.z { max.z = x; };
                   },
                   _ => {}
               }
           }

           self.aabox = Some(geometry::AABox::new(min,max));
           println!("__________aabox is : {:?}", self.aabox);

           let bufname = String::from("position");

           let buf = Buffer::new(
               bufname.clone(),
               vvv,
               BufferType::Vertex);

           self.buffers_f32_base.insert(bufname.clone(), box buf.copy());
           self.buffers_f32.insert(bufname.clone(), box buf);

       }

       {
           let faces_count = file.read_u16::<LittleEndian>().unwrap();
           let count = (faces_count as usize) * 3usize;
           let mut fff : Vec<u32> = Vec::with_capacity(count);

           println!("faces count : {} ", faces_count);
           for _ in 0usize..count
           {
               let x = file.read_u16::<LittleEndian>().unwrap();
               fff.push(x as u32);
           }

           let bufname = String::from("faces");

           self.buffers_u32.insert(bufname.clone(), box Buffer::new(
                   bufname.clone(),
                   fff,
                   BufferType::Index));
       }

       {
           let normals_count = file.read_u16::<LittleEndian>().unwrap();
           if normals_count > 0 {
               let count = (normals_count as usize) * 3usize;
               let mut nnn : Vec<f32> = Vec::with_capacity(count);

               println!("normals count : {} ", normals_count);
               for _ in 0usize..count
               {
                   let x = file.read_f32::<LittleEndian>().unwrap();
                   nnn.push(x as f32);
               }

               let bufname = String::from("normal");
               let buf = Buffer::new(
                   bufname.clone(),
                   nnn,
                   BufferType::Normal);

               self.buffers_f32_base.insert(bufname.clone(), box buf.copy());
               self.buffers_f32.insert(bufname.clone(), box buf);
           }
       }

       {
           let uv_count = file.read_u16::<LittleEndian>().unwrap();
           if uv_count > 0 {
               let count = (uv_count as usize) * 2usize;
               let mut uuu : Vec<f32> = Vec::with_capacity(count);

               println!("uvs count : {} ", uv_count);
               for _ in 0usize..count
               {
                   let x = file.read_f32::<LittleEndian>().unwrap();
                   //TODO invert y in png
                   /*
                   if i % 2u == 1u
                   {
                       uuu.push((1f32 - x) as f32);
                   }
                   else 
                   */
                   {
                       uuu.push(x as f32);
                   }
               }

               let bufname = String::from("texcoord");

               self.buffers_f32.insert(bufname.clone(), box Buffer::new(
                       bufname.clone(),
                       uuu,
                       BufferType::Uv));
           }
       }

       //TODO weights
       {
           let group_count = file.read_u16::<LittleEndian>().unwrap();
           if group_count > 0 {
               for g in 0..group_count {
                   //TODO just this name is used
                   let group_name = read_string(&mut file);
                   let weight_count = file.read_u16::<LittleEndian>().unwrap();
                   println!("group name : {}, weight count : {} ", group_name, weight_count);
                   //TODO this is not used
                   for w in 0..weight_count as usize {
                       let index = file.read_u16::<LittleEndian>().unwrap();
                       let weight = file.read_f32::<LittleEndian>().unwrap();
                   }
               }
           }
       }

       {
           let vertex_weight_count = file.read_u16::<LittleEndian>().unwrap();
           println!("vertex weight count : {} ", vertex_weight_count);
           for _ in 0..vertex_weight_count {
               let weight_count = file.read_u16::<LittleEndian>().unwrap();
               let mut weights = Vec::with_capacity(weight_count as usize);
                
               if weight_count > 0 {
                   println!("  weight count : {} ", weight_count);
               }
               for _ in 0..weight_count {
                   let index = file.read_u16::<LittleEndian>().unwrap();
                   let weight = file.read_f32::<LittleEndian>().unwrap();
                    println!("    index, weight : {}, {} ", index, weight);
                    let w = Weight { index : index, weight : weight };
                    weights.push(w);
               }

               self.weights.push(weights);
           }
       }

       self.state = 1;
    }

    /*
    pub fn inittt(&mut self)
    {
        if self.state == 0 {
            //TODO can be read anywhere
            self.file_read();
        }
    }
    */
    pub fn init_buffers(&mut self)
    {
        if self.state == 1 {
            for (_,b) in self.buffers_u32.iter_mut() {
                b.send();
            }
            for (_,b) in self.buffers_f32.iter_mut() {
                b.send();
            }
            self.state = 11;
        }
    }

    pub fn buffer_f32_get(&self, name : &str) -> Option<&Box<Buffer<f32>>>
    {
        let s = String::from(name);

        match self.buffers_f32.get(&s) {
            Some(b) => return Some(b),
            None => None,
        }
    }

    pub fn buffer_f32_get_mut(&mut self, name : &str) -> Option<&mut Box<Buffer<f32>>>
    {
        let s = String::from(name);

        match self.buffers_f32.get_mut(&s) {
            Some(b) => return Some(b),
            None => None,
        }
    }

    pub fn buffer_u32_get(&self, name : &str) -> Option<&Box<Buffer<u32>>>
    {
        let s = String::from(name);

        match self.buffers_u32.get(&s) {
            Some(b) => return Some(b),
            None => None,
        }
    }

    pub fn add_line(&mut self, s : geometry::Segment, color : vec::Vec4)
    {
        let count = 6;
        let mut vvv : Vec<f32> = Vec::with_capacity(count);
        vvv.push(s.p0.x as f32);
        vvv.push(s.p0.y as f32);
        vvv.push(s.p0.z as f32);
        vvv.push(s.p1.x as f32);
        vvv.push(s.p1.y as f32);
        vvv.push(s.p1.z as f32);

        let mut colbuf : Vec<f32> = Vec::with_capacity(8);
        for i in 0u32..2 {
            colbuf.push(color.x as f32);
            colbuf.push(color.y as f32);
            colbuf.push(color.z as f32);
            colbuf.push(color.w as f32);
        }

        let name = String::from("position");
        match self.buffers_f32.entry(name.clone()) {
            Vacant(entry) => {
                let buffer = box Buffer::new(
                    name.clone(),
                    vvv,
                    BufferType::Vertex);

                entry.insert(buffer);
            },
            Occupied(entry) => {
                let en = entry.into_mut();
                en.data.push_all(vvv.as_ref());
            }
        };

        let name = String::from("color");
        match self.buffers_f32.entry(name.clone()) {
            Vacant(entry) => {
                let buffer = box Buffer::new(
                    name.clone(),
                    colbuf,
                    BufferType::Vertex);

                entry.insert(buffer);
            },
            Occupied(entry) => {
                let en = entry.into_mut();
                en.data.push_all(colbuf.as_ref());
            }
        };

        self.draw_type = Lines;

        self.state = 1;
    }

    pub fn add_aabox(&mut self, aabox : &geometry::AABox, color : vec::Vec4)
    {
        let s = geometry::Segment::new(
            aabox.min,
            vec::Vec3::new(aabox.max.x, aabox.min.y, aabox.min.z));
        self.add_line(s, color);
        let s = geometry::Segment::new(
            aabox.min,
            vec::Vec3::new(aabox.min.x, aabox.max.y, aabox.min.z));
        self.add_line(s, color);
        let s = geometry::Segment::new(
            aabox.min,
            vec::Vec3::new(aabox.min.x, aabox.min.y, aabox.max.z));
        self.add_line(s, color);
        let s = geometry::Segment::new(
            vec::Vec3::new(aabox.min.x, aabox.max.y, aabox.min.z),
            vec::Vec3::new(aabox.max.x, aabox.max.y, aabox.min.z));
        self.add_line(s, color);

        let s = geometry::Segment::new(
            vec::Vec3::new(aabox.min.x, aabox.max.y, aabox.min.z),
            vec::Vec3::new(aabox.min.x, aabox.max.y, aabox.max.z));
        self.add_line(s, color);

        let s = geometry::Segment::new(
            vec::Vec3::new(aabox.min.x, aabox.min.y, aabox.max.z),
            vec::Vec3::new(aabox.min.x, aabox.max.y, aabox.max.z));
        self.add_line(s, color);

        let s = geometry::Segment::new(
            vec::Vec3::new(aabox.min.x, aabox.min.y, aabox.max.z),
            vec::Vec3::new(aabox.max.x, aabox.min.y, aabox.max.z));
        self.add_line(s, color);

        let s = geometry::Segment::new(
            vec::Vec3::new(aabox.min.x, aabox.max.y, aabox.max.z),
            vec::Vec3::new(aabox.max.x, aabox.max.y, aabox.max.z));
        self.add_line(s, color);

        let s = geometry::Segment::new(
            vec::Vec3::new(aabox.max.x, aabox.min.y, aabox.min.z),
            vec::Vec3::new(aabox.max.x, aabox.max.y, aabox.min.z));
        self.add_line(s, color);

        let s = geometry::Segment::new(
            vec::Vec3::new(aabox.max.x, aabox.min.y, aabox.max.z),
            vec::Vec3::new(aabox.max.x, aabox.max.y, aabox.max.z));
        self.add_line(s, color);

        let s = geometry::Segment::new(
            vec::Vec3::new(aabox.max.x, aabox.min.y, aabox.min.z),
            vec::Vec3::new(aabox.max.x, aabox.min.y, aabox.max.z));
        self.add_line(s, color);

        let s = geometry::Segment::new(
            vec::Vec3::new(aabox.max.x, aabox.max.y, aabox.min.z),
            vec::Vec3::new(aabox.max.x, aabox.max.y, aabox.max.z));
        self.add_line(s, color);


    }

    pub fn clear_lines(&mut self)
    {
        let name = String::from("position");
        self.buffers_f32.remove(&name);
        let name = String::from("color");
        self.buffers_f32.remove(&name);
    }

    pub fn add_quad(&mut self, w : f32, h : f32)
    {
        {
        let name = String::from("position");
        let hw = w/2f32;
        let hh = h/2f32;
        let mut vvv : Vec<f32> = Vec::with_capacity(4*3);
        vvv.push(-hw);
        vvv.push(-hh);
        vvv.push(0f32);

        vvv.push(hw);
        vvv.push(-hh);
        vvv.push(0f32);

        vvv.push(-hw);
        vvv.push(hh);
        vvv.push(0f32);

        vvv.push(hw);
        vvv.push(hh);
        vvv.push(0f32);


        let buffer = box Buffer::new(
            name.clone(),
            vvv,
            BufferType::Vertex);

        match self.buffers_f32.entry(name) {
            Vacant(entry) => {entry.insert(buffer);},
            Occupied(entry) => {
                let en = entry.into_mut();
                *en = buffer;
            }
        };
        }

        {
        let name = String::from("faces");
        let mut fff : Vec<u32> = Vec::with_capacity(6);
        fff.push(0u32);
        fff.push(1u32);
        fff.push(3u32);
        fff.push(0u32);
        fff.push(3u32);
        fff.push(2u32);

        let buffer = box Buffer::new(
            name.clone(),
            fff,
            BufferType::Index);

        match self.buffers_u32.entry(name) {
            Vacant(entry) => {entry.insert(buffer);},
            Occupied(entry) => {
                let en = entry.into_mut();
                *en = buffer;
            }
        };
        }

        {
            let name = String::from("texcoord");
            let mut uuu : Vec<f32> = Vec::with_capacity(4*2);
            uuu.push(0f32);
            uuu.push(0f32);

            uuu.push(1f32);
            uuu.push(0f32);

            uuu.push(0f32);
            uuu.push(1f32);

            uuu.push(1f32);
            uuu.push(1f32);

            let buffer = box Buffer::new(
                name.clone(),
                uuu,
                BufferType::Uv);

            match self.buffers_f32.entry(name) {
                Vacant(entry) => { entry.insert(buffer); },
                Occupied(entry) => {
                    let en = entry.into_mut();
                    *en = buffer;
                }
            };
        }

        self.state = 1;
    }

}

impl resource::ResourceT for Mesh
{
    fn init(&mut self)
    {
        if self.state == 0 {
            self.file_read();
        }
        
        if self.state == 1 {
            for (_,b) in self.buffers_f32.iter_mut() {
                Some(b.send());
            }
            for (_,b) in self.buffers_u32.iter_mut() {
                Some(b.send());
            }
            self.state = 11;
        }
    }
}

impl Encodable for Mesh {
  fn encode<E : Encoder>(&self, encoder: &mut E) -> Result<(), E::Error> {
      encoder.emit_struct("Mesh", 1, |encoder| {
          try!(encoder.emit_struct_field( "name", 0usize, |encoder| self.name.encode(encoder)));
          Ok(())
      })
  }
}

impl Decodable for Mesh {
  fn decode<D : Decoder>(decoder: &mut D) -> Result<Mesh, D::Error> {
    decoder.read_struct("root", 0, |decoder| {
         Ok(Mesh{
          name: try!(decoder.read_struct_field("name", 0, |decoder| Decodable::decode(decoder))),
           state : 0,
           buffers_f32 : HashMap::new(),
           buffers_u32 : HashMap::new(),
           draw_type : Faces,
           aabox : None,
           buffers_f32_base : HashMap::new(),
           weights : Vec::new()
        })
    })
  }
}

pub fn read_string(file: &mut File ) -> String
{
    let typelen = file.read_u16::<LittleEndian>().unwrap();
    println!("number : {} ", typelen);
    let mut typevec = vec![0u8; typelen as usize];
    file.read(&mut typevec);
    String::from_utf8(typevec).unwrap()
}

/*
//static VERTEX_DATA: [GLfloat, ..6] = [
static VERTEX_DATA: [f32, ..6] = [
    0.0,  0.5,
    0.5, -0.5,
    -0.5, -0.5
      ];
      */


