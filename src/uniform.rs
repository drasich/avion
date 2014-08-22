extern crate libc;

use self::libc::{c_float};
use shader;
use matrix;
use vec;

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

pub trait UniformSend
{
    fn uniform_send(&self, uni : *const shader::CglShaderUniform) ->();
}

impl UniformSend for f32 {

    fn uniform_send(&self, uni : *const shader::CglShaderUniform) ->()
    {
        unsafe {
            cgl_shader_uniform_float_set(uni, *self);
        }
    }
}

impl UniformSend for vec::Vec4 {

    fn uniform_send(&self, uni : *const shader::CglShaderUniform) ->()
    {
        unsafe {
            cgl_shader_uniform_vec4_set(uni, self.x as f32, self.y as f32, self.z as f32, self.w as f32);
        }
    }
}

impl UniformSend for matrix::Matrix4 {

    fn uniform_send(&self, uni : *const shader::CglShaderUniform) ->()
    {
        let data = self.to_f32();
        unsafe {
            cgl_shader_uniform_mat4_set(uni, data.as_ptr());
        }
    }
}


