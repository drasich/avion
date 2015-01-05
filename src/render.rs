use std::collections::{DList};
use std::rc::Rc;
use std::cell::RefCell;
use libc::{c_uint, c_int};
use std::sync;
use std::sync::{RWLock, Arc,RWLockReadGuard};
use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied,Vacant};
use uuid;

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
use transform;

use geometry;

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
        return (*r).draw();
    }
}

pub extern fn resize_cb(r : *mut Render, w : c_int, h : c_int) -> () {
    unsafe {
        return (*r).resize(w, h);
    }
}

struct CameraPass
{
    camera : Rc<RefCell<camera::Camera>>,
    objects : DList<Arc<RWLock<object::Object>>>,
}

impl CameraPass
{
    fn new(camera : Rc<RefCell<camera::Camera>>) -> CameraPass
    {
        CameraPass {
            camera : camera,
            objects : DList::new()
        }
    }

    fn add_object(&mut self, o : Arc<RWLock<object::Object>>)
    {
        self.objects.push_back(o);
    }
}

struct RenderPass
{
    pub name : String,
    pub material : Arc<sync::RWLock<material::Material>>,
    //pub objects : DList<Arc<RWLock<object::Object>>>,
    //pub camera : Rc<RefCell<camera::Camera>>,
    pub passes : HashMap<uuid::Uuid, Box<CameraPass>>,
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
                  //objects : DList::new(),
                  //camera : camera,
                  passes : HashMap::new()
              }
    }

    pub fn draw_frame(
        &self,
        mesh_manager : Arc<RWLock<resource::ResourceManager<mesh::Mesh>>>,
        shader_manager : Arc<RWLock<resource::ResourceManager<shader::Shader>>>,
        texture_manager : Arc<RWLock<resource::ResourceManager<texture::Texture>>>,
        fbo_manager : Arc<RWLock<resource::ResourceManager<fbo::Fbo>>>
        ) -> ()
    {
        {
            let mut matm = self.material.write().unwrap();
            //println!("material : {}", matm.name);

            for (_,t) in matm.textures.iter_mut() {
                match *t {
                    material::Sampler::ImageFile(ref mut img) => {
                        let yep = resource::resource_get(&mut *texture_manager.write().unwrap(), img);
                        match yep.clone() {
                            None => {},
                            Some(yy) => {
                                let mut yoyo = yy.write().unwrap();
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
            let mut matm = self.material.write().unwrap();

            let shaderres = &mut matm.shader;
            match *shaderres  {
                None => {println!("shaderes none");},
                Some(ref mut s) => {
                    yep = resource::resource_get(&mut *shader_manager.write().unwrap(), s);
                    match yep.clone() {
                        None => {println!("no shader yet {}", s.name);},
                        Some(yy) => {
                            let mut yoyo = yy.write().unwrap();
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
                cr = c.read().unwrap();
                shader = & *cr;
            }
        }

        shader.utilise();
        {
            //TODO for shader textures, for uniform textures instead of 'for material
            //tex/uniforms'?
            let mut material = self.material.write().unwrap();

            let mut i = 0u32;
            for (name,t) in material.textures.iter_mut() {
                match *t {
                    material::Sampler::ImageFile(ref mut img) => {
                        let yep = resource::resource_get(&mut *texture_manager.write().unwrap(), img);
                        match yep {
                            Some(yoyo) => {
                                shader.texture_set(name.as_slice(), & *yoyo.read().unwrap(),i);
                                i = i +1;
                            },
                            None => {}
                        }
                    },
                    material::Sampler::Fbo(ref mut fbo) => {
                        let yep = resource::resource_get(&mut *fbo_manager.write().unwrap(), fbo);
                        match yep {
                            Some(yoyo) => {
                                shader.texture_set(name.as_slice(), & *yoyo.read().unwrap(),i);
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

        for (_,p) in self.passes.iter() {
            let cam_mat = p.camera.borrow().object.read().unwrap().get_world_matrix();
            let cam_projection = p.camera.borrow().perspective_get();
            let cam_mat_inv = cam_mat.inverse_get();
            let matrix = &cam_projection * &cam_mat_inv;

            for o in p.objects.iter() {
                let mut ob = o.write().unwrap();
                self.draw_object(shader, &mut *ob, &matrix, mesh_manager.clone());
            }
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
            Some(ref mut mr) => 
                resource::resource_get(&mut *mesh_manager.write().unwrap(), &mut mr.mesh),
            None => {
                println!("no mesh render");
                return;
            }
        };

        //TODO chris
        match themesh  {
            None => return,
            Some(ref m) => {
                let mut mb = m.write().unwrap();
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
                            //println!("while sending attributes, this mesh does 
                            //not have the '{}' buffer, not rendering", name);
                            can_render = false;
                            break;
                        }
                    }
                }

                if can_render {
                    let object = ob.get_world_matrix();
                    let m = matrix * &object ;
                    shader.uniform_set("matrix", &m);

                    match mb.buffer_u32_get("faces") {
                        //Some(ref bind) =>
                        Some(bind) => unsafe {
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
                                mesh::DrawType::Lines => {
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

    pub context : Rc<RefCell<context::Context>>,

    pub grid : Arc<RWLock<object::Object>>,
    pub camera_repere : Arc<RWLock<object::Object>>,

    pub dragger : Arc<RWLock<object::Object>>,
}

impl Render {

    //TODO remove dragger and put "view_objects"
    //TODO dont create the scene here
    pub fn new(factory: &mut factory::Factory,
               context: Rc<RefCell<context::Context>>,
               dragger : Arc<RWLock<object::Object>>,
               ) -> Render
    {
        let scene_path = "scene/simple.scene";

        let fbo_manager = Arc::new(RWLock::new(resource::ResourceManager::new()));
        let fbo_all = fbo_manager.write().unwrap().request_use_no_proc("fbo_all");
        let fbo_selected = fbo_manager.write().unwrap().request_use_no_proc("fbo_selected");

        let camera = Rc::new(RefCell::new(factory.create_camera()));
        {
            let mut cam = camera.borrow_mut();
            cam.pan(&vec::Vec3::new(100f64,20f64,100f64));
            cam.lookat(vec::Vec3::new(0f64,5f64,0f64));
        }
        let camera_ortho = Rc::new(RefCell::new(factory.create_camera()));
        {
            let mut cam = camera_ortho.borrow_mut();
            cam.data.projection = camera::Projection::Orthographic;
            cam.pan(&vec::Vec3::new(0f64,0f64,50f64));
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
            fbo_all : fbo_all,
            fbo_selected : fbo_selected,
            //quad_outline : Arc::new(RWLock::new(object::Object::new("quad_outline")))
            quad_outline : Arc::new(RWLock::new(factory.create_object("quad_outline"))),

            context : context,

            grid : Arc::new(RWLock::new(factory.create_object("grid"))),
            camera_repere : Arc::new(RWLock::new(
                    factory.create_object("camera_repere"))),

            dragger : dragger// Arc::new(RWLock::new(
                    //factory.create_object("dragger"))),
        };

        {
            let m = Arc::new(RWLock::new(mesh::Mesh::new()));
            create_grid(&mut *m.write().unwrap(), 100i32, 10i32);
            let rs = resource::ResTest::ResData(m);
            let mr = resource::ResTT::new_with_res("grid", rs);

            r.grid.write().unwrap().mesh_render =
                Some(mesh_render::MeshRender::new_with_mesh(mr, "material/line.mat"));
        }

        {
            let m = Arc::new(RWLock::new(mesh::Mesh::new()));
            create_repere(&mut *m.write().unwrap(), 40f64 );
            let rs = resource::ResTest::ResData(m);
            let mr = resource::ResTT::new_with_res("repere", rs);

            r.camera_repere.write().unwrap().mesh_render =
                Some(mesh_render::MeshRender::new_with_mesh(mr, "material/line.mat"));
        }

        {
            let m = Arc::new(RWLock::new(mesh::Mesh::new()));
            m.write().unwrap().add_quad(1f32, 1f32);
            let rs = resource::ResTest::ResData(m);
            let mr = resource::ResTT::new_with_res("quad", rs);

            shader_manager.write().unwrap().request_use_no_proc("shader/outline.sh");
            let outline_mat = material_manager.write().unwrap().request_use_no_proc("material/outline.mat");
            let outline_res = resource::ResTT::new_with_res("material/outline.mat", resource::ResTest::ResData(outline_mat));

            r.quad_outline.write().unwrap().mesh_render = Some(
                mesh_render::MeshRender::new_with_mesh_and_mat(
                    mr,
                    outline_res));
        }

        r
    }

    fn resolution_set(&mut self, w : c_int, h : c_int)
    {
        self.quad_outline.clone().read().unwrap().set_uniform_data(
            "resolution",
            shader::UniformData::Vec2(vec::Vec2::new(w as f64, h as f64)));
    }

    fn prepare_passes(&mut self)
    {
        for (_,p) in self.passes.iter_mut()
        {
            //p.objects.clear();
            p.passes.clear();
        }

        let objects = &self.scene.read().unwrap().objects;
        //self.passes.clear();
        for o in objects.iter() {
            prepare_passes_object(
                o.clone(),
                &mut self.passes,
                self.material_manager.clone(),
                self.camera.clone());
        }

        prepare_passes_object(
            self.grid.clone(),
            &mut self.passes,
            self.material_manager.clone(),
            self.camera.clone());

        let m = 40f64;
        self.camera_repere.write().unwrap().position = 
            vec::Vec3::new(
                -self.camera_ortho.borrow().data.width/2f64 +m, 
                -self.camera_ortho.borrow().data.height/2f64 +m, 
                -10f64);
        self.camera_repere.write().unwrap().orientation = 
            self.camera.borrow().object.read().unwrap().orientation.inverse();

        prepare_passes_object(
            self.camera_repere.clone(),
            &mut self.passes,
            self.material_manager.clone(),
            self.camera_ortho.clone());

        let sel_len = match self.context.try_borrow() {
            Some(c) => c.selected.len(),
            None => {println!("cannot borrow context"); 0}
        };

        if sel_len > 0 {
            prepare_passes_object(
                self.dragger.clone(),
                &mut self.passes,
                self.material_manager.clone(),
                self.camera.clone());
        }
    }

    fn prepare_passes_selected(&mut self)
    {
        for (_,p) in self.passes.iter_mut()
        {
            //p.objects.clear();
            p.passes.clear();
        }

        let context = match self.context.try_borrow() {
            Some(c) => c,
            None => {println!("cannot borrow context"); return;}
        };

        let objects = &context.selected;

        let mut center = vec::Vec3::zero();
        let mut ori = vec::Quat::identity();
        for o in objects.iter() {
            center = center + o.read().unwrap().position;
            ori = ori * o.read().unwrap().world_orientation();
            prepare_passes_object(
                o.clone(),
                &mut self.passes,
                self.material_manager.clone(),
                self.camera.clone());
        }

        if objects.len() > 0 {
            center = center / (objects.len() as f64);
            self.dragger.write().unwrap().position = center;
            self.dragger.write().unwrap().orientation = transform::Orientation::Quat(ori);
        }
    }

    fn prepare_passes_quad_outline(&mut self)
    {
        for (_,p) in self.passes.iter_mut()
        {
            //p.objects.clear();
            p.passes.clear();
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
        let occ = o.clone();
        for c in occ.read().unwrap().children.iter()
        {
            prepare_passes_object(c.clone(), passes, material_manager.clone(), camera.clone());
        }
    }

    {
        let oc = o.clone();
        let render = &mut oc.write().unwrap().mesh_render;
        let mesh_render_material = match *render {
            Some(ref mut mr) => &mut mr.material,
            None => return
        };

        let material = resource::resource_get(
            &mut *material_manager.write().unwrap(),
            mesh_render_material);

        let mat = match material {
            None => return,
            Some(mat) => mat
        };

        {
            let rp = match passes.entry(mesh_render_material.name.clone()) {
                Vacant(entry) => 
                    entry.set(box RenderPass::new(mat.clone(), camera.clone())),
                Occupied(entry) => entry.into_mut(),
            };

            //rp.objects.push_back(o.clone());

            let cam_pass = match rp.passes.entry(camera.borrow().id.clone()) {
                Vacant(entry) => 
                    entry.set(box CameraPass::new(camera.clone())),
                Occupied(entry) => entry.into_mut(),
            };

            cam_pass.add_object(o.clone());
        }
    }

    //done at the beginning
    /*
    {
        let occ = o.clone();
        for c in occ.read().children.iter()
        {
            println!("prepare child : {}", c.read().name);
            prepare_passes_object(c.clone(), passes, material_manager.clone(), camera.clone());
        }
    }
    */
}

pub trait Renderer
{
    fn init(&mut self);
    fn draw(&mut self);
    fn resize(&mut self, w : c_int, h : c_int);
}

impl Renderer for Render
{
    fn init(&mut self)
    {
        self.fbo_all.write().unwrap().cgl_create();
        self.fbo_selected.write().unwrap().cgl_create();
    }

    fn resize(&mut self, w : c_int, h : c_int)
    {
        {
            self.quad_outline.write().unwrap().scale = 
                vec::Vec3::new(w as f64, h as f64, 1f64);

            let mut cam = self.camera.borrow_mut();
            cam.resolution_set(w, h);

            let mut cam_ortho = self.camera_ortho.borrow_mut();
            cam_ortho.resolution_set(w, h);

            self.fbo_all.write().unwrap().cgl_resize(w, h);
            self.fbo_selected.write().unwrap().cgl_resize(w, h);
        }

        self.resolution_set(w,h);
    }

    fn draw(&mut self) -> ()
    {
        self.prepare_passes_selected();
        self.fbo_selected.read().unwrap().cgl_use();
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

        self.fbo_all.read().unwrap().cgl_use();
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
}

fn create_grid(m : &mut mesh::Mesh, num : i32, space : i32)
{
    //TODO make something better then using add_line
    //ie create the vec and then add the buffer

    let color = vec::Vec4::new(1f64,1f64,1f64,0.1f64);
    let xc = vec::Vec4::new(1.0f64,0.247f64,0.188f64,0.4f64);
    let zc = vec::Vec4::new(0f64,0.4745f64,1f64,0.4f64);

    for i in  range(-num, num) {
        let p1 = vec::Vec3::new((i*space) as f64, 0f64, (-space*num) as f64);
        let p2 = vec::Vec3::new((i*space) as f64, 0f64, (space*num) as f64);
        let s = geometry::Segment::new(p1,p2);
        if i == 0 {
            m.add_line(s, zc);
        }
        else {
            m.add_line(s, color);
        }
    }

    for i in  range(-num, num) {
        let p1 = vec::Vec3::new((-space*num) as f64, 0f64, (i*space) as f64);
        let p2 = vec::Vec3::new((space*num) as f64, 0f64, (i*space) as f64);
        let s = geometry::Segment::new(p1,p2);
        if i == 0 {
            m.add_line(s, xc);
        }
        else {
            m.add_line(s, color);
        }
    }
}

fn create_repere(m : &mut mesh::Mesh, len : f64)
{
    let red = vec::Vec4::new(1.0f64,0.247f64,0.188f64,1f64);
    let green = vec::Vec4::new(0.2117f64,0.949f64,0.4156f64,1f64);
    let blue = vec::Vec4::new(0f64,0.4745f64,1f64,1f64);

    let s = geometry::Segment::new(
        vec::Vec3::zero(), vec::Vec3::new(len, 0f64, 0f64));
    m.add_line(s, red);

    let s = geometry::Segment::new(
        vec::Vec3::zero(), vec::Vec3::new(0f64, len, 0f64));
    m.add_line(s, green);

    let s = geometry::Segment::new(
        vec::Vec3::zero(), vec::Vec3::new(0f64, 0f64, len));
    m.add_line(s, blue);
}

