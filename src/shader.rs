use std::collections::HashMap;
use rustc_serialize::{json, Encodable, Encoder, Decoder, Decodable};
use std::fs::File;
use std::io::{BufReader, BufRead, Read};
use libc::{c_char, c_uint};
use std::ptr;
use std::str::FromStr;
use std::ffi::CString;
use std::path::Path;
//use std::default::Default;
//use toml;

use vec;
use resource;
use uniform::UniformSend;
use uniform::TextureSend;
use texture;

#[repr(C)]
pub struct CglShader;
#[repr(C)]
pub struct CglShaderAttribute;
#[repr(C)]
pub struct CglShaderUniform;

pub struct Shader
{
    pub name : String,
    pub attributes : HashMap<String, *const CglShaderAttribute>,
    pub uniforms : HashMap<String, *const CglShaderUniform>,
    pub state : i32,

    vert : Option<String>,
    frag : Option<String>,

    cgl_shader : Option<*const CglShader>, 
}

unsafe impl Send for Shader {}
unsafe impl Sync for Shader {}

impl Shader
{
    fn attribute_add(&mut self, name : &str, size : u32)
    {
        let attc = CString::new(name.as_bytes()).unwrap();

        match self.cgl_shader {
            None => {},
            Some(cs) =>
                unsafe {
                    let cgl_att = cgl_shader_attribute_new(cs, attc.as_ptr(), size);
                    if cgl_att != ptr::null() {
                        self.attributes.insert(String::from(name), cgl_att);
                    }
                }
        }

    }

    fn uniform_add(&mut self, name : &str)
    {
        let unic = CString::new(name.as_bytes()).unwrap();

        match self.cgl_shader {
            None => {},
            Some(cs) =>
                unsafe {
                    let cgl_uni = cgl_shader_uniform_new(cs, unic.as_ptr());
                    if cgl_uni != ptr::null() {
                        self.uniforms.insert(String::from(name), cgl_uni);
                    }
                }
        }
    }

    pub fn uniform_set(&self, name : &str, value : &UniformSend)
    {
        match self.uniforms.get(&String::from(name)) {
            Some(uni) => value.uniform_send(*uni),
            None => {
                println!("ERR!!!! : could not find such uniform '{}'",name)
            }
        }
    }

    pub fn texture_set(&self, name : &str, value : &TextureSend, index : u32)
    {
        match self.uniforms.get(&String::from(name)) {
            Some(uni) => value.uniform_send(*uni, index),
            None => {}//println!("ERR!!!! : could not find such uniform '{}'",name)
        }
    }

    pub fn utilise(&self)
    {
        match self.cgl_shader {
            None => {},
            Some(cs) =>
                unsafe {
                    cgl_shader_use(cs);
                }
        }
    }

    pub fn new(name : &str) -> Shader
    {
        Shader {
            name : String::from(name),
            cgl_shader : None,
            attributes : HashMap::new(),
            uniforms : HashMap::new(),
            vert : None,
            frag : None,
            state : 0
        }
    }

    pub fn read(&mut self)
    {
        let mut file = {
            let path = Path::new(&self.name);
            BufReader::new(File::open(&path).ok().unwrap())
        };

        let mut frag = String::new();
        let mut vert = String::new();

        match file.read_line(&mut vert) {
            Ok(_) => { vert.pop(); },
            Err(_) => return
        }
 
        match file.read_line(&mut frag) {
            Ok(_) => { frag.pop();},
            Err(_) => return
        }

        self.read_vert_frag(vert.as_ref(), frag.as_ref());

        //TODO remove from here
        self.cgl_init();

        for line in file.lines() {
            let l = line.unwrap();
            let split : Vec<&str> = l.split(',').collect();
            if split[0] == "att" {
                let size : u32;
                /*let op : Option<u32> = FromStr::from_str(split[2]);
                match op {
                    Some(u) => size = u,
                    None => continue
                }
                */
                let op = FromStr::from_str(split[2]);
                match op {
                    Ok(u) => size = u,
                    _ => continue
                }
                println!("it's an attribute {}, {}", split[1], size);
                self.attribute_add(split[1], size);
            }
            else if split[0] == "uni" {
                self.uniform_add(split[1]);
                println!("it's an uniform {} yoo", split[1]);
                if split[2] == "vec4" {
                    //TODO
                }
            }
        }

        self.state = 2;
    }

