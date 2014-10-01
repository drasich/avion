
//use std::sync::Arc;
use std::collections::{DList,Deque};
use std::rc::Rc;
use std::cell::RefCell;
use libc::{c_char,c_uint};
use sync::{RWLock, Arc,RWLockReadGuard};
use std::collections::HashMap;
use std::collections::hashmap::{Occupied,Vacant};

use resource;
use shader;
use mesh;
use object;
use camera;
use matrix;
use texture;
use scene;

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
        for req in self.requests.iter_mut() {
            req.handle();
        }

        self.requests.clear();

        for req in self.requests_material.iter_mut() {
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
    pub fn cgl_draw_end() -> ();
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
    //pub material : Rc<RefCell<shader::Material>>,
    pub material : Arc<RWLock<shader::Material>>,
    //pub objects : DList<object::Object>,
    pub objects : DList<Rc<RefCell<object::Object>>>,
    pub camera : Rc<RefCell<camera::Camera>>,
}

fn resource_get<T:'static+resource::Create+Send+Sync>(
    manager : &mut resource::ResourceManager<T>,
    res: &mut resource::ResTT<T>) 
    -> Option<Arc<RWLock<T>>>
{
    let mut the_res : Option<Arc<RWLock<T>>> = None;
    match res.resource{
        resource::ResNone => {
            //rc_mesh = self.mesh_manager.borrow_mut().request_use(r.name.as_slice());
            //ob.mesht.resource = self.mesh_manager.borrow_mut().request_use(ob.mesht.name.as_slice());
            println!("resource is none I request it");
            res.resource = manager.request_use(res.name.as_slice());
        },
        resource::ResData(ref data) => {
            //println!("now I have some data!!! and I can use it !!!");
            the_res = Some(data.clone());
        },
        resource::ResWait => {
            res.resource = manager.request_use(res.name.as_slice());
            println!("now I have to wait");
        }
    }

    the_res
}

impl RenderPass
{
    pub fn new(material : Arc<RWLock<shader::Material>>) -> RenderPass
    {
        //TODO 
        let cam = camera::Camera::new();

        RenderPass {
                  name : String::from_str("passtest"),
                  material : material.clone(),
                  objects : DList::new(),
                  camera : Rc::new(RefCell::new(cam)),
              }
    }

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

    pub fn draw_frame(&self,
                      mesh_manager : Arc<RWLock<resource::ResourceManager<mesh::Mesh>>>,
                      shader_manager : Arc<RWLock<resource::ResourceManager<shader::Shader>>>,
                      texture_manager : Arc<RWLock<resource::ResourceManager<texture::Texture>>>
                     ) -> ()
    {
        println!("draw frame");
        {
            let mut matm = self.material.write();

            for t in matm.textures.iter_mut() {
                let yep = resource_get(&mut *texture_manager.write(), t);
                match yep.clone() {
                    None => {},
                    Some(yy) => {
                        let mut yoyo = yy.write();
                        if yoyo.state == 1 {
                            yoyo.init();
                        }
                    }
                }
            }
        }

        //let shader : &shader::Shader;
        let mut yep : Option<Arc<RWLock<shader::Shader>>> = None;

        {
            //let mut matm = self.material.borrow_mut();
            let mut matm = self.material.write();

            let shaderres = &mut matm.shader;
            match *shaderres  {
                None => {},
                Some(ref mut s) => {
                    yep = resource_get(&mut *shader_manager.write(), s);
                    match yep.clone() {
                        None => {},
                        Some(yy) => {
                            let mut yoyo = yy.write();
                            if yoyo.state == 0 {
                                yoyo.read();
                            }
                        }
                    }
                }
            }
        }

        let c : Arc<RWLock<shader::Shader>>;
        let cr : RWLockReadGuard<shader::Shader>;

        let shader : &shader::Shader;
        match yep
        {
            None => {
                println!("something wrong with the shader ");
                return;},
            Some(ref sh) => {
                c = sh.clone();
                cr = c.read();
                shader = & *cr;
            }
        }

        shader.utilise();
        {
            //TODO for shader textures, for uniform textures instead of 'for material
            //tex/uniforms'?
            let mut material = self.material.write();

            let mut i = 0u32;
            for t in material.textures.iter_mut() {
                let yep = resource_get(&mut *texture_manager.write(), t);
                match yep {
                    Some(yoyo) => {
                        shader.texture_set("texture", & *yoyo.read(),i);
                        i = i +1;
                    },
                    None => {}
                }
            }

            for (k,v) in material.uniforms.iter() {
                shader.uniform_set(k.as_slice(), &(**v));
            }
        }

        let cam_mat = self.camera.borrow().object.borrow().matrix_get();
        let cam_projection = self.camera.borrow().perspective_get();
        let cam_mat_inv = cam_mat.inverse_get();
        let matrix = cam_projection * cam_mat_inv;

        for o in self.objects.iter() {
            let mut ob = o.borrow_mut();
            //RenderPass::draw_object(shader, &*ob, &matrix);
            self.draw_object(shader, &mut *ob, &matrix, mesh_manager.clone());
        }
    }

