extern crate libc;

//use std::sync::Arc;
use std::collections::{DList,Deque};
use std::rc::Rc;
use std::cell::RefCell;
use self::libc::{c_char,c_uint};

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
    //pub resource : Rc<RefCell<resource::ResourceS>>
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
   pub requests : DList<Box<Request<mesh::Mesh>>>,
   pub requests_material : DList<Box<Request<shader::Material>>>
   //pub requests : DList<Box<Request>>
}

impl RequestManager
{
    pub fn handle_requests(&mut self)
    {
        for req in self.requests.mut_iter() {
            req.handle();
        }

        self.requests.clear();

        for req in self.requests_material.mut_iter() {
            req.handle();
        }

        self.requests_material.clear();

    }

}

#[link(name = "cypher")]
extern {
    pub fn draw_callback_set(
        cb: extern fn(*mut Render) -> (),
        render: *const Render
        ) -> ();

    pub fn cgl_draw(vertex_count : c_uint) -> ();
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
    //pub material : Box<shader::Material>,
    pub material : Rc<RefCell<shader::Material>>,
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

        let material = self.material.borrow();
        let shader : &shader::Shader;

        match (*material).shader  {
            None => return,
            Some(ref sh) => shader = sh
        }

        shader.utilise();

        for o in self.objects.iter() {
            match  o.mesh  {
                None => continue,
                Some(ref m) => {
                    let mb = m.borrow();
                    let mut can_render = true;
                    for (name, cgl_att) in shader.attributes.iter() {
                        let cgl_buf = mb.buffers.find(name);
                        match cgl_buf {
                            Some(ref cb) => cb.utilise(*cgl_att),
                            None => {
                                println!("while sending attributes, this mesh does not have the '{}' buffer, not rendering", name);
                                can_render = false;
                                break;
                            }
                        }
                    }

                    if can_render {
                        //TODO if has indices
                        unsafe {
                            cgl_draw(mb.vertex.len() as c_uint);
                        }
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
    }

    pub fn draw(&mut self)
    {
    }

    pub fn draw_frame(&mut self) -> ()
    {
        self.request_manager.handle_requests();
        return (*self.pass).draw_frame();
    }

}

