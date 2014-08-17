extern crate libc;

//use std::sync::Arc;
use std::collections::{DList,Deque};
use std::rc::Rc;
use std::cell::RefCell;

use shader;
use mesh;
use object;

pub struct MeshManager
{
    pub mesh : Rc<RefCell<mesh::Mesh>>
}

pub struct Request
{
    pub mesh : Rc<RefCell<mesh::Mesh>>
}

impl Request
{
    pub fn handle(&mut self)
    {
        let mut mesh = self.mesh.borrow_mut();
        //mesh.name = String::from_str("newnewnew");
        //mesh::mesh_buffer_init(&mut *mesh);
        println!(" I am handling request of {} ", mesh.name);
        mesh.init();
    }
}

pub struct RequestManager
{
    pub requests : DList<Box<Request>>
}

impl RequestManager
{
    pub fn handle_requests(&mut self)
    {
        match self.requests.front_mut(){
            None => (),
            Some(req) => req.handle()
        }

        self.requests.clear();
    }

}

#[link(name = "cypher")]
extern {
    pub fn draw_callback_set(
        cb: extern fn(*mut Render) -> (),
        render: *const Render
        ) -> ();

    pub fn shader_draw(shader : *const shader::Shader, buffer : *const mesh::Buffer) -> ();
}

pub extern fn draw_cb(r : *mut Render) -> () {
    unsafe {
        return (*r).draw_frame();
    }
}


pub struct RenderPass
{
    pub name : String,
    //pub material : Arc<shader::Material>,
    pub material : Box<shader::Material>,
    pub objects : DList<object::Object>,
}

impl RenderPass
{
    pub fn init(&mut self)
    {
        /*
        for o in self.objects.mut_iter() {
            match (*o).mesh {
                None => continue,
                Some(ref mut m) => if m.state == 0 {
                    mesh::mesh_buffer_init(m);
                }
            }
        }
        */
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
                    match m.borrow().buffer {
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
    pub pass : Box<RenderPass>,
    pub request_manager : Box<RequestManager>
        
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
    }

    pub fn draw(&mut self)
    {
    }

    pub fn draw_frame(&mut self) -> ()
    {
        println!("render draw frame");
        self.request_manager.handle_requests();
        return (*self.pass).draw_frame();
    }

}

