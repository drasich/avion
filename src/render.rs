
//use std::sync::Arc;
use std::collections::{DList,Deque};
use std::rc::Rc;
use std::cell::RefCell;
use libc::{c_char,c_uint};
use sync::{RWLock, Arc};

use resource;
use shader;
use mesh;
use object;
use camera;
use matrix;

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
    pub fn cgl_draw_faces(buffer : *const mesh::CglBuffer, index_count : c_uint) -> ();
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
    //pub objects : DList<object::Object>,
    pub objects : DList<Rc<RefCell<object::Object>>>,
    pub camera : Rc<RefCell<camera::Camera>>,
    pub resource_manager : Arc<RWLock<resource::ResourceManager>>
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

        let cam_mat = self.camera.borrow().object.borrow().matrix_get();
        let cam_projection = self.camera.borrow().perspective_get();
        let cam_mat_inv = cam_mat.inverse_get();
        let matrix = cam_projection * cam_mat_inv;

        for o in self.objects.iter() {
            let mut ob = o.borrow_mut();
            //RenderPass::draw_object(shader, &*ob, &matrix);
            self.draw_object(shader, &mut *ob, &matrix);
        }
    }

    fn draw_object(
        &self,
        shader : &shader::Shader,
        ob : &mut object::Object,
        matrix : &matrix::Matrix4)
    {
        let mut themesh : Option<Arc<RWLock<mesh::Mesh>>> = None;
        match ob.mesh.resource{
            resource::ResNone => {
                //rc_mesh = self.resource_manager.borrow_mut().request_use(r.name.as_slice());
                //ob.mesht.resource = self.resource_manager.borrow_mut().request_use(ob.mesht.name.as_slice());
                println!("resource is none I request it");
                ob.mesh.resource = self.resource_manager.write().request_use(ob.mesh.name.as_slice());
            },
            resource::ResData(ref data) => {
                //println!("now I have some data!!! and I can use it !!!");
                themesh = Some(data.clone());
            },
            resource::ResWait => {
                ob.mesh.resource = self.resource_manager.write().request_use(ob.mesh.name.as_slice());
                println!("now I have to wait");
            }
        }

        //TODO chris
        match  themesh  {
            None => { println!("no mesh wait "); return;},
            Some(ref m) => {
                let mut mb = m.write();
                if mb.state == 1 {
                    println!("init buffers");
                    mb.init_buffers();
                }
                let mut can_render = true;
                let mut vertex_data_count = 0;
                for (name, cgl_att) in shader.attributes.iter() {
                    let cgl_buf = mb.buffers.find(name);
                    match cgl_buf {
                        Some(ref cb) => {
                            cb.utilise(*cgl_att);
                            if name.as_slice() == "position" {
                                vertex_data_count = cb.size_get();
                            }
                        },
                        None => {
                            println!("while sending attributes, this mesh does not have the '{}' buffer, not rendering", name);
                            can_render = false;
                            break;
                        }
                    }
                }

                if can_render {
                    let object = ob.matrix_get();
                    let m = matrix * object ;
                    shader.uniform_set("matrix", &m);

                    match mb.buffers.find(&String::from_str("faces")) {
                        Some(ref bind) =>
                            unsafe{
                                match bind.buffer_get() {
                                    Some(b) => {
                                        let faces_data_count = bind.size_get();
                                        cgl_draw_faces(b, faces_data_count as c_uint);
                                    },
                                    None => ()
                                }
                            },
                            None => unsafe {
                                cgl_draw(vertex_data_count as c_uint);
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

