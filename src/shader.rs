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
                    self.attributes.insert(String::from_str(name), cgl_att);
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
                    self.uniforms.insert(String::from_str(name), cgl_uni);
                }
        }
    }

    pub fn uniform_set(&self, name : &str, value : &UniformSend)
    {
        match self.uniforms.find(&String::from_str(name)) {
            Some(uni) => value.uniform_send(*uni),
            None => println!("ERR!!!! : could not find such uniform '{}'",name)
        }
    }

    pub fn texture_set(&self, name : &str, value : &TextureSend, index : u32)
    {
        match self.uniforms.find(&String::from_str(name)) {
            Some(uni) => value.uniform_send(*uni, index),
            None => println!("ERR!!!! : could not find such uniform '{}'",name)
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

//#[deriving(Decodable, Encodable, Default)]
//#[deriving(Encodable, Default)]
pub struct Material
{
    pub name : String,
    pub shader: Option<resource::ResTT<Shader>>,
    pub state : i32,
    pub textures : Vec<resource::ResTT<texture::Texture>>,
    //pub uniforms : HashMap<String, Box<UniformSend+'static>>,
    pub uniforms : HashMap<String, Box<UniformData>>,
}


/*
impl Default for Material
{
    fn default() -> Material {
          Material {
            name : String::from_str("default"),
            shader : None,
            state : 0,
            texture : None,
            textures : DList::new()
        }
    }
}
*/

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

impl Material
{
    pub fn new(name : &str) -> Material
    {
        Material {
            name : String::from_str(name),
            shader : None,
            state : 0,
            textures : Vec::new(),
            uniforms : HashMap::new(),
        }
    }

    pub fn new_from_file(file_path : &str) -> Material
    {
        let file = File::open(&Path::new(file_path)).read_to_string().unwrap();
        let mat : Material = json::decode(file.as_slice()).unwrap();
        mat
    }

    pub fn read(&mut self)
    {
        //TODO 

        let file = File::open(&Path::new(self.name.as_slice())).read_to_string().unwrap();
        //let mut mat : Material = json::decode(file.as_slice()).unwrap();
        let mat : Material = match json::decode(file.as_slice()){
            Ok(m) => m,
            Err(e) => { 
                println!("{}, line {}: error reading material '{}': {}, creating new material",
                         file!(),
                         line!(),
                         self.name,
                         e); Material::new(self.name.as_slice()) }
        };

        self.name = mat.name.clone();
        match mat.shader {
            Some(s) => 
                self.shader = Some(resource::ResTT::new(s.name.as_slice())),
            None => self.shader = None
        }

        for t in mat.textures.iter()
        {
            self.textures.push(resource::ResTT::new(t.name.as_slice()));
        }
    }

    pub fn save(&self)
    {
        let mut file = File::create(&Path::new(self.name.as_slice()));
        //let mut stdwriter = stdio::stdout();
        let mut encoder = json::PrettyEncoder::new(&mut file);
        //let mut encoder = json::Encoder::new(&mut file);
        self.encode(&mut encoder).unwrap();
    }

    pub fn savetoml(&self)
    {
        /*
        let mut encoder = toml::Encoder::new();
        let yep = self.encode(&mut encoder).unwrap();
        println!("yep : {} ", yep );
        //println!("encoder : {} ", encoder );
        println!("encoder toml : {} ", encoder.toml );
        */
        let s = toml::encode_str(self);
        println!("encoder toml : {} ", s );
    }

    pub fn new_toml(s : &str) -> Material
    {
        let mat : Material = toml::decode_str(s).unwrap();
        mat
    }



    /*
    pub fn new(name : &str, shader : &str) -> Material
    {
        Material {
            name : String::from_str(name),
            shader : resource::ResTT::new(shader),
            state : 0,
            texture : None
        }

    }
    */
}

impl resource::ResourceT for Material
{
    fn init(&mut self)
    {
        match self.shader {
            //TODO now
            _ => {},
            /*
            None => return,
            Some(ref mut s) => {
                s.read();
                //TODO remove
                s.utilise();
                s.uniform_set("color", &vec::Vec4::new(0.0f64, 0.5f64, 0.5f64, 1f64));
            }
            */
        }

    }

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

impl <M: Encoder<E>, E> Encodable<M, E> for Material {
  fn encode(&self, encoder: &mut M) -> Result<(), E> {
      encoder.emit_struct("Material", 1, |encoder| {
          try!(encoder.emit_struct_field( "name", 0u, |encoder| self.name.encode(encoder)));
          try!(encoder.emit_struct_field( "shader", 1u, |encoder| self.shader.encode(encoder)));
          try!(encoder.emit_struct_field( "textures", 2u, |encoder| self.textures.encode(encoder)));
          try!(encoder.emit_struct_field( "uniforms", 3u, |encoder| self.uniforms.encode(encoder)));
          Ok(())
      })
  }
}


impl<M: Decoder<E>, E> Decodable<M, E> for Material {
  fn decode(decoder: &mut M) -> Result<Material, E> {
    decoder.read_struct("root", 0, |decoder| {
         Ok(Material{
          name: try!(decoder.read_struct_field("name", 0, |decoder| Decodable::decode(decoder))),
          shader: try!(decoder.read_struct_field("shader", 0, |decoder| Decodable::decode(decoder))),
          state : 0,
          //texture: try!(decoder.read_struct_field("texture", 0, |decoder| Decodable::decode(decoder))),
          textures: try!(decoder.read_struct_field("textures", 0, |decoder| Decodable::decode(decoder))),
          uniforms: try!(decoder.read_struct_field("uniforms", 0, |decoder| Decodable::decode(decoder))),
          //uniforms: HashMap::new()
        })
    })
  }
}

