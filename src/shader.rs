extern crate libc;

use self::libc::{c_int, c_char};

#[link(name = "cypher")]
extern {
    fn shader_request_add(vert : *const c_char, frag : *const c_char, att : *const c_char) -> c_int;
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
        shader_request_add(vertcp, fragcp, attc.as_ptr());
    }
}


