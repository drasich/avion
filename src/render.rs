extern crate libc;

use std::sync::Arc;
use std::ptr;
use std::mem;
use std::collections::{DList,Deque};
use self::libc::{c_void};
use std::rc::Rc;

use shader;
use mesh;
use object;

pub struct Drawable
{
    shader: *const shader::Shader,
    buffer: *const mesh::Buffer
}

pub struct Request
{
    data : Rc<Vec<f32>>,
    mesh : Rc<mesh::Mesh>
}

#[link(name = "cypher")]
extern {
    pub fn draw_data_set(
        cb: extern fn(*const Render) -> *const Drawable,
        render: *const Render
        ) -> ();

    pub fn draw_callback_set(
        cb: extern fn(*const Render) -> (),
        render: *const Render
        ) -> ();

    pub fn shader_draw(shader : *const shader::Shader, buffer : *const mesh::Buffer) -> ();
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

    pub fn draw_frame(&self) -> ()
    {
        println!("draw frame");

        if (*(self.material)).shader == None {
            return;
        }

        let s : *const shader::Shader;
        match (*self.material).shader  {
            None => return,
            Some(sh) => s = sh
        }

        for o in self.objects.iter() {
            match  o.mesh  {
                None => continue,
                Some(ref m) => {
                    match m.buffer {
                        None => { continue;}
                        Some(b) => unsafe { shader_draw(s,b); }
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

/*
pub struct RequestManager<'rm>
{
    pub requests : DList<Request<'rm>>
}


impl<'rm> RequestManager<'rm>
{
    pub fn add_req(&mut self, mesh : &'rm mut mesh::Mesh, data : Rc<Vec<f32>>)
    {
        mesh.state = 1;
        self.requests.push( Request{mesh : mesh, data: data.clone()});
    }

    pub fn add_req(&mut self, mesh : &'rm mut mesh::Mesh, data : &'rm [f32])
    {
        mesh.state = 1;
        self.requests.push( Request{mesh : mesh, data: data});
    }
}
*/


impl Render {
    pub fn init(&mut self)
    {
        shader::material_shader_init(&mut *(*self.pass).material);

        //let pass = & *self.pass;
        //pass.init(self);

        //(*self.pass).init();
        //for o in self.objects.mut_iter() {
        //for o in pass.objects.iter() {
        for o in (*self.pass).objects.mut_iter() {
            match (*o).mesh {
                None => continue,
                Some(ref mut m) => if m.state == 0 {
                    mesh::mesh_buffer_init(m);
                //self.draw();
                }
            }
        }
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

    pub fn draw_frame(&self) -> ()
    {
        println!("render draw frame");
        return (*self.pass).draw_frame();
    }

}

