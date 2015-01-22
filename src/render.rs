use std::collections::{DList};
use std::rc::Rc;
use std::cell::RefCell;
use libc::{c_uint, c_int};
use std::sync;
use std::sync::{RwLock, Arc, RwLockReadGuard};
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
use mesh_render;
use fbo;
use vec;
use factory;
use transform;
use uniform;

use geometry;

use mesh::BufferSend;

#[link(name = "cypher")]
extern {
    /*
    pub fn draw_callback_set(
        init_cb: extern fn(*mut Render) -> (),
        draw_cb: extern fn(*mut Render) -> (),
        resize_cb: extern fn(*mut Render, w : c_int, h : c_int) -> (),
        render: *const Render
        ) -> ();
        */

    pub fn cgl_draw(vertex_count : c_uint) -> ();
    pub fn cgl_draw_lines(vertex_count : c_uint) -> ();
    pub fn cgl_draw_faces(buffer : *const mesh::CglBuffer, index_count : c_uint) -> ();
    pub fn cgl_draw_end() -> ();

    pub fn cgl_clear() -> ();
}

/*
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
*/

struct CameraPass
{
    camera : Rc<RefCell<camera::Camera>>,
    objects : DList<Arc<RwLock<object::Object>>>,
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

    fn add_object(&mut self, o : Arc<RwLock<object::Object>>)
    {
        self.objects.push_back(o);
    }
}

struct RenderPass
{
    pub name : String,
    //pub material : Arc<sync::RwLock<material::Material>>,
    pub shader : Arc<sync::RwLock<shader::Shader>>,
    //pub objects : DList<Arc<RwLock<object::Object>>>,
    //pub camera : Rc<RefCell<camera::Camera>>,
    pub passes : HashMap<uuid::Uuid, Box<CameraPass>>,
}

impl RenderPass
{
    pub fn new(
        shader : Arc<RwLock<shader::Shader>>,
        camera : Rc<RefCell<camera::Camera>>) -> RenderPass
    {
        RenderPass {
                  name : String::from_str("passtest"),
                  shader : shader.clone(),
                  //objects : DList::new(),
                  //camera : camera,
                  passes : HashMap::new()
              }
    }

    pub fn draw_frame(
        &self,
        mesh_manager : Arc<RwLock<resource::ResourceManager<mesh::Mesh>>>,
        material_manager : Arc<RwLock<resource::ResourceManager<material::Material>>>,
        shader_manager : Arc<RwLock<resource::ResourceManager<shader::Shader>>>,
        texture_manager : Arc<RwLock<resource::ResourceManager<texture::Texture>>>,
        fbo_manager : Arc<RwLock<resource::ResourceManager<fbo::Fbo>>>
        ) -> ()
    {
        let shader = &mut *self.shader.write().unwrap();

        if shader.state == 0 {
            shader.read();
        }

        shader.utilise();

        for (_,p) in self.passes.iter() {
            let cam_mat = p.camera.borrow().object.read().unwrap().get_world_matrix();
            let cam_projection = p.camera.borrow().get_perspective();
            let cam_mat_inv = cam_mat.get_inverse();
            let matrix = &cam_projection * &cam_mat_inv;

            for o in p.objects.iter() {
                let mut ob = o.write().unwrap();
                self.draw_object(
                    shader,
                    &mut *ob,
                    &matrix, 
                    mesh_manager.clone(),
                    material_manager.clone(),
                    texture_manager.clone(),
                    fbo_manager.clone()
                    );
            }
        }

    }

