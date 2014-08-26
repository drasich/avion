use std::collections::{DList};
use std::rc::Rc;
use std::cell::RefCell;

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

        let mut r = box render::Render { 
            pass :box render::RenderPass{
                      name : String::from_str("passtest"),
                      material : mat.clone(),
                      objects : DList::new(),
                  },
            request_manager : box render::RequestManager {
                                  requests : DList::new(),
                                  requests_material : DList::new()
                              }
        };

        r.request_manager.requests_material.push(
            box render::Request { resource : mat.clone() });

        let mut mesh = Rc::new(RefCell::new( mesh::Mesh::new_from_file("model/skeletonmesh.mesh")));

        r.request_manager.requests.push(
            box render::Request { resource : mesh.clone() });

        let mut o = object::Object::new("yep");
        o.mesh_set(mesh.clone());
        o.position = vec::Vec3::new(10f64,0f64,0f64);
        o.position.x = 7f64;
        r.pass.objects.push(Rc::new(RefCell::new(o)));

        let mut cam = camera::Camera::new();
        let mut scene = scene::Scene::new("the_scene");

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

