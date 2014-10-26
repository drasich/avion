use std::collections::HashMap;
use serialize::{json, Encodable, Encoder, Decoder, Decodable};
use std::io::File;
use std::io::BufferedReader;
//use std::default::Default;
use toml;


use vec;
//use matrix;
use resource;
//use uniform;
use uniform::UniformSend;
use uniform::TextureSend;
use texture;
use std::ptr;

use libc::{c_char, c_uint};

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

    vert : Option<String>,
    frag : Option<String>,

    cgl_shader : Option<*const CglShader>, 
    pub state : int,
}

impl Shader
{
    fn attribute_add(&mut self, name : &str, size : u32)
    {
        let attc = name.to_c_str();

        match self.cgl_shader {
            None => {},
            Some(cs) =>
                unsafe {
                    let cgl_att = cgl_shader_attribute_new(cs, attc.as_ptr(), size);
                    if cgl_att != ptr::null() {
                        self.attributes.insert(String::from_str(name), cgl_att);
                    }
                }
        }

    }

    fn uniform_add(&mut self, name : &str)
    {
        let unic = name.to_c_str();

        match self.cgl_shader {
            None => {},
            Some(cs) =>
                unsafe {
                    let cgl_uni = cgl_shader_uniform_new(cs, unic.as_ptr());
                    if cgl_uni != ptr::null() {
                        self.uniforms.insert(String::from_str(name), cgl_uni);
                    }
                }
        }
    }

    pub fn uniform_set(&self, name : &str, value : &UniformSend)
    {
        match self.uniforms.find(&String::from_str(name)) {
            Some(uni) => value.uniform_send(*uni),
            None => {
                println!("ERR!!!! : could not find such uniform '{}'",name)
            }
        }
    }

    pub fn texture_set(&self, name : &str, value : &TextureSend, index : u32)
    {
        match self.uniforms.find(&String::from_str(name)) {
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
            name : String::from_str(name),
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
        let path = Path::new(self.name.clone());
        let mut file = BufferedReader::new(File::open(&path));

        let mut frag : String;
        let mut vert : String;

        match file.read_line() {
            Ok(l) => { vert = l; vert.pop(); },
            Err(_) => return
        }
 
        match file.read_line() {
            Ok(l) => { frag = l; frag.pop();},
            Err(_) => return
        }

        self.read_vert_frag(vert.as_slice(), frag.as_slice());

        //TODO remove from here
        self.cgl_init();

        for line in file.lines() {
            let l = line.unwrap();
            let split : Vec<&str> = l.as_slice().split(',').collect();
            if split[0] == "att" {
                let size : u32;
                match from_str(split[2]) {
                    Some(u) => size = u,
                    None => continue
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

        let contents = File::open(&Path::new(fragpath)).read_to_string();

        match contents {
            Ok(r) => self.frag = Some(r),
            _ => return
        }

        let contents = File::open(&Path::new(vertpath)).read_to_string();

        match contents {
            Ok(r) => self.vert = Some(r),
            _ => return
        }

        self.state = 1;
    }

    pub fn cgl_init(&mut self)
    {
        let mut vertc;
        match self.vert {
            None => return,
            Some(ref v) => {
                vertc = v.to_c_str();
            }
        }

        let mut fragc;
        match self.frag {
            None => return,
            Some(ref f) => {
                fragc = f.to_c_str();
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


#[deriving(Decodable,Encodable)]
pub enum UniformData
{
    Int(i32),
    Float(f32),
    Vec2(vec::Vec2),
    Vec3(vec::Vec3),
    Color(vec::Vec4),
}

macro_rules! unimatch(
    ($inp:expr, $uni:expr, [ $($sp:ident)|+ ]) => (
        match $inp {
            $(
                $sp(x) => { x.uniform_send($uni); }
             )+
            _ => {}
        }
    );
)

impl UniformSend for UniformData
{
    fn uniform_send(&self, uni : *const CglShaderUniform) ->()
    {
        unimatch!(*self, uni, [Float|Color]);
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

impl <S: Encoder<E>, E> Encodable<S, E> for Shader {
  fn encode(&self, encoder: &mut S) -> Result<(), E> {
      encoder.emit_struct("Mesh", 1, |encoder| {
          try!(encoder.emit_struct_field( "name", 0u, |encoder| self.name.encode(encoder)));
          Ok(())
      })
  }
}

impl<S: Decoder<E>, E> Decodable<S, E> for Shader {
  fn decode(decoder: &mut S) -> Result<Shader, E> {
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

