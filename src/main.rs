#![feature(macro_rules)]
#![feature(log_syntax)]
#![feature(trace_macros)]
#![feature(slicing_syntax)]

//TODO remove
#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(unused_imports)]
//#![allow(improper_ctypes)]
//#![allow(ctypes)]

#![feature(globs)]
extern crate serialize;
extern crate libc;
extern crate sync;
extern crate png;
extern crate toml;
//extern crate debug;
extern crate uuid;

//use serialize::{json, Encodable, Encoder, Decoder, Decodable};
//use std::collections::HashMap;
//use sync::{RWLock, Arc};
//use std::rc::Rc;
//use std::cell::RefCell;
use std::mem;
//use property::ChrisProperty;
//use std::any::{Any, AnyRefExt};

mod resource;
mod shader;
mod material;
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
mod ui;
mod property;
mod geometry;
mod intersection;
mod fbo;
mod factory;
mod operation;

mod context;
mod control;

mod transform;

fn main() {
    /*
    let mut c = property::Chris::new();
    //let yep  = c.cget_property("x");
    c.cset_property_hier(
        [String::from_str("boxpos"),String::from_str("x")].to_vec(),
        &555f64);
    c.cset_property_hier(
        [String::from_str("boxpos")].to_vec(),
        &vec::Vec3::new(111f64,-9f64,-33f64));
    let yep  = c.cget_property_hier(
        [String::from_str("boxpos"),String::from_str("x")].to_vec());
    match yep {
        property::BoxAny(v) => {
            match v.downcast_ref::<f64>() {
                Some(vv) => println!("I found the value {}", *vv ),
                None => {}
            }
        },
        _ => {}
    }
    */


    //spawn(proc() {
    /*
       let mut mattt = Arc::new(RWLock::new( material::Material {
            name : String::from_str("material/my_mat"),
            //shader : Some(shader::Shader::new("shader/simple.sh")),
            shader : Some(resource::ResTT::new("shader/simple.sh")),
            state : 0,
            //texture : Some(t)
            textures : HashMap::new(),
            uniforms : HashMap::new(),
        }));
       //mattt.write().textures.insert(String::from_str("texnametest"), material::SamplerImageFile(resource::ResTT::new("image/base_skeleton_col.png")));
       mattt.write().textures.insert(String::from_str("texnametest"), material::SamplerFbo(resource::ResTT::new("fbo_all")));
       mattt.write().uniforms.insert(String::from_str("unitest"), box shader::Vec2(vec::Vec2::new(5.6f64, 1.2f64)));
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

    //let mat = Arc::new(RWLock::new(shader::Material::new_from_file("material/simple.mat")));
    //mat.read().save();

    //let cam = camera::Camera::new();

    /*
    let oo = r.scene.read().object_find("yep");
    match oo {
        Some(o) => { println!("I found the obj");
            o.write().child_add(Arc::new(RWLock::new(object::Object::new("my_child"))));
            println!("yooooooooooooooooooooooooooooooooo");
            property::print_pt(o.read().get_property("name"));
        }
        None => {}
    }
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

    //let m = box ui::Master::new();
    //let m = Rc::new(RefCell::new(ui::Master::new()));
    let m = ui::Master::new();

    unsafe {
        //render::draw_callback_set(render::init_cb, render::draw_cb, render::resize_cb, &m.render);
        //ui::init_callback_set(ui::init_cb, &*m);
        render::draw_callback_set(
            render::init_cb,
            render::draw_cb,
            render::resize_cb,
            //&m.render);
            &m.borrow().render);

        ui::init_callback_set(
            ui::init_cb,
            mem::transmute(box m.clone()));
    }

    unsafe { 
        ui::elm_simple_window_main();
    };

    //m.borrow().render.scene.read().save();
}


