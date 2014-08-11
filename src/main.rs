use std::collections::{DList,Deque};
use std::mem;

#[link(name = "joker")]
extern {
    fn simple_window_init();
    fn elm_simple_window_main();
}

mod shader;
mod mesh;
mod render;
mod object;

static vertex_data_test: [f32, ..6] = [
    0.0,  0.5,
    0.5, -0.5,
    -0.5, -0.5
      ];

fn main() {
    unsafe {
        simple_window_init();
    };

    /*
    spawn(proc() {
        shader::shader_init();
        println!("its been called, its over right");
    });
    */

    /*
    spawn(proc() {
        mesh::mesh_init();
    });
    */

    spawn(proc() {

        let mut r = box render::Render { 
            pass :box render::RenderPass{
                      name : String::from_str("passtest"),
                      material : box shader::Material { name : String::from_str("nouveau"), shader : None, state : 0 },
                      objects : DList::new(),
                      drawables : DList::new(),
                  } };

        let mut mesh = box mesh::Mesh { name : String::from_str("mesh_name"), buffer : None, state : 0 };
        unsafe {
            mesh::buffer_request_add(&mut *mesh, mem::transmute(&vertex_data_test[0]), 6, mesh::mesh_cb_result);
        }

        r.pass.objects.push(object::Object{name : String::from_str("objectyep"), mesh : Some(*mesh)}); 

        //println!("my mesh : {}", (*mesh).name);
        r.init();
        r.draw();
    });

    unsafe { 
        elm_simple_window_main();
    };
}

