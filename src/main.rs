#![feature(macro_rules)]
extern crate serialize;
extern crate libc;
extern crate sync;
extern crate png;
extern crate toml;

use libc::{c_char, c_void};
use std::mem;
use std::collections::{DList,HashMap};
//use serialize::{json, Encodable, Encoder, Decoder, Decodable};
use sync::{RWLock, Arc};
use std::c_str::CString;
use std::ptr;

#[repr(C)]
pub struct JkList;
#[repr(C)]
pub struct Tree;
#[repr(C)]
pub struct Creator;

#[link(name = "joker")]
extern {
    //fn simple_window_init();
    fn elm_simple_window_main();
    fn jk_list_new() -> *const JkList;
    fn tree_widget_new() -> *const Tree;
    fn tree_register_cb(
        tree : *mut Tree,
        name_get : extern fn(data : *const c_void) -> *const c_char,
        select : extern fn(data : *const c_void) -> (),
        can_expand : extern fn(data : *const c_void) -> bool,
        expand : extern fn(tree: *mut Tree, data : *const c_void, parent: *const c_void) -> (),
        );

    fn tree_object_add(
        tree : *mut Tree,
        object : *const c_void,
        parent : *const c_void,
        );
    fn creator_new() -> *const Creator;
    fn creator_tree_new(creator : *const Creator) -> *mut Tree;
    fn creator_button_new(creator : *const Creator);

    pub fn init_callback_set(
        cb: extern fn(*mut render::Render) -> (),
        render: *const render::Render
        ) -> ();
}

mod resource;
mod shader;
mod mesh;
mod mesh_render;
mod render;
mod object;
mod uniform;
mod matrix;
mod vec;
mod camera;
mod scene;
mod texture;

fn main() {

    unsafe {
    let l = jk_list_new();
    };

    //spawn(proc() {
    /*
       let mut mattt = Arc::new(RWLock::new( shader::Material {
            name : String::from_str("material/my_mat"),
            //shader : Some(shader::Shader::new("shader/simple.sh")),
            shader : Some(resource::ResTT::new("shader/simple.sh")),
            state : 0,
            //texture : Some(t)
            textures : Vec::new(),
            uniforms : HashMap::new(),
        }));
            mattt.write().textures.push(resource::ResTT::new("image/base_skeleton_col.png"));
            mattt.write().uniforms.insert(String::from_str("unitest"), shader::Float(5.6f32));
    //mattt.read().savetoml();
    mattt.read().save();
     */

    /*
    let matoml = r#"name = "material/my_mat"
state = 0
[shader]
name = "shader/simple.sh"
[texture]
name = "image/base_skeleton_col.png"
"#;

    let mattt = shader::Material::new_toml(matoml);
    */


    let mat = Arc::new(RWLock::new(shader::Material::new_from_file("material/simple.mat")));
    mat.read().save();

    //let cam = camera::Camera::new();

    let scene_path = "scene/simple.scene";
    //let mut scene = scene::Scene::new_from_file(scene_path);

    let mut r = box render::Render { 
        passes : HashMap::new(),
        mesh_manager : Arc::new(RWLock::new(resource::ResourceManager::new())),
        shader_manager : Arc::new(RWLock::new(resource::ResourceManager::new())),
        texture_manager : Arc::new(RWLock::new(resource::ResourceManager::new())),
        material_manager : Arc::new(RWLock::new(resource::ResourceManager::new())),
        scene : box scene::Scene::new_from_file(scene_path)
    };

    let oo = r.scene.object_find("yep");
    match oo {
        Some(o) => { println!("I found the obj");
            o.write().child_add(Arc::new(RWLock::new(object::Object::new("my_child"))));
        }
        None => {}
    }


        //r.prepare_passes();

        /*
        for o in scene.objects.iter() {
            r.pass.objects.push((*o).clone());
        }
        */


        /*
        {
            let mut o = object::Object::new("nouvel object");
            o.mesh_render = Some(mesh_render::MeshRender::new("model/skeletonmesh.mesh", "material/simple.mat"));
            //o.mesh_set(mesh::new::from_file("model/skeletonmesh.mesh");
            //o.position = vec::Vec3::new(100f64,0f64,-1000f64);
            let mut oref = Rc::new(RefCell::new(o));
            scene.objects.push(oref.clone());
            //r.pass.objects.push(oref.clone());
        }
        */

        /*
        {
            let mut o = object::Object::new("yep2");
            o.mesh_set(mesh.clone());
            o.position = vec::Vec3::new(-100f64,-100f64,-1000f64);
            o.orientation = vec::Quat::new_axis_angle(vec::Vec3::new(0f64,1f64,0f64), consts::PI/2f64);
            o.scale = vec::Vec3::new(2f64,2f64,2f64);
            let mut oref = Rc::new(RefCell::new(o));
            //scene.objects.push(oref.clone());
            r.pass.objects.push(oref.clone());

            ////println!("{}", json::encode(&*oref.borrow()));
            //let ob_encoded = json::encode(&*oref.borrow());
            //let ob_decoded : object::Object = json::decode(ob_encoded.as_slice()).unwrap();
            //println!(" ob decoded {}", json::encode(&ob_decoded));
            //
        }
        */

        //scene.save();

    unsafe {
        render::draw_callback_set(render::draw_cb, &*r);
        init_callback_set(init_cb, &*r);
    }

    r.init();

    
    unsafe { 
        elm_simple_window_main();
    };
}


pub extern fn name_get(data : *const c_void) -> *const c_char {
    let o = data as *const object::Object;

    unsafe {
        let cs = (*o).name.to_c_str();
        //cs.clone().unwrap()
        cs.unwrap()
    }
}

pub extern fn select(data : *const c_void) -> () {
    let o = data as *const object::Object;
    unsafe {
        println!("select ! {} ", (*o).name);
    }
}

pub extern fn can_expand(data : *const c_void) -> bool {
    let o = data as *const object::Object;
    unsafe {
        println!("can expand :{}", (*o).children.is_empty());
        return !(*o).children.is_empty();
    }
    //return false;
}

pub extern fn expand(tree: *mut Tree, data : *const c_void, parent : *const c_void) -> () {
    let o = data as *const object::Object;

    unsafe {
    println!("expanding ! {} ", (*o).name);

        for c in (*o).children.iter() {
            println!("expanding ! with child {} ", (*c).read().name);
            let oc = c.clone();
            let oo : &object::Object = &*oc.read();
            tree_object_add(tree, mem::transmute(oo), parent);
        }


        println!("expand ! {} ", (*o).name);
    }
}

pub extern fn init_cb(render: *mut render::Render) -> () {
    unsafe {
        let c = creator_new();
        let t = creator_tree_new(c);
        tree_register_cb(
            t,
            name_get,
            select,
            can_expand,
            expand);

        for o in (*render).scene.objects.iter() {
            let oc = o.clone();
            let oo : &object::Object = &*oc.read();
            tree_object_add(t, mem::transmute(oo), ptr::null());
        }

        //creator_button_new(c);
    }
}

