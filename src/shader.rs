extern crate libc;
use resource;
use std::collections::HashMap;

use self::libc::{c_char, c_uint};
pub struct CglShader;
pub struct CglShaderAttribute;

pub struct Shader
{
    pub cgl_shader : *const CglShader,
    pub attributes : HashMap<String, *const CglShaderAttribute>
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

    pub fn cgl_shader_attribute_get(
        shader : *const CglShader,
        name : *const c_char,
        size : c_uint) -> *const CglShaderAttribute;
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
void main (void)
{
gl_FragColor = vec4(0.3, 0.3, 0.4, 1.0);
}
";

    unsafe {
        let vertc = vert.to_c_str();
        let vertcp = vertc.as_ptr();
        let fragc = frag.to_c_str();
        let fragcp = fragc.as_ptr();

        let shader = cgl_shader_init_string(vertcp, fragcp);

        let att_name = String::from_str("position");
        let attc = att_name.to_c_str();

        self.shader = Some(Shader { cgl_shader : shader, attributes : HashMap::new()});

        let cgl_att = cgl_shader_attribute_get(shader, attc.as_ptr(), 2);

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


