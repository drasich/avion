extern crate libc;
use resource;
use uniform;
use std::collections::HashMap;
use std::io::File;
use std::io::BufferedReader;
use std::io::File;
use std::uint;
use vec;
use matrix;
use uniform;
use uniform::UniformSend;

use self::libc::{c_char, c_uint};
pub struct CglShader;
pub struct CglShaderAttribute;
pub struct CglShaderUniform;

pub struct Shader
{
    cgl_shader : *const CglShader, //TODO private
    pub attributes : HashMap<String, *const CglShaderAttribute>,
    pub uniforms : HashMap<String, *const CglShaderUniform>
}

impl Shader
{
    fn attribute_add(&mut self, name : &str, size : u32)
    {
        let attc = name.to_c_str();

        unsafe {
            let cgl_att = cgl_shader_attribute_new(self.cgl_shader, attc.as_ptr(), size);
            self.attributes.insert(String::from_str(name), cgl_att);
        }
    }

    fn uniform_add(&mut self, name : &str)
    {
        let unic = name.to_c_str();

        unsafe {
            let cgl_uni = cgl_shader_uniform_new(self.cgl_shader, unic.as_ptr());
            self.uniforms.insert(String::from_str(name), cgl_uni);
        }
    }

    fn uniform_set(&self, name : &str, value : &UniformSend)
    {
        match self.uniforms.find(&String::from_str(name)) {
            Some(uni) => value.uniform_send(*uni),
            None => println!("could not find such uniform '{}'",name)
        }
    }

    pub fn utilise(&self)
    {
        unsafe {
            cgl_shader_use(self.cgl_shader);
        }
    }

    pub fn new(cgl_shader : *const CglShader) -> Shader
    {
        Shader {
            cgl_shader : cgl_shader,
            attributes : HashMap::new(),
            uniforms : HashMap::new()
        }
    }

}

pub struct Material
{
    pub name : String,
    pub shader: Option<Shader>,
    pub state : i32
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

impl Material
{
    fn read(&mut self, vertpath : &str, fragpath : &str)
    {
        if self.state == 11 {
            return
        }

        let contents = File::open(&Path::new(fragpath)).read_to_string();
        let frag : String;

        match contents {
            Ok(r) => frag = r,
            _ => return
        }

        let contents = File::open(&Path::new(vertpath)).read_to_string();
        let vert : String;

        match contents {
            Ok(r) => vert = r,
            _ => return
        }

        let vertc = vert.to_c_str();
        let vertcp = vertc.as_ptr();
        let fragc = frag.to_c_str();
        let fragcp = fragc.as_ptr();

        unsafe {
            let shader = cgl_shader_init_string(vertcp, fragcp);
            self.shader = Some(Shader::new(shader));
        }

        self.state = 11;
    }
}

impl resource::ResourceT for Material
{
    fn init(&mut self)
    {
        let path = Path::new("shader/simple.sh");
        let mut file = BufferedReader::new(File::open(&path));

        let mut frag : String;
        let mut vert : String;

        match file.read_line() {
            Ok(l) => { vert = l; vert.pop_char(); },
            Err(_) => return
        }
 
        match file.read_line() {
            Ok(l) => { frag = l; frag.pop_char();},
            Err(_) => return
        }

        self.read(vert.as_slice(), frag.as_slice());

        let shader : &mut Shader;

        match self.shader {
            Some(ref mut s) => shader = s,
            None => return
        }

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
                shader.attribute_add(split[1], size);
            }
            else if split[0] == "uni" {
                shader.uniform_add(split[1]);
                println!("it's an uniform {} yoo", split[1]);
                if split[2] == "vec4" {
                    //TODO
                }
            }
        }

        //TODO remove
        shader.utilise();
        shader.uniform_set("color", &vec::Vec4::new(0.0f64, 0.5f64, 0.5f64, 1f64));
        let camera = matrix::Matrix4::identity();
        let object = matrix::Matrix4::translation(vec::Vec3::new(0f64, 0f64, -20f64));
        let projection = matrix::Matrix4::perspective(0.4f64,1f64,1f64,1000f64);
        let m = projection * camera.inverse_get() * object ;
        //shader.uniform_set("matrix", &matrix::Matrix4::identity());
        shader.uniform_set("matrix", &m);

    }
}

