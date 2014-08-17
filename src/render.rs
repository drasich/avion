extern crate libc;

//use std::sync::Arc;
use std::collections::{DList,Deque};
use std::rc::Rc;
use std::cell::RefCell;

use resource;
use shader;
use mesh;
use object;

pub struct MeshManager
{
    pub mesh : Rc<RefCell<mesh::Mesh>>
}

pub struct Request<T>
{
    //pub mesh : Rc<RefCell<mesh::Mesh>>
    pub resource : Rc<RefCell<T>>
}

impl<T : resource::ResourceT> Request<T>
{
    pub fn handle(&mut self)
    {
        let mut resource = self.resource.borrow_mut();
        resource.init();
    }
}


pub struct RequestManager
{
   pub requests : DList<Box<Request<mesh::Mesh>>>
}

impl RequestManager
{
    pub fn handle_requests(&mut self)
    {
        for req in self.requests.mut_iter() {
            req.handle();
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

