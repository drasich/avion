extern crate libc;
use resource;

use self::libc::{c_char};
pub struct CglShader;

pub struct Material
{
    pub name : String,
    pub shader: Option<*const CglShader>,
    pub state : i32
}

#[link(name = "cypher")]
extern {
    fn cgl_shader_init_string(
        vert : *const c_char,
        frat : *const c_char,
        att : *const c_char) -> *const CglShader;

    pub fn cgl_shader_use(shader : *const CglShader);
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

        let attc = "position".to_c_str();
        self.shader = Some(cgl_shader_init_string(vertcp, fragcp, attc.as_ptr()));
    }


            self.state = 11;
        }
    }
}


