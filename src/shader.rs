extern crate libc;

use self::libc::{c_int, c_char};
pub struct Shader;

#[link(name = "cypher")]
extern {
    fn shader_request_add(
        vert : *const c_char,
        frag : *const c_char,
        att : *const c_char,
        //cb: extern fn(i32)) -> c_int;
        cb: extern fn(*mut Material, i32, *const Shader)) -> c_int;
}

extern fn callback(a:i32) {
    println!("I am called from C with value {}", a);
}

pub struct Material
{
    shader: *mut Shader,
    state : i32
}

extern fn callback_result(material : *mut Material, answer_code :i32, shader : *const Shader) {
    println!("I am called from C with value {}", answer_code);
}


pub fn shader_init() -> ()
{
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
        //let id = shader_request_add(vertcp, fragcp, attc.as_ptr());
        //shader_request_add(vertcp, fragcp, attc.as_ptr(), callback);
        shader_request_add(vertcp, fragcp, attc.as_ptr(), callback_result);
    }
}


