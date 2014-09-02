extern crate serialize;
extern crate libc;
extern crate sync;

use std::collections::{DList};
use std::rc::Rc;
use std::cell::RefCell;
use std::f64::consts;
use serialize::json::ToJson;
use serialize::{json, Encodable, Encoder, Decoder, Decodable};
use std::io::stdio;
use std::io::File;

#[link(name = "joker")]
extern {
    fn simple_window_init();
    fn elm_simple_window_main();
}

mod resource;
mod shader;
mod mesh;
mod render;
mod object;
mod uniform;
mod matrix;
mod vec;
mod camera;
mod scene;

fn main() {

    //spawn(proc() {
    //
        let mut mat = Rc::new(RefCell::new( shader::Material {
            name : String::from_str("my_mat"),
            shader : None,
            state : 0 }));

        let mut cam = camera::Camera::new();

        let scene_path = "scene/simple.scene";
        let file = File::open(&Path::new(scene_path)).read_to_string().unwrap();
        let mut scene : scene::Scene = json::decode(file.as_slice()).unwrap();
        //let mut scene = scene::Scene::new("the_scene");

        let mut r = box render::Render { 
            pass :box render::RenderPass{
                      name : String::from_str("passtest"),
                      material : mat.clone(),
                      objects : DList::new(),
                      camera : Rc::new(RefCell::new(cam)),
                      resource_manager : Rc::new(RefCell::new(resource::ResourceManager::new()))
                  },
            request_manager : box render::RequestManager {
                                  requests : DList::new(),
                                  requests_material : DList::new()
                              }
        };

        r.request_manager.requests_material.push(
            box render::Request { resource : mat.clone() });

        for o in scene.objects.iter() {
            r.pass.objects.push((*o).clone());
        }


        /*
        {
            let mut o = object::Object::new("yep");
            o.mesh_set(mesh.clone());
            o.position = vec::Vec3::new(100f64,0f64,-1000f64);
            let mut oref = Rc::new(RefCell::new(o));
            //scene.objects.push(oref.clone());
            r.pass.objects.push(oref.clone());
            
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

        /*
        {
            let mut file = File::create(&Path::new(scene_path));
            let mut stdwriter = stdio::stdout();
            //let mut encoder = json::Encoder::new(&mut stdwriter);
            //let mut encoder = json::PrettyEncoder::new(&mut stdwriter);
            let mut encoder = json::PrettyEncoder::new(&mut file);
            //let mut encoder = json::Encoder::new(&mut file);

            //println!("scene : \n\n {}", json::encode(&scene));

            scene.encode(&mut encoder).unwrap();
        }
        */

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

