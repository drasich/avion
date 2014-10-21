//use std::sync::Arc;
use std::collections::{DList};
use std::rc::Rc;
use std::cell::RefCell;
use libc::{c_uint, c_int};
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
use mesh_render;

use mesh::BufferSend;

#[link(name = "cypher")]
extern {
    pub fn draw_callback_set(
        draw_cb: extern fn(*mut Render) -> (),
        resize_cb: extern fn(*mut Render, w : c_int, h : c_int) -> (),
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

pub extern fn resize_cb(r : *mut Render, w : c_int, h : c_int) -> () {
    unsafe {
        return (*r).resize(w, h);
    }
}



pub struct RenderPass
{
    pub name : String,
    pub material : Arc<RWLock<shader::Material>>,
    pub objects : DList<Arc<RWLock<object::Object>>>,
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
            res.resource = manager.request_use(res.name.as_slice());
        },
        resource::ResData(ref data) => {
            the_res = Some(data.clone());
        },
        resource::ResWait => {
            res.resource = manager.request_use(res.name.as_slice());
        }
    }

    the_res
}

impl RenderPass
{
    pub fn new(
        material : Arc<RWLock<shader::Material>>,
        camera : Rc<RefCell<camera::Camera>>) -> RenderPass
    {
        //TODO 
        //let cam = camera::Camera::new();

        RenderPass {
                  name : String::from_str("passtest"),
                  material : material.clone(),
                  objects : DList::new(),
                  //camera : Rc::new(RefCell::new(cam)),
                  camera : camera,
              }
    }

    pub fn draw_frame(&self,
                      mesh_manager : Arc<RWLock<resource::ResourceManager<mesh::Mesh>>>,
                      shader_manager : Arc<RWLock<resource::ResourceManager<shader::Shader>>>,
                      texture_manager : Arc<RWLock<resource::ResourceManager<texture::Texture>>>
                     ) -> ()
    {
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
                        None => {println!("no shader yet {}", s.name);},
                        Some(yy) => {
                            let mut yoyo = yy.write();
                            if yoyo.state == 0 {
                                yoyo.read();
                                println!("find shader!!! {}", s.name);
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
                println!("shader not yet available");
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

        let cam_mat = self.camera.borrow().object.read().matrix_get();
        let cam_projection = self.camera.borrow().perspective_get();
        let cam_mat_inv = cam_mat.inverse_get();
        let matrix = cam_projection * cam_mat_inv;

        for o in self.objects.iter() {
            let mut ob = o.write();
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
            None => return,
            Some(ref m) => {
                let mut mb = m.write();
                if mb.state == 1 {
                    println!("init buffers");
                    mb.init_buffers();
                }
                let mut can_render = true;
                let mut vertex_data_count = 0;
                for (name, cgl_att) in shader.attributes.iter() {
                    match mb.buffer_get(name.as_slice()){
                        Some(ref cb) => {
                            cb.utilise(*cgl_att);
                            if name.as_slice() == "position" {
                                vertex_data_count = cb.size_get();
                            }
                            continue;
                        },
                        None => ()
                    }

                    match mb.buffer_f32_get(name.as_slice()){
                        Some(ref cb) => {
                            cb.utilise(*cgl_att);
                            if name.as_slice() == "position" {
                                vertex_data_count = cb.size_get();
                            }
                            continue;
                        },
                        None => (),
                    }

                    match mb.buffer_u32_get(name.as_slice()){
                        Some(ref cb) => {
                            cb.utilise(*cgl_att);
                            if name.as_slice() == "position" {
                                vertex_data_count = cb.size_get();
                            }
                            continue;
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

                    match mb.buffer_u32_get("faces") {
                        //Some(ref bind) =>
                        Some(bind) =>
                            unsafe{
                                match (**bind).cgl_buffer_get() {
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
    pub passes : HashMap<String, Box<RenderPass>>, //TODO check
    pub mesh_manager : Arc<RWLock<resource::ResourceManager<mesh::Mesh>>>,
    pub shader_manager : Arc<RWLock<resource::ResourceManager<shader::Shader>>>,
    pub texture_manager : Arc<RWLock<resource::ResourceManager<texture::Texture>>>,
    pub material_manager : Arc<RWLock<resource::ResourceManager<shader::Material>>>,
    pub scene : Arc<RWLock<scene::Scene>>,
    pub camera : Rc<RefCell<camera::Camera>>,
    pub line : Arc<RWLock<object::Object>>,
}

impl Render {

    pub fn new() -> Render
    {
        let scene_path = "scene/simple.scene";
        let mut r = Render { 
            passes : HashMap::new(),
            mesh_manager : Arc::new(RWLock::new(resource::ResourceManager::new())),
            shader_manager : Arc::new(RWLock::new(resource::ResourceManager::new())),
            texture_manager : Arc::new(RWLock::new(resource::ResourceManager::new())),
            material_manager : Arc::new(RWLock::new(resource::ResourceManager::new())),
            //scene : box scene::Scene::new_from_file(scene_path)
            scene : Arc::new(RWLock::new(scene::Scene::new_from_file(scene_path))),
            camera : Rc::new(RefCell::new(camera::Camera::new())),
            line : Arc::new(RWLock::new(object::Object::new("line"))),
        };

        let m = Arc::new(RWLock::new(mesh::Mesh::new()));
        let rs = resource::ResData(m);
        let mr = resource::ResTT::new_with_res("yep", rs);

        r.line.write().mesh_render = Some(mesh_render::MeshRender::new_with_mesh(mr, "material/line.mat"));

        r
    }

    /*
    pub fn init(&mut self)
    {
    }
    */

    pub fn resize(&mut self, w : c_int, h : c_int)
    {
        println!("resize !!!! {}, {}", w, h);
        let mut cam = self.camera.borrow_mut();

        cam.resolution_set(w, h);
    }

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

        let objects = &self.scene.read().objects;
        //self.passes.clear();
        for o in objects.iter() {
            prepare_passes_object(
                o.clone(),
                &mut self.passes,
                self.material_manager.clone(),
                self.camera.clone());
        }

        prepare_passes_object(
            self.line.clone(),
            &mut self.passes,
            self.material_manager.clone(),
            self.camera.clone());
    }
}

fn prepare_passes_object(
    o : Arc<RWLock<object::Object>>,
    passes : &mut HashMap<String, Box<RenderPass>>, 
    material_manager : Arc<RWLock<resource::ResourceManager<shader::Material>>>,
    camera : Rc<RefCell<camera::Camera>>
    )
{
    {
        let oc = o.clone();
        let render = &mut oc.write().mesh_render;
        let mesh_render_material = match *render {
            Some(ref mut mr) => &mut mr.material,
            None => return
        };

        let material = resource_get(&mut *material_manager.write(), mesh_render_material);

        let mat = match material.clone() {
            None => return,
            Some(mat) => mat
        };

        {
            let rp = match passes.entry(mesh_render_material.name.clone()) {
                Vacant(entry) => entry.set(box RenderPass::new(mat.clone(), camera.clone())),
                Occupied(entry) => entry.into_mut(),
            };

            rp.objects.push(o.clone());
        }
    }

    {
        let occ = o.clone();
        for c in occ.read().children.iter()
        {
            prepare_passes_object(c.clone(), passes, material_manager.clone(), camera.clone());
        }
    }
}

