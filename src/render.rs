use std::sync::Arc;
use std::collections::{DList,Deque};

use shader;
use object;

pub struct Drawable;

#[link(name = "cypher")]
extern {
    pub fn draw_data_set(
        cb: extern fn() -> *const Drawable
        ) -> ();
}


pub struct RenderPass
{
    pub name : String,
    //pub material : Arc<shader::Material>,
    pub material : Box<shader::Material>,
    pub objects : DList<object::Object>
}

impl RenderPass
{
    pub fn getStuff() -> &Drawable;
}

pub struct Render
{
    pub pass : Box<RenderPass>
}

impl Render{
    pub fn init(&mut self)
    {
        shader::material_shader_init(&mut *(*self.pass).material);
    }

    pub fn draw(&self)
    {

    }
}

