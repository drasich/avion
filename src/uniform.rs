extern crate libc;

use self::libc::{c_float, c_uint};
use shader;
use matrix;
use vec;
use texture;
use fbo;

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

    pub fn cgl_shader_uniform_vec2_set(
        uniform : *const shader::CglShaderUniform,
        x : c_float,
        y : c_float,
        ) -> ();

    pub fn cgl_shader_uniform_mat4_set(
        uniform : *const shader::CglShaderUniform,
        x : *const c_float) -> ();

    pub fn cgl_shader_uniform_texture_set(
        uniform : *const shader::CglShaderUniform,
        tex : *const texture::CglTexture,
        index : c_uint
        ) -> ();

    pub fn cgl_shader_uniform_fbo_set(
        uniform : *const shader::CglShaderUniform,
        fbo : *const fbo::CglFbo,
        index : c_uint
        ) -> ();
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

impl UniformSend for vec::Vec2 {

    fn uniform_send(&self, uni : *const shader::CglShaderUniform) ->()
    {
        unsafe {
            cgl_shader_uniform_vec2_set(uni, self.x as c_float, self.y as c_float);
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

/*
impl UniformSend for texture::Texture {

    fn uniform_send(&self, uni : *const shader::CglShaderUniform) ->()
    {
        match self.cgl_texture {
            None => {},
            Some(t) => unsafe {
                //TODO just one tex for now
                cgl_shader_uniform_texture_set(uni, t);
            }
        }
    }
}
*/

pub trait TextureSend
{
    fn uniform_send(&self, uni : *const shader::CglShaderUniform, index : u32) ->();
}

impl TextureSend for texture::Texture {

    fn uniform_send(&self, uni : *const shader::CglShaderUniform, index : u32) ->()
    {
        match self.cgl_texture {
            None => {},
            Some(t) => unsafe {
                //TODO just one tex for now
                cgl_shader_uniform_texture_set(uni, t, index);
            }
        }
    }
}

impl TextureSend for fbo::Fbo {

    fn uniform_send(&self, uni : *const shader::CglShaderUniform, index : u32) ->()
    {
        match self.cgl_fbo {
            None => {},
            Some(f) => unsafe {
                //TODO just one tex for now
                cgl_shader_uniform_fbo_set(uni, f, index);
            }
        }
    }
}
