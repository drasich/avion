extern crate libc;
use resource;
use uniform;
use std::collections::HashMap;

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

impl resource::ResourceT for Material
{
    fn init(&mut self)
    {
        if self.state != 11 {

    let vert  = 
"
attribute vec2 position;
void main(void)
{
//gl_Position = matrix * vec4(vertex, 1.0);
//gl_Position = vec4(0, 0, 0, 1.0);
gl_Position = vec4(position, 0.0, 1.0);
}
";

    let frag = 
"
uniform vec4 color;
void main (void)
{
//gl_FragColor = vec4(0.3, 0.3, 0.4, 1.0);
//gl_FragColor = vec4(red, 0.3, 0.4, 1.0);
gl_FragColor = color;
}
";

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

        let cgl_att = cgl_shader_attribute_new(shader, attc.as_ptr(), 2);
        let cgl_uni = cgl_shader_uniform_new(shader, unic.as_ptr());
        cgl_shader_use(shader);
        //uniform::cgl_shader_uniform_float_set(cgl_uni, 0.5f32);
        uniform::cgl_shader_uniform_vec4_set(cgl_uni, 0.0f32, 1.0f32, 0.5f32, 1.0f32);

        match self.shader {
            Some(ref mut s) => {
                let b = s.attributes.insert(att_name, cgl_att);
            }
            None => ()
        }

    }

            self.state = 11;
        }
    }
}


