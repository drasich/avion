use std::collections::{DList,Deque};

#[link(name = "joker")]
extern {
    fn simple_window_init();
    fn elm_simple_window_main();
}

mod shader;
mod mesh;
mod render;
mod object;

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

    spawn(proc() {
        mesh::mesh_init();
    });

    spawn(proc() {

        let mut r = box render::Render { 
            pass :box render::RenderPass{
                      name : String::from_str("passtest"),
                      material : box shader::Material { name : String::from_str("nouveau"), shader : None, state : 0 },
                      objects : DList::new()
                  } };

        r.pass.objects.push(object::Object{name : String::from_str("objectyep"), mesh : None}); 

        r.init();
        r.draw();
    });

    unsafe { 
        elm_simple_window_main();
    };
}

