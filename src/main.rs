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

fn main() {
    unsafe {
        simple_window_init();
    };

    //spawn(proc() {

        let mut r = box render::Render { 
            pass :box render::RenderPass{
                      name : String::from_str("passtest"),
                      material : box shader::Material { name : String::from_str("nouveau"), shader : None, state : 0 },
                      objects : DList::new(),
                  },
            request_manager : box render::RequestManager {
                                  requests : DList::new()
                              }
        };

        let mut mesh = Rc::new(RefCell::new( mesh::Mesh {
            name : String::from_str("mesh_name"),
            buffer : None,
            state : 0 }));

        r.request_manager.requests.push(
            box render::Request { resource : mesh.clone() });

        r.pass.objects.push(object::Object{name : String::from_str("objectyep"), mesh : Some(mesh.clone())}); 

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

