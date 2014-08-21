extern crate libc;
use resource;
use uniform;
use std::collections::HashMap;
use std::io::File;
use std::io::BufferedReader;
use std::io::File;
use std::uint;

use self::libc::{c_char, c_uint};
pub struct CglShader;
pub struct CglShaderAttribute;
pub struct CglShaderUniform;

pub struct Shader
{
    pub cgl_shader : *const CglShader,
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
            //TODO remove
            cgl_shader_use(self.cgl_shader);
            uniform::cgl_shader_uniform_vec4_set(cgl_uni, 1.0f32, 1.0f32, 0.5f32, 1.0f32);
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

        //let contents = File::open(&Path::new("shader/simple.frag")).read_to_string();
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

        let att_name = String::from_str("position");
        let attc = att_name.to_c_str();

        let uni_name = String::from_str("color");
        let unic = uni_name.to_c_str();


        unsafe {
            let shader = cgl_shader_init_string(vertcp, fragcp);

            self.shader = Some(Shader {
                cgl_shader : shader,
                attributes : HashMap::new(),
                uniforms : HashMap::new()
            });

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
                if split[2] == "float" && split[3] == "4" {
                    //TODO
                }
            }
        }

    }
}

