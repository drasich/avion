extern crate libc;

use self::libc::{c_float};
use shader;

#[link(name = "cypher")]
extern {
    pub fn cgl_shader_uniform_float_set(
        uniform : *const shader::CglShaderUniform,
        value : c_float) -> ();

    pub fn cgl_shader_uniform_vec4_set(
        uniform : *const shader::CglShaderUniform,
        x : c_float,
        y : c_float,
        z : c_float,
        w : c_float
        ) -> ();

    pub fn cgl_shader_uniform_mat4_set(
        uniform : *const shader::CglShaderUniform,
        x : *const c_float) -> ();
}

