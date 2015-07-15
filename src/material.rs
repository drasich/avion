use std::collections::HashMap;
use rustc_serialize::{json, Encodable, Encoder, Decoder, Decodable};
use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::collections::hash_map::Entry::{Occupied,Vacant};
use std::path::Path;
//use std::default::Default;
//use toml;


use vec;
use shader;
//use matrix;
use resource;
//use uniform;
//use uniform::UniformSend;
use uniform::TextureSend;
use texture;
use fbo;
//#[derive(Decodable, Encodable, Default)]
//#[derive(Encodable, Default)]
use self::Sampler::{ImageFile,Fbo};

#[derive(RustcDecodable, RustcEncodable, Clone)]
pub enum Sampler
{
    ImageFile(resource::ResTT<texture::Texture>),
    Fbo(resource::ResTT<fbo::Fbo>, fbo::Attachment)
}

impl Sampler
{
    pub fn name(&self) -> &str
    {
        match *self {
            ImageFile(ref img) => {
                img.name.as_ref()
            },
            Fbo(ref f, _) => {
                f.name.as_ref()
            }
        }
    }
}

//#[derive(RustcDecodable, RustcEncodable)]
#[derive(Clone)]
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

unsafe impl Send for Material {}
unsafe impl Sync for Material {}

/*
impl Default for Material
{
    fn default() -> Material {
          Material {
            name : String::from("default"),
            shader : None,
            state : 0,
            texture : None,
            textures : LinkedList::new()
        }
    }
}
*/

impl Material
{
    pub fn new(name : &str) -> Material
    {
        Material {
            name : String::from(name),
            shader : None,
            state : 0,
            textures : HashMap::new(),
            uniforms : HashMap::new(),
        }
    }

    pub fn new_from_file(file_path : &str) -> Material
    {
        let mut file = String::new();
        File::open(&Path::new(file_path)).ok().unwrap().read_to_string(&mut file);
        let mat : Material = json::decode(file.as_ref()).unwrap();
        mat
    }

    pub fn read(&mut self)
    {
        //TODO 
        //let mut file = String::new();

        let file = {
            let path : &Path = self.name.as_ref();
            match File::open(path){
                Ok(mut f) => {
                    let mut file = String::new();
                    f.read_to_string(&mut file);
                    file
                },
                Err(e) => {
                    println!("Error reading file '{}. Error : {}", self.name, e);
                    return;
                }
            }
        };
        //let mut mat : Material = json::decode(file.as_ref()).unwrap();

        let mat : Material = match json::decode(file.as_ref()){
            Ok(m) => m,
            Err(e) => { 
                println!("{}, line {}: error reading material '{}': {:?}, creating new material",
                         file!(),
                         line!(),
                         self.name,
                         e); 
                Material::new(self.name.as_ref())
            }
        };

        self.name = mat.name.clone();
        match mat.shader {
            Some(s) => 
                self.shader = Some(resource::ResTT::new(s.name.as_ref())),
            None => self.shader = None
        }

        for (k,v) in mat.textures.iter()
        {
            match *v {
                ImageFile(ref img) => {
                    self.textures.insert(k.clone(), ImageFile(resource::ResTT::new(img.name.as_ref())));
                },
                Fbo(ref f, ref a) => {
                    self.textures.insert(k.clone(), Fbo(resource::ResTT::new(f.name.as_ref()), *a));
                }
            }
        }

        self.uniforms = mat.uniforms.clone();
    }

    pub fn save(&self)
    {
        let path : &Path = self.name.as_ref();
        let mut file = File::create(path).ok().unwrap();
        /*
        //let mut stdwriter = stdio::stdout();
        let mut encoder = json::PrettyEncoder::new(&mut file.unwrap());
        //let mut encoder = json::Encoder::new(&mut file.unwrap());
        self.encode(&mut encoder).unwrap();
        */

        let mut s = String::new();
        {
            //let mut encoder = json::PrettyEncoder::new(&mut s);
            let mut encoder = json::Encoder::new_pretty(&mut s);
            let _ = self.encode(&mut encoder);
        }

        let result = file.write(s.as_bytes());
    }

    /*
    pub fn savetoml(&self)
    {
        let s = toml::encode_str(self);
        println!("encoder toml : {} ", s );
    }

    pub fn new_toml(s : &str) -> Material
    {
        let mat : Material = toml::decode_str(s).unwrap();
        mat
    }
    */



    /*
    pub fn new(name : &str, shader : &str) -> Material
    {
        Material {
            name : String::from(name),
            shader : resource::ResTT::new(shader),
            state : 0,
            texture : None
        }

    }
    */

    pub fn set_uniform_data(&mut self, name : &str, data : shader::UniformData)
    {
        let key = name.to_string();
        let yep = match self.uniforms.entry(key){
            Vacant(entry) => entry.insert(box data),
            Occupied(entry) => {
                let entry = entry.into_mut();
                *entry = box data;
                entry
            }
        };
    }

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

impl Encodable for Material {
  fn encode<E : Encoder>(&self, encoder: &mut E) -> Result<(), E::Error> {
      encoder.emit_struct("Material", 1, |encoder| {
          try!(encoder.emit_struct_field( "name", 0usize, |encoder| self.name.encode(encoder)));
          try!(encoder.emit_struct_field( "shader", 1usize, |encoder| self.shader.encode(encoder)));
          try!(encoder.emit_struct_field( "textures", 2usize, |encoder| self.textures.encode(encoder)));
          try!(encoder.emit_struct_field( "uniforms", 3usize, |encoder| self.uniforms.encode(encoder)));
          Ok(())
      })
  }
}

impl Decodable for Material {
  fn decode<D : Decoder>(decoder: &mut D) -> Result<Material, D::Error> {
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

