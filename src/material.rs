use std::collections::HashMap;
use serialize::{json, Encodable, Encoder, Decoder, Decodable};
use std::io::File;
use std::io::BufferedReader;
//use std::default::Default;
use toml;


use vec;
use shader;
//use matrix;
use resource;
//use uniform;
//use uniform::UniformSend;
//use uniform::TextureSend;
use texture;
use fbo;
//#[deriving(Decodable, Encodable, Default)]
//#[deriving(Encodable, Default)]

#[deriving(Decodable, Encodable)]
pub enum Sampler
{
    SamplerImageFile(resource::ResTT<texture::Texture>),
    SamplerFbo(resource::ResTT<fbo::Fbo>)
}

impl Sampler
{
    pub fn name(&self) -> &str
    {
        match *self {
            SamplerImageFile(ref img) => {
                img.name.as_slice()
            },
            SamplerFbo(ref f) => {
                f.name.as_slice()
            }
        }
    }
}

pub struct Material
{
    pub name : String,
    pub shader: Option<resource::ResTT<shader::Shader>>,
    pub state : i32,
    //pub textures : HashMap<String, resource::ResTT<texture::Texture>>,
    pub textures : HashMap<String, Sampler>,
    //pub uniforms : HashMap<String, Box<UniformSend+'static>>,
    pub uniforms : HashMap<String, Box<shader::UniformData>>,
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

impl Material
{
    pub fn new(name : &str) -> Material
    {
        Material {
            name : String::from_str(name),
            shader : None,
            state : 0,
            textures : HashMap::new(),
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

        for (k,v) in mat.textures.iter()
        {
            match *v {
                SamplerImageFile(ref img) => {
                    self.textures.insert(k.clone(), SamplerImageFile(resource::ResTT::new(img.name.as_slice())));
                },
                SamplerFbo(ref f) => {
                    self.textures.insert(k.clone(), SamplerFbo(resource::ResTT::new(f.name.as_slice())));
                }
            }
        }

        /*
        for t in mat.textures.iter()
        {
            self.textures.push(resource::ResTT::new(t.name.as_slice()));
        }
        */
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
