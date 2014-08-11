use std::sync::Arc;
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
        cb: extern fn() -> *const Drawable
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

    /*
    pub fn getStuff(&self) -> Drawable
    {
        match self.drawables.front() {
            Some(d) => ,
            None
        }
        return Drawable {
    }
    */

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

        //println!("tet");
        self.drawables.clear();
        for o in self.objects.iter() {
            match  o.mesh  {
                None => continue,
                Some(ref m) => {
                    //println!("there is a mesh {}", m.name);
                    match m.buffer {
                        None => { continue;}//println!("but no buffer");continue},
                        Some(b) => println!("tu viens jamais?") //self.drawables.push( Drawable{ shader: s, buffer :b})
                    }
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
    }

    pub fn draw(&mut self)
    {
        loop {
            (*self.pass).draw();
        }
    }

}