    fn draw_object(
        &self,
        shader : &shader::Shader,
        ob : &mut object::Object,
        matrix : &matrix::Matrix4,
        mesh_manager : Arc<RwLock<resource::ResourceManager<mesh::Mesh>>>,
        material_manager : Arc<RwLock<resource::ResourceManager<material::Material>>>,
        texture_manager : Arc<RwLock<resource::ResourceManager<texture::Texture>>>,
        fbo_manager : Arc<RwLock<resource::ResourceManager<fbo::Fbo>>>
        )
    {

        let (themesh, the_mat) = match ob.mesh_render {
            Some(ref mut mr) => { 
                let mmm = match mr.material.resource {
                    resource::ResTest::ResData(ref rd) => Some(rd.clone()),
                    _ =>
                        resource::resource_get(&mut *material_manager.write().unwrap(), &mut mr.material)
                };

                (resource::resource_get(&mut *mesh_manager.write().unwrap(), &mut mr.mesh),
                mmm)
            },
            None => return
        };

        if let Some(ref mat) = the_mat {
            let mut material = mat.write().unwrap();

            for (_,t) in material.textures.iter_mut() {
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
                    material::Sampler::Fbo(ref mut fbo, ref attachment) => {
                        let yep = resource::resource_get(&mut *fbo_manager.write().unwrap(), fbo);
                        match yep {
                            Some(yoyo) => {
                                let fc = yoyo.clone();
                                let fff = fc.read().unwrap();
                                let fbosamp = uniform::FboSampler { 
                                    fbo : & *fff,
                                    attachment : *attachment
                                };
                                //shader.texture_set(name.as_slice(), & *yoyo.read().unwrap(),i);
                                shader.texture_set(name.as_slice(), &fbosamp,i);
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

        //TODO chris
        match themesh  {
            None => return,
            Some(ref m) => {
                let mut mb = m.write().unwrap();
                if mb.state == 1 {
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
                                    let vc : usize = vertex_data_count/3;
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

    pub mesh_manager : Arc<RwLock<resource::ResourceManager<mesh::Mesh>>>,
    pub shader_manager : Arc<RwLock<resource::ResourceManager<shader::Shader>>>,
    pub texture_manager : Arc<RwLock<resource::ResourceManager<texture::Texture>>>,
    pub material_manager : Arc<RwLock<resource::ResourceManager<material::Material>>>,
    pub fbo_manager : Arc<RwLock<resource::ResourceManager<fbo::Fbo>>>,

    pub camera : Rc<RefCell<camera::Camera>>,
    pub camera_ortho : Rc<RefCell<camera::Camera>>,

    pub fbo_all : Arc<RwLock<fbo::Fbo>>,
    pub fbo_selected : Arc<RwLock<fbo::Fbo>>,

    pub quad_outline : Arc<RwLock<object::Object>>,
    pub quad_all : Arc<RwLock<object::Object>>,

    pub grid : Arc<RwLock<object::Object>>,
    pub camera_repere : Arc<RwLock<object::Object>>,

    pub line : Arc<RwLock<object::Object>>

    //pub dragger : Arc<RwLock<object::Object>>,
}

impl Render {

    //TODO remove dragger and put "view_objects"
    pub fn new(factory: &mut factory::Factory,
               camera : Rc<RefCell<camera::Camera>>,
               //dragger : Arc<RwLock<object::Object>>,
               ) -> Render
    {
        let fbo_manager = Arc::new(RwLock::new(resource::ResourceManager::new()));
        let fbo_all = fbo_manager.write().unwrap().request_use_no_proc("fbo_all");
        let fbo_selected = fbo_manager.write().unwrap().request_use_no_proc("fbo_selected");

        let camera_ortho = Rc::new(RefCell::new(factory.create_camera()));
        {
            let mut cam = camera_ortho.borrow_mut();
            cam.data.projection = camera::Projection::Orthographic;
            cam.pan(&vec::Vec3::new(0f64,0f64,50f64));
        }

        let material_manager = Arc::new(RwLock::new(resource::ResourceManager::new()));
        let shader_manager = Arc::new(RwLock::new(resource::ResourceManager::new()));


        let r = Render { 
            passes : HashMap::new(),
            mesh_manager : Arc::new(RwLock::new(resource::ResourceManager::new())),
            shader_manager : shader_manager.clone(),
            texture_manager : Arc::new(RwLock::new(resource::ResourceManager::new())),
            material_manager : material_manager.clone(),
            fbo_manager : fbo_manager.clone(),
            camera : camera,
            camera_ortho : camera_ortho,
            fbo_all : fbo_all,
            fbo_selected : fbo_selected,
            quad_outline : Arc::new(RwLock::new(factory.create_object("quad_outline"))),
            quad_all : Arc::new(RwLock::new(factory.create_object("quad_all"))),


            grid : Arc::new(RwLock::new(factory.create_object("grid"))),
            camera_repere : Arc::new(RwLock::new(
                    factory.create_object("camera_repere"))),

            //dragger : dragger// Arc::new(RwLock::new(
                    //factory.create_object("dragger"))),
            line : Arc::new(RwLock::new(factory.create_object("line"))),
        };

        {
            let m = Arc::new(RwLock::new(mesh::Mesh::new()));
            create_grid(&mut *m.write().unwrap(), 100i32, 10i32);
            let rs = resource::ResTest::ResData(m);
            let mr = resource::ResTT::new_with_res("grid", rs);

            r.grid.write().unwrap().mesh_render =
                Some(mesh_render::MeshRender::new_with_mesh(mr, "material/line.mat"));
        }

        {
            let m = Arc::new(RwLock::new(mesh::Mesh::new()));
            create_repere(&mut *m.write().unwrap(), 40f64 );
            let rs = resource::ResTest::ResData(m);
            let mr = resource::ResTT::new_with_res("repere", rs);

            r.camera_repere.write().unwrap().mesh_render =
                Some(mesh_render::MeshRender::new_with_mesh(mr, "material/line.mat"));
        }

        {
            let m = Arc::new(RwLock::new(mesh::Mesh::new()));
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

        {
            let m = Arc::new(RwLock::new(mesh::Mesh::new()));
            m.write().unwrap().add_quad(1f32, 1f32);
            let rs = resource::ResTest::ResData(m);
            let mr = resource::ResTT::new_with_res("quad", rs);

            //shader_manager.write().unwrap().request_use_no_proc("shader/all.sh");
            let all_mat = material_manager.write().unwrap().request_use_no_proc("material/fbo_all.mat");
            let all_res = resource::ResTT::new_with_res("material/fbo_all.mat", resource::ResTest::ResData(all_mat));

            r.quad_all.write().unwrap().mesh_render = Some(
                mesh_render::MeshRender::new_with_mesh_and_mat(
                    mr,
                    all_res));
        }

        {
            let m = Arc::new(RwLock::new(mesh::Mesh::new()));
            let rs = resource::ResTest::ResData(m);
            let mr = resource::ResTT::new_with_res("line", rs);

            r.line.write().unwrap().mesh_render =
                Some(mesh_render::MeshRender::new_with_mesh(mr, "material/line.mat"));
        }

        r
    }

    pub fn init(&mut self)
    {
        self.fbo_all.write().unwrap().cgl_create();
        self.fbo_selected.write().unwrap().cgl_create();
    }

    pub fn resize(&mut self, w : c_int, h : c_int)
    {
        {
            self.quad_outline.write().unwrap().scale = 
                vec::Vec3::new(w as f64, h as f64, 1f64);

            self.quad_all.write().unwrap().scale = 
                vec::Vec3::new(w as f64, h as f64, 1f64);

            let mut cam = self.camera.borrow_mut();
            cam.set_resolution(w, h);

            let mut cam_ortho = self.camera_ortho.borrow_mut();
            cam_ortho.set_resolution(w, h);

            self.fbo_all.write().unwrap().cgl_resize(w, h);
            self.fbo_selected.write().unwrap().cgl_resize(w, h);
        }

        self.resolution_set(w,h);
    }


    fn resolution_set(&mut self, w : c_int, h : c_int)
    {
        self.quad_outline.clone().read().unwrap().set_uniform_data(
            "resolution",
            shader::UniformData::Vec2(vec::Vec2::new(w as f64, h as f64)));

        /*
        self.quad_all.clone().read().unwrap().set_uniform_data(
            "resolution",
            shader::UniformData::Vec2(vec::Vec2::new(w as f64, h as f64)));
            */
    }

    fn prepare_passes(&mut self, objects : &DList<Arc<RwLock<object::Object>>>)
    {
        for (_,p) in self.passes.iter_mut()
        {
            //p.objects.clear();
            p.passes.clear();
        }

        //self.passes.clear();
        for o in objects.iter() {
            prepare_passes_object(
                o.clone(),
                &mut self.passes,
                self.material_manager.clone(),
                self.shader_manager.clone(),
                self.camera.clone());
        }

        prepare_passes_object(
            self.grid.clone(),
            &mut self.passes,
            self.material_manager.clone(),
            self.shader_manager.clone(),
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
                self.shader_manager.clone(),
            self.camera_ortho.clone());
    }

    fn prepare_passes_selected(
        &mut self,
        objects : &DList<Arc<RwLock<object::Object>>>)
    {
        for (_,p) in self.passes.iter_mut()
        {
            p.passes.clear();
        }

        let mut center = vec::Vec3::zero();
        let mut ori = vec::Quat::identity();
        for o in objects.iter() {
            center = center + o.read().unwrap().position;
            ori = ori * o.read().unwrap().world_orientation();
            prepare_passes_object(
                o.clone(),
                &mut self.passes,
                self.material_manager.clone(),
                self.shader_manager.clone(),
                self.camera.clone());
        }
    }

    fn prepare_passes_objects_ortho(&mut self, list : DList<Arc<RwLock<object::Object>>>)
    {
        for (_,p) in self.passes.iter_mut()
        {
            //p.objects.clear();
            p.passes.clear();
        }

        for o in list.iter() {

        prepare_passes_object(
            o.clone(),
            &mut self.passes,
            self.material_manager.clone(),
            self.shader_manager.clone(),
            self.camera_ortho.clone());
        }
    }

    fn prepare_passes_objects_per(&mut self, list : &DList<Arc<RwLock<object::Object>>>)
    {
        for (_,p) in self.passes.iter_mut()
        {
            //p.objects.clear();
            p.passes.clear();
        }

        for o in list.iter() {

        prepare_passes_object(
            o.clone(),
            &mut self.passes,
            self.material_manager.clone(),
            self.shader_manager.clone(),
            self.camera.clone());
        }
    }

    pub fn draw(
        &mut self,
        objects : &DList<Arc<RwLock<object::Object>>>,
        selected : &DList<Arc<RwLock<object::Object>>>,
        draggers : &DList<Arc<RwLock<object::Object>>>,
        ) -> ()
    {
        self.prepare_passes_selected(selected);
        self.fbo_selected.read().unwrap().cgl_use();
        for p in self.passes.values()
        {
            p.draw_frame(
                self.mesh_manager.clone(),
                self.material_manager.clone(),
                self.shader_manager.clone(),
                self.texture_manager.clone(),
                self.fbo_manager.clone(),
                );
        }
        fbo::Fbo::cgl_use_end();

        self.prepare_passes(objects);

        self.fbo_all.read().unwrap().cgl_use();
        for p in self.passes.values()
        {
            p.draw_frame(
                self.mesh_manager.clone(),
                self.material_manager.clone(),
                self.shader_manager.clone(),
                self.texture_manager.clone(),
                self.fbo_manager.clone(),
                );
        }
        fbo::Fbo::cgl_use_end();

        /*
        for p in self.passes.values()
        {
            p.draw_frame(
                self.mesh_manager.clone(),
                self.material_manager.clone(),
                self.shader_manager.clone(),
                self.texture_manager.clone(),
                self.fbo_manager.clone(),
                );
        }
        */


        //*
        let mut l = DList::new();
        l.push_back(self.quad_all.clone());
        self.prepare_passes_objects_ortho(l);

        for p in self.passes.values()
        {
            p.draw_frame(
                self.mesh_manager.clone(),
                self.material_manager.clone(),
                self.shader_manager.clone(),
                self.texture_manager.clone(),
                self.fbo_manager.clone(),
                );
        }
        //*/

        let sel_len = selected.len();

        if sel_len > 0 {
            let mut l = DList::new();
            l.push_back(self.quad_outline.clone());
            self.prepare_passes_objects_ortho(l);

            for p in self.passes.values()
            {
                p.draw_frame(
                    self.mesh_manager.clone(),
                    self.material_manager.clone(),
                    self.shader_manager.clone(),
                    self.texture_manager.clone(),
                    self.fbo_manager.clone(),
                    );
            }

            //* TODO dragger
            unsafe { cgl_clear(); }
            //let mut ld = DList::new();
            //ld.push_back(self.dragger.clone());
            //self.prepare_passes_objects_per(ld);
            self.prepare_passes_objects_per(draggers);

            let scale = self.camera.borrow().get_camera_resize_w(0.05f64);
            //add_box(&mut *self.line.write().unwrap(), selected, scale as f32);
            add_box(&mut *self.line.write().unwrap(), draggers, scale as f32);

            prepare_passes_object(
                self.line.clone(),
                &mut self.passes,
                self.material_manager.clone(),
                self.shader_manager.clone(),
                self.camera.clone());

            for p in self.passes.values()
            {
                p.draw_frame(
                    self.mesh_manager.clone(),
                    self.material_manager.clone(),
                    self.shader_manager.clone(),
                    self.texture_manager.clone(),
                    self.fbo_manager.clone(),
                    );
            }
            //*/
        }
    }
}

fn prepare_passes_object(
    o : Arc<RwLock<object::Object>>,
    passes : &mut HashMap<String, Box<RenderPass>>, 
    material_manager : Arc<RwLock<resource::ResourceManager<material::Material>>>,
    shader_manager : Arc<RwLock<resource::ResourceManager<shader::Shader>>>,
    camera : Rc<RefCell<camera::Camera>>
    )
{
    {
        let occ = o.clone();
        for c in occ.read().unwrap().children.iter()
        {
            prepare_passes_object(
                c.clone(),
                passes,
                material_manager.clone(),
                shader_manager.clone(),
                camera.clone());
        }
    }

    {
        let oc = o.clone();
        let mut occ = oc.write().unwrap();
        let ocname = occ.name.clone();
        let render = &mut occ.mesh_render;

        let material = match *render {
            Some(ref mut mr) => { 
                mr.material.get_resource(&mut *material_manager.write().unwrap())
            },
            None => return
        };

        let mat = match material {
            None => return,
            Some(m) => m
        };

        let mmm = &mut mat.write().unwrap().shader;

        let mut shader_yep = match *mmm {
            Some(ref mut s) => s,
            None =>  return
        };

        let shader = match shader_yep.get_resource(&mut *shader_manager.write().unwrap()) {
            Some(s) => s,
            None => return
        };

        {
            let key = shader.read().unwrap().name.clone();
            let rp = match passes.entry(key) {
                Vacant(entry) => 
                    entry.insert(box RenderPass::new(shader.clone(), camera.clone())),
                Occupied(entry) => entry.into_mut(),
            };

            let key_cam = camera.borrow().id.clone();
            let cam_pass = match rp.passes.entry(key_cam) {
                Vacant(entry) => 
                    entry.insert(box CameraPass::new(camera.clone())),
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


fn add_box(
    line : &mut object::Object, 
    objects : &DList<Arc<RwLock<object::Object>>>, 
    scale : f32
    )
{
    let mut m = if let Some(ref mr) = line.mesh_render {
        if let resource::ResTest::ResData(ref mesh_arc) = mr.mesh.resource {
            mesh_arc.write().unwrap()
        }
        else  {
            return;
        }
    }
    else {
        return;
    };

    let color = vec::Vec4::new(0f64,1f64,0f64,0.7f64);

    m.clear_lines();

    for o in objects.iter() {
        let ob = o.read().unwrap();
        line.position = ob.world_position();
        line.orientation = transform::Orientation::Quat(ob.world_orientation());
        //line.scale = ob.world_scale();

        let obm = if let Some(ref mr) = ob.mesh_render {
            if let resource::ResTest::ResData(ref mesh_arc) = mr.mesh.resource {
                mesh_arc.read().unwrap()
            }
            else  {
                break;
            }
        }
        else {
            break;
        };

        let aabox = if let Some(ref m) = obm.aabox {
            m
        }
        else {
            break;
        };

        let scaled_box = aabox * scale;

        m.add_aabox(&scaled_box, color);

        break;
    }
}


