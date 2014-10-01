#![feature(macro_rules)]
extern crate serialize;
extern crate libc;
extern crate sync;
extern crate png;
extern crate toml;

use std::collections::{DList,HashMap};
use serialize::{json, Encodable, Encoder, Decoder, Decodable};
use std::io::stdio;
use std::io::File;
use sync::{RWLock, Arc};

#[link(name = "joker")]
extern {
    fn simple_window_init();
    fn elm_simple_window_main();
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
        /*
        pass :box render::RenderPass{
            name : String::from_str("passtest"),
            material : mat.clone(),
            objects : DList::new(),
            camera : Rc::new(RefCell::new(cam)),
        },
        */
        request_manager : box render::RequestManager {
                              requests : DList::new(),
                              requests_material : DList::new()
                          },

        mesh_manager : Arc::new(RWLock::new(resource::ResourceManager::new())),
        shader_manager : Arc::new(RWLock::new(resource::ResourceManager::new())),
        texture_manager : Arc::new(RWLock::new(resource::ResourceManager::new())),
        material_manager : Arc::new(RWLock::new(resource::ResourceManager::new())),
        scene : box scene::Scene::new_from_file(scene_path)
    };

        /*
        r.request_manager.requests_material.push(
            box render::Request { resource : mat.clone() });
            */

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
        }

        r.init();
      //  r.draw();

    //});
    
    unsafe { 
        elm_simple_window_main();
    };
}

