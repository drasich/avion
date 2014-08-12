use std::sync::Arc;
use std::ptr;
use std::mem;
use std::collections::{DList,Deque};

use shader;
use mesh;
use object;

pub struct Drawable
{
    shader: *const shader::Shader,
    buffer: *const mesh::Buffer
}

#[link(name = "cypher")]
extern {
    pub fn draw_data_set(
        cb: extern fn(*const Render) -> *const Drawable,
        render: *const Render
        ) -> ();
}


pub struct RenderPass
{
    pub name : String,
    //pub material : Arc<shader::Material>,
    pub material : Box<shader::Material>,
    pub objects : DList<object::Object>,
    pub drawables : DList<Drawable>
}

impl RenderPass
{
    pub fn canDraw(&self) -> bool
    {
        return !self.drawables.is_empty();
    }

    pub fn getDrawable(&self) -> *const Drawable
    {
        match self.drawables.front() {
            Some(d) => unsafe {return mem::transmute(&d)},
            None => return ptr::null()
        }
    }

    pub fn draw(&mut self)
    {
        if (*(self.material)).shader == None {
            return;
        }

        let s : *const shader::Shader;
        match (*self.material).shader  {
            None => return,
            Some(sh) => s = sh
        }

        if !self.drawables.is_empty() {
            return; //for now... TODO
        }

        self.drawables.clear();
        for o in self.objects.iter() {
            match  o.mesh  {
                None => continue,
                Some(ref m) => {
                    match m.buffer {
                        None => { continue;}
                        Some(b) => self.drawables.push( Drawable{ shader: s, buffer :b})
                    }
                }
            }
        }

    }

    pub fn init(&mut self)
    {
        for o in self.objects.mut_iter() {
            match (*o).mesh {
                None => continue,
                Some(ref mut m) => if m.state == 0 {
                    mesh::mesh_buffer_init(m);
                }
            }
        }
    }
}

pub struct Render
{
    pub pass : Box<RenderPass>
}

impl Render{
    pub fn init(&mut self)
    {
        shader::material_shader_init(&mut *(*self.pass).material);

        (*self.pass).init();
    }

    pub fn draw(&mut self)
    {
        loop {
            (*self.pass).draw();
        }
    }

    pub fn getDrawable(&self) -> *const Drawable
    {
        return (*self.pass).getDrawable();
    }

}

