//use std::sync::Arc;
use std::collections::{DList};
use std::rc::Rc;
use std::cell::RefCell;
use libc::{c_uint, c_int};
use sync::{RWLock, Arc,RWLockReadGuard};
use std::collections::HashMap;
use std::collections::hash_map::{Occupied,Vacant};

use resource;
use shader;
use material;
use mesh;
use object;
use camera;
use matrix;
use texture;
use scene;
use mesh_render;
use fbo;
use vec;
use factory;
use context;

use mesh::BufferSend;


#[link(name = "cypher")]
extern {
    pub fn draw_callback_set(
        init_cb: extern fn(*mut Render) -> (),
        draw_cb: extern fn(*mut Render) -> (),
        resize_cb: extern fn(*mut Render, w : c_int, h : c_int) -> (),
        render: *const Render
        ) -> ();

    pub fn cgl_draw(vertex_count : c_uint) -> ();
    pub fn cgl_draw_lines(vertex_count : c_uint) -> ();
    pub fn cgl_draw_faces(buffer : *const mesh::CglBuffer, index_count : c_uint) -> ();
    pub fn cgl_draw_end() -> ();
}

pub extern fn init_cb(r : *mut Render) -> () {
    unsafe {
        return (*r).init();
    }
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
    pub material : Arc<RWLock<material::Material>>,
    pub objects : DList<Arc<RWLock<object::Object>>>,
    pub camera : Rc<RefCell<camera::Camera>>,
}

impl RenderPass
{
    pub fn new(
        material : Arc<RWLock<material::Material>>,
        camera : Rc<RefCell<camera::Camera>>) -> RenderPass
    {
        RenderPass {
                  name : String::from_str("passtest"),
                  material : material.clone(),
                  objects : DList::new(),
                  camera : camera,
              }
    }

