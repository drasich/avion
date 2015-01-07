extern crate libc;

use self::libc::{c_float, c_uint, c_int};
use shader;
use matrix;
use vec;
use texture;
use fbo;

#[link(name = "cypher")]
extern {
    pub fn cgl_shader_uniform_int_set(
        uniform : *const shader::CglShaderUniform,
        value : c_int) -> ();

    pub fn cgl_shader_uniform_float_set(
        uniform : *const shader::CglShaderUniform,
        value : c_float) -> ();

    pub fn cgl_shader_uniform_vec2_set(
        uniform : *const shader::CglShaderUniform,
        x : c_float,
        y : c_float,
        ) -> ();

    pub fn cgl_shader_uniform_vec3_set(
        uniform : *const shader::CglShaderUniform,
        x : c_float,
        y : c_float,
        z : c_float,
        ) -> ();

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

    pub fn cgl_shader_uniform_texture_set(
        uniform : *const shader::CglShaderUniform,
        tex : *const texture::CglTexture,
        index : c_uint
        ) -> ();

    pub fn cgl_shader_uniform_fbo_depth_set(
        uniform : *const shader::CglShaderUniform,
        fbo : *const fbo::CglFbo,
        index : c_uint
        ) -> ();

    pub fn cgl_shader_uniform_fbo_color_set(
        uniform : *const shader::CglShaderUniform,
        fbo : *const fbo::CglFbo,
        index : c_uint
        ) -> ();
}

pub trait UniformSend
{
    fn uniform_send(&self, uni : *const shader::CglShaderUniform) ->();
}

impl UniformSend for i32 {

    fn uniform_send(&self, uni : *const shader::CglShaderUniform) ->()
    {
        unsafe {
            cgl_shader_uniform_int_set(uni, *self);
        }
    }
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

impl UniformSend for vec::Vec3 {

    fn uniform_send(&self, uni : *const shader::CglShaderUniform) ->()
    {
        unsafe {
            cgl_shader_uniform_vec3_set(
                uni,
                self.x as c_float,
                self.y as c_float,
                self.z as c_float);
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
                cgl_shader_uniform_texture_set(uni, t, index);
            }
        }
    }
}

pub struct FboSampler<'a>
{
    pub fbo : &'a fbo::Fbo,
    pub attachment :  fbo::Attachment
}

impl<'a> TextureSend for FboSampler<'a> {

    fn uniform_send(&self, uni : *const shader::CglShaderUniform, index : u32) ->()
    {
        let f = match self.fbo.cgl_fbo {
            None => return,
            Some(f) => f
        };

        match self.attachment {
            fbo::Attachment::Depth => unsafe {
                cgl_shader_uniform_fbo_depth_set(uni, f, index);
            },
            fbo::Attachment::Color => unsafe {
                cgl_shader_uniform_fbo_color_set(uni, f, index);
            }
        }
    }
}