    fn draw_object(
        &self,
        shader : &shader::Shader,
        ob : &mut object::Object,
        matrix : &matrix::Matrix4,
        mesh_manager : Arc<RWLock<resource::ResourceManager<mesh::Mesh>>>
        )
    {
        let themesh = match ob.mesh_render {
            Some(ref mut mr) => resource_get(&mut *mesh_manager.write(), &mut mr.mesh),
            None => {
                println!("no mesh render");
                return;
            }
        };

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
                                        cgl_draw_end();
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
    //pub pass : Box<RenderPass>,
    pub passes : HashMap<String, Box<RenderPass>>, //TODO check
    //pub passes : DList<Box<RenderPass>>,
    //TODO remove request manager?
    pub request_manager : Box<RequestManager>,

    pub mesh_manager : Arc<RWLock<resource::ResourceManager<mesh::Mesh>>>,
    pub shader_manager : Arc<RWLock<resource::ResourceManager<shader::Shader>>>,
    pub texture_manager : Arc<RWLock<resource::ResourceManager<texture::Texture>>>,
    pub material_manager : Arc<RWLock<resource::ResourceManager<shader::Material>>>,
    pub scene : Box<scene::Scene>,
}

impl Render {

    /*
    pub fn new() -> Render
    {
        Render { 
            pass : box render::RenderPass{
                      name : String::from_str("passtest"),
                      material : mat.clone(),
                      objects : DList::new(),
                      camera : Rc::new(RefCell::new(cam)),
                      mesh_manager : Arc::new(RWLock::new(resource::ResourceManager::new()))
                  },
            request_manager : box render::RequestManager {
                                  requests : DList::new(),
                                  requests_material : DList::new()
                              }
        }
    }
    */

    pub fn init(&mut self)
    {
    }

    /*
    pub fn draw(&mut self)
    {
    }
    */

    pub fn draw_frame(&mut self) -> ()
    {
        self.prepare_passes();
        //self.request_manager.handle_requests();
        //return (*self.pass).draw_frame();
        for p in self.passes.values()
        {
            p.draw_frame(
                self.mesh_manager.clone(),
                self.shader_manager.clone(),
                self.texture_manager.clone(),
                        );
        }
    }

    pub fn prepare_passes(&mut self)
    {
        for (_,p) in self.passes.iter_mut()
        {
            p.objects.clear();
        }

        //self.passes.clear();
        for o in self.scene.objects.iter_mut() {
            let oc = o.clone();
            let render = &mut oc.borrow_mut().mesh_render;
            let mesh_render_material = match *render {
                Some(ref mut mr) => &mut mr.material,
                None => continue
            };

            let material = resource_get(&mut *self.material_manager.write(), mesh_render_material);

            let mat = match material.clone() {
                None => continue,
                Some(mat) => mat
            };

            let rp = match self.passes.entry(mesh_render_material.name.clone()) {
                Vacant(entry) => entry.set(box RenderPass::new(mat.clone())),
                Occupied(entry) => entry.into_mut(),
            };

            rp.objects.push(o.clone());
        }
    }

}