    pub fn draw_frame(&self,
                      mesh_manager : Arc<RWLock<resource::ResourceManager<mesh::Mesh>>>,
                      shader_manager : Arc<RWLock<resource::ResourceManager<shader::Shader>>>,
                      texture_manager : Arc<RWLock<resource::ResourceManager<texture::Texture>>>,
                      fbo_manager : Arc<RWLock<resource::ResourceManager<fbo::Fbo>>>
                     ) -> ()
    {
        {
            let mut matm = self.material.write();

            for (_,t) in matm.textures.iter_mut() {
                match *t {
                    material::SamplerImageFile(ref mut img) => {
                        let yep = resource::resource_get(&mut *texture_manager.write(), img);
                        match yep.clone() {
                            None => {},
                            Some(yy) => {
                                let mut yoyo = yy.write();
                                if yoyo.state == 1 {
                                    yoyo.init();
                                }
                            }
                        }
                    },
                    _ => {} //fbo so nothing to do
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
                    yep = resource::resource_get(&mut *shader_manager.write(), s);
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
            for (name,t) in material.textures.iter_mut() {
                match *t {
                    material::SamplerImageFile(ref mut img) => {
                        let yep = resource::resource_get(&mut *texture_manager.write(), img);
                        match yep {
                            Some(yoyo) => {
                                shader.texture_set(name.as_slice(), & *yoyo.read(),i);
                                i = i +1;
                            },
                            None => {}
                        }
                    },
                    material::SamplerFbo(ref mut fbo) => {
                        let yep = resource::resource_get(&mut *fbo_manager.write(), fbo);
                        match yep {
                            Some(yoyo) => {
                                shader.texture_set(name.as_slice(), & *yoyo.read(),i);
                                i = i +1;
                            },
                            None => {}
                        }
                    },
                    //_ => {println!("todo fbo"); }
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
            Some(ref mut mr) => resource::resource_get(&mut *mesh_manager.write(), &mut mr.mesh),
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
                            //println!("while sending attributes, this mesh does not have the '{}' buffer, not rendering", name);
                            can_render = false;
                            break;
                        }
                    }
                }

                if can_render {
                    let object = ob.matrix_get();
                    let m = *matrix * object ;
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
                            None => {
                                match mb.draw_type {
                                    mesh::Lines => {
                                        let vc : uint = vertex_data_count/3;
                                        unsafe {
                                            cgl_draw_lines(vc as c_uint);
                                        }
                                    },
                                    _ => {
                                        unsafe {
                                            cgl_draw(vertex_data_count as c_uint);
                                        }
                                    }
                                }
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
    pub material_manager : Arc<RWLock<resource::ResourceManager<material::Material>>>,
    pub fbo_manager : Arc<RWLock<resource::ResourceManager<fbo::Fbo>>>,

    pub scene : Arc<RWLock<scene::Scene>>,
    pub camera : Rc<RefCell<camera::Camera>>,
    pub camera_ortho : Rc<RefCell<camera::Camera>>,

    pub fbo_all : Arc<RWLock<fbo::Fbo>>,
    pub fbo_selected : Arc<RWLock<fbo::Fbo>>,

    pub quad_outline : Arc<RWLock<object::Object>>,
    pub line : Arc<RWLock<object::Object>>,

    pub context : Rc<RefCell<context::Context>>
}

impl Render {

    pub fn new(factory: &mut factory::Factory,
               context: Rc<RefCell<context::Context>>
               ) -> Render
    {
        let scene_path = "scene/simple.scene";

        let fbo_manager = Arc::new(RWLock::new(resource::ResourceManager::new()));
        let fbo_all = fbo_manager.write().request_use_no_proc("fbo_all");
        let fbo_selected = fbo_manager.write().request_use_no_proc("fbo_selected");

        let camera = Rc::new(RefCell::new(factory.create_camera()));
        let camera_ortho = Rc::new(RefCell::new(factory.create_camera()));
        {
            let mut cam = camera_ortho.borrow_mut();
            cam.data.projection = camera::Orthographic;
            cam.object.write().position = vec::Vec3::new(0f64,0f64,10f64);
        }

        let material_manager = Arc::new(RWLock::new(resource::ResourceManager::new()));
        let shader_manager = Arc::new(RWLock::new(resource::ResourceManager::new()));


        let r = Render { 
            passes : HashMap::new(),
            mesh_manager : Arc::new(RWLock::new(resource::ResourceManager::new())),
            shader_manager : shader_manager.clone(),
            texture_manager : Arc::new(RWLock::new(resource::ResourceManager::new())),
            material_manager : material_manager.clone(),
            fbo_manager : fbo_manager.clone(),
            //scene : box scene::Scene::new_from_file(scene_path)
            scene : Arc::new(RWLock::new(scene::Scene::new_from_file(scene_path))),
            camera : camera,
            camera_ortho : camera_ortho,
            //line : Arc::new(RWLock::new(object::Object::new("line"))),
            line : Arc::new(RWLock::new(factory.create_object("line"))),
            fbo_all : fbo_all,
            fbo_selected : fbo_selected,
            //quad_outline : Arc::new(RWLock::new(object::Object::new("quad_outline")))
            quad_outline : Arc::new(RWLock::new(factory.create_object("quad_outline"))),

            context : context
        };

        {
            let m = Arc::new(RWLock::new(mesh::Mesh::new()));
            let rs = resource::ResData(m);
            let mr = resource::ResTT::new_with_res("line", rs);

            r.line.write().mesh_render =
                Some(mesh_render::MeshRender::new_with_mesh(mr, "material/line.mat"));
        }

        {
            let m = Arc::new(RWLock::new(mesh::Mesh::new()));
            m.write().add_quad(1f32, 1f32);
            let rs = resource::ResData(m);
            let mr = resource::ResTT::new_with_res("quad", rs);

            shader_manager.write().request_use_no_proc("shader/outline.sh");
            let outline_mat = material_manager.write().request_use_no_proc("material/outline.mat");
            let outline_res = resource::ResTT::new_with_res("material/outline.mat", resource::ResData(outline_mat));

            r.quad_outline.write().mesh_render = Some(
                mesh_render::MeshRender::new_with_mesh_and_mat(
                    mr,
                    outline_res));
        }

        r
    }

    pub fn init(&mut self)
    {
        self.fbo_all.write().cgl_create();
        self.fbo_selected.write().cgl_create();
    }

    fn resolution_set(&mut self, w : c_int, h : c_int)
    {
        let oc = self.quad_outline.clone();
        let render = &mut oc.write().mesh_render;
        let mesh_render_material = match *render {
            Some(ref mut mr) => &mut mr.material,
            None => return
        };

        let material = resource::resource_get(&mut *self.material_manager.write(), mesh_render_material);

        let mat = match material.clone() {
            None => return,
            Some(mat) => mat
        };

        let shaderop = &mut mat.write().shader;
        let shader = match *shaderop {
            Some(ref mut s) => s,
            None => return
        };

        match shader.resource {
            resource::ResData(ref mut ss) => {
                {ss.write().utilise();}
                ss.write().uniform_set("resolution", &vec::Vec2::new(w as f64, h as f64));
            },
            _ => {}
        }

    }

    pub fn resize(&mut self, w : c_int, h : c_int)
    {
        {
            self.quad_outline.write().scale = 
                vec::Vec3::new(w as f64, h as f64, 1f64);

            let mut cam = self.camera.borrow_mut();
            cam.resolution_set(w, h);

            let mut cam_ortho = self.camera_ortho.borrow_mut();
            cam_ortho.resolution_set(w, h);

            self.fbo_all.write().cgl_resize(w, h);
            self.fbo_selected.write().cgl_resize(w, h);
        }

        self.resolution_set(w,h);
    }

    pub fn draw_frame(&mut self) -> ()
    {
        self.prepare_passes_selected();
        self.fbo_selected.read().cgl_use();
        for p in self.passes.values()
        {
            p.draw_frame(
                self.mesh_manager.clone(),
                self.shader_manager.clone(),
                self.texture_manager.clone(),
                self.fbo_manager.clone(),
                );
        }
        fbo::Fbo::cgl_use_end();

        self.prepare_passes();

        self.fbo_all.read().cgl_use();
        for p in self.passes.values()
        {
            p.draw_frame(
                self.mesh_manager.clone(),
                self.shader_manager.clone(),
                self.texture_manager.clone(),
                self.fbo_manager.clone(),
                );
        }
        fbo::Fbo::cgl_use_end();

        //self.request_manager.handle_requests();
        //return (*self.pass).draw_frame();

        for p in self.passes.values()
        {
            p.draw_frame(
                self.mesh_manager.clone(),
                self.shader_manager.clone(),
                self.texture_manager.clone(),
                self.fbo_manager.clone(),
                );
        }

        self.prepare_passes_quad_outline();
        //TODO draw with ortho

        for p in self.passes.values()
        {
            p.draw_frame(
                self.mesh_manager.clone(),
                self.shader_manager.clone(),
                self.texture_manager.clone(),
                self.fbo_manager.clone(),
                );
        }
    }

    fn prepare_passes(&mut self)
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

    fn prepare_passes_selected(&mut self)
    {
        for (_,p) in self.passes.iter_mut()
        {
            p.objects.clear();
        }

        let context = match self.context.try_borrow() {
            Some(c) => c,
            None => {println!("cannot borrow context"); return;}
        };
        let objects = &context.selected;

        for o in objects.iter() {
            prepare_passes_object(
                o.clone(),
                &mut self.passes,
                self.material_manager.clone(),
                self.camera.clone());
        }
    }

    fn prepare_passes_quad_outline(&mut self)
    {
        for (_,p) in self.passes.iter_mut()
        {
            p.objects.clear();
        }

        prepare_passes_object(
            self.quad_outline.clone(),
            &mut self.passes,
            self.material_manager.clone(),
            self.camera_ortho.clone());
    }
}

fn prepare_passes_object(
    o : Arc<RWLock<object::Object>>,
    passes : &mut HashMap<String, Box<RenderPass>>, 
    material_manager : Arc<RWLock<resource::ResourceManager<material::Material>>>,
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

        let material = resource::resource_get(&mut *material_manager.write(), mesh_render_material);

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