    fn read_vert_frag(&mut self, vertpath : &str, fragpath : &str)
    {
        if self.state > 1 {
            return
        }

        {
            let mut contents = String::new();
            match File::open(&Path::new(fragpath)).ok().unwrap().read_to_string(&mut contents){
                Ok(_) => self.frag = Some(contents),
                _ => return
            }
        }

        {
            let mut contents = String::new();
            match File::open(&Path::new(vertpath)).ok().unwrap().read_to_string(&mut contents) {
                Ok(_) => self.vert = Some(contents),
                _ => return
            }
        }

        self.state = 1;
    }

    pub fn cgl_init(&mut self)
    {
        let vertc;
        match self.vert {
            None => return,
            Some(ref v) => {
                vertc = CString::new(v.as_bytes()).unwrap();
            }
        }

        let fragc;
        match self.frag {
            None => return,
            Some(ref f) => {
                fragc = CString::new(f.as_bytes()).unwrap();
            }
        }

        let vertcp = vertc.as_ptr();
        let fragcp = fragc.as_ptr();

        unsafe {
            let shader = cgl_shader_init_string(vertcp, fragcp);
            self.cgl_shader = Some(shader);
        }

        self.state = 3;
    }


}


#[derive(Clone,RustcDecodable,RustcEncodable)]
pub enum UniformData
{
    Int(i32),
    Float(f32),
    Vec2(vec::Vec2),
    Vec3(vec::Vec3),
    Vec4(vec::Vec4),
}

macro_rules! unimatch(
    ($inp:expr, $uni:expr, [ $($sp:ident)|+ ]) => (
        match $inp {
            $(
                UniformData::$sp(ref x) => { x.uniform_send($uni); }
             )+
            //_ => {}
        }
    );
);

impl UniformSend for UniformData
{
    fn uniform_send(&self, uni : *const CglShaderUniform) ->()
    {
        unimatch!(*self, uni, [Int|Float|Vec2|Vec3|Vec4]);
    }
}

#[link(name = "cypher")]
extern {
    fn cgl_shader_init_string(
        vert : *const c_char,
        frat : *const c_char) -> *const CglShader;

    pub fn cgl_shader_use(shader : *const CglShader);

    pub fn cgl_shader_attribute_new(
        shader : *const CglShader,
        name : *const c_char,
        size : c_uint) -> *const CglShaderAttribute;

    pub fn cgl_shader_uniform_new(
        shader : *const CglShader,
        name : *const c_char) -> *const CglShaderUniform;

}

//impl <S: Encoder<E>, E> Encodable<S, E> for Shader {
impl Encodable for Shader {
  fn encode<S : Encoder>(&self, encoder: &mut S) -> Result<(), S::Error> {
      encoder.emit_struct("Mesh", 1, |encoder| {
          try!(encoder.emit_struct_field( "name", 0usize, |encoder| self.name.encode(encoder)));
          Ok(())
      })
  }
}

impl Decodable for Shader {
  fn decode<D : Decoder>(decoder: &mut D) -> Result<Shader, D::Error> {
    decoder.read_struct("root", 0, |decoder| {
         Ok(Shader{
          name: try!(decoder.read_struct_field("name", 0, |decoder| Decodable::decode(decoder))),
             cgl_shader : None,
             attributes : HashMap::new(),
             uniforms : HashMap::new(),
             vert : None,
             frag : None,
             state : 0
        })
    })
  }
}

